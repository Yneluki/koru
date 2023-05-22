use crate::application::store::MultiRepository;
use crate::domain::errors::GetGroupError;
use crate::domain::usecases::dto::dtos::DetailedGroupDto;
use crate::domain::usecases::group::GetGroupRequest;
use anyhow::Context;
use std::sync::Arc;

pub async fn get(
    data: GetGroupRequest,
    store: Arc<impl MultiRepository>,
) -> Result<DetailedGroupDto, GetGroupError> {
    let group = store
        .groups()
        .find(&data.group_id)
        .await
        .context("Failed to fetch group.")
        .map_err(GetGroupError::Unexpected)?;
    // check user is a member of the group
    match group {
        Some(group) => {
            if !group.is_member(&data.user_id) {
                return Err(GetGroupError::Unauthorized("User is not a member."));
            }
            let expenses = store
                .expenses()
                .get_unsettled_expenses(&data.group_id)
                .await
                .context("Failed to get expenses.")
                .map_err(GetGroupError::Unexpected)?;

            Ok(DetailedGroupDto::from(group, expenses))
        }
        None => Err(GetGroupError::NotFound("Group not found.")),
    }
}
