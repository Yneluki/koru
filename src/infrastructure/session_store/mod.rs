use crate::configuration::application::SessionStoreSettings;
use crate::infrastructure::session_store::memory_session_store::MemorySessionStore;
#[cfg(feature = "redis-session")]
use actix_session::storage::RedisSessionStore;
use actix_session::storage::{LoadError, SaveError, SessionKey, SessionStore, UpdateError};

use actix_web::cookie::time::Duration;
use anyhow::Error;
#[cfg(feature = "redis-session")]
use secrecy::ExposeSecret;
use std::collections::HashMap;
use std::sync::Arc;

pub type SessionState = HashMap<String, String>;
mod memory_session_store;

#[derive(Clone)]
pub enum SessionStoreImpl {
    #[cfg(feature = "redis-session")]
    Redis(RedisSessionStore),
    Memory(Arc<MemorySessionStore>),
}

impl SessionStoreImpl {
    pub async fn build(configuration: &SessionStoreSettings) -> Result<Self, anyhow::Error> {
        match configuration {
            #[cfg(feature = "redis-session")]
            SessionStoreSettings::Redis(redis) => {
                let store =
                    RedisSessionStore::new(redis.connection_string().expose_secret()).await?;
                Ok(SessionStoreImpl::Redis(store))
            }
            SessionStoreSettings::Memory => Ok(SessionStoreImpl::Memory(Arc::new(
                MemorySessionStore::new(),
            ))),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SessionStore for SessionStoreImpl {
    async fn load(&self, session_key: &SessionKey) -> Result<Option<SessionState>, LoadError> {
        match self {
            #[cfg(feature = "redis-session")]
            SessionStoreImpl::Redis(store) => store.load(session_key).await,
            SessionStoreImpl::Memory(store) => store.load(session_key).await,
        }
    }

    async fn save(
        &self,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, SaveError> {
        match self {
            #[cfg(feature = "redis-session")]
            SessionStoreImpl::Redis(store) => store.save(session_state, ttl).await,
            SessionStoreImpl::Memory(store) => store.save(session_state, ttl).await,
        }
    }

    async fn update(
        &self,
        session_key: SessionKey,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, UpdateError> {
        match self {
            #[cfg(feature = "redis-session")]
            SessionStoreImpl::Redis(store) => store.update(session_key, session_state, ttl).await,
            SessionStoreImpl::Memory(store) => store.update(session_key, session_state, ttl).await,
        }
    }

    async fn update_ttl(&self, session_key: &SessionKey, ttl: &Duration) -> Result<(), Error> {
        match self {
            #[cfg(feature = "redis-session")]
            SessionStoreImpl::Redis(store) => store.update_ttl(session_key, ttl).await,
            SessionStoreImpl::Memory(store) => store.update_ttl(session_key, ttl).await,
        }
    }

    async fn delete(&self, session_key: &SessionKey) -> Result<(), Error> {
        match self {
            #[cfg(feature = "redis-session")]
            SessionStoreImpl::Redis(store) => store.delete(session_key).await,
            SessionStoreImpl::Memory(store) => store.delete(session_key).await,
        }
    }
}
