use crate::domain::errors::{GenerateGroupTokenError, JoinGroupError};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait TokenGenerator: Send + Sync {
    async fn generate(&self, group_id: &Uuid) -> Result<String, GenerateGroupTokenError>;

    async fn verify(&self, token: String, group_id: &Uuid) -> Result<(), JoinGroupError>;
}
