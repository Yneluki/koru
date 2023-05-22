use crate::application::auth::UserCredentials;
use crate::application::store::Tx;
use crate::domain::Email;
use crate::error_chain;
use async_trait::async_trait;
use std::cell::RefCell;

error_chain! {
    #[derive(thiserror::Error)]
    pub enum CredentialRepositoryError {
        #[error("Failed to insert credential.")]
        Insert(#[source] anyhow::Error),
        #[error("Failed to fetch credential.")]
        Fetch(#[source] anyhow::Error),
        #[error("Corrupted data in DB: {0}")]
        CorruptedData(&'static str),
    }
}

#[async_trait]
pub trait CredentialRepository: Send + Sync {
    type Tr: Tx;

    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        user: &UserCredentials,
    ) -> Result<(), CredentialRepositoryError>;

    async fn fetch_by_email(
        &self,
        email: &Email,
    ) -> Result<Option<UserCredentials>, CredentialRepositoryError>;
}
