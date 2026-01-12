//! JWT Authentication Module
//!
//! Provides token creation, validation, and middleware for protected routes.

use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

/// JWT Claims payload
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (agent ID or user ID)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Roles/permissions
    pub roles: Vec<String>,
    /// Namespace (for multi-tenancy)
    pub namespace: Option<String>,
}

/// JWT configuration
#[derive(Clone)]
pub struct JwtConfig {
    /// Secret key for signing tokens
    pub secret: String,
    /// Token expiration in hours
    pub expiration_hours: i64,
    /// Issuer name
    pub issuer: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "agentkern-dev-secret-change-in-production".to_string()),
            expiration_hours: 24,
            issuer: "agentkern".to_string(),
        }
    }
}

/// Create a new JWT token
pub fn create_token(
    config: &JwtConfig,
    subject: &str,
    roles: Vec<String>,
    namespace: Option<String>,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::hours(config.expiration_hours);

    let claims = Claims {
        sub: subject.to_string(),
        iss: config.issuer.clone(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        roles,
        namespace,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
}

/// Validate a JWT token and extract claims
pub fn validate_token(
    config: &JwtConfig,
    token: &str,
) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.set_issuer(&[&config.issuer]);

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
}

/// Public routes that don't require authentication
const PUBLIC_ROUTES: &[&str] = &[
    "/health",
    "/api/v1/identity/health",
    "/api/v1/gate/health",
    "/api/v1/arbiter/health",
    "/api/v1/nexus/health",
    "/api/v1/synapse/health",
    "/api/v1/treasury/health",
    "/api/v1/identity/verify",
    "/api/v1/auth/login",
    "/api/v1/auth/token",
];

/// Authentication middleware
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();

    // Allow public routes
    if PUBLIC_ROUTES.iter().any(|r| path.starts_with(r)) {
        return Ok(next.run(request).await);
    }

    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = &auth[7..];

            match validate_token(&state.jwt_config, token) {
                Ok(token_data) => {
                    tracing::debug!(
                        subject = %token_data.claims.sub,
                        "Authenticated request to {}",
                        path
                    );
                    // Inject claims into request extensions for handlers to use
                    request.extensions_mut().insert(token_data.claims);
                    Ok(next.run(request).await)
                }
                Err(e) => {
                    tracing::warn!("Invalid token for {}: {}", path, e);
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
        }
        Some(_) => {
            tracing::warn!("Invalid auth scheme for {}", path);
            Err(StatusCode::UNAUTHORIZED)
        }
        None => {
            tracing::warn!("Missing Authorization header for {}", path);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

// ============================================================================
// Auth API Endpoints
// ============================================================================

/// Login request
#[derive(Deserialize)]
pub struct LoginRequest {
    pub agent_id: String,
    pub secret: String,
}

/// Token response
#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_in: i64,
    pub token_type: String,
}

/// Login endpoint - exchange credentials for JWT
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Validate agent_id and secret against database
    // For now, accept any non-empty credentials (development mode)
    if payload.agent_id.is_empty() || payload.secret.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing credentials" })),
        ));
    }

    // Check if agent exists in database
    if let Some(ref pool) = state.pool {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM agent_records WHERE id = $1"
        )
        .bind(&payload.agent_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        if exists == 0 {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Agent not found" })),
            ));
        }
    }

    // Create token
    match create_token(
        &state.jwt_config,
        &payload.agent_id,
        vec!["agent".to_string()],
        None,
    ) {
        Ok(token) => Ok(Json(TokenResponse {
            token,
            expires_in: state.jwt_config.expiration_hours * 3600,
            token_type: "Bearer".to_string(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Refresh token endpoint
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    claims: Option<axum::Extension<Claims>>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<serde_json::Value>)> {
    let claims = claims.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "No valid token" })),
    ))?;

    // Create new token with same claims
    match create_token(
        &state.jwt_config,
        &claims.sub,
        claims.roles.clone(),
        claims.namespace.clone(),
    ) {
        Ok(token) => Ok(Json(TokenResponse {
            token,
            expires_in: state.jwt_config.expiration_hours * 3600,
            token_type: "Bearer".to_string(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Get current user info from token
pub async fn me(
    claims: Option<axum::Extension<Claims>>,
) -> Result<Json<Claims>, StatusCode> {
    claims
        .map(|c| Json(c.0))
        .ok_or(StatusCode::UNAUTHORIZED)
}
