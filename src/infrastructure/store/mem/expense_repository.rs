use crate::application::store::{ExpenseRepository, ExpenseRepositoryError};
use crate::domain::Expense;
use crate::infrastructure::store::mem::mem_store::{InMemTx, InMemoryStore, InnerExpense};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use std::cell::RefCell;
use std::sync::atomic::Ordering::Relaxed;
use uuid::Uuid;

#[async_trait]
impl ExpenseRepository for InMemoryStore {
    type Tr = InMemTx;

    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        expense: &Expense,
    ) -> Result<(), ExpenseRepositoryError> {
        if self.crash_expenses.load(Relaxed) {
            return Err(ExpenseRepositoryError::CorruptedData("Crashed store"));
        }
        let expense = InnerExpense {
            id: expense.id,
            group_id: expense.group_id,
            title: String::from(expense.title.clone()),
            amount: f32::from(expense.amount),
            member_id: expense.member_id,
            created_at: expense.created_at,
            modified_at: expense.modified_at,
            settled: expense.settled,
        };
        tx.get_mut()
            .expenses
            .lock()
            .unwrap()
            .insert(expense.id, expense);
        Ok(())
    }

    async fn delete(
        &self,
        tx: &mut RefCell<Self::Tr>,
        expense_id: &Uuid,
    ) -> Result<(), ExpenseRepositoryError> {
        if self.crash_expenses.load(Relaxed) {
            return Err(ExpenseRepositoryError::CorruptedData("Crashed store"));
        }
        tx.get_mut()
            .deleted_expenses
            .lock()
            .unwrap()
            .insert(*expense_id);
        Ok(())
    }

    async fn find(&self, expense_id: &Uuid) -> Result<Option<Expense>, ExpenseRepositoryError> {
        if self.crash_expenses.load(Relaxed) {
            return Err(ExpenseRepositoryError::CorruptedData("Crashed store"));
        }
        self.expenses
            .lock()
            .unwrap()
            .get(expense_id)
            .map(|g| Expense::try_from(g.clone()))
            .map_or(Ok(None), |g| g.map(Some))
            .map_err(ExpenseRepositoryError::CorruptedData)
    }

    async fn get_expenses(
        &self,
        group_id: &Uuid,
        start_date: Option<&DateTime<Utc>>,
        end_date: Option<&DateTime<Utc>>,
    ) -> Result<Vec<Expense>, ExpenseRepositoryError> {
        if self.crash_expenses.load(Relaxed) {
            return Err(ExpenseRepositoryError::CorruptedData("Crashed store"));
        }
        let r = self
            .expenses
            .lock()
            .unwrap()
            .values()
            .filter(|e| e.group_id == *group_id)
            .filter(|e| match start_date {
                None => true,
                Some(start_date) => e.created_at > *start_date,
            })
            .filter(|e| match end_date {
                None => true,
                Some(end_date) => e.created_at <= *end_date,
            })
            .cloned()
            .collect_vec();
        let mut expenses = Vec::new();
        for expense in r {
            expenses
                .push(Expense::try_from(expense).map_err(ExpenseRepositoryError::CorruptedData)?);
        }
        Ok(expenses)
    }

    async fn get_expenses_by_id(
        &self,
        expense_ids: &[Uuid],
    ) -> Result<Vec<Expense>, ExpenseRepositoryError> {
        if self.crash_expenses.load(Relaxed) {
            return Err(ExpenseRepositoryError::CorruptedData("Crashed store"));
        }
        let r = self
            .expenses
            .lock()
            .unwrap()
            .iter()
            .filter(|(id, _)| expense_ids.contains(*id))
            .map(|(_, expense)| expense.clone())
            .collect_vec();
        let mut expenses = Vec::new();
        for expense in r {
            expenses
                .push(Expense::try_from(expense).map_err(ExpenseRepositoryError::CorruptedData)?);
        }
        Ok(expenses)
    }

    async fn get_unsettled_expenses(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<Expense>, ExpenseRepositoryError> {
        if self.crash_expenses.load(Relaxed) {
            return Err(ExpenseRepositoryError::CorruptedData("Crashed store"));
        }
        let r = self
            .expenses
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, expense)| expense.group_id == *group_id && !expense.settled)
            .map(|(_, expense)| expense.clone())
            .collect_vec();
        let mut expenses = Vec::new();
        for expense in r {
            expenses
                .push(Expense::try_from(expense).map_err(ExpenseRepositoryError::CorruptedData)?);
        }
        Ok(expenses)
    }
}
