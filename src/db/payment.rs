use std::collections::HashMap;

use sqlx::types::Json;
use uuid::Uuid;

use crate::db::operation::create_operation_tx;
use crate::db::DatabaseClient;
use crate::errors::AccountingError;
use crate::models::payment::{
    CreatePaymentRequest, PatchPaymentRequest, Payment, PaymentEvent, PaymentMethod,
    PaymentPurpose, PaymentStatus,
};
use crate::payment::state_machine::{can_transition, ledger_effect, valid_combo};

/// Insert a method-less payment row in the given status plus its initiating
/// event, inside an existing transaction. `method`/`operation_id` are left NULL.
async fn insert_payment(
    conn: &mut sqlx::PgConnection,
    account_id: Uuid,
    amount: i32,
    purpose: PaymentPurpose,
    status: PaymentStatus,
    labels: HashMap<String, String>,
    reason: Option<String>,
    from: Option<PaymentStatus>,
) -> Result<Payment, AccountingError> {
    let payment = sqlx::query_as!(
        Payment,
        r#"
        INSERT INTO payments
            (id, account_id, amount, purpose, status, labels, reason, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, now(), now())
        RETURNING id, account_id, amount,
                  method as "method?: PaymentMethod",
                  purpose as "purpose: PaymentPurpose",
                  status as "status: PaymentStatus",
                  operation_id,
                  labels as "labels: Json<HashMap<String, String>>",
                  reason, created_at, updated_at
        "#,
        Uuid::new_v4(),
        account_id,
        amount,
        purpose as PaymentPurpose,
        status as PaymentStatus,
        Json(labels) as _,
        reason,
    )
    .fetch_one(&mut *conn)
    .await?;

    insert_payment_event(&mut *conn, payment.id, from, status, payment.reason.clone()).await?;
    Ok(payment)
}

/// Append an immutable status-transition event for a payment.
async fn insert_payment_event(
    conn: &mut sqlx::PgConnection,
    payment_id: Uuid,
    from: Option<PaymentStatus>,
    to: PaymentStatus,
    reason: Option<String>,
) -> Result<(), AccountingError> {
    sqlx::query!(
        r#"
        INSERT INTO payment_events
            (id, payment_id, from_status, to_status, timestamp, reason, actor)
        VALUES ($1, $2, $3, $4, now(), $5, NULL)
        "#,
        Uuid::new_v4(),
        payment_id,
        from as Option<PaymentStatus>,
        to as PaymentStatus,
        reason,
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// PENDING → PROCESSING: record the chosen method.
async fn set_processing_tx(
    conn: &mut sqlx::PgConnection,
    payment_id: Uuid,
    method: PaymentMethod,
) -> Result<Payment, AccountingError> {
    let payment = sqlx::query_as!(
        Payment,
        r#"
        UPDATE payments
        SET method = $2, status = 'PROCESSING', updated_at = now()
        WHERE id = $1
        RETURNING id, account_id, amount,
                  method as "method?: PaymentMethod",
                  purpose as "purpose: PaymentPurpose",
                  status as "status: PaymentStatus",
                  operation_id,
                  labels as "labels: Json<HashMap<String, String>>",
                  reason, created_at, updated_at
        "#,
        payment_id,
        method as PaymentMethod,
    )
    .fetch_one(&mut *conn)
    .await?;
    Ok(payment)
}

/// Set a terminal status (and optionally the settling operation), keeping method.
async fn finalize_status_tx(
    conn: &mut sqlx::PgConnection,
    payment_id: Uuid,
    status: PaymentStatus,
    operation_id: Option<Uuid>,
) -> Result<Payment, AccountingError> {
    let payment = sqlx::query_as!(
        Payment,
        r#"
        UPDATE payments
        SET status = $2,
            operation_id = COALESCE($3, operation_id),
            updated_at = now()
        WHERE id = $1
        RETURNING id, account_id, amount,
                  method as "method?: PaymentMethod",
                  purpose as "purpose: PaymentPurpose",
                  status as "status: PaymentStatus",
                  operation_id,
                  labels as "labels: Json<HashMap<String, String>>",
                  reason, created_at, updated_at
        "#,
        payment_id,
        status as PaymentStatus,
        operation_id,
    )
    .fetch_one(&mut *conn)
    .await?;
    Ok(payment)
}

/// PROCESSING → PENDING revert: clear the method so the payment is re-payable.
async fn revert_pending_tx(
    conn: &mut sqlx::PgConnection,
    payment_id: Uuid,
) -> Result<Payment, AccountingError> {
    let payment = sqlx::query_as!(
        Payment,
        r#"
        UPDATE payments
        SET method = NULL, status = 'PENDING', updated_at = now()
        WHERE id = $1
        RETURNING id, account_id, amount,
                  method as "method?: PaymentMethod",
                  purpose as "purpose: PaymentPurpose",
                  status as "status: PaymentStatus",
                  operation_id,
                  labels as "labels: Json<HashMap<String, String>>",
                  reason, created_at, updated_at
        "#,
        payment_id,
    )
    .fetch_one(&mut *conn)
    .await?;
    Ok(payment)
}

impl DatabaseClient {
    /// Create a payment as a method-less PENDING intent. No ledger effect yet.
    pub async fn create_payment(
        &self,
        req: CreatePaymentRequest,
    ) -> Result<Payment, AccountingError> {
        if req.amount < 0 {
            return Err(AccountingError::Validation(
                "payment amount must be non-negative".into(),
            ));
        }

        sqlx::query!(r#"SELECT id FROM accounts WHERE id = $1"#, req.account_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AccountingError::NotFound)?;

        let mut tx = self.pool.begin().await?;
        let payment = insert_payment(
            &mut tx,
            req.account_id,
            req.amount,
            req.purpose,
            PaymentStatus::Pending,
            req.labels,
            req.reason,
            None,
        )
        .await?;
        tx.commit().await?;
        Ok(payment)
    }

    /// Pay a PENDING payment by choosing a method. ACCOUNT_BALANCE resolves
    /// synchronously (PENDING→PROCESSING→SUCCEEDED, or revert to PENDING on
    /// insufficient funds); EXTERNAL is left PROCESSING for admin confirmation.
    pub async fn pay_payment(
        &self,
        payment_id: Uuid,
        method: PaymentMethod,
    ) -> Result<Payment, AccountingError> {
        let mut tx = self.pool.begin().await?;

        let current = sqlx::query!(
            r#"
            SELECT account_id, amount,
                   purpose as "purpose: PaymentPurpose",
                   status as "status: PaymentStatus",
                   labels as "labels: Json<HashMap<String, String>>"
            FROM payments
            WHERE id = $1
            FOR UPDATE
            "#,
            payment_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AccountingError::NotFound)?;

        if current.status != PaymentStatus::Pending {
            return Err(AccountingError::InvalidTransition {
                from: current.status.to_string(),
                to: PaymentStatus::Processing.to_string(),
            });
        }
        if !valid_combo(method, current.purpose) {
            return Err(AccountingError::InvalidCombination(
                "ACCOUNT_BALANCE payments cannot be TOP_UP".into(),
            ));
        }

        // PENDING -> PROCESSING (record the chosen method).
        let processing = set_processing_tx(&mut tx, payment_id, method).await?;
        insert_payment_event(
            &mut tx,
            payment_id,
            Some(PaymentStatus::Pending),
            PaymentStatus::Processing,
            None,
        )
        .await?;

        match method {
            // Synchronous: apply the ledger move now and finalize.
            PaymentMethod::AccountBalance => {
                let kind = ledger_effect(method, current.purpose)
                    .expect("ACCOUNT_BALANCE+PURCHASE always has a ledger effect");
                match create_operation_tx(
                    &mut tx,
                    current.account_id,
                    kind,
                    current.amount,
                    current.labels.0.clone(),
                )
                .await
                {
                    Ok(op) => {
                        let payment = finalize_status_tx(
                            &mut tx,
                            payment_id,
                            PaymentStatus::Succeeded,
                            Some(op.id),
                        )
                        .await?;
                        insert_payment_event(
                            &mut tx,
                            payment_id,
                            Some(PaymentStatus::Processing),
                            PaymentStatus::Succeeded,
                            None,
                        )
                        .await?;
                        tx.commit().await?;
                        Ok(payment)
                    }
                    // The funds check fails before any ledger write, so the tx
                    // stays valid: revert to PENDING + clear method (retryable).
                    Err(AccountingError::InsufficientFunds { amount }) => {
                        revert_pending_tx(&mut tx, payment_id).await?;
                        insert_payment_event(
                            &mut tx,
                            payment_id,
                            Some(PaymentStatus::Processing),
                            PaymentStatus::Pending,
                            Some("insufficient funds (ACCOUNT_BALANCE)".into()),
                        )
                        .await?;
                        tx.commit().await?;
                        Err(AccountingError::InsufficientFunds { amount })
                    }
                    Err(e) => Err(e),
                }
            }
            // Asynchronous: remains PROCESSING until an admin confirm/reject.
            PaymentMethod::External => {
                tx.commit().await?;
                Ok(processing)
            }
        }
    }

    /// Generic confirm/reject/cancel transition. On reaching SUCCEEDED the
    /// ledger effect runs (external top-up credits the account).
    pub async fn transition_payment(
        &self,
        payment_id: Uuid,
        to: PaymentStatus,
        reason: Option<String>,
    ) -> Result<Payment, AccountingError> {
        let mut tx = self.pool.begin().await?;

        let current = sqlx::query!(
            r#"
            SELECT account_id, amount,
                   method as "method?: PaymentMethod",
                   purpose as "purpose: PaymentPurpose",
                   status as "status: PaymentStatus",
                   labels as "labels: Json<HashMap<String, String>>"
            FROM payments
            WHERE id = $1
            FOR UPDATE
            "#,
            payment_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AccountingError::NotFound)?;

        if !can_transition(current.status, to) {
            return Err(AccountingError::InvalidTransition {
                from: current.status.to_string(),
                to: to.to_string(),
            });
        }

        let operation_id: Option<Uuid> = if to == PaymentStatus::Succeeded {
            let method = current.method.ok_or_else(|| {
                AccountingError::Validation("payment has no method to settle".into())
            })?;
            match ledger_effect(method, current.purpose) {
                Some(kind) => {
                    let op = create_operation_tx(
                        &mut tx,
                        current.account_id,
                        kind,
                        current.amount,
                        current.labels.0.clone(),
                    )
                    .await?;
                    Some(op.id)
                }
                None => None,
            }
        } else {
            None
        };

        let payment = finalize_status_tx(&mut tx, payment_id, to, operation_id).await?;
        insert_payment_event(&mut tx, payment_id, Some(current.status), to, reason).await?;

        tx.commit().await?;
        Ok(payment)
    }

    /// Filtered listing via the runtime `QueryBuilder` (dynamic `WHERE`).
    pub async fn list_payments(
        &self,
        filter: &crate::filters::PaymentFilter,
    ) -> Result<Vec<Payment>, AccountingError> {
        let payments = filter
            .build()
            .build_query_as::<Payment>()
            .fetch_all(&self.pool)
            .await?;
        Ok(payments)
    }

    pub async fn get_payment(&self, id: Uuid) -> Result<Option<Payment>, AccountingError> {
        let payment = sqlx::query_as!(
            Payment,
            r#"
            SELECT id, account_id, amount,
                   method as "method?: PaymentMethod",
                   purpose as "purpose: PaymentPurpose",
                   status as "status: PaymentStatus",
                   operation_id,
                   labels as "labels: Json<HashMap<String, String>>",
                   reason, created_at, updated_at
            FROM payments
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(payment)
    }

    pub async fn list_payment_events(
        &self,
        payment_id: Uuid,
    ) -> Result<Vec<PaymentEvent>, AccountingError> {
        let events = sqlx::query_as!(
            PaymentEvent,
            r#"
            SELECT id, payment_id,
                   from_status as "from_status?: PaymentStatus",
                   to_status as "to_status: PaymentStatus",
                   timestamp, reason, actor
            FROM payment_events
            WHERE payment_id = $1
            ORDER BY timestamp ASC
            "#,
            payment_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(events)
    }

    pub async fn patch_payment(
        &self,
        id: Uuid,
        req: PatchPaymentRequest,
    ) -> Result<Option<Payment>, AccountingError> {
        let payment = sqlx::query_as!(
            Payment,
            r#"
            UPDATE payments
            SET labels = COALESCE($2, labels),
                reason = COALESCE($3::text, reason),
                updated_at = now()
            WHERE id = $1
            RETURNING id, account_id, amount,
                      method as "method?: PaymentMethod",
                      purpose as "purpose: PaymentPurpose",
                      status as "status: PaymentStatus",
                      operation_id,
                      labels as "labels: Json<HashMap<String, String>>",
                      reason, created_at, updated_at
            "#,
            id,
            req.labels.map(Json) as _,
            req.reason,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(payment)
    }
}
