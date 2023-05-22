use crate::domain::{Expense, Group, GroupMember, MemberColor, Transaction, User, UserRole};
use chrono::{DateTime, Utc};
use itertools::Itertools;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(serde::Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SettlementDto {
    pub id: Uuid,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: DateTime<Utc>,
    pub transactions: Vec<TransactionDto>,
}

#[derive(serde::Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TransactionDto {
    pub from: MemberDto,
    pub to: MemberDto,
    pub amount: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ColorDto {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(serde::Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct UserDto {
    pub id: Uuid,
    pub name: String,
}

#[derive(serde::Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct DetailedUserDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(serde::Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct MemberDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub is_admin: bool,
    pub color: ColorDto,
    pub joined_at: DateTime<Utc>,
}

#[derive(serde::Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ExpenseDto {
    pub id: Uuid,
    pub description: String,
    pub amount: f32,
    pub user: MemberDto,
    pub date: DateTime<Utc>,
}

#[derive(serde::Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct GroupDto {
    pub id: Uuid,
    pub name: String,
    pub members: Vec<MemberDto>,
}

#[derive(serde::Serialize, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct DetailedGroupDto {
    pub id: Uuid,
    pub name: String,
    pub members: Vec<MemberDto>,
    pub expenses: Vec<ExpenseDto>,
}

impl From<MemberColor> for ColorDto {
    fn from(n: MemberColor) -> Self {
        ColorDto {
            red: n.red,
            green: n.green,
            blue: n.blue,
        }
    }
}

impl From<ColorDto> for MemberColor {
    fn from(n: ColorDto) -> Self {
        MemberColor {
            red: n.red,
            green: n.green,
            blue: n.blue,
        }
    }
}

impl From<User> for UserDto {
    fn from(value: User) -> Self {
        UserDto {
            id: value.id,
            name: String::from(value.name),
        }
    }
}

impl From<User> for DetailedUserDto {
    fn from(value: User) -> Self {
        DetailedUserDto {
            id: value.id,
            name: String::from(value.name),
            email: String::from(value.email),
            role: match value.role {
                UserRole::Administrator => "Administrator".to_string(),
                UserRole::User => "User".to_string(),
            },
            created_at: value.created_at,
        }
    }
}

impl MemberDto {
    pub fn from(member: GroupMember) -> Self {
        MemberDto {
            id: member.id,
            name: String::from(member.name),
            email: String::from(member.email),
            is_admin: member.is_admin,
            color: ColorDto::from(member.color),
            joined_at: member.joined_at,
        }
    }
}

impl GroupDto {
    pub fn from(grp: Group) -> Self {
        GroupDto {
            id: grp.id,
            name: String::from(grp.name),
            members: grp.members.into_iter().map(MemberDto::from).collect(),
        }
    }
}

impl DetailedGroupDto {
    pub fn from(grp: Group, expenses: Vec<Expense>) -> Self {
        DetailedGroupDto {
            id: grp.id,
            name: String::from(grp.name),
            members: grp
                .members
                .iter()
                .map(|m| MemberDto::from(m.clone()))
                .collect(),
            expenses: expenses
                .into_iter()
                .map(|e| {
                    let member = grp
                        .members
                        .iter()
                        .find(|m| m.id == e.member_id)
                        .cloned()
                        .unwrap_or_default();
                    ExpenseDto::from(e, member)
                })
                .sorted_by(|a, b| {
                    b.date
                        .partial_cmp(&a.date)
                        .expect("expenses dates to be comparable")
                })
                .collect(),
        }
    }
}

impl ExpenseDto {
    pub fn from(e: Expense, m: GroupMember) -> Self {
        ExpenseDto {
            id: e.id,
            description: String::from(e.title),
            amount: f32::from(e.amount),
            user: MemberDto::from(m),
            date: e.created_at,
        }
    }
}

impl TransactionDto {
    pub fn from(transaction: Transaction, members: &[GroupMember]) -> Self {
        TransactionDto {
            from: MemberDto::from(
                members
                    .iter()
                    .find(|m| m.id == transaction.from)
                    .cloned()
                    .unwrap_or_default(),
            ),
            to: MemberDto::from(
                members
                    .iter()
                    .find(|m| m.id == transaction.to)
                    .cloned()
                    .unwrap_or_default(),
            ),
            amount: f32::from(transaction.amount),
        }
    }
    pub fn from_vec(transactions: Vec<Transaction>, members: &[GroupMember]) -> Vec<Self> {
        transactions
            .into_iter()
            .map(|tr| Self::from(tr, members))
            .sorted_by(|a, b| {
                b.amount
                    .partial_cmp(&a.amount)
                    .expect("expenses amount to be comparable f32")
            })
            .collect()
    }
}
