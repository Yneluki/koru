use crate::application::store::MultiRepository;
use crate::domain::errors::CreateExpenseError;
use crate::domain::usecases::group::CreateExpenseRequest;
use crate::domain::{Expense, Group};
use anyhow::Context;
use std::sync::Arc;

pub async fn create(
    expense_data: CreateExpenseRequest,
    store: Arc<impl MultiRepository>,
) -> Result<(Group, Expense), CreateExpenseError> {
    let group = store
        .groups()
        .find(&expense_data.group_id)
        .await
        .context("Failed to fetch group.")
        .map_err(CreateExpenseError::Unexpected)?;
    match group {
        Some(mut group) => {
            let expense = group.add_expense(
                expense_data.title,
                expense_data.amount,
                expense_data.member_id,
            )?;
            Ok((group, expense))
        }
        None => Err(CreateExpenseError::GroupNotFound()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::tests::TestContext;
    use crate::domain::usecases::group::GroupUseCase;
    use crate::infrastructure::store::mem::mem_store::InnerEventKind;
    use claim::{assert_err, assert_ok, assert_some};
    use float_cmp::assert_approx_eq;
    use uuid::Uuid;

    #[tokio::test]
    async fn it_should_return_the_expense_id_and_add_it_when_user_is_admin(
    ) -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;

        let req = CreateExpenseRequest {
            group_id: group.id,
            member_id: group.admin_id,
            title: "My expense".to_string(),
            amount: 12.0,
        };

        // when
        let resp = ctx.group().create_expense(req.clone()).await;

        // then
        let expense_id = assert_ok!(resp);
        let group = ctx.get_group(&group.id).await;
        assert_eq!(group.expense_ids.len(), 1);
        assert_eq!(group.expense_ids[0], expense_id);
        let expense = ctx.get_expense(&expense_id).await;
        assert_eq!(expense.member_id, group.admin_id);
        assert_approx_eq!(f32, f32::from(expense.amount), req.amount);
        assert_eq!(String::from(expense.title), req.title);
        assert_eq!(expense.settled, false);
        let event = assert_some!(ctx.last_stored_event());
        match event.event {
            InnerEventKind::ExpenseCreated { .. } => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect event expected ExpenseCreated, got: {:?}", e)
            ),
        }
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_the_expense_id_and_add_it_when_user_is_member(
    ) -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;

        let req = CreateExpenseRequest {
            group_id: group.id,
            member_id: member.id,
            title: "My expense".to_string(),
            amount: 12.0,
        };

        // when
        let resp = ctx.group().create_expense(req.clone()).await;

        // then
        let expense_id = assert_ok!(resp);
        let group = ctx.get_group(&group.id).await;
        assert_eq!(group.expense_ids.len(), 1);
        assert_eq!(group.expense_ids[0], expense_id);
        let expense = ctx.get_expense(&expense_id).await;
        assert_eq!(expense.member_id, member.id);
        assert_approx_eq!(f32, f32::from(expense.amount), req.amount);
        assert_eq!(String::from(expense.title), req.title);
        assert_eq!(expense.settled, false);
        let event = assert_some!(ctx.last_stored_event());
        match event.event {
            InnerEventKind::ExpenseCreated { .. } => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect event expected ExpenseCreated, got: {:?}", e)
            ),
        }
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_unauthorized_when_user_is_not_member() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;
        let user = ctx.with_user().await;

        let req = CreateExpenseRequest {
            group_id: group.id,
            member_id: user.id,
            title: "My expense".to_string(),
            amount: 12.0,
        };

        // when
        let resp = ctx.group().create_expense(req.clone()).await;

        // then
        let expense_id = assert_err!(resp);
        match expense_id {
            CreateExpenseError::Unauthorized() => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected Unauthorized, got: {:?}", e)
            ),
        }
        let group = ctx.get_group(&group.id).await;
        assert_eq!(group.expense_ids.len(), 0);
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::ExpenseCreated { .. } => {
                    unreachable!("Last event should not be ExpenseCreated",)
                }
                _ => {}
            },
        }
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_unauthenticated_when_user_is_unknown() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;

        let req = CreateExpenseRequest {
            group_id: group.id,
            member_id: Uuid::new_v4(),
            title: "My expense".to_string(),
            amount: 12.0,
        };

        // when
        let resp = ctx.group().create_expense(req.clone()).await;

        // then
        let expense_id = assert_err!(resp);
        match expense_id {
            CreateExpenseError::Unauthenticated() => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected Unauthenticated, got: {:?}", e)
            ),
        }
        let group = ctx.get_group(&group.id).await;
        assert_eq!(group.expense_ids.len(), 0);
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::ExpenseCreated { .. } => {
                    unreachable!("Last event should not be ExpenseCreated",)
                }
                _ => {}
            },
        }
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_not_found_when_group_does_not_exist() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;

        let req = CreateExpenseRequest {
            group_id: Uuid::new_v4(),
            member_id: group.admin_id,
            title: "My expense".to_string(),
            amount: 12.0,
        };

        // when
        let resp = ctx.group().create_expense(req.clone()).await;

        // then
        let expense_id = assert_err!(resp);
        match expense_id {
            CreateExpenseError::GroupNotFound() => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected Not Found, got: {:?}", e)
            ),
        }
        let group = ctx.get_group(&group.id).await;
        assert_eq!(group.expense_ids.len(), 0);
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::ExpenseCreated { .. } => {
                    unreachable!("Last event should not be ExpenseCreated",)
                }
                _ => {}
            },
        }
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_validation_error_when_expense_is_invalid() -> Result<(), anyhow::Error>
    {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;

        let cases = vec![
            ("", 12.95, "empty description"),
            ("my expense", 0.0, "0 amount"),
            ("my expense", -10.0, "negative amount"),
        ];

        for (title, amount, desc) in cases {
            let req = CreateExpenseRequest {
                group_id: group.id,
                member_id: group.admin_id,
                title: title.to_string(),
                amount,
            };

            // when
            let resp = ctx.group().create_expense(req.clone()).await;

            // then
            let expense_id = assert_err!(resp, "It did not return an error for case {}.", desc);
            match expense_id {
                CreateExpenseError::Validation(_) => {}
                e => unreachable!(
                    "{}",
                    format!(
                        "Got incorrect error for case {}, expected Validation, got: {:?}",
                        desc, e
                    )
                ),
            }
            let group = ctx.get_group(&group.id).await;
            assert_eq!(
                group.expense_ids.len(),
                0,
                "Expected 0 expenses for case {}.",
                desc
            );
            match ctx.last_stored_event() {
                None => {}
                Some(e) => match e.event {
                    InnerEventKind::ExpenseCreated { .. } => {
                        unreachable!("Last event should not be ExpenseCreated",)
                    }
                    _ => {}
                },
            }
        }
        Ok(())
    }
}
