use webauthn_rs::prelude::*;
use uuid::Uuid;
use thiserror::Error;
use std::sync::Arc;

#[derive(Error, Debug)]
pub enum WebAuthnError {
    #[error("WebAuthn error: {0}")]
    Core(#[from] WebauthnError),
    #[error("Database error")]
    Database,
    #[error("User not found")]
    UserNotFound,
}

pub struct WebAuthnService {
    webauthn: Arc<Webauthn>,
}

impl WebAuthnService {
    pub fn new(rp_id: &str, rp_origin: &str) -> Self {
        let rp_origin_url = Url::parse(rp_origin).expect("Invalid RP Origin URL");
        let builder = WebauthnBuilder::new(rp_id, &rp_origin_url)
            .expect("Invalid RP Protocol");
        let webauthn = Arc::new(builder.build().expect("Failed to build WebAuthn instance"));
        Self { webauthn }
    }

    pub async fn start_registration(&self, username: &str) -> Result<(CreationChallengeResponse, PasskeyRegistration), WebAuthnError> {
        let user_id = Uuid::new_v4();
        let (challenge, state) = self.webauthn
            .start_passkey_registration(
                user_id,
                username,
                username, // display_name
                None, // exclude_credentials
            )
            .map_err(WebAuthnError::Core)?;

        Ok((challenge, state))
    }

    pub async fn finish_registration(&self, reg: &RegisterPublicKeyCredential, state: &PasskeyRegistration) -> Result<Passkey, WebAuthnError> {
        let passkey = self.webauthn
            .finish_passkey_registration(reg, state)
            .map_err(WebAuthnError::Core)?;
        
        Ok(passkey)
    }
}
