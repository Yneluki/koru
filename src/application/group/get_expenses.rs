use crate::application::store::MultiRepository;
use crate::domain::errors::GetExpensesError;
use crate::domain::usecases::dto::dtos::ExpenseDto;
use crate::domain::usecases::group::GetExpensesRequest;
use anyhow::Context;
use itertools::Itertools;
use std::sync::Arc;

pub async fn get(
    data: GetExpensesRequest,
    store: Arc<impl MultiRepository>,
) -> Result<Vec<ExpenseDto>, GetExpensesError> {
    let group = store
        .groups()
        .find(&data.group_id)
        .await
        .context("Failed to fetch group.")
        .map_err(GetExpensesError::Unexpected)?;
    // check user is a member of the group
    match &group {
        Some(group) => {
            if !group.is_member(&data.user_id) {
                return Err(GetExpensesError::Unauthorized("User is not a member."));
            }
        }
        None => return Err(GetExpensesError::NotFound("Group not found.")),
    }
    // if using settlement filter, check it exists
    match &data.settlement_id {
        None => {}
        Some(stl_id) => {
            let exists = store
                .settlements()
                .exists(stl_id)
                .await
                .context("Failed to find settlement.")
                .map_err(GetExpensesError::Unexpected)?;
            if !exists {
                return Err(GetExpensesError::NotFound("Settlement not found."));
            }
        }
    }
    // fetch expenses according to filters
    let expenses = match (data.from.as_ref(), data.to.as_ref(), &data.settlement_id) {
        (None, None, None) => store
            .expenses()
            .get_unsettled_expenses(&data.group_id)
            .await
            .context("Failed to fetch expenses")
            .map_err(GetExpensesError::Unexpected)?,
        (from, to, None) => store
            .expenses()
            .get_expenses(&data.group_id, from, to)
            .await
            .context("Failed to fetch expenses")
            .map_err(GetExpensesError::Unexpected)?,
        (_, _, Some(stl_id)) => {
            let expense_ids = store
                .settlements()
                .get_expenses(stl_id)
                .await
                .context("Failed to get expenses ids.")
                .map_err(GetExpensesError::Unexpected)?;
            store
                .expenses()
                .get_expenses_by_id(&expense_ids)
                .await
                .context("Failed to fetch expenses")
                .map_err(GetExpensesError::Unexpected)?
        }
    };

    // fetch user infos
    let members = group.unwrap().members;

    // build response
    Ok(expenses
        .into_iter()
        .map(|e| {
            let member = members
                .iter()
                .find(|m| m.id == e.member_id)
                .cloned()
                .unwrap_or_default();
            ExpenseDto::from(e, member)
        })
        .sorted_by(|a, b| {
            b.date
                .partial_cmp(&a.date)
                .expect("expenses dates to be comparable")
        })
        .collect())
}
