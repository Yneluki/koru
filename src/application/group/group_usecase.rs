use crate::application::event_bus::EventBus;
use crate::application::group::change_member_color::change_color;
use crate::application::group::create_expense::create as create_expense;
use crate::application::group::create_group::create;
use crate::application::group::delete_expense::delete as delete_expense;
use crate::application::group::delete_group::delete;
use crate::application::group::generate_token::generate;
use crate::application::group::get_expenses::get as get_expenses;
use crate::application::group::get_group::get as get_group;
use crate::application::group::get_groups::get as get_groups;
use crate::application::group::get_settlements::get as get_settlements;
use crate::application::group::join_group::join;
use crate::application::group::settle::execute;
use crate::application::group::update_expense::update;
use crate::application::store::MultiRepository;
use crate::application::user::UserUsecase;
use crate::domain::errors::{
    ChangeMemberColorError, CreateExpenseError, CreateGroupError, DeleteExpenseError,
    DeleteGroupError, GenerateGroupTokenError, GetExpensesError, GetGroupError, GetGroupsError,
    GetSettlementsError, JoinGroupError, SettlementError, UpdateExpenseError,
};
use crate::domain::usecases::dto::dtos::{
    DetailedGroupDto, ExpenseDto, GroupDto, SettlementDto, TransactionDto,
};
use crate::domain::usecases::group::{
    ChangeMemberColorRequest, CreateExpenseRequest, CreateGroupRequest, DeleteExpenseRequest,
    DeleteGroupRequest, GenerateGroupTokenRequest, GetExpensesRequest, GetGroupRequest,
    GetGroupsRequest, GetSettlementsRequest, GroupUseCase, JoinGroupRequest, SettleRequest,
    UpdateExpenseRequest,
};
use crate::domain::usecases::user::UserUseCase;
use crate::domain::GroupEventKind::{ExpenseDeleted, GroupDeleted};
use crate::domain::{Event, Expense, Group, Settlement, TokenGenerator};
use anyhow::Context;
use async_trait::async_trait;
use itertools::Itertools;
use log::warn;
use std::sync::Arc;
use uuid::Uuid;

pub struct GroupUsecase<Store: MultiRepository> {
    store: Arc<Store>,
    event_bus: Arc<dyn EventBus>,
    token_generator: Arc<dyn TokenGenerator>,
    users: Arc<UserUsecase<Store>>,
}

impl<Store: MultiRepository> GroupUsecase<Store> {
    pub fn new(
        store: Arc<Store>,
        event_bus: Arc<dyn EventBus>,
        token_svc: Arc<dyn TokenGenerator>,
        users: Arc<UserUsecase<Store>>,
    ) -> Self {
        Self {
            store,
            event_bus,
            token_generator: token_svc,
            users,
        }
    }

    async fn finalize_settlement(
        &self,
        group: &Group,
        settlement: &Settlement,
        expenses: &[Expense],
    ) -> Result<(), anyhow::Error> {
        self.save_settlement(settlement, group, expenses).await?;
        self.publish(group).await;
        Ok(())
    }

    async fn finalize_expense(
        &self,
        group: &Group,
        expense: &Expense,
    ) -> Result<(), anyhow::Error> {
        self.save_expense(expense, group).await?;
        self.publish(group).await;
        Ok(())
    }

    async fn finalize(&self, group: &Group) -> Result<(), anyhow::Error> {
        self.save(group).await?;
        self.publish(group).await;
        Ok(())
    }

    async fn publish(&self, group: &Group) {
        self.event_bus
            .publish(&group.events.iter().map(|e| e.id).collect_vec())
            .await
            .context("Failed to notify event bus.")
            .unwrap_or_else(|failure| {
                warn!("{:?}", failure);
            });
    }

    async fn save_expense(&self, expense: &Expense, group: &Group) -> Result<(), anyhow::Error> {
        let mut tx = self.store.tx().await?;
        self.store
            .groups()
            .save(&mut tx, group)
            .await
            .context("Failed to insert group")?;
        if group
            .events
            .iter()
            .any(|e| matches!(e.event, ExpenseDeleted { .. }))
        {
            self.store
                .expenses()
                .delete(&mut tx, &expense.id)
                .await
                .context("Failed to delete expense")?;
        } else {
            self.store
                .expenses()
                .save(&mut tx, expense)
                .await
                .context("Failed to insert expense")?;
        }
        self.store
            .events()
            .save(
                &mut tx,
                &group.events.iter().cloned().map(Event::Group).collect_vec(),
            )
            .await
            .context("Failed to insert event")?;
        self.store.commit(tx.into_inner()).await?;
        Ok(())
    }

    async fn save_settlement(
        &self,
        settlement: &Settlement,
        group: &Group,
        expenses: &[Expense],
    ) -> Result<(), anyhow::Error> {
        let mut tx = self.store.tx().await?;
        self.store
            .settlements()
            .save(&mut tx, settlement)
            .await
            .context("Failed to insert settlement")?;
        self.store
            .groups()
            .save(&mut tx, group)
            .await
            .context("Failed to insert group")?;
        for expense in expenses {
            self.store
                .expenses()
                .save(&mut tx, expense)
                .await
                .context("Failed to insert expense")?;
        }
        self.store
            .events()
            .save(
                &mut tx,
                &group.events.iter().cloned().map(Event::Group).collect_vec(),
            )
            .await
            .context("Failed to insert event")?;
        self.store.commit(tx.into_inner()).await?;
        Ok(())
    }

    async fn save(&self, group: &Group) -> Result<(), anyhow::Error> {
        let mut tx = self.store.tx().await?;
        if group.events.iter().any(|e| matches!(e.event, GroupDeleted)) {
            self.store
                .groups()
                .delete(&mut tx, &group.id)
                .await
                .context("Failed to delete group")?;
        } else {
            self.store
                .groups()
                .save(&mut tx, group)
                .await
                .context("Failed to insert group")?;
            for member in group.members.iter() {
                self.store
                    .members()
                    .save(&mut tx, member)
                    .await
                    .context("Failed to insert member")?;
            }
        }
        self.store
            .events()
            .save(
                &mut tx,
                &group.events.iter().cloned().map(Event::Group).collect_vec(),
            )
            .await
            .context("Failed to insert event")?;
        self.store.commit(tx.into_inner()).await?;
        Ok(())
    }
}

#[async_trait(?Send)]
impl<Store: MultiRepository> GroupUseCase for GroupUsecase<Store> {
    async fn create_group(&self, request: CreateGroupRequest) -> Result<Uuid, CreateGroupError> {
        if !self.users.is_valid_user(&request.admin_id).await? {
            return Err(CreateGroupError::Unauthenticated());
        }
        let group = create(request, self.store.clone()).await?;
        self.finalize(&group)
            .await
            .map_err(CreateGroupError::Unexpected)?;
        Ok(group.id)
    }
    async fn join_group(&self, request: JoinGroupRequest) -> Result<(), JoinGroupError> {
        if !self.users.is_valid_user(&request.user_id).await? {
            return Err(JoinGroupError::Unauthenticated());
        }
        let group = join(request, self.store.clone(), self.token_generator.clone()).await?;
        self.finalize(&group)
            .await
            .map_err(JoinGroupError::Unexpected)?;
        Ok(())
    }
    async fn change_member_color(
        &self,
        request: ChangeMemberColorRequest,
    ) -> Result<(), ChangeMemberColorError> {
        if !self.users.is_valid_user(&request.user_id).await? {
            return Err(ChangeMemberColorError::Unauthenticated());
        }
        let group = change_color(request, self.store.clone()).await?;
        self.finalize(&group)
            .await
            .map_err(ChangeMemberColorError::Unexpected)?;
        Ok(())
    }
    async fn generate_token(
        &self,
        request: GenerateGroupTokenRequest,
    ) -> Result<String, GenerateGroupTokenError> {
        if !self.users.is_valid_user(&request.user_id).await? {
            return Err(GenerateGroupTokenError::Unauthenticated());
        }
        generate(request, self.store.clone(), self.token_generator.clone()).await
    }
    async fn delete_group(&self, request: DeleteGroupRequest) -> Result<(), DeleteGroupError> {
        if !self.users.is_valid_user(&request.user_id).await? {
            return Err(DeleteGroupError::Unauthenticated());
        }
        let group = delete(request, self.store.clone()).await?;
        self.finalize(&group)
            .await
            .map_err(DeleteGroupError::Unexpected)?;
        Ok(())
    }
    async fn get_group(&self, request: GetGroupRequest) -> Result<DetailedGroupDto, GetGroupError> {
        if !self.users.is_valid_user(&request.user_id).await? {
            return Err(GetGroupError::Unauthenticated());
        }
        get_group(request, self.store.clone()).await
    }
    async fn get_groups(&self, request: GetGroupsRequest) -> Result<Vec<GroupDto>, GetGroupsError> {
        if !self.users.is_valid_user(&request.user_id).await? {
            return Err(GetGroupsError::Unauthenticated());
        }
        get_groups(request, self.store.clone()).await
    }
    async fn get_expenses(
        &self,
        request: GetExpensesRequest,
    ) -> Result<Vec<ExpenseDto>, GetExpensesError> {
        if !self.users.is_valid_user(&request.user_id).await? {
            return Err(GetExpensesError::Unauthenticated());
        }
        get_expenses(request, self.store.clone()).await
    }
    async fn get_settlements(
        &self,
        request: GetSettlementsRequest,
    ) -> Result<Vec<SettlementDto>, GetSettlementsError> {
        if !self.users.is_valid_user(&request.user_id).await? {
            return Err(GetSettlementsError::Unauthenticated());
        }
        get_settlements(request, self.store.clone()).await
    }
    async fn create_expense(
        &self,
        request: CreateExpenseRequest,
    ) -> Result<Uuid, CreateExpenseError> {
        if !self.users.is_valid_user(&request.member_id).await? {
            return Err(CreateExpenseError::Unauthenticated());
        }
        let (group, expense) = create_expense(request, self.store.clone()).await?;
        self.finalize_expense(&group, &expense)
            .await
            .map_err(CreateExpenseError::Unexpected)?;
        Ok(expense.id)
    }
    async fn delete_expense(
        &self,
        request: DeleteExpenseRequest,
    ) -> Result<(), DeleteExpenseError> {
        if !self.users.is_valid_user(&request.user_id).await? {
            return Err(DeleteExpenseError::Unauthenticated());
        }
        let (group, expense) = delete_expense(request, self.store.clone()).await?;
        self.finalize_expense(&group, &expense)
            .await
            .map_err(DeleteExpenseError::Unexpected)?;
        Ok(())
    }
    async fn update_expense(
        &self,
        request: UpdateExpenseRequest,
    ) -> Result<(), UpdateExpenseError> {
        if !self.users.is_valid_user(&request.user_id).await? {
            return Err(UpdateExpenseError::Unauthenticated());
        }
        let (group, expense) = update(request, self.store.clone()).await?;
        self.finalize_expense(&group, &expense)
            .await
            .map_err(UpdateExpenseError::Unexpected)?;
        Ok(())
    }
    async fn settle(&self, request: SettleRequest) -> Result<SettlementDto, SettlementError> {
        if !self.users.is_valid_user(&request.user_id).await? {
            return Err(SettlementError::Unauthenticated());
        }
        let (group, settlement, expenses) = execute(request, self.store.clone()).await?;
        self.finalize_settlement(&group, &settlement, &expenses)
            .await
            .map_err(SettlementError::Unexpected)?;
        Ok(SettlementDto {
            id: settlement.id,
            start_date: settlement.start_date,
            end_date: settlement.end_date,
            transactions: TransactionDto::from_vec(settlement.transactions, &group.members),
        })
    }
}
