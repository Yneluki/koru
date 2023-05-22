use crate::domain::errors::EventHandlerError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait EventProcessor: Send + Sync {
    async fn handle(&self, event_id: &Uuid) -> Result<(), EventHandlerError>;
}
