use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;
use webauthn_rs::prelude::*;

#[derive(Error, Debug)]
pub enum WebAuthnError {
    #[error("WebAuthn error: {0}")]
    Core(#[from] WebauthnError),
    #[error("WebAuthn configuration error: {0}")]
    InvalidConfig(String),
    #[error("Database error")]
    Database,
    #[error("User not found")]
    UserNotFound,
}

pub struct WebAuthnService {
    webauthn: Arc<Webauthn>,
}

impl WebAuthnService {
    pub fn new(rp_id: &str, rp_origin: &str) -> Result<Self, WebAuthnError> {
        let rp_origin_url = Url::parse(rp_origin)
            .map_err(|e| WebAuthnError::InvalidConfig(format!("Invalid RP origin URL: {e}")))?;
        let builder = WebauthnBuilder::new(rp_id, &rp_origin_url)
            .map_err(|e| WebAuthnError::InvalidConfig(format!("Invalid RP protocol: {e}")))?;
        let webauthn = Arc::new(builder.build().map_err(|e| {
            WebAuthnError::InvalidConfig(format!("Failed to build WebAuthn instance: {e}"))
        })?);
        Ok(Self { webauthn })
    }

    pub async fn start_registration(
        &self,
        username: &str,
    ) -> Result<(CreationChallengeResponse, PasskeyRegistration), WebAuthnError> {
        let user_id = Uuid::new_v4();
        let (challenge, state) = self
            .webauthn
            .start_passkey_registration(
                user_id, username, username, // display_name
                None,     // exclude_credentials
            )
            .map_err(WebAuthnError::Core)?;

        Ok((challenge, state))
    }

    pub async fn finish_registration(
        &self,
        reg: &RegisterPublicKeyCredential,
        state: &PasskeyRegistration,
    ) -> Result<Passkey, WebAuthnError> {
        let passkey = self
            .webauthn
            .finish_passkey_registration(reg, state)
            .map_err(WebAuthnError::Core)?;

        Ok(passkey)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_invalid_origin() {
        let result = WebAuthnService::new("localhost", "not-a-url");
        assert!(matches!(result, Err(WebAuthnError::InvalidConfig(_))));
    }
}
