use crate::models::*;
use reqwest;
use serde_json::Value;
use tracing::warn;

/// Client for the NOAA / National Weather Service alerts API.
/// Requires a `User-Agent` header per NWS API policy.
pub struct NoaaStormClient {
    client: reqwest::Client,
}

impl NoaaStormClient {
    /// Create a new client. The underlying reqwest::Client is configured with a
    /// `User-Agent` header as required by the NWS API.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("RoofingLeadGenerator/1.0 (contact@example.com)")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client }
    }

    /// Fetch active weather alerts for a geographic point from the NWS API.
    ///
    /// Endpoint: `https://api.weather.gov/alerts/active?point={lat},{lon}&status=actual`
    ///
    /// On failure the method logs a warning and returns an empty vector.
    pub async fn get_active_alerts(&self, lat: f64, lon: f64) -> Vec<WeatherAlert> {
        let url = format!(
            "https://api.weather.gov/alerts/active?point={:.4},{:.4}&status=actual",
            lat, lon
        );

        match self.fetch_alerts(&url).await {
            Ok(alerts) => alerts,
            Err(e) => {
                warn!(
                    "Failed to fetch NOAA alerts for ({}, {}): {}",
                    lat, lon, e
                );
                Vec::new()
            }
        }
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    async fn fetch_alerts(&self, url: &str) -> Result<Vec<WeatherAlert>, String> {
        let resp = self
            .client
            .get(url)
            .header("Accept", "application/geo+json")
            .send()
            .await
            .map_err(|e| format!("NOAA request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("NOAA API returned status {}", resp.status()));
        }

        let body: Value = resp
            .json()
            .await
            .map_err(|e| format!("NOAA JSON parse failed: {}", e))?;

        let features = body
            .get("features")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let alerts: Vec<WeatherAlert> = features
            .iter()
            .filter_map(|feat| {
                let props = feat.get("properties")?;
                Some(self.parse_alert(props))
            })
            .collect();

        Ok(alerts)
    }

    fn parse_alert(&self, props: &Value) -> WeatherAlert {
        WeatherAlert {
            id: get_string(props, "id"),
            event: get_string(props, "event"),
            severity: get_string(props, "severity"),
            urgency: get_string(props, "urgency"),
            headline: get_string(props, "headline"),
            description: get_string(props, "description"),
            effective: props
                .get("effective")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            expires: props
                .get("expires")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
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
