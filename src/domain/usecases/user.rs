use crate::domain::errors::{CreateUserError, LoginError, LogoutError};
use async_trait::async_trait;
use secrecy::Secret;
use uuid::Uuid;

#[async_trait(?Send)]
pub trait UserUseCase {
    async fn register(&self, request: RegistrationRequest) -> Result<Uuid, CreateUserError>;
    async fn login(&self, request: LoginRequest) -> Result<Uuid, LoginError>;
    async fn logout(&self, request: LogoutRequest) -> Result<(), LogoutError>;
    async fn is_valid_user(&self, user_id: &Uuid) -> Result<bool, anyhow::Error>;
}

#[derive(Clone)]
pub struct RegistrationRequest {
    pub name: String,
    pub email: String,
    pub password: Option<Secret<String>>,
}

#[derive(Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: Option<Secret<String>>,
}

#[derive(Clone)]
pub struct LogoutRequest {
    pub user_id: Uuid,
}
