use crate::models::*;
use chrono::NaiveDateTime;
use reqwest;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::warn;

/// Client for searching Florida property records via FDOT, statewide cadastral,
/// US Census geocoder, and Miami-Dade open data APIs.
pub struct FloridaPropertyDataClient {
    client: reqwest::Client,
}

// ── Static lookup tables ─────────────────────────────────────────────────────

/// FDOT county parcel FeatureServer layer IDs (0-based, all 67 Florida counties).
static FDOT_COUNTY_LAYERS: LazyLock<HashMap<&'static str, i32>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("Alachua", 0);
    m.insert("Baker", 1);
    m.insert("Bay", 2);
    m.insert("Bradford", 3);
    m.insert("Brevard", 4);
    m.insert("Broward", 5);
    m.insert("Calhoun", 6);
    m.insert("Charlotte", 7);
    m.insert("Citrus", 8);
    m.insert("Clay", 9);
    m.insert("Collier", 10);
    m.insert("Columbia", 11);
    m.insert("DeSoto", 12);
    m.insert("Dixie", 13);
    m.insert("Duval", 14);
    m.insert("Escambia", 15);
    m.insert("Flagler", 16);
    m.insert("Franklin", 17);
    m.insert("Gadsden", 18);
    m.insert("Gilchrist", 19);
    m.insert("Glades", 20);
    m.insert("Gulf", 21);
    m.insert("Hamilton", 22);
    m.insert("Hardee", 23);
    m.insert("Hendry", 24);
    m.insert("Hernando", 25);
    m.insert("Highlands", 26);
    m.insert("Hillsborough", 27);
    m.insert("Holmes", 28);
    m.insert("Indian River", 29);
    m.insert("Jackson", 30);
    m.insert("Jefferson", 31);
    m.insert("Lafayette", 32);
    m.insert("Lake", 33);
    m.insert("Lee", 34);
    m.insert("Leon", 35);
    m.insert("Levy", 36);
    m.insert("Liberty", 37);
    m.insert("Madison", 38);
    m.insert("Manatee", 39);
    m.insert("Marion", 40);
    m.insert("Martin", 41);
    m.insert("Miami-Dade", 42);
    m.insert("Monroe", 43);
    m.insert("Nassau", 44);
    m.insert("Okaloosa", 45);
    m.insert("Okeechobee", 46);
    m.insert("Orange", 47);
    m.insert("Osceola", 48);
    m.insert("Palm Beach", 49);
    m.insert("Pasco", 50);
    m.insert("Pinellas", 51);
    m.insert("Polk", 52);
    m.insert("Putnam", 53);
    m.insert("Santa Rosa", 54);
    m.insert("Sarasota", 55);
    m.insert("Seminole", 56);
    m.insert("St. Johns", 57);
    m.insert("St. Lucie", 58);
    m.insert("Sumter", 59);
    m.insert("Suwannee", 60);
    m.insert("Taylor", 61);
    m.insert("Union", 62);
    m.insert("Volusia", 63);
    m.insert("Wakulla", 64);
    m.insert("Walton", 65);
    m.insert("Washington", 66);
    m
});

/// Florida Department of Revenue (DOR) county codes (1-67).
static DOR_COUNTY_CODES: LazyLock<HashMap<&'static str, i32>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("Alachua", 1);
    m.insert("Baker", 2);
    m.insert("Bay", 3);
    m.insert("Bradford", 4);
    m.insert("Brevard", 5);
    m.insert("Broward", 6);
    m.insert("Calhoun", 7);
    m.insert("Charlotte", 8);
    m.insert("Citrus", 9);
    m.insert("Clay", 10);
    m.insert("Collier", 11);
    m.insert("Columbia", 12);
    m.insert("DeSoto", 13);
    m.insert("Dixie", 14);
    m.insert("Duval", 15);
    m.insert("Escambia", 16);
    m.insert("Flagler", 17);
    m.insert("Franklin", 18);
    m.insert("Gadsden", 19);
    m.insert("Gilchrist", 20);
    m.insert("Glades", 21);
    m.insert("Gulf", 22);
    m.insert("Hamilton", 23);
    m.insert("Hardee", 24);
    m.insert("Hendry", 25);
    m.insert("Hernando", 26);
    m.insert("Highlands", 27);
    m.insert("Hillsborough", 28);
    m.insert("Holmes", 29);
    m.insert("Indian River", 30);
    m.insert("Jackson", 31);
    m.insert("Jefferson", 32);
    m.insert("Lafayette", 33);
    m.insert("Lake", 34);
    m.insert("Lee", 35);
    m.insert("Leon", 36);
    m.insert("Levy", 37);
    m.insert("Liberty", 38);
    m.insert("Madison", 39);
    m.insert("Manatee", 40);
    m.insert("Marion", 41);
    m.insert("Martin", 42);
    m.insert("Miami-Dade", 43);
    m.insert("Monroe", 44);
    m.insert("Nassau", 45);
    m.insert("Okaloosa", 46);
    m.insert("Okeechobee", 47);
    m.insert("Orange", 48);
    m.insert("Osceola", 49);
    m.insert("Palm Beach", 50);
    m.insert("Pasco", 51);
    m.insert("Pinellas", 52);
    m.insert("Polk", 53);
    m.insert("Putnam", 54);
    m.insert("Santa Rosa", 55);
    m.insert("Sarasota", 56);
    m.insert("Seminole", 57);
    m.insert("St. Johns", 58);
    m.insert("St. Lucie", 59);
    m.insert("Sumter", 60);
    m.insert("Suwannee", 61);
    m.insert("Taylor", 62);
    m.insert("Union", 63);
    m.insert("Volusia", 64);
    m.insert("Wakulla", 65);
    m.insert("Walton", 66);
    m.insert("Washington", 67);
    m
});

impl FloridaPropertyDataClient {
    /// Create a new client backed by a default reqwest HTTP client.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Search Florida property records by owner name through the FDOT
    /// county parcel FeatureServer layer.
    pub async fn search_by_owner_name(
        &self,
        owner: &str,
        county: &str,
        max: i32,
    ) -> Vec<PropertyRecord> {
        let layer_id = match FDOT_COUNTY_LAYERS.get(county) {
            Some(id) => *id,
            None => {
                warn!("Unsupported county for FDOT lookup: {}", county);
                return Vec::new();
            }
        };

        let encoded_owner = urlencoding::encode(owner);
        let url = format!(
            "https://services1.arcgis.com/O1JpcwDW8sjYuddV/ArcGIS/rest/services/\
             Florida_Parcels/FeatureServer/{}/query?\
             where=OWN_NAME+LIKE+%27%25{}%25%27\
             &outFields=*&returnGeometry=true&resultRecordCount={}&f=json",
            layer_id, encoded_owner, max
        );

        match self.fetch_and_parse_fdot(&url, county).await {
            Ok(records) => records,
            Err(e) => {
                warn!("FDOT owner search failed for '{}' in {}: {}", owner, county, e);
                Vec::new()
            }
        }
    }

    /// Search Florida property records by site address through the FDOT
    /// county parcel FeatureServer layer.
    pub async fn search_by_address(
        &self,
        address: &str,
        county: &str,
        max: i32,
    ) -> Vec<PropertyRecord> {
        let layer_id = match FDOT_COUNTY_LAYERS.get(county) {
            Some(id) => *id,
            None => {
                warn!("Unsupported county for FDOT lookup: {}", county);
                return Vec::new();
            }
        };

        let encoded_address = urlencoding::encode(address);
        let url = format!(
            "https://services1.arcgis.com/O1JpcwDW8sjYuddV/ArcGIS/rest/services/\
             Florida_Parcels/FeatureServer/{}/query?\
             where=SITEADDR+LIKE+%27%25{}%25%27\
             &outFields=*&returnGeometry=true&resultRecordCount={}&f=json",
            layer_id, encoded_address, max
        );

        match self.fetch_and_parse_fdot(&url, county).await {
            Ok(records) => records,
            Err(e) => {
                warn!(
                    "FDOT address search failed for '{}' in {}: {}",
                    address, county, e
                );
                Vec::new()
            }
        }
    }

    /// Search the FL Statewide Cadastral / DOR dataset for property parcels.
    pub async fn search_statewide_cadastral(
        &self,
        county: &str,
        address: Option<&str>,
        max: i32,
    ) -> Vec<PropertyRecord> {
        let county_code = match DOR_COUNTY_CODES.get(county) {
            Some(code) => *code,
            None => {
                warn!("Unsupported county for DOR cadastral: {}", county);
                return Vec::new();
            }
        };

        let where_clause = if let Some(addr) = address {
            let encoded = urlencoding::encode(addr);
            format!(
                "CO_NO={}+AND+SITEADDR+LIKE+%27%25{}%25%27",
                county_code, encoded
            )
        } else {
            format!("CO_NO={}", county_code)
        };

        let url = format!(
            "https://ca.dep.state.fl.us/arcgis/rest/services/OpenData/\
             FL_Statewide_Cadastral/FeatureServer/0/query?\
             where={}&outFields=*&returnGeometry=true&resultRecordCount={}&f=json",
            where_clause, max
        );

        match self.fetch_and_parse_cadastral(&url, county).await {
            Ok(records) => records,
            Err(e) => {
                warn!("Statewide cadastral search failed for {}: {}", county, e);
                Vec::new()
            }
        }
    }

    /// High-level property search: tries FDOT first, falls back to statewide
    /// cadastral, and finally the US Census geocoder.
    pub async fn search_florida_properties(
        &self,
        query: &str,
        county: Option<&str>,
    ) -> Vec<PropertyRecord> {
        // 1. Try FDOT if a county is specified.
        if let Some(county_name) = county {
            let results = self.search_by_address(query, county_name, 25).await;
            if !results.is_empty() {
                return results;
            }

            // 2. Fall back to statewide cadastral.
            let cadastral = self
                .search_statewide_cadastral(county_name, Some(query), 25)
                .await;
            if !cadastral.is_empty() {
                return cadastral;
            }
        }

        // 3. Last resort: US Census geocoder.
        self.search_via_census_geocoder(query).await
    }

    /// Geocode an address through the US Census Bureau Geocoding API and
    /// return a single-entry property record with the resolved coordinates.
    pub async fn search_via_census_geocoder(&self, query: &str) -> Vec<PropertyRecord> {
        let encoded = urlencoding::encode(query);
        let url = format!(
            "https://geocoding.geo.census.gov/geocoder/locations/onelineaddress?\
             address={}&benchmark=Public_AR_Current&format=json",
            encoded
        );

        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Census geocoder request failed: {}", e);
                return Vec::new();
            }
        };

        let body: Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                warn!("Census geocoder JSON parse failed: {}", e);
                return Vec::new();
            }
        };

        let matches = body
            .pointer("/result/addressMatches")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        matches
            .iter()
            .map(|m| {
                let coords = m.get("coordinates").unwrap_or(&Value::Null);
                PropertyRecord {
                    address: m.get("matchedAddress").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    latitude: coords.get("y").and_then(|v| v.as_f64()),
                    longitude: coords.get("x").and_then(|v| v.as_f64()),
                    state: Some("FL".to_string()),
                    data_source: Some("US Census Geocoder".to_string()),
                    ..Default::default()
                }
            })
            .collect()
    }

    /// Fetch building permits from the Miami-Dade County Open Data Hub.
    pub async fn get_miami_dade_permits(
        &self,
        address: &str,
        max: i32,
    ) -> Vec<PermitRecord> {
        let encoded = urlencoding::encode(address);
        let url = format!(
            "https://opendata.miamidade.gov/resource/gkvh-5bge.json?\
             $where=processaddr1+like+%27%25{}%25%27&$limit={}",
            encoded, max
        );

        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Miami-Dade permits request failed: {}", e);
                return Vec::new();
            }
        };

        let body: Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                warn!("Miami-Dade permits JSON parse failed: {}", e);
                return Vec::new();
            }
        };

        let items = body.as_array().cloned().unwrap_or_default();

        items
            .iter()
            .map(|item| PermitRecord {
                permit_number: get_string_opt(item, "permit_num"),
                permit_type: get_string_opt(item, "permit_type"),
                description: get_string_opt(item, "scope_of_work"),
                status: get_string_opt(item, "status_current"),
                issued_date: parse_iso_datetime(item, "issue_date"),
                contractor_name: get_string_opt(item, "contractor_name"),
                contractor_license: get_string_opt(item, "contractor_cert"),
                estimated_cost: get_f64_opt(item, "est_value"),
                address: get_string_opt(item, "processaddr1"),
            })
            .collect()
    }

    /// Return the list of all 67 supported Florida counties.
    pub fn get_supported_counties() -> Vec<String> {
        let mut counties: Vec<String> = FDOT_COUNTY_LAYERS
            .keys()
            .map(|k| k.to_string())
            .collect();
        counties.sort();
        counties
    }

    /// Parse an ArcGIS FeatureServer JSON response (FDOT parcels) into
    /// `PropertyRecord` values. Handles both point and polygon geometry for
    /// extracting lat/lon.
    pub fn parse_fdot_response(json: &str, county: &str) -> Vec<PropertyRecord> {
        let body: Value = match serde_json::from_str(json) {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to parse FDOT JSON: {}", e);
                return Vec::new();
            }
        };

        Self::extract_property_records(&body, county)
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    async fn fetch_and_parse_fdot(
        &self,
        url: &str,
        county: &str,
    ) -> Result<Vec<PropertyRecord>, String> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("FDOT request failed: {}", e))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| format!("FDOT JSON parse failed: {}", e))?;

        Ok(Self::extract_property_records(&body, county))
    }

    async fn fetch_and_parse_cadastral(
        &self,
        url: &str,
        county: &str,
    ) -> Result<Vec<PropertyRecord>, String> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Cadastral request failed: {}", e))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| format!("Cadastral JSON parse failed: {}", e))?;

        Ok(Self::extract_property_records(&body, county))
    }

    fn extract_property_records(body: &Value, county: &str) -> Vec<PropertyRecord> {
        let features = body
            .get("features")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        features
            .iter()
            .map(|feat| {
                let attrs = feat.get("attributes").unwrap_or(&Value::Null);
                let geom = feat.get("geometry").unwrap_or(&Value::Null);

                // Handle both point and polygon geometry for lat/lon
                let (lat, lon) = Self::extract_coordinates(geom);

                PropertyRecord {
                    parcel_id: get_string_opt(attrs, "PARCELNO")
                        .or_else(|| get_string_opt(attrs, "PARCEL_ID")),
                    owner_name: get_string_opt(attrs, "OWN_NAME")
                        .or_else(|| get_string_opt(attrs, "OWNER")),
                    owner_address: get_string_opt(attrs, "OWN_ADDR1"),
                    owner_city: get_string_opt(attrs, "OWN_CITY"),
                    owner_state: get_string_opt(attrs, "OWN_STATE"),
                    owner_zip: get_string_opt(attrs, "OWN_ZIPCD"),
                    address: get_string_opt(attrs, "SITEADDR")
                        .or_else(|| get_string_opt(attrs, "PHY_ADDR1")),
                    city: get_string_opt(attrs, "SITECITY")
                        .or_else(|| get_string_opt(attrs, "PHY_CITY")),
                    state: Some("FL".to_string()),
                    zip_code: get_string_opt(attrs, "SITEZIP")
                        .or_else(|| get_string_opt(attrs, "PHY_ZIPCD")),
                    county: Some(county.to_string()),
                    property_type: get_string_opt(attrs, "DOR_UC")
                        .or_else(|| get_string_opt(attrs, "USE_CODE")),
                    just_value: get_f64_opt(attrs, "JV"),
                    assessed_value: get_f64_opt(attrs, "AV_NM"),
                    taxable_value: get_f64_opt(attrs, "TV_NM"),
                    year_built: attrs
                        .get("ACT_YR_BLT")
                        .and_then(|v| v.as_i64()),
                    square_footage: get_f64_opt(attrs, "TOT_LVG_AR"),
                    lot_size: get_f64_opt(attrs, "LND_SQFOOT"),
                    bedrooms: attrs
                        .get("NO_BDRM")
                        .and_then(|v| v.as_i64()),
                    bathrooms: attrs
                        .get("NO_BATHS")
                        .and_then(|v| v.as_i64()),
                    latitude: if lat != 0.0 { Some(lat) } else { None },
                    longitude: if lon != 0.0 { Some(lon) } else { None },
                    data_source: Some("FDOT Florida Parcels".to_string()),
                }
            })
            .collect()
    }

    /// Extract lat/lon from ArcGIS geometry. Handles three shapes:
    /// - **Point**: `{ "x": ..., "y": ... }`
    /// - **Polygon**: `{ "rings": [[[x,y], ...]] }` -- uses centroid of first ring
    /// - **Envelope / missing**: returns `(0.0, 0.0)`
    fn extract_coordinates(geom: &Value) -> (f64, f64) {
        // Point geometry
        if let (Some(x), Some(y)) = (
            geom.get("x").and_then(|v| v.as_f64()),
            geom.get("y").and_then(|v| v.as_f64()),
        ) {
            return (y, x); // lat = y, lon = x
        }

        // Polygon geometry -- compute centroid of first ring
        if let Some(rings) = geom.get("rings").and_then(|v| v.as_array()) {
            if let Some(ring) = rings.first().and_then(|r| r.as_array()) {
                if !ring.is_empty() {
                    let (sum_x, sum_y, count) = ring.iter().fold(
                        (0.0_f64, 0.0_f64, 0_usize),
                        |(sx, sy, c), pt| {
                            let arr = pt.as_array();
                            let px = arr
                                .and_then(|a| a.first())
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);
                            let py = arr
                                .and_then(|a| a.get(1))
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);
                            (sx + px, sy + py, c + 1)
                        },
                    );
                    if count > 0 {
                        return (sum_y / count as f64, sum_x / count as f64);
                    }
                }
            }
        }

        (0.0, 0.0)
    }
}

// ── Implement Default for PropertyRecord ─────────────────────────────────────

impl Default for PropertyRecord {
    fn default() -> Self {
        Self {
            address: None,
            city: None,
            county: None,
            state: None,
            zip_code: None,
            parcel_id: None,
            latitude: None,
            longitude: None,
            data_source: None,
            year_built: None,
            square_footage: None,
            lot_size: None,
            property_type: None,
            just_value: None,
            assessed_value: None,
            taxable_value: None,
            bedrooms: None,
            bathrooms: None,
            owner_name: None,
            owner_address: None,
            owner_city: None,
            owner_state: None,
            owner_zip: None,
        }
    }
}

// ── Free-standing JSON helpers ───────────────────────────────────────────────

/// Extract a string field from a `serde_json::Value` map, returning `None` on
/// missing, null, or empty.
fn get_string_opt(val: &Value, key: &str) -> Option<String> {
    val.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Extract a floating-point field, returning `None` on missing/null.
fn get_f64_opt(val: &Value, key: &str) -> Option<f64> {
    val.get(key).and_then(|v| v.as_f64())
}

/// Extract an integer field, returning `0` on missing/null.
#[allow(dead_code)]
fn get_i64(val: &Value, key: &str) -> i64 {
    val.get(key).and_then(|v| v.as_i64()).unwrap_or(0)
}

/// Try to read a timestamp field: either an ISO-8601 string or epoch
/// milliseconds (common in ArcGIS).
fn parse_iso_datetime(val: &Value, key: &str) -> Option<NaiveDateTime> {
    let v = val.get(key)?;

    // ISO-8601 string
    if let Some(s) = v.as_str() {
        if !s.is_empty() {
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.fZ") {
                return Some(dt);
            }
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ") {
                return Some(dt);
            }
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f") {
                return Some(dt);
            }
            if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                return d.and_hms_opt(0, 0, 0);
            }
        }
    }

    // Epoch milliseconds (ArcGIS)
    if let Some(millis) = v.as_i64() {
        let secs = millis / 1000;
        if let Some(dt) = chrono::DateTime::from_timestamp(secs, 0) {
            return Some(dt.naive_utc());
        }
    }

    None
}
