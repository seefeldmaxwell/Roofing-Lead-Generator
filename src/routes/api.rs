use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;

use crate::AppState;

// ── Request / Response types ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
    pub mode: Option<String>,
    pub county: Option<String>,
    pub city: Option<String>,
    pub zip: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// Lightweight search result for the JSON API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSearchResult {
    pub id: i64,
    pub parcel_id: String,
    pub address: String,
    pub city: String,
    pub county: String,
    pub zip_code: String,
    pub year_built: Option<i64>,
    pub estimated_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiPaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

/// Dashboard statistics used by both the API and server-rendered pages.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardStats {
    pub total_properties: i64,
    pub total_permits: i64,
    pub total_disasters: i64,
    pub total_claims: i64,
    pub properties_in_flood_zone: i64,
    pub properties_high_risk_flood: i64,
    pub properties_high_fire_risk: i64,
    pub total_insurance_claims: i64,
    pub roofs_over_15_years: i64,
    pub counties_tracked: i64,
    pub cities_tracked: i64,
    pub county_breakdowns: Vec<CountyBreakdownRow>,
    pub roof_age_distributions: Vec<RoofAgeDistRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CountyBreakdownRow {
    pub county: String,
    pub property_count: i64,
    pub flood_zone_count: i64,
    pub high_fire_risk_count: i64,
    pub old_roof_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoofAgeDistRow {
    pub range: String,
    pub count: i64,
}

#[derive(Debug, Deserialize)]
pub struct CountyFilter {
    pub county: Option<String>,
}

// ── Route Handlers ──────────────────────────────────────────────────────────

/// GET /api/properties/search
pub async fn search_properties(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Json<ApiPaginatedResult<ApiSearchResult>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(25).clamp(1, 100);
    let offset = (page - 1) * page_size;
    let query = params.q.unwrap_or_default();

    let mut conditions: Vec<String> = Vec::new();
    let mut bind_values: Vec<String> = Vec::new();

    if !query.is_empty() {
        conditions.push("(p.address LIKE ? OR p.parcel_id LIKE ?)".to_string());
        let like = format!("%{}%", query);
        bind_values.push(like.clone());
        bind_values.push(like);
    }

    if let Some(ref city) = params.city {
        if !city.is_empty() {
            conditions.push("p.city = ?".to_string());
            bind_values.push(city.clone());
        }
    }
    if let Some(ref county) = params.county {
        if !county.is_empty() {
            conditions.push("p.county = ?".to_string());
            bind_values.push(county.clone());
        }
    }
    if let Some(ref zip) = params.zip {
        if !zip.is_empty() {
            conditions.push("p.zip_code = ?".to_string());
            bind_values.push(zip.clone());
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Count
    let count_sql = format!("SELECT COUNT(*) as cnt FROM properties p {}", where_clause);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for val in &bind_values {
        count_query = count_query.bind(val);
    }
    let total = count_query.fetch_one(&state.db).await.unwrap_or(0);

    // Data
    let data_sql = format!(
        "SELECT p.id, p.parcel_id, p.address, p.city, p.county, p.zip_code, \
         p.year_built, p.estimated_value \
         FROM properties p {} ORDER BY p.id LIMIT ? OFFSET ?",
        where_clause
    );

    let mut data_query = sqlx::query(&data_sql);
    for val in &bind_values {
        data_query = data_query.bind(val);
    }
    data_query = data_query.bind(page_size).bind(offset);

    let rows = data_query.fetch_all(&state.db).await.unwrap_or_default();

    let items: Vec<ApiSearchResult> = rows
        .iter()
        .map(|r| ApiSearchResult {
            id: r.get("id"),
            parcel_id: r.get("parcel_id"),
            address: r.get("address"),
            city: r.get("city"),
            county: r.get("county"),
            zip_code: r.get("zip_code"),
            year_built: r.get("year_built"),
            estimated_value: r.get("estimated_value"),
        })
        .collect();

    let total_pages = if total == 0 {
        0
    } else {
        (total + page_size - 1) / page_size
    };

    Json(ApiPaginatedResult {
        items,
        total,
        page,
        page_size,
        total_pages,
    })
}

/// GET /api/properties/:id
pub async fn get_property(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let row = sqlx::query("SELECT * FROM properties WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await;

    match row {
        Ok(Some(r)) => {
            let property = serde_json::json!({
                "id": r.get::<i64, _>("id"),
                "address": r.get::<String, _>("address"),
                "city": r.get::<String, _>("city"),
                "county": r.get::<String, _>("county"),
                "zip_code": r.get::<String, _>("zip_code"),
                "parcel_id": r.get::<String, _>("parcel_id"),
                "year_built": r.get::<Option<i64>, _>("year_built"),
                "estimated_value": r.get::<Option<f64>, _>("estimated_value"),
                "latitude": r.get::<Option<f64>, _>("latitude"),
                "longitude": r.get::<Option<f64>, _>("longitude"),
            });
            Json(property).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Property not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("DB error fetching property {}: {:?}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal error"})),
            )
                .into_response()
        }
    }
}

/// GET /api/properties/:id/permits
pub async fn get_property_permits(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let rows = sqlx::query(
        "SELECT id, property_id, permit_number, permit_type, work_category, \
         description, issued_date, status, contractor_name, estimated_cost, is_roofing_permit \
         FROM permits WHERE property_id = ? ORDER BY issued_date DESC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let permits: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<i64, _>("id"),
                "property_id": r.get::<i64, _>("property_id"),
                "permit_number": r.get::<String, _>("permit_number"),
                "permit_type": r.get::<String, _>("permit_type"),
                "work_category": r.get::<String, _>("work_category"),
                "description": r.get::<Option<String>, _>("description"),
                "issued_date": r.get::<Option<String>, _>("issued_date"),
                "status": r.get::<String, _>("status"),
                "contractor_name": r.get::<Option<String>, _>("contractor_name"),
                "estimated_cost": r.get::<Option<f64>, _>("estimated_cost"),
                "is_roofing_permit": r.get::<bool, _>("is_roofing_permit"),
            })
        })
        .collect();

    Json(permits)
}

/// GET /api/stats
pub async fn get_stats(State(state): State<Arc<AppState>>) -> Json<DashboardStats> {
    Json(fetch_dashboard_stats(&state).await)
}

/// GET /api/counties
pub async fn get_counties(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
    let counties = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT county FROM properties ORDER BY county",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(counties)
}

/// GET /api/cities?county=...
pub async fn get_cities(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CountyFilter>,
) -> Json<Vec<String>> {
    let cities = if let Some(county) = params.county {
        sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT city FROM properties WHERE county = ? ORDER BY city",
        )
        .bind(county)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT city FROM properties ORDER BY city",
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    };

    Json(cities)
}

// ── Shared helper ───────────────────────────────────────────────────────────

/// Fetch dashboard stats used by both the API and server-rendered pages.
pub async fn fetch_dashboard_stats(state: &AppState) -> DashboardStats {
    let total_properties = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM properties")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let total_permits = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM permits")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let total_disasters = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM fema_disasters")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let total_claims = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM insurance_claims")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let properties_in_flood_zone =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM properties WHERE is_in_flood_zone = 1")
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

    let properties_high_risk_flood = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM properties WHERE is_in_high_risk_flood_zone = 1",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let properties_high_fire_risk = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM properties WHERE fire_risk_level IN ('High', 'Extreme')",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let roofs_over_15_years = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM properties WHERE roof_age IS NOT NULL AND roof_age > 15",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let counties_tracked = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT county) FROM properties",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let cities_tracked =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT city) FROM properties")
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

    // County breakdowns
    let county_rows = sqlx::query(
        "SELECT county, \
         COUNT(*) as property_count, \
         COALESCE(SUM(CASE WHEN is_in_flood_zone = 1 THEN 1 ELSE 0 END), 0) as flood_zone_count, \
         COALESCE(SUM(CASE WHEN fire_risk_level IN ('High', 'Extreme') THEN 1 ELSE 0 END), 0) as high_fire_risk_count, \
         COALESCE(SUM(CASE WHEN roof_age > 15 THEN 1 ELSE 0 END), 0) as old_roof_count \
         FROM properties \
         WHERE county != '' \
         GROUP BY county \
         ORDER BY property_count DESC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let county_breakdowns: Vec<CountyBreakdownRow> = county_rows
        .iter()
        .map(|r| CountyBreakdownRow {
            county: r.get("county"),
            property_count: r.get("property_count"),
            flood_zone_count: r.get("flood_zone_count"),
            high_fire_risk_count: r.get("high_fire_risk_count"),
            old_roof_count: r.get("old_roof_count"),
        })
        .collect();

    // Roof age distribution
    let roof_age_distributions = vec![
        RoofAgeDistRow {
            range: "0-5 years".to_string(),
            count: sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM properties WHERE roof_age BETWEEN 0 AND 5",
            )
            .fetch_one(&state.db)
            .await
            .unwrap_or(0),
        },
        RoofAgeDistRow {
            range: "6-10 years".to_string(),
            count: sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM properties WHERE roof_age BETWEEN 6 AND 10",
            )
            .fetch_one(&state.db)
            .await
            .unwrap_or(0),
        },
        RoofAgeDistRow {
            range: "11-15 years".to_string(),
            count: sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM properties WHERE roof_age BETWEEN 11 AND 15",
            )
            .fetch_one(&state.db)
            .await
            .unwrap_or(0),
        },
        RoofAgeDistRow {
            range: "16-20 years".to_string(),
            count: sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM properties WHERE roof_age BETWEEN 16 AND 20",
            )
            .fetch_one(&state.db)
            .await
            .unwrap_or(0),
        },
        RoofAgeDistRow {
            range: "20+ years".to_string(),
            count: sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM properties WHERE roof_age > 20",
            )
            .fetch_one(&state.db)
            .await
            .unwrap_or(0),
        },
    ];

    DashboardStats {
        total_properties,
        total_permits,
        total_disasters,
        total_claims,
        properties_in_flood_zone,
        properties_high_risk_flood,
        properties_high_fire_risk,
        total_insurance_claims: total_claims,
        roofs_over_15_years,
        counties_tracked,
        cities_tracked,
        county_breakdowns,
        roof_age_distributions,
    }
}
