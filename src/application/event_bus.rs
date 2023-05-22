use crate::domain::usecases::event_processor::EventProcessor;
use crate::error_chain;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event_ids: &[Uuid]) -> Result<(), EventBusError>;
}

#[async_trait]
pub trait EventListener {
    fn register(&mut self, processor: impl EventProcessor + 'static);
    async fn listen(self) -> Result<(), anyhow::Error>;
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum EventBusError {
        #[error("Failed to publish event.")]
        Publish(#[source] anyhow::Error),
    }
}
