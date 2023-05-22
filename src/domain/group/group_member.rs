use crate::domain::{Email, MemberColor, UserName};
use crate::utils::date;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct GroupMember {
    pub id: Uuid,
    pub name: UserName,
    pub email: Email,
    pub group_id: Uuid,
    pub is_admin: bool,
    pub color: MemberColor,
    pub joined_at: DateTime<Utc>,
}

impl GroupMember {
    pub fn create(
        id: Uuid,
        name: UserName,
        email: Email,
        group_id: Uuid,
        is_admin: bool,
        color: MemberColor,
    ) -> Self {
        GroupMember {
            id,
            name,
            email,
            group_id,
            is_admin,
            color,
            joined_at: date::now(),
        }
    }

    pub fn update_color(&mut self, color: MemberColor) {
        self.color = color;
    }
}
