use crate::application::store::MultiRepository;
use crate::domain::errors::DeleteGroupError;
use crate::domain::usecases::group::DeleteGroupRequest;
use crate::domain::Group;
use anyhow::Context;
use std::sync::Arc;

pub async fn delete(
    data: DeleteGroupRequest,
    store: Arc<impl MultiRepository>,
) -> Result<Group, DeleteGroupError> {
    let opt_group = store
        .groups()
        .find(&data.group_id)
        .await
        .context("Failed to fetch group.")
        .map_err(DeleteGroupError::Unexpected)?;

    match opt_group {
        Some(mut group) => {
            group.delete(&data.user_id)?;
            Ok(group)
        }
        None => Err(DeleteGroupError::NotFound()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::tests::TestContext;
    use crate::domain::usecases::group::GroupUseCase;
    use claim::{assert_err, assert_none, assert_ok, assert_some};
    use uuid::Uuid;

    #[tokio::test]
    async fn it_should_delete_the_group_when_user_is_admin() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;

        let req = DeleteGroupRequest {
            group_id: group.id,
            user_id: group.admin_id,
        };

        // when
        let resp = ctx.group().delete_group(req.clone()).await;

        // then
        let _ = assert_ok!(resp);
        let group = ctx.find_group(&group.id).await;
        assert_none!(group);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_unauthorized_when_user_is_not_admin() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;

        let req = DeleteGroupRequest {
            group_id: group.id,
            user_id: member.id,
        };

        // when
        let resp = ctx.group().delete_group(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            DeleteGroupError::Unauthorized() => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected Unauthorized, got: {:?}", e)
            ),
        }
        let group = ctx.find_group(&group.id).await;
        assert_some!(group);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_unauthenticated_when_user_is_unknown() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;

        let req = DeleteGroupRequest {
            group_id: group.id,
            user_id: Uuid::new_v4(),
        };

        // when
        let resp = ctx.group().delete_group(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            DeleteGroupError::Unauthenticated() => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected Unauthenticated, got: {:?}", e)
            ),
        }
        let group = ctx.find_group(&group.id).await;
        assert_some!(group);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_not_fount_when_group_does_not_exist() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;

        let req = DeleteGroupRequest {
            group_id: Uuid::new_v4(),
            user_id: group.admin_id,
        };

        // when
        let resp = ctx.group().delete_group(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            DeleteGroupError::NotFound() => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error, expected Not Found, got: {:?}", e)
            ),
        }
        let group = ctx.find_group(&group.id).await;
        assert_some!(group);
        Ok(())
    }
}
