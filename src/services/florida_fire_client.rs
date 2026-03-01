use crate::models::*;
use chrono::NaiveDateTime;
use reqwest;
use serde_json::Value;
use tracing::warn;

/// Client for querying active wildfire incidents in Florida from the
/// NIFC IRWIN (Integrated Reporting of Wildland-Fire Information) ArcGIS service
/// and computing county-level wildfire risk scores.
pub struct FloridaFireDataClient {
    client: reqwest::Client,
}

impl FloridaFireDataClient {
    /// Create a new client with default HTTP settings.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch active wildfire incidents in the state of Florida from the NIFC
    /// IRWIN ArcGIS FeatureServer.
    pub async fn get_active_wildfires_in_florida(&self) -> Vec<WildfireIncident> {
        // NIFC IRWIN Incident Feature Service -- filter to Florida (POOState = 'US-FL')
        // and only active incidents.
        let url = "https://services3.arcgis.com/T4QMspbfLg3qTGWY/ArcGIS/rest/services/\
                   IRWIN_Perimeters_to_Date/FeatureServer/0/query?\
                   where=POOState%3D%27US-FL%27+AND+IsActive%3D%27Y%27\
                   &outFields=IncidentName%2CIrwinID%2CFireDiscoveryDateTime%2C\
                   ContainmentDateTime%2CDailyAcres%2CPercentContained%2C\
                   POOCounty%2CFireCause%2CInitialLatitude%2CInitialLongitude%2C\
                   ResidencesDestroyed%2CResidencesThreatened%2CIsActive\
                   &returnGeometry=false&f=json";

        match self.fetch_wildfires(url).await {
            Ok(incidents) => incidents,
            Err(e) => {
                warn!("Failed to fetch active FL wildfires from NIFC IRWIN: {}", e);
                Vec::new()
            }
        }
    }

    /// Calculate a wildfire risk score (0-100) for a given location based on its
    /// county. Scores are derived from historical wildfire frequency, vegetation
    /// density, drought susceptibility, and proximity to wildland-urban interface
    /// areas within each Florida county.
    pub fn calculate_wildfire_risk_score(&self, _lat: f64, _lon: f64, county: &str) -> i32 {
        let normalized = county
            .to_lowercase()
            .replace(" county", "")
            .trim()
            .to_string();

        match normalized.as_str() {
            // Very high risk (80-85) -- large rural / forested counties
            "collier" => 85,
            "liberty" => 82,
            "glades" => 82,
            "hendry" => 80,

            // High risk (70-79)
            "dixie" => 78,
            "taylor" => 78,
            "franklin" => 78,
            "highlands" => 78,
            "calhoun" => 78,
            "polk" => 75,
            "marion" => 75,
            "putnam" => 75,
            "lafayette" => 75,
            "wakulla" => 75,
            "gulf" => 75,
            "desoto" => 75,
            "okeechobee" => 75,
            "osceola" => 73,
            "volusia" => 72,
            "lake" => 72,
            "baker" => 72,
            "madison" => 72,
            "jefferson" => 72,
            "bay" => 72,
            "washington" => 72,
            "gilchrist" => 72,
            "levy" => 72,
            "hardee" => 72,
            "brevard" => 70,
            "citrus" => 70,
            "flagler" => 70,
            "columbia" => 70,
            "suwannee" => 70,
            "gadsden" => 70,
            "walton" => 70,

            // Moderate risk (60-69)
            "hernando" => 68,
            "sumter" => 68,
            "clay" => 68,
            "jackson" => 68,
            "holmes" => 68,
            "santa rosa" => 68,
            "charlotte" => 68,
            "union" => 68,
            "pasco" => 65,
            "nassau" => 65,
            "alachua" => 65,
            "leon" => 65,
            "okaloosa" => 65,
            "bradford" => 65,
            "lee" => 65,
            "indian river" => 65,
            "st. johns" | "st johns" => 62,
            "st. lucie" | "st lucie" => 62,
            "escambia" => 60,
            "martin" => 60,

            // Lower risk (40-59) -- urban / coastal
            "orange" => 58,
            "manatee" => 58,
            "duval" => 55,
            "hillsborough" => 55,
            "miami-dade" | "miami dade" => 55,
            "palm beach" => 55,
            "sarasota" => 55,
            "seminole" => 55,
            "broward" => 50,
            "monroe" => 45,
            "pinellas" => 40,

            // Default for unknown counties
            _ => 50,
        }
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    async fn fetch_wildfires(&self, url: &str) -> Result<Vec<WildfireIncident>, String> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("NIFC IRWIN request failed: {}", e))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| format!("NIFC IRWIN JSON parse failed: {}", e))?;

        let features = body
            .get("features")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let incidents: Vec<WildfireIncident> = features
            .iter()
            .filter_map(|feat| {
                let attrs = feat.get("attributes")?;
                Some(self.parse_wildfire(attrs))
            })
            .collect();

        Ok(incidents)
    }

    fn parse_wildfire(&self, attrs: &Value) -> WildfireIncident {
        WildfireIncident {
            incident_name: get_string(attrs, "IncidentName"),
            irwin_id: get_string(attrs, "IrwinID"),
            discovery_date: parse_epoch_datetime(attrs, "FireDiscoveryDateTime"),
            contained_date: parse_epoch_datetime(attrs, "ContainmentDateTime"),
            acres_burned: get_f64(attrs, "DailyAcres"),
            percent_contained: get_f64(attrs, "PercentContained"),
            county: get_string(attrs, "POOCounty"),
            cause: get_string(attrs, "FireCause"),
            latitude: get_f64(attrs, "InitialLatitude"),
            longitude: get_f64(attrs, "InitialLongitude"),
            residences_destroyed: get_i64(attrs, "ResidencesDestroyed"),
            residences_threatened: get_i64(attrs, "ResidencesThreatened"),
            is_active: attrs
                .get("IsActive")
                .and_then(|v| v.as_str())
                .map(|s| s.eq_ignore_ascii_case("Y") || s.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
        }
    }
}

// ── Free-standing JSON helpers ───────────────────────────────────────────────

fn get_string(val: &Value, key: &str) -> String {
    val.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn get_f64(val: &Value, key: &str) -> f64 {
    val.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0)
}

fn get_i64(val: &Value, key: &str) -> i64 {
    val.get(key).and_then(|v| v.as_i64()).unwrap_or(0)
}

/// Convert an epoch-millisecond value (common in ArcGIS) or an ISO-8601 string
/// to a `chrono::NaiveDateTime`.
fn parse_epoch_datetime(val: &Value, key: &str) -> Option<NaiveDateTime> {
    let v = val.get(key)?;

    // Epoch milliseconds
    if let Some(millis) = v.as_i64() {
        let secs = millis / 1000;
        if let Some(dt) = chrono::DateTime::from_timestamp(secs, 0) {
            return Some(dt.naive_utc());
        }
    }

    // ISO-8601 string
    if let Some(s) = v.as_str() {
        if !s.is_empty() {
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.fZ") {
                return Some(dt);
            }
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ") {
                return Some(dt);
            }
        }
    }

    None
}
