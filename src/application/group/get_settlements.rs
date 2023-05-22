use crate::application::store::MultiRepository;
use crate::domain::errors::GetSettlementsError;
use crate::domain::usecases::dto::dtos::{SettlementDto, TransactionDto};
use crate::domain::usecases::group::GetSettlementsRequest;
use anyhow::Context;
use itertools::Itertools;
use std::sync::Arc;

pub async fn get(
    data: GetSettlementsRequest,
    store: Arc<impl MultiRepository>,
) -> Result<Vec<SettlementDto>, GetSettlementsError> {
    let group = store
        .groups()
        .find(&data.group_id)
        .await
        .context("Failed to fetch group.")
        .map_err(GetSettlementsError::Unexpected)?;
    // check user is a member of the group
    match &group {
        Some(group) => {
            if !group.is_member(&data.user_id) {
                return Err(GetSettlementsError::Unauthorized("User is not a member."));
            }
        }
        None => return Err(GetSettlementsError::NotFound("Group not found.")),
    }
    let settlements = store
        .settlements()
        .get_settlements(&data.group_id)
        .await
        .context("Failed to get settlements.")
        .map_err(GetSettlementsError::Unexpected)?;

    let members = group.unwrap().members;

    Ok(settlements
        .into_iter()
        .map(|settlement| SettlementDto {
            id: settlement.id,
            start_date: settlement.start_date,
            end_date: settlement.end_date,
            transactions: TransactionDto::from_vec(settlement.transactions, &members),
        })
        .sorted_by(|a, b| {
            b.end_date
                .partial_cmp(&a.end_date)
                .expect("settlement end dates to be comparable")
        })
        .collect())
}
