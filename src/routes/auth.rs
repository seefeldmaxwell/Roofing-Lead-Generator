use axum::{
    extract::{Query, State},
    http::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::AppState;

// ── OAuth Configuration ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub microsoft_client_id: String,
    pub microsoft_client_secret: String,
    pub base_url: String,
}

impl OAuthConfig {
    pub fn from_env() -> Self {
        Self {
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
            microsoft_client_id: std::env::var("MICROSOFT_CLIENT_ID").unwrap_or_default(),
            microsoft_client_secret: std::env::var("MICROSOFT_CLIENT_SECRET").unwrap_or_default(),
            base_url: std::env::var("BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        }
    }
}

// ── OAuth Client Builders ───────────────────────────────────────────────────

fn build_google_client(config: &OAuthConfig) -> BasicClient {
    let auth_url =
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .expect("Invalid Google auth URL");
    let token_url =
        TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
            .expect("Invalid Google token URL");
    let redirect_url = RedirectUrl::new(format!(
        "{}/account/callback/google",
        config.base_url
    ))
    .expect("Invalid Google redirect URL");

    BasicClient::new(
        ClientId::new(config.google_client_id.clone()),
        Some(ClientSecret::new(config.google_client_secret.clone())),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(redirect_url)
}

fn build_microsoft_client(config: &OAuthConfig) -> BasicClient {
    let auth_url = AuthUrl::new(
        "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
    )
    .expect("Invalid Microsoft auth URL");
    let token_url = TokenUrl::new(
        "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
    )
    .expect("Invalid Microsoft token URL");
    let redirect_url = RedirectUrl::new(format!(
        "{}/account/callback/microsoft",
        config.base_url
    ))
    .expect("Invalid Microsoft redirect URL");

    BasicClient::new(
        ClientId::new(config.microsoft_client_id.clone()),
        Some(ClientSecret::new(config.microsoft_client_secret.clone())),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(redirect_url)
}

// ── Query / Form Params ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ExternalLoginParams {
    pub provider: String,
    #[serde(default)]
    pub return_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    pub code: String,
    pub state: String,
}

// ── User / Session helpers ──────────────────────────────────────────────────

/// User struct matching the migrations schema: id TEXT, email TEXT, full_name TEXT, etc.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub full_name: Option<String>,
    pub provider: String,
    pub created_at: Option<String>,
}

/// Find an existing user by email or create a new one.
async fn find_or_create_user(
    pool: &SqlitePool,
    email: &str,
    name: &str,
    provider: &str,
) -> Result<User, sqlx::Error> {
    let user_id = uuid_simple();

    sqlx::query(
        "INSERT OR IGNORE INTO users (id, email, full_name, provider, created_at) \
         VALUES (?, ?, ?, ?, datetime('now'))",
    )
    .bind(&user_id)
    .bind(email)
    .bind(name)
    .bind(provider)
    .execute(pool)
    .await?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, full_name, provider, created_at FROM users WHERE email = ?",
    )
    .bind(email)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Create a new session row and return the session id.
async fn create_session(pool: &SqlitePool, user_id: &str) -> Result<String, sqlx::Error> {
    let session_id = uuid_simple();

    sqlx::query(
        "INSERT INTO sessions (id, user_id, created_at, expires_at) \
         VALUES (?, ?, datetime('now'), datetime('now', '+30 days'))",
    )
    .bind(&session_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(session_id)
}

/// Minimal UUID-like random hex string.
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:032x}", nanos ^ 0xdeadbeefcafebabe_u128)
}

// ── Google / Microsoft user-info response ───────────────────────────────────

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    email: String,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MicrosoftUserInfo {
    #[serde(alias = "mail", alias = "userPrincipalName")]
    email: Option<String>,
    #[serde(alias = "displayName")]
    display_name: Option<String>,
}

// ── Login template ──────────────────────────────────────────────────────────

#[derive(askama::Template)]
#[template(path = "account/login.html")]
struct LoginTemplate {
    return_url: String,
    error: Option<String>,
}

// ── Route Handlers ──────────────────────────────────────────────────────────

/// GET /account/login — render the login page.
pub async fn login(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let error = params.get("error").map(|e| match e.as_str() {
        "csrf" => "Security token mismatch. Please try again.".to_string(),
        "token" => "Failed to authenticate with provider.".to_string(),
        "userinfo" => "Could not retrieve your profile info.".to_string(),
        "db" => "A database error occurred. Please try again.".to_string(),
        _ => "An unknown error occurred.".to_string(),
    });
    let template = LoginTemplate {
        return_url: "/app/dashboard".to_string(),
        error,
    };
    askama_axum::IntoResponse::into_response(template)
}

/// GET /account/external-login?provider=google&return_url=/app/dashboard
pub async fn external_login(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExternalLoginParams>,
    jar: CookieJar,
) -> impl IntoResponse {
    let client = match params.provider.as_str() {
        "google" => build_google_client(&state.oauth),
        "microsoft" => build_microsoft_client(&state.oauth),
        _ => {
            return (jar, Redirect::to("/account/login")).into_response();
        }
    };

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    let return_url = params.return_url.unwrap_or_else(|| "/app/dashboard".to_string());

    let cookie_value = format!("{}|{}", csrf_token.secret(), return_url);
    let cookie = Cookie::build(("oauth_state", cookie_value))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build();

    let jar = jar.add(cookie);

    (jar, Redirect::to(auth_url.as_str())).into_response()
}

/// GET /account/callback/google
pub async fn oauth_callback_google(
    State(state): State<Arc<AppState>>,
    Query(params): Query<OAuthCallbackParams>,
    jar: CookieJar,
) -> impl IntoResponse {
    let (expected_csrf, return_url) = extract_oauth_state(&jar);
    if expected_csrf.as_deref() != Some(&params.state) {
        return (jar, Redirect::to("/account/login?error=csrf")).into_response();
    }

    let client = build_google_client(&state.oauth);
    let http_client = reqwest::Client::new();

    let token_result = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await;

    let token = match token_result {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Google token exchange failed: {:?}", e);
            return (jar, Redirect::to("/account/login?error=token")).into_response();
        }
    };

    let user_info = http_client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .ok();

    let user_info: Option<GoogleUserInfo> = match user_info {
        Some(resp) => resp.json().await.ok(),
        None => None,
    };

    let (email, name) = match user_info {
        Some(info) => (info.email, info.name.unwrap_or_default()),
        None => {
            return (jar, Redirect::to("/account/login?error=userinfo")).into_response();
        }
    };

    match create_user_session(&state.db, &email, &name, "google").await {
        Ok(session_cookie) => {
            let jar = jar.remove(Cookie::from("oauth_state"));
            let jar = jar.add(session_cookie);
            (
                jar,
                Redirect::to(&return_url.unwrap_or_else(|| "/app/dashboard".to_string())),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("DB error during Google OAuth: {:?}", e);
            (jar, Redirect::to("/account/login?error=db")).into_response()
        }
    }
}

/// GET /account/callback/microsoft
pub async fn oauth_callback_microsoft(
    State(state): State<Arc<AppState>>,
    Query(params): Query<OAuthCallbackParams>,
    jar: CookieJar,
) -> impl IntoResponse {
    let (expected_csrf, return_url) = extract_oauth_state(&jar);
    if expected_csrf.as_deref() != Some(&params.state) {
        return (jar, Redirect::to("/account/login?error=csrf")).into_response();
    }

    let client = build_microsoft_client(&state.oauth);
    let http_client = reqwest::Client::new();

    let token_result = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await;

    let token = match token_result {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Microsoft token exchange failed: {:?}", e);
            return (jar, Redirect::to("/account/login?error=token")).into_response();
        }
    };

    let user_info = http_client
        .get("https://graph.microsoft.com/v1.0/me")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .ok();

    let user_info: Option<MicrosoftUserInfo> = match user_info {
        Some(resp) => resp.json().await.ok(),
        None => None,
    };

    let (email, name) = match user_info {
        Some(info) => {
            let email = info.email.unwrap_or_default();
            let name = info.display_name.unwrap_or_default();
            if email.is_empty() {
                return (jar, Redirect::to("/account/login?error=userinfo")).into_response();
            }
            (email, name)
        }
        None => {
            return (jar, Redirect::to("/account/login?error=userinfo")).into_response();
        }
    };

    match create_user_session(&state.db, &email, &name, "microsoft").await {
        Ok(session_cookie) => {
            let jar = jar.remove(Cookie::from("oauth_state"));
            let jar = jar.add(session_cookie);
            (
                jar,
                Redirect::to(&return_url.unwrap_or_else(|| "/app/dashboard".to_string())),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("DB error during Microsoft OAuth: {:?}", e);
            (jar, Redirect::to("/account/login?error=db")).into_response()
        }
    }
}

/// GET /account/dummy-login — bypass OAuth with a demo account (dev only)
pub async fn dummy_login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> impl IntoResponse {
    match create_user_session(&state.db, "demo@test.com", "Demo User", "dummy").await {
        Ok(session_cookie) => {
            let jar = jar.add(session_cookie);
            (jar, Redirect::to("/app/dashboard")).into_response()
        }
        Err(e) => {
            tracing::error!("DB error during dummy login: {:?}", e);
            (jar, Redirect::to("/account/login?error=db")).into_response()
        }
    }
}

/// POST /account/logout
pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let jar = jar.remove(Cookie::from("session_id"));
    (jar, Redirect::to("/"))
}

// ── Internal helpers ────────────────────────────────────────────────────────

fn extract_oauth_state(jar: &CookieJar) -> (Option<String>, Option<String>) {
    match jar.get("oauth_state") {
        Some(cookie) => {
            let parts: Vec<&str> = cookie.value().splitn(2, '|').collect();
            let csrf = parts.first().map(|s| s.to_string());
            let return_url = parts.get(1).map(|s| s.to_string());
            (csrf, return_url)
        }
        None => (None, None),
    }
}

/// Shared logic: persist user, create session, return session cookie.
async fn create_user_session(
    pool: &SqlitePool,
    email: &str,
    name: &str,
    provider: &str,
) -> Result<Cookie<'static>, sqlx::Error> {
    let user = find_or_create_user(pool, email, name, provider).await?;
    let session_id = create_session(pool, &user.id).await?;

    let cookie = Cookie::build(("session_id", session_id))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build();

    Ok(cookie)
}

// ── Authentication Middleware ───────────────────────────────────────────────

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let session_id = match jar.get("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return Redirect::to("/account/login").into_response();
        }
    };

    let user = sqlx::query_as::<_, User>(
        "SELECT u.id, u.email, u.full_name, u.provider, u.created_at \
         FROM sessions s \
         JOIN users u ON u.id = s.user_id \
         WHERE s.id = ? AND s.expires_at > datetime('now')",
    )
    .bind(&session_id)
    .fetch_optional(&state.db)
    .await;

    match user {
        Ok(Some(user)) => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        _ => Redirect::to("/account/login").into_response(),
    }
}
