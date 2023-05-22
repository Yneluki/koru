use crate::application::auth::CredentialRepository;
use crate::application::store::{
    DeviceRepository, EventRepository, ExpenseRepository, GroupRepository, MemberRepository,
    MultiRepository, SettlementRepository, Tx, UserRepository,
};
use crate::domain::{
    Amount, Email, Event, Expense, ExpenseTitle, Group, GroupEvent, GroupEventKind, GroupMember,
    GroupName, MemberColor, Settlement, SettlementDescription, Transaction, User, UserEvent,
    UserEventKind, UserName, UserRole,
};
use anyhow::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryStore {
    pub crash_groups: AtomicBool,
    pub crash_users: AtomicBool,
    pub crash_user_devices: AtomicBool,
    pub crash_user_credentials: AtomicBool,
    pub crash_members: AtomicBool,
    pub crash_expenses: AtomicBool,
    pub crash_settlements: AtomicBool,
    pub crash_events: AtomicBool,
    pub users: Mutex<HashMap<Uuid, InnerUser>>,
    pub user_devices: Mutex<HashMap<Uuid, String>>,
    pub user_credentials: Mutex<HashMap<String, String>>,
    pub groups: Mutex<HashMap<Uuid, InnerGroup>>,
    pub members: Mutex<HashMap<(Uuid, Uuid), InnerMember>>,
    pub expenses: Mutex<HashMap<Uuid, InnerExpense>>,
    pub settlements: Mutex<HashMap<Uuid, InnerSettlement>>,
    pub events: Mutex<Vec<InnerEvent>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            crash_groups: AtomicBool::from(false),
            crash_users: AtomicBool::from(false),
            crash_user_devices: AtomicBool::from(false),
            crash_user_credentials: AtomicBool::from(false),
            crash_members: AtomicBool::from(false),
            crash_expenses: AtomicBool::from(false),
            crash_settlements: AtomicBool::from(false),
            crash_events: AtomicBool::from(false),
            users: Mutex::new(HashMap::new()),
            user_devices: Mutex::new(HashMap::new()),
            user_credentials: Mutex::new(HashMap::new()),
            groups: Mutex::new(HashMap::new()),
            members: Mutex::new(HashMap::new()),
            expenses: Mutex::new(HashMap::new()),
            settlements: Mutex::new(HashMap::new()),
            events: Mutex::new(Vec::new()),
        }
    }

    pub fn crash_groups(&self) {
        self.crash_groups.store(true, Relaxed);
    }

    pub fn crash_users(&self) {
        self.crash_users.store(true, Relaxed);
    }

    pub fn crash_credentials(&self) {
        self.crash_user_credentials.store(true, Relaxed);
    }

    #[cfg(test)]
    pub async fn get_event(&self, id: &Uuid) -> InnerEvent {
        self.events
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.id == *id)
            .map(|e| e.clone())
            .unwrap()
    }
}

#[derive(Default)]
pub struct InMemTx {
    pub users: Mutex<HashMap<Uuid, InnerUser>>,
    pub user_devices: Mutex<HashMap<Uuid, String>>,
    pub user_credentials: Mutex<HashMap<String, String>>,
    pub groups: Mutex<HashMap<Uuid, InnerGroup>>,
    pub deleted_groups: Mutex<HashSet<Uuid>>,
    pub members: Mutex<HashMap<(Uuid, Uuid), InnerMember>>,
    pub expenses: Mutex<HashMap<Uuid, InnerExpense>>,
    pub deleted_expenses: Mutex<HashSet<Uuid>>,
    pub settlements: Mutex<HashMap<Uuid, InnerSettlement>>,
    pub events: Mutex<Vec<InnerEvent>>,
}

impl InMemTx {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
            user_devices: Mutex::new(HashMap::new()),
            user_credentials: Mutex::new(HashMap::new()),
            groups: Mutex::new(HashMap::new()),
            deleted_groups: Mutex::new(HashSet::new()),
            members: Mutex::new(HashMap::new()),
            expenses: Mutex::new(HashMap::new()),
            deleted_expenses: Mutex::new(HashSet::new()),
            settlements: Mutex::new(HashMap::new()),
            events: Mutex::new(Vec::new()),
        }
    }
}

impl Tx for InMemTx {}

#[async_trait]
impl MultiRepository for InMemoryStore {
    type KTransaction = InMemTx;

    async fn tx(&self) -> Result<RefCell<Self::KTransaction>, Error> {
        Ok(RefCell::new(InMemTx::new()))
    }

    async fn commit(&self, tx: Self::KTransaction) -> Result<(), Error> {
        {
            let guard = tx.expenses.lock().unwrap();
            let expenses = guard.iter();
            for (id, expense) in expenses {
                self.expenses.lock().unwrap().insert(*id, expense.clone());
            }
        }
        {
            let guard = tx.groups.lock().unwrap();
            let groups = guard.iter();
            for (id, group) in groups {
                self.groups.lock().unwrap().insert(*id, group.clone());
            }
        }
        {
            let guard = tx.members.lock().unwrap();
            let members = guard.iter();
            for (id, member) in members {
                self.members.lock().unwrap().insert(*id, member.clone());
            }
        }
        {
            let guard = tx.settlements.lock().unwrap();
            let settlements = guard.iter();
            for (id, settlement) in settlements {
                self.settlements
                    .lock()
                    .unwrap()
                    .insert(*id, settlement.clone());
            }
        }
        {
            let guard = tx.users.lock().unwrap();
            let users = guard.iter();
            for (id, user) in users {
                self.users.lock().unwrap().insert(*id, user.clone());
            }
        }
        {
            let guard = tx.user_devices.lock().unwrap();
            let user_devices = guard.iter();
            for (user_id, device) in user_devices {
                self.user_devices
                    .lock()
                    .unwrap()
                    .insert(*user_id, device.clone());
            }
        }
        {
            let guard = tx.user_credentials.lock().unwrap();
            let user_creds = guard.iter();
            for (email, password) in user_creds {
                self.user_credentials
                    .lock()
                    .unwrap()
                    .insert(email.clone(), password.clone());
            }
        }
        {
            let guard = tx.events.lock().unwrap();
            let events = guard.iter();
            for event in events {
                self.events.lock().unwrap().push(event.clone());
            }
        }
        {
            let guard = tx.deleted_expenses.lock().unwrap();
            let del_expenses = guard.iter();
            for id in del_expenses {
                self.expenses.lock().unwrap().remove(id);
            }
        }
        {
            let guard = tx.deleted_groups.lock().unwrap();
            let del_groups = guard.iter();
            for id in del_groups {
                self.groups.lock().unwrap().remove(id);
            }
        }
        Ok(())
    }

    fn users(&self) -> &dyn UserRepository<Tr = Self::KTransaction> {
        self
    }

    fn credentials(&self) -> &dyn CredentialRepository<Tr = Self::KTransaction> {
        self
    }

    fn device(&self) -> &dyn DeviceRepository<Tr = Self::KTransaction> {
        self
    }

    fn groups(&self) -> &dyn GroupRepository<Tr = Self::KTransaction> {
        self
    }

    fn members(&self) -> &dyn MemberRepository<Tr = Self::KTransaction> {
        self
    }

    fn expenses(&self) -> &dyn ExpenseRepository<Tr = Self::KTransaction> {
        self
    }

    fn settlements(&self) -> &dyn SettlementRepository<Tr = Self::KTransaction> {
        self
    }

    fn events(&self) -> &dyn EventRepository<Tr = Self::KTransaction> {
        self
    }
}

#[derive(Clone, Debug)]
pub struct InnerUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: InnerRole,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug)]
pub enum InnerRole {
    ADMINISTRATOR,
    USER,
}

impl TryFrom<InnerUser> for User {
    type Error = &'static str;
    fn try_from(value: InnerUser) -> Result<Self, Self::Error> {
        let name = UserName::try_from(value.name)?;
        let email = Email::try_from(value.email)?;
        let role = match value.role {
            InnerRole::ADMINISTRATOR => UserRole::Administrator,
            InnerRole::USER => UserRole::User,
        };
        Ok(Self {
            id: value.id,
            name,
            email,
            role,
            created_at: value.created_at,
        })
    }
}

#[derive(Clone, Debug)]
pub struct InnerMember {
    pub id: (Uuid, Uuid),
    pub is_admin: bool,
    pub color: InnerColor,
    pub joined_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct InnerColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl From<InnerColor> for String {
    fn from(n: InnerColor) -> Self {
        format!("{},{},{}", n.red, n.green, n.blue)
    }
}

impl GroupMember {
    pub fn try_from(value: InnerMember, user: InnerUser) -> Result<Self, &'static str> {
        let name = UserName::try_from(user.name)?;
        let email = Email::try_from(user.email)?;
        let color = MemberColor::from(value.color);
        Ok(Self {
            id: value.id.0,
            name,
            email,
            group_id: value.id.1,
            is_admin: value.is_admin,
            color,
            joined_at: value.joined_at,
        })
    }
}

impl From<InnerColor> for MemberColor {
    fn from(value: InnerColor) -> Self {
        Self {
            red: value.red,
            green: value.green,
            blue: value.blue,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InnerGroup {
    pub id: Uuid,
    pub name: String,
    pub admin_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub member_ids: Vec<Uuid>,
    pub expenses: Vec<Uuid>,
    pub settlements: Vec<Uuid>,
}

impl InnerGroup {
    pub fn build_group(self, members: Vec<GroupMember>) -> Result<Group, &'static str> {
        let name = GroupName::try_from(self.name)?;
        Ok(Group {
            id: self.id,
            name,
            admin_id: self.admin_id,
            created_at: self.created_at,
            members,
            expense_ids: self.expenses,
            settlement_ids: self.settlements,
            events: vec![],
        })
    }
}

#[derive(Clone, Debug)]
pub struct InnerExpense {
    pub id: Uuid,
    pub group_id: Uuid,
    pub title: String,
    pub amount: f32,
    pub member_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
    pub settled: bool,
}

impl TryFrom<InnerExpense> for Expense {
    type Error = &'static str;
    fn try_from(value: InnerExpense) -> Result<Self, Self::Error> {
        let title = ExpenseTitle::try_from(value.title)?;
        let amount = Amount::try_from(value.amount)?;
        Ok(Self {
            id: value.id,
            group_id: value.group_id,
            title,
            amount,
            member_id: value.member_id,
            created_at: value.created_at,
            modified_at: value.modified_at,
            settled: value.settled,
        })
    }
}

#[derive(Clone, Debug)]
pub struct InnerSettlement {
    pub id: Uuid,
    pub group_id: Uuid,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: DateTime<Utc>,
    pub transactions: Vec<InnerTransaction>,
    pub expenses: Vec<Uuid>,
}

impl TryFrom<InnerSettlement> for Settlement {
    type Error = &'static str;
    fn try_from(value: InnerSettlement) -> Result<Self, Self::Error> {
        let mut transactions = Vec::new();
        for tr in value.transactions {
            transactions.push(Transaction::try_from(tr)?);
        }
        Ok(Self {
            id: value.id,
            group_id: value.group_id,
            start_date: value.start_date,
            end_date: value.end_date,
            transactions,
            expense_ids: value.expenses,
        })
    }
}

impl TryFrom<InnerSettlement> for SettlementDescription {
    type Error = &'static str;
    fn try_from(value: InnerSettlement) -> Result<Self, Self::Error> {
        let mut transactions = Vec::new();
        for tr in value.transactions {
            transactions.push(Transaction::try_from(tr)?);
        }
        Ok(Self {
            id: value.id,
            group_id: value.group_id,
            start_date: value.start_date,
            end_date: value.end_date,
        })
    }
}

#[derive(Clone, Debug)]
pub struct InnerTransaction {
    pub from: Uuid,
    pub to: Uuid,
    pub amount: f32,
}

impl TryFrom<InnerTransaction> for Transaction {
    type Error = &'static str;
    fn try_from(value: InnerTransaction) -> Result<Self, Self::Error> {
        let amount = Amount::try_from(value.amount)?;
        Ok(Self {
            from: value.from,
            to: value.to,
            amount,
        })
    }
}

#[derive(Clone, Debug)]
pub struct InnerEvent {
    pub id: Uuid,
    pub event: InnerEventKind,
    pub date: DateTime<Utc>,
    pub processed_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub enum InnerEventKind {
    GroupCreated {
        id: Uuid,
        admin_id: Uuid,
        name: String,
        color: InnerColor,
    },
    MemberJoined {
        group_id: Uuid,
        member_id: Uuid,
        color: InnerColor,
    },
    MemberColorChanged {
        group_id: Uuid,
        member_id: Uuid,
        previous_color: InnerColor,
        new_color: InnerColor,
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
        transactions: Vec<InnerTransaction>,
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

impl From<Event> for InnerEvent {
    fn from(value: Event) -> Self {
        match value {
            Event::User(u) => InnerEvent::from(u),
            Event::Group(g) => InnerEvent::from(g),
        }
    }
}

impl From<GroupEvent> for InnerEvent {
    fn from(e: GroupEvent) -> Self {
        InnerEvent {
            id: e.id,
            event: InnerEventKind::from_group(e.event, e.group_id, e.member_id),
            date: e.event_date,
            processed_date: None,
        }
    }
}

impl From<UserEvent> for InnerEvent {
    fn from(e: UserEvent) -> Self {
        InnerEvent {
            id: e.id,
            event: InnerEventKind::from_user(e.event, e.user_id),
            date: e.event_date,
            processed_date: None,
        }
    }
}

impl InnerEventKind {
    pub fn name(&self) -> &'static str {
        match self {
            InnerEventKind::GroupCreated { .. } => "GroupCreated",
            InnerEventKind::MemberJoined { .. } => "MemberJoined",
            InnerEventKind::MemberColorChanged { .. } => "MemberColorChanged",
            InnerEventKind::ExpenseCreated { .. } => "ExpenseCreated",
            InnerEventKind::ExpenseModified { .. } => "ExpenseModified",
            InnerEventKind::ExpenseDeleted { .. } => "ExpenseDeleted",
            InnerEventKind::Settled { .. } => "Settled",
            InnerEventKind::GroupDeleted { .. } => "GroupDeleted",
            InnerEventKind::UserCreated { .. } => "UserCreated",
            InnerEventKind::UserDeleted { .. } => "UserDeleted",
            InnerEventKind::UserLogin { .. } => "UserLogin",
            InnerEventKind::UserLogout { .. } => "UserLogout",
        }
    }

    fn from_user(e: UserEventKind, user_id: Uuid) -> Self {
        match e {
            UserEventKind::Created { name, email } => InnerEventKind::UserCreated {
                user_id,
                name,
                email,
            },
            UserEventKind::Login => InnerEventKind::UserLogin { user_id },
            UserEventKind::Logout => InnerEventKind::UserLogout { user_id },
            UserEventKind::Deleted => InnerEventKind::UserDeleted { user_id },
        }
    }

    fn from_group(e: GroupEventKind, group_id: Uuid, member_id: Uuid) -> Self {
        match e {
            GroupEventKind::GroupCreated { name, color } => InnerEventKind::GroupCreated {
                id: group_id,
                admin_id: member_id,
                name,
                color: InnerColor {
                    red: color.red,
                    green: color.green,
                    blue: color.blue,
                },
            },
            GroupEventKind::MemberJoined { color } => InnerEventKind::MemberJoined {
                group_id,
                member_id,
                color: InnerColor {
                    red: color.red,
                    green: color.green,
                    blue: color.blue,
                },
            },
            GroupEventKind::MemberColorChanged {
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
            GroupEventKind::ExpenseCreated {
                id,
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
            GroupEventKind::ExpenseModified {
                id,
                previous_description,
                new_description,
                previous_amount,
                new_amount,
            } => InnerEventKind::ExpenseModified {
                id,
                group_id,
                member_id,
                previous_description,
                new_description,
                previous_amount,
                new_amount,
            },
            GroupEventKind::ExpenseDeleted { id } => InnerEventKind::ExpenseDeleted {
                id,
                group_id,
                member_id,
            },
            GroupEventKind::Settled {
                id,
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
                        amount: f32::from(t.amount),
                    })
                    .collect_vec(),
            },
            GroupEventKind::GroupDeleted => InnerEventKind::GroupDeleted {
                id: group_id,
                admin_id: member_id,
            },
        }
    }
}

impl InnerEvent {
    pub fn event(self) -> Result<Event, &'static str> {
        let e = match self.event {
            InnerEventKind::GroupCreated {
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
            InnerEventKind::MemberJoined {
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
            InnerEventKind::MemberColorChanged {
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
            InnerEventKind::ExpenseCreated {
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
            InnerEventKind::ExpenseModified {
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
            InnerEventKind::ExpenseDeleted {
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
            InnerEventKind::Settled {
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
            InnerEventKind::GroupDeleted { id, admin_id } => Event::Group(GroupEvent {
                id: self.id,
                event_date: self.date,
                group_id: id,
                member_id: admin_id,
                event: GroupEventKind::GroupDeleted,
            }),

            InnerEventKind::UserCreated {
                user_id,
                name,
                email,
            } => Event::User(UserEvent {
                id: self.id,
                event_date: self.date,
                user_id,
                event: UserEventKind::Created { name, email },
            }),
            InnerEventKind::UserDeleted { user_id } => Event::User(UserEvent {
                id: self.id,
                event_date: self.date,
                user_id,
                event: UserEventKind::Deleted,
            }),
            InnerEventKind::UserLogin { user_id } => Event::User(UserEvent {
                id: self.id,
                event_date: self.date,
                user_id,
                event: UserEventKind::Login,
            }),
            InnerEventKind::UserLogout { user_id } => Event::User(UserEvent {
                id: self.id,
                event_date: self.date,
                user_id,
                event: UserEventKind::Logout,
            }),
        };
        Ok(e)
    }
}
