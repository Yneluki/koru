use crate::application::auth::CredentialRepository;
use crate::domain::{
    Email, Event, Expense, Group, GroupMember, Settlement, SettlementDescription, User,
};
use crate::error_chain;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::cell::RefCell;
use uuid::Uuid;

pub trait Tx {}

#[async_trait]
pub trait MultiRepository: Send + Sync {
    type KTransaction: Tx;
    async fn tx(&self) -> Result<RefCell<Self::KTransaction>, anyhow::Error>;
    async fn commit(&self, mut tx: Self::KTransaction) -> Result<(), anyhow::Error>;
    fn users(&self) -> &dyn UserRepository<Tr = Self::KTransaction>;
    fn credentials(&self) -> &dyn CredentialRepository<Tr = Self::KTransaction>;
    #[cfg(feature = "pushy")]
    fn device(&self) -> &dyn DeviceRepository<Tr = Self::KTransaction>;
    fn groups(&self) -> &dyn GroupRepository<Tr = Self::KTransaction>;
    fn members(&self) -> &dyn MemberRepository<Tr = Self::KTransaction>;
    fn expenses(&self) -> &dyn ExpenseRepository<Tr = Self::KTransaction>;
    fn settlements(&self) -> &dyn SettlementRepository<Tr = Self::KTransaction>;
    fn events(&self) -> &dyn EventRepository<Tr = Self::KTransaction>;
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum EventRepositoryError {
        #[error("Failed to insert event.")]
        Insert(#[source] anyhow::Error),
        #[error("Failed to fetch events.")]
        Fetch(#[source] anyhow::Error),
        #[error("Failed to update events.")]
        Update(#[source] anyhow::Error),
        #[error("Corrupted data in DB: {0}")]
        CorruptedData(&'static str),
    }
}

#[async_trait]
pub trait EventRepository: Send + Sync {
    type Tr: Tx;

    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        events: &[Event],
    ) -> Result<(), EventRepositoryError>;

    async fn find(&self, id: &Uuid) -> Result<Option<Event>, EventRepositoryError>;

    async fn mark_processed(&self, id: &Uuid) -> Result<(), EventRepositoryError>;
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum ExpenseRepositoryError {
        #[error("Failed to insert expense.")]
        Insert(#[source] anyhow::Error),
        #[error("Failed to fetch expense.")]
        Fetch(#[source] anyhow::Error),
        #[error("Failed to delete expense.")]
        Delete(#[source] anyhow::Error),
        #[error("Failed to update expense.")]
        Update(#[source] anyhow::Error),
        #[error("Corrupted data in DB: {0}")]
        CorruptedData(&'static str),
    }
}

#[async_trait]
pub trait ExpenseRepository: Send + Sync {
    type Tr: Tx;

    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        expense: &Expense,
    ) -> Result<(), ExpenseRepositoryError>;

    async fn delete(
        &self,
        tx: &mut RefCell<Self::Tr>,
        expense_id: &Uuid,
    ) -> Result<(), ExpenseRepositoryError>;

    async fn find(&self, expense_id: &Uuid) -> Result<Option<Expense>, ExpenseRepositoryError>;

    async fn get_expenses(
        &self,
        group_id: &Uuid,
        start_date: Option<&DateTime<Utc>>,
        end_date: Option<&DateTime<Utc>>,
    ) -> Result<Vec<Expense>, ExpenseRepositoryError>;

    async fn get_expenses_by_id(
        &self,
        expense_ids: &[Uuid],
    ) -> Result<Vec<Expense>, ExpenseRepositoryError>;

    async fn get_unsettled_expenses(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<Expense>, ExpenseRepositoryError>;
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum GroupRepositoryError {
        #[error("Failed to insert group.")]
        Insert(#[source] anyhow::Error),
        #[error("Failed to fetch group.")]
        Fetch(#[source] anyhow::Error),
        #[error("Failed to delete group.")]
        Delete(#[source] anyhow::Error),
        #[error("Corrupted data in DB: {0}")]
        CorruptedData(&'static str),
    }
}

#[async_trait]
pub trait GroupRepository: Send + Sync {
    type Tr: Tx;

    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        group: &Group,
    ) -> Result<(), GroupRepositoryError>;

    async fn delete(
        &self,
        tx: &mut RefCell<Self::Tr>,
        group_id: &Uuid,
    ) -> Result<(), GroupRepositoryError>;

    async fn find(&self, group_id: &Uuid) -> Result<Option<Group>, GroupRepositoryError>;

    async fn get_user_groups(&self, user_id: &Uuid) -> Result<Vec<Group>, GroupRepositoryError>;

    async fn fetch_all_groups(&self) -> Result<Vec<Group>, GroupRepositoryError>;
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum MemberRepositoryError {
        #[error("Failed to insert member.")]
        Insert(#[source] anyhow::Error),
        #[error("Failed to fetch member.")]
        Fetch(#[source] anyhow::Error),
        #[error("Corrupted data in DB: {0}")]
        CorruptedData(&'static str),
    }
}

#[async_trait]
pub trait MemberRepository: Send + Sync {
    type Tr: Tx;

    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        member: &GroupMember,
    ) -> Result<(), MemberRepositoryError>;

    async fn fetch_members(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<GroupMember>, MemberRepositoryError>;
}

#[cfg(feature = "pushy")]
error_chain! {
    #[derive(thiserror::Error)]
    pub enum DeviceRepositoryError {
        #[error("Failed to insert device.")]
        Insert(#[source] anyhow::Error),
        #[error("Failed to fetch device.")]
        Fetch(#[source] anyhow::Error),
        #[error("Corrupted data in DB: {0}")]
        CorruptedData(&'static str),
    }
}

#[cfg(feature = "pushy")]
#[async_trait]
pub trait DeviceRepository: Send + Sync {
    type Tr: Tx;

    async fn fetch_device(&self, user_id: &Uuid) -> Result<Option<String>, DeviceRepositoryError>;

    async fn save_device(
        &self,
        user_id: &Uuid,
        device_id: String,
    ) -> Result<(), DeviceRepositoryError>;

    async fn remove_device(&self, user_id: &Uuid) -> Result<(), DeviceRepositoryError>;
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum SettlementRepositoryError {
        #[error("Failed to insert settlement.")]
        Insert(#[source] anyhow::Error),
        #[error("Failed to fetch settlement.")]
        Fetch(#[source] anyhow::Error),
        #[error("Failed to delete settlement.")]
        Delete(#[source] anyhow::Error),
        #[error("Corrupted data in DB: {0}")]
        CorruptedData(&'static str),
    }
}

#[async_trait]
pub trait SettlementRepository: Send + Sync {
    type Tr: Tx;

    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        settlement: &Settlement,
    ) -> Result<(), SettlementRepositoryError>;

    async fn get_settlements(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<Settlement>, SettlementRepositoryError>;

    async fn get_settlement_description(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Option<SettlementDescription>, SettlementRepositoryError>;

    async fn find(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Option<Settlement>, SettlementRepositoryError>;

    async fn exists(&self, settlement_id: &Uuid) -> Result<bool, SettlementRepositoryError>;

    async fn get_expenses(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Vec<Uuid>, SettlementRepositoryError>;
}

error_chain! {
    #[derive(thiserror::Error)]
    pub enum UserRepositoryError {
        #[error("Failed to insert user.")]
        Insert(#[source] anyhow::Error),
        #[error("Failed to fetch user.")]
        Fetch(#[source] anyhow::Error),
        #[error("Corrupted data in DB: {0}")]
        CorruptedData(&'static str),
    }
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    type Tr: Tx;

    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        user: &User,
    ) -> Result<(), UserRepositoryError>;

    async fn find(&self, id: &Uuid) -> Result<Option<User>, UserRepositoryError>;

    async fn exists_by_email(&self, email: &Email) -> Result<bool, UserRepositoryError>;

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, UserRepositoryError>;

    async fn fetch_users(&self, user_ids: &[Uuid]) -> Result<Vec<User>, UserRepositoryError>;

    async fn fetch_all_users(&self) -> Result<Vec<User>, UserRepositoryError>;
}
