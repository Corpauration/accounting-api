use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminAction {
    pub id: Uuid,
    pub account_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub timestamp: OffsetDateTime,
    pub labels: HashMap<String, String>,
    pub reason: String,
    pub action: AdminActionKind,
    pub actor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
            actor: None,
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
            actor: None,
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
            actor: None,
        }
    }
}

/// Body for the deactivate / reactivate shortcut endpoints.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ShortcutRequest {
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

/// Body for `PUT /accounts/{account_id}/max-debt-allowed`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct MaxDebtRequest {
    pub value: u64,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}
