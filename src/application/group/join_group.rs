use crate::application::store::MultiRepository;
use crate::domain::errors::JoinGroupError;
use crate::domain::usecases::group::JoinGroupRequest;
use crate::domain::{Group, MemberColor, TokenGenerator};
use anyhow::{anyhow, Context};
use std::sync::Arc;

pub async fn join(
    data: JoinGroupRequest,
    store: Arc<impl MultiRepository>,
    token_svc: Arc<dyn TokenGenerator>,
) -> Result<Group, JoinGroupError> {
    // validate token
    token_svc.verify(data.token, &data.group_id).await?;
    // fetch group & add member
    let group = store
        .groups()
        .find(&data.group_id)
        .await
        .context("Failed to fetch group.")
        .map_err(JoinGroupError::Unexpected)?;
    match group {
        Some(mut group) => {
            let user = store
                .users()
                .find(&data.user_id)
                .await
                .context("Failed to fetch admin user")
                .map_err(JoinGroupError::Unexpected)?;
            match user {
                None => Err(JoinGroupError::Unexpected(anyhow!("User not found."))),
                Some(user) => {
                    let color = MemberColor::from(data.color);
                    let _member = group.add_member(data.user_id, user.name, user.email, color)?;
                    Ok(group)
                }
            }
        }
        None => Err(JoinGroupError::NotFound("Group not found.")),
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
    async fn it_should_return_ok_and_add_the_user_to_the_group() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;
        let user = ctx.with_user().await;
        let token = ctx.group_token(&group).await;

        let req = JoinGroupRequest {
            group_id: group.id,
            user_id: user.id,
            color: ColorDto {
                red: 255,
                green: 10,
                blue: 10,
            },
            token,
        };

        // when
        let resp = ctx.group().join_group(req.clone()).await;

        // then
        let _ = assert_ok!(resp);
        let group = ctx.get_group(&group.id).await;
        assert_eq!(group.members.len(), 2);
        assert_some!(group.members.iter().find(|m| m.id == group.admin_id));
        assert_some!(group.members.iter().find(|m| m.id == req.user_id));
        let event = assert_some!(ctx.last_stored_event());
        match event.event {
            InnerEventKind::MemberJoined { .. } => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect event expected MemberJoined, got: {:?}", e)
            ),
        }
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_unauthorized_with_invalid_token() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;
        let user = ctx.with_user().await;

        let req = JoinGroupRequest {
            group_id: group.id,
            user_id: user.id,
            token: "my token".to_string(),
            color: ColorDto {
                red: 255,
                green: 10,
                blue: 10,
            },
        };

        // when
        let resp = ctx.group().join_group(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            JoinGroupError::Unauthorized(_) => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected Unauthorized, got: {:?}", e)
            ),
        }
        let group = ctx.get_group(&group.id).await;
        assert_eq!(group.members.len(), 1);
        assert_eq!(group.members[0].id, group.admin_id);
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::MemberJoined { .. } => {
                    unreachable!("Last event should not be MemberJoined",)
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
        let group = ctx.with_group().await;
        let user = ctx.with_user().await;
        let token = ctx.group_token(&group).await;
        ctx.remove_group(&group.id).await;

        let req = JoinGroupRequest {
            group_id: group.id,
            user_id: user.id,
            token,
            color: ColorDto {
                red: 255,
                green: 10,
                blue: 10,
            },
        };

        // when
        let resp = ctx.group().join_group(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            JoinGroupError::NotFound(_) => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected NotFound, got: {:?}", e)
            ),
        }
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::MemberJoined { .. } => {
                    unreachable!("Last event should not be MemberJoined",)
                }
                _ => {}
            },
        }
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_unauthorized_for_token_not_matching_group(
    ) -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;
        let second_group = ctx.with_group().await;
        let user = ctx.with_user().await;
        let token = ctx.group_token(&second_group).await;

        let req = JoinGroupRequest {
            group_id: group.id,
            user_id: user.id,
            color: ColorDto {
                red: 255,
                green: 10,
                blue: 10,
            },
            token,
        };

        // when
        let resp = ctx.group().join_group(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            JoinGroupError::Unauthorized(_) => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected Unauthorized, got: {:?}", e)
            ),
        }
        let group = ctx.get_group(&group.id).await;
        assert_eq!(group.members.len(), 1);
        assert_eq!(group.members[0].id, group.admin_id);
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::MemberJoined { .. } => {
                    unreachable!("Last event should not be MemberJoined",)
                }
                _ => {}
            },
        }
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_conflict_when_user_is_admin() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;
        let token = ctx.group_token(&group).await;

        let req = JoinGroupRequest {
            group_id: group.id,
            user_id: group.admin_id,
            color: ColorDto {
                red: 255,
                green: 10,
                blue: 10,
            },
            token,
        };

        // when
        let resp = ctx.group().join_group(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            JoinGroupError::Conflict() => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected Conflict, got: {:?}", e)
            ),
        }
        let group = ctx.get_group(&group.id).await;
        assert_eq!(group.members.len(), 1);
        assert_eq!(group.members[0].id, group.admin_id);
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::MemberJoined { .. } => {
                    unreachable!("Last event should not be MemberJoined",)
                }
                _ => {}
            },
        }
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_conflict_when_user_is_already_member() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;
        let token = ctx.group_token(&group).await;
        let user = ctx.with_user().await;
        ctx.group()
            .join_group(JoinGroupRequest {
                group_id: group.id,
                user_id: user.id,
                color: ColorDto {
                    red: 255,
                    green: 10,
                    blue: 10,
                },
                token: token.clone(),
            })
            .await?;

        let req = JoinGroupRequest {
            group_id: group.id,
            user_id: user.id,
            color: ColorDto {
                red: 255,
                green: 10,
                blue: 10,
            },
            token,
        };

        // when
        let resp = ctx.group().join_group(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            JoinGroupError::Conflict() => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected Conflict, got: {:?}", e)
            ),
        }
        let group = ctx.get_group(&group.id).await;
        assert_eq!(group.members.len(), 2);
        assert_some!(group.members.iter().find(|m| m.id == group.admin_id));
        assert_some!(group.members.iter().find(|m| m.id == req.user_id));
        Ok(())
    }
}
