use crate::models::*;
use chrono::NaiveDateTime;
use reqwest;
use serde_json::Value;
use tracing::warn;

/// Client for querying FEMA disaster declarations and flood zone data.
pub struct FemaApiClient {
    client: reqwest::Client,
}

impl FemaApiClient {
    /// Create a new FEMA API client with default reqwest settings.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch the most recent Florida disaster declarations from FEMA OpenAPI.
    pub async fn get_florida_disasters(&self, limit: i32) -> Vec<FemaDisaster> {
        let url = format!(
            "https://www.fema.gov/api/open/v2/DisasterDeclarationsSummaries?\
             $filter=state eq 'Florida'&$orderby=declarationDate desc&$top={}",
            limit
        );

        match self.fetch_disasters(&url).await {
            Ok(disasters) => disasters,
            Err(e) => {
                warn!("Failed to fetch Florida disasters from FEMA: {}", e);
                Vec::new()
            }
        }
    }

    /// Fetch disaster declarations filtered by a specific Florida county.
    pub async fn get_disasters_by_county(&self, county: &str, limit: i32) -> Vec<FemaDisaster> {
        let url = format!(
            "https://www.fema.gov/api/open/v2/DisasterDeclarationsSummaries?\
             $filter=state eq 'Florida' and designatedArea eq '{}'&$orderby=declarationDate desc&$top={}",
            county, limit
        );

        match self.fetch_disasters(&url).await {
            Ok(disasters) => disasters,
            Err(e) => {
                warn!(
                    "Failed to fetch disasters for county '{}' from FEMA: {}",
                    county, e
                );
                Vec::new()
            }
        }
    }

    /// Query the FEMA National Flood Hazard Layer (NFHL) ArcGIS endpoint for
    /// the flood zone at a given lat/lon coordinate.
    pub async fn get_flood_zone_by_coordinates(
        &self,
        lat: f64,
        lon: f64,
    ) -> Result<FloodZoneInfo, String> {
        let url = format!(
            "https://hazards.fema.gov/gis/nfhl/rest/services/public/NFHL/MapServer/28/query?\
             where=1%3D1&geometry={}%2C{}&geometryType=esriGeometryPoint\
             &spatialRel=esriSpatialRelIntersects\
             &outFields=FLD_ZONE%2CZONE_SUBTY%2CSFHA_TF%2CDFIRM_ID\
             &returnGeometry=false&f=json",
            lon, lat
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("FEMA NFHL request failed: {}", e))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse FEMA NFHL response: {}", e))?;

        let features = body
            .get("features")
            .and_then(|f| f.as_array())
            .ok_or_else(|| "No features array in NFHL response".to_string())?;

        if features.is_empty() {
            return Ok(FloodZoneInfo {
                flood_zone: "X".to_string(),
                zone_description: Self::get_flood_zone_description("X").to_string(),
                is_high_risk: false,
                requires_insurance: false,
                map_panel: None,
            });
        }

        let attrs = features[0]
            .get("attributes")
            .ok_or_else(|| "No attributes in NFHL feature".to_string())?;

        let zone = attrs
            .get("FLD_ZONE")
            .and_then(|v| v.as_str())
            .unwrap_or("X")
            .to_string();

        let sfha = attrs
            .get("SFHA_TF")
            .and_then(|v| v.as_str())
            .map(|v| v.eq_ignore_ascii_case("T") || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let dfirm_id = attrs
            .get("DFIRM_ID")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let is_high_risk = Self::is_high_risk_zone(&zone);
        let description = Self::get_flood_zone_description(&zone).to_string();

        Ok(FloodZoneInfo {
            flood_zone: zone,
            zone_description: description,
            is_high_risk: sfha || is_high_risk,
            requires_insurance: sfha || is_high_risk,
            map_panel: if dfirm_id.is_empty() {
                None
            } else {
                Some(dfirm_id)
            },
        })
    }

    /// Return a human-readable description for a FEMA flood zone designation.
    pub fn get_flood_zone_description(zone: &str) -> &'static str {
        match zone.to_uppercase().as_str() {
            "A" => {
                "Zone A - High risk area within the 1% annual chance (100-year) floodplain. \
                 No base flood elevations determined."
            }
            "AE" => {
                "Zone AE - High risk area within the 1% annual chance (100-year) floodplain. \
                 Base flood elevations determined."
            }
            "AH" => {
                "Zone AH - High risk area of shallow flooding (usually ponding), \
                 depths of 1-3 feet. Base flood elevations determined."
            }
            "AO" => {
                "Zone AO - High risk area of shallow flooding (usually sheet flow on sloping terrain), \
                 depths of 1-3 feet. Average depths determined."
            }
            "AR" => {
                "Zone AR - Area with a temporarily increased flood risk due to the building or \
                 restoration of a flood control system (e.g., levee). Mandatory flood insurance purchase."
            }
            "A99" => {
                "Zone A99 - Area to be protected from the 1% annual chance flood by a Federal \
                 flood protection system under construction. No base flood elevations determined."
            }
            "V" => {
                "Zone V - Coastal high risk area within the 1% annual chance floodplain with \
                 additional hazards due to storm-induced velocity wave action. No base flood \
                 elevations determined."
            }
            "VE" => {
                "Zone VE - Coastal high risk area within the 1% annual chance floodplain with \
                 additional hazards due to storm-induced velocity wave action. Base flood \
                 elevations determined."
            }
            "X" => {
                "Zone X - Minimal risk area outside the 1% annual chance floodplain. \
                 No mandatory flood insurance purchase requirement."
            }
            "B" | "X500" => {
                "Zone B / X500 - Moderate risk area within the 0.2% annual chance (500-year) \
                 floodplain. Also used for areas within the 1% annual chance floodplain with \
                 average depths of less than 1 foot or drainage areas less than 1 square mile."
            }
            "C" => {
                "Zone C - Minimal risk area outside the 0.2% annual chance floodplain. \
                 No mandatory flood insurance purchase requirement."
            }
            "D" => {
                "Zone D - Undetermined risk area. No analysis of flood hazards has been conducted. \
                 Flood insurance is available but not required."
            }
            _ => "Unknown flood zone designation.",
        }
    }

    /// Returns `true` when the zone code corresponds to a Special Flood Hazard
    /// Area (SFHA) or Coastal High Hazard Area.
    fn is_high_risk_zone(zone: &str) -> bool {
        matches!(
            zone.to_uppercase().as_str(),
            "A" | "AE" | "AH" | "AO" | "AR" | "A99" | "V" | "VE"
        )
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    /// Fetch and parse a list of disaster declarations from a FEMA OpenAPI URL.
    async fn fetch_disasters(&self, url: &str) -> Result<Vec<FemaDisaster>, String> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("FEMA request failed: {}", e))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse FEMA JSON: {}", e))?;

        let summaries = body
            .get("DisasterDeclarationsSummaries")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let disasters: Vec<FemaDisaster> = summaries
            .iter()
            .map(|item| self.parse_disaster(item))
            .collect();

        Ok(disasters)
    }

    /// Map a single JSON object from FEMA's response to a `FemaDisaster`.
    ///
    /// The resulting struct matches the database schema in models.rs; the `id`
    /// field is set to 0 because it will be assigned by SQLite on INSERT.
    fn parse_disaster(&self, item: &Value) -> FemaDisaster {
        let declaration_type = get_string(item, "declarationType");
        let ih = get_bool(item, "ihProgramDeclared");
        let ia = get_bool(item, "iaProgramDeclared");
        let pa = get_bool(item, "paProgramDeclared");
        let hm = get_bool(item, "hmProgramDeclared");

        // Build a human-readable "programs available" string.
        let mut programs = Vec::new();
        if ih || ia {
            programs.push("Individual Assistance");
        }
        if pa {
            programs.push("Public Assistance");
        }
        if hm {
            programs.push("Hazard Mitigation");
        }
        let programs_str = programs.join(", ");

        let disaster_number_raw = item
            .get("disasterNumber")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let fema_url = if disaster_number_raw > 0 {
            Some(format!(
                "https://www.fema.gov/disaster/{}",
                disaster_number_raw
            ))
        } else {
            None
        };

        FemaDisaster {
            id: 0, // assigned by DB on INSERT
            disaster_number: disaster_number_raw.to_string(),
            title: get_string(item, "declarationTitle"),
            disaster_type: declaration_type,
            declaration_date: parse_iso_datetime(item, "declarationDate"),
            incident_begin_date: parse_iso_datetime(item, "incidentBeginDate"),
            incident_end_date: parse_iso_datetime(item, "incidentEndDate"),
            closeout_date: parse_iso_datetime(item, "closeoutDate"),
            state: get_string(item, "state"),
            declared_county: item
                .get("designatedArea")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            incident_type: item
                .get("incidentType")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            programs_available: if programs_str.is_empty() {
                None
            } else {
                Some(programs_str)
            },
            individual_assistance: ih || ia,
            public_assistance: pa,
            hazard_mitigation: hm,
            fema_url,
        }
    }
}

// ── Free-standing helpers ────────────────────────────────────────────────────

fn get_string(val: &Value, key: &str) -> String {
    val.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn get_bool(val: &Value, key: &str) -> bool {
    val.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

/// Parse an ISO-8601 datetime string (e.g. `"2024-10-09T04:00:00.000Z"`) into
/// a `chrono::NaiveDateTime`. Returns `None` when the field is missing, null,
/// or unparseable.
fn parse_iso_datetime(val: &Value, key: &str) -> Option<NaiveDateTime> {
    let s = val.get(key)?.as_str()?;

    // Try the full ISO-8601 with fractional seconds.
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.fZ") {
        return Some(dt);
    }
    // Without fractional seconds.
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ") {
        return Some(dt);
    }
    // Date-only (midnight).
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(d.and_hms_opt(0, 0, 0).unwrap());
    }
    None
}
