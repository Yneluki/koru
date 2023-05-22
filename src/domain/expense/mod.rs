mod expense_title;

pub use expense_title::ExpenseTitle;

use crate::domain::errors::{CreateExpenseError, UpdateExpenseError};
use crate::domain::Amount;
use crate::utils::date;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct Expense {
    pub id: Uuid,
    pub group_id: Uuid,
    pub member_id: Uuid,
    pub title: ExpenseTitle,
    pub amount: Amount,
    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
    pub settled: bool,
}

impl Expense {
    pub fn create(
        title: String,
        amount: f32,
        user_id: Uuid,
        group_id: Uuid,
    ) -> Result<Self, CreateExpenseError> {
        Ok(Self {
            id: Uuid::new_v4(),
            group_id,
            member_id: user_id,
            title: ExpenseTitle::try_from(title).map_err(CreateExpenseError::Validation)?,
            amount: Amount::try_from(amount).map_err(CreateExpenseError::Validation)?,
            created_at: date::now(),
            modified_at: None,
            settled: false,
        })
    }

    pub fn settle(&mut self) {
        self.settled = true;
    }

    pub fn update(&mut self, title: String, amount: f32) -> Result<(), UpdateExpenseError> {
        self.title = ExpenseTitle::try_from(title).map_err(UpdateExpenseError::Validation)?;
        self.amount = Amount::try_from(amount).map_err(UpdateExpenseError::Validation)?;
        Ok(())
    }
}
