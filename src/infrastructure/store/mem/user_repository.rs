use crate::application::store::{UserRepository, UserRepositoryError};
use crate::domain::{Email, User, UserRole};
use crate::infrastructure::store::mem::mem_store::{InMemTx, InMemoryStore, InnerRole, InnerUser};
use async_trait::async_trait;
use itertools::Itertools;
use std::cell::RefCell;
use std::sync::atomic::Ordering::Relaxed;
use uuid::Uuid;

#[async_trait]
impl UserRepository for InMemoryStore {
    type Tr = InMemTx;

    async fn save(
        &self,
        _tx: &mut RefCell<Self::Tr>,
        user: &User,
    ) -> Result<(), UserRepositoryError> {
        if self.crash_users.load(Relaxed) {
            return Err(UserRepositoryError::CorruptedData("Crashed store"));
        }
        let role = match user.role {
            UserRole::Administrator => InnerRole::ADMINISTRATOR,
            UserRole::User => InnerRole::USER,
        };
        let user = InnerUser {
            id: user.id,
            name: String::from(user.name.clone()),
            email: String::from(user.email.clone()),
            role,
            created_at: user.created_at,
        };
        self.users.lock().unwrap().insert(user.id, user);
        Ok(())
    }

    async fn find(&self, user_id: &Uuid) -> Result<Option<User>, UserRepositoryError> {
        if self.crash_users.load(Relaxed) {
            return Err(UserRepositoryError::CorruptedData("Crashed store"));
        }
        self.users
            .lock()
            .unwrap()
            .iter()
            .find(|(id, _)| user_id == *id)
            .map(|(_, user)| User::try_from(user.clone()))
            .map_or(Ok(None), |u| u.map(Some))
            .map_err(UserRepositoryError::CorruptedData)
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, UserRepositoryError> {
        if self.crash_users.load(Relaxed) {
            return Err(UserRepositoryError::CorruptedData("Crashed store"));
        }
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .any(|(_, u)| String::from(email.clone()) == u.email))
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, UserRepositoryError> {
        if self.crash_users.load(Relaxed) {
            return Err(UserRepositoryError::CorruptedData("Crashed store"));
        }
        self.users
            .lock()
            .unwrap()
            .iter()
            .find(|(_, u)| String::from(email.clone()) == u.email)
            .map(|(_, user)| User::try_from(user.clone()))
            .map_or(Ok(None), |u| u.map(Some))
            .map_err(UserRepositoryError::CorruptedData)
    }

    async fn fetch_users(&self, user_ids: &[Uuid]) -> Result<Vec<User>, UserRepositoryError> {
        if self.crash_users.load(Relaxed) {
            return Err(UserRepositoryError::CorruptedData("Crashed store"));
        }
        let r = self
            .users
            .lock()
            .unwrap()
            .iter()
            .filter(|(id, _)| user_ids.contains(*id))
            .map(|(_, user)| user.clone())
            .collect_vec();
        let mut users = Vec::new();
        for user in r {
            users.push(User::try_from(user).map_err(UserRepositoryError::CorruptedData)?);
        }
        Ok(users)
    }

    async fn fetch_all_users(&self) -> Result<Vec<User>, UserRepositoryError> {
        if self.crash_users.load(Relaxed) {
            return Err(UserRepositoryError::CorruptedData("Crashed store"));
        }
        let r = self
            .users
            .lock()
            .unwrap()
            .iter()
            .map(|(_, user)| user.clone())
            .collect_vec();
        let mut users = Vec::new();
        for user in r {
            users.push(User::try_from(user).map_err(UserRepositoryError::CorruptedData)?);
        }
        Ok(users)
    }
}
