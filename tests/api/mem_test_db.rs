use crate::test_app::{
    ExpenseDto, GroupDto, MemberDto, SettlementDto, TransactionDto, UserDevice, UserDto,
};
use itertools::Itertools;
use koru::infrastructure::store::mem::mem_store::InnerRole::ADMINISTRATOR;
use koru::infrastructure::store::InMemoryStore;
use std::sync::Arc;
use uuid::Uuid;

pub struct MemTestDb {
    pub store: Arc<InMemoryStore>,
}

impl MemTestDb {
    pub async fn build() -> Self {
        let store = Arc::new(InMemoryStore::new());
        Self { store }
    }

    pub async fn get_member_by_id(&self, id: Uuid) -> Option<MemberDto> {
        self.store
            .members
            .lock()
            .unwrap()
            .iter()
            .find(|((user_id, _), _)| id == *user_id)
            .map(|(_, member)| MemberDto {
                group_id: member.id.1,
                user_id: member.id.0,
                color: String::from(member.color.clone()),
            })
    }
    pub async fn get_expense(&self) -> Option<ExpenseDto> {
        self.store
            .expenses
            .lock()
            .unwrap()
            .iter()
            .next()
            .map(|(_, e)| ExpenseDto {
                id: e.id,
                group_id: e.group_id,
                member_id: e.member_id,
                description: e.title.clone(),
                amount: e.amount,
            })
    }
    pub async fn get_expense_by_id(&self, id: Uuid) -> Option<ExpenseDto> {
        self.store
            .expenses
            .lock()
            .unwrap()
            .get(&id)
            .map(|e| ExpenseDto {
                id: e.id,
                group_id: e.group_id,
                member_id: e.member_id,
                description: e.title.clone(),
                amount: e.amount,
            })
    }
    pub async fn get_event_type(&self) -> Option<String> {
        let evts = self.store.events.lock().unwrap();
        let length = evts.len();
        if length > 0 {
            Some(evts[length - 1].clone().event.name().to_string())
        } else {
            None
        }
    }
    pub async fn get_group(&self) -> Option<GroupDto> {
        self.store
            .groups
            .lock()
            .unwrap()
            .iter()
            .next()
            .map(|(_, group)| GroupDto {
                id: group.id,
                name: group.name.clone(),
                admin_id: group.admin_id,
            })
    }
    pub async fn get_group_by_id(&self, id: Uuid) -> Option<GroupDto> {
        self.store
            .groups
            .lock()
            .unwrap()
            .get(&id)
            .map(|group| GroupDto {
                id: group.id,
                name: group.name.clone(),
                admin_id: group.admin_id,
            })
    }
    pub async fn get_user_id_by_email(&self, email: String) -> Uuid {
        self.store
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|(_, u)| u.email == email)
            .map(|(id, _)| *id)
            .unwrap()
    }
    pub async fn get_user(&self) -> UserDto {
        let password = self
            .store
            .user_credentials
            .lock()
            .unwrap()
            .iter()
            .next()
            .map(|(_, u)| u.clone())
            .unwrap();
        self.store
            .users
            .lock()
            .unwrap()
            .iter()
            .next()
            .map(|(_, u)| UserDto {
                name: u.name.clone(),
                email: u.email.clone(),
                password,
            })
            .unwrap()
    }
    pub async fn get_device(&self) -> Option<UserDevice> {
        self.store
            .user_devices
            .lock()
            .unwrap()
            .iter()
            .next()
            .map(|(id, device)| UserDevice {
                user_id: *id,
                device: Some(device.clone()),
            })
    }
    pub async fn get_settlement(&self) -> Option<SettlementDto> {
        self.store
            .settlements
            .lock()
            .unwrap()
            .iter()
            .next()
            .map(|(_, stl)| SettlementDto {
                id: stl.id,
                group_id: stl.group_id,
            })
    }
    pub async fn get_expenses_status(&self, ids: &[Uuid]) -> Vec<(Uuid, bool)> {
        self.store
            .expenses
            .lock()
            .unwrap()
            .iter()
            .filter(|(id, _)| ids.contains(*id))
            .map(|(_, e)| (e.id, e.settled))
            .collect_vec()
    }
    pub async fn get_transactions(&self) -> Vec<TransactionDto> {
        self.store
            .settlements
            .lock()
            .unwrap()
            .iter()
            .next()
            .map(|(_, stl)| {
                stl.transactions
                    .iter()
                    .map(|tr| TransactionDto {
                        settlement_id: stl.id,
                        from_user_id: tr.from,
                        to_user_id: tr.to,
                        amount: tr.amount,
                    })
                    .sorted_by(|a, b| {
                        a.amount
                            .partial_cmp(&b.amount)
                            .expect("expenses amount to be comparable f32")
                    })
                    .collect_vec()
            })
            .unwrap_or(Vec::new())
    }
    pub async fn settled_expenses(&self) -> Vec<(Uuid, Uuid)> {
        self.store
            .settlements
            .lock()
            .unwrap()
            .iter()
            .next()
            .map(|(_, stl)| {
                stl.expenses
                    .iter()
                    .map(|e_id| (stl.id, *e_id))
                    .collect_vec()
            })
            .unwrap()
    }

    pub async fn delete_group(&self, id: Uuid) {
        self.store.groups.lock().unwrap().remove(&id);
    }

    pub async fn delete_user(&self, id: Uuid) {
        self.store.users.lock().unwrap().remove(&id);
    }

    pub async fn break_group_db(&self) {
        self.store.crash_groups();
    }

    pub async fn break_user_db(&self) {
        self.store.crash_users();
    }

    pub async fn break_credentials_db(&self) {
        self.store.crash_credentials();
    }

    pub async fn make_admin(&self, id: &Uuid) {
        self.store
            .users
            .lock()
            .unwrap()
            .get_mut(id)
            .map(|u| u.role = ADMINISTRATOR);
    }
}
