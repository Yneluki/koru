use crate::application::admin::AdminUsecase;
use crate::application::auth::AuthService;
use crate::application::group::GroupUsecase;
use crate::application::notification::DeviceService;
use crate::application::store::MultiRepository;
use crate::application::user::UserUsecase;
use crate::configuration::application::{ApplicationSettings, AuthSettings};
use crate::infrastructure::event_bus::EventBusImpl;
use crate::infrastructure::services::credentials_hasher::ArgonCredentialsHasher;
use std::sync::Arc;

pub struct Application<Store: MultiRepository> {
    group_uc: Arc<GroupUsecase<Store>>,
    user_uc: Arc<UserUsecase<Store>>,
    admin_uc: Arc<AdminUsecase<Store>>,
    #[cfg(feature = "pushy")]
    device_service: Arc<DeviceService<Store>>,
}

impl<Store: MultiRepository> Application<Store> {
    pub fn build(
        configuration: &ApplicationSettings,
        store: Arc<Store>,
        event_bus: EventBusImpl,
        argon_memory: Option<u32>,
    ) -> Result<Self, anyhow::Error> {
        let event_bus = Arc::new(event_bus);
        let token_generator = configuration.token.setup_token_generator()?;
        let auth_service = match configuration.auth {
            AuthSettings::None => None,
            AuthSettings::Internal => Some(AuthService::new(
                store.clone(),
                ArgonCredentialsHasher::new(argon_memory),
            )),
        };
        #[cfg(feature = "pushy")]
        let device_service = Arc::new(DeviceService::new(store.clone()));
        let user_uc = Arc::new(UserUsecase::new(
            store.clone(),
            event_bus.clone(),
            auth_service,
            #[cfg(feature = "pushy")]
            device_service.clone(),
        ));
        let group_uc = Arc::new(GroupUsecase::new(
            store.clone(),
            event_bus.clone(),
            Arc::new(token_generator),
            user_uc.clone(),
        ));
        let admin_uc = Arc::new(AdminUsecase::new(store, event_bus));

        Ok(Self {
            group_uc,
            user_uc,
            admin_uc,
            #[cfg(feature = "pushy")]
            device_service,
        })
    }

    pub fn groups(&self) -> &GroupUsecase<Store> {
        self.group_uc.as_ref()
    }

    pub fn users(&self) -> &UserUsecase<Store> {
        self.user_uc.as_ref()
    }

    pub fn admin(&self) -> &AdminUsecase<Store> {
        self.admin_uc.as_ref()
    }

    #[cfg(feature = "pushy")]
    pub fn devices(&self) -> &DeviceService<Store> {
        self.device_service.as_ref()
    }
}
