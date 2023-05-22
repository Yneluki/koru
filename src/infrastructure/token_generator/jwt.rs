use crate::domain::errors::{GenerateGroupTokenError, JoinGroupError};
use crate::domain::TokenGenerator;
use anyhow::Context;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey};
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

pub struct JwtTokenGenerator {
    key: Secret<String>,
}

impl JwtTokenGenerator {
    pub fn new(key: Secret<String>) -> Self {
        Self { key }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Claims {
    sub: Uuid,
    exp: usize,
}

#[async_trait]
impl TokenGenerator for JwtTokenGenerator {
    #[tracing::instrument(name = "Building jwt", skip(self))]
    async fn generate(&self, group_id: &Uuid) -> Result<String, GenerateGroupTokenError> {
        let key = &EncodingKey::from_secret(self.key.expose_secret().as_bytes());
        let expiration = Utc::now()
            .checked_add_signed(Duration::minutes(15))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: *group_id,
            exp: expiration as usize,
        };

        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512);
        jsonwebtoken::encode(&header, &claims, key)
            .context("Failed to build token.")
            .map_err(GenerateGroupTokenError::Unexpected)
    }

    #[tracing::instrument(name = "Validating jwt", skip(self, token, group_id))]
    async fn verify(&self, token: String, group_id: &Uuid) -> Result<(), JoinGroupError> {
        let key = &DecodingKey::from_secret(self.key.expose_secret().as_bytes());
        let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512);
        let decoded = jsonwebtoken::decode::<Claims>(token.as_str(), key, &validation)
            .map_err(|_| JoinGroupError::Unauthorized("Invalid token."))?;
        let token_id = decoded.claims.sub;
        if token_id == *group_id {
            Ok(())
        } else {
            Err(JoinGroupError::Unauthorized("Token does not match group."))
        }
    }
}
