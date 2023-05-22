mod group_member;
mod group_name;
mod member_color;
mod token_generator;

pub use group_member::GroupMember;
pub use group_name::GroupName;
pub use member_color::MemberColor;
use std::sync::Arc;
pub use token_generator::TokenGenerator;

use crate::domain::errors::{
    ChangeMemberColorError, CreateExpenseError, CreateGroupError, DeleteExpenseError,
    DeleteGroupError, GenerateGroupTokenError, JoinGroupError, SettlementError, UpdateExpenseError,
};
use crate::domain::{
    Email, Expense, GroupEvent, GroupEventKind, Settlement, SettlementDescription, UserName,
};
use crate::utils::date;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct Group {
    pub id: Uuid,
    pub name: GroupName,
    pub admin_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub members: Vec<GroupMember>,
    pub expense_ids: Vec<Uuid>,
    pub settlement_ids: Vec<Uuid>,
    pub events: Vec<GroupEvent>,
}

impl Group {
    pub fn create(
        name: String,
        admin_id: Uuid,
        admin_name: UserName,
        admin_email: Email,
        admin_color: MemberColor,
    ) -> Result<Self, CreateGroupError> {
        let id = Uuid::new_v4();
        let admin = GroupMember::create(admin_id, admin_name, admin_email, id, true, admin_color);
        Ok(Group {
            id,
            name: GroupName::try_from(name.clone()).map_err(CreateGroupError::Validation)?,
            admin_id,
            created_at: date::now(),
            members: vec![admin.clone()],
            expense_ids: vec![],
            settlement_ids: vec![],
            events: vec![GroupEvent::new(
                id,
                admin_id,
                GroupEventKind::GroupCreated {
                    name,
                    color: admin.color,
                },
            )],
        })
    }

    pub fn add_expense(
        &mut self,
        title: String,
        amount: f32,
        user_id: Uuid,
    ) -> Result<Expense, CreateExpenseError> {
        if !self.is_member(&user_id) {
            return Err(CreateExpenseError::Unauthorized());
        }
        let expense = Expense::create(title, amount, user_id, self.id)?;
        self.expense_ids.push(expense.id);
        self.events.push(GroupEvent::new(
            self.id,
            expense.member_id,
            GroupEventKind::ExpenseCreated {
                id: expense.id,
                description: String::from(expense.title.clone()),
                amount: f32::from(expense.amount),
                date: expense.created_at,
            },
        ));
        Ok(expense)
    }

    pub fn update_expense(
        &mut self,
        expense_id: Uuid,
        title: String,
        amount: f32,
        user_id: Uuid,
        expenses: Vec<Expense>,
    ) -> Result<Expense, UpdateExpenseError> {
        if !self.is_member(&user_id) {
            return Err(UpdateExpenseError::Unauthorized("User is not a member"));
        }
        let expense = expenses.into_iter().find(|e| e.id == expense_id);
        match expense {
            Some(mut expense) => {
                if !self.is_admin(&user_id) && expense.member_id != user_id {
                    return Err(UpdateExpenseError::Unauthorized(
                        "User is not admin or expense owner",
                    ));
                }
                let previous_description = expense.title.clone();
                let previous_amount = expense.amount;
                expense.update(title, amount)?;
                self.events.push(GroupEvent::new(
                    self.id,
                    user_id,
                    GroupEventKind::ExpenseModified {
                        id: expense.id,
                        previous_description: String::from(previous_description),
                        new_description: String::from(expense.title.clone()),
                        previous_amount: f32::from(previous_amount),
                        new_amount: f32::from(expense.amount),
                    },
                ));
                Ok(expense)
            }
            None => Err(UpdateExpenseError::NotFound("Expense not found.")),
        }
    }

    pub fn delete_expense(
        &mut self,
        expense_id: Uuid,
        user_id: Uuid,
        expenses: Vec<Expense>,
    ) -> Result<Expense, DeleteExpenseError> {
        if !self.is_member(&user_id) {
            return Err(DeleteExpenseError::Unauthorized("User is not a member"));
        }
        let expense = expenses.into_iter().find(|e| e.id == expense_id);
        match expense {
            Some(expense) => {
                if !self.is_admin(&user_id) && expense.member_id != user_id {
                    return Err(DeleteExpenseError::Unauthorized(
                        "User is not admin or expense owner",
                    ));
                }
                if let Some(index) = self.expense_ids.iter().position(|e| e == &expense.id) {
                    self.expense_ids.remove(index);
                }
                self.events.push(GroupEvent::new(
                    self.id,
                    user_id,
                    GroupEventKind::ExpenseDeleted { id: expense.id },
                ));
                Ok(expense)
            }
            None => Err(DeleteExpenseError::NotFound("Expense not found.")),
        }
    }

    pub fn add_member(
        &mut self,
        user_id: Uuid,
        name: UserName,
        email: Email,
        color: MemberColor,
    ) -> Result<GroupMember, JoinGroupError> {
        if self.is_member(&user_id) {
            return Err(JoinGroupError::Conflict());
        }
        let member = GroupMember::create(user_id, name, email, self.id, false, color);
        self.members.push(member.clone());
        self.events.push(GroupEvent::new(
            self.id,
            member.id,
            GroupEventKind::MemberJoined {
                color: member.color.clone(),
            },
        ));
        Ok(member)
    }

    pub fn update_member(
        &mut self,
        user_id: Uuid,
        color: MemberColor,
    ) -> Result<GroupMember, ChangeMemberColorError> {
        match self.members.iter().position(|m| m.id == user_id) {
            Some(index) => {
                let mut updated = self.members[index].clone();
                let prev_color = updated.color.clone();
                updated.update_color(color);
                self.members[index] = updated.clone();
                self.events.push(GroupEvent::new(
                    self.id,
                    updated.id,
                    GroupEventKind::MemberColorChanged {
                        previous_color: prev_color,
                        new_color: updated.color.clone(),
                    },
                ));
                Ok(updated)
            }
            None => Err(ChangeMemberColorError::Unauthorized(
                "User is not a member.",
            )),
        }
    }

    pub fn is_member(&self, user_id: &Uuid) -> bool {
        self.is_admin(user_id) || self.members.iter().any(|m| m.id == *user_id)
    }

    pub fn is_admin(&self, user_id: &Uuid) -> bool {
        user_id == &self.admin_id
    }

    pub fn admin(&self) -> &GroupMember {
        self.members
            .iter()
            .find(|m| m.is_admin)
            .expect("Group should have an admin")
    }

    pub fn settle(
        &mut self,
        expenses: &mut [Expense],
        last_settlement: Option<SettlementDescription>,
        user: Uuid,
    ) -> Result<Settlement, SettlementError> {
        if !self.is_admin(&user) {
            return Err(SettlementError::Unauthorized("User is not group admin."));
        }
        let member_ids: Vec<Uuid> = self.members.iter().map(|m| m.id).collect();
        let settlement = Settlement::create(
            self.id,
            last_settlement.map(|d| d.end_date),
            expenses,
            &member_ids,
        )?;
        self.settlement_ids.push(settlement.id);
        self.expense_ids.clear();
        self.events.push(GroupEvent::new(
            self.id,
            user,
            GroupEventKind::Settled {
                id: settlement.id,
                start_date: settlement.start_date,
                end_date: settlement.end_date,
                transactions: settlement.transactions.to_vec(),
            },
        ));
        Ok(settlement)
    }

    pub fn delete(&mut self, user: &Uuid) -> Result<(), DeleteGroupError> {
        if !self.is_admin(user) {
            return Err(DeleteGroupError::Unauthorized());
        }
        self.events.push(GroupEvent::new(
            self.id,
            *user,
            GroupEventKind::GroupDeleted,
        ));
        Ok(())
    }

    pub async fn generate_join_token<'a>(
        &'a self,
        user_id: &'a Uuid,
        token_generator: Arc<dyn TokenGenerator>,
    ) -> Result<String, GenerateGroupTokenError> {
        if self.is_admin(user_id) {
            token_generator.generate(&self.id).await
        } else {
            Err(GenerateGroupTokenError::Unauthorized())
        }
    }
}
