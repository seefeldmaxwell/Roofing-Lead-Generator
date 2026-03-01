use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::Row;
use std::sync::Arc;

use crate::models::{
    FemaDisaster, FireIncident, InsuranceClaim, Owner, Permit, Property,
};
use crate::routes::api::{fetch_dashboard_stats, DashboardStats};
use crate::AppState;

// ── Askama Templates ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardPageTemplate {
    stats: DashboardStats,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchPageTemplate {
    counties: Vec<String>,
    query: String,
    mode: String,
    selected_county: String,
    min_roof_age_str: String,
    max_roof_age_str: String,
    results: Option<SearchResults>,
    has_searched: bool,
}

struct SearchResults {
    items: Vec<SearchResultItem>,
    total_count: i64,
    page: i64,
    page_size: i64,
}

impl SearchResults {
    fn total_pages(&self) -> i64 {
        if self.page_size <= 0 { return 0; }
        (self.total_count + self.page_size - 1) / self.page_size
    }
    fn has_previous_page(&self) -> bool { self.page > 1 }
    fn has_next_page(&self) -> bool { self.page < self.total_pages() }
}

struct SearchResultItem {
    property_id: i64,
    address: String,
    city: String,
    zip_code: String,
    owner_name: String,
    roof_age: Option<i64>,
    flood_zone: Option<String>,
    is_in_flood_zone: bool,
    is_in_high_risk_flood_zone: bool,
    fire_risk_level: Option<String>,
    overall_risk_score: Option<i64>,
    lead_priority: Option<String>,
}

#[derive(Template)]
#[template(path = "property_detail.html")]
struct PropertyDetailPageTemplate {
    property: Property,
    owner: Option<Owner>,
    permits: Vec<Permit>,
    claims: Vec<InsuranceClaim>,
    fire_incidents: Vec<FireIncident>,
}

#[derive(Template)]
#[template(path = "permits.html")]
struct PermitsPageTemplate {
    permits: Vec<Permit>,
}

#[derive(Template)]
#[template(path = "disasters.html")]
struct DisastersPageTemplate {
    disasters: Vec<FemaDisaster>,
}

#[derive(Template)]
#[template(path = "risk.html")]
struct RiskPageTemplate {
    stats: DashboardStats,
}

#[derive(Template)]
#[template(path = "api_status.html")]
struct ApiStatusPageTemplate;

// ── Query param structs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    pub q: Option<String>,
    pub mode: Option<String>,
    pub county: Option<String>,
    pub min_roof_age: Option<String>,
    pub max_roof_age: Option<String>,
    pub page: Option<i64>,
}

// ── Route Handlers ──────────────────────────────────────────────────────────

/// GET /app/dashboard
pub async fn dashboard(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let stats = fetch_dashboard_stats(&state).await;
    let template = DashboardPageTemplate { stats };
    askama_axum::IntoResponse::into_response(template)
}

/// GET /app/search
pub async fn search_page(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQueryParams>,
) -> impl IntoResponse {
    let counties = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT county FROM properties ORDER BY county",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let query = params.q.clone().unwrap_or_default();
    let mode = params.mode.clone().unwrap_or_else(|| "address".to_string());
    let selected_county = params.county.clone().unwrap_or_default();
    let min_roof_age_str = params.min_roof_age.clone().unwrap_or_default();
    let max_roof_age_str = params.max_roof_age.clone().unwrap_or_default();
    let has_searched = !query.is_empty();

    let results = if has_searched {
        let page = params.page.unwrap_or(1).max(1);
        let page_size: i64 = 25;
        let offset = (page - 1) * page_size;

        let mut conditions: Vec<String> = Vec::new();
        let mut bind_values: Vec<String> = Vec::new();

        if mode == "owner" {
            conditions.push(
                "(o.first_name LIKE ? OR o.last_name LIKE ? OR (o.first_name || ' ' || o.last_name) LIKE ?)"
                    .to_string(),
            );
            let like = format!("%{}%", query);
            bind_values.push(like.clone());
            bind_values.push(like.clone());
            bind_values.push(like);
        } else {
            conditions.push("(p.address LIKE ? OR p.city LIKE ? OR p.zip_code LIKE ?)".to_string());
            let like = format!("%{}%", query);
            bind_values.push(like.clone());
            bind_values.push(like.clone());
            bind_values.push(like);
        }

        if !selected_county.is_empty() {
            conditions.push("p.county = ?".to_string());
            bind_values.push(selected_county.clone());
        }

        let where_clause = format!("WHERE {}", conditions.join(" AND "));
        let join_clause = if mode == "owner" {
            "LEFT JOIN owners o ON o.id = p.owner_id"
        } else {
            "LEFT JOIN owners o ON o.id = p.owner_id"
        };

        let count_sql = format!(
            "SELECT COUNT(*) FROM properties p {} {}",
            join_clause, where_clause
        );
        let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val);
        }
        let total_count = count_q.fetch_one(&state.db).await.unwrap_or(0);

        let data_sql = format!(
            "SELECT p.id, p.address, p.city, p.zip_code, p.county, \
             COALESCE(o.first_name || ' ' || o.last_name, '') as owner_name, \
             p.roof_age, p.flood_zone, p.is_in_flood_zone, p.is_in_high_risk_flood_zone, \
             p.fire_risk_level, p.overall_risk_score, p.lead_priority \
             FROM properties p {} {} \
             ORDER BY p.id LIMIT ? OFFSET ?",
            join_clause, where_clause
        );
        let mut data_q = sqlx::query(&data_sql);
        for val in &bind_values {
            data_q = data_q.bind(val);
        }
        data_q = data_q.bind(page_size).bind(offset);

        let rows = data_q.fetch_all(&state.db).await.unwrap_or_default();
        let items: Vec<SearchResultItem> = rows
            .iter()
            .map(|r| SearchResultItem {
                property_id: r.get("id"),
                address: r.get("address"),
                city: r.get("city"),
                zip_code: r.get("zip_code"),
                owner_name: r.get("owner_name"),
                roof_age: r.get("roof_age"),
                flood_zone: r.get("flood_zone"),
                is_in_flood_zone: r.get::<bool, _>("is_in_flood_zone"),
                is_in_high_risk_flood_zone: r.get::<bool, _>("is_in_high_risk_flood_zone"),
                fire_risk_level: r.get("fire_risk_level"),
                overall_risk_score: r.get("overall_risk_score"),
                lead_priority: r.get("lead_priority"),
            })
            .collect();

        Some(SearchResults {
            items,
            total_count,
            page,
            page_size,
        })
    } else {
        None
    };

    let template = SearchPageTemplate {
        counties,
        query,
        mode,
        selected_county,
        min_roof_age_str,
        max_roof_age_str,
        results,
        has_searched,
    };
    askama_axum::IntoResponse::into_response(template)
}

/// GET /app/property/:id
pub async fn property_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    // Property
    let property = sqlx::query_as::<_, Property>("SELECT * FROM properties WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await;

    let property = match property {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Property not found").into_response();
        }
        Err(e) => {
            tracing::error!("DB error fetching property {}: {:?}", id, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response();
        }
    };

    // Owner
    let owner = if let Some(owner_id) = property.owner_id {
        sqlx::query_as::<_, Owner>("SELECT * FROM owners WHERE id = ?")
            .bind(owner_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
    } else {
        None
    };

    // Permits
    let permits = sqlx::query_as::<_, Permit>(
        "SELECT * FROM permits WHERE property_id = ? ORDER BY issued_date DESC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Insurance Claims
    let claims = sqlx::query_as::<_, InsuranceClaim>(
        "SELECT * FROM insurance_claims WHERE property_id = ? ORDER BY date_of_loss DESC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Fire Incidents
    let fire_incidents = sqlx::query_as::<_, FireIncident>(
        "SELECT * FROM fire_incidents WHERE property_id = ? ORDER BY discovery_date DESC LIMIT 20",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let template = PropertyDetailPageTemplate {
        property,
        owner,
        permits,
        claims,
        fire_incidents,
    };

    askama_axum::IntoResponse::into_response(template)
}

/// GET /app/permits
pub async fn permits_page(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let permits = sqlx::query_as::<_, Permit>(
        "SELECT * FROM permits \
         WHERE issued_date >= date('now', '-365 days') \
         ORDER BY issued_date DESC \
         LIMIT 500",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let template = PermitsPageTemplate { permits };
    askama_axum::IntoResponse::into_response(template)
}

/// GET /app/disasters
pub async fn disasters_page(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let disasters = sqlx::query_as::<_, FemaDisaster>(
        "SELECT * FROM fema_disasters \
         WHERE state = 'FL' OR state = 'Florida' \
         ORDER BY declaration_date DESC \
         LIMIT 200",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let template = DisastersPageTemplate { disasters };
    askama_axum::IntoResponse::into_response(template)
}

/// GET /app/risk
pub async fn risk_page(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let stats = fetch_dashboard_stats(&state).await;
    let template = RiskPageTemplate { stats };
    askama_axum::IntoResponse::into_response(template)
}

/// GET /app/api-status
pub async fn api_status_page() -> impl IntoResponse {
    let template = ApiStatusPageTemplate;
    askama_axum::IntoResponse::into_response(template)
}
