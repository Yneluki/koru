use crate::application::store::{GroupRepository, GroupRepositoryError, MemberRepository};
use crate::domain::Group;
use crate::infrastructure::store::mem::mem_store::{InMemTx, InMemoryStore, InnerGroup};
use async_trait::async_trait;
use std::cell::RefCell;
use std::sync::atomic::Ordering::Relaxed;
use uuid::Uuid;

#[async_trait]
impl GroupRepository for InMemoryStore {
    type Tr = InMemTx;

    async fn save(
        &self,
        tx: &mut RefCell<InMemTx>,
        group: &Group,
    ) -> Result<(), GroupRepositoryError> {
        if self.crash_groups.load(Relaxed) {
            return Err(GroupRepositoryError::CorruptedData("Crashed store"));
        }
        let group = InnerGroup {
            id: group.id,
            name: group.name.clone().into(),
            admin_id: group.admin_id,
            created_at: group.created_at,
            member_ids: group.members.iter().map(|m| m.id).collect(),
            expenses: group.expense_ids.clone(),
            settlements: group.settlement_ids.clone(),
        };
        tx.get_mut().groups.lock().unwrap().insert(group.id, group);
        Ok(())
    }

    async fn delete(
        &self,
        tx: &mut RefCell<Self::Tr>,
        group_id: &Uuid,
    ) -> Result<(), GroupRepositoryError> {
        if self.crash_groups.load(Relaxed) {
            return Err(GroupRepositoryError::CorruptedData("Crashed store"));
        }
        tx.get_mut()
            .deleted_groups
            .lock()
            .unwrap()
            .insert(*group_id);
        Ok(())
    }

    async fn find(&self, group_id: &Uuid) -> Result<Option<Group>, GroupRepositoryError> {
        if self.crash_groups.load(Relaxed) {
            return Err(GroupRepositoryError::CorruptedData("Crashed store"));
        }
        let group = self.groups.lock().unwrap().get(group_id).cloned();
        match group {
            Some(group) => {
                let members = self
                    .fetch_members(group_id)
                    .await
                    .map_err(|_| GroupRepositoryError::CorruptedData("No members"))?;
                group
                    .build_group(members)
                    .map_err(GroupRepositoryError::CorruptedData)
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    async fn get_user_groups(&self, user_id: &Uuid) -> Result<Vec<Group>, GroupRepositoryError> {
        if self.crash_groups.load(Relaxed) {
            return Err(GroupRepositoryError::CorruptedData("Crashed store"));
        }
        let groups: Vec<InnerGroup> = self
            .groups
            .lock()
            .unwrap()
            .values()
            .filter(|g| g.admin_id == *user_id || g.member_ids.contains(user_id))
            .cloned()
            .collect();
        let mut res = Vec::new();
        for group in groups {
            let members = self
                .fetch_members(&group.id)
                .await
                .map_err(|_| GroupRepositoryError::CorruptedData("No members"))?;
            let g = group
                .build_group(members)
                .map_err(GroupRepositoryError::CorruptedData)?;
            res.push(g);
        }
        Ok(res)
    }

    async fn fetch_all_groups(&self) -> Result<Vec<Group>, GroupRepositoryError> {
        if self.crash_groups.load(Relaxed) {
            return Err(GroupRepositoryError::CorruptedData("Crashed store"));
        }
        let groups: Vec<InnerGroup> = self.groups.lock().unwrap().values().cloned().collect();
        let mut res = Vec::new();
        for group in groups {
            let members = self
                .fetch_members(&group.id)
                .await
                .map_err(|_| GroupRepositoryError::CorruptedData("No members"))?;
            let g = group
                .build_group(members)
                .map_err(GroupRepositoryError::CorruptedData)?;
            res.push(g);
        }
        Ok(res)
    }
}
