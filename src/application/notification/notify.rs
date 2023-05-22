use crate::application::store::MultiRepository;
use crate::domain::errors::NotifyError;
use crate::domain::notification::NotificationService;
use crate::domain::{Event, Group, GroupEventKind, GroupMember};
use anyhow::Context;
use itertools::Itertools;
use log::warn;
use std::sync::Arc;
use uuid::Uuid;

struct Notification {
    pub title: String,
    pub text: String,
}

pub async fn notify(
    event_id: &Uuid,
    store: Arc<impl MultiRepository>,
    notification_svc: Arc<dyn NotificationService>,
) -> Result<(), NotifyError> {
    let event = store
        .events()
        .find(event_id)
        .await
        .context("Failed to fetch event.")
        .map_err(NotifyError::Unexpected)?
        .map_or_else(|| Err(NotifyError::NotFound("Event not found")), Ok)?;
    let event = match event {
        Event::User(_) => return Ok(()),
        Event::Group(e) => e,
    };
    let group = store
        .groups()
        .find(&event.group_id)
        .await
        .context("Failed to fetch group.")
        .map_err(NotifyError::Unexpected)?
        .map_or_else(|| Err(NotifyError::NotFound("Group not found")), Ok)?;
    let member = group
        .members
        .iter()
        .find(|m| m.id == event.member_id)
        .map_or_else(|| Err(NotifyError::NotFound("Member not found")), Ok)?;
    let recipients = group
        .members
        .iter()
        .filter(|m| m.id != member.id)
        .map(|m| m.id)
        .collect_vec();
    let notification = to_notification(&event.event, &group, member).await;
    if let Some(notification) = notification {
        for recipient in recipients {
            notification_svc
                .send(
                    &recipient,
                    notification.title.clone(),
                    notification.text.clone(),
                )
                .await
                .unwrap_or_else(|failure| {
                    warn!("{:?}", failure);
                });
        }
        store
            .events()
            .mark_processed(&event.id)
            .await
            .context("Failed to mark as processed")
            .map_err(NotifyError::Unexpected)?;
    }
    Ok(())
}

async fn to_notification(
    event: &GroupEventKind,
    group: &Group,
    member: &GroupMember,
) -> Option<Notification> {
    match event {
        GroupEventKind::GroupCreated { .. } => None,
        GroupEventKind::MemberJoined { .. } => {
            let notification_title = format!(
                "{} joined group {}",
                String::from(member.name.clone()),
                String::from(group.name.clone())
            );
            let notification = String::from(member.email.clone());
            Some(Notification {
                title: notification_title,
                text: notification,
            })
        }
        GroupEventKind::MemberColorChanged { .. } => None,
        GroupEventKind::ExpenseCreated {
            description,
            amount,
            ..
        } => {
            let notification_title = format!(
                "Expense from {} in {}",
                String::from(member.name.clone()),
                String::from(group.name.clone())
            );
            let notification = format!("{}: {}", description, amount);
            Some(Notification {
                title: notification_title,
                text: notification,
            })
        }
        GroupEventKind::ExpenseModified { .. } => None,
        GroupEventKind::ExpenseDeleted { .. } => None,
        GroupEventKind::GroupDeleted { .. } => None,
        GroupEventKind::Settled { transactions, .. } => {
            let notification_title =
                format!("Group {} was settled", String::from(group.name.clone()));
            let mut notification = transactions
                .iter()
                .map(|tr| {
                    let from = group
                        .members
                        .iter()
                        .find(|m| m.id == tr.from)
                        .map(|m| String::from(m.name.clone()))
                        .unwrap_or_else(|| String::from("Unknown"));
                    let to = group
                        .members
                        .iter()
                        .find(|m| m.id == tr.to)
                        .map(|m| String::from(m.name.clone()))
                        .unwrap_or_else(|| String::from("Unknown"));
                    format!("{} owes {:.2} to {}", from, f32::from(tr.amount), to)
                })
                .join("\n");
            if notification.is_empty() {
                notification = "You are all good !".to_string();
            }
            Some(Notification {
                title: notification_title,
                text: notification,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::tests::TestContext;
    use claim::{assert_err, assert_some};
    use uuid::Uuid;

    #[tokio::test]
    async fn it_should_send_a_notification_to_other_members_on_member_join(
    ) -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let admin = group.admin_id;
        let user_1 = ctx.with_member(&mut group).await;
        // when
        let user_2 = ctx.with_member(&mut group).await;
        let event = ctx.last_published_event().unwrap();
        notify(&event, ctx.store().clone(), ctx.notification_svc().clone()).await?;
        // then
        let notifications = ctx.notifications();
        assert_eq!(notifications.len(), 2);
        assert_some!(notifications.iter().find(|n| n.user == user_1.id));
        assert_some!(notifications.iter().find(|n| n.user == admin));
        let expected_title = format!(
            "{} joined group {}",
            String::from(user_2.name),
            String::from(group.name)
        );
        let expected_text = String::from(user_2.email);
        for notif in notifications {
            assert_eq!(notif.title, expected_title);
            assert_eq!(notif.text, expected_text);
        }
        assert_some!(ctx.get_event_process_date(&event).await);

        Ok(())
    }

    #[tokio::test]
    async fn it_should_send_a_notification_to_other_members_on_member_expense_created(
    ) -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let admin = group.admin_id;
        let user_1 = ctx.with_member(&mut group).await;
        let user_2 = ctx.with_member(&mut group).await;
        // when
        let expense = ctx.with_expense(&mut group, user_2.id).await;
        let event = ctx.last_published_event().unwrap();
        notify(&event, ctx.store().clone(), ctx.notification_svc().clone()).await?;
        // then
        let notifications = ctx.notifications();
        assert_eq!(notifications.len(), 2);
        assert_some!(notifications.iter().find(|n| n.user == user_1.id));
        assert_some!(notifications.iter().find(|n| n.user == admin));
        let expected_title = format!(
            "Expense from {} in {}",
            String::from(user_2.name),
            String::from(group.name)
        );
        let expected_text = format!(
            "{}: {}",
            String::from(expense.title),
            f32::from(expense.amount)
        );
        for notif in notifications {
            assert_eq!(notif.title, expected_title);
            assert_eq!(notif.text, expected_text);
        }
        assert_some!(ctx.get_event_process_date(&event).await);

        Ok(())
    }

    #[tokio::test]
    async fn it_should_send_a_notification_to_other_members_on_settlement(
    ) -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let user_1 = ctx.with_member(&mut group).await;
        let user_2 = ctx.with_member(&mut group).await;
        let mut expenses = Vec::new();
        expenses.push(ctx.with_expense_of(&mut group, 10.0, user_2.id).await);
        expenses.push(ctx.with_expense_of(&mut group, 50.0, user_1.id).await);

        // when
        let _ = ctx.settle(&mut group, &mut expenses).await;
        let event = ctx.last_published_event().unwrap();
        notify(&event, ctx.store().clone(), ctx.notification_svc().clone()).await?;
        // then
        let notifications = ctx.notifications();
        assert_eq!(notifications.len(), 2);
        assert_some!(notifications.iter().find(|n| n.user == user_1.id));
        assert_some!(notifications.iter().find(|n| n.user == user_2.id));
        let expected_title = format!("Group {} was settled", String::from(group.name.clone()));
        let expected_text = format!(
            "{} owes {:.2} to {}\n{} owes {:.2} to {}",
            String::from(group.admin().name.clone()),
            20.0,
            String::from(user_1.name.clone()),
            String::from(user_2.name.clone()),
            10.0,
            String::from(user_1.name.clone()),
        );
        for notif in notifications {
            assert_eq!(notif.title, expected_title);
            assert_eq!(notif.text, expected_text);
        }
        assert_some!(ctx.get_event_process_date(&event).await);

        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_an_error_if_event_is_not_found() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let _user_1 = ctx.with_member(&mut group).await;
        // when
        let resp = notify(
            &Uuid::new_v4(),
            ctx.store().clone(),
            ctx.notification_svc().clone(),
        )
        .await;
        // then
        assert_err!(resp);
        assert!(ctx.notifications().is_empty());

        Ok(())
    }
}
