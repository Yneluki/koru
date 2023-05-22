use crate::infrastructure::session_store::SessionState;
use actix_session::storage::{LoadError, SaveError, SessionKey, SessionStore, UpdateError};
use actix_web::cookie::time::Duration;
use anyhow::{anyhow, Error};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct MemorySessionStore {
    sessions: Mutex<HashMap<String, (SessionState, DateTime<Utc>)>>,
}

impl MemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SessionStore for MemorySessionStore {
    async fn load(&self, session_key: &SessionKey) -> Result<Option<SessionState>, LoadError> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_key.as_ref()).cloned();
        match session {
            None => Ok(None),
            Some((state, expiry)) => {
                if Utc::now() > expiry {
                    sessions.remove(session_key.as_ref());
                    Ok(None)
                } else {
                    Ok(Some(state))
                }
            }
        }
    }

    async fn save(
        &self,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, SaveError> {
        let key = Uuid::new_v4().to_string();
        let expiry = Utc::now() + chrono::Duration::seconds(ttl.whole_seconds());
        self.sessions
            .lock()
            .unwrap()
            .insert(key.clone(), (session_state, expiry));
        Ok(SessionKey::try_from(key).map_err(|e| SaveError::Other(anyhow!(e)))?)
    }

    async fn update(
        &self,
        session_key: SessionKey,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, UpdateError> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_key.as_ref()).cloned();
        match session {
            None => {
                let key = Uuid::new_v4().to_string();
                let expiry = Utc::now() + chrono::Duration::seconds(ttl.whole_seconds());
                sessions.insert(key.clone(), (session_state, expiry));
                Ok(SessionKey::try_from(key).map_err(|e| UpdateError::Other(anyhow!(e)))?)
            }
            Some(_) => {
                let expiry = Utc::now() + chrono::Duration::seconds(ttl.whole_seconds());
                let key = session_key.as_ref().to_string();
                sessions.insert(key, (session_state, expiry));
                Ok(session_key)
            }
        }
    }

    async fn update_ttl(&self, session_key: &SessionKey, ttl: &Duration) -> Result<(), Error> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_key.as_ref()).cloned();
        match session {
            None => Ok(()),
            Some((state, _)) => {
                let expiry = Utc::now() + chrono::Duration::seconds(ttl.whole_seconds());
                let key = session_key.as_ref().to_string();
                sessions.insert(key, (state, expiry));
                Ok(())
            }
        }
    }

    async fn delete(&self, session_key: &SessionKey) -> Result<(), Error> {
        self.sessions.lock().unwrap().remove(session_key.as_ref());
        Ok(())
    }
}
