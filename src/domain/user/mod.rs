mod user_name;

pub use user_name::UserName;

use crate::domain::errors::CreateUserError;
use crate::domain::shared::email::Email;
use crate::utils::date;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub name: UserName,
    pub email: Email,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn create(name: String, email: String) -> Result<Self, CreateUserError> {
        let id = Uuid::new_v4();
        let name = UserName::try_from(name).map_err(CreateUserError::Validation)?;
        let email = Email::try_from(email).map_err(CreateUserError::Validation)?;
        Ok(Self {
            id,
            name,
            email,
            role: UserRole::User,
            created_at: date::now(),
        })
    }

    pub fn is_admin(&self) -> bool {
        match self.role {
            UserRole::Administrator => true,
            UserRole::User => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum UserRole {
    Administrator,
    User,
}
