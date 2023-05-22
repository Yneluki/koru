pub mod direct_event_bus;

#[cfg(feature = "redis-bus")]
mod redis_event_bus;

use crate::application::event_bus::{EventBus, EventBusError, EventListener};
use crate::configuration::event_bus::EventBusSettings;
use crate::domain::usecases::event_processor::EventProcessor;
use crate::infrastructure::event_bus::direct_event_bus::{DirectEventBus, DirectEventListener};
#[cfg(feature = "redis-bus")]
use crate::infrastructure::event_bus::redis_event_bus::{RedisEventBus, RedisEventListener};
use anyhow::Error;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

pub enum EventBusImpl {
    #[cfg(feature = "redis-bus")]
    Redis(Arc<RedisEventBus>),
    Direct(Arc<DirectEventBus>),
}

impl EventBusImpl {
    pub async fn build(
        configuration: &EventBusSettings,
    ) -> Result<(EventBusImpl, EventListenerImpl), anyhow::Error> {
        match configuration {
            EventBusSettings::Memory => {
                let bus = Arc::new(DirectEventBus::new());
                let listener = DirectEventListener::new(bus.clone());
                Ok((
                    EventBusImpl::Direct(bus),
                    EventListenerImpl::Direct(listener),
                ))
            }
            #[cfg(feature = "redis-bus")]
            EventBusSettings::Redis(config) => {
                let bus = RedisEventBus::try_new(config).await?;
                let listener = RedisEventListener::try_new(config).await?;
                Ok((
                    EventBusImpl::Redis(Arc::new(bus)),
                    EventListenerImpl::Redis(listener),
                ))
            }
        }
    }
}

#[async_trait]
impl EventBus for EventBusImpl {
    async fn publish(&self, event_ids: &[Uuid]) -> Result<(), EventBusError> {
        match self {
            #[cfg(feature = "redis-bus")]
            EventBusImpl::Redis(r) => r.publish(event_ids).await,
            EventBusImpl::Direct(d) => d.publish(event_ids).await,
        }
    }
}

pub enum EventListenerImpl {
    #[cfg(feature = "redis-bus")]
    Redis(RedisEventListener),
    Direct(DirectEventListener),
}

#[async_trait]
impl EventListener for EventListenerImpl {
    fn register(&mut self, processor: impl EventProcessor + 'static) {
        match self {
            #[cfg(feature = "redis-bus")]
            EventListenerImpl::Redis(redis) => redis.register(processor),
            EventListenerImpl::Direct(direct) => direct.register(processor),
        }
    }

    async fn listen(self) -> Result<(), Error> {
        match self {
            #[cfg(feature = "redis-bus")]
            EventListenerImpl::Redis(redis) => redis.listen().await,
            EventListenerImpl::Direct(direct) => direct.listen().await,
        }
    }
}
