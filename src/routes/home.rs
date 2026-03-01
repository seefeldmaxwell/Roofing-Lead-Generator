use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::models::CountyStats;
use crate::services::landing_page_service::{LandingPageService, FLORIDA_COUNTIES, FLORIDA_CITIES};
use crate::AppState;

// ── Askama Templates ────────────────────────────────────────────────────────

/// Landing page: global stats + list of all Florida counties.
#[derive(Template)]
#[template(path = "home/index.html")]
struct IndexTemplate {
    total_properties: i64,
    total_permits: i64,
    total_disasters: i64,
    counties_tracked: i64,
    /// Sorted vec of (slug, display_name) for every county.
    counties: Vec<(String, String)>,
}

/// Per-county page: county stats + list of cities in the county.
#[derive(Template)]
#[template(path = "home/county.html")]
struct CountyTemplate {
    county_slug: String,
    county_name: String,
    stats: CountyStats,
    /// (city_slug, city_name) pairs for this county.
    cities: Vec<(String, String)>,
}

/// Per-city page: city-level stats within a county.
#[derive(Template)]
#[template(path = "home/city.html")]
struct CityTemplate {
    county_slug: String,
    county_name: String,
    city_slug: String,
    city_name: String,
    stats: CountyStats,
}

/// Full directory of all counties and their cities.
#[derive(Template)]
#[template(path = "home/counties.html")]
struct CountiesTemplate {
    /// (county_slug, county_name, Vec<(city_slug, city_name)>)
    counties: Vec<(String, String, Vec<(String, String)>)>,
}

// ── Route Handlers ──────────────────────────────────────────────────────────

/// GET / — public landing page with global statistics and a list of counties.
pub async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let service = LandingPageService::new(state.db.clone());

    let (total_properties, total_permits, total_disasters, counties_tracked) =
        service.get_global_stats().await;

    let mut counties: Vec<(String, String)> = FLORIDA_COUNTIES
        .iter()
        .map(|(slug, name)| (slug.to_string(), name.to_string()))
        .collect();
    counties.sort_by(|a, b| a.1.cmp(&b.1));

    let template = IndexTemplate {
        total_properties,
        total_permits,
        total_disasters,
        counties_tracked,
        counties,
    };

    askama_axum::IntoResponse::into_response(template)
}

/// GET /florida/:county_slug — stats and cities for a single Florida county.
pub async fn county(
    State(state): State<Arc<AppState>>,
    Path(county_slug): Path<String>,
) -> impl IntoResponse {
    let county_name = match LandingPageService::county_name_from_slug(&county_slug) {
        Some(n) => n.to_string(),
        None => {
            return (StatusCode::NOT_FOUND, "County not found").into_response();
        }
    };

    let service = LandingPageService::new(state.db.clone());
    let stats = service.get_county_stats(&county_name).await;

    // Collect cities belonging to this county.
    let mut cities: Vec<(String, String)> = FLORIDA_CITIES
        .iter()
        .filter(|(_, (_, cn))| *cn == county_name)
        .map(|(slug, (city_name, _))| (slug.to_string(), city_name.to_string()))
        .collect();
    cities.sort_by(|a, b| a.1.cmp(&b.1));

    let template = CountyTemplate {
        county_slug,
        county_name,
        stats,
        cities,
    };

    askama_axum::IntoResponse::into_response(template)
}

/// GET /florida/:county_slug/:city_slug — stats for a specific city.
pub async fn city(
    State(state): State<Arc<AppState>>,
    Path((county_slug, city_slug)): Path<(String, String)>,
) -> impl IntoResponse {
    let county_name = match LandingPageService::county_name_from_slug(&county_slug) {
        Some(n) => n.to_string(),
        None => {
            return (StatusCode::NOT_FOUND, "County not found").into_response();
        }
    };

    let (city_name, _) = match LandingPageService::city_info_from_slug(&city_slug) {
        Some(info) => info,
        None => {
            return (StatusCode::NOT_FOUND, "City not found").into_response();
        }
    };

    let service = LandingPageService::new(state.db.clone());
    let stats = service.get_county_stats(&county_name).await;

    let template = CityTemplate {
        county_slug,
        county_name,
        city_slug,
        city_name: city_name.to_string(),
        stats,
    };

    askama_axum::IntoResponse::into_response(template)
}

/// GET /counties — directory of all counties with their cities.
pub async fn counties(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut county_map: std::collections::BTreeMap<String, (String, Vec<(String, String)>)> =
        std::collections::BTreeMap::new();

    // Seed every county (so counties with zero cities still appear).
    for (slug, name) in FLORIDA_COUNTIES.iter() {
        county_map
            .entry(slug.to_string())
            .or_insert_with(|| (name.to_string(), Vec::new()));
    }

    // Attach cities to their counties.
    for (city_slug, (city_name, county_name)) in FLORIDA_CITIES.iter() {
        // Find the county slug for this county name
        for (cslug, cname) in FLORIDA_COUNTIES.iter() {
            if *cname == *county_name {
                if let Some(entry) = county_map.get_mut(*cslug) {
                    entry.1.push((city_slug.to_string(), city_name.to_string()));
                }
                break;
            }
        }
    }

    // Sort cities within each county.
    for (_, cities) in county_map.values_mut() {
        cities.sort_by(|a, b| a.1.cmp(&b.1));
    }

    let counties: Vec<(String, String, Vec<(String, String)>)> = county_map
        .into_iter()
        .map(|(slug, (name, cities))| (slug, name, cities))
        .collect();

    let template = CountiesTemplate { counties };

    askama_axum::IntoResponse::into_response(template)
}
