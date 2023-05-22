use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct SettlementDescription {
    pub id: Uuid,
    pub group_id: Uuid,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: DateTime<Utc>,
}
