use crate::application::auth::{CredentialsHasher, Password};
use crate::domain::errors::CreateUserError;
use crate::domain::Email;
use secrecy::Secret;
use std::sync::Arc;

#[derive(Debug)]
pub struct UserCredentials {
    pub email: Email,
    pub password: Password,
}

impl UserCredentials {
    pub fn create(
        email: String,
        password: Secret<String>,
        hasher: Arc<dyn CredentialsHasher>,
    ) -> Result<Self, CreateUserError> {
        let email = Email::try_from(email).map_err(CreateUserError::Validation)?;
        let password = Password::try_from(password)
            .map_err(CreateUserError::Validation)?
            .hash(hasher)?;
        Ok(UserCredentials { email, password })
    }
}
