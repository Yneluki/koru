use crate::domain::notification::{NotificationService, NotificationServiceError};
use async_trait::async_trait;
use log::info;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct FakeNotificationService {
    pub notifications: Mutex<Vec<InnerNotification>>,
}

impl FakeNotificationService {
    pub fn new() -> Self {
        Self {
            notifications: Mutex::new(Vec::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InnerNotification {
    pub user: Uuid,
    pub title: String,
    pub text: String,
}

#[async_trait]
impl NotificationService for FakeNotificationService {
    async fn send(
        &self,
        user_id: &Uuid,
        notification_title: String,
        notification_text: String,
    ) -> Result<(), NotificationServiceError> {
        let notification = InnerNotification {
            user: *user_id,
            title: notification_title,
            text: notification_text,
        };
        info!("Sending {:?}", notification);
        self.notifications.lock().unwrap().push(notification);
        Ok(())
    }
}
