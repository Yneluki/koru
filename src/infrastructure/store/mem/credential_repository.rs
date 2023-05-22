use crate::application::auth::{
    CredentialRepository, CredentialRepositoryError, Password, UserCredentials,
};
use crate::domain::Email;
use crate::infrastructure::store::mem::mem_store::{InMemTx, InMemoryStore};
use async_trait::async_trait;
use secrecy::{ExposeSecret, Secret};
use std::cell::RefCell;
use std::sync::atomic::Ordering::Relaxed;

#[async_trait]
impl CredentialRepository for InMemoryStore {
    type Tr = InMemTx;

    async fn save(
        &self,
        _tx: &mut RefCell<Self::Tr>,
        user_credentials: &UserCredentials,
    ) -> Result<(), CredentialRepositoryError> {
        if self.crash_user_credentials.load(Relaxed) {
            return Err(CredentialRepositoryError::CorruptedData("Crashed store"));
        }
        self.user_credentials.lock().unwrap().insert(
            String::from(user_credentials.email.clone()),
            user_credentials
                .password
                .get()
                .map_err(|_| CredentialRepositoryError::CorruptedData("Password corrupted"))?
                .expose_secret()
                .clone(),
        );
        Ok(())
    }

    async fn fetch_by_email(
        &self,
        email: &Email,
    ) -> Result<Option<UserCredentials>, CredentialRepositoryError> {
        if self.crash_user_credentials.load(Relaxed) {
            return Err(CredentialRepositoryError::CorruptedData("Crashed store"));
        }
        let pass = self
            .user_credentials
            .lock()
            .unwrap()
            .get(&String::from(email.clone()))
            .cloned();
        match pass {
            Some(pass) => Ok(Some(UserCredentials {
                email: email.clone(),
                password: Password::build_from_hash(Secret::new(pass)),
            })),
            None => Ok(None),
        }
    }
}
