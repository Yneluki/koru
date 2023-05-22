use crate::application::event_bus::EventBus;
use crate::application::store::MultiRepository;
use crate::domain::errors::{GetAllGroupsError, GetUsersError};
use crate::domain::usecases::admin::AdminUseCase;
use crate::domain::usecases::dto::dtos::{DetailedUserDto, GroupDto};
use crate::domain::User;
use anyhow::Context;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

pub struct AdminUsecase<Store: MultiRepository> {
    store: Arc<Store>,
    _event_bus: Arc<dyn EventBus>,
}

impl<Store: MultiRepository> AdminUsecase<Store> {
    pub fn new(store: Arc<Store>, _event_bus: Arc<dyn EventBus>) -> Self {
        Self { store, _event_bus }
    }

    async fn is_admin(&self, requester: &Uuid) -> Result<User, GetUsersError> {
        let user = self
            .store
            .users()
            .find(requester)
            .await
            .context("Failed to fetch user")?
            .ok_or(GetUsersError::Unauthenticated())?;
        if user.is_admin() {
            Ok(user)
        } else {
            Err(GetUsersError::Unauthorized())
        }
    }
}

#[async_trait(?Send)]
impl<Store: MultiRepository> AdminUseCase for AdminUsecase<Store> {
    async fn get_users(&self, requester: &Uuid) -> Result<Vec<DetailedUserDto>, GetUsersError> {
        self.is_admin(requester).await?;
        let users = self
            .store
            .users()
            .fetch_all_users()
            .await
            .context("Failed to fetch users")?;
        Ok(users.into_iter().map(DetailedUserDto::from).collect())
    }

    async fn get_groups(&self, requester: &Uuid) -> Result<Vec<GroupDto>, GetAllGroupsError> {
        self.is_admin(requester).await.map_err(|e| match e {
            GetUsersError::Unauthenticated() => GetAllGroupsError::Unauthenticated(),
            GetUsersError::Unauthorized() => GetAllGroupsError::Unauthorized(),
            GetUsersError::Unexpected(a) => GetAllGroupsError::Unexpected(a),
        })?;
        let groups = self
            .store
            .groups()
            .fetch_all_groups()
            .await
            .context("Failed to fetch groups")?;
        Ok(groups.into_iter().map(GroupDto::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::application::tests::TestContext;
    use crate::domain::errors::{GetAllGroupsError, GetUsersError};
    use crate::domain::usecases::admin::AdminUseCase;
    use claim::{assert_err, assert_matches, assert_ok, assert_some};

    #[tokio::test]
    async fn get_users_should_return_all_users_when_requested_by_admin() -> Result<(), anyhow::Error>
    {
        let ctx = TestContext::new();
        let user = ctx.with_admin_user().await;
        let user_1 = ctx.with_user().await;
        let user_2 = ctx.with_user().await;

        let res = ctx.admin().get_users(&user.id).await;

        let users = assert_ok!(res);
        assert_eq!(users.len(), 3);
        let admin = assert_some!(users.iter().find(|u| u.id == user.id));
        assert_eq!(admin.role, "Administrator".to_string());
        let u1 = assert_some!(users.iter().find(|u| u.id == user_1.id));
        assert_eq!(u1.role, "User".to_string());
        let u2 = assert_some!(users.iter().find(|u| u.id == user_2.id));
        assert_eq!(u2.role, "User".to_string());
        Ok(())
    }

    #[tokio::test]
    async fn get_users_should_return_unauthorized_when_requested_by_a_non_user(
    ) -> Result<(), anyhow::Error> {
        let ctx = TestContext::new();
        let user = ctx.with_user().await;
        let _user_1 = ctx.with_user().await;
        let _user_2 = ctx.with_user().await;

        let res = ctx.admin().get_users(&user.id).await;

        let err = assert_err!(res);
        assert_matches!(err, GetUsersError::Unauthorized());
        Ok(())
    }

    #[tokio::test]
    async fn get_groups_should_return_all_groups_when_requested_by_admin(
    ) -> Result<(), anyhow::Error> {
        let ctx = TestContext::new();
        let user = ctx.with_admin_user().await;
        let grp_1 = ctx.with_group().await;
        let grp_2 = ctx.with_group().await;

        let res = ctx.admin().get_groups(&user.id).await;

        let groups = assert_ok!(res);
        assert_eq!(groups.len(), 2);
        assert_some!(groups.iter().find(|u| u.id == grp_1.id));
        assert_some!(groups.iter().find(|u| u.id == grp_2.id));
        Ok(())
    }

    #[tokio::test]
    async fn get_groups_should_return_unauthorized_when_requested_by_a_non_user(
    ) -> Result<(), anyhow::Error> {
        let ctx = TestContext::new();
        let user = ctx.with_user().await;

        let res = ctx.admin().get_groups(&user.id).await;

        let err = assert_err!(res);
        assert_matches!(err, GetAllGroupsError::Unauthorized());
        Ok(())
    }
}
