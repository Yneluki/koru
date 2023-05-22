use crate::application::store::{MemberRepository, MemberRepositoryError};
use crate::domain::GroupMember;
use crate::infrastructure::store::mem::mem_store::{
    InMemTx, InMemoryStore, InnerColor, InnerMember,
};
use async_trait::async_trait;
use itertools::Itertools;
use std::cell::RefCell;
use std::sync::atomic::Ordering::Relaxed;
use uuid::Uuid;

#[async_trait]
impl MemberRepository for InMemoryStore {
    type Tr = InMemTx;

    async fn save(
        &self,
        tx: &mut RefCell<InMemTx>,
        member: &GroupMember,
    ) -> Result<(), MemberRepositoryError> {
        if self.crash_members.load(Relaxed) {
            return Err(MemberRepositoryError::CorruptedData("Crashed store"));
        }
        let member = InnerMember {
            id: (member.id, member.group_id),
            is_admin: member.is_admin,
            color: InnerColor {
                red: member.color.red,
                green: member.color.green,
                blue: member.color.blue,
            },
            joined_at: member.joined_at,
        };
        tx.get_mut()
            .members
            .lock()
            .unwrap()
            .insert(member.id, member);
        Ok(())
    }

    async fn fetch_members(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<GroupMember>, MemberRepositoryError> {
        if self.crash_members.load(Relaxed) {
            return Err(MemberRepositoryError::CorruptedData("Crashed store"));
        }
        let r = self
            .members
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, member)| *group_id == member.id.1)
            .map(|(_, member)| member.clone())
            .collect_vec();
        let mut members = Vec::new();
        for member in r {
            let user = self
                .users
                .lock()
                .unwrap()
                .get(&member.id.0)
                .cloned()
                .unwrap();
            members.push(
                GroupMember::try_from(member, user)
                    .map_err(MemberRepositoryError::CorruptedData)?,
            );
        }
        Ok(members)
    }
}
