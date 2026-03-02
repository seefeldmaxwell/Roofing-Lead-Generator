mod db;
mod models;
mod routes;
mod services;

use axum::{
    middleware,
    routing::get,
    Router,
};
use routes::auth::OAuthConfig;
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct AppState {
    pub db: SqlitePool,
    pub oauth: OAuthConfig,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "roofing_lead_gen=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = db::create_pool().await;
    db::run_migrations(&pool).await;

    let fema_client = services::fema_client::FemaApiClient::new();
    db::seed_fema_data(&pool, &fema_client).await;
    db::seed_sample_data(&pool).await;

    let oauth = OAuthConfig::from_env();
    let state = Arc::new(AppState { db: pool, oauth });

    // Protected app routes (require auth) — nested under /app
    let app_routes = Router::new()
        .route("/dashboard", get(routes::app::dashboard))
        .route("/search", get(routes::app::search_page))
        .route("/property/{id}", get(routes::app::property_detail))
        .route("/permits", get(routes::app::permits_page))
        .route("/disasters", get(routes::app::disasters_page))
        .route("/risk", get(routes::app::risk_page))
        .route("/api-status", get(routes::app::api_status_page))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::auth::require_auth,
        ));

    // API routes
    let api_routes = Router::new()
        .route("/properties/search", get(routes::api::search_properties))
        .route("/properties/stats", get(routes::api::get_stats))
        .route("/properties/counties", get(routes::api::get_counties))
        .route("/properties/cities", get(routes::api::get_cities))
        .route("/properties/{id}", get(routes::api::get_property))
        .route("/properties/{id}/permits", get(routes::api::get_property_permits));

    // Public routes
    let public_routes = Router::new()
        .route("/", get(routes::home::index))
        .route("/counties", get(routes::home::counties))
        .route("/florida/{county_slug}", get(routes::home::county))
        .route("/florida/{county_slug}/{city_slug}", get(routes::home::city));

    // Auth routes
    let auth_routes = Router::new()
        .route("/account/login", get(routes::auth::login))
        .route("/account/register", get(routes::auth::login))
        .route("/account/external-login", get(routes::auth::external_login))
        .route("/account/callback/google", get(routes::auth::oauth_callback_google))
        .route("/account/callback/microsoft", get(routes::auth::oauth_callback_microsoft))
        .route("/account/logout", get(routes::auth::logout))
        .route("/account/dummy-login", get(routes::auth::dummy_login));

    let app = Router::new()
        .nest("/app", app_routes)
        .nest("/api", api_routes)
        .merge(public_routes)
        .merge(auth_routes)
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    tracing::info!("Starting server on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
