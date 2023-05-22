use crate::application::store::{SettlementRepository, SettlementRepositoryError};
use crate::domain::{Settlement, SettlementDescription};
use crate::infrastructure::store::mem::mem_store::{
    InMemTx, InMemoryStore, InnerSettlement, InnerTransaction,
};
use async_trait::async_trait;
use std::cell::RefCell;
use std::sync::atomic::Ordering::Relaxed;
use uuid::Uuid;

#[async_trait]
impl SettlementRepository for InMemoryStore {
    type Tr = InMemTx;

    async fn save(
        &self,
        tx: &mut RefCell<Self::Tr>,
        settlement: &Settlement,
    ) -> Result<(), SettlementRepositoryError> {
        if self.crash_settlements.load(Relaxed) {
            return Err(SettlementRepositoryError::CorruptedData("Crashed store"));
        }
        let transactions = settlement
            .transactions
            .iter()
            .map(|tr| InnerTransaction {
                from: tr.from,
                to: tr.to,
                amount: f32::from(tr.amount),
            })
            .collect();
        let settlement = InnerSettlement {
            id: settlement.id,
            group_id: settlement.group_id,
            start_date: settlement.start_date,
            end_date: settlement.end_date,
            transactions,
            expenses: settlement.expense_ids.clone(),
        };
        tx.get_mut()
            .settlements
            .lock()
            .unwrap()
            .insert(settlement.id, settlement);
        Ok(())
    }

    async fn get_settlements(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<Settlement>, SettlementRepositoryError> {
        if self.crash_settlements.load(Relaxed) {
            return Err(SettlementRepositoryError::CorruptedData("Crashed store"));
        }
        let ids = self
            .groups
            .lock()
            .unwrap()
            .get(group_id)
            .map(|g| g.settlements.clone())
            .map_or(Vec::new(), |ids| ids);
        let mut stls = Vec::new();
        for id in ids {
            if let Some(stl) = self.find(&id).await.unwrap() {
                stls.push(stl);
            }
        }
        Ok(stls)
    }

    async fn get_settlement_description(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Option<SettlementDescription>, SettlementRepositoryError> {
        if self.crash_settlements.load(Relaxed) {
            return Err(SettlementRepositoryError::CorruptedData("Crashed store"));
        }
        self.settlements
            .lock()
            .unwrap()
            .get(settlement_id)
            .map(|g| SettlementDescription::try_from(g.clone()))
            .map_or(Ok(None), |g| g.map(Some))
            .map_err(SettlementRepositoryError::CorruptedData)
    }

    async fn find(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Option<Settlement>, SettlementRepositoryError> {
        if self.crash_settlements.load(Relaxed) {
            return Err(SettlementRepositoryError::CorruptedData("Crashed store"));
        }
        self.settlements
            .lock()
            .unwrap()
            .get(settlement_id)
            .map(|g| Settlement::try_from(g.clone()))
            .map_or(Ok(None), |g| g.map(Some))
            .map_err(SettlementRepositoryError::CorruptedData)
    }

    async fn exists(&self, settlement_id: &Uuid) -> Result<bool, SettlementRepositoryError> {
        if self.crash_settlements.load(Relaxed) {
            return Err(SettlementRepositoryError::CorruptedData("Crashed store"));
        }
        self.settlements
            .lock()
            .unwrap()
            .get(settlement_id)
            .map_or(Ok(false), |_| Ok(true))
    }

    async fn get_expenses(
        &self,
        settlement_id: &Uuid,
    ) -> Result<Vec<Uuid>, SettlementRepositoryError> {
        if self.crash_settlements.load(Relaxed) {
            return Err(SettlementRepositoryError::CorruptedData("Crashed store"));
        }
        self.settlements
            .lock()
            .unwrap()
            .get(settlement_id)
            .map(|s| s.expenses.clone())
            .map_or(Ok(Vec::new()), Ok)
    }
}
