use crate::domain::errors::CredentialServiceError;
use secrecy::Secret;

pub trait CredentialsHasher: Send + Sync {
    fn hash_password(
        &self,
        password: Secret<String>,
    ) -> Result<Secret<String>, CredentialServiceError>;

    fn verify(
        &self,
        password_to_test: Secret<String>,
        password: Secret<String>,
    ) -> Result<(), CredentialServiceError>;
}
