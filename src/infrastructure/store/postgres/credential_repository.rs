use crate::application::auth::{
    CredentialRepository, CredentialRepositoryError, Password, UserCredentials,
};
use crate::domain::Email;
use crate::infrastructure::store::postgres::pg_store::PgStore;
use anyhow::anyhow;
use async_trait::async_trait;
use secrecy::{ExposeSecret, Secret};
use sqlx::{Postgres, Transaction};
use std::cell::RefCell;

#[async_trait]
impl CredentialRepository for PgStore {
    type Tr = Transaction<'static, Postgres>;

    #[tracing::instrument(name = "Save user in DB", skip(self, user_credentials))]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        user_credentials: &UserCredentials,
    ) -> Result<(), CredentialRepositoryError> {
        sqlx::query!(
            r#"
        INSERT INTO koru_user_credentials (email, password) VALUES ($1, $2)
        "#,
            String::from(user_credentials.email.clone()),
            user_credentials
                .password
                .get()
                .map_err(|_| CredentialRepositoryError::CorruptedData("Password corrupted"))?
                .expose_secret()
                .clone()
        )
        .execute(tx.get_mut())
        .await
        .map_err(|e| CredentialRepositoryError::Insert(anyhow!(e)))?;
        Ok(())
    }

    #[tracing::instrument(name = "Fetch user credentials by email from DB", skip(self))]
    async fn fetch_by_email(
        &self,
        email: &Email,
    ) -> Result<Option<UserCredentials>, CredentialRepositoryError> {
        let row = sqlx::query!(
            r#"
        SELECT email, password FROM koru_user_credentials WHERE email = $1
        "#,
            String::from(email.clone()),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CredentialRepositoryError::Fetch(anyhow!(e)))?;
        match row {
            Some(row) => Ok(Some(UserCredentials {
                email: Email::try_from(row.email)
                    .map_err(CredentialRepositoryError::CorruptedData)?,
                password: Password::build_from_hash(Secret::from(row.password)),
            })),
            None => Ok(None),
        }
    }
}
