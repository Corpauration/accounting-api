use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::errors::RepositoryError;
use crate::db::DatabaseClient;
use crate::errors::AccountingError;
use crate::models::admin_action::{AdminAction, AdminActionKind};
use crate::models::Labels;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdminActionEntity {
    pub id: Uuid,
    pub account_id: Uuid,
    pub timestamp: OffsetDateTime,
    pub labels: Labels,
    pub reason: String,
    pub kind: String,
    pub data: serde_json::Value,
    pub actor: Option<String>,
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
            labels: Json(action.labels),
            reason: action.reason,
            kind,
            data,
            actor: action.actor,
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
                    .ok_or(RepositoryError::InvalidMaxDebtValue)?;
                AdminActionKind::MaxDebtUpdated { value }
            }
            kind => return Err(RepositoryError::InvalidActionKind(kind.to_string()).into()),
        };

        Ok(Self {
            id: entity.id,
            account_id: entity.account_id,
            timestamp: entity.timestamp,
            labels: entity.labels.0,
            reason: entity.reason,
            action,
            actor: entity.actor,
        })
    }
}

impl DatabaseClient {
    /// Apply an admin action's side effect to the account and record the action,
    /// atomically in a single transaction.
    pub async fn apply_admin_action(
        &self,
        account_id: Uuid,
        action: AdminActionKind,
        reason: String,
        labels: HashMap<String, String>,
    ) -> Result<AdminAction, AccountingError> {
        let mut tx = self.pool.begin().await?;

        // 1) Apply the side effect to the account, then build the typed action.
        let admin_action = match action {
            AdminActionKind::AccountDeactivated => {
                sqlx::query!(
                    r#"
                    UPDATE accounts
                    SET status = 'DEACTIVATED', updated_at = now()
                    WHERE id = $1
                    RETURNING id
                    "#,
                    account_id
                )
                .fetch_optional(&mut *tx)
                .await?
                .ok_or(AccountingError::NotFound)?;

                AdminAction::new_account_deactivated(account_id, labels.clone(), reason.clone())
            }
            AdminActionKind::AccountReactivated => {
                sqlx::query!(
                    r#"
                    UPDATE accounts
                    SET status = 'ACTIVE', updated_at = now()
                    WHERE id = $1
                    RETURNING id
                    "#,
                    account_id
                )
                .fetch_optional(&mut *tx)
                .await?
                .ok_or(AccountingError::NotFound)?;

                AdminAction::new_account_reactivated(account_id, labels.clone(), reason.clone())
            }
            AdminActionKind::MaxDebtUpdated { value } => {
                let max_debt = i32::try_from(value).map_err(|_| {
                    AccountingError::Validation(format!(
                        "max_allowed_debt value {value} does not fit in i32"
                    ))
                })?;

                sqlx::query!(
                    r#"
                    UPDATE accounts
                    SET max_allowed_debt = $2, updated_at = now()
                    WHERE id = $1
                    RETURNING id
                    "#,
                    account_id,
                    max_debt
                )
                .fetch_optional(&mut *tx)
                .await?
                .ok_or(AccountingError::NotFound)?;

                AdminAction::new_max_debt_updated(account_id, value, labels.clone(), reason.clone())
            }
        };

        // 2) Record the admin action. Convert to the entity (the storage shape),
        // insert it, then convert back so we return exactly what was persisted.
        let entity = AdminActionEntity::from(admin_action);
        sqlx::query!(
            r#"
            INSERT INTO admin_actions (id, account_id, timestamp, labels, reason, kind, data, actor)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            entity.id,
            entity.account_id,
            entity.timestamp,
            Json(&entity.labels.0) as _,
            entity.reason,
            entity.kind,
            Json(&entity.data) as _,
            entity.actor,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        AdminAction::try_from(entity)
    }

    /// Filtered audit listing across all accounts. Dynamic `WHERE` → runtime
    /// `QueryBuilder` (the entity derives `FromRow` for `build_query_as`).
    pub async fn list_audit_actions(
        &self,
        filter: &crate::filters::AdminActionFilter,
    ) -> Result<Vec<AdminAction>, AccountingError> {
        let entities = filter
            .build()
            .build_query_as::<AdminActionEntity>()
            .fetch_all(&self.pool)
            .await?;

        entities
            .into_iter()
            .map(AdminAction::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_audit_action(&self, id: Uuid) -> Result<Option<AdminAction>, AccountingError> {
        let entity = sqlx::query_as!(
            AdminActionEntity,
            r#"
            SELECT id, account_id, timestamp,
                   labels as "labels: Json<HashMap<String, String>>",
                   reason, kind,
                   data as "data: serde_json::Value",
                   actor
            FROM admin_actions
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        entity.map(AdminAction::try_from).transpose()
    }
}
