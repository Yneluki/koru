#[cfg(any(feature = "production", feature = "development"))]
use crate::external_test_app::ExtTestApp;
use crate::memory_test_app::MemTestApp;
use chrono::{DateTime, Utc};
use koru::configuration::event_bus::EventBusSettings;
use koru::configuration::notification::NotificationSettings;
use koru::configuration::store::DatabaseSettings;
use koru::configuration::{get_configuration, Settings};
use koru::utils::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use test_context::AsyncTestContext;
use uuid::Uuid;
use wiremock::MockServer;

// Ensure that the `tracing` stack is only initialised once
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "sqlx=error,info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub enum Infra {
    Memory,
    #[cfg(any(feature = "production", feature = "development"))]
    External,
}

impl TryFrom<String> for Infra {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Memory),
            #[cfg(any(feature = "production", feature = "development"))]
            "integration" => Ok(Self::External),
            other => Err(format!(
                "`{}` is not a supported infra. Use either `local` or `integration` (requires `--features full`).",
                other
            )),
        }
    }
}

pub enum TestApp {
    #[cfg(any(feature = "production", feature = "development"))]
    External(ExtTestApp),
    Memory(MemTestApp),
}

fn in_memory_config(mock_server: String) -> Settings {
    let mut c = get_configuration().expect("Failed to read configuration.");
    c.database = DatabaseSettings::Memory;
    // Use mock server for notifications
    c.application.notification = match c.application.notification {
        None => None,
        Some(notif) => match notif {
            NotificationSettings::Pushy(mut pushy) => {
                pushy.url = mock_server;
                Some(NotificationSettings::Pushy(pushy))
            }
        },
    };
    c.event_bus = EventBusSettings::Memory;
    c
}

#[cfg(any(feature = "production", feature = "development"))]
fn ext_config(mock_server: String) -> Settings {
    let mut c = get_configuration().expect("Failed to read configuration.");
    // Use a different database for each test case
    c.database = match c.database {
        DatabaseSettings::Postgres(mut postgres) => {
            postgres.name = Uuid::new_v4().to_string();
            DatabaseSettings::Postgres(postgres)
        }
        DatabaseSettings::Memory => DatabaseSettings::Memory,
    };
    // Use mock server for notifications
    c.application.notification = match c.application.notification {
        None => None,
        Some(notif) => match notif {
            NotificationSettings::Pushy(mut pushy) => {
                pushy.url = mock_server;
                Some(NotificationSettings::Pushy(pushy))
            }
        },
    };
    // Use a different channel for each test case
    c.event_bus = match c.event_bus {
        EventBusSettings::Redis(mut redis) => {
            redis.event_channel = Uuid::new_v4().to_string();
            EventBusSettings::Redis(redis)
        }
        EventBusSettings::Memory => EventBusSettings::Memory,
    };
    c
}

#[async_trait::async_trait]
impl AsyncTestContext for TestApp {
    async fn setup() -> TestApp {
        Lazy::force(&TRACING);

        let notification_server = MockServer::start().await;

        let infra: Infra = std::env::var("KORU_ENV")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse KORU_ENV.");

        match infra {
            Infra::Memory => {
                let configuration = in_memory_config(notification_server.uri());
                let app = MemTestApp::build(notification_server, &configuration).await;
                TestApp::Memory(app)
            }
            #[cfg(any(feature = "production", feature = "development"))]
            Infra::External => {
                let configuration = ext_config(notification_server.uri());
                let app = ExtTestApp::build(notification_server, &configuration).await;
                TestApp::External(app)
            }
        }
    }

    async fn teardown(self) {
        match self {
            #[cfg(any(feature = "production", feature = "development"))]
            TestApp::External(app) => {
                app.teardown().await;
            }
            TestApp::Memory(_) => {}
        }
    }
}

impl TestApp {
    pub fn notification_server(&self) -> &MockServer {
        match self {
            #[cfg(any(feature = "production", feature = "development"))]
            TestApp::External(app) => &app.notification_server,
            TestApp::Memory(app) => &app.notification_server,
        }
    }

    pub async fn publish_event(&self, event_id: Uuid) {
        match self {
            #[cfg(any(feature = "production", feature = "development"))]
            TestApp::External(app) => app.publish_event(event_id).await,
            TestApp::Memory(app) => app.publish_event(event_id).await,
        }
    }

    pub async fn with_user(&self, name: String, email: String, device: String) -> Uuid {
        match self {
            #[cfg(any(feature = "production", feature = "development"))]
            TestApp::External(app) => app.with_user(name, email, device).await,
            TestApp::Memory(app) => app.with_user(name, email, device).await,
        }
    }

    pub async fn with_group(&self, name: String, admin: Uuid) -> Uuid {
        match self {
            #[cfg(any(feature = "production", feature = "development"))]
            TestApp::External(app) => app.with_group(name, admin).await,
            TestApp::Memory(app) => app.with_group(name, admin).await,
        }
    }

    pub async fn with_member(&self, group: Uuid, user: Uuid) {
        match self {
            #[cfg(any(feature = "production", feature = "development"))]
            TestApp::External(app) => app.with_member(group, user).await,
            TestApp::Memory(app) => app.with_member(group, user).await,
        }
    }

    pub async fn with_expense(&self, group: Uuid, user: Uuid, desc: String, amount: f32) -> Uuid {
        match self {
            #[cfg(any(feature = "production", feature = "development"))]
            TestApp::External(app) => app.with_expense(group, user, desc, amount).await,
            TestApp::Memory(app) => app.with_expense(group, user, desc, amount).await,
        }
    }

    pub async fn with_event(&self, event: EventKindDto) -> Uuid {
        match self {
            #[cfg(any(feature = "production", feature = "development"))]
            TestApp::External(app) => app.with_event(event).await,
            TestApp::Memory(app) => app.with_event(event).await,
        }
    }

    pub async fn get_event_process_date(&self, event: Uuid) -> Option<DateTime<Utc>> {
        match self {
            #[cfg(any(feature = "production", feature = "development"))]
            TestApp::External(app) => app.get_event_process_date(event).await,
            TestApp::Memory(app) => app.get_event_process_date(event).await,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventKindDto {
    GroupCreated {
        id: Uuid,
        admin_id: Uuid,
        name: String,
        color: ColorDto,
    },
    MemberJoined {
        group_id: Uuid,
        member_id: Uuid,
        color: ColorDto,
    },
    MemberColorChanged {
        group_id: Uuid,
        member_id: Uuid,
        previous_color: ColorDto,
        new_color: ColorDto,
    },
    ExpenseCreated {
        id: Uuid,
        group_id: Uuid,
        member_id: Uuid,
        description: String,
        amount: f32,
        date: DateTime<Utc>,
    },
    ExpenseModified {
        id: Uuid,
        group_id: Uuid,
        member_id: Uuid,
        previous_description: String,
        new_description: String,
        previous_amount: f32,
        new_amount: f32,
    },
    ExpenseDeleted {
        id: Uuid,
        group_id: Uuid,
        member_id: Uuid,
    },
    Settled {
        id: Uuid,
        group_id: Uuid,
        member_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: DateTime<Utc>,
        transactions: Vec<TransactionDto>,
    },
    GroupDeleted {
        id: Uuid,
        admin_id: Uuid,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ColorDto {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionDto {
    pub from: Uuid,
    pub to: Uuid,
    pub amount: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Notification {
    pub to: String,
    pub data: NotificationData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotificationData {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PushyOkResponse {
    pub id: String,
    pub success: bool,
    pub info: PushyInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PushyInfo {
    pub devices: i64,
}
