use crate::application::store::MultiRepository;
use crate::domain::errors::GenerateGroupTokenError;
use crate::domain::usecases::group::GenerateGroupTokenRequest;
use crate::domain::TokenGenerator;
use anyhow::Context;
use std::sync::Arc;

pub async fn generate(
    data: GenerateGroupTokenRequest,
    store: Arc<impl MultiRepository>,
    token_svc: Arc<dyn TokenGenerator>,
) -> Result<String, GenerateGroupTokenError> {
    let opt_group = store
        .groups()
        .find(&data.group_id)
        .await
        .context("Failed to fetch group.")
        .map_err(GenerateGroupTokenError::Unexpected)?;

    match opt_group {
        Some(group) => {
            group
                .generate_join_token(&data.user_id, token_svc.clone())
                .await
        }
        None => Err(GenerateGroupTokenError::NotFound("Group not found.")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::tests::TestContext;
    use crate::domain::usecases::group::GroupUseCase;
    use claim::{assert_err, assert_gt, assert_ok};
    use uuid::Uuid;

    #[tokio::test]
    async fn it_should_return_a_token_when_user_is_admin() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;

        let req = GenerateGroupTokenRequest {
            group_id: group.id,
            user_id: group.admin_id,
        };

        // when
        let resp = ctx.group().generate_token(req.clone()).await;

        // then
        let token = assert_ok!(resp);
        assert_gt!(token.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_not_found_when_group_does_not_exist() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let user = ctx.with_user().await;

        let req = GenerateGroupTokenRequest {
            group_id: Uuid::new_v4(),
            user_id: user.id,
        };

        // when
        let resp = ctx.group().generate_token(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            GenerateGroupTokenError::NotFound(_) => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected NotFound, got: {:?}", e)
            ),
        }
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_unauthorized_if_user_is_not_admin() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let mut group = ctx.with_group().await;
        let member = ctx.with_member(&mut group).await;

        let req = GenerateGroupTokenRequest {
            group_id: group.id,
            user_id: member.id,
        };

        // when
        let resp = ctx.group().generate_token(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            GenerateGroupTokenError::Unauthorized() => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected Unauthorized, got: {:?}", e)
            ),
        }
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_unauthenticated_if_user_is_unknown() -> Result<(), anyhow::Error> {
        // given
        let ctx = TestContext::new();
        let group = ctx.with_group().await;

        let req = GenerateGroupTokenRequest {
            group_id: group.id,
            user_id: Uuid::new_v4(),
        };

        // when
        let resp = ctx.group().generate_token(req.clone()).await;

        // then
        let err = assert_err!(resp);
        match err {
            GenerateGroupTokenError::Unauthenticated() => {}
            e => unreachable!(
                "{}",
                format!("Got incorrect error expected Unauthenticated, got: {:?}", e)
            ),
        }
        Ok(())
    }
}
