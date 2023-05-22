use crate::application::auth::CredentialsHasher;
use crate::domain::errors::CredentialServiceError;
use anyhow::Context;
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use secrecy::{ExposeSecret, Secret};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Default)]
pub struct FakeCredentialsHasher {}

impl FakeCredentialsHasher {
    pub fn new() -> Self {
        Self {}
    }
}

impl CredentialsHasher for FakeCredentialsHasher {
    fn hash_password(
        &self,
        password: Secret<String>,
    ) -> Result<Secret<String>, CredentialServiceError> {
        let mut hasher = DefaultHasher::new();
        password.expose_secret().hash(&mut hasher);
        Ok(Secret::new(hasher.finish().to_string()))
    }

    fn verify(
        &self,
        password_to_test: Secret<String>,
        password: Secret<String>,
    ) -> Result<(), CredentialServiceError> {
        let mut hasher = DefaultHasher::new();
        password_to_test.expose_secret().hash(&mut hasher);
        let candidate = hasher.finish().to_string();
        if candidate == password.expose_secret().clone() {
            Ok(())
        } else {
            Err(CredentialServiceError::InvalidCredentials())
        }
    }
}

pub struct ArgonCredentialsHasher {
    memory: u32,
}

impl ArgonCredentialsHasher {
    pub fn new(memory: Option<u32>) -> Self {
        Self {
            memory: memory.unwrap_or(16384),
        }
    }
}

impl CredentialsHasher for ArgonCredentialsHasher {
    #[tracing::instrument(name = "Hashing password", skip(self, password))]
    fn hash_password(
        &self,
        password: Secret<String>,
    ) -> Result<Secret<String>, CredentialServiceError> {
        let salt = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(self.memory, 2, 1, None).unwrap(),
        )
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .map_err(|e| CredentialServiceError::Unexpected(e.into()))?
        .to_string();
        Ok(Secret::new(password_hash))
    }

    #[tracing::instrument(name = "Verify password", skip(self, password_to_test, password))]
    fn verify(
        &self,
        password_to_test: Secret<String>,
        password: Secret<String>,
    ) -> Result<(), CredentialServiceError> {
        let expected_password_hash = PasswordHash::new(password.expose_secret())
            .context("Failed to parse hash in PHC string format.")?;
        Argon2::default()
            .verify_password(
                password_to_test.expose_secret().as_bytes(),
                &expected_password_hash,
            )
            .map_err(|_| CredentialServiceError::InvalidCredentials())
    }
}
