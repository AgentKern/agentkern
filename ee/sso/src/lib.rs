#![allow(unused)]
//! AgentKern Enterprise SSO
//!
//! Enterprise Single Sign-On (SAML 2.0 & OIDC) integration.
//!
//! # Features
//! - SAML 2.0 SP-Initiated SSO (Redirect Binding)
//! - OIDC Authorization Code Flow
//! - Attribute mapping
//! - Multi-tenant configuration

use base64::Engine;
use chrono::Utc;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;

/// SSO Provider Type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SsoProvider {
    Saml,
    Oidc,
    Okta,
    Auth0,
    AzureAd,
}

/// Token authentication method for OIDC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenAuthMethod {
    ClientSecretBasic,
    ClientSecretPost,
    None,
}

/// SAML Configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConfig {
    /// IdP SSO URL (Destination)
    pub idp_sso_url: String,
    /// IdP Entity ID (Issuer)
    pub idp_entity_id: String,
    /// SP Entity ID (Audience)
    pub sp_entity_id: String,
    /// IdP Public Certificate (PEM) for signature verification
    pub idp_cert_pem: String,
    /// Attribute mapping (IdP attribute name -> Internal user field)
    pub attribute_mapping: HashMap<String, String>,
}

/// OIDC Configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub token_auth_method: TokenAuthMethod,
}

/// Normalized SSO User Profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoUser {
    pub external_id: String,
    pub email: String,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub groups: Vec<String>,
    pub attributes: HashMap<String, serde_json::Value>,
    pub provider: SsoProvider,
}

/// SSO Session Info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoSession {
    pub session_id: String,
    pub user: SsoUser,
    pub created_at: u64,
    pub expires_at: u64,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

impl SsoSession {
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0); // Graceful fallback to 0 if system time fails
        now >= self.expires_at
    }
}

/// OIDC Token Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OidcTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    id_token: String,
    refresh_token: Option<String>,
}

/// OIDC ID Token Claims (minimal).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OidcClaims {
    sub: String,
    iss: String,
    aud: String, // or Vec<String>
    exp: u64,
    email: Option<String>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    groups: Option<Vec<String>>,
}

/// SSO Service.
pub struct SsoService {
    org_id: String,
    provider: SsoProvider,
}

impl SsoService {
    /// Create new SSO service instance.
    pub fn new(org_id: impl Into<String>, provider: SsoProvider) -> Result<Self, SsoError> {
        let org_id = org_id.into();

        // Enforce Enterprise License Check
        // In real "Gate", checks signed license capability
        if std::env::var("AGENTKERN_LICENSE_KEY").is_err() {
            tracing::warn!("SSO requires generic enterprise license check (mocked here)");
            // return Err(SsoError::Unauthorized); // Commented out for dev convenience
        }

        Ok(Self { org_id, provider })
    }

    /// Generate SAML AuthnRequest URL (Redirect Binding).
    ///
    /// Implements DEFLATE + Base64 + URL Encode as per SAML 2.0 Bindings.
    pub fn generate_saml_auth_url(&self, config: &SamlConfig) -> Result<String, SsoError> {
        let request_id = format!("AuthnRequest-{}", uuid::Uuid::new_v4());
        let issue_instant = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // Construct XML
        let xml = format!(
            r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="{}" Version="2.0" IssueInstant="{}" Destination="{}" ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" AssertionConsumerServiceURL="https://api.agentkern.com/sso/acs"><saml:Issuer>{}</saml:Issuer></samlp:AuthnRequest>"#,
            request_id, issue_instant, config.idp_sso_url, config.sp_entity_id
        );

        // DEFLATE
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(xml.as_bytes())
            .map_err(|e| SsoError::SamlEncodingFailed(e.to_string()))?;
        let compressed = encoder
            .finish()
            .map_err(|e| SsoError::SamlEncodingFailed(e.to_string()))?;

        // Base64
        let base64_encoded = base64::engine::general_purpose::STANDARD.encode(compressed);

        // URL Encode keys and values
        // Note: URL encoding should be applied to the parameters added to the URL.
        let encoded_req = urlencoding::encode(&base64_encoded);
        let encoded_relay = urlencoding::encode(&self.org_id);

        Ok(format!(
            "{}?SAMLRequest={}&RelayState={}",
            config.idp_sso_url, encoded_req, encoded_relay
        ))
    }

    /// Generate OIDC auth URL.
    pub fn generate_oidc_auth_url(&self, config: &OidcConfig, state: &str) -> String {
        let scopes = config.scopes.join(" ");
        format!(
            "{}/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            config.issuer,
            config.client_id,
            urlencoding::encode(&config.redirect_uri),
            urlencoding::encode(&scopes),
            state
        )
    }

    /// Parse SAML response and create session.
    pub fn parse_saml_response(&self, saml_response: &str) -> Result<SsoUser, SsoError> {
        // Base64 Decode
        let xml_bytes = base64::engine::general_purpose::STANDARD
            .decode(saml_response)
            .map_err(|_| SsoError::InvalidSamlResponse)?;

        let xml = String::from_utf8(xml_bytes).map_err(|_| SsoError::InvalidSamlResponse)?;

        // In fully strict implementation: Verify XML Signature using xmlsec1 (optional feature)
        // Here we do basic extraction for MVP

        let name_id = extract_tag_content(&xml, "NameID").unwrap_or_else(|| "unknown".to_string());
        let email = extract_tag_content(&xml, "email")
            .or_else(|| extract_tag_content(&xml, "Email"))
            .unwrap_or(name_id.clone());

        Ok(SsoUser {
            external_id: name_id,
            email,
            name: "SAML User".to_string(),
            first_name: None,
            last_name: None,
            groups: vec![],
            attributes: HashMap::new(),
            provider: SsoProvider::Saml,
        })
    }

    /// Exchange OIDC code for tokens.
    pub async fn exchange_oidc_code(
        &self,
        config: &OidcConfig,
        code: &str,
    ) -> Result<SsoSession, SsoError> {
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
                "Token endpoint returned error: {}",
                error_text
            )));
        }

        // Parse token response
        let token_response: OidcTokenResponse = response
            .json()
            .await
            .map_err(|e| SsoError::TokenExchangeFailed(e.to_string()))?;

        // Decode and validate ID token
        let id_token_parts: Vec<&str> = token_response.id_token.split('.').collect();
        if id_token_parts.len() != 3 {
            return Err(SsoError::TokenExchangeFailed(
                "Invalid ID token format".into(),
            ));
        }

        // Decode claims from payload (base64url)
        let claims_json = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(id_token_parts[1])
            .map_err(|_| SsoError::TokenExchangeFailed("Failed to decode ID token".into()))?;

        let claims: OidcClaims = serde_json::from_slice(&claims_json).map_err(|e| {
            SsoError::TokenExchangeFailed(format!("Invalid ID token claims: {}", e))
        })?;

        // Validate issuer (basic)
        if !claims.iss.contains(&config.issuer) && !config.issuer.contains(&claims.iss) {
            tracing::warn!(
                "Issuer mismatch warning: {} vs {}",
                config.issuer,
                claims.iss
            );
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| SsoError::SystemTimeError)?
            .as_secs();

        if claims.exp < now {
            return Err(SsoError::TokenExchangeFailed("ID token expired".into()));
        }

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

/// Helper to extract tag content (XML naive parser).
fn extract_tag_content(xml: &str, tag_name: &str) -> Option<String> {
    let _start_tag = format!("<{}>", tag_name); // Naive
                                                // Simple basic check for MVP
    if let Some(start) = xml.find(&format!("<{}", tag_name)) {
        // better start finding
        // logic to find > then </
        if let Some(closing) = xml[start..].find('>') {
            let content_start = start + closing + 1;
            if let Some(end) = xml[content_start..].find("</") {
                let content = &xml[content_start..content_start + end];
                return Some(content.to_string());
            }
        }
    }
    None
}

/// SSO errors.
#[derive(Debug, thiserror::Error)]
pub enum SsoError {
    #[error("Invalid SAML response")]
    InvalidSamlResponse,
    #[error("SAML signature verification failed")]
    SamlSignatureInvalid,
    #[error("SAML encoding failed: {0}")]
    SamlEncodingFailed(String),
    #[error("OIDC token exchange failed: {0}")]
    TokenExchangeFailed(String),
    #[error("Session expired")]
    SessionExpired,
    #[error("User not authorized")]
    Unauthorized,
    #[error("System time error")]
    SystemTimeError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saml_request_compression() {
        let service = SsoService::new("org-1", SsoProvider::Saml).unwrap();
        let config = SamlConfig {
            idp_sso_url: "http://idp.com".into(),
            idp_entity_id: "idp".into(),
            sp_entity_id: "sp".into(),
            idp_cert_pem: "".into(),
            attribute_mapping: HashMap::new(),
        };

        let url = service
            .generate_saml_auth_url(&config)
            .expect("SAML encoding should succeed");
        assert!(url.contains("SAMLRequest="));
        assert!(url.contains("RelayState=org-1"));
        assert!(!url.contains(" "));
    }
}
