use crate::application::store::MultiRepository;
use crate::configuration::notification::PushySettings;
use crate::domain::notification::{NotificationService, NotificationServiceError};
use anyhow::{anyhow, Context, Error};
use async_trait::async_trait;
use log::info;
use reqwest::{Client, Response};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub struct PushyNotificationService<Store: MultiRepository> {
    url: String,
    api_token: Secret<String>,
    client: Client,
    store: Arc<Store>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PushyNotification {
    pub to: String,
    pub data: PushyNotificationData,
}

#[derive(Debug, Clone, Serialize)]
pub struct PushyNotificationData {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PushyErrorResponse {
    pub code: String,
    pub error: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PushyOkResponse {
    pub id: String,
    pub success: bool,
    pub info: PushyInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PushyInfo {
    pub devices: i64,
}

impl<Store: MultiRepository> PushyNotificationService<Store> {
    pub fn try_new(
        configuration: &PushySettings,
        store: Arc<Store>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            url: configuration.url.clone(),
            api_token: configuration.token.clone(),
            client: Client::builder()
                .build()
                .context("Failed to initialize REST Client")?,
            store,
        })
    }

    async fn fetch_device(&self, user_id: &Uuid) -> Result<Option<String>, Error> {
        self.store
            .device()
            .fetch_device(user_id)
            .await
            .context("Failed to fetch user device")
    }

    #[tracing::instrument(name = "Sending notification", skip(self))]
    async fn send(&self, notification: PushyNotification) -> Result<Response, Error> {
        self.client
            .post(
                format!(
                    "{}/push?api_key={}",
                    self.url,
                    self.api_token.expose_secret()
                )
                .as_str(),
            )
            .json(&notification)
            .send()
            .await
            .context("Failed to send notification to Pushy")
    }
}

#[async_trait]
impl<Store: MultiRepository> NotificationService for PushyNotificationService<Store> {
    #[tracing::instrument(name = "Handling notification with pushy", skip(self))]
    async fn send(
        &self,
        user_id: &Uuid,
        notification_title: String,
        notification_text: String,
    ) -> Result<(), NotificationServiceError> {
        let device_id = self
            .fetch_device(user_id)
            .await
            .map_err(NotificationServiceError::Send)?
            .map_or_else(
                || Err(NotificationServiceError::Send(anyhow!("Device not found"))),
                Ok,
            )?;
        let notification = PushyNotification {
            to: device_id,
            data: PushyNotificationData {
                message: [notification_title, notification_text].join("|"),
            },
        };
        let response = self
            .send(notification)
            .await
            .map_err(NotificationServiceError::Send)?;
        let status_code = response.status().as_u16();
        if status_code != 200 {
            let resp = response
                .json::<PushyErrorResponse>()
                .await
                .context("Failed to parse error response from Pushy")
                .map_err(NotificationServiceError::Send)?;
            Err(NotificationServiceError::Send(anyhow!(
                "Got error response from Pushy: {}/{:?}",
                status_code,
                resp
            )))
        } else {
            let resp = response
                .json::<PushyOkResponse>()
                .await
                .context("Failed to parse ok response from Pushy")
                .map_err(NotificationServiceError::Send)?;
            info!("Pushy response {:?}", resp);
            Ok(())
        }
    }
}
