use crate::application::store::MultiRepository;
use crate::domain::errors::CreateGroupError;
use crate::domain::usecases::group::CreateGroupRequest;
use crate::domain::{Group, MemberColor};
use anyhow::{anyhow, Context};
use std::sync::Arc;

pub async fn create<Store: MultiRepository>(
    create_group_data: CreateGroupRequest,
    store: Arc<Store>,
) -> Result<Group, CreateGroupError> {
    let user = store
        .users()
        .find(&create_group_data.admin_id)
        .await
        .context("Failed to fetch admin user")
        .map_err(CreateGroupError::Unexpected)?;
    match user {
        Some(user) => {
            let color = MemberColor::from(create_group_data.admin_color);
            let group = Group::create(
                create_group_data.name,
                user.id,
                user.name,
                user.email,
                color,
            )?;
            Ok(group)
        }
        None => Err(CreateGroupError::Unexpected(anyhow!(
            "Admin user not found"
        ))),
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
    async fn it_should_return_the_new_group_id_and_save_it_to_the_repo_given_a_valid_request() {
        // given
        let ctx = TestContext::new();
        let user = ctx.with_user().await;
        let req = CreateGroupRequest {
            name: "My group".to_string(),
            admin_id: user.id,
            admin_color: ColorDto {
                red: 255,
                green: 10,
                blue: 10,
            },
        };
        // when
        let resp = ctx.group().create_group(req.clone()).await;
        // then
        let resp = assert_ok!(resp);
        let grp = ctx.get_group(&resp).await;
        assert_eq!(String::from(grp.name.clone()), req.name);
        assert_eq!(grp.admin_id, req.admin_id);
        assert_eq!(grp.members.len(), 1);
        assert_eq!(grp.members[0].id, req.admin_id);
        assert_eq!(grp.members[0].color.red, req.admin_color.red);
        assert_eq!(grp.members[0].color.green, req.admin_color.green);
        assert_eq!(grp.members[0].color.blue, req.admin_color.blue);
        let event = assert_some!(ctx.last_stored_event());
        match event.event {
            InnerEventKind::GroupCreated { .. } => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect event expected GroupCreated, got: {:?}", e)
            ),
        }
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
    }

    #[tokio::test]
    async fn it_should_return_validation_error_given_an_invalid_request() {
        // given
        let ctx = TestContext::new();
        let user = ctx.with_user().await;
        let req = CreateGroupRequest {
            name: "".to_string(),
            admin_id: user.id,
            admin_color: ColorDto {
                red: 255,
                green: 10,
                blue: 10,
            },
        };
        // when
        let resp = ctx.group().create_group(req.clone()).await;
        // then
        let resp = assert_err!(resp);
        match resp {
            CreateGroupError::Validation(_) => {}
            e => {
                unreachable!("Expected validation error, got {:?}", e)
            }
        }
        match ctx.last_stored_event() {
            None => {}
            Some(e) => match e.event {
                InnerEventKind::GroupCreated { .. } => {
                    unreachable!("Last event should not be GroupCreated",)
                }
                _ => {}
            },
        }
    }
}
