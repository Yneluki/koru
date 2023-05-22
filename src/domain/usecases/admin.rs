use crate::domain::errors::{GetAllGroupsError, GetUsersError};
use crate::domain::usecases::dto::dtos::{DetailedUserDto, GroupDto};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait(?Send)]
pub trait AdminUseCase {
    async fn get_users(&self, requester: &Uuid) -> Result<Vec<DetailedUserDto>, GetUsersError>;
    async fn get_groups(&self, requester: &Uuid) -> Result<Vec<GroupDto>, GetAllGroupsError>;
}
