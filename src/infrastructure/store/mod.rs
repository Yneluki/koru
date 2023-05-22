pub mod mem;
#[cfg(feature = "postgres")]
mod postgres;

use crate::application::auth::{CredentialRepository, CredentialRepositoryError, UserCredentials};
use crate::application::store::{
    DeviceRepository, DeviceRepositoryError, EventRepository, EventRepositoryError,
    ExpenseRepository, ExpenseRepositoryError, GroupRepository, GroupRepositoryError,
    MemberRepository, MemberRepositoryError, MultiRepository, SettlementRepository,
    SettlementRepositoryError, Tx, UserRepository, UserRepositoryError,
};
use crate::configuration::store::DatabaseSettings;
use crate::domain::{
    Email, Event, Expense, Group, GroupMember, Settlement, SettlementDescription, User,
};
use crate::infrastructure::store::mem::mem_store::InMemTx;
use anyhow::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
pub use mem::mem_store::InMemoryStore;
#[cfg(feature = "postgres")]
pub use postgres::pg_store::PgStore;
#[cfg(feature = "postgres")]
use secrecy::ExposeSecret;
#[cfg(feature = "postgres")]
use sqlx::{PgPool, Postgres, Transaction};
use std::cell::RefCell;
use std::sync::Arc;
use uuid::Uuid;

pub enum StoreImpl {
    #[cfg(feature = "postgres")]
    Postgres(Box<PgStore>),
    Memory(Arc<InMemoryStore>),
}

pub enum TransactionImpl {
    #[cfg(feature = "postgres")]
    Postgres(Box<RefCell<Transaction<'static, Postgres>>>),
    Memory(Box<RefCell<InMemTx>>),
}

impl Tx for TransactionImpl {}

impl StoreImpl {
    pub async fn build(configuration: &DatabaseSettings) -> Result<StoreImpl, anyhow::Error> {
        match configuration {
            #[cfg(feature = "postgres")]
            DatabaseSettings::Postgres(config) => {
                let pool = PgPool::connect(config.connection_string().expose_secret())
                    .await
                    .expect("Failed to connect to Postgres.");
                Ok(StoreImpl::Postgres(Box::new(PgStore::new(pool))))
            }
            DatabaseSettings::Memory => Ok(StoreImpl::Memory(Arc::new(InMemoryStore::new()))),
        }
    }
}

#[async_trait]
impl MultiRepository for StoreImpl {
    type KTransaction = TransactionImpl;

    async fn tx(&self) -> Result<RefCell<Self::KTransaction>, Error> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p
                .tx()
                .await
                .map(|tx| RefCell::new(TransactionImpl::Postgres(Box::new(tx)))),
            StoreImpl::Memory(m) => m
                .tx()
                .await
                .map(|tx| RefCell::new(TransactionImpl::Memory(Box::new(tx)))),
        }
    }

    #[allow(unreachable_patterns)]
    async fn commit(&self, tx: Self::KTransaction) -> Result<(), Error> {
        match (self, tx) {
            #[cfg(feature = "postgres")]
            (StoreImpl::Postgres(p), TransactionImpl::Postgres(tx)) => {
                p.commit(tx.into_inner()).await
            }
            (StoreImpl::Memory(m), TransactionImpl::Memory(tx)) => m.commit(tx.into_inner()).await,
            (_, _) => panic!("Tried to pass non-matching store & transaction !!"),
        }
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

#[async_trait]
impl EventRepository for StoreImpl {
    type Tr = TransactionImpl;

    #[allow(unreachable_patterns)]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        events: &[Event],
    ) -> Result<(), EventRepositoryError> {
        match (self, tx.get_mut()) {
            #[cfg(feature = "postgres")]
            (StoreImpl::Postgres(p), TransactionImpl::Postgres(tx)) => {
                p.events().save(tx, events).await
            }
            (StoreImpl::Memory(m), TransactionImpl::Memory(tx)) => {
                m.events().save(tx, events).await
            }
            (_, _) => panic!("Tried to pass non-matching store & transaction !!"),
        }
    }

    async fn find(&self, id: &Uuid) -> Result<Option<Event>, EventRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.events().find(id).await,
            StoreImpl::Memory(m) => m.events().find(id).await,
        }
    }

    async fn mark_processed(&self, id: &Uuid) -> Result<(), EventRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.events().mark_processed(id).await,
            StoreImpl::Memory(m) => m.events().mark_processed(id).await,
        }
    }
}

#[async_trait]
impl UserRepository for StoreImpl {
    type Tr = TransactionImpl;

    #[allow(unreachable_patterns)]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        user: &User,
    ) -> Result<(), UserRepositoryError> {
        match (self, tx.get_mut()) {
            #[cfg(feature = "postgres")]
            (StoreImpl::Postgres(p), TransactionImpl::Postgres(tx)) => {
                p.users().save(tx, user).await
            }
            (StoreImpl::Memory(m), TransactionImpl::Memory(tx)) => m.users().save(tx, user).await,
            (_, _) => panic!("Tried to pass non-matching store & transaction !!"),
        }
    }

    async fn find(&self, user_id: &Uuid) -> Result<Option<User>, UserRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.users().find(user_id).await,
            StoreImpl::Memory(m) => m.users().find(user_id).await,
        }
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, UserRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.users().find_by_email(email).await,
            StoreImpl::Memory(m) => m.users().find_by_email(email).await,
        }
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, UserRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.users().exists_by_email(email).await,
            StoreImpl::Memory(m) => m.users().exists_by_email(email).await,
        }
    }

    async fn fetch_users(&self, user_ids: &[Uuid]) -> Result<Vec<User>, UserRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.users().fetch_users(user_ids).await,
            StoreImpl::Memory(m) => m.users().fetch_users(user_ids).await,
        }
    }

    async fn fetch_all_users(&self) -> Result<Vec<User>, UserRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.users().fetch_all_users().await,
            StoreImpl::Memory(m) => m.users().fetch_all_users().await,
        }
    }
}

#[async_trait]
impl CredentialRepository for StoreImpl {
    type Tr = TransactionImpl;

    #[allow(unreachable_patterns)]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        user_credentials: &UserCredentials,
    ) -> Result<(), CredentialRepositoryError> {
        match (self, tx.get_mut()) {
            #[cfg(feature = "postgres")]
            (StoreImpl::Postgres(p), TransactionImpl::Postgres(tx)) => {
                p.credentials().save(tx, user_credentials).await
            }
            (StoreImpl::Memory(m), TransactionImpl::Memory(tx)) => {
                m.credentials().save(tx, user_credentials).await
            }
            (_, _) => panic!("Tried to pass non-matching store & transaction !!"),
        }
    }

    async fn fetch_by_email(
        &self,
        email: &Email,
    ) -> Result<Option<UserCredentials>, CredentialRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.credentials().fetch_by_email(email).await,
            StoreImpl::Memory(m) => m.credentials().fetch_by_email(email).await,
        }
    }
}

#[async_trait]
impl DeviceRepository for StoreImpl {
    type Tr = TransactionImpl;

    async fn fetch_device(&self, user_id: &Uuid) -> Result<Option<String>, DeviceRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.device().fetch_device(user_id).await,
            StoreImpl::Memory(m) => m.device().fetch_device(user_id).await,
        }
    }

    async fn save_device(
        &self,
        user_id: &Uuid,
        device_id: String,
    ) -> Result<(), DeviceRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.device().save_device(user_id, device_id).await,
            StoreImpl::Memory(m) => m.device().save_device(user_id, device_id).await,
        }
    }

    async fn remove_device(&self, user_id: &Uuid) -> Result<(), DeviceRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.device().remove_device(user_id).await,
            StoreImpl::Memory(m) => m.device().remove_device(user_id).await,
        }
    }
}

#[async_trait]
impl GroupRepository for StoreImpl {
    type Tr = TransactionImpl;

    #[allow(unreachable_patterns)]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        group: &Group,
    ) -> Result<(), GroupRepositoryError> {
        match (self, tx.get_mut()) {
            #[cfg(feature = "postgres")]
            (StoreImpl::Postgres(p), TransactionImpl::Postgres(tx)) => {
                p.groups().save(tx, group).await
            }
            (StoreImpl::Memory(m), TransactionImpl::Memory(tx)) => m.groups().save(tx, group).await,
            (_, _) => panic!("Tried to pass non-matching store & transaction !!"),
        }
    }

    #[allow(unreachable_patterns)]
    async fn delete(
        &self,
        tx: &mut RefCell<Self::Tr>,
        group_id: &Uuid,
    ) -> Result<(), GroupRepositoryError> {
        match (self, tx.get_mut()) {
            #[cfg(feature = "postgres")]
            (StoreImpl::Postgres(p), TransactionImpl::Postgres(tx)) => {
                p.groups().delete(tx, group_id).await
            }
            (StoreImpl::Memory(m), TransactionImpl::Memory(tx)) => {
                m.groups().delete(tx, group_id).await
            }
            (_, _) => panic!("Tried to pass non-matching store & transaction !!"),
        }
    }

    async fn find(&self, group_id: &Uuid) -> Result<Option<Group>, GroupRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.groups().find(group_id).await,
            StoreImpl::Memory(m) => m.groups().find(group_id).await,
        }
    }

    async fn get_user_groups(&self, user_id: &Uuid) -> Result<Vec<Group>, GroupRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.groups().get_user_groups(user_id).await,
            StoreImpl::Memory(m) => m.groups().get_user_groups(user_id).await,
        }
    }

    async fn fetch_all_groups(&self) -> Result<Vec<Group>, GroupRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.groups().fetch_all_groups().await,
            StoreImpl::Memory(m) => m.groups().fetch_all_groups().await,
        }
    }
}

#[async_trait]
impl MemberRepository for StoreImpl {
    type Tr = TransactionImpl;

    #[allow(unreachable_patterns)]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        member: &GroupMember,
    ) -> Result<(), MemberRepositoryError> {
        match (self, tx.get_mut()) {
            #[cfg(feature = "postgres")]
            (StoreImpl::Postgres(p), TransactionImpl::Postgres(tx)) => {
                p.members().save(tx, member).await
            }
            (StoreImpl::Memory(m), TransactionImpl::Memory(tx)) => {
                m.members().save(tx, member).await
            }
            (_, _) => panic!("Tried to pass non-matching store & transaction !!"),
        }
    }

    async fn fetch_members(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<GroupMember>, MemberRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.members().fetch_members(group_id).await,
            StoreImpl::Memory(m) => m.members().fetch_members(group_id).await,
        }
    }
}

#[async_trait]
impl ExpenseRepository for StoreImpl {
    type Tr = TransactionImpl;

    #[allow(unreachable_patterns)]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        expense: &Expense,
    ) -> Result<(), ExpenseRepositoryError> {
        match (self, tx.get_mut()) {
            #[cfg(feature = "postgres")]
            (StoreImpl::Postgres(p), TransactionImpl::Postgres(tx)) => {
                p.expenses().save(tx, expense).await
            }
            (StoreImpl::Memory(m), TransactionImpl::Memory(tx)) => {
                m.expenses().save(tx, expense).await
            }
            (_, _) => panic!("Tried to pass non-matching store & transaction !!"),
        }
    }

    #[allow(unreachable_patterns)]
    async fn delete(
        &self,
        tx: &mut RefCell<Self::Tr>,
        expense_id: &Uuid,
    ) -> Result<(), ExpenseRepositoryError> {
        match (self, tx.get_mut()) {
            #[cfg(feature = "postgres")]
            (StoreImpl::Postgres(p), TransactionImpl::Postgres(tx)) => {
                p.expenses().delete(tx, expense_id).await
            }
            (StoreImpl::Memory(m), TransactionImpl::Memory(tx)) => {
                m.expenses().delete(tx, expense_id).await
            }
            (_, _) => panic!("Tried to pass non-matching store & transaction !!"),
        }
    }

    async fn find(&self, expense_id: &Uuid) -> Result<Option<Expense>, ExpenseRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.expenses().find(expense_id).await,
            StoreImpl::Memory(m) => m.expenses().find(expense_id).await,
        }
    }

    async fn get_expenses(
        &self,
        group_id: &Uuid,
        start_date: Option<&DateTime<Utc>>,
        end_date: Option<&DateTime<Utc>>,
    ) -> Result<Vec<Expense>, ExpenseRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => {
                p.expenses()
                    .get_expenses(group_id, start_date, end_date)
                    .await
            }
            StoreImpl::Memory(m) => {
                m.expenses()
                    .get_expenses(group_id, start_date, end_date)
                    .await
            }
        }
    }

    async fn get_expenses_by_id(
        &self,
        expense_ids: &[Uuid],
    ) -> Result<Vec<Expense>, ExpenseRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.expenses().get_expenses_by_id(expense_ids).await,
            StoreImpl::Memory(m) => m.expenses().get_expenses_by_id(expense_ids).await,
        }
    }

    async fn get_unsettled_expenses(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<Expense>, ExpenseRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.expenses().get_unsettled_expenses(group_id).await,
            StoreImpl::Memory(m) => m.expenses().get_unsettled_expenses(group_id).await,
        }
    }
}

#[async_trait]
impl SettlementRepository for StoreImpl {
    type Tr = TransactionImpl;

    #[allow(unreachable_patterns)]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        settlement: &Settlement,
    ) -> Result<(), SettlementRepositoryError> {
        match (self, tx.get_mut()) {
            #[cfg(feature = "postgres")]
            (StoreImpl::Postgres(p), TransactionImpl::Postgres(tx)) => {
                p.settlements().save(tx, settlement).await
            }
            (StoreImpl::Memory(m), TransactionImpl::Memory(tx)) => {
                m.settlements().save(tx, settlement).await
            }
            (_, _) => panic!("Tried to pass non-matching store & transaction !!"),
        }
    }

    async fn get_settlements(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<Settlement>, SettlementRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.settlements().get_settlements(group_id).await,
            StoreImpl::Memory(m) => m.settlements().get_settlements(group_id).await,
        }
    }

    async fn get_settlement_description(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Option<SettlementDescription>, SettlementRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => {
                p.settlements()
                    .get_settlement_description(settlement_id)
                    .await
            }
            StoreImpl::Memory(m) => {
                m.settlements()
                    .get_settlement_description(settlement_id)
                    .await
            }
        }
    }

    async fn find(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Option<Settlement>, SettlementRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.settlements().find(settlement_id).await,
            StoreImpl::Memory(m) => m.settlements().find(settlement_id).await,
        }
    }

    async fn exists(&self, settlement_id: &Uuid) -> Result<bool, SettlementRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.settlements().exists(settlement_id).await,
            StoreImpl::Memory(m) => m.settlements().exists(settlement_id).await,
        }
    }

    async fn get_expenses(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Vec<Uuid>, SettlementRepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            StoreImpl::Postgres(p) => p.settlements().get_expenses(settlement_id).await,
            StoreImpl::Memory(m) => m.settlements().get_expenses(settlement_id).await,
        }
    }
}
