use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub timestamp: OffsetDateTime,
    pub labels: HashMap<String, String>,
    pub reason: String,
    pub action: AdminActionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdminActionKind {
    AccountDeactivated,
    AccountReactivated,
    MaxDebtUpdated { value: u64 },
}

impl AdminAction {
    pub fn new_account_deactivated(
        account_id: Uuid,
        labels: HashMap<String, String>,
        reason: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            timestamp: OffsetDateTime::now_utc(),
            labels,
            reason,
            action: AdminActionKind::AccountDeactivated,
        }
    }

    pub fn new_account_reactivated(
        account_id: Uuid,
        labels: HashMap<String, String>,
        reason: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            timestamp: OffsetDateTime::now_utc(),
            labels,
            reason,
            action: AdminActionKind::AccountReactivated,
        }
    }

    pub fn new_max_debt_updated(
        account_id: Uuid,
        value: u64,
        labels: HashMap<String, String>,
        reason: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            timestamp: OffsetDateTime::now_utc(),
            labels,
            reason,
            action: AdminActionKind::MaxDebtUpdated { value },
        }
    }
}
