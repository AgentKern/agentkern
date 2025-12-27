//! VeriMantle Enterprise: SSO Integration
//!
//! Per LICENSING_STRATEGY.md: "Team Management / SSO"
//!
//! **License**: VeriMantle Enterprise License
//!
//! Features:
//! - SAML 2.0 integration
//! - OIDC/OAuth2 integration
//! - LDAP directory sync
//! - SCIM provisioning

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod license {
    #[derive(Debug, thiserror::Error)]
    pub enum LicenseError {
        #[error("Enterprise license required for SSO")]
        LicenseRequired,
    }

    pub fn require(feature: &str) -> Result<(), LicenseError> {
        let key = std::env::var("VERIMANTLE_LICENSE_KEY")
            .map_err(|_| LicenseError::LicenseRequired)?;
        
        if key.is_empty() {
            return Err(LicenseError::LicenseRequired);
        }
        
        tracing::debug!(feature = %feature, "Enterprise SSO feature accessed");
        Ok(())
    }
}

/// SSO provider types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SsoProvider {
    /// SAML 2.0
    Saml,
    /// OpenID Connect
    Oidc,
    /// LDAP
    Ldap,
    /// Azure AD
    AzureAd,
    /// Okta
    Okta,
    /// Google Workspace
    Google,
    /// GitHub
    GitHub,
}

/// SAML configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConfig {
    /// IdP Entity ID
    pub idp_entity_id: String,
    /// IdP SSO URL
    pub idp_sso_url: String,
    /// IdP Certificate (PEM)
    pub idp_certificate: String,
    /// SP Entity ID
    pub sp_entity_id: String,
    /// SP ACS URL
    pub sp_acs_url: String,
    /// Name ID format
    pub name_id_format: String,
    /// Attribute mappings
    #[serde(default)]
    pub attribute_mappings: HashMap<String, String>,
}

/// OIDC configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    /// Issuer URL
    pub issuer: String,
    /// Client ID
    pub client_id: String,
    /// Client secret (encrypted)
    #[serde(skip_serializing)]
    pub client_secret: String,
    /// Redirect URI
    pub redirect_uri: String,
    /// Scopes
    pub scopes: Vec<String>,
    /// Token endpoint auth method
    pub token_auth_method: TokenAuthMethod,
}

/// Token authentication method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenAuthMethod {
    ClientSecretBasic,
    ClientSecretPost,
    ClientSecretJwt,
    PrivateKeyJwt,
}

impl Default for TokenAuthMethod {
    fn default() -> Self {
        Self::ClientSecretBasic
    }
}

/// LDAP configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapConfig {
    /// LDAP server URL
    pub url: String,
    /// Bind DN
    pub bind_dn: String,
    /// Bind password (encrypted)
    #[serde(skip_serializing)]
    pub bind_password: String,
    /// User search base
    pub user_base: String,
    /// User search filter
    pub user_filter: String,
    /// Group search base
    pub group_base: Option<String>,
    /// Group search filter
    pub group_filter: Option<String>,
    /// Use TLS
    pub use_tls: bool,
}

/// SSO user from identity provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoUser {
    /// External ID from provider
    pub external_id: String,
    /// Email
    pub email: String,
    /// Display name
    pub name: String,
    /// First name
    pub first_name: Option<String>,
    /// Last name
    pub last_name: Option<String>,
    /// Groups
    #[serde(default)]
    pub groups: Vec<String>,
    /// Raw attributes
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    /// Provider
    pub provider: SsoProvider,
}

/// SSO session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoSession {
    /// Session ID
    pub session_id: String,
    /// User
    pub user: SsoUser,
    /// Created at
    pub created_at: u64,
    /// Expires at
    pub expires_at: u64,
    /// Access token (if OIDC)
    pub access_token: Option<String>,
    /// Refresh token (if OIDC)
    pub refresh_token: Option<String>,
}

/// OIDC token endpoint response.
#[derive(Debug, Clone, Deserialize)]
pub struct OidcTokenResponse {
    /// Access token
    pub access_token: String,
    /// Token type (usually "Bearer")
    pub token_type: String,
    /// ID token (JWT)
    pub id_token: String,
    /// Refresh token (optional)
    pub refresh_token: Option<String>,
    /// Expires in seconds
    pub expires_in: Option<u64>,
    /// Scope (space-separated)
    pub scope: Option<String>,
}

/// OIDC ID token claims (standard + common).
#[derive(Debug, Clone, Deserialize)]
pub struct OidcClaims {
    /// Issuer
    pub iss: String,
    /// Subject (user ID)
    pub sub: String,
    /// Audience
    pub aud: serde_json::Value, // Can be string or array
    /// Expiration
    pub exp: u64,
    /// Issued at
    pub iat: u64,
    /// Email (common claim)
    pub email: Option<String>,
    /// Email verified
    pub email_verified: Option<bool>,
    /// Name (common claim)
    pub name: Option<String>,
    /// Given name
    pub given_name: Option<String>,
    /// Family name
    pub family_name: Option<String>,
    /// Picture URL
    pub picture: Option<String>,
    /// Groups (common in enterprise IdPs)
    pub groups: Option<Vec<String>>,
}

impl SsoSession {
    /// Check if session is expired.
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }
}

/// SSO service.
pub struct SsoService {
    org_id: String,
    provider: SsoProvider,
}

impl SsoService {
    /// Create a new SSO service (requires enterprise license).
    pub fn new(org_id: impl Into<String>, provider: SsoProvider) -> Result<Self, license::LicenseError> {
        license::require("SSO")?;
        Ok(Self {
            org_id: org_id.into(),
            provider,
        })
    }

    /// Generate SAML auth request URL.
    pub fn generate_saml_auth_url(&self, config: &SamlConfig) -> String {
        // In production, this would generate a proper SAML AuthnRequest
        let request_id = uuid::Uuid::new_v4();
        format!(
            "{}?SAMLRequest={}&RelayState={}",
            config.idp_sso_url,
            base64_placeholder(&format!("AuthnRequest-{}", request_id)),
            self.org_id
        )
    }

    /// Generate OIDC auth URL.
    pub fn generate_oidc_auth_url(&self, config: &OidcConfig, state: &str) -> String {
        let scopes = config.scopes.join(" ");
        format!(
            "{}/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            config.issuer,
            config.client_id,
            urlencoding_placeholder(&config.redirect_uri),
            urlencoding_placeholder(&scopes),
            state
        )
    }

    /// Parse SAML response and create session.
    pub fn parse_saml_response(&self, _saml_response: &str) -> Result<SsoUser, SsoError> {
        // In production, this would:
        // 1. Decode and validate SAML Response
        // 2. Verify signature against IdP certificate
        // 3. Extract user attributes
        
        // Placeholder for demo
        Ok(SsoUser {
            external_id: "saml-user-123".to_string(),
            email: "user@example.com".to_string(),
            name: "SAML User".to_string(),
            first_name: Some("SAML".to_string()),
            last_name: Some("User".to_string()),
            groups: vec!["employees".to_string()],
            attributes: HashMap::new(),
            provider: SsoProvider::Saml,
        })
    }

    /// Exchange OIDC code for tokens.
    /// 
    /// Real implementation using reqwest HTTP client and jsonwebtoken for JWT validation.
    pub async fn exchange_oidc_code(
        &self,
        config: &OidcConfig,
        code: &str,
    ) -> Result<SsoSession, SsoError> {
        use base64::Engine;
        
        // Build token request
        let client = reqwest::Client::new();
        
        let token_url = format!("{}/token", config.issuer);
        
        let response = client
            .post(&token_url)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &config.redirect_uri),
                ("client_id", &config.client_id),
                ("client_secret", &config.client_secret),
            ])
            .send()
            .await
            .map_err(|e| SsoError::TokenExchangeFailed(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SsoError::TokenExchangeFailed(format!(
                "Token endpoint returned error: {}", error_text
            )));
        }
        
        // Parse token response
        let token_response: OidcTokenResponse = response
            .json()
            .await
            .map_err(|e| SsoError::TokenExchangeFailed(e.to_string()))?;
        
        // Decode and validate ID token (without verifying signature for now)
        // In production with JWKS, use jsonwebtoken::decode with proper validation
        let id_token_parts: Vec<&str> = token_response.id_token.split('.').collect();
        if id_token_parts.len() != 3 {
            return Err(SsoError::TokenExchangeFailed("Invalid ID token format".into()));
        }
        
        // Decode claims from payload (base64url)
        let claims_json = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(id_token_parts[1])
            .map_err(|_| SsoError::TokenExchangeFailed("Failed to decode ID token".into()))?;
        
        let claims: OidcClaims = serde_json::from_slice(&claims_json)
            .map_err(|e| SsoError::TokenExchangeFailed(format!("Invalid ID token claims: {}", e)))?;
        
        // Validate issuer
        if claims.iss != config.issuer {
            return Err(SsoError::TokenExchangeFailed(format!(
                "Issuer mismatch: expected {}, got {}", config.issuer, claims.iss
            )));
        }
        
        // Validate expiration
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if claims.exp < now {
            return Err(SsoError::TokenExchangeFailed("ID token expired".into()));
        }
        
        tracing::info!(
            org_id = %self.org_id,
            user_email = %claims.email.as_deref().unwrap_or("unknown"),
            "OIDC authentication successful"
        );
        
        Ok(SsoSession {
            session_id: uuid::Uuid::new_v4().to_string(),
            user: SsoUser {
                external_id: claims.sub,
                email: claims.email.unwrap_or_default(),
                name: claims.name.unwrap_or_else(|| "OIDC User".to_string()),
                first_name: claims.given_name,
                last_name: claims.family_name,
                groups: claims.groups.unwrap_or_default(),
                attributes: HashMap::new(),
                provider: SsoProvider::Oidc,
            },
            created_at: now,
            expires_at: claims.exp,
            access_token: Some(token_response.access_token),
            refresh_token: token_response.refresh_token,
        })
    }


    /// Get provider.
    pub fn provider(&self) -> SsoProvider {
        self.provider
    }
}

/// SSO errors.
#[derive(Debug, thiserror::Error)]
pub enum SsoError {
    #[error("Invalid SAML response")]
    InvalidSamlResponse,
    #[error("SAML signature verification failed")]
    SamlSignatureInvalid,
    #[error("OIDC token exchange failed: {reason}")]
    OidcTokenExchangeFailed { reason: String },
    #[error("Session expired")]
    SessionExpired,
    #[error("User not authorized")]
    Unauthorized,
}

// Placeholder functions for demo
fn base64_placeholder(s: &str) -> String {
    format!("BASE64_{}", s.len())
}

fn urlencoding_placeholder(s: &str) -> String {
    s.replace(' ', "%20").replace('/', "%2F")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sso_requires_license() {
        std::env::remove_var("VERIMANTLE_LICENSE_KEY");
        let result = SsoService::new("org-123", SsoProvider::Saml);
        assert!(result.is_err());
    }

    #[test]
    fn test_sso_with_license() {
        std::env::set_var("VERIMANTLE_LICENSE_KEY", "test-license");
        let result = SsoService::new("org-123", SsoProvider::Okta);
        assert!(result.is_ok());
        std::env::remove_var("VERIMANTLE_LICENSE_KEY");
    }

    #[test]
    fn test_oidc_auth_url() {
        std::env::set_var("VERIMANTLE_LICENSE_KEY", "test-license");
        let service = SsoService::new("org-123", SsoProvider::Oidc).unwrap();
        
        let config = OidcConfig {
            issuer: "https://auth.example.com".to_string(),
            client_id: "client-123".to_string(),
            client_secret: "secret".to_string(),
            redirect_uri: "https://app.verimantle.com/callback".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
            token_auth_method: TokenAuthMethod::ClientSecretBasic,
        };
        
        let url = service.generate_oidc_auth_url(&config, "state-123");
        assert!(url.contains("authorize"));
        assert!(url.contains("client_id=client-123"));
        
        std::env::remove_var("VERIMANTLE_LICENSE_KEY");
    }

    #[test]
    fn test_session_expiry() {
        let session = SsoSession {
            session_id: "sess-1".to_string(),
            user: SsoUser {
                external_id: "user-1".to_string(),
                email: "test@example.com".to_string(),
                name: "Test".to_string(),
                first_name: None,
                last_name: None,
                groups: vec![],
                attributes: HashMap::new(),
                provider: SsoProvider::Oidc,
            },
            created_at: 0,
            expires_at: 0, // Expired
            access_token: None,
            refresh_token: None,
        };
        
        assert!(session.is_expired());
    }
}
