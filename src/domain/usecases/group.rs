use crate::domain::errors::{
    ChangeMemberColorError, CreateExpenseError, CreateGroupError, DeleteExpenseError,
    DeleteGroupError, GenerateGroupTokenError, GetExpensesError, GetGroupError, GetGroupsError,
    GetSettlementsError, JoinGroupError, SettlementError, UpdateExpenseError,
};
use crate::domain::usecases::dto::dtos::{
    ColorDto, DetailedGroupDto, ExpenseDto, GroupDto, SettlementDto,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait(?Send)]
pub trait GroupUseCase {
    async fn create_group(&self, request: CreateGroupRequest) -> Result<Uuid, CreateGroupError>;
    async fn join_group(&self, request: JoinGroupRequest) -> Result<(), JoinGroupError>;
    async fn change_member_color(
        &self,
        request: ChangeMemberColorRequest,
    ) -> Result<(), ChangeMemberColorError>;
    async fn generate_token(
        &self,
        request: GenerateGroupTokenRequest,
    ) -> Result<String, GenerateGroupTokenError>;
    async fn delete_group(&self, request: DeleteGroupRequest) -> Result<(), DeleteGroupError>;
    async fn get_group(&self, request: GetGroupRequest) -> Result<DetailedGroupDto, GetGroupError>;
    async fn get_groups(&self, request: GetGroupsRequest) -> Result<Vec<GroupDto>, GetGroupsError>;
    async fn get_expenses(
        &self,
        request: GetExpensesRequest,
    ) -> Result<Vec<ExpenseDto>, GetExpensesError>;
    async fn get_settlements(
        &self,
        request: GetSettlementsRequest,
    ) -> Result<Vec<SettlementDto>, GetSettlementsError>;
    async fn create_expense(
        &self,
        request: CreateExpenseRequest,
    ) -> Result<Uuid, CreateExpenseError>;
    async fn delete_expense(&self, request: DeleteExpenseRequest)
        -> Result<(), DeleteExpenseError>;
    async fn update_expense(&self, request: UpdateExpenseRequest)
        -> Result<(), UpdateExpenseError>;
    async fn settle(&self, request: SettleRequest) -> Result<SettlementDto, SettlementError>;
}

#[derive(Clone)]
pub struct UpdateExpenseRequest {
    pub group_id: Uuid,
    pub expense_id: Uuid,
    pub user_id: Uuid,
    pub description: String,
    pub amount: f32,
}

#[derive(Clone)]
pub struct SettleRequest {
    pub group_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Clone)]
pub struct JoinGroupRequest {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub color: ColorDto,
    pub token: String,
}

#[derive(Clone)]
pub struct GetSettlementsRequest {
    pub group_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Clone)]
pub struct GetGroupsRequest {
    pub user_id: Uuid,
}

#[derive(Clone)]
pub struct GetGroupRequest {
    pub group_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Clone)]
pub struct GetExpensesRequest {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub settlement_id: Option<Uuid>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct GenerateGroupTokenRequest {
    pub group_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Clone)]
pub struct DeleteGroupRequest {
    pub group_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Clone)]
pub struct CreateGroupRequest {
    pub name: String,
    pub admin_id: Uuid,
    pub admin_color: ColorDto,
}

#[derive(Clone)]
pub struct CreateExpenseRequest {
    pub group_id: Uuid,
    pub member_id: Uuid,
    pub title: String,
    pub amount: f32,
}

#[derive(Clone)]
pub struct DeleteExpenseRequest {
    pub group_id: Uuid,
    pub expense_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Clone)]
pub struct ChangeMemberColorRequest {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub color: ColorDto,
}
