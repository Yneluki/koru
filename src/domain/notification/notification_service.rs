use crate::error_chain;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send(
        &self,
        user_id: &Uuid,
        notification_title: String,
        notification_text: String,
    ) -> Result<(), NotificationServiceError>;
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum NotificationServiceError {
        #[error("Failed to send event.")]
        Send(#[source] anyhow::Error),
    }
}
