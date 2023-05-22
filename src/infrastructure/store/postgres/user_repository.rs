use crate::application::store::{UserRepository, UserRepositoryError};
use crate::domain::{Email, User, UserName, UserRole};
use crate::infrastructure::store::postgres::pg_store::PgStore;
use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{Postgres, Transaction};
use std::cell::RefCell;
use uuid::Uuid;

#[async_trait]
impl UserRepository for PgStore {
    type Tr = Transaction<'static, Postgres>;

    #[tracing::instrument(name = "Save user in DB", skip(self, user))]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        user: &User,
    ) -> Result<(), UserRepositoryError> {
        sqlx::query!(
            r#"
        INSERT INTO koru_user (id, email, name, created_at) VALUES ($1, $2, $3, $4)
        "#,
            user.id,
            String::from(user.email.clone()),
            String::from(user.name.clone()),
            user.created_at
        )
        .execute(tx.get_mut())
        .await
        .map_err(|e| UserRepositoryError::Insert(anyhow!(e)))?;
        sqlx::query!(
            r#"
        INSERT INTO koru_user_roles (user_id, role) VALUES ($1, $2)
        "#,
            user.id,
            PgUserRole::from(user.role) as PgUserRole,
        )
        .execute(tx.get_mut())
        .await
        .map_err(|e| UserRepositoryError::Insert(anyhow!(e)))?;
        Ok(())
    }

    #[tracing::instrument(name = "Fetch user from DB", skip(self, user_id))]
    async fn find(&self, user_id: &Uuid) -> Result<Option<User>, UserRepositoryError> {
        let row = sqlx::query!(
            r#"
        SELECT id, email, name, created_at, role as "role: PgUserRole"
        FROM koru_user JOIN koru_user_roles ON user_id = id WHERE id = $1
        "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserRepositoryError::Fetch(anyhow!(e)))?;
        match row {
            Some(row) => {
                let u = User {
                    id: row.id,
                    name: UserName::try_from(row.name)
                        .map_err(UserRepositoryError::CorruptedData)?,
                    email: Email::try_from(row.email)
                        .map_err(UserRepositoryError::CorruptedData)?,
                    role: UserRole::from(row.role),
                    created_at: row.created_at,
                };
                Ok(Some(u))
            }
            None => Ok(None),
        }
    }

    #[tracing::instrument(name = "Fetch user by email from DB", skip(self, email))]
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, UserRepositoryError> {
        let row = sqlx::query!(
            r#"
        SELECT id, email, name, created_at, role as "role: PgUserRole" FROM koru_user
        JOIN koru_user_roles ON user_id = id WHERE email = $1
        "#,
            String::from(email.clone())
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserRepositoryError::Fetch(anyhow!(e)))?;
        match row {
            Some(row) => {
                let u = User {
                    id: row.id,
                    name: UserName::try_from(row.name)
                        .map_err(UserRepositoryError::CorruptedData)?,
                    email: Email::try_from(row.email)
                        .map_err(UserRepositoryError::CorruptedData)?,
                    role: UserRole::from(row.role),
                    created_at: row.created_at,
                };
                Ok(Some(u))
            }
            None => Ok(None),
        }
    }

    #[tracing::instrument(name = "Check user email exists in DB", skip(self))]
    async fn exists_by_email(&self, email: &Email) -> Result<bool, UserRepositoryError> {
        let row = sqlx::query!(
            r#"
        SELECT id FROM koru_user WHERE email = $1
        "#,
            String::from(email.clone())
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserRepositoryError::Fetch(anyhow!(e)))?;
        match row {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    #[tracing::instrument(name = "Fetch users from DB", skip(self, user_ids))]
    async fn fetch_users(&self, user_ids: &[Uuid]) -> Result<Vec<User>, UserRepositoryError> {
        let rows = sqlx::query!(
            r#"
        SELECT id, email, name, created_at, role as "role: PgUserRole" FROM koru_user
        JOIN koru_user_roles ON user_id = id WHERE id = ANY($1)
        "#,
            user_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| UserRepositoryError::Fetch(anyhow!(e)))?;
        let mut users = Vec::new();
        for row in rows {
            let u = User {
                id: row.id,
                name: UserName::try_from(row.name).map_err(UserRepositoryError::CorruptedData)?,
                email: Email::try_from(row.email).map_err(UserRepositoryError::CorruptedData)?,
                role: UserRole::from(row.role),
                created_at: row.created_at,
            };
            users.push(u);
        }
        Ok(users)
    }

    #[tracing::instrument(name = "Fetch all users from DB", skip(self))]
    async fn fetch_all_users(&self) -> Result<Vec<User>, UserRepositoryError> {
        let rows = sqlx::query!(
            r#"
        SELECT id, email, name, created_at, role as "role: PgUserRole" FROM koru_user
        JOIN koru_user_roles ON user_id = id
        "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| UserRepositoryError::Fetch(anyhow!(e)))?;
        let mut users = Vec::new();
        for row in rows {
            let u = User {
                id: row.id,
                name: UserName::try_from(row.name).map_err(UserRepositoryError::CorruptedData)?,
                email: Email::try_from(row.email).map_err(UserRepositoryError::CorruptedData)?,
                role: UserRole::from(row.role),
                created_at: row.created_at,
            };
            users.push(u);
        }
        Ok(users)
    }
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "role", rename_all = "snake_case")]
enum PgUserRole {
    Admin,
    User,
}

impl From<PgUserRole> for UserRole {
    fn from(value: PgUserRole) -> Self {
        match value {
            PgUserRole::Admin => UserRole::Administrator,
            PgUserRole::User => UserRole::User,
        }
    }
}

impl From<UserRole> for PgUserRole {
    fn from(value: UserRole) -> Self {
        match value {
            UserRole::Administrator => PgUserRole::Admin,
            UserRole::User => PgUserRole::User,
        }
    }
}
