use crate::application::store::MultiRepository;
use crate::domain::errors::UpdateExpenseError;
use crate::domain::usecases::group::UpdateExpenseRequest;
use crate::domain::{Expense, Group};
use anyhow::Context;
use std::sync::Arc;

pub async fn update(
    data: UpdateExpenseRequest,
    store: Arc<impl MultiRepository>,
) -> Result<(Group, Expense), UpdateExpenseError> {
    let group = store
        .groups()
        .find(&data.group_id)
        .await
        .context("Failed to fetch group.")
        .map_err(UpdateExpenseError::Unexpected)?;

    match group {
        Some(mut group) => {
            let expenses = store
                .expenses()
                .get_expenses_by_id(&group.expense_ids)
                .await
                .context("Failed to fetch expenses.")
                .map_err(UpdateExpenseError::Unexpected)?;
            let expense = group.update_expense(
                data.expense_id,
                data.description,
                data.amount,
                data.user_id,
                expenses,
            )?;
            Ok((group, expense))
        }
        None => Err(UpdateExpenseError::NotFound("Group not found.")),
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
    async fn it_should_update_the_expense_when_user_is_admin() {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;
        let expense = ctx.with_expense(&mut group, member.id).await;
        let req = UpdateExpenseRequest {
            group_id: group.id,
            expense_id: expense.id,
            user_id: group.admin_id,
            description: "New name".to_string(),
            amount: 30.0,
        };
        // when
        let resp = ctx.group().update_expense(req.clone()).await;
        // then
        assert_ok!(resp);
        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.expense_ids.len(), 1);
        assert_eq!(grp.expense_ids[0], expense.id);
        let exp = ctx.get_expense(&expense.id).await;
        assert_eq!(String::from(exp.title.clone()), req.description);
        assert_eq!(f32::from(exp.amount.clone()), req.amount);
        let event = assert_some!(ctx.last_stored_event());
        match event.event {
            InnerEventKind::ExpenseModified { .. } => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect event expected ExpenseModified, got: {:?}", e)
            ),
        }
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
    }

    #[tokio::test]
    async fn it_should_update_the_expense_when_user_is_owner() {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;
        let expense = ctx.with_expense(&mut group, member.id).await;
        let req = UpdateExpenseRequest {
            group_id: group.id,
            expense_id: expense.id,
            user_id: member.id,
            description: "New name".to_string(),
            amount: 30.0,
        };
        // when
        let resp = ctx.group().update_expense(req.clone()).await;
        // then
        assert_ok!(resp);
        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.expense_ids.len(), 1);
        assert_eq!(grp.expense_ids[0], expense.id);
        let exp = ctx.get_expense(&expense.id).await;
        assert_eq!(String::from(exp.title.clone()), req.description);
        assert_eq!(f32::from(exp.amount.clone()), req.amount);
        let event = assert_some!(ctx.last_stored_event());
        match event.event {
            InnerEventKind::ExpenseModified { .. } => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect event expected ExpenseModified, got: {:?}", e)
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
        let req = UpdateExpenseRequest {
            group_id: group.id,
            expense_id: expense.id,
            user_id: user.id,
            description: "New name".to_string(),
            amount: 30.0,
        };
        // when
        let resp = ctx.group().update_expense(req.clone()).await;
        // then
        let resp = assert_err!(resp);
        match resp {
            UpdateExpenseError::Unauthorized(_) => {}
            e => {
                unreachable!("{}", format!("Expected Unauthorized error, got {:?}", e))
            }
        }
        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.expense_ids.len(), 1);
        assert_eq!(grp.expense_ids[0], expense.id);
        let exp = ctx.get_expense(&expense.id).await;
        assert_eq!(
            String::from(exp.title.clone()),
            String::from(expense.title.clone())
        );
        assert_eq!(
            f32::from(exp.amount.clone()),
            f32::from(expense.amount.clone())
        );
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::ExpenseModified { .. } => {
                    unreachable!("Last event should not be ExpenseModified",)
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
        let req = UpdateExpenseRequest {
            group_id: group.id,
            expense_id: expense.id,
            user_id: Uuid::new_v4(),
            description: "New name".to_string(),
            amount: 30.0,
        };
        // when
        let resp = ctx.group().update_expense(req.clone()).await;
        // then
        let resp = assert_err!(resp);
        match resp {
            UpdateExpenseError::Unauthenticated() => {}
            e => {
                unreachable!("{}", format!("Expected Unauthenticated error, got {:?}", e))
            }
        }
        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.expense_ids.len(), 1);
        assert_eq!(grp.expense_ids[0], expense.id);
        let exp = ctx.get_expense(&expense.id).await;
        assert_eq!(
            String::from(exp.title.clone()),
            String::from(expense.title.clone())
        );
        assert_eq!(
            f32::from(exp.amount.clone()),
            f32::from(expense.amount.clone())
        );
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::ExpenseModified { .. } => {
                    unreachable!("Last event should not be ExpenseModified",)
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
        let req = UpdateExpenseRequest {
            group_id: Uuid::new_v4(),
            expense_id: expense.id,
            user_id: member.id,
            description: "New name".to_string(),
            amount: 30.0,
        };
        // when
        let resp = ctx.group().update_expense(req.clone()).await;
        // then
        let resp = assert_err!(resp);
        match resp {
            UpdateExpenseError::NotFound(_) => {}
            e => {
                unreachable!("{}", format!("Expected NotFound error, got {:?}", e))
            }
        }
        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.expense_ids.len(), 1);
        assert_eq!(grp.expense_ids[0], expense.id);
        let exp = ctx.get_expense(&expense.id).await;
        assert_eq!(
            String::from(exp.title.clone()),
            String::from(expense.title.clone())
        );
        assert_eq!(
            f32::from(exp.amount.clone()),
            f32::from(expense.amount.clone())
        );
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::ExpenseModified { .. } => {
                    unreachable!("Last event should not be ExpenseModified",)
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
        let req = UpdateExpenseRequest {
            group_id: group.id,
            expense_id: Uuid::new_v4(),
            user_id: member.id,
            description: "New name".to_string(),
            amount: 30.0,
        };
        // when
        let resp = ctx.group().update_expense(req.clone()).await;
        // then
        let resp = assert_err!(resp);
        match resp {
            UpdateExpenseError::NotFound(_) => {}
            e => {
                unreachable!("{}", format!("Expected NotFound error, got {:?}", e))
            }
        }
        let grp = ctx.get_group(&group.id).await;
        assert_eq!(grp.expense_ids.len(), 1);
        assert_eq!(grp.expense_ids[0], expense.id);
        let exp = ctx.get_expense(&expense.id).await;
        assert_eq!(
            String::from(exp.title.clone()),
            String::from(expense.title.clone())
        );
        assert_eq!(
            f32::from(exp.amount.clone()),
            f32::from(expense.amount.clone())
        );
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::ExpenseModified { .. } => {
                    unreachable!("Last event should not be ExpenseModified",)
                }
                _ => {}
            },
        }
    }

    #[tokio::test]
    async fn it_should_return_validation_error_if_expense_data_is_invalid() {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;
        let expense = ctx.with_expense(&mut group, member.id).await;
        let cases = vec![
            ("", 12.95, "empty description"),
            ("new expense", 0.0, "0 amount"),
            ("new expense", -10.0, "negative amount"),
        ];

        for (title, amount, desc) in cases {
            let req = UpdateExpenseRequest {
                group_id: group.id,
                expense_id: expense.id,
                user_id: member.id,
                description: title.to_string(),
                amount,
            };
            // when
            let resp = ctx.group().update_expense(req.clone()).await;
            // then
            let resp = assert_err!(resp, "It did not return an error for case {}.", desc);
            match resp {
                UpdateExpenseError::Validation(_) => {}
                e => {
                    unreachable!(
                        "{}",
                        format!(
                            "Got incorrect error for case {}, expected Validation, got: {:?}",
                            desc, e
                        )
                    )
                }
            }
            let grp = ctx.get_group(&group.id).await;
            assert_eq!(grp.expense_ids.len(), 1);
            assert_eq!(grp.expense_ids[0], expense.id);
            let exp = ctx.get_expense(&expense.id).await;
            assert_eq!(
                String::from(exp.title.clone()),
                String::from(expense.title.clone())
            );
            assert_eq!(
                f32::from(exp.amount.clone()),
                f32::from(expense.amount.clone())
            );
            match ctx.last_stored_event() {
                None => {}
                Some(e) => match e.event {
                    InnerEventKind::ExpenseModified { .. } => {
                        unreachable!("Last event should not be ExpenseModified",)
                    }
                    _ => {}
                },
            }
        }
    }
}
