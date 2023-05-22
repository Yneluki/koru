mod settlement_description;
mod transaction;

pub use settlement_description::SettlementDescription;
pub use transaction::Transaction;

use crate::domain::errors::SettlementError;
use crate::domain::{Amount, Expense};
use crate::utils::date;
use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use float_cmp::{ApproxEq, F32Margin};
use itertools::Itertools;
use log::info;
use std::collections::HashMap;
use uuid::Uuid;

const MARGIN: f32 = 0.001;

#[derive(Debug)]
pub struct Settlement {
    pub id: Uuid,
    pub group_id: Uuid,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: DateTime<Utc>,
    pub transactions: Vec<Transaction>,
    pub expense_ids: Vec<Uuid>,
}

impl Settlement {
    pub fn create(
        group_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        expenses: &mut [Expense],
        users: &[Uuid],
    ) -> Result<Self, SettlementError> {
        let transactions = Self::compute_transactions(expenses, users)?;
        let mut expense_ids = Vec::new();
        for expense in expenses {
            expense.settle();
            expense_ids.push(expense.id);
        }
        let end_date = date::now();
        Ok(Self {
            id: Uuid::new_v4(),
            group_id,
            start_date,
            end_date,
            transactions,
            expense_ids,
        })
    }

    fn compute_transactions(
        expenses: &[Expense],
        users: &[Uuid],
    ) -> Result<Vec<Transaction>, SettlementError> {
        let deltas_by_user = Self::deltas_by_user(expenses, users);
        Self::settle(deltas_by_user).map_err(SettlementError::Unexpected)
    }

    fn deltas_by_user(expenses: &[Expense], users: &[Uuid]) -> HashMap<Uuid, f32> {
        let mut expenses_by_user = HashMap::new();

        // total expenses by user
        for expense in expenses {
            let amount = f32::from(expense.amount);
            expenses_by_user
                .entry(expense.member_id)
                .and_modify(|v| *v += amount)
                .or_insert(amount);
        }
        info!("expenses by user {:?}", expenses_by_user);

        let total = expenses_by_user
            .iter()
            .fold(0.0, |acc, entry| acc + entry.1);
        let avg = total / (users.len() as f32);
        info!("total: {}, avg: {} ", total, avg);

        // compute delta by user
        for user in users {
            expenses_by_user
                .entry(*user)
                .and_modify(|v| *v -= avg)
                .or_insert(-avg);
        }
        expenses_by_user
    }

    fn settle(deltas_by_user: HashMap<Uuid, f32>) -> Result<Vec<Transaction>, anyhow::Error> {
        let deltas = deltas_by_user.into_iter().collect_vec();
        if deltas.len() <= 1 {
            return Ok(Vec::new());
        }

        let margin = F32Margin::epsilon(F32Margin::default(), MARGIN);
        // validate the data, sum of deltas should be 0
        let total: f32 = deltas.iter().map(|e| e.1).sum();
        if total.approx_ne(0.0, margin) {
            return Err(anyhow!("Cannot settle, invalid deltas, sum should be 0"));
        }
        // sort the deltas in ascending order of expense
        let mut deltas = deltas
            .into_iter()
            .sorted_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .expect("expenses amount to be comparable f32")
            })
            .collect_vec();

        // if we need more than 10x the number of users something is wrong
        let iter_limit = (deltas.len() * 10) as i32;

        let mut res = Vec::new();
        let mut nb_iter = 0;
        let mut i = 0;
        let mut j = deltas.len() - 1;
        while i < j && nb_iter < iter_limit {
            let mut from = *deltas
                .get(i)
                .context("settlement computation failed: index should be in vec range")?;
            let mut to = *deltas
                .get(j)
                .context("settlement computation failed: index should be in vec range")?;
            let transfer = from.1.abs().min(to.1);

            res.push(Transaction {
                from: from.0,
                to: to.0,
                amount: Amount::try_from(transfer).map_err(|e| anyhow!(e))?,
            });

            from.1 += transfer;
            to.1 -= transfer;
            deltas[i] = (from.0, from.1);
            deltas[j] = (to.0, to.1);

            if from.1.abs().approx_eq(0.0, margin) {
                i += 1;
            }
            if to.1.abs().approx_eq(0.0, margin) {
                j -= 1;
            }
            nb_iter += 1;
        }
        if nb_iter == iter_limit {
            return Err(anyhow!("Failed to settle !"));
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::assert_err;
    use float_cmp::assert_approx_eq;

    #[test]
    fn it_should_return_an_empty_array_if_there_is_only_one_user() -> anyhow::Result<()> {
        let deltas = [(Uuid::new_v4(), 10.0 as f32)];
        let res = Settlement::settle(HashMap::from(deltas))?;
        assert_eq!(res.len(), 0);
        Ok(())
    }

    #[test]
    fn it_should_return_a_transaction_to_the_second_user() -> anyhow::Result<()> {
        let deltas = [
            (Uuid::new_v4(), -10.0 as f32),
            (Uuid::new_v4(), 10.0 as f32),
        ];
        let res = Settlement::settle(HashMap::from(deltas.clone()))?;
        assert_eq!(res.len(), 1);
        let transaction = res.get(0).unwrap();
        assert_eq!(transaction.from, deltas.get(0).unwrap().0);
        assert_eq!(transaction.to, deltas.get(1).unwrap().0);
        assert_approx_eq!(f32, f32::from(transaction.amount.clone()), 10.0);
        Ok(())
    }

    #[test]
    fn it_should_return_transactions_to_the_last_user() -> anyhow::Result<()> {
        let deltas = [
            (Uuid::new_v4(), -12.0 as f32),
            (Uuid::new_v4(), -10.0 as f32),
            (Uuid::new_v4(), 22.0 as f32),
        ];
        let res = Settlement::settle(HashMap::from(deltas.clone()))?;
        assert_eq!(res.len(), 2);
        let transaction_1 = res.get(0).unwrap();
        assert_eq!(transaction_1.from, deltas.get(0).unwrap().0);
        assert_eq!(transaction_1.to, deltas.get(2).unwrap().0);
        assert_approx_eq!(f32, f32::from(transaction_1.amount.clone()), 12.0);
        let transaction_2 = res.get(1).unwrap();
        assert_eq!(transaction_2.from, deltas.get(1).unwrap().0);
        assert_eq!(transaction_2.to, deltas.get(2).unwrap().0);
        assert_approx_eq!(f32, f32::from(transaction_2.amount.clone()), 10.0);
        Ok(())
    }

    #[test]
    fn it_should_return_transactions_to_the_last_users() -> anyhow::Result<()> {
        let deltas = [
            (Uuid::new_v4(), -12.0 as f32),
            (Uuid::new_v4(), 5.0 as f32),
            (Uuid::new_v4(), 7.0 as f32),
        ];
        let res = Settlement::settle(HashMap::from(deltas.clone()))?;
        assert_eq!(res.len(), 2);
        let transaction_1 = res.get(0).unwrap();
        assert_eq!(transaction_1.from, deltas.get(0).unwrap().0);
        assert_eq!(transaction_1.to, deltas.get(2).unwrap().0);
        assert_approx_eq!(f32, f32::from(transaction_1.amount.clone()), 7.0);
        let transaction_2 = res.get(1).unwrap();
        assert_eq!(transaction_2.from, deltas.get(0).unwrap().0);
        assert_eq!(transaction_2.to, deltas.get(1).unwrap().0);
        assert_approx_eq!(f32, f32::from(transaction_2.amount.clone()), 5.0);
        Ok(())
    }

    #[test]
    fn it_should_return_transactions_to_the_users() -> anyhow::Result<()> {
        let deltas = [
            (Uuid::new_v4(), -52.5 as f32),
            (Uuid::new_v4(), -42.5 as f32),
            (Uuid::new_v4(), 37.5 as f32),
            (Uuid::new_v4(), 57.5 as f32),
        ];
        let res = Settlement::settle(HashMap::from(deltas.clone()))?;
        assert_eq!(res.len(), 3);
        let transaction_1 = res.get(0).unwrap();
        assert_eq!(transaction_1.from, deltas.get(0).unwrap().0);
        assert_eq!(transaction_1.to, deltas.get(3).unwrap().0);
        assert_approx_eq!(f32, f32::from(transaction_1.amount.clone()), 52.5);
        let transaction_2 = res.get(1).unwrap();
        assert_eq!(transaction_2.from, deltas.get(1).unwrap().0);
        assert_eq!(transaction_2.to, deltas.get(3).unwrap().0);
        assert_approx_eq!(f32, f32::from(transaction_2.amount.clone()), 5.0);
        let transaction_3 = res.get(2).unwrap();
        assert_eq!(transaction_3.from, deltas.get(1).unwrap().0);
        assert_eq!(transaction_3.to, deltas.get(2).unwrap().0);
        assert_approx_eq!(f32, f32::from(transaction_3.amount.clone()), 37.5);
        Ok(())
    }

    #[test]
    fn it_should_fail_if_deltas_are_invalid() {
        let deltas = [
            (Uuid::new_v4(), -52.5 as f32),
            (Uuid::new_v4(), 37.5 as f32),
            (Uuid::new_v4(), 57.5 as f32),
        ];
        let res = Settlement::settle(HashMap::from(deltas.clone()));
        assert_err!(res);
    }

    #[test]
    fn it_should_return_transactions_to_the_users_with_rounding() -> anyhow::Result<()> {
        let deltas = [
            (Uuid::new_v4(), (20.0 - 25.0 / 3.0) as f32),
            (Uuid::new_v4(), (5.0 - 25.0 / 3.0) as f32),
            (Uuid::new_v4(), (-25.0 / 3.0) as f32),
        ];
        let res = Settlement::settle(HashMap::from(deltas.clone()))?;
        assert_eq!(res.len(), 2);
        let transaction_1 = res.get(0).unwrap();
        assert_eq!(transaction_1.from, deltas.get(2).unwrap().0);
        assert_eq!(transaction_1.to, deltas.get(0).unwrap().0);
        assert_approx_eq!(
            f32,
            f32::from(transaction_1.amount.clone()),
            (25.0 / 3.0) as f32
        );
        let transaction_2 = res.get(1).unwrap();
        assert_eq!(transaction_2.from, deltas.get(1).unwrap().0);
        assert_eq!(transaction_2.to, deltas.get(0).unwrap().0);
        assert_approx_eq!(
            f32,
            f32::from(transaction_2.amount.clone()),
            (-5.0 + 25.0 / 3.0) as f32
        );
        Ok(())
    }
}
