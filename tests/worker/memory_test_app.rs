use crate::test_app::EventKindDto;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use koru::application::event_bus::EventBus;
use koru::configuration::Settings;
use koru::infrastructure::event_bus::direct_event_bus::DirectEventBus;
use koru::infrastructure::event_bus::EventBusImpl;
use koru::infrastructure::store::mem::mem_store::{
    InnerColor, InnerEvent, InnerEventKind, InnerExpense, InnerGroup, InnerMember, InnerRole,
    InnerTransaction, InnerUser,
};
use koru::infrastructure::store::{InMemoryStore, StoreImpl};
use koru::worker::Worker;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use wiremock::MockServer;

pub struct MemTestApp {
    pub notification_server: MockServer,
    bus: Arc<DirectEventBus>,
    store: Arc<InMemoryStore>,
}

impl MemTestApp {
    pub async fn build(notification_server: MockServer, configuration: &Settings) -> Self {
        let (event_bus, event_listener) = EventBusImpl::build(&configuration.event_bus)
            .await
            .expect("Failed to setup event bus.");
        let store = Arc::new(InMemoryStore::new());
        let worker = Worker::build(
            &configuration.application,
            event_listener,
            Arc::new(StoreImpl::Memory(store.clone())),
        )
        .await
        .expect("Failed to setup worker.");
        let _ = tokio::spawn(worker.run());
        MemTestApp {
            notification_server,
            bus: match event_bus {
                #[cfg(feature = "redis-bus")]
                EventBusImpl::Redis(_) => {
                    panic!("Cannot have redis settings with in memory bus")
                }
                EventBusImpl::Direct(bus) => bus.clone(),
            },
            store: store.clone(),
        }
    }

    pub async fn publish_event(&self, event_id: Uuid) {
        self.bus.publish(&[event_id]).await.unwrap();
        // As the event is published asynchronously, we need to wait a while for the event to be processed.
        // otherwise all tests will fail as they will directly return after having sent the event.
        sleep(Duration::from_millis(100)).await
    }

    pub async fn with_user(&self, name: String, email: String, device: String) -> Uuid {
        let id = Uuid::new_v4();
        self.store.users.lock().unwrap().insert(
            id,
            InnerUser {
                id,
                name,
                email,
                role: InnerRole::USER,
                created_at: Utc::now(),
            },
        );
        self.store.user_devices.lock().unwrap().insert(id, device);
        id
    }

    pub async fn with_group(&self, name: String, admin: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        self.store.groups.lock().unwrap().insert(
            id,
            InnerGroup {
                id,
                name,
                admin_id: admin,
                created_at: Utc::now(),
                member_ids: vec![admin],
                expenses: vec![],
                settlements: vec![],
            },
        );
        self.store.members.lock().unwrap().insert(
            (admin, id),
            InnerMember {
                id: (admin, id),
                is_admin: true,
                color: InnerColor {
                    red: 0,
                    green: 255,
                    blue: 0,
                },
                joined_at: Utc::now(),
            },
        );
        id
    }

    pub async fn with_member(&self, group: Uuid, user: Uuid) {
        self.store.members.lock().unwrap().insert(
            (user, group),
            InnerMember {
                id: (user, group),
                is_admin: false,
                color: InnerColor {
                    red: 0,
                    green: 255,
                    blue: 0,
                },
                joined_at: Utc::now(),
            },
        );
    }

    pub async fn with_expense(&self, group: Uuid, user: Uuid, desc: String, amount: f32) -> Uuid {
        let id = Uuid::new_v4();
        self.store.expenses.lock().unwrap().insert(
            id,
            InnerExpense {
                id,
                group_id: group,
                title: desc.clone(),
                amount,
                member_id: user,
                created_at: Utc::now(),
                modified_at: None,
                settled: false,
            },
        );
        id
    }

    pub async fn with_event(&self, event: EventKindDto) -> Uuid {
        let id = Uuid::new_v4();
        self.store.events.lock().unwrap().push(InnerEvent {
            id,
            event: InnerEventKind::from(event),
            date: Utc::now(),
            processed_date: None,
        });
        id
    }

    pub async fn get_event_process_date(&self, event: Uuid) -> Option<DateTime<Utc>> {
        self.store
            .events
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.id == event)
            .map(|e| e.processed_date)
            .unwrap()
    }
}

impl From<EventKindDto> for InnerEventKind {
    fn from(value: EventKindDto) -> Self {
        match value {
            EventKindDto::GroupCreated {
                id,
                admin_id,
                name,
                color,
            } => InnerEventKind::GroupCreated {
                id,
                admin_id,
                name,
                color: InnerColor {
                    red: color.red,
                    green: color.green,
                    blue: color.blue,
                },
            },
            EventKindDto::MemberJoined {
                group_id,
                member_id,
                color,
            } => InnerEventKind::MemberJoined {
                group_id,
                member_id,
                color: InnerColor {
                    red: color.red,
                    green: color.green,
                    blue: color.blue,
                },
            },
            EventKindDto::MemberColorChanged {
                group_id,
                member_id,
                previous_color,
                new_color,
            } => InnerEventKind::MemberColorChanged {
                group_id,
                member_id,
                previous_color: InnerColor {
                    red: previous_color.red,
                    green: previous_color.green,
                    blue: previous_color.blue,
                },
                new_color: InnerColor {
                    red: new_color.red,
                    green: new_color.green,
                    blue: new_color.blue,
                },
            },
            EventKindDto::ExpenseCreated {
                id,
                group_id,
                member_id,
                description,
                amount,
                date,
            } => InnerEventKind::ExpenseCreated {
                id,
                group_id,
                member_id,
                description,
                amount,
                date,
            },
            EventKindDto::ExpenseModified {
                id,
                group_id,
                member_id,
                previous_description,
                new_description,
                previous_amount,
                new_amount,
            } => InnerEventKind::ExpenseModified {
                id,
                group_id,
                member_id,
                previous_description,
                previous_amount,
                new_description,
                new_amount,
            },
            EventKindDto::ExpenseDeleted {
                id,
                group_id,
                member_id,
            } => InnerEventKind::ExpenseDeleted {
                id,
                group_id,
                member_id,
            },
            EventKindDto::Settled {
                id,
                group_id,
                member_id,
                start_date,
                end_date,
                transactions,
            } => InnerEventKind::Settled {
                id,
                group_id,
                member_id,
                start_date,
                end_date,
                transactions: transactions
                    .iter()
                    .map(|t| InnerTransaction {
                        from: t.from,
                        to: t.to,
                        amount: t.amount,
                    })
                    .collect_vec(),
            },
            EventKindDto::GroupDeleted { id, admin_id } => {
                InnerEventKind::GroupDeleted { id, admin_id }
            }
        }
    }
}
