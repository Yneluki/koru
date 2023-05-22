use crate::application::store::{GroupRepository, GroupRepositoryError, MemberRepository};
use crate::domain::{Group, GroupName};
use crate::infrastructure::store::postgres::pg_store::PgStore;
use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{Postgres, Transaction};
use std::cell::RefCell;
use uuid::Uuid;

impl PgStore {
    #[tracing::instrument(name = "Get group expenses from DB", skip(self))]
    async fn get_expenses(&self, group_id: &Uuid) -> Result<Vec<Uuid>, GroupRepositoryError> {
        sqlx::query!(
            r#"
        SELECT id FROM koru_expense WHERE group_id = $1 and settled = false
        "#,
            group_id,
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|r| r.id).collect())
        .map_err(|e| GroupRepositoryError::Fetch(anyhow!(e)))
    }

    #[tracing::instrument(name = "Get group settlements from DB", skip(self))]
    async fn get_settlements(&self, group_id: &Uuid) -> Result<Vec<Uuid>, GroupRepositoryError> {
        sqlx::query!(
            r#"
        SELECT id FROM koru_settlement WHERE group_id = $1
        "#,
            group_id,
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|r| r.id).collect())
        .map_err(|e| GroupRepositoryError::Fetch(anyhow!(e)))
    }
}

#[async_trait]
impl GroupRepository for PgStore {
    type Tr = Transaction<'static, Postgres>;

    #[tracing::instrument(name = "Save group in DB", skip(self, tx, group))]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        group: &Group,
    ) -> Result<(), GroupRepositoryError> {
        sqlx::query!(
            r#"
        INSERT INTO koru_group (id, name, admin_id, created_at) VALUES ($1, $2, $3, $4)
        ON CONFLICT DO NOTHING
        "#,
            group.id,
            String::from(group.name.clone()),
            group.admin_id,
            group.created_at
        )
        .execute(tx.get_mut())
        .await
        .map_err(|e| GroupRepositoryError::Insert(anyhow!(e)))?;
        Ok(())
    }

    #[tracing::instrument(name = "Delete group in DB", skip(self, tx))]
    async fn delete(
        &self,
        tx: &mut RefCell<Self::Tr>,
        group_id: &Uuid,
    ) -> Result<(), GroupRepositoryError> {
        sqlx::query!(
            r#"
        DELETE FROM koru_group WHERE id = $1
        "#,
            group_id,
        )
        .execute(tx.get_mut())
        .await
        .map_err(|e| GroupRepositoryError::Delete(anyhow!(e)))?;
        Ok(())
    }

    #[tracing::instrument(name = "Get group from DB", skip(self))]
    async fn find(&self, group_id: &Uuid) -> Result<Option<Group>, GroupRepositoryError> {
        let row = sqlx::query!(
            r#"
        SELECT id, name, admin_id, created_at FROM koru_group WHERE id = $1
        "#,
            group_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GroupRepositoryError::Fetch(anyhow!(e)))?;
        match row {
            Some(r) => {
                let members = self
                    .fetch_members(group_id)
                    .await
                    .map_err(|e| GroupRepositoryError::Fetch(anyhow!(e)))?;
                let expenses = self.get_expenses(group_id).await?;
                let settlements = self.get_settlements(group_id).await?;
                Ok(Some(Group {
                    id: r.id,
                    name: GroupName::try_from(r.name)
                        .map_err(GroupRepositoryError::CorruptedData)?,
                    admin_id: r.admin_id,
                    created_at: r.created_at,
                    members,
                    expense_ids: expenses,
                    settlement_ids: settlements,
                    events: vec![],
                }))
            }
            None => Ok(None),
        }
    }

    #[tracing::instrument(name = "Get user groups from DB", skip(self))]
    async fn get_user_groups(&self, user_id: &Uuid) -> Result<Vec<Group>, GroupRepositoryError> {
        let ids: Vec<Uuid> = sqlx::query!(
            r#"
        SELECT id as "id!" FROM koru_group WHERE admin_id = $1
        UNION
        SELECT group_id as "id!" FROM koru_group_members WHERE user_id = $1
        "#,
            user_id,
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|r| r.id).collect())
        .map_err(|e| GroupRepositoryError::Fetch(anyhow!(e)))?;
        let mut groups = Vec::new();
        for id in ids {
            if let Some(group) = self.find(&id).await? {
                groups.push(group);
            }
        }
        Ok(groups)
    }

    async fn fetch_all_groups(&self) -> Result<Vec<Group>, GroupRepositoryError> {
        let ids: Vec<Uuid> = sqlx::query!(
            r#"
        SELECT id as "id!" FROM koru_group
        "#,
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|r| r.id).collect())
        .map_err(|e| GroupRepositoryError::Fetch(anyhow!(e)))?;
        let mut groups = Vec::new();
        for id in ids {
            if let Some(group) = self.find(&id).await? {
                groups.push(group);
            }
        }
        Ok(groups)
    }
}
