use crate::domain::errors::{GenerateGroupTokenError, JoinGroupError};
use crate::domain::TokenGenerator;
use anyhow::anyhow;
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct FakeTokenGenerator {
    tokens: Mutex<HashSet<String>>,
}

impl FakeTokenGenerator {
    pub fn new() -> Self {
        Self {
            tokens: Mutex::new(HashSet::new()),
        }
    }
}

#[async_trait]
impl TokenGenerator for FakeTokenGenerator {
    async fn generate(&self, group_id: &Uuid) -> Result<String, GenerateGroupTokenError> {
        let token = group_id.to_string();
        self.tokens.lock().unwrap().insert(token.clone());
        Ok(token)
    }

    async fn verify(&self, token: String, group_id: &Uuid) -> Result<(), JoinGroupError> {
        let valid = self.tokens.lock().unwrap().contains(&token);
        if valid {
            let token_id = Uuid::parse_str(token.as_str())
                .map_err(|e| JoinGroupError::Unexpected(anyhow!(e)))?;
            if token_id == *group_id {
                Ok(())
            } else {
                Err(JoinGroupError::Unauthorized("Token does not match group."))
            }
        } else {
            Err(JoinGroupError::Unauthorized("Token not found."))
        }
    }
}
