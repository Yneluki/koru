use crate::application::store::MultiRepository;
use crate::domain::errors::ChangeMemberColorError;
use crate::domain::usecases::group::ChangeMemberColorRequest;
use crate::domain::{Group, MemberColor};
use anyhow::Context;
use std::sync::Arc;

pub async fn change_color(
    data: ChangeMemberColorRequest,
    store: Arc<impl MultiRepository>,
) -> Result<Group, ChangeMemberColorError> {
    // fetch group
    let group = store
        .groups()
        .find(&data.group_id)
        .await
        .context("Failed to fetch group.")
        .map_err(ChangeMemberColorError::Unexpected)?;
    match group {
        Some(mut group) => {
            let color = MemberColor::from(data.color);
            let _member = group.update_member(data.user_id, color)?;
            Ok(group)
        }
        None => Err(ChangeMemberColorError::NotFound("Group not found.")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::tests::TestContext;
    use crate::domain::usecases::dto::dtos::ColorDto;
    use crate::domain::usecases::group::GroupUseCase;
    use crate::infrastructure::store::mem::mem_store::InnerEventKind;
    use claim::{assert_err, assert_ok, assert_some};

    #[tokio::test]
    async fn it_should_return_ok_and_update_the_user_color_when_user_is_member(
    ) -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;

        let req = ChangeMemberColorRequest {
            group_id: group.id,
            user_id: member.id,
            color: ColorDto {
                red: 255,
                green: 255,
                blue: 255,
            },
        };

        // when
        let resp = ctx.group().change_member_color(req.clone()).await;

        // then
        let _ = assert_ok!(resp);
        let group = ctx.get_group(&group.id).await;
        assert_some!(group.members.iter().find(|m| m.id == req.user_id));
        let user = group.members.iter().find(|m| m.id == req.user_id).unwrap();
        assert_eq!(user.color.red, req.color.red);
        assert_eq!(user.color.green, req.color.green);
        assert_eq!(user.color.blue, req.color.blue);
        let event = assert_some!(ctx.last_stored_event());
        match event.event {
            InnerEventKind::MemberColorChanged { .. } => {}
            e => unreachable!(
                "{}",
                format!(
                    "Got incorrect event expected MemberColorChanged, got: {:?}",
                    e
                )
            ),
        }
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_ok_and_update_the_user_color_when_user_is_admin(
    ) -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;

        let req = ChangeMemberColorRequest {
            group_id: group.id,
            user_id: group.admin_id,
            color: ColorDto {
                red: 255,
                green: 255,
                blue: 255,
            },
        };

        // when
        let resp = ctx.group().change_member_color(req.clone()).await;

        // then
        let _ = assert_ok!(resp);
        let group = ctx.get_group(&group.id).await;
        assert_some!(group.members.iter().find(|m| m.id == req.user_id));
        let user = group.members.iter().find(|m| m.id == req.user_id).unwrap();
        assert_eq!(user.color.red, req.color.red);
        assert_eq!(user.color.green, req.color.green);
        assert_eq!(user.color.blue, req.color.blue);
        let event = assert_some!(ctx.last_stored_event());
        match event.event {
            InnerEventKind::MemberColorChanged { .. } => {}
            e => unreachable!(
                "{}",
                format!(
                    "Got incorrect event expected MemberColorChanged, got: {:?}",
                    e
                )
            ),
        }
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_unauthorized_if_user_is_not_member() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;
        let user = ctx.with_user().await;

        let req = ChangeMemberColorRequest {
            group_id: group.id,
            user_id: user.id,
            color: ColorDto {
                red: 255,
                green: 255,
                blue: 255,
            },
        };

        // when
        let resp = ctx.group().change_member_color(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            ChangeMemberColorError::Unauthorized(_) => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected Unauthorized, got: {:?}", e)
            ),
        }
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::MemberColorChanged { .. } => {
                    unreachable!("Last event should not be MemberColorChanged",)
                }
                _ => {}
            },
        }
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_not_found_for_unknown_group() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;
        ctx.remove_group(&group.id).await;

        let req = ChangeMemberColorRequest {
            group_id: group.id,
            user_id: member.id,
            color: ColorDto {
                red: 255,
                green: 255,
                blue: 255,
            },
        };

        // when
        let resp = ctx.group().change_member_color(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            ChangeMemberColorError::NotFound(_) => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected NotFound, got: {:?}", e)
            ),
        }
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::MemberColorChanged { .. } => {
                    unreachable!("Last event should not be MemberColorChanged",)
                }
                _ => {}
            },
        }
        Ok(())
    }
}
