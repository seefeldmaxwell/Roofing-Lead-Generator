use chrono::NaiveDateTime;
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};
use tracing::warn;

use crate::models::*;
use crate::services::fema_client::FemaApiClient;
use crate::services::florida_fire_client::FloridaFireDataClient;
use crate::services::florida_property_client::FloridaPropertyDataClient;

/// Supported FDOT counties used to supplement the database list.
const SUPPORTED_COUNTIES: &[&str] = &[
    "Alachua", "Baker", "Bay", "Bradford", "Brevard", "Broward", "Calhoun",
    "Charlotte", "Citrus", "Clay", "Collier", "Columbia", "DeSoto",
    "Dixie", "Duval", "Escambia", "Flagler", "Franklin", "Gadsden",
    "Gilchrist", "Glades", "Gulf", "Hamilton", "Hardee", "Hendry",
    "Hernando", "Highlands", "Hillsborough", "Holmes", "Indian River",
    "Jackson", "Jefferson", "Lafayette", "Lake", "Lee", "Leon", "Levy",
    "Liberty", "Madison", "Manatee", "Marion", "Martin", "Miami-Dade",
    "Monroe", "Nassau", "Okaloosa", "Okeechobee", "Orange", "Osceola",
    "Palm Beach", "Pasco", "Pinellas", "Polk", "Putnam", "Santa Rosa",
    "Sarasota", "Seminole", "St. Johns", "St. Lucie", "Sumter",
    "Suwannee", "Taylor", "Union", "Volusia", "Wakulla", "Walton",
    "Washington",
];

// ── Bind-value helper for dynamic queries ───────────────────────────────────

/// Helper enum so we can build dynamic WHERE clauses with heterogeneous
/// parameter types and bind them in order.
enum BindValue {
    String(String),
    Int(i64),
    Bool(bool),
}

// ── PropertyService ─────────────────────────────────────────────────────────

/// Core business-logic service for property search, detail retrieval,
/// caching from live APIs, and dashboard statistics.
pub struct PropertyService {
    pool: SqlitePool,
    property_client: FloridaPropertyDataClient,
    fema_client: FemaApiClient,
    fire_client: FloridaFireDataClient,
}

impl PropertyService {
    /// Build a new `PropertyService` with all required dependencies.
    pub fn new(
        pool: SqlitePool,
        property_client: FloridaPropertyDataClient,
        fema_client: FemaApiClient,
        fire_client: FloridaFireDataClient,
    ) -> Self {
        Self {
            pool,
            property_client,
            fema_client,
            fire_client,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Public: search by address
    // ═══════════════════════════════════════════════════════════════════════

    /// Search properties by address and various filters.  When the local
    /// database has no matches and a query string was supplied the service
    /// attempts to fetch from the live Florida property API, cache locally,
    /// and re-query.
    pub async fn search_by_address(
        &self,
        req: &SearchRequest,
    ) -> PaginatedResult<SearchResult> {
        let page = req.page.unwrap_or(1).max(1);
        let page_size = req.page_size.unwrap_or(25).clamp(1, 100);
        let offset = (page - 1) * page_size;

        // -- dynamic WHERE clause ------------------------------------------------
        let mut conditions: Vec<String> = Vec::new();
        let mut bind_values: Vec<BindValue> = Vec::new();

        if let Some(ref q) = req.query {
            if !q.is_empty() {
                conditions.push("p.address LIKE ?".into());
                bind_values.push(BindValue::String(format!("%{}%", q)));
            }
        }
        Self::push_common_filters(req, &mut conditions, &mut bind_values);

        let where_sql = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // -- count ---------------------------------------------------------------
        let count_sql = format!(
            "SELECT COUNT(*) AS cnt FROM properties p \
             LEFT JOIN owners o ON p.owner_id = o.id {}",
            where_sql
        );

        let mut total_count = self.execute_count(&count_sql, &bind_values).await;

        // If nothing in DB and a query was given, try live API then re-count.
        if total_count == 0 {
            if let Some(ref q) = req.query {
                if !q.is_empty() {
                    self.fetch_and_cache_from_live_api(q, req.county.as_deref())
                        .await;
                    total_count = self.execute_count(&count_sql, &bind_values).await;
                }
            }
        }

        if total_count == 0 {
            return PaginatedResult {
                items: Vec::new(),
                total_count: 0,
                page,
                page_size,
            };
        }

        // -- fetch page ----------------------------------------------------------
        let select_sql = format!(
            "SELECT p.id AS property_id, p.address, p.city, p.county, p.zip_code, \
             p.parcel_id, p.property_type, p.year_built, p.roof_type, p.roof_age, \
             p.estimated_value, p.flood_zone, p.is_in_flood_zone, \
             p.is_in_high_risk_flood_zone, p.fire_risk_level, \
             p.overall_risk_score, p.lead_priority, \
             COALESCE(o.first_name || ' ' || o.last_name, '') AS owner_name, \
             (SELECT COUNT(*) FROM permits  WHERE property_id = p.id) AS permit_count, \
             (SELECT MAX(issued_date) FROM permits WHERE property_id = p.id) AS last_permit_date, \
             (SELECT COUNT(*) FROM insurance_claims WHERE property_id = p.id) AS claim_count \
             FROM properties p \
             LEFT JOIN owners o ON p.owner_id = o.id \
             {} ORDER BY p.id DESC LIMIT ? OFFSET ?",
            where_sql
        );

        let items = self
            .fetch_search_results(&select_sql, &bind_values, page_size, offset)
            .await;

        PaginatedResult {
            items,
            total_count,
            page,
            page_size,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Public: search by owner
    // ═══════════════════════════════════════════════════════════════════════

    /// Search properties by owner first/last name.  Falls back to the live
    /// API when no local matches are found.
    pub async fn search_by_owner(
        &self,
        req: &SearchRequest,
    ) -> PaginatedResult<SearchResult> {
        let page = req.page.unwrap_or(1).max(1);
        let page_size = req.page_size.unwrap_or(25).clamp(1, 100);
        let offset = (page - 1) * page_size;

        let mut conditions: Vec<String> = Vec::new();
        let mut bind_values: Vec<BindValue> = Vec::new();

        if let Some(ref q) = req.query {
            if !q.is_empty() {
                conditions.push(
                    "(o.first_name LIKE ? OR o.last_name LIKE ? \
                     OR (o.first_name || ' ' || o.last_name) LIKE ?)"
                        .into(),
                );
                let pat = format!("%{}%", q);
                bind_values.push(BindValue::String(pat.clone()));
                bind_values.push(BindValue::String(pat.clone()));
                bind_values.push(BindValue::String(pat));
            }
        }
        Self::push_common_filters(req, &mut conditions, &mut bind_values);

        let where_sql = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_sql = format!(
            "SELECT COUNT(*) AS cnt FROM properties p \
             LEFT JOIN owners o ON p.owner_id = o.id {}",
            where_sql
        );

        let mut total_count = self.execute_count(&count_sql, &bind_values).await;

        // Fallback to live API
        if total_count == 0 {
            if let Some(ref q) = req.query {
                if !q.is_empty() {
                    self.fetch_and_cache_owner_from_live_api(q, req.county.as_deref())
                        .await;
                    total_count = self.execute_count(&count_sql, &bind_values).await;
                }
            }
        }

        if total_count == 0 {
            return PaginatedResult {
                items: Vec::new(),
                total_count: 0,
                page,
                page_size,
            };
        }

        let select_sql = format!(
            "SELECT p.id AS property_id, p.address, p.city, p.county, p.zip_code, \
             p.parcel_id, p.property_type, p.year_built, p.roof_type, p.roof_age, \
             p.estimated_value, p.flood_zone, p.is_in_flood_zone, \
             p.is_in_high_risk_flood_zone, p.fire_risk_level, \
             p.overall_risk_score, p.lead_priority, \
             COALESCE(o.first_name || ' ' || o.last_name, '') AS owner_name, \
             (SELECT COUNT(*) FROM permits  WHERE property_id = p.id) AS permit_count, \
             (SELECT MAX(issued_date) FROM permits WHERE property_id = p.id) AS last_permit_date, \
             (SELECT COUNT(*) FROM insurance_claims WHERE property_id = p.id) AS claim_count \
             FROM properties p \
             LEFT JOIN owners o ON p.owner_id = o.id \
             {} ORDER BY p.id DESC LIMIT ? OFFSET ?",
            where_sql
        );

        let items = self
            .fetch_search_results(&select_sql, &bind_values, page_size, offset)
            .await;

        PaginatedResult {
            items,
            total_count,
            page,
            page_size,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Public: get property by id
    // ═══════════════════════════════════════════════════════════════════════

    /// Load a single property by database id.  Also loads the owner, permits,
    /// claims, and fire incidents via separate queries.  When flood data or
    /// permits are missing the service will attempt to enrich from live APIs.
    ///
    /// Note: Because `Property` derives `sqlx::FromRow` and does not carry
    /// inline relations, the associated owner / permits / claims / fire
    /// incidents are returned in a tuple alongside the property.
    pub async fn get_property_by_id(
        &self,
        id: i64,
    ) -> Option<(Property, Option<Owner>, Vec<Permit>, Vec<InsuranceClaim>, Vec<FireIncident>)>
    {
        // ── base property row ──
        let mut property: Property = sqlx::query_as::<_, Property>(
            "SELECT * FROM properties WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .ok()??;

        // ── owner ──
        let owner: Option<Owner> = if let Some(owner_id) = property.owner_id {
            sqlx::query_as::<_, Owner>("SELECT * FROM owners WHERE id = ?")
                .bind(owner_id)
                .fetch_optional(&self.pool)
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        // ── permits ──
        let mut permits = self.load_permits_for(id).await;

        // ── insurance claims ──
        let claims: Vec<InsuranceClaim> = sqlx::query_as::<_, InsuranceClaim>(
            "SELECT * FROM insurance_claims WHERE property_id = ? ORDER BY date_of_loss DESC",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        // ── fire incidents ──
        let fire_incidents: Vec<FireIncident> = sqlx::query_as::<_, FireIncident>(
            "SELECT * FROM fire_incidents WHERE property_id = ? ORDER BY discovery_date DESC",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        // ── enrich: flood data ──
        if property.flood_zone.is_none() {
            if let (Some(lat), Some(lon)) = (property.latitude, property.longitude) {
                self.enrich_with_flood_data(property.id, lat, lon).await;

                // Re-read just the flood columns.
                if let Ok(Some(upd)) = sqlx::query(
                    "SELECT flood_zone, flood_zone_description, is_in_flood_zone, \
                     is_in_high_risk_flood_zone, fema_map_panel, requires_flood_insurance \
                     FROM properties WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                {
                    property.flood_zone = upd.get("flood_zone");
                    property.flood_zone_description = upd.get("flood_zone_description");
                    property.is_in_flood_zone =
                        upd.get::<bool, _>("is_in_flood_zone");
                    property.is_in_high_risk_flood_zone =
                        upd.get::<bool, _>("is_in_high_risk_flood_zone");
                    property.fema_map_panel = upd.get("fema_map_panel");
                    property.requires_flood_insurance =
                        upd.get::<bool, _>("requires_flood_insurance");
                }
            }
        }

        // ── enrich: permits ──
        if permits.is_empty() {
            self.fetch_and_cache_permits(
                property.id,
                &property.address,
                &property.county,
            )
            .await;
            permits = self.load_permits_for(id).await;
        }

        Some((property, owner, permits, claims, fire_incidents))
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Public: dashboard stats
    // ═══════════════════════════════════════════════════════════════════════

    /// Aggregate counts and breakdowns for the dashboard view.
    pub async fn get_dashboard_stats(&self) -> DashboardStats {
        let total_properties = self
            .scalar_count("SELECT COUNT(*) AS cnt FROM properties")
            .await;
        let total_permits = self
            .scalar_count("SELECT COUNT(*) AS cnt FROM permits")
            .await;
        let total_claims = self
            .scalar_count("SELECT COUNT(*) AS cnt FROM insurance_claims")
            .await;
        let total_disasters = self
            .scalar_count("SELECT COUNT(*) AS cnt FROM fema_disasters")
            .await;
        let properties_in_flood_zone = self
            .scalar_count(
                "SELECT COUNT(*) AS cnt FROM properties WHERE is_in_flood_zone = 1",
            )
            .await;
        let properties_high_fire_risk = self
            .scalar_count(
                "SELECT COUNT(*) AS cnt FROM properties \
                 WHERE fire_risk_level IS NOT NULL AND fire_risk_level IN ('High', 'Extreme')",
            )
            .await;

        let average_roof_age: f64 = sqlx::query(
            "SELECT COALESCE(AVG(roof_age), 0.0) AS val FROM properties WHERE roof_age IS NOT NULL",
        )
        .fetch_one(&self.pool)
        .await
        .map(|r| r.get::<f64, _>("val"))
        .unwrap_or(0.0);

        let high_priority_leads = self
            .scalar_count(
                "SELECT COUNT(*) AS cnt FROM properties WHERE lead_priority = 'High'",
            )
            .await;
        let medium_priority_leads = self
            .scalar_count(
                "SELECT COUNT(*) AS cnt FROM properties WHERE lead_priority = 'Medium'",
            )
            .await;
        let low_priority_leads = self
            .scalar_count(
                "SELECT COUNT(*) AS cnt FROM properties WHERE lead_priority = 'Low'",
            )
            .await;

        // ── county breakdowns ──
        let county_rows = sqlx::query(
            "SELECT p.county, \
                    COUNT(p.id) AS property_count, \
                    (SELECT COUNT(*) FROM permits pm \
                     WHERE pm.property_id IN \
                        (SELECT id FROM properties WHERE county = p.county)) AS permit_count, \
                    AVG(p.roof_age) AS average_roof_age, \
                    SUM(CASE WHEN p.is_in_flood_zone = 1 THEN 1 ELSE 0 END) AS flood_zone_count, \
                    SUM(CASE WHEN p.fire_risk_level IN ('High', 'Extreme') THEN 1 ELSE 0 END) \
                        AS high_fire_risk_count \
             FROM properties p \
             WHERE p.county != '' \
             GROUP BY p.county \
             ORDER BY property_count DESC",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let county_breakdown: Vec<CountyBreakdown> = county_rows
            .iter()
            .map(|row| CountyBreakdown {
                county: row.get::<String, _>("county"),
                property_count: row.get::<i64, _>("property_count"),
                permit_count: row.get::<i64, _>("permit_count"),
                average_roof_age: row.get::<Option<f64>, _>("average_roof_age"),
                flood_zone_count: row.get::<i64, _>("flood_zone_count"),
                high_fire_risk_count: row.get::<i64, _>("high_fire_risk_count"),
            })
            .collect();

        // ── roof age distribution ──
        let roof_age_distribution = self.build_roof_age_distribution().await;

        DashboardStats {
            total_properties,
            total_permits,
            total_claims,
            total_disasters,
            properties_in_flood_zone,
            properties_high_fire_risk,
            average_roof_age,
            high_priority_leads,
            medium_priority_leads,
            low_priority_leads,
            county_breakdown,
            roof_age_distribution,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Public: counties, cities, recent disasters
    // ═══════════════════════════════════════════════════════════════════════

    /// Return a sorted, de-duplicated list of counties drawn from both the
    /// local database and the full set of supported FDOT counties.
    pub async fn get_counties(&self) -> Vec<String> {
        let rows = sqlx::query(
            "SELECT DISTINCT county FROM properties WHERE county != '' ORDER BY county",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let mut counties: Vec<String> = rows
            .iter()
            .map(|r| r.get::<String, _>("county"))
            .collect();

        // Merge supported counties not yet in the list.
        for &c in SUPPORTED_COUNTIES {
            let s = c.to_string();
            if !counties.iter().any(|existing| existing.eq_ignore_ascii_case(&s)) {
                counties.push(s);
            }
        }

        counties.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        counties.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        counties
    }

    /// Distinct cities from the database, optionally filtered by county.
    pub async fn get_cities(&self, county: Option<&str>) -> Vec<String> {
        let rows = if let Some(c) = county {
            sqlx::query(
                "SELECT DISTINCT city FROM properties \
                 WHERE city != '' AND county = ? ORDER BY city",
            )
            .bind(c)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default()
        } else {
            sqlx::query(
                "SELECT DISTINCT city FROM properties WHERE city != '' ORDER BY city",
            )
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default()
        };

        rows.iter().map(|r| r.get::<String, _>("city")).collect()
    }

    /// The most recent FEMA disaster declarations stored locally.
    pub async fn get_recent_disasters(&self, count: i32) -> Vec<FemaDisaster> {
        sqlx::query_as::<_, FemaDisaster>(
            "SELECT * FROM fema_disasters ORDER BY declaration_date DESC LIMIT ?",
        )
        .bind(count)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Private: live-API fetch & cache
    // ═══════════════════════════════════════════════════════════════════════

    /// Search the live Florida property data API by address, create Owner +
    /// Property rows in the local database, and compute wildfire risk for
    /// each cached result.
    async fn fetch_and_cache_from_live_api(&self, query: &str, county: Option<&str>) {
        let records = self
            .property_client
            .search_florida_properties(query, county)
            .await;

        if records.is_empty() {
            warn!(
                "Live API returned no results for query='{}' county='{:?}'",
                query, county
            );
            return;
        }

        for record in &records {
            self.cache_property_record(record).await;
        }
    }

    /// Fetch property records by owner name from the live API and cache locally.
    async fn fetch_and_cache_owner_from_live_api(
        &self,
        owner_name: &str,
        county: Option<&str>,
    ) {
        let county_str = match county {
            Some(c) if !c.is_empty() => c,
            _ => {
                warn!(
                    "Cannot search by owner without a county (owner='{}')",
                    owner_name
                );
                return;
            }
        };

        let records = self
            .property_client
            .search_by_owner_name(owner_name, county_str, 25)
            .await;

        if records.is_empty() {
            warn!(
                "Live API returned no results for owner='{}' county='{}'",
                owner_name, county_str
            );
            return;
        }

        for record in &records {
            self.cache_property_record(record).await;
        }
    }

    /// Shared logic to insert a `PropertyRecord` (and its owner) into the DB.
    async fn cache_property_record(&self, record: &PropertyRecord) {
        let owner_id = self.create_or_get_owner(record).await;

        let current_year = chrono::Utc::now()
            .format("%Y")
            .to_string()
            .parse::<i64>()
            .unwrap_or(2026);

        let roof_age: Option<i64> = record
            .year_built
            .filter(|&yb| yb > 0)
            .map(|yb| current_year - yb);

        let county = record.county.as_deref().unwrap_or("");
        let lat = record.latitude.unwrap_or(0.0);
        let lon = record.longitude.unwrap_or(0.0);

        let wildfire_score = self
            .fire_client
            .calculate_wildfire_risk_score(lat, lon, county);

        let fire_risk_level = match wildfire_score {
            s if s >= 80 => "Extreme",
            s if s >= 60 => "High",
            s if s >= 40 => "Moderate",
            s if s >= 20 => "Low",
            _ => "Minimal",
        };

        let result = sqlx::query(
            "INSERT INTO properties \
             (address, city, zip_code, county, state, parcel_id, year_built, \
              square_footage, lot_size, property_type, estimated_value, \
              bedrooms, bathrooms, latitude, longitude, roof_age, \
              wildfire_risk_score, fire_risk_level, is_in_wildfire_zone, \
              data_source, last_data_refresh, owner_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(record.address.as_deref().unwrap_or(""))
        .bind(record.city.as_deref().unwrap_or(""))
        .bind(record.zip_code.as_deref().unwrap_or(""))
        .bind(county)
        .bind(record.state.as_deref().unwrap_or("FL"))
        .bind(record.parcel_id.as_deref().unwrap_or(""))
        .bind(record.year_built)
        .bind(record.square_footage)
        .bind(record.lot_size)
        .bind(record.property_type.as_deref().unwrap_or(""))
        .bind(record.just_value.or(record.assessed_value))
        .bind(record.bedrooms)
        .bind(record.bathrooms)
        .bind(record.latitude)
        .bind(record.longitude)
        .bind(roof_age)
        .bind(wildfire_score as i64)
        .bind(fire_risk_level)
        .bind(wildfire_score >= 40)
        .bind(record.data_source.as_deref().unwrap_or("Florida Property API"))
        .bind(chrono::Utc::now().naive_utc())
        .bind(owner_id)
        .execute(&self.pool)
        .await;

        if let Err(e) = result {
            warn!(
                "Failed to insert property '{}': {}",
                record.address.as_deref().unwrap_or("?"),
                e
            );
        }
    }

    /// Find an existing owner by first + last name, or create a new row.
    /// Returns `None` when the name is entirely empty.
    async fn create_or_get_owner(&self, record: &PropertyRecord) -> Option<i64> {
        let raw_name = record.owner_name.as_deref().unwrap_or("");
        let (first, last) = Self::parse_owner_name(raw_name);
        if first.is_empty() && last.is_empty() {
            return None;
        }

        // Try to find an existing row.
        let existing = sqlx::query(
            "SELECT id FROM owners WHERE first_name = ? AND last_name = ? LIMIT 1",
        )
        .bind(&first)
        .bind(&last)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        if let Some(row) = existing {
            return Some(row.get::<i64, _>("id"));
        }

        // Create new.
        let ins = sqlx::query(
            "INSERT INTO owners (first_name, last_name, mailing_address, mailing_city, \
             mailing_state, mailing_zip) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&first)
        .bind(&last)
        .bind(record.owner_address.as_deref())
        .bind(record.owner_city.as_deref())
        .bind(record.owner_state.as_deref())
        .bind(record.owner_zip.as_deref())
        .execute(&self.pool)
        .await
        .ok()?;

        Some(ins.last_insert_rowid())
    }

    /// Call the FEMA flood zone API and update the property record.
    async fn enrich_with_flood_data(&self, property_id: i64, lat: f64, lon: f64) {
        match self
            .fema_client
            .get_flood_zone_by_coordinates(lat, lon)
            .await
        {
            Ok(info) => {
                let is_flood = info.flood_zone != "X" && info.flood_zone != "C";
                let is_high_risk = info.is_high_risk;
                let requires_ins = info.requires_insurance;

                let result = sqlx::query(
                    "UPDATE properties SET \
                     flood_zone = ?, flood_zone_description = ?, \
                     is_in_flood_zone = ?, is_in_high_risk_flood_zone = ?, \
                     fema_map_panel = ?, requires_flood_insurance = ?, \
                     flood_zone_last_updated = ? \
                     WHERE id = ?",
                )
                .bind(&info.flood_zone)
                .bind(&info.zone_description)
                .bind(is_flood)
                .bind(is_high_risk)
                .bind(&info.map_panel)
                .bind(requires_ins)
                .bind(chrono::Utc::now().naive_utc())
                .bind(property_id)
                .execute(&self.pool)
                .await;

                if let Err(e) = result {
                    warn!(
                        "Failed to update flood data for property {}: {}",
                        property_id, e
                    );
                }
            }
            Err(e) => {
                warn!(
                    "Failed to fetch flood zone for property {} (lat={}, lon={}): {}",
                    property_id, lat, lon, e
                );
            }
        }
    }

    /// Fetch building permits for a property from the appropriate county API,
    /// classify each one, and insert into the local database.
    async fn fetch_and_cache_permits(
        &self,
        property_id: i64,
        address: &str,
        county: &str,
    ) {
        // Currently only Miami-Dade exposes an open data permits endpoint.
        let permit_records = if county.eq_ignore_ascii_case("Miami-Dade") {
            self.property_client
                .get_miami_dade_permits(address, 50)
                .await
        } else {
            Vec::new()
        };

        for pr in &permit_records {
            let ptype = pr.permit_type.as_deref().unwrap_or("");
            let desc = pr.description.as_deref().unwrap_or("");
            let (work_category, is_roofing) = classify_permit(ptype, desc);

            let result = sqlx::query(
                "INSERT INTO permits \
                 (permit_number, permit_type, work_category, description, \
                  issued_date, completed_date, status, contractor_name, \
                  contractor_license, estimated_cost, is_roofing_permit, \
                  data_source, property_id) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(pr.permit_number.as_deref().unwrap_or(""))
            .bind(ptype)
            .bind(&work_category)
            .bind(desc)
            .bind(&pr.issued_date)
            .bind::<Option<NaiveDateTime>>(None) // completed_date not in API
            .bind(pr.status.as_deref().unwrap_or(""))
            .bind(pr.contractor_name.as_deref())
            .bind(pr.contractor_license.as_deref())
            .bind(pr.estimated_cost)
            .bind(is_roofing)
            .bind("Miami-Dade Open Data")
            .bind(property_id)
            .execute(&self.pool)
            .await;

            if let Err(e) = result {
                warn!(
                    "Failed to insert permit '{}' for property {}: {}",
                    pr.permit_number.as_deref().unwrap_or("?"),
                    property_id,
                    e
                );
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Private: query helpers
    // ═══════════════════════════════════════════════════════════════════════

    /// Append the common search-filter clauses that are shared between the
    /// address and owner search paths.
    fn push_common_filters(
        req: &SearchRequest,
        conditions: &mut Vec<String>,
        binds: &mut Vec<BindValue>,
    ) {
        if let Some(ref city) = req.city {
            if !city.is_empty() {
                conditions.push("p.city = ?".into());
                binds.push(BindValue::String(city.clone()));
            }
        }
        if let Some(ref county) = req.county {
            if !county.is_empty() {
                conditions.push("p.county = ?".into());
                binds.push(BindValue::String(county.clone()));
            }
        }
        if let Some(ref zip) = req.zip_code {
            if !zip.is_empty() {
                conditions.push("p.zip_code = ?".into());
                binds.push(BindValue::String(zip.clone()));
            }
        }
        if let Some(min) = req.min_roof_age {
            conditions.push("p.roof_age >= ?".into());
            binds.push(BindValue::Int(min));
        }
        if let Some(max) = req.max_roof_age {
            conditions.push("p.roof_age <= ?".into());
            binds.push(BindValue::Int(max));
        }
        if let Some(in_flood) = req.in_flood_zone {
            conditions.push("p.is_in_flood_zone = ?".into());
            binds.push(BindValue::Bool(in_flood));
        }
        if let Some(high_fire) = req.high_fire_risk {
            if high_fire {
                conditions.push(
                    "p.fire_risk_level IN ('High', 'Extreme')".into(),
                );
                // no bind needed -- literal
            }
        }
        if let Some(ref lp) = req.lead_priority {
            if !lp.is_empty() {
                conditions.push("p.lead_priority = ?".into());
                binds.push(BindValue::String(lp.clone()));
            }
        }
    }

    /// Execute a COUNT query that uses the dynamic `BindValue` vec.
    async fn execute_count(&self, sql: &str, binds: &[BindValue]) -> i64 {
        let mut query = sqlx::query(sql);
        for v in binds {
            query = match v {
                BindValue::String(s) => query.bind(s.clone()),
                BindValue::Int(i) => query.bind(*i),
                BindValue::Bool(b) => query.bind(*b),
            };
        }
        match query.fetch_one(&self.pool).await {
            Ok(row) => row.get::<i64, _>("cnt"),
            Err(e) => {
                warn!("execute_count error: {}", e);
                0
            }
        }
    }

    /// Run a paginated search SELECT, bind the dynamic filter params plus
    /// LIMIT and OFFSET, and return `SearchResult` items.
    async fn fetch_search_results(
        &self,
        sql: &str,
        binds: &[BindValue],
        limit: i64,
        offset: i64,
    ) -> Vec<SearchResult> {
        let mut query = sqlx::query(sql);
        for v in binds {
            query = match v {
                BindValue::String(s) => query.bind(s.clone()),
                BindValue::Int(i) => query.bind(*i),
                BindValue::Bool(b) => query.bind(*b),
            };
        }
        query = query.bind(limit).bind(offset);

        match query.fetch_all(&self.pool).await {
            Ok(rows) => rows.iter().map(Self::map_search_row).collect(),
            Err(e) => {
                warn!("fetch_search_results error: {}", e);
                Vec::new()
            }
        }
    }

    /// Shorthand for a parameter-less `COUNT(*)` query.
    async fn scalar_count(&self, sql: &str) -> i64 {
        match sqlx::query(sql).fetch_one(&self.pool).await {
            Ok(row) => row.get::<i64, _>("cnt"),
            Err(e) => {
                warn!("scalar_count error: {}", e);
                0
            }
        }
    }

    /// Load all permits for a property, ordered by issued_date DESC.
    async fn load_permits_for(&self, property_id: i64) -> Vec<Permit> {
        sqlx::query_as::<_, Permit>(
            "SELECT * FROM permits WHERE property_id = ? ORDER BY issued_date DESC",
        )
        .bind(property_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    /// Build the five roof-age distribution buckets.
    async fn build_roof_age_distribution(&self) -> Vec<RoofAgeDistribution> {
        let buckets: &[(&str, &str)] = &[
            (
                "0-5 years",
                "SELECT COUNT(*) AS cnt FROM properties WHERE roof_age BETWEEN 0 AND 5",
            ),
            (
                "6-10 years",
                "SELECT COUNT(*) AS cnt FROM properties WHERE roof_age BETWEEN 6 AND 10",
            ),
            (
                "11-15 years",
                "SELECT COUNT(*) AS cnt FROM properties WHERE roof_age BETWEEN 11 AND 15",
            ),
            (
                "16-20 years",
                "SELECT COUNT(*) AS cnt FROM properties WHERE roof_age BETWEEN 16 AND 20",
            ),
            (
                "20+ years",
                "SELECT COUNT(*) AS cnt FROM properties WHERE roof_age > 20",
            ),
        ];

        let mut groups = Vec::with_capacity(buckets.len());
        for &(label, sql) in buckets {
            let count = self.scalar_count(sql).await;
            groups.push(RoofAgeDistribution {
                range: label.to_string(),
                count,
            });
        }
        groups
    }

    /// Parse an owner name string into (first_name, last_name).
    /// Handles "LAST, FIRST" and "FIRST LAST" formats.
    fn parse_owner_name(raw: &str) -> (String, String) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return (String::new(), String::new());
        }

        if let Some(comma_pos) = trimmed.find(',') {
            let last = trimmed[..comma_pos].trim().to_string();
            let first = trimmed[comma_pos + 1..].trim().to_string();
            (first, last)
        } else {
            let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
            match parts.len() {
                1 => (String::new(), parts[0].to_string()),
                _ => (parts[0].to_string(), parts[1].to_string()),
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Row mapper (SearchResult -- built from raw query, not FromRow)
    // ═══════════════════════════════════════════════════════════════════════

    fn map_search_row(row: &SqliteRow) -> SearchResult {
        SearchResult {
            property_id: row.get::<i64, _>("property_id"),
            address: row.get::<String, _>("address"),
            city: row.get::<String, _>("city"),
            county: row.get::<String, _>("county"),
            zip_code: row.get::<String, _>("zip_code"),
            parcel_id: row.get::<String, _>("parcel_id"),
            property_type: row.get::<String, _>("property_type"),
            year_built: row.get::<Option<i64>, _>("year_built"),
            roof_type: row.get::<String, _>("roof_type"),
            roof_age: row.get::<Option<i64>, _>("roof_age"),
            estimated_value: row.get::<Option<f64>, _>("estimated_value"),
            flood_zone: row.get::<Option<String>, _>("flood_zone"),
            is_in_flood_zone: row.get::<bool, _>("is_in_flood_zone"),
            is_in_high_risk_flood_zone: row.get::<bool, _>("is_in_high_risk_flood_zone"),
            fire_risk_level: row.get::<Option<String>, _>("fire_risk_level"),
            overall_risk_score: row.get::<Option<i64>, _>("overall_risk_score"),
            lead_priority: row.get::<Option<String>, _>("lead_priority"),
            owner_name: {
                let name: String = row.get("owner_name");
                if name.trim().is_empty() { None } else { Some(name) }
            },
            permit_count: row.get::<Option<i64>, _>("permit_count"),
            last_permit_date: row.get::<Option<NaiveDateTime>, _>("last_permit_date"),
            claim_count: row.get::<Option<i64>, _>("claim_count"),
        }
    }
}
