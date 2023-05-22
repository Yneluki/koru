use crate::domain::Amount;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub from: Uuid,
    pub to: Uuid,
    pub amount: Amount,
}
