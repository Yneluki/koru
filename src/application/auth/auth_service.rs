use crate::application::auth::UserCredentials;
use crate::application::auth::{CredentialsHasher, Password};
use crate::application::store::MultiRepository;
use crate::domain::errors::{CreateUserError, CredentialServiceError, LoginError};
use crate::domain::Email;
use crate::utils::telemetry::spawn_blocking_with_tracing;
use anyhow::{anyhow, Context};
use secrecy::Secret;
use std::sync::Arc;

pub struct AuthService<Store: MultiRepository> {
    store: Arc<Store>,
    hasher: Arc<dyn CredentialsHasher>,
}

impl<Store: MultiRepository> AuthService<Store> {
    pub fn new(store: Arc<Store>, hasher: impl CredentialsHasher + 'static) -> Self {
        Self {
            store,
            hasher: Arc::new(hasher),
        }
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<(), CreateUserError> {
        let email = request.email.clone();
        let creds = self.hasher.clone();
        let user_credentials = spawn_blocking_with_tracing(move || {
            UserCredentials::create(email, request.password.clone(), creds)
        })
        .await
        .context("Failed to build user credentials")??;

        let mut tx = self.store.tx().await?;
        self.store
            .credentials()
            .save(&mut tx, &user_credentials)
            .await
            .context("Failed to insert user")
            .map_err(CreateUserError::Unexpected)?;
        self.store.commit(tx.into_inner()).await?;
        Ok(())
    }

    pub async fn login(&self, request: LoginRequest) -> Result<(), LoginError> {
        let email = Email::try_from(request.email).map_err(LoginError::Validation)?;
        let _ = Password::try_from(request.password.clone()).map_err(LoginError::Validation)?;
        let credentials = self
            .store
            .credentials()
            .fetch_by_email(&email)
            .await
            .context("Failed to fetch user")
            .map_err(LoginError::Unexpected)?;
        match credentials {
            None => Err(LoginError::InvalidCredentials()),
            Some(credentials) => {
                let expected_password = credentials
                    .password
                    .get()
                    .map_err(|_| LoginError::Unexpected(anyhow!("Corrupted password")))?;
                let creds = self.hasher.clone();
                let password = request.password.clone();
                spawn_blocking_with_tracing(move || {
                    creds
                        .verify(password, expected_password)
                        .map_err(|e| match e {
                            CredentialServiceError::InvalidCredentials() => {
                                LoginError::InvalidCredentials()
                            }
                            CredentialServiceError::Unexpected(e) => LoginError::Unexpected(e),
                        })
                })
                .await
                .context("Failed to verify password.")??;
                Ok(())
            }
        }
    }
}

pub struct RegisterRequest {
    pub email: String,
    pub password: Secret<String>,
}

pub struct LoginRequest {
    pub email: String,
    pub password: Secret<String>,
}
