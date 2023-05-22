use crate::application::event_bus::{EventBus, EventBusError, EventListener};
use crate::configuration::event_bus::RedisSettings;
use crate::domain::usecases::event_processor::EventProcessor;
use anyhow::anyhow;
use async_trait::async_trait;
use futures_util::StreamExt as _;
use log::{info, warn};
use redis::{AsyncCommands, Client};
use secrecy::ExposeSecret;
use std::sync::Arc;
use uuid::Uuid;

pub struct RedisEventBus {
    client: Client,
    channel: String,
}

impl RedisEventBus {
    pub async fn try_new(configuration: &RedisSettings) -> Result<Self, anyhow::Error> {
        Ok(Self {
            client: Client::open(configuration.connection_string().expose_secret().clone())?,
            channel: configuration.event_channel.clone(),
        })
    }
}

#[async_trait]
impl EventBus for RedisEventBus {
    #[tracing::instrument(name = "Publishing events", skip(self, event_ids))]
    async fn publish(&self, event_ids: &[Uuid]) -> Result<(), EventBusError> {
        let mut connection = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| EventBusError::Publish(anyhow!(e)))?;
        for event in event_ids {
            connection
                .publish(self.channel.clone(), event.to_string())
                .await
                .map_err(|e| EventBusError::Publish(anyhow!(e)))?;
        }
        Ok(())
    }
}

pub struct RedisEventListener {
    client: Client,
    channel: String,
    processors: Vec<Arc<dyn EventProcessor>>,
}

#[async_trait]
impl EventListener for RedisEventListener {
    fn register(&mut self, processor: impl EventProcessor + 'static) {
        self.processors.push(Arc::new(processor))
    }

    #[tracing::instrument(name = "Listening to events", skip(self))]
    async fn listen(self) -> Result<(), anyhow::Error> {
        let con = self.client.get_async_connection().await?;
        let mut pubsub = con.into_pubsub();
        info!("Listening to channel {}", self.channel);
        pubsub.subscribe(self.channel.clone()).await?;
        let mut stream = pubsub.on_message();
        loop {
            if let Some(msg) = stream.next().await {
                let payload: String = msg.get_payload()?;
                self.handle_payload(payload).await;
            }
        }
    }
}

impl RedisEventListener {
    pub async fn try_new(configuration: &RedisSettings) -> Result<Self, anyhow::Error> {
        Ok(Self {
            client: Client::open(configuration.connection_string().expose_secret().clone())?,
            channel: configuration.event_channel.clone(),
            processors: Vec::new(),
        })
    }

    #[tracing::instrument(name = "Handling event", skip(self))]
    async fn handle_payload(&self, payload: String) {
        if let Ok(id) = Uuid::parse_str(payload.as_str()) {
            for processor in &self.processors {
                processor.handle(&id).await.unwrap_or_else(|failure| {
                    warn!("{:?}", failure);
                });
            }
        }
    }
}
