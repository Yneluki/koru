use crate::application::store::MultiRepository;
use crate::domain::errors::DeleteExpenseError;
use crate::domain::usecases::group::DeleteExpenseRequest;
use crate::domain::{Expense, Group};
use anyhow::Context;
use std::sync::Arc;

pub async fn delete(
    data: DeleteExpenseRequest,
    store: Arc<impl MultiRepository>,
) -> Result<(Group, Expense), DeleteExpenseError> {
    let group = store
        .groups()
        .find(&data.group_id)
        .await
        .context("Failed to fetch group.")
        .map_err(DeleteExpenseError::Unexpected)?;

    match group {
        Some(mut group) => {
            let expenses = store
                .expenses()
                .get_expenses_by_id(&group.expense_ids)
                .await
                .context("Failed to fetch expenses.")
                .map_err(DeleteExpenseError::Unexpected)?;
            let expense = group.delete_expense(data.expense_id, data.user_id, expenses)?;
            Ok((group, expense))
        }
        None => Err(DeleteExpenseError::NotFound("Group not found.")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::tests::TestContext;
    use crate::domain::usecases::group::GroupUseCase;
    use crate::infrastructure::store::mem::mem_store::InnerEventKind;
    use claim::{assert_err, assert_none, assert_ok, assert_some};
    use uuid::Uuid;

    #[tokio::test]
    async fn it_should_delete_the_expense_when_user_is_admin() {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;
        let expense = ctx.with_expense(&mut group, member.id).await;
        let req = DeleteExpenseRequest {
            group_id: group.id,
            expense_id: expense.id,
            user_id: group.admin_id,
        };
        // when
        let resp = ctx.group().delete_expense(req.clone()).await;
        // then
        assert_ok!(resp);
        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.expense_ids.len(), 0);
        assert_none!(ctx.find_expense(&expense.id).await);
        let event = assert_some!(ctx.last_stored_event());
        match event.event {
            InnerEventKind::ExpenseDeleted { .. } => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect event expected ExpenseDeleted, got: {:?}", e)
            ),
        }
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
    }

    #[tokio::test]
    async fn it_should_delete_the_expense_when_user_is_owner() {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;
        let expense = ctx.with_expense(&mut group, member.id).await;
        let req = DeleteExpenseRequest {
            group_id: group.id,
            expense_id: expense.id,
            user_id: member.id,
        };
        // when
        let resp = ctx.group().delete_expense(req.clone()).await;
        // then
        assert_ok!(resp);
        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.expense_ids.len(), 0);
        assert_none!(ctx.find_expense(&expense.id).await);
        let event = assert_some!(ctx.last_stored_event());
        match event.event {
            InnerEventKind::ExpenseDeleted { .. } => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect event expected ExpenseDeleted, got: {:?}", e)
            ),
        }
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
    }

    #[tokio::test]
    async fn it_should_return_unauthorized_if_user_is_not_member() {
        // given
        let ctx = TestContext::new();
        let user = ctx.with_user().await;
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;
        let expense = ctx.with_expense(&mut group, member.id).await;
        let req = DeleteExpenseRequest {
            group_id: group.id,
            expense_id: expense.id,
            user_id: user.id,
        };
        // when
        let resp = ctx.group().delete_expense(req.clone()).await;
        // then
        let resp = assert_err!(resp);
        match resp {
            DeleteExpenseError::Unauthorized(_) => {}
            e => {
                unreachable!("{}", format!("Expected Unauthorized error, got {:?}", e))
            }
        }
        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.expense_ids.len(), 1);
        assert_some!(ctx.find_expense(&expense.id).await);
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::ExpenseDeleted { .. } => {
                    unreachable!("Last event should not be ExpenseDeleted",)
                }
                _ => {}
            },
        }
    }

    #[tokio::test]
    async fn it_should_return_unauthenticated_if_user_is_unknown() {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;
        let expense = ctx.with_expense(&mut group, member.id).await;
        let req = DeleteExpenseRequest {
            group_id: group.id,
            expense_id: expense.id,
            user_id: Uuid::new_v4(),
        };
        // when
        let resp = ctx.group().delete_expense(req.clone()).await;
        // then
        let resp = assert_err!(resp);
        match resp {
            DeleteExpenseError::Unauthenticated() => {}
            e => {
                unreachable!("{}", format!("Expected Unauthenticated error, got {:?}", e))
            }
        }
        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.expense_ids.len(), 1);
        assert_some!(ctx.find_expense(&expense.id).await);
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::ExpenseDeleted { .. } => {
                    unreachable!("Last event should not be ExpenseDeleted",)
                }
                _ => {}
            },
        }
    }

    #[tokio::test]
    async fn it_should_return_not_found_if_group_does_not_exists() {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;
        let expense = ctx.with_expense(&mut group, member.id).await;
        let req = DeleteExpenseRequest {
            group_id: Uuid::new_v4(),
            expense_id: expense.id,
            user_id: member.id,
        };
        // when
        let resp = ctx.group().delete_expense(req.clone()).await;
        // then
        let resp = assert_err!(resp);
        match resp {
            DeleteExpenseError::NotFound(_) => {}
            e => {
                unreachable!("{}", format!("Expected NotFound error, got {:?}", e))
            }
        }
        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.expense_ids.len(), 1);
        assert_some!(ctx.find_expense(&expense.id).await);
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::ExpenseDeleted { .. } => {
                    unreachable!("Last event should not be ExpenseDeleted",)
                }
                _ => {}
            },
        }
    }

    #[tokio::test]
    async fn it_should_return_not_found_if_expense_does_not_exists() {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;
        let expense = ctx.with_expense(&mut group, member.id).await;
        let req = DeleteExpenseRequest {
            group_id: group.id,
            expense_id: Uuid::new_v4(),
            user_id: member.id,
        };
        // when
        let resp = ctx.group().delete_expense(req.clone()).await;
        // then
        let resp = assert_err!(resp);
        match resp {
            DeleteExpenseError::NotFound(_) => {}
            e => {
                unreachable!("{}", format!("Expected NotFound error, got {:?}", e))
            }
        }
        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.expense_ids.len(), 1);
        assert_some!(ctx.find_expense(&expense.id).await);
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::ExpenseDeleted { .. } => {
                    unreachable!("Last event should not be ExpenseDeleted",)
                }
                _ => {}
            },
        }
    }
}
