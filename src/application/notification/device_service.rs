use crate::application::store::MultiRepository;
use anyhow::Context;
use log::warn;
use std::sync::Arc;
use uuid::Uuid;

pub struct DeviceService<Store: MultiRepository> {
    store: Arc<Store>,
}

impl<Store: MultiRepository> DeviceService<Store> {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub async fn save(&self, user_id: &Uuid, device: String) {
        self.store
            .device()
            .save_device(user_id, device)
            .await
            .context("Failed to save device")
            .unwrap_or_else(|failure| {
                warn!("{:?}", failure);
            });
    }

    pub async fn delete(&self, user_id: &Uuid) {
        self.store
            .device()
            .remove_device(user_id)
            .await
            .context("Failed to delete device")
            .unwrap_or_else(|failure| {
                warn!("{:?}", failure);
            });
    }
}
