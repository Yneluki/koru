use crate::application::store::MultiRepository;
use crate::domain::errors::GetGroupsError;
use crate::domain::usecases::dto::dtos::GroupDto;
use crate::domain::usecases::group::GetGroupsRequest;
use anyhow::Context;
use std::sync::Arc;

pub async fn get(
    request: GetGroupsRequest,
    store: Arc<impl MultiRepository>,
) -> Result<Vec<GroupDto>, GetGroupsError> {
    let groups = store
        .groups()
        .get_user_groups(&request.user_id)
        .await
        .context("Failed to get user groups.")
        .map_err(GetGroupsError::Unexpected)?;

    Ok(groups.into_iter().map(GroupDto::from).collect())
}
