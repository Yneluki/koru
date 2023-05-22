use crate::application::store::{EventRepository, EventRepositoryError};
use crate::domain::{
    Amount, Event, GroupEvent, GroupEventKind, MemberColor, UserEvent, UserEventKind,
};
use crate::infrastructure::store::postgres::pg_store::PgStore;
use crate::utils::date;
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{Postgres, QueryBuilder, Transaction};
use std::cell::RefCell;
use uuid::Uuid;

#[async_trait]
impl EventRepository for PgStore {
    type Tr = Transaction<'static, Postgres>;

    #[tracing::instrument(name = "Save events in DB", skip(self, tx))]
    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        events: &[Event],
    ) -> Result<(), EventRepositoryError> {
        let mut query: QueryBuilder<Postgres> =
            QueryBuilder::new("INSERT INTO koru_event (id, event_date, event_data) ");
        query.push_values(events, |mut b, event| {
            let event = EventDto::from(event.clone());
            b.push_bind(event.id)
                .push_bind(event.date)
                .push_bind(event.data);
        });
        query
            .build()
            .execute(tx.get_mut())
            .await
            .map_err(|e| EventRepositoryError::Insert(anyhow!(e)))?;
        Ok(())
    }

    #[tracing::instrument(name = "Find event from DB", skip(self))]
    async fn find(&self, id: &Uuid) -> Result<Option<Event>, EventRepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT id, event_date, event_data
                FROM koru_event
                WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EventRepositoryError::Fetch(anyhow!(e)))?;
        match row {
            None => Ok(None),
            Some(row) => {
                let dto = EventDto {
                    id: row.id,
                    date: row.event_date,
                    data: serde_json::from_value(row.event_data)
                        .map_err(|e| EventRepositoryError::Fetch(anyhow!(e)))?,
                };
                let event = dto.event().map_err(EventRepositoryError::CorruptedData)?;
                Ok(Some(event))
            }
        }
    }

    #[tracing::instrument(name = "Mark event processed in DB", skip(self))]
    async fn mark_processed(&self, id: &Uuid) -> Result<(), EventRepositoryError> {
        sqlx::query!(
            "UPDATE koru_event SET process_date = $1 WHERE id= $2",
            date::now(),
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| EventRepositoryError::Update(anyhow!(e)))?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EventDto {
    pub id: Uuid,
    pub date: DateTime<Utc>,
    pub data: Json<EventKindDto>,
}

impl EventDto {
    pub fn event(self) -> Result<Event, &'static str> {
        let e = match self.data.0 {
            EventKindDto::GroupCreated {
                id,
                admin_id,
                name,
                color,
            } => Event::Group(GroupEvent {
                id: self.id,
                event_date: self.date,
                group_id: id,
                member_id: admin_id,
                event: GroupEventKind::GroupCreated {
                    name,
                    color: MemberColor::from(color),
                },
            }),
            EventKindDto::MemberJoined {
                group_id,
                member_id,
                color,
            } => Event::Group(GroupEvent {
                id: self.id,
                event_date: self.date,
                group_id,
                member_id,
                event: GroupEventKind::MemberJoined {
                    color: MemberColor::from(color),
                },
            }),
            EventKindDto::MemberColorChanged {
                group_id,
                member_id,
                previous_color,
                new_color,
            } => Event::Group(GroupEvent {
                id: self.id,
                event_date: self.date,
                group_id,
                member_id,
                event: GroupEventKind::MemberColorChanged {
                    previous_color: MemberColor::from(previous_color),
                    new_color: MemberColor::from(new_color),
                },
            }),
            EventKindDto::ExpenseCreated {
                id,
                group_id,
                member_id,
                description,
                amount,
                date,
            } => Event::Group(GroupEvent {
                id: self.id,
                event_date: self.date,
                group_id,
                member_id,
                event: GroupEventKind::ExpenseCreated {
                    id,
                    description,
                    amount,
                    date,
                },
            }),
            EventKindDto::ExpenseModified {
                id,
                group_id,
                member_id,
                previous_description,
                new_description,
                previous_amount,
                new_amount,
            } => Event::Group(GroupEvent {
                id: self.id,
                event_date: self.date,
                group_id,
                member_id,
                event: GroupEventKind::ExpenseModified {
                    id,
                    previous_description,
                    new_description,
                    previous_amount,
                    new_amount,
                },
            }),
            EventKindDto::ExpenseDeleted {
                id,
                group_id,
                member_id,
            } => Event::Group(GroupEvent {
                id: self.id,
                event_date: self.date,
                group_id,
                member_id,
                event: GroupEventKind::ExpenseDeleted { id },
            }),
            EventKindDto::Settled {
                id,
                group_id,
                member_id,
                start_date,
                end_date,
                transactions,
            } => {
                let mut trs = Vec::new();
                for tr in transactions {
                    trs.push(crate::domain::Transaction::try_from(tr)?);
                }
                Event::Group(GroupEvent {
                    id: self.id,
                    event_date: self.date,
                    group_id,
                    member_id,
                    event: GroupEventKind::Settled {
                        id,
                        start_date,
                        end_date,
                        transactions: trs,
                    },
                })
            }
            EventKindDto::GroupDeleted { id, admin_id } => Event::Group(GroupEvent {
                id: self.id,
                event_date: self.date,
                group_id: id,
                member_id: admin_id,
                event: GroupEventKind::GroupDeleted,
            }),

            EventKindDto::UserCreated {
                user_id,
                name,
                email,
            } => Event::User(UserEvent {
                id: self.id,
                event_date: self.date,
                user_id,
                event: UserEventKind::Created { name, email },
            }),
            EventKindDto::UserDeleted { user_id } => Event::User(UserEvent {
                id: self.id,
                event_date: self.date,
                user_id,
                event: UserEventKind::Deleted,
            }),
            EventKindDto::UserLogin { user_id } => Event::User(UserEvent {
                id: self.id,
                event_date: self.date,
                user_id,
                event: UserEventKind::Login,
            }),
            EventKindDto::UserLogout { user_id } => Event::User(UserEvent {
                id: self.id,
                event_date: self.date,
                user_id,
                event: UserEventKind::Logout,
            }),
        };
        Ok(e)
    }
}

impl From<Event> for EventDto {
    fn from(e: Event) -> Self {
        match e {
            Event::User(u) => EventDto::from(u),
            Event::Group(g) => EventDto::from(g),
        }
    }
}

impl From<UserEvent> for EventDto {
    fn from(e: UserEvent) -> Self {
        EventDto {
            id: e.id,
            date: e.event_date,
            data: Json(EventKindDto::from_user(e.event, e.user_id)),
        }
    }
}

impl From<GroupEvent> for EventDto {
    fn from(e: GroupEvent) -> Self {
        EventDto {
            id: e.id,
            date: e.event_date,
            data: Json(EventKindDto::from_group(e.event, e.group_id, e.member_id)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum EventKindDto {
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
    UserCreated {
        user_id: Uuid,
        name: String,
        email: String,
    },
    UserDeleted {
        user_id: Uuid,
    },
    UserLogin {
        user_id: Uuid,
    },
    UserLogout {
        user_id: Uuid,
    },
}

impl EventKindDto {
    fn from_group(e: GroupEventKind, group_id: Uuid, member_id: Uuid) -> Self {
        match e {
            GroupEventKind::GroupCreated { name, color } => EventKindDto::GroupCreated {
                id: group_id,
                admin_id: member_id,
                name,
                color: ColorDto::from(color),
            },
            GroupEventKind::MemberJoined { color } => EventKindDto::MemberJoined {
                group_id,
                member_id,
                color: ColorDto::from(color),
            },
            GroupEventKind::MemberColorChanged {
                previous_color,
                new_color,
            } => EventKindDto::MemberColorChanged {
                group_id,
                member_id,
                previous_color: ColorDto::from(previous_color),
                new_color: ColorDto::from(new_color),
            },
            GroupEventKind::ExpenseCreated {
                id,
                description,
                amount,
                date,
            } => EventKindDto::ExpenseCreated {
                id,
                group_id,
                member_id,
                description,
                amount,
                date,
            },
            GroupEventKind::ExpenseModified {
                id,
                previous_description,
                new_description,
                previous_amount,
                new_amount,
            } => EventKindDto::ExpenseModified {
                id,
                group_id,
                member_id,
                previous_description,
                new_description,
                previous_amount,
                new_amount,
            },
            GroupEventKind::ExpenseDeleted { id } => EventKindDto::ExpenseDeleted {
                id,
                group_id,
                member_id,
            },
            GroupEventKind::Settled {
                id,
                start_date,
                end_date,
                transactions,
            } => EventKindDto::Settled {
                id,
                group_id,
                member_id,
                start_date,
                end_date,
                transactions: transactions
                    .iter()
                    .map(|t| TransactionDto::from(t.clone()))
                    .collect_vec(),
            },
            GroupEventKind::GroupDeleted => EventKindDto::GroupDeleted {
                id: group_id,
                admin_id: member_id,
            },
        }
    }
    fn from_user(e: UserEventKind, user_id: Uuid) -> Self {
        match e {
            UserEventKind::Created { name, email } => EventKindDto::UserCreated {
                user_id,
                name,
                email,
            },
            UserEventKind::Login => EventKindDto::UserLogin { user_id },
            UserEventKind::Logout => EventKindDto::UserLogout { user_id },
            UserEventKind::Deleted => EventKindDto::UserDeleted { user_id },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TransactionDto {
    pub from: Uuid,
    pub to: Uuid,
    pub amount: f32,
}

impl From<crate::domain::Transaction> for TransactionDto {
    fn from(t: crate::domain::Transaction) -> Self {
        TransactionDto {
            from: t.from,
            to: t.to,
            amount: f32::from(t.amount),
        }
    }
}

impl TryFrom<TransactionDto> for crate::domain::Transaction {
    type Error = &'static str;
    fn try_from(t: TransactionDto) -> Result<Self, Self::Error> {
        Ok(crate::domain::Transaction {
            from: t.from,
            to: t.to,
            amount: Amount::try_from(t.amount)?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ColorDto {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl From<MemberColor> for ColorDto {
    fn from(c: MemberColor) -> Self {
        ColorDto {
            red: c.red,
            green: c.green,
            blue: c.blue,
        }
    }
}

impl From<ColorDto> for MemberColor {
    fn from(c: ColorDto) -> Self {
        MemberColor {
            red: c.red,
            green: c.green,
            blue: c.blue,
        }
    }
}
