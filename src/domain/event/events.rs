use crate::domain::{MemberColor, Transaction};
use crate::utils::date;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Event {
    User(UserEvent),
    Group(GroupEvent),
}

impl Event {
    pub fn id(&self) -> Uuid {
        match self {
            Event::User(u) => u.id,
            Event::Group(g) => g.id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserEvent {
    pub id: Uuid,
    pub event_date: DateTime<Utc>,
    pub user_id: Uuid,
    pub event: UserEventKind,
}

impl UserEvent {
    pub fn new(user_id: Uuid, event: UserEventKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_date: date::now(),
            user_id,
            event,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GroupEvent {
    pub id: Uuid,
    pub event_date: DateTime<Utc>,
    pub group_id: Uuid,
    pub member_id: Uuid,
    pub event: GroupEventKind,
}

impl GroupEvent {
    pub fn new(group_id: Uuid, member_id: Uuid, event: GroupEventKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_date: date::now(),
            group_id,
            member_id,
            event,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GroupEventKind {
    GroupCreated {
        name: String,
        color: MemberColor,
    },
    MemberJoined {
        color: MemberColor,
    },
    MemberColorChanged {
        previous_color: MemberColor,
        new_color: MemberColor,
    },
    ExpenseCreated {
        id: Uuid,
        description: String,
        amount: f32,
        date: DateTime<Utc>,
    },
    ExpenseModified {
        id: Uuid,
        previous_description: String,
        new_description: String,
        previous_amount: f32,
        new_amount: f32,
    },
    ExpenseDeleted {
        id: Uuid,
    },
    Settled {
        id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: DateTime<Utc>,
        transactions: Vec<Transaction>,
    },
    GroupDeleted,
}

#[derive(Debug, Clone)]
pub enum UserEventKind {
    Created { name: String, email: String },
    Login,
    Logout,
    Deleted,
}
