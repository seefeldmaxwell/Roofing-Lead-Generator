use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::warn;

use crate::models::CountyStats;

// ── Static lookup tables ────────────────────────────────────────────────────

/// All 67 Florida county slugs mapped to their display names.
pub static FLORIDA_COUNTIES: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        let mut m = HashMap::with_capacity(67);
        m.insert("alachua", "Alachua");
        m.insert("baker", "Baker");
        m.insert("bay", "Bay");
        m.insert("bradford", "Bradford");
        m.insert("brevard", "Brevard");
        m.insert("broward", "Broward");
        m.insert("calhoun", "Calhoun");
        m.insert("charlotte", "Charlotte");
        m.insert("citrus", "Citrus");
        m.insert("clay", "Clay");
        m.insert("collier", "Collier");
        m.insert("columbia", "Columbia");
        m.insert("desoto", "DeSoto");
        m.insert("dixie", "Dixie");
        m.insert("duval", "Duval");
        m.insert("escambia", "Escambia");
        m.insert("flagler", "Flagler");
        m.insert("franklin", "Franklin");
        m.insert("gadsden", "Gadsden");
        m.insert("gilchrist", "Gilchrist");
        m.insert("glades", "Glades");
        m.insert("gulf", "Gulf");
        m.insert("hamilton", "Hamilton");
        m.insert("hardee", "Hardee");
        m.insert("hendry", "Hendry");
        m.insert("hernando", "Hernando");
        m.insert("highlands", "Highlands");
        m.insert("hillsborough", "Hillsborough");
        m.insert("holmes", "Holmes");
        m.insert("indian-river", "Indian River");
        m.insert("jackson", "Jackson");
        m.insert("jefferson", "Jefferson");
        m.insert("lafayette", "Lafayette");
        m.insert("lake", "Lake");
        m.insert("lee", "Lee");
        m.insert("leon", "Leon");
        m.insert("levy", "Levy");
        m.insert("liberty", "Liberty");
        m.insert("madison", "Madison");
        m.insert("manatee", "Manatee");
        m.insert("marion", "Marion");
        m.insert("martin", "Martin");
        m.insert("miami-dade", "Miami-Dade");
        m.insert("monroe", "Monroe");
        m.insert("nassau", "Nassau");
        m.insert("okaloosa", "Okaloosa");
        m.insert("okeechobee", "Okeechobee");
        m.insert("orange", "Orange");
        m.insert("osceola", "Osceola");
        m.insert("palm-beach", "Palm Beach");
        m.insert("pasco", "Pasco");
        m.insert("pinellas", "Pinellas");
        m.insert("polk", "Polk");
        m.insert("putnam", "Putnam");
        m.insert("santa-rosa", "Santa Rosa");
        m.insert("sarasota", "Sarasota");
        m.insert("seminole", "Seminole");
        m.insert("st-johns", "St. Johns");
        m.insert("st-lucie", "St. Lucie");
        m.insert("sumter", "Sumter");
        m.insert("suwannee", "Suwannee");
        m.insert("taylor", "Taylor");
        m.insert("union", "Union");
        m.insert("volusia", "Volusia");
        m.insert("wakulla", "Wakulla");
        m.insert("walton", "Walton");
        m.insert("washington", "Washington");
        m
    });

/// Major Florida city slugs mapped to (city_name, county_name).
pub static FLORIDA_CITIES: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> =
    LazyLock::new(|| {
        let mut m = HashMap::with_capacity(38);
        m.insert("miami", ("Miami", "Miami-Dade"));
        m.insert("miami-beach", ("Miami Beach", "Miami-Dade"));
        m.insert("hialeah", ("Hialeah", "Miami-Dade"));
        m.insert("homestead", ("Homestead", "Miami-Dade"));
        m.insert("fort-lauderdale", ("Fort Lauderdale", "Broward"));
        m.insert("hollywood", ("Hollywood", "Broward"));
        m.insert("pembroke-pines", ("Pembroke Pines", "Broward"));
        m.insert("coral-springs", ("Coral Springs", "Broward"));
        m.insert("west-palm-beach", ("West Palm Beach", "Palm Beach"));
        m.insert("boca-raton", ("Boca Raton", "Palm Beach"));
        m.insert("boynton-beach", ("Boynton Beach", "Palm Beach"));
        m.insert("delray-beach", ("Delray Beach", "Palm Beach"));
        m.insert("orlando", ("Orlando", "Orange"));
        m.insert("kissimmee", ("Kissimmee", "Osceola"));
        m.insert("tampa", ("Tampa", "Hillsborough"));
        m.insert("st-petersburg", ("St. Petersburg", "Pinellas"));
        m.insert("clearwater", ("Clearwater", "Pinellas"));
        m.insert("jacksonville", ("Jacksonville", "Duval"));
        m.insert("tallahassee", ("Tallahassee", "Leon"));
        m.insert("gainesville", ("Gainesville", "Alachua"));
        m.insert("fort-myers", ("Fort Myers", "Lee"));
        m.insert("cape-coral", ("Cape Coral", "Lee"));
        m.insert("naples", ("Naples", "Collier"));
        m.insert("sarasota", ("Sarasota", "Sarasota"));
        m.insert("bradenton", ("Bradenton", "Manatee"));
        m.insert("lakeland", ("Lakeland", "Polk"));
        m.insert("daytona-beach", ("Daytona Beach", "Volusia"));
        m.insert("port-st-lucie", ("Port St. Lucie", "St. Lucie"));
        m.insert("melbourne", ("Melbourne", "Brevard"));
        m.insert("pensacola", ("Pensacola", "Escambia"));
        m.insert("ocala", ("Ocala", "Marion"));
        m.insert("key-west", ("Key West", "Monroe"));
        m.insert("panama-city", ("Panama City", "Bay"));
        m.insert("doral", ("Doral", "Miami-Dade"));
        m.insert("sunrise", ("Sunrise", "Broward"));
        m.insert("plantation", ("Plantation", "Broward"));
        m.insert("pompano-beach", ("Pompano Beach", "Broward"));
        m.insert("miramar", ("Miramar", "Broward"));
        m
    });

// ── LandingPageService ──────────────────────────────────────────────────────

/// Service that powers the public-facing SEO landing pages for individual
/// Florida counties and cities.
pub struct LandingPageService {
    pool: SqlitePool,
}

impl LandingPageService {
    /// Create a new `LandingPageService`.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Aggregate statistics for a single county.  Returns a `CountyStats`
    /// matching the models.rs definition (county, property_count, permit_count,
    /// old_roof_count, flood_zone_count, high_fire_risk_count, disaster_count).
    pub async fn get_county_stats(&self, county_name: &str) -> CountyStats {
        // Property-level aggregates
        let prop_row = sqlx::query(
            "SELECT \
                 COUNT(*) AS property_count, \
                 COALESCE(SUM(CASE WHEN roof_age > 15 THEN 1 ELSE 0 END), 0) \
                     AS old_roof_count, \
                 COALESCE(SUM(CASE WHEN is_in_flood_zone = 1 THEN 1 ELSE 0 END), 0) \
                     AS flood_zone_count, \
                 COALESCE(SUM(CASE WHEN fire_risk_level IN ('High', 'Extreme') \
                              THEN 1 ELSE 0 END), 0) \
                     AS high_fire_risk_count \
             FROM properties \
             WHERE county = ?",
        )
        .bind(county_name)
        .fetch_one(&self.pool)
        .await;

        let (property_count, old_roof_count, flood_zone_count, high_fire_risk_count) =
            match prop_row {
                Ok(r) => (
                    r.get::<i64, _>("property_count"),
                    r.get::<i64, _>("old_roof_count"),
                    r.get::<i64, _>("flood_zone_count"),
                    r.get::<i64, _>("high_fire_risk_count"),
                ),
                Err(e) => {
                    warn!(
                        "Failed to fetch county property stats for '{}': {}",
                        county_name, e
                    );
                    (0, 0, 0, 0)
                }
            };

        // Permit count for the county
        let permit_count = self
            .scalar_count_with_bind(
                "SELECT COUNT(*) AS cnt FROM permits \
                 WHERE property_id IN (SELECT id FROM properties WHERE county = ?)",
                county_name,
            )
            .await;

        // Disaster count for the county
        let disaster_count = self
            .scalar_count_with_bind(
                "SELECT COUNT(*) AS cnt FROM fema_disasters \
                 WHERE declared_county = ?",
                county_name,
            )
            .await;

        CountyStats {
            county: county_name.to_string(),
            property_count,
            permit_count,
            old_roof_count,
            flood_zone_count,
            high_fire_risk_count,
            disaster_count,
        }
    }

    /// Return global aggregate numbers used on the top-level landing page:
    /// `(total_properties, total_permits, total_disasters, total_counties)`.
    pub async fn get_global_stats(&self) -> (i64, i64, i64, i64) {
        let total_properties = self
            .scalar_count("SELECT COUNT(*) AS cnt FROM properties")
            .await;
        let total_permits = self
            .scalar_count("SELECT COUNT(*) AS cnt FROM permits")
            .await;
        let total_disasters = self
            .scalar_count("SELECT COUNT(*) AS cnt FROM fema_disasters")
            .await;
        let total_counties = self
            .scalar_count(
                "SELECT COUNT(DISTINCT county) AS cnt FROM properties WHERE county != ''",
            )
            .await;

        (total_properties, total_permits, total_disasters, total_counties)
    }

    /// Resolve a county slug (e.g. `"miami-dade"`) to its display name.
    /// Returns `None` if the slug is not recognised.
    pub fn county_name_from_slug(slug: &str) -> Option<&'static str> {
        FLORIDA_COUNTIES.get(slug).copied()
    }

    /// Resolve a city slug (e.g. `"fort-lauderdale"`) to
    /// `(city_name, county_name)`.  Returns `None` if the slug is not recognised.
    pub fn city_info_from_slug(slug: &str) -> Option<(&'static str, &'static str)> {
        FLORIDA_CITIES.get(slug).copied()
    }

    // ── Private helpers ─────────────────────────────────────────────────────

    /// Execute a parameter-less COUNT query.
    async fn scalar_count(&self, sql: &str) -> i64 {
        match sqlx::query(sql).fetch_one(&self.pool).await {
            Ok(row) => row.get::<i64, _>("cnt"),
            Err(e) => {
                warn!("scalar_count error: {}", e);
                0
            }
        }
    }

    /// Execute a COUNT query with a single string bind parameter.
    async fn scalar_count_with_bind(&self, sql: &str, param: &str) -> i64 {
        match sqlx::query(sql).bind(param).fetch_one(&self.pool).await {
            Ok(row) => row.get::<i64, _>("cnt"),
            Err(e) => {
                warn!("scalar_count_with_bind error: {}", e);
                0
            }
        }
    }
}
