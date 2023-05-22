use crate::application::notification::notify::notify;
use crate::application::store::MultiRepository;
use crate::domain::errors::EventHandlerError;
use crate::domain::notification::NotificationService;
use crate::domain::usecases::event_processor::EventProcessor;
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

pub struct Notifier<Store: MultiRepository> {
    store: Arc<Store>,
    notification_svc: Arc<dyn NotificationService>,
}

impl<Store: MultiRepository> Notifier<Store> {
    pub fn new(store: Arc<Store>, notification_svc: Arc<dyn NotificationService>) -> Self {
        Self {
            store,
            notification_svc,
        }
    }
}

#[async_trait]
impl<Store: MultiRepository> EventProcessor for Notifier<Store> {
    async fn handle(&self, event_id: &Uuid) -> Result<(), EventHandlerError> {
        notify(event_id, self.store.clone(), self.notification_svc.clone())
            .await
            .map_err(|e| EventHandlerError::Unexpected(anyhow!(e)))
    }
}
