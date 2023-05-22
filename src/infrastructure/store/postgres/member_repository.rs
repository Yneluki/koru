use crate::application::store::{MemberRepository, MemberRepositoryError};
use crate::domain::{Email, GroupMember, MemberColor, UserName};
use crate::infrastructure::store::postgres::pg_store::PgStore;
use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{Postgres, Transaction};
use std::cell::RefCell;
use uuid::Uuid;

#[async_trait]
impl MemberRepository for PgStore {
    type Tr = Transaction<'static, Postgres>;

    #[tracing::instrument(name = "Save member in DB", skip(self, tx))]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        member: &GroupMember,
    ) -> Result<(), MemberRepositoryError> {
        sqlx::query!(
            r#"
        INSERT INTO koru_group_members (group_id, user_id, joined_at, color) VALUES ($1, $2, $3, $4)
        ON CONFLICT (group_id, user_id) DO UPDATE SET
            color = EXCLUDED.color;
        "#,
            member.group_id,
            member.id,
            member.joined_at,
            String::from(member.color.clone()),
        )
        .execute(tx.get_mut())
        .await
        .map_err(|e| MemberRepositoryError::Insert(anyhow!(e)))?;
        Ok(())
    }

    #[tracing::instrument(name = "Get group members from DB", skip(self))]
    async fn fetch_members(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<GroupMember>, MemberRepositoryError> {
        let admin_id = sqlx::query!(r#"SELECT admin_id FROM koru_group WHERE id= $1"#, group_id)
            .fetch_one(&self.pool)
            .await
            .map(|r| r.admin_id)
            .map_err(|e| MemberRepositoryError::Fetch(anyhow!(e)))?;
        let rows = sqlx::query!(
            r#"
        SELECT user_id as "user_id!", group_id, joined_at, color, name, email
        FROM koru_group_members LEFT JOIN koru_user ON user_id = koru_user.id
        WHERE group_id = $1
        "#,
            group_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| MemberRepositoryError::Fetch(anyhow!(e)))?;
        let mut members = Vec::new();
        for row in rows {
            members.push(GroupMember {
                id: row.user_id,
                name: UserName::try_from(row.name).map_err(MemberRepositoryError::CorruptedData)?,
                email: Email::try_from(row.email).map_err(MemberRepositoryError::CorruptedData)?,
                group_id: row.group_id,
                is_admin: row.user_id == admin_id,
                color: MemberColor::try_from(row.color)
                    .map_err(MemberRepositoryError::CorruptedData)?,
                joined_at: row.joined_at,
            })
        }
        Ok(members)
    }
}
