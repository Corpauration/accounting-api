use crate::db::errors::RepositoryError;
use crate::errors::AccountingError;
use crate::models::admin_action::{AdminAction, AdminActionKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminActionEntity {
    pub id: Uuid,
    pub account_id: Uuid,
    pub timestamp: OffsetDateTime,
    pub labels: HashMap<String, String>,
    pub reason: String,
    pub kind: String,
    pub data: serde_json::Value,
}

impl From<AdminAction> for AdminActionEntity {
    fn from(action: AdminAction) -> Self {
        let (kind, data) = match action.action {
            AdminActionKind::AccountDeactivated => {
                ("AccountDeactivated".to_string(), serde_json::json!({}))
            }
            AdminActionKind::AccountReactivated => {
                ("AccountReactivated".to_string(), serde_json::json!({}))
            }
            AdminActionKind::MaxDebtUpdated { value } => (
                "MaxDebtUpdated".to_string(),
                serde_json::json!({ "value": value }),
            ),
        };

        Self {
            id: action.id,
            account_id: action.account_id,
            timestamp: action.timestamp,
            labels: action.labels,
            reason: action.reason,
            kind,
            data,
        }
    }
}

impl TryFrom<AdminActionEntity> for AdminAction {
    type Error = AccountingError;

    fn try_from(entity: AdminActionEntity) -> Result<Self, Self::Error> {
        let action = match entity.kind.as_str() {
            "AccountDeactivated" => AdminActionKind::AccountDeactivated,
            "AccountReactivated" => AdminActionKind::AccountReactivated,
            "MaxDebtUpdated" => {
                let value = entity
                    .data
                    .get("value")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| RepositoryError::InvalidMaxDebtValue.into())?;
                AdminActionKind::MaxDebtUpdated { value }
            }
            kind => return Err(RepositoryError::InvalidActionKind(kind.to_string()).into()),
        };

        Ok(Self {
            id: entity.id,
            account_id: entity.account_id,
            timestamp: entity.timestamp,
            labels: entity.labels,
            reason: entity.reason,
            action,
        })
    }
}
