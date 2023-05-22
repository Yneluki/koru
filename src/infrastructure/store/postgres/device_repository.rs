use crate::application::store::{DeviceRepository, DeviceRepositoryError};
use crate::infrastructure::store::postgres::pg_store::PgStore;
use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[async_trait]
impl DeviceRepository for PgStore {
    type Tr = Transaction<'static, Postgres>;

    #[tracing::instrument(name = "Fetch user device from DB", skip(self))]
    async fn fetch_device(&self, user_id: &Uuid) -> Result<Option<String>, DeviceRepositoryError> {
        Ok(sqlx::query!(
            r#"
        SELECT device FROM koru_user_device WHERE user_id = $1
        "#,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DeviceRepositoryError::Fetch(anyhow!(e)))?
        .and_then(|r| r.device))
    }

    #[tracing::instrument(name = "Save user device to DB", skip(self))]
    async fn save_device(
        &self,
        user_id: &Uuid,
        device_id: String,
    ) -> Result<(), DeviceRepositoryError> {
        sqlx::query!(
            r#"
        INSERT INTO koru_user_device (user_id, device) VALUES ($1, $2)
        ON CONFLICT (user_id) DO UPDATE SET
            device = EXCLUDED.device;
        "#,
            user_id,
            device_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DeviceRepositoryError::Insert(anyhow!(e)))?;
        Ok(())
    }

    #[tracing::instrument(name = "Remove user device from DB", skip(self))]
    async fn remove_device(&self, user_id: &Uuid) -> Result<(), DeviceRepositoryError> {
        sqlx::query!(
            r#"
        DELETE FROM koru_user_device WHERE user_id = $1
        "#,
            user_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DeviceRepositoryError::Fetch(anyhow!(e)))?;
        Ok(())
    }
}
