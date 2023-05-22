use crate::application::event_bus::EventListener;
#[cfg(feature = "notification")]
use crate::application::notification::Notifier;
use crate::configuration::application::ApplicationSettings;
use crate::infrastructure::event_bus::EventListenerImpl;
use crate::infrastructure::store::StoreImpl;
use futures_util::future::BoxFuture;
use std::sync::Arc;

pub struct Worker {
    fut: BoxFuture<'static, Result<(), anyhow::Error>>,
}

impl Worker {
    pub async fn build(
        configuration: &ApplicationSettings,
        mut listener: EventListenerImpl,
        store: Arc<StoreImpl>,
    ) -> Result<Self, anyhow::Error> {
        #[cfg(feature = "notification")]
        {
            let notification_svc = configuration
                .notification
                .as_ref()
                .map(|conf| conf.setup_notification_svc(store.clone()));
            match notification_svc {
                None => {}
                Some(notification_svc) => {
                    let notification_svc = notification_svc?;
                    listener.register(Notifier::new(store, Arc::new(notification_svc)));
                }
            }
        }
        let a = listener.listen();
        Ok(Self { fut: a })
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        self.fut.await
    }
}
