use crate::application::store::MultiRepository;
use crate::domain::errors::SettlementError;
use crate::domain::usecases::group::SettleRequest;
use crate::domain::{Expense, Group, Settlement};
use anyhow::Context;
use std::sync::Arc;

pub async fn execute(
    data: SettleRequest,
    store: Arc<impl MultiRepository>,
) -> Result<(Group, Settlement, Vec<Expense>), SettlementError> {
    let group = store
        .groups()
        .find(&data.group_id)
        .await
        .context("Failed to fetch group.")
        .map_err(SettlementError::Unexpected)?;
    match group {
        Some(mut group) => {
            let mut expenses = store
                .expenses()
                .get_expenses_by_id(&group.expense_ids)
                .await
                .context("Failed to fetch expenses.")
                .map_err(SettlementError::Unexpected)?;
            let last_settlement = match group.settlement_ids.last() {
                Some(id) => store
                    .settlements()
                    .get_settlement_description(id)
                    .await
                    .context("Failed to fetch settlement.")
                    .map_err(SettlementError::Unexpected)?,
                None => None,
            };
            let settlement = group.settle(&mut expenses, last_settlement, data.user_id)?;
            Ok((group, settlement, expenses))
        }
        None => Err(SettlementError::NotFound("Group not found.")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::tests::TestContext;
    use crate::domain::usecases::group::GroupUseCase;
    use crate::infrastructure::store::mem::mem_store::InnerEventKind;
    use claim::{assert_err, assert_ok, assert_some};
    use uuid::Uuid;

    #[tokio::test]
    async fn it_should_settle_when_user_is_admin() {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member_1 = ctx.with_member(&mut group).await;
        let member_2 = ctx.with_member(&mut group).await;
        let member_3 = ctx.with_member(&mut group).await;
        let admin = group.admin_id;
        let expense_1 = ctx.with_expense_of(&mut group, 10.0, member_1.id).await;
        let expense_2 = ctx.with_expense_of(&mut group, 10.0, member_1.id).await;
        let expense_3 = ctx.with_expense_of(&mut group, 5.0, member_2.id).await;
        let expense_4 = ctx.with_expense_of(&mut group, 2.5, member_2.id).await;
        let expense_5 = ctx.with_expense_of(&mut group, 2.5, member_2.id).await;
        let expense_6 = ctx.with_expense_of(&mut group, 50.0, member_3.id).await;
        let expense_7 = ctx.with_expense_of(&mut group, 15.0, member_3.id).await;
        let expense_8 = ctx.with_expense_of(&mut group, 35.0, member_3.id).await;
        let expense_9 = ctx.with_expense_of(&mut group, 70.0, admin).await;
        let expense_10 = ctx.with_expense_of(&mut group, 50.0, admin).await;
        let mut expenses = vec![
            expense_1.id,
            expense_2.id,
            expense_3.id,
            expense_4.id,
            expense_5.id,
            expense_6.id,
            expense_7.id,
            expense_8.id,
            expense_9.id,
            expense_10.id,
        ];
        expenses.sort();
        let req = SettleRequest {
            group_id: group.id,
            user_id: admin,
        };

        // when
        let resp = ctx.group().settle(req.clone()).await;

        // then
        let stl_dto = assert_ok!(resp);

        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.settlement_ids.len(), 1);
        assert_eq!(grp.settlement_ids[0], stl_dto.id);
        assert_eq!(grp.expense_ids.len(), 0);
        for exp in &expenses {
            assert!(ctx.get_expense(exp).await.settled);
        }
        let stl = assert_some!(ctx.find_settlement(&stl_dto.id).await);
        assert_eq!(stl.id, stl_dto.id);
        assert_eq!(stl.group_id, group.id);
        assert_eq!(stl.start_date, stl_dto.start_date);
        assert_eq!(stl.end_date, stl_dto.end_date);
        let mut exps = stl.expense_ids.clone();
        exps.sort();
        assert_eq!(exps, expenses);
        assert_eq!(stl.transactions.len(), 3);
        assert_eq!(stl.transactions.get(1).unwrap().from, member_1.id);
        assert_eq!(stl.transactions.get(1).unwrap().to, admin);
        assert_eq!(
            f32::from(stl.transactions.get(1).unwrap().amount.clone()),
            5.0
        );
        assert_eq!(stl.transactions.get(2).unwrap().from, member_1.id);
        assert_eq!(stl.transactions.get(2).unwrap().to, member_3.id);
        assert_eq!(
            f32::from(stl.transactions.get(2).unwrap().amount.clone()),
            37.5
        );
        assert_eq!(stl.transactions.get(0).unwrap().from, member_2.id);
        assert_eq!(stl.transactions.get(0).unwrap().to, admin);
        assert_eq!(
            f32::from(stl.transactions.get(0).unwrap().amount.clone()),
            52.5
        );

        assert_eq!(stl_dto.transactions.len(), 3);
        assert_eq!(stl_dto.transactions.get(0).unwrap().from.id, member_2.id);
        assert_eq!(stl_dto.transactions.get(0).unwrap().to.id, admin);
        assert_eq!(
            f32::from(stl_dto.transactions.get(0).unwrap().amount.clone()),
            52.5
        );
        assert_eq!(stl_dto.transactions.get(1).unwrap().from.id, member_1.id);
        assert_eq!(stl_dto.transactions.get(1).unwrap().to.id, member_3.id);
        assert_eq!(
            f32::from(stl_dto.transactions.get(1).unwrap().amount.clone()),
            37.5
        );
        assert_eq!(stl_dto.transactions.get(2).unwrap().from.id, member_1.id);
        assert_eq!(stl_dto.transactions.get(2).unwrap().to.id, admin);
        assert_eq!(
            f32::from(stl_dto.transactions.get(2).unwrap().amount.clone()),
            5.0
        );

        let event = assert_some!(ctx.last_stored_event());
        match event.event {
            InnerEventKind::Settled { .. } => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect event expected Settled, got: {:?}", e)
            ),
        }
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
    }

    #[tokio::test]
    async fn it_should_return_unauthorized_when_user_is_not_admin() {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member_1 = ctx.with_member(&mut group).await;
        let member_2 = ctx.with_member(&mut group).await;
        let member_3 = ctx.with_member(&mut group).await;
        let admin = group.admin_id;
        let expense_1 = ctx.with_expense_of(&mut group, 10.0, member_1.id).await;
        let expense_2 = ctx.with_expense_of(&mut group, 10.0, member_1.id).await;
        let expense_3 = ctx.with_expense_of(&mut group, 5.0, member_2.id).await;
        let expense_4 = ctx.with_expense_of(&mut group, 2.5, member_2.id).await;
        let expense_5 = ctx.with_expense_of(&mut group, 2.5, member_2.id).await;
        let expense_6 = ctx.with_expense_of(&mut group, 50.0, member_3.id).await;
        let expense_7 = ctx.with_expense_of(&mut group, 15.0, member_3.id).await;
        let expense_8 = ctx.with_expense_of(&mut group, 35.0, member_3.id).await;
        let expense_9 = ctx.with_expense_of(&mut group, 70.0, admin).await;
        let expense_10 = ctx.with_expense_of(&mut group, 50.0, admin).await;
        let mut expenses = vec![
            expense_1.id,
            expense_2.id,
            expense_3.id,
            expense_4.id,
            expense_5.id,
            expense_6.id,
            expense_7.id,
            expense_8.id,
            expense_9.id,
            expense_10.id,
        ];
        expenses.sort();
        let req = SettleRequest {
            group_id: group.id,
            user_id: member_1.id,
        };

        // when
        let resp = ctx.group().settle(req.clone()).await;

        // then
        let resp = assert_err!(resp);
        match resp {
            SettlementError::Unauthorized(_) => {}
            e => {
                unreachable!("{}", format!("Expected Unauthorized error, got {:?}", e))
            }
        }

        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.settlement_ids.len(), 0);
        assert_eq!(grp.expense_ids.len(), 10);
        for exp in &expenses {
            assert!(!ctx.get_expense(exp).await.settled);
        }
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::Settled { .. } => unreachable!("Last event should not be Settled",),
                _ => {}
            },
        }
    }

    #[tokio::test]
    async fn it_should_return_unauthenticated_when_user_is_unknown() {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member_1 = ctx.with_member(&mut group).await;
        let member_2 = ctx.with_member(&mut group).await;
        let member_3 = ctx.with_member(&mut group).await;
        let admin = group.admin_id;
        let expense_1 = ctx.with_expense_of(&mut group, 10.0, member_1.id).await;
        let expense_2 = ctx.with_expense_of(&mut group, 10.0, member_1.id).await;
        let expense_3 = ctx.with_expense_of(&mut group, 5.0, member_2.id).await;
        let expense_4 = ctx.with_expense_of(&mut group, 2.5, member_2.id).await;
        let expense_5 = ctx.with_expense_of(&mut group, 2.5, member_2.id).await;
        let expense_6 = ctx.with_expense_of(&mut group, 50.0, member_3.id).await;
        let expense_7 = ctx.with_expense_of(&mut group, 15.0, member_3.id).await;
        let expense_8 = ctx.with_expense_of(&mut group, 35.0, member_3.id).await;
        let expense_9 = ctx.with_expense_of(&mut group, 70.0, admin).await;
        let expense_10 = ctx.with_expense_of(&mut group, 50.0, admin).await;
        let mut expenses = vec![
            expense_1.id,
            expense_2.id,
            expense_3.id,
            expense_4.id,
            expense_5.id,
            expense_6.id,
            expense_7.id,
            expense_8.id,
            expense_9.id,
            expense_10.id,
        ];
        expenses.sort();
        let req = SettleRequest {
            group_id: group.id,
            user_id: Uuid::new_v4(),
        };

        // when
        let resp = ctx.group().settle(req.clone()).await;

        // then
        let resp = assert_err!(resp);
        match resp {
            SettlementError::Unauthenticated() => {}
            e => {
                unreachable!("{}", format!("Expected Unauthenticated error, got {:?}", e))
            }
        }

        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.settlement_ids.len(), 0);
        assert_eq!(grp.expense_ids.len(), 10);
        for exp in &expenses {
            assert!(!ctx.get_expense(exp).await.settled);
        }
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::Settled { .. } => unreachable!("Last event should not be Settled",),
                _ => {}
            },
        }
    }

    #[tokio::test]
    async fn it_should_return_unauthorized_when_user_is_not_member() {
        // given
        let ctx = TestContext::new();
        let user = ctx.with_user().await;
        let mut group = ctx.with_group().await;
        let member_1 = ctx.with_member(&mut group).await;
        let member_2 = ctx.with_member(&mut group).await;
        let member_3 = ctx.with_member(&mut group).await;
        let admin = group.admin_id;
        let expense_1 = ctx.with_expense_of(&mut group, 10.0, member_1.id).await;
        let expense_2 = ctx.with_expense_of(&mut group, 10.0, member_1.id).await;
        let expense_3 = ctx.with_expense_of(&mut group, 5.0, member_2.id).await;
        let expense_4 = ctx.with_expense_of(&mut group, 2.5, member_2.id).await;
        let expense_5 = ctx.with_expense_of(&mut group, 2.5, member_2.id).await;
        let expense_6 = ctx.with_expense_of(&mut group, 50.0, member_3.id).await;
        let expense_7 = ctx.with_expense_of(&mut group, 15.0, member_3.id).await;
        let expense_8 = ctx.with_expense_of(&mut group, 35.0, member_3.id).await;
        let expense_9 = ctx.with_expense_of(&mut group, 70.0, admin).await;
        let expense_10 = ctx.with_expense_of(&mut group, 50.0, admin).await;
        let mut expenses = vec![
            expense_1.id,
            expense_2.id,
            expense_3.id,
            expense_4.id,
            expense_5.id,
            expense_6.id,
            expense_7.id,
            expense_8.id,
            expense_9.id,
            expense_10.id,
        ];
        expenses.sort();
        let req = SettleRequest {
            group_id: group.id,
            user_id: user.id,
        };

        // when
        let resp = ctx.group().settle(req.clone()).await;

        // then
        let resp = assert_err!(resp);
        match resp {
            SettlementError::Unauthorized(_) => {}
            e => {
                unreachable!("{}", format!("Expected Unauthorized error, got {:?}", e))
            }
        }

        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.settlement_ids.len(), 0);
        assert_eq!(grp.expense_ids.len(), 10);
        for exp in &expenses {
            assert!(!ctx.get_expense(exp).await.settled);
        }
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::Settled { .. } => unreachable!("Last event should not be Settled",),
                _ => {}
            },
        }
    }
}
