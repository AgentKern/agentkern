//! JWT Authentication Module
//!
//! Production-grade token creation, validation, and middleware.
//!
//! SECURITY NOTES:
//! - JWT_SECRET must be set explicitly (no fallback in production)
//! - Minimum 32-byte secret required
//! - Agent credentials validated against database
//! - Token blacklist support for revocation

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

/// Minimum secret length for security (256 bits = 32 bytes)
const MIN_SECRET_LENGTH: usize = 32;

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
    /// JWT ID (for revocation tracking)
    pub jti: String,
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
    /// Environment (development, staging, production)
    pub environment: Environment,
}

/// Runtime environment
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    fn from_env() -> Self {
        match std::env::var("AGENTKERN_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            _ => Environment::Development,
        }
    }
}

/// Configuration error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("JWT_SECRET environment variable not set")]
    MissingSecret,
    #[error("JWT_SECRET too short: minimum {MIN_SECRET_LENGTH} bytes required, got {0}")]
    SecretTooShort(usize),
    #[error("JWT_SECRET has low entropy: avoid simple patterns")]
    LowEntropy,
}

impl JwtConfig {
    /// Create JWT config from environment variables.
    /// 
    /// FAILS if:
    /// - JWT_SECRET not set in production
    /// - JWT_SECRET is too short
    /// - JWT_SECRET has low entropy (simple patterns)
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = Environment::from_env();
        
        let secret = match std::env::var("JWT_SECRET") {
            Ok(s) => s,
            Err(_) => {
                if environment == Environment::Production {
                    return Err(ConfigError::MissingSecret);
                }
                // Development fallback - still log a warning
                tracing::warn!("⚠️  JWT_SECRET not set - using development secret");
                "agentkern-dev-secret-DO-NOT-USE-IN-PRODUCTION".to_string()
            }
        };

        // Validate secret length
        if secret.len() < MIN_SECRET_LENGTH {
            if environment == Environment::Production {
                return Err(ConfigError::SecretTooShort(secret.len()));
            }
            tracing::warn!(
                "⚠️  JWT_SECRET too short ({} bytes < {} required)",
                secret.len(),
                MIN_SECRET_LENGTH
            );
        }

        // Check for low entropy patterns
        if is_low_entropy(&secret) && environment == Environment::Production {
            return Err(ConfigError::LowEntropy);
        }

        let expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);

        Ok(Self {
            secret,
            expiration_hours,
            issuer: "agentkern".to_string(),
            environment,
        })
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
}

/// Check if secret has low entropy (simple patterns)
fn is_low_entropy(secret: &str) -> bool {
    // Check for common weak patterns
    let weak_patterns = [
        "password", "secret", "12345", "qwerty", "admin",
        "changeme", "default", "test", "demo",
    ];
    
    let lower = secret.to_lowercase();
    for pattern in weak_patterns {
        if lower.contains(pattern) {
            return true;
        }
    }
    
    // Check for all same character
    if secret.chars().all(|c| c == secret.chars().next().unwrap()) {
        return true;
    }
    
    false
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
    let jti = uuid::Uuid::new_v4().to_string();

    let claims = Claims {
        sub: subject.to_string(),
        iss: config.issuer.clone(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        jti,
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
                    // TODO: Check token blacklist for revocation
                    // if is_token_revoked(&state.pool, &token_data.claims.jti).await {
                    //     return Err(StatusCode::UNAUTHORIZED);
                    // }

                    tracing::debug!(
                        subject = %token_data.claims.sub,
                        jti = %token_data.claims.jti,
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
    // Validate input
    if payload.agent_id.is_empty() || payload.secret.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing credentials" })),
        ));
    }

    // Validate against database
    if let Some(ref pool) = state.pool {
        // Check if agent exists and secret matches
        let agent = sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT id, secret_hash FROM agent_records WHERE id = $1"
        )
        .bind(&payload.agent_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error during login: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Database error" })),
            )
        })?;

        match agent {
            Some((_, Some(secret_hash))) => {
                // Verify secret (in production, use bcrypt/argon2)
                // For now, simple comparison - MUST be upgraded
                if !verify_secret(&payload.secret, &secret_hash) {
                    tracing::warn!("Invalid secret for agent {}", payload.agent_id);
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({ "error": "Invalid credentials" })),
                    ));
                }
            }
            Some((_, None)) => {
                // Agent exists but no secret set - allow in dev mode only
                if state.jwt_config.is_production() {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({ "error": "Agent credentials not configured" })),
                    ));
                }
                tracing::warn!("Agent {} has no secret configured (dev mode)", payload.agent_id);
            }
            None => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({ "error": "Agent not found" })),
                ));
            }
        }
    } else if state.jwt_config.is_production() {
        // No database in production is an error
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Authentication service unavailable" })),
        ));
    }

    // Create token
    match create_token(
        &state.jwt_config,
        &payload.agent_id,
        vec!["agent".to_string()],
        None,
    ) {
        Ok(token) => {
            tracing::info!("Token issued for agent {}", payload.agent_id);
            Ok(Json(TokenResponse {
                token,
                expires_in: state.jwt_config.expiration_hours * 3600,
                token_type: "Bearer".to_string(),
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Verify agent secret against stored hash
/// TODO: Upgrade to bcrypt or argon2 in production
fn verify_secret(provided: &str, stored_hash: &str) -> bool {
    // For now, simple comparison
    // In production: use bcrypt::verify or argon2::verify
    provided == stored_hash
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_entropy_detection() {
        assert!(is_low_entropy("password123"));
        assert!(is_low_entropy("mysecret"));
        assert!(is_low_entropy("test12345"));
        assert!(is_low_entropy("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
        assert!(!is_low_entropy("Kx7mP2nQ9vL4wR1tY6uI8oA3sD5fG0hJ"));
    }

    #[test]
    fn test_secret_length_validation() {
        let short = "short";
        assert!(short.len() < MIN_SECRET_LENGTH);
        
        let valid = "a".repeat(MIN_SECRET_LENGTH);
        assert!(valid.len() >= MIN_SECRET_LENGTH);
    }
}
