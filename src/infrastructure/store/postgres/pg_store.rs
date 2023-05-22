use crate::application::auth::CredentialRepository;
use crate::application::store::{
    DeviceRepository, EventRepository, ExpenseRepository, GroupRepository, MemberRepository,
    MultiRepository, SettlementRepository, Tx, UserRepository,
};
use anyhow::{Context, Error};
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use std::cell::RefCell;

pub struct PgStore {
    pub pool: PgPool,
}

impl PgStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Tx for Transaction<'static, Postgres> {}

#[async_trait]
impl MultiRepository for PgStore {
    type KTransaction = Transaction<'static, Postgres>;

    async fn tx(&self) -> Result<RefCell<Self::KTransaction>, anyhow::Error> {
        Ok(RefCell::new(
            self.pool
                .begin()
                .await
                .context("Failed to start transaction.")?,
        ))
    }

    async fn commit(&self, tx: Self::KTransaction) -> Result<(), Error> {
        tx.commit().await.context("Failed to commit transaction.")
    }

    fn users(&self) -> &dyn UserRepository<Tr = Self::KTransaction> {
        self
    }

    fn credentials(&self) -> &dyn CredentialRepository<Tr = Self::KTransaction> {
        self
    }

    fn device(&self) -> &dyn DeviceRepository<Tr = Self::KTransaction> {
        self
    }

    fn groups(&self) -> &dyn GroupRepository<Tr = Self::KTransaction> {
        self
    }

    fn members(&self) -> &dyn MemberRepository<Tr = Self::KTransaction> {
        self
    }

    fn expenses(&self) -> &dyn ExpenseRepository<Tr = Self::KTransaction> {
        self
    }

    fn settlements(&self) -> &dyn SettlementRepository<Tr = Self::KTransaction> {
        self
    }

    fn events(&self) -> &dyn EventRepository<Tr = Self::KTransaction> {
        self
    }
}
