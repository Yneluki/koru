use crate::application::auth;
use crate::application::auth::{AuthService, RegisterRequest};
use crate::application::event_bus::EventBus;
use crate::application::notification::DeviceService;
use crate::application::store::MultiRepository;
use crate::domain::errors::{CreateUserError, LoginError, LogoutError};
use crate::domain::usecases::user::{
    LoginRequest, LogoutRequest, RegistrationRequest, UserUseCase,
};
use crate::domain::{Email, Event, User, UserEvent, UserEventKind};
use crate::utils::date;
use anyhow::Context;
use async_trait::async_trait;
use itertools::Itertools;
use log::warn;
use std::sync::Arc;
use uuid::Uuid;

pub struct UserUsecase<Store: MultiRepository> {
    store: Arc<Store>,
    event_bus: Arc<dyn EventBus>,
    auth_service: Option<AuthService<Store>>,
    #[cfg(feature = "pushy")]
    device_service: Arc<DeviceService<Store>>,
}

impl<Store: MultiRepository> UserUsecase<Store> {
    pub fn new(
        store: Arc<Store>,
        event_bus: Arc<dyn EventBus>,
        auth_service: Option<AuthService<Store>>,
        #[cfg(feature = "pushy")] device_service: Arc<DeviceService<Store>>,
    ) -> Self {
        Self {
            store,
            event_bus,
            auth_service,
            #[cfg(feature = "pushy")]
            device_service,
        }
    }
}

#[async_trait(?Send)]
impl<Store: MultiRepository> UserUseCase for UserUsecase<Store> {
    async fn register(&self, request: RegistrationRequest) -> Result<Uuid, CreateUserError> {
        let user = User::create(request.name, request.email.clone())?;
        let exists = self
            .store
            .users()
            .exists_by_email(&user.email)
            .await
            .context("Failed to fetch user by email")
            .map_err(CreateUserError::Unexpected)?;
        if exists {
            return Err(CreateUserError::Conflict());
        }
        match &self.auth_service {
            None => {}
            Some(auth_service) => {
                let req = RegisterRequest {
                    email: request.email,
                    password: match request.password {
                        None => Err(CreateUserError::Validation("No password provided"))?,
                        Some(password) => password,
                    },
                };
                auth_service.register(req).await?;
            }
        }
        let events = [Event::User(UserEvent {
            id: Uuid::new_v4(),
            event_date: date::now(),
            user_id: user.id,
            event: UserEventKind::Created {
                name: String::from(user.name.clone()),
                email: String::from(user.email.clone()),
            },
        })];

        self.finalize(Some(&user), &events)
            .await
            .map_err(CreateUserError::Unexpected)?;
        Ok(user.id)
    }

    async fn login(&self, request: LoginRequest) -> Result<Uuid, LoginError> {
        match &self.auth_service {
            None => {}
            Some(auth_service) => {
                let req = auth::LoginRequest {
                    email: request.email.clone(),
                    password: match request.password {
                        None => Err(LoginError::Validation("No password provided"))?,
                        Some(password) => password,
                    },
                };
                auth_service.login(req).await?;
            }
        }
        let email = Email::try_from(request.email).map_err(LoginError::Validation)?;
        let user = self
            .store
            .users()
            .find_by_email(&email)
            .await
            .context("Failed to fetch user")
            .map(|u| u.map(|u| u.id))
            .map_err(LoginError::Unexpected)?;

        match user {
            None => Err(LoginError::InvalidCredentials()),
            Some(user) => {
                let events = [Event::User(UserEvent {
                    id: Uuid::new_v4(),
                    event_date: date::now(),
                    user_id: user,
                    event: UserEventKind::Login,
                })];
                self.finalize(None, &events)
                    .await
                    .map_err(LoginError::Unexpected)?;
                Ok(user)
            }
        }
    }

    async fn logout(&self, request: LogoutRequest) -> Result<(), LogoutError> {
        self.is_valid_user(&request.user_id)
            .await
            .map_err(|_| LogoutError::Unauthenticated())?;
        #[cfg(feature = "pushy")]
        self.device_service.delete(&request.user_id).await;
        let events = [Event::User(UserEvent {
            id: Uuid::new_v4(),
            event_date: date::now(),
            user_id: request.user_id,
            event: UserEventKind::Logout,
        })];
        self.finalize(None, &events)
            .await
            .map_err(LogoutError::Unexpected)?;
        Ok(())
    }

    async fn is_valid_user(&self, user_id: &Uuid) -> Result<bool, anyhow::Error> {
        Ok(self
            .store
            .users()
            .find(user_id)
            .await
            .context("Failed to fetch user")?
            .is_some())
    }
}

impl<Store: MultiRepository> UserUsecase<Store> {
    async fn finalize(&self, user: Option<&User>, events: &[Event]) -> Result<(), anyhow::Error> {
        let mut tx = self.store.tx().await?;
        if let Some(user) = user {
            self.store
                .users()
                .save(&mut tx, user)
                .await
                .context("Failed to insert user")?;
        }
        self.store
            .events()
            .save(&mut tx, events)
            .await
            .context("Failed to save event")?;
        self.store.commit(tx.into_inner()).await?;
        self.event_bus
            .publish(&events.iter().map(|e| e.id()).collect_vec())
            .await
            .context("Failed to notify event bus.")
            .unwrap_or_else(|failure| {
                warn!("{:?}", failure);
            });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::application::tests::TestContext;
    use crate::domain::errors::{CreateUserError, LoginError};
    use crate::domain::usecases::user::{LoginRequest, RegistrationRequest};
    use crate::domain::usecases::user::{LogoutRequest, UserUseCase};
    use crate::infrastructure::store::mem::mem_store::InnerEventKind;
    use claim::{assert_err, assert_matches, assert_none, assert_ok, assert_some};
    use secrecy::{ExposeSecret, Secret};

    #[tokio::test]
    async fn it_should_create_a_user_given_valid_credentials() -> Result<(), anyhow::Error> {
        let ctx = TestContext::new();

        let req = RegistrationRequest {
            name: "bob".to_string(),
            email: "r@r.com".to_string(),
            password: Some(Secret::new("password".to_string())),
        };

        let res = ctx.user().register(req.clone()).await;

        let user_id = assert_ok!(res);
        let user = ctx.get_user(&user_id).await;
        assert_eq!(String::from(user.name.clone()), req.name);
        assert_eq!(String::from(user.email.clone()), req.email);
        let creds = ctx.get_credentials(&user_id).await;
        assert_eq!(String::from(creds.email.clone()), req.email);
        assert_ne!(
            creds.password.get()?.expose_secret(),
            &"password".to_string()
        );
        let event = assert_some!(ctx.last_stored_event());
        assert_matches!(
            event.event,
            InnerEventKind::UserCreated { .. },
            "Got incorrect event expected UserCreated"
        );
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_conflict_if_email_is_already_in_use() -> Result<(), anyhow::Error> {
        let ctx = TestContext::new();
        let user = ctx.with_user().await;

        let req = RegistrationRequest {
            name: "bob".to_string(),
            email: String::from(user.email.clone()),
            password: Some(Secret::new("password".to_string())),
        };

        let res = ctx.user().register(req.clone()).await;

        let err = assert_err!(res);
        assert_matches!(err, CreateUserError::Conflict());
        assert_none!(ctx.last_stored_event());

        let req = RegistrationRequest {
            name: "bob".to_string(),
            email: String::from(user.email.clone()),
            password: None,
        };

        let res = ctx.user_no_auth().register(req.clone()).await;

        let err = assert_err!(res);
        assert_matches!(err, CreateUserError::Conflict());
        assert_none!(ctx.last_stored_event());

        Ok(())
    }

    #[tokio::test]
    async fn it_should_create_a_user_given_valid_info_and_auth_disabled(
    ) -> Result<(), anyhow::Error> {
        let ctx = TestContext::new();

        let req = RegistrationRequest {
            name: "bob".to_string(),
            email: "r@r.com".to_string(),
            password: None,
        };

        let res = ctx.user_no_auth().register(req.clone()).await;

        let user_id = assert_ok!(res);
        let user = ctx.get_user(&user_id).await;
        assert_eq!(String::from(user.name.clone()), req.name);
        assert_eq!(String::from(user.email.clone()), req.email);
        assert_none!(ctx.find_credentials(&user_id).await);
        let event = assert_some!(ctx.last_stored_event());
        assert_matches!(
            event.event,
            InnerEventKind::UserCreated { .. },
            "Got incorrect event expected UserCreated"
        );
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_validation_error_given_invalid_request() -> Result<(), anyhow::Error>
    {
        let ctx = TestContext::new();

        let cases = vec![
            (
                RegistrationRequest {
                    name: "rbiland".to_string(),
                    email: "r@r.com".to_string(),
                    password: Some(Secret::new("".to_string())),
                },
                "empty password with auth",
            ),
            (
                RegistrationRequest {
                    name: "".to_string(),
                    email: "r@r.com".to_string(),
                    password: Some(Secret::new("201".to_string())),
                },
                "empty name with auth",
            ),
            (
                RegistrationRequest {
                    name: "rbiland".to_string(),
                    email: "".to_string(),
                    password: Some(Secret::new("201".to_string())),
                },
                "empty email with auth",
            ),
            (
                RegistrationRequest {
                    name: "rbiland".to_string(),
                    email: "rrrrr".to_string(),
                    password: Some(Secret::new("201".to_string())),
                },
                "invalid email with auth",
            ),
        ];

        for (req, desc) in cases {
            let res = ctx.user().register(req.clone()).await;
            let err = assert_err!(res);
            assert_matches!(
                err,
                CreateUserError::Validation(_),
                "Incorrect error for case {:?}",
                desc
            );
            assert_none!(ctx.last_stored_event());
        }

        let cases = vec![
            (
                RegistrationRequest {
                    name: "".to_string(),
                    email: "r@r.com".to_string(),
                    password: None,
                },
                "empty name without auth",
            ),
            (
                RegistrationRequest {
                    name: "rbiland".to_string(),
                    email: "".to_string(),
                    password: None,
                },
                "empty email without auth",
            ),
            (
                RegistrationRequest {
                    name: "rbiland".to_string(),
                    email: "rrrrr".to_string(),
                    password: None,
                },
                "invalid email without auth",
            ),
        ];
        for (req, desc) in cases {
            let res = ctx.user_no_auth().register(req.clone()).await;
            let err = assert_err!(res);
            assert_matches!(
                err,
                CreateUserError::Validation(_),
                "Incorrect error for case {:?}",
                desc
            );
            assert_none!(ctx.last_stored_event());
        }

        Ok(())
    }

    #[tokio::test]
    async fn it_should_logout_the_user_given_a_valid_user() -> Result<(), anyhow::Error> {
        let ctx = TestContext::new();
        let user = ctx.with_user().await;

        let req = LogoutRequest { user_id: user.id };

        let res = ctx.user().logout(req.clone()).await;
        assert_ok!(res);
        let event = assert_some!(ctx.last_stored_event());
        assert_matches!(
            event.event,
            InnerEventKind::UserLogout { .. },
            "Got incorrect event expected UserLogout"
        );
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);

        let res = ctx.user_no_auth().logout(req.clone()).await;
        assert_ok!(res);
        let event = assert_some!(ctx.last_stored_event());
        assert_matches!(
            event.event,
            InnerEventKind::UserLogout { .. },
            "Got incorrect event expected UserLogout"
        );
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_the_user_id_given_valid_credentials() -> Result<(), anyhow::Error> {
        let ctx = TestContext::new();
        let user = ctx.with_user().await;

        let req = LoginRequest {
            email: String::from(user.email.clone()),
            password: Some(Secret::new(
                String::from(user.email.clone()).replace("@", "p_"),
            )),
        };

        let res = ctx.user().login(req.clone()).await;

        let user_id = assert_ok!(res);
        assert_eq!(user.id, user_id);
        let event = assert_some!(ctx.last_stored_event());
        assert_matches!(
            event.event,
            InnerEventKind::UserLogin { .. },
            "Got incorrect event expected UserLogin"
        );
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);

        let req = LoginRequest {
            email: String::from(user.email.clone()),
            password: None,
        };

        let res = ctx.user_no_auth().login(req.clone()).await;

        let user_id = assert_ok!(res);
        assert_eq!(user.id, user_id);
        let event = assert_some!(ctx.last_stored_event());
        assert_matches!(
            event.event,
            InnerEventKind::UserLogin { .. },
            "Got incorrect event expected UserLogin"
        );
        let event_id = assert_some!(ctx.last_published_event());
        assert_eq!(event.id, event_id);
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_invalid_credentials_on_invalid_credentials(
    ) -> Result<(), anyhow::Error> {
        let ctx = TestContext::new();
        let user = ctx.with_user().await;

        let cases = vec![
            (
                LoginRequest {
                    email: "rrrr@rrrr.com".to_string(),
                    password: Some(Secret::new(
                        String::from(user.email.clone()).replace("@", "p_"),
                    )),
                },
                "incorrect email",
            ),
            (
                LoginRequest {
                    email: String::from(user.email.clone()),
                    password: Some(Secret::new("123".to_string())),
                },
                "incorrect password",
            ),
        ];

        for (req, desc) in cases {
            let res = ctx.user().login(req.clone()).await;

            let err = assert_err!(res);
            assert_matches!(
                err,
                LoginError::InvalidCredentials(),
                "Incorrect error for case {:?}",
                desc
            );
            assert_none!(ctx.last_stored_event());
        }

        let cases = vec![(
            LoginRequest {
                email: "rrrr@rrrr.com".to_string(),
                password: None,
            },
            "incorrect email",
        )];

        for (req, desc) in cases {
            let res = ctx.user_no_auth().login(req.clone()).await;

            let err = assert_err!(res);
            assert_matches!(
                err,
                LoginError::InvalidCredentials(),
                "Incorrect error for case {:?}",
                desc
            );
            assert_none!(ctx.last_stored_event());
        }
        Ok(())
    }

    #[tokio::test]
    async fn it_should_return_validation_error_given_invalid_input() -> Result<(), anyhow::Error> {
        let ctx = TestContext::new();
        let user = ctx.with_user().await;

        let cases = vec![
            (
                LoginRequest {
                    email: "".to_string(),
                    password: Some(Secret::new(
                        String::from(user.email.clone()).replace("@", "p_"),
                    )),
                },
                "empty email",
            ),
            (
                LoginRequest {
                    email: "rrr".to_string(),
                    password: Some(Secret::new(
                        String::from(user.email.clone()).replace("@", "p_"),
                    )),
                },
                "invalid email",
            ),
            (
                LoginRequest {
                    email: "r@r.com".to_string(),
                    password: None,
                },
                "no password",
            ),
            (
                LoginRequest {
                    email: "r@r.com".to_string(),
                    password: Some(Secret::new("".to_string())),
                },
                "empty password",
            ),
        ];

        for (req, desc) in cases {
            let res = ctx.user().login(req.clone()).await;

            let err = assert_err!(res);
            assert_matches!(
                err,
                LoginError::Validation(_),
                "Incorrect error for case {:?}",
                desc
            );
            assert_none!(ctx.last_stored_event());
        }

        let cases = vec![
            (
                LoginRequest {
                    email: "".to_string(),
                    password: None,
                },
                "empty email",
            ),
            (
                LoginRequest {
                    email: "rrr".to_string(),
                    password: None,
                },
                "invalid email",
            ),
        ];

        for (req, desc) in cases {
            let res = ctx.user_no_auth().login(req.clone()).await;

            let err = assert_err!(res);
            assert_matches!(
                err,
                LoginError::Validation(_),
                "Incorrect error for case {:?}",
                desc
            );
            assert_none!(ctx.last_stored_event());
        }
        Ok(())
    }
}
