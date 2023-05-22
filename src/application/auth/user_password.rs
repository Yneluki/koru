use crate::application::auth::CredentialsHasher;
use crate::domain::errors::CreateUserError;
use anyhow::anyhow;
use secrecy::{ExposeSecret, Secret};
use std::sync::Arc;

#[derive(Debug)]
pub struct Password {
    password: Secret<String>,
    hashed: bool,
}

impl TryFrom<Secret<String>> for Password {
    type Error = &'static str;

    fn try_from(n: Secret<String>) -> Result<Self, Self::Error> {
        if n.expose_secret().is_empty() {
            Err("User Password cannot be empty")
        } else {
            Ok(Self {
                password: n,
                hashed: false,
            })
        }
    }
}

impl Password {
    pub fn build_from_hash(password_hash: Secret<String>) -> Self {
        Password {
            password: password_hash,
            hashed: true,
        }
    }

    pub fn hash(&self, hasher: Arc<dyn CredentialsHasher>) -> Result<Self, CreateUserError> {
        let hashed = hasher
            .hash_password(self.password.clone())
            .map_err(|e| CreateUserError::Unexpected(anyhow!(e)))?;
        Ok(Password {
            password: hashed,
            hashed: true,
        })
    }

    pub fn get(&self) -> Result<Secret<String>, CreateUserError> {
        if !self.hashed {
            return Err(CreateUserError::Unexpected(anyhow!(
                "Password is not hashed !!"
            )));
        }
        Ok(self.password.clone())
    }
}
