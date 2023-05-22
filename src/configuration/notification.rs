use crate::application::store::MultiRepository;
use crate::domain::notification::NotificationService;
#[cfg(feature = "pushy")]
use crate::infrastructure::notification_service::PushyNotificationService;
#[cfg(feature = "pushy")]
use secrecy::Secret;
use std::sync::Arc;

#[derive(serde::Deserialize, Debug)]
pub enum NotificationSettings {
    #[cfg(feature = "pushy")]
    #[serde(rename = "pushy")]
    Pushy(PushySettings),
}

#[cfg(feature = "pushy")]
#[derive(serde::Deserialize, Debug)]
pub struct PushySettings {
    pub token: Secret<String>,
    pub url: String,
}

impl NotificationSettings {
    pub fn setup_notification_svc(
        &self,
        store: Arc<impl MultiRepository>,
    ) -> anyhow::Result<impl NotificationService> {
        match self {
            #[cfg(feature = "pushy")]
            NotificationSettings::Pushy(conf) => PushyNotificationService::try_new(conf, store),
        }
    }
}
