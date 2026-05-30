use std::collections::HashMap;

use sqlx::types::Json;
use uuid::Uuid;

use crate::db::DatabaseClient;
use crate::errors::AccountingError;
use crate::models::operation::{Operation, OperationKind};

/// Append a ledger operation **within an existing transaction**, enforcing the
/// debt limit on DEBIT and updating the denormalized account balance so that
/// `balance == sum(CREDIT) - sum(DEBIT)` always holds. Reused by the payment
/// lanes so the ledger move and the payment update commit atomically together.
pub async fn create_operation_tx(
    conn: &mut sqlx::PgConnection,
    account_id: Uuid,
    kind: OperationKind,
    amount: i32,
    labels: HashMap<String, String>,
) -> Result<Operation, AccountingError> {
    if amount < 0 {
        return Err(AccountingError::Validation(
            "operation amount must be non-negative".to_string(),
        ));
    }

    // Lock the account row to serialize concurrent balance changes.
    let account = sqlx::query!(
        r#"SELECT balance, max_allowed_debt FROM accounts WHERE id = $1 FOR UPDATE"#,
        account_id
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(AccountingError::NotFound)?;

    if matches!(kind, OperationKind::Debit)
        && (amount as i64) > account.balance as i64 + account.max_allowed_debt as i64
    {
        return Err(AccountingError::InsufficientFunds { amount });
    }

    let operation = sqlx::query_as!(
        Operation,
        r#"
        INSERT INTO operations (id, account_id, amount, timestamp, labels, kind)
        VALUES ($1, $2, $3, now(), $4, $5)
        RETURNING id, account_id, amount, timestamp,
                  labels as "labels: Json<HashMap<String, String>>",
                  kind as "kind: OperationKind"
        "#,
        Uuid::new_v4(),
        account_id,
        amount,
        Json(labels) as _,
        kind as OperationKind,
    )
    .fetch_one(&mut *conn)
    .await?;

    let delta = match kind {
        OperationKind::Credit => amount,
        OperationKind::Debit => -amount,
    };
    sqlx::query!(
        r#"UPDATE accounts SET balance = balance + $2, updated_at = now() WHERE id = $1"#,
        account_id,
        delta
    )
    .execute(&mut *conn)
    .await?;

    Ok(operation)
}

impl DatabaseClient {
    /// Standalone operation creation (its own transaction).
    pub async fn create_operation(
        &self,
        account_id: Uuid,
        kind: OperationKind,
        amount: i32,
        labels: HashMap<String, String>,
    ) -> Result<Operation, AccountingError> {
        let mut tx = self.pool.begin().await?;
        let operation = create_operation_tx(&mut tx, account_id, kind, amount, labels).await?;
        tx.commit().await?;
        Ok(operation)
    }

    pub async fn list_operations(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<Operation>, AccountingError> {
        let operations = sqlx::query_as!(
            Operation,
            r#"
            SELECT id, account_id, amount, timestamp,
                   labels as "labels: Json<HashMap<String, String>>",
                   kind as "kind: OperationKind"
            FROM operations
            WHERE account_id = $1
            ORDER BY timestamp DESC
            "#,
            account_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(operations)
    }

    pub async fn get_operation(
        &self,
        account_id: Uuid,
        operation_id: Uuid,
    ) -> Result<Option<Operation>, AccountingError> {
        let operation = sqlx::query_as!(
            Operation,
            r#"
            SELECT id, account_id, amount, timestamp,
                   labels as "labels: Json<HashMap<String, String>>",
                   kind as "kind: OperationKind"
            FROM operations
            WHERE account_id = $1 AND id = $2
            "#,
            account_id,
            operation_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(operation)
    }
}
