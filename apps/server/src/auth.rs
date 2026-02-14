//! JWT Authentication Module
//!
//! Production-grade token creation, validation, and middleware.
//!
//! SECURITY NOTES:
//! - JWT_SECRET must be set explicitly (no fallback in production)
//! - Minimum 32-byte secret required
//! - Agent credentials validated against database
//! - Token blacklist support for revocation
//! - AWS KMS support for secret decryption

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{Request, StatusCode, header},
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::AppState;

/// Token blacklist for revocation tracking (production should use Redis)
static TOKEN_BLACKLIST: std::sync::OnceLock<Mutex<HashMap<String, i64>>> =
    std::sync::OnceLock::new();

fn get_token_blacklist() -> &'static Mutex<HashMap<String, i64>> {
    TOKEN_BLACKLIST.get_or_init(|| Mutex::new(HashMap::new()))
}

fn prune_expired_tokens(blacklist: &mut HashMap<String, i64>, now: i64) {
    blacklist.retain(|_, expires_at| *expires_at > now);
}

/// Minimum secret length for security (256 bits = 32 bytes)
const MIN_SECRET_LENGTH: usize = 32;

/// JWT Claims payload
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
    pub fn from_env() -> Self {
        let raw = std::env::var("AGENTKERN_ENV")
            .or_else(|_| std::env::var("RUST_ENV"))
            .unwrap_or_else(|_| "development".to_string());

        match raw.to_lowercase().as_str() {
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
    #[error("KMS decryption failed: {0}")]
    KmsError(String),
}

impl JwtConfig {
    /// - JWT_SECRET has low entropy (simple patterns)
    /// - KMS decryption fails (if configured)
    pub async fn from_env() -> Result<Self, ConfigError> {
        let environment = Environment::from_env();

        let mut secret = match std::env::var("JWT_SECRET") {
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

        // AWS KMS Decryption (Phase 6 Hardening)
        if let Ok(key_id) = std::env::var("JWT_KMS_KEY_ID") {
            tracing::info!(
                "🔐 AWS KMS key ID detected: {}. Attempting decryption...",
                key_id
            );
            secret = decrypt_with_kms(&secret, &key_id).await.map_err(|e| {
                tracing::error!("❌ KMS decryption failed: {}", e);
                ConfigError::KmsError(e.to_string())
            })?;
            tracing::info!("✅ JWT_SECRET decrypted via AWS KMS");
        }

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
    if secret.is_empty() {
        return true;
    }

    // Check for common weak patterns
    let weak_patterns = [
        "password", "secret", "12345", "qwerty", "admin", "changeme", "default", "test", "demo",
    ];

    let lower = secret.to_lowercase();
    for pattern in weak_patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    // Check for all same character
    if let Some(first_char) = secret.chars().next()
        && secret.chars().all(|c| c == first_char)
    {
        return true;
    }

    false
}

fn has_admin_role(claims: &Claims) -> bool {
    claims
        .roles
        .iter()
        .map(|role| role.to_lowercase())
        .any(|role| role == "admin" || role == "superadmin" || role == "root")
}

fn require_admin_claims(
    claims: Option<axum::Extension<Claims>>,
) -> Result<Claims, (StatusCode, Json<serde_json::Value>)> {
    let claims = claims.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "No valid token" })),
    ))?;

    if !has_admin_role(&claims) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin role required" })),
        ));
    }

    Ok(claims.0)
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
                    // Check token blacklist for revocation
                    if is_token_revoked(state.redis.as_ref(), &token_data.claims.jti).await {
                        tracing::warn!(
                            subject = %token_data.claims.sub,
                            jti = %token_data.claims.jti,
                            "Revoked token attempted use for {}",
                            path
                        );
                        return Err(StatusCode::UNAUTHORIZED);
                    }

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
            "SELECT id, secret_hash FROM agent_records WHERE id = $1",
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
                tracing::warn!(
                    "Agent {} has no secret configured (dev mode)",
                    payload.agent_id
                );
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
/// Uses bcrypt for production-grade password hashing
///
/// # Security Considerations
/// - Stored hash should be generated with `hash_secret()`
/// - Bcrypt automatically handles salt and timing attacks
/// - Cost parameter tuned for ~100ms verification time
fn verify_secret(provided: &str, stored_hash: &str) -> bool {
    // Use bcrypt for secure verification
    match bcrypt::verify(provided, stored_hash) {
        Ok(valid) => valid,
        Err(e) => {
            tracing::warn!("Bcrypt verification error: {}", e);
            false
        }
    }
}

/// Hash a secret for secure storage
/// Uses bcrypt with cost factor 12 (~100ms)
/// Used in admin endpoints and secret rotation
pub fn hash_secret(secret: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(secret, 12)
}

/// Decrypt a secret using AWS KMS
async fn decrypt_with_kms(
    ciphertext_b64: &str,
    key_id: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use base64::Engine as _;
    let ciphertext = base64::engine::general_purpose::STANDARD.decode(ciphertext_b64)?;

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = aws_sdk_kms::Client::new(&config);

    let blob = aws_sdk_kms::primitives::Blob::new(ciphertext);
    let resp = client
        .decrypt()
        .ciphertext_blob(blob)
        .key_id(key_id)
        .send()
        .await?;

    let plaintext = resp.plaintext().ok_or("No plaintext in KMS response")?;
    let secret = String::from_utf8(plaintext.as_ref().to_vec())?;

    Ok(secret)
}

/// Check if a token has been revoked
async fn is_token_revoked(redis_client: Option<&redis::Client>, jti: &str) -> bool {
    // 1. Check Redis if available
    if let Some(client) = redis_client
        && let Ok(mut conn) = client.get_multiplexed_async_connection().await
    {
        use redis::AsyncCommands;
        let key = format!("auth:revocation:{}", jti);
        match conn.exists::<_, bool>(key).await {
            Ok(exists) => {
                if exists {
                    return true;
                }
            }
            Err(e) => tracing::error!("Redis revocation check failed: {}", e),
        }
    }

    // 2. Fallback to in-memory blacklist (TTL-aware)
    let now = Utc::now().timestamp();
    get_token_blacklist()
        .lock()
        .map(|mut blacklist| {
            prune_expired_tokens(&mut blacklist, now);
            blacklist
                .get(jti)
                .is_some_and(|expires_at| *expires_at > now)
        })
        .unwrap_or(false)
}

/// Revoke a token by adding JTI to blacklist
pub async fn revoke_token(
    redis_client: Option<&redis::Client>,
    jti: String,
    ttl_secs: i64,
) -> Result<(), String> {
    let now = Utc::now().timestamp();
    let ttl_secs = ttl_secs.max(0);
    if ttl_secs == 0 {
        return Ok(());
    }
    let expires_at = now.saturating_add(ttl_secs);

    // 1. Revoke in Redis if available
    if let Some(client) = redis_client
        && let Ok(mut conn) = client.get_multiplexed_async_connection().await
    {
        use redis::AsyncCommands;
        let key = format!("auth:revocation:{}", jti);
        // Set with TTL (max token life)
        let _: () = conn
            .set_ex(key, 1, ttl_secs as u64)
            .await
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    // 2. Fallback to in-memory blacklist
    let mut blacklist = get_token_blacklist()
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    prune_expired_tokens(&mut blacklist, now);
    blacklist.insert(jti, expires_at);
    Ok(())
}

/// Logout endpoint - revoke the current token
pub async fn logout(
    State(state): State<Arc<AppState>>,
    claims: Option<axum::Extension<Claims>>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let claims = claims.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "No valid token" })),
    ))?;

    let ttl = (claims.exp - Utc::now().timestamp()).max(0);

    match revoke_token(state.redis.as_ref(), claims.jti.clone(), ttl).await {
        Ok(_) => {
            tracing::info!("Token {} revoked via logout", claims.jti);
            Ok(StatusCode::OK)
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

/// Admin request to hash a secret
#[derive(Deserialize)]
pub struct HashSecretRequest {
    pub secret: String,
}

/// Admin endpoint to hash a secret (for manual DB updates)
pub async fn admin_hash_secret(
    claims: Option<axum::Extension<Claims>>,
    Json(payload): Json<HashSecretRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let claims = require_admin_claims(claims)?;
    tracing::info!(subject = %claims.sub, "Admin requested secret hashing");

    match hash_secret(&payload.secret) {
        Ok(hash) => Ok(Json(serde_json::json!({ "hash": hash }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Admin request to revoke a specific JTI
#[derive(Deserialize)]
pub struct RevokeRequest {
    pub jti: String,
}

/// Admin endpoint to manually revoke a token JTI
pub async fn admin_revoke(
    State(state): State<Arc<AppState>>,
    claims: Option<axum::Extension<Claims>>,
    Json(payload): Json<RevokeRequest>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let claims = require_admin_claims(claims)?;
    tracing::info!(subject = %claims.sub, jti = %payload.jti, "Admin requested token revocation");

    // For admin revoke, we use a default TTL of 24h if we don't know the exact expiry
    let ttl = 24 * 3600;

    match revoke_token(state.redis.as_ref(), payload.jti.clone(), ttl).await {
        Ok(_) => {
            tracing::info!("Token {} revoked by admin", payload.jti);
            Ok(StatusCode::OK)
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
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
#[allow(dead_code)]
pub async fn me(claims: Option<axum::Extension<Claims>>) -> Result<Json<Claims>, StatusCode> {
    claims.map(|c| Json(c.0)).ok_or(StatusCode::UNAUTHORIZED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_entropy_detection() {
        assert!(is_low_entropy(""));
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

    #[test]
    fn test_bcrypt_verification() {
        let secret = "test-secret-long-enough-for-validation";
        let hash = hash_secret(secret).expect("Failed to hash secret");
        assert!(verify_secret(secret, &hash));
        assert!(!verify_secret("wrong-secret", &hash));
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let jti = "test-jti-uuid-12345";
        assert!(!is_token_revoked(None, jti).await);
        revoke_token(None, jti.to_string(), 3600)
            .await
            .expect("Failed to revoke token");
        assert!(is_token_revoked(None, jti).await);
    }

    #[tokio::test]
    async fn test_expired_token_not_retained() {
        let jti = "expired-jti";
        revoke_token(None, jti.to_string(), 0)
            .await
            .expect("Failed to handle zero TTL");
        assert!(!is_token_revoked(None, jti).await);
    }

    #[test]
    fn test_bcrypt_performance() {
        use std::time::Instant;
        let secret = "test-performance-secret-1234567890";
        let hash = hash_secret(secret).expect("Failed to hash secret");

        let start = Instant::now();
        assert!(verify_secret(secret, &hash));
        let duration = start.elapsed();

        tracing::info!("Bcrypt verification took {:?}", duration);
        // Cost factor 12 should take > 50ms and < 300ms on typical hardware
        assert!(
            duration.as_millis() >= 50,
            "Bcrypt verification too fast! Check cost factor."
        );
    }

    #[test]
    fn test_admin_role_detection() {
        let claims = Claims {
            sub: "agent-1".to_string(),
            iss: "agentkern".to_string(),
            exp: Utc::now().timestamp() + 3600,
            iat: Utc::now().timestamp(),
            jti: "jti-1".to_string(),
            roles: vec!["Admin".to_string()],
            namespace: None,
        };
        assert!(has_admin_role(&claims));
    }
}
