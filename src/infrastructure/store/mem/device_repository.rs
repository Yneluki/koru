use crate::application::store::{DeviceRepository, DeviceRepositoryError};
use crate::infrastructure::store::mem::mem_store::{InMemTx, InMemoryStore};
use async_trait::async_trait;
use std::sync::atomic::Ordering::Relaxed;
use uuid::Uuid;

#[async_trait]
impl DeviceRepository for InMemoryStore {
    type Tr = InMemTx;

    async fn fetch_device(&self, user_id: &Uuid) -> Result<Option<String>, DeviceRepositoryError> {
        if self.crash_user_devices.load(Relaxed) {
            return Err(DeviceRepositoryError::CorruptedData("Crashed store"));
        }
        Ok(self.user_devices.lock().unwrap().get(user_id).cloned())
    }

    async fn save_device(
        &self,
        user_id: &Uuid,
        device_id: String,
    ) -> Result<(), DeviceRepositoryError> {
        if self.crash_user_devices.load(Relaxed) {
            return Err(DeviceRepositoryError::CorruptedData("Crashed store"));
        }
        self.user_devices
            .lock()
            .unwrap()
            .insert(*user_id, device_id);
        Ok(())
    }

    async fn remove_device(&self, user_id: &Uuid) -> Result<(), DeviceRepositoryError> {
        if self.crash_user_devices.load(Relaxed) {
            return Err(DeviceRepositoryError::CorruptedData("Crashed store"));
        }
        self.user_devices.lock().unwrap().remove(user_id);
        Ok(())
    }
}
