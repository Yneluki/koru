use crate::application::store::{ExpenseRepository, ExpenseRepositoryError};
use crate::domain::{Amount, Expense, ExpenseTitle};
use crate::infrastructure::store::postgres::pg_store::PgStore;
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Postgres, Row, Transaction};
use std::cell::RefCell;
use uuid::Uuid;

#[async_trait]
impl ExpenseRepository for PgStore {
    type Tr = Transaction<'static, Postgres>;

    #[tracing::instrument(name = "Save expense in DB", skip(self, tx))]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        expense: &Expense,
    ) -> Result<(), ExpenseRepositoryError> {
        sqlx::query!(
            r#"
        INSERT INTO koru_expense (id, group_id, member_id, description, amount, created_at, modified_at, settled)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (id) DO UPDATE SET 
            description = EXCLUDED.description, 
            amount = EXCLUDED.amount, 
            modified_at = EXCLUDED.modified_at, 
            settled = EXCLUDED.settled;
        "#,
            expense.id,
            expense.group_id,
            expense.member_id,
            String::from(expense.title.clone()),
            f32::from(expense.amount.clone()),
            expense.created_at,
            expense.modified_at,
            expense.settled,
        )
        .execute(tx.get_mut())
        .await
        .map_err(|e| ExpenseRepositoryError::Insert(anyhow!(e)))?;
        Ok(())
    }

    #[tracing::instrument(name = "Delete expense in DB", skip(self, tx))]
    async fn delete(
        &self,
        tx: &mut RefCell<Self::Tr>,
        expense_id: &Uuid,
    ) -> Result<(), ExpenseRepositoryError> {
        sqlx::query!(
            r#"
        DELETE FROM koru_expense WHERE id = $1
        "#,
            expense_id,
        )
        .execute(tx.get_mut())
        .await
        .map_err(|e| ExpenseRepositoryError::Delete(anyhow!(e)))?;
        Ok(())
    }

    #[tracing::instrument(name = "Find expense in DB", skip(self))]
    async fn find(&self, expense_id: &Uuid) -> Result<Option<Expense>, ExpenseRepositoryError> {
        let row = sqlx::query!(
            r#"
        SELECT id, group_id, member_id, description, amount, created_at, modified_at, settled
        FROM koru_expense WHERE id = $1
        "#,
            expense_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ExpenseRepositoryError::Fetch(anyhow!(e)))?;
        match row {
            Some(row) => Ok(Some(Expense {
                id: row.id,
                group_id: row.group_id,
                member_id: row.member_id,
                title: ExpenseTitle::try_from(row.description)
                    .map_err(ExpenseRepositoryError::CorruptedData)?,
                amount: Amount::try_from(row.amount)
                    .map_err(ExpenseRepositoryError::CorruptedData)?,
                created_at: row.created_at,
                modified_at: row.modified_at,
                settled: row.settled,
            })),
            None => Ok(None),
        }
    }

    #[tracing::instrument(name = "Get expenses in range from DB", skip(self))]
    async fn get_expenses(
        &self,
        group_id: &Uuid,
        start_date: Option<&DateTime<Utc>>,
        end_date: Option<&DateTime<Utc>>,
    ) -> Result<Vec<Expense>, ExpenseRepositoryError> {
        let query = match (start_date, end_date) {
            (Some(start), Some(end)) => sqlx::query(
                r#"
                SELECT id, group_id, member_id, description, amount, created_at, modified_at, settled
                FROM koru_expense
                WHERE group_id= $1 AND created_at > $2 AND created_at <= $3
                "#,
            )
            .bind(group_id)
            .bind(start)
            .bind(end),
            (Some(start), None) => sqlx::query(
                r#"
                SELECT id, group_id, member_id, description, amount, created_at, modified_at, settled
                FROM koru_expense
                WHERE group_id= $1 AND created_at > $2
                "#,
            )
            .bind(group_id)
            .bind(start),
            (None, Some(end)) => sqlx::query(
                r#"
                SELECT id, group_id, member_id, description, amount, created_at, modified_at, settled
                FROM koru_expense
                WHERE group_id= $1 AND created_at <= $2
                "#,
            )
            .bind(group_id)
            .bind(end),
            (None, None) => sqlx::query(
                r#"
                SELECT id, group_id, member_id, description, amount, created_at, modified_at, settled
                FROM koru_expense
                WHERE group_id= $1
                "#,
            )
            .bind(group_id),
        };
        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ExpenseRepositoryError::Fetch(anyhow!(e)))?;
        let mut expenses = Vec::new();
        for row in rows {
            expenses.push(Expense {
                id: row.get("id"),
                group_id: row.get("group_id"),
                member_id: row.get("member_id"),
                title: ExpenseTitle::try_from(row.get::<String, &str>("description"))
                    .map_err(ExpenseRepositoryError::CorruptedData)?,
                amount: Amount::try_from(row.get::<f32, &str>("amount"))
                    .map_err(ExpenseRepositoryError::CorruptedData)?,
                created_at: row.get("created_at"),
                modified_at: row.get("modified_at"),
                settled: row.get("settled"),
            });
        }
        Ok(expenses)
    }

    #[tracing::instrument(name = "Get expenses by id from DB", skip(self))]
    async fn get_expenses_by_id(
        &self,
        expenses_id: &[Uuid],
    ) -> Result<Vec<Expense>, ExpenseRepositoryError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, group_id, member_id, description, amount, created_at, modified_at, settled
                FROM koru_expense
                WHERE id = ANY($1)
            "#,
            expenses_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ExpenseRepositoryError::Fetch(anyhow!(e)))?;
        let mut expenses = Vec::new();
        for row in rows {
            expenses.push(Expense {
                id: row.id,
                group_id: row.group_id,
                member_id: row.member_id,
                title: ExpenseTitle::try_from(row.description)
                    .map_err(ExpenseRepositoryError::CorruptedData)?,
                amount: Amount::try_from(row.amount)
                    .map_err(ExpenseRepositoryError::CorruptedData)?,
                created_at: row.created_at,
                modified_at: row.modified_at,
                settled: row.settled,
            });
        }
        Ok(expenses)
    }

    #[tracing::instrument(name = "Get unsettled expenses from DB", skip(self))]
    async fn get_unsettled_expenses(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<Expense>, ExpenseRepositoryError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, group_id, member_id, description, amount, created_at, modified_at, settled
                FROM koru_expense
                WHERE group_id= $1 AND settled = false
            "#,
            group_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ExpenseRepositoryError::Fetch(anyhow!(e)))?;
        let mut expenses = Vec::new();
        for row in rows {
            expenses.push(Expense {
                id: row.id,
                group_id: row.group_id,
                member_id: row.member_id,
                title: ExpenseTitle::try_from(row.description)
                    .map_err(ExpenseRepositoryError::CorruptedData)?,
                amount: Amount::try_from(row.amount)
                    .map_err(ExpenseRepositoryError::CorruptedData)?,
                created_at: row.created_at,
                modified_at: row.modified_at,
                settled: row.settled,
            });
        }
        Ok(expenses)
    }
}
