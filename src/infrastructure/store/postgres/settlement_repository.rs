use crate::application::store::{SettlementRepository, SettlementRepositoryError};
use crate::domain::{Amount, Settlement, SettlementDescription, Transaction};
use crate::infrastructure::store::postgres::pg_store::PgStore;
use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{Postgres, QueryBuilder};
use std::cell::RefCell;
use uuid::Uuid;

impl PgStore {
    #[tracing::instrument(name = "Save transactions in DB", skip(self, tx, transactions))]
    async fn save_transactions<'a>(
        &self,
        tx: &'a mut sqlx::Transaction<'_, Postgres>,
        settlement_id: &'a Uuid,
        transactions: &'a [Transaction],
    ) -> Result<(), SettlementRepositoryError> {
        let mut query: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO koru_transaction (settlement_id, from_user_id, to_user_id, amount) ",
        );
        query.push_values(transactions, |mut b, transaction| {
            b.push_bind(settlement_id)
                .push_bind(transaction.from)
                .push_bind(transaction.to)
                .push_bind(f32::from(transaction.amount));
        });
        query
            .build()
            .execute(tx)
            .await
            .map_err(|e| SettlementRepositoryError::Insert(anyhow!(e)))?;
        Ok(())
    }

    #[tracing::instrument(name = "Link expenses in DB", skip(self, tx, expense_ids))]
    async fn link_expenses<'a>(
        &self,
        tx: &'a mut sqlx::Transaction<'_, Postgres>,
        settlement_id: &'a Uuid,
        expense_ids: &'a [Uuid],
    ) -> Result<(), SettlementRepositoryError> {
        let mut query: QueryBuilder<Postgres> =
            QueryBuilder::new("INSERT INTO koru_settlement_expenses (settlement_id, expense_id) ");
        query.push_values(expense_ids, |mut b, user| {
            b.push_bind(settlement_id).push_bind(user);
        });
        query
            .build()
            .execute(tx)
            .await
            .map_err(|e| SettlementRepositoryError::Insert(anyhow!(e)))?;
        Ok(())
    }

    #[tracing::instrument(name = "Get transactions from DB", skip(self))]
    async fn get_transactions(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Vec<Transaction>, SettlementRepositoryError> {
        let rows = sqlx::query!(
            r#"
        SELECT settlement_id, from_user_id, to_user_id, amount
        FROM koru_transaction
        WHERE settlement_id = $1
        ORDER BY amount DESC;
        "#,
            settlement_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SettlementRepositoryError::Fetch(anyhow!(e)))?;
        let mut res = Vec::new();
        for row in rows {
            res.push(Transaction {
                from: row.from_user_id,
                to: row.to_user_id,
                amount: Amount::try_from(row.amount)
                    .map_err(SettlementRepositoryError::CorruptedData)?,
            })
        }
        Ok(res)
    }

    #[tracing::instrument(name = "Get expenses ids from DB", skip(self))]
    async fn get_expenses_by_stl(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Vec<Uuid>, SettlementRepositoryError> {
        sqlx::query!(
            r#"
        SELECT settlement_id, expense_id
        FROM koru_settlement_expenses
        WHERE settlement_id = $1
        "#,
            settlement_id,
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|r| r.expense_id).collect())
        .map_err(|e| SettlementRepositoryError::Fetch(anyhow!(e)))
    }
}

#[async_trait]
impl SettlementRepository for PgStore {
    type Tr = sqlx::Transaction<'static, Postgres>;

    #[tracing::instrument(name = "Save settlement to DB", skip(self, tx))]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        settlement: &Settlement,
    ) -> Result<(), SettlementRepositoryError> {
        sqlx::query!(
            r#"
        INSERT INTO koru_settlement (id, group_id, start_date, end_date) VALUES ($1, $2, $3, $4)
        "#,
            settlement.id,
            settlement.group_id,
            settlement.start_date,
            settlement.end_date
        )
        .execute(tx.get_mut())
        .await
        .map_err(|e| SettlementRepositoryError::Insert(anyhow!(e)))?;
        self.save_transactions(tx.get_mut(), &settlement.id, &settlement.transactions)
            .await?;
        self.link_expenses(tx.get_mut(), &settlement.id, &settlement.expense_ids)
            .await?;
        Ok(())
    }

    #[tracing::instrument(name = "Get settlements from DB", skip(self))]
    async fn get_settlements(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<Settlement>, SettlementRepositoryError> {
        let descriptions: Vec<SettlementDescription> = sqlx::query!(
            r#"
        SELECT id, group_id, start_date, end_date FROM koru_settlement WHERE group_id = $1 ORDER BY end_date ASC;
        "#,
            group_id
        )
            .fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter().map(|row| SettlementDescription {
                    id: row.id,
                    group_id: row.group_id,
                    start_date: row.start_date,
                    end_date: row.end_date,
                }).collect()
            })
            .map_err(|e| SettlementRepositoryError::Fetch(anyhow!(e)))?;
        let mut res = Vec::new();
        for description in descriptions {
            let transactions = self.get_transactions(&description.id).await?;
            let expenses = self.get_expenses_by_stl(&description.id).await?;
            res.push(Settlement {
                id: description.id,
                group_id: description.group_id,
                start_date: description.start_date,
                end_date: description.end_date,
                transactions,
                expense_ids: expenses,
            })
        }
        Ok(res)
    }

    #[tracing::instrument(name = "Get settlement from DB", skip(self))]
    async fn get_settlement_description(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Option<SettlementDescription>, SettlementRepositoryError> {
        sqlx::query!(
            r#"
        SELECT id, group_id, start_date, end_date FROM koru_settlement WHERE id = $1;
        "#,
            settlement_id
        )
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|row| SettlementDescription {
                id: row.id,
                group_id: row.group_id,
                start_date: row.start_date,
                end_date: row.end_date,
            })
        })
        .map_err(|e| SettlementRepositoryError::Fetch(anyhow!(e)))
    }

    #[tracing::instrument(name = "Find settlement from DB", skip(self))]
    async fn find(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Option<Settlement>, SettlementRepositoryError> {
        let description = sqlx::query!(
            r#"
        SELECT id, group_id, start_date, end_date FROM koru_settlement WHERE id = $1;
        "#,
            settlement_id
        )
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|row| SettlementDescription {
                id: row.id,
                group_id: row.group_id,
                start_date: row.start_date,
                end_date: row.end_date,
            })
        })
        .map_err(|e| SettlementRepositoryError::Fetch(anyhow!(e)))?;
        match description {
            Some(description) => {
                let transactions = self.get_transactions(&description.id).await?;
                let expenses = self.get_expenses_by_stl(&description.id).await?;
                Ok(Some(Settlement {
                    id: description.id,
                    group_id: description.group_id,
                    start_date: description.start_date,
                    end_date: description.end_date,
                    transactions,
                    expense_ids: expenses,
                }))
            }
            None => Ok(None),
        }
    }

    #[tracing::instrument(name = "Find settlement in DB", skip(self))]
    async fn exists(&self, settlement_id: &Uuid) -> Result<bool, SettlementRepositoryError> {
        let stl = sqlx::query!(
            r#"
        SELECT id FROM koru_settlement WHERE id = $1
        "#,
            settlement_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SettlementRepositoryError::Fetch(anyhow!(e)))?;
        match stl {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    #[tracing::instrument(name = "Get expenses ids from DB", skip(self))]
    async fn get_expenses(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Vec<Uuid>, SettlementRepositoryError> {
        self.get_expenses_by_stl(settlement_id).await
    }
}
