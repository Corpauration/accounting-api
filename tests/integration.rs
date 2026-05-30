//! End-to-end DB-layer tests. Each `#[sqlx::test]` gets a fresh database with
//! the migrations applied; we drive the `DatabaseClient` directly.

use std::collections::HashMap;

use accounting_api::db::DatabaseClient;
use accounting_api::errors::AccountingError;
use accounting_api::filters::AccountFilter;
use accounting_api::models::account::CreateAccountRequest;
use accounting_api::models::operation::OperationKind;
use accounting_api::models::payment::{
    CreatePaymentRequest, PaymentMethod, PaymentPurpose, PaymentStatus,
};
use sqlx::PgPool;
use uuid::Uuid;

fn new_account(owner: &str, max_debt: i32) -> CreateAccountRequest {
    CreateAccountRequest {
        owner_id: owner.to_string(),
        name: "Test".to_string(),
        description: None,
        tags: vec!["t".to_string()],
        labels: HashMap::new(),
        max_allowed_debt: max_debt,
    }
}

async fn ledger_sum(pool: &PgPool, account_id: Uuid) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT COALESCE(SUM(CASE kind WHEN 'CREDIT' THEN amount ELSE -amount END), 0)::bigint as "s!"
           FROM operations WHERE account_id = $1"#,
        account_id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test]
async fn operations_enforce_debt_limit_and_reconcile(pool: PgPool) {
    let db = DatabaseClient::new(pool.clone());
    let acc = db.create_account(new_account("owner-1", 50)).await.unwrap();
    assert_eq!(acc.balance, 0);

    db.create_operation(acc.id, OperationKind::Credit, 100, HashMap::new())
        .await
        .unwrap();
    // 130 <= 100 + 50, allowed → balance -30.
    db.create_operation(acc.id, OperationKind::Debit, 130, HashMap::new())
        .await
        .unwrap();
    let acc = db.get_account_by_id(acc.id).await.unwrap().unwrap();
    assert_eq!(acc.balance, -30);

    // 100 more would reach -130 < -50 → rejected.
    let err = db
        .create_operation(acc.id, OperationKind::Debit, 100, HashMap::new())
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        AccountingError::InsufficientFunds { amount: 100 }
    ));

    // Invariant: cached balance equals the ledger sum.
    assert_eq!(acc.balance as i64, ledger_sum(&pool, acc.id).await);
}

fn new_payment(account_id: Uuid, amount: i32, purpose: PaymentPurpose) -> CreatePaymentRequest {
    CreatePaymentRequest {
        account_id,
        amount,
        purpose,
        labels: HashMap::new(),
        reason: None,
    }
}

#[sqlx::test]
async fn account_balance_top_up_is_rejected(pool: PgPool) {
    let db = DatabaseClient::new(pool);
    let acc = db.create_account(new_account("owner-2", 0)).await.unwrap();
    let payment = db
        .create_payment(new_payment(acc.id, 10, PaymentPurpose::TopUp))
        .await
        .unwrap();
    assert_eq!(payment.status, PaymentStatus::Pending);
    assert!(payment.method.is_none());

    // Paying a TOP_UP from the balance itself is invalid.
    let err = db
        .pay_payment(payment.id, PaymentMethod::AccountBalance)
        .await
        .unwrap_err();
    assert!(matches!(err, AccountingError::InvalidCombination(_)));

    // Still a payable PENDING intent.
    let p = db.get_payment(payment.id).await.unwrap().unwrap();
    assert_eq!(p.status, PaymentStatus::Pending);
    assert!(p.method.is_none());
}

#[sqlx::test]
async fn insufficient_balance_pay_reverts_then_retries(pool: PgPool) {
    let db = DatabaseClient::new(pool);
    let acc = db.create_account(new_account("owner-3", 0)).await.unwrap();
    let payment = db
        .create_payment(new_payment(acc.id, 100, PaymentPurpose::Purchase))
        .await
        .unwrap();

    // Insufficient funds → reverts to PENDING (retryable), method cleared, no ledger move.
    let err = db
        .pay_payment(payment.id, PaymentMethod::AccountBalance)
        .await
        .unwrap_err();
    assert!(matches!(err, AccountingError::InsufficientFunds { .. }));
    let p = db.get_payment(payment.id).await.unwrap().unwrap();
    assert_eq!(p.status, PaymentStatus::Pending);
    assert!(p.method.is_none());
    assert_eq!(
        db.get_account_by_id(acc.id).await.unwrap().unwrap().balance,
        0
    );

    // Fund the account, then retry the same payment → SUCCEEDED.
    db.create_operation(acc.id, OperationKind::Credit, 100, HashMap::new())
        .await
        .unwrap();
    let paid = db
        .pay_payment(payment.id, PaymentMethod::AccountBalance)
        .await
        .unwrap();
    assert_eq!(paid.status, PaymentStatus::Succeeded);
    assert!(paid.operation_id.is_some());
    assert_eq!(
        db.get_account_by_id(acc.id).await.unwrap().unwrap().balance,
        0
    );
}

#[sqlx::test]
async fn account_balance_purchase_succeeds(pool: PgPool) {
    let db = DatabaseClient::new(pool);
    let acc = db.create_account(new_account("owner-5", 0)).await.unwrap();
    db.create_operation(acc.id, OperationKind::Credit, 100, HashMap::new())
        .await
        .unwrap();
    let payment = db
        .create_payment(new_payment(acc.id, 40, PaymentPurpose::Purchase))
        .await
        .unwrap();
    let paid = db
        .pay_payment(payment.id, PaymentMethod::AccountBalance)
        .await
        .unwrap();
    assert_eq!(paid.status, PaymentStatus::Succeeded);
    assert_eq!(paid.method, Some(PaymentMethod::AccountBalance));
    assert_eq!(
        db.get_account_by_id(acc.id).await.unwrap().unwrap().balance,
        60
    );
}

#[sqlx::test]
async fn cannot_confirm_unpaid_payment(pool: PgPool) {
    let db = DatabaseClient::new(pool);
    let acc = db.create_account(new_account("owner-6", 0)).await.unwrap();
    let payment = db
        .create_payment(new_payment(acc.id, 10, PaymentPurpose::TopUp))
        .await
        .unwrap();
    // A PENDING (unpaid) payment can't be confirmed — it must be paid first.
    let err = db
        .transition_payment(payment.id, PaymentStatus::Succeeded, None)
        .await
        .unwrap_err();
    assert!(matches!(err, AccountingError::InvalidTransition { .. }));
}

#[sqlx::test]
async fn external_top_up_credits_on_confirm(pool: PgPool) {
    let db = DatabaseClient::new(pool);
    let acc = db.create_account(new_account("owner-4", 0)).await.unwrap();
    let payment = db
        .create_payment(new_payment(acc.id, 200, PaymentPurpose::TopUp))
        .await
        .unwrap();
    assert_eq!(payment.status, PaymentStatus::Pending);

    // Pay with EXTERNAL → PROCESSING, no ledger yet.
    let processing = db
        .pay_payment(payment.id, PaymentMethod::External)
        .await
        .unwrap();
    assert_eq!(processing.status, PaymentStatus::Processing);
    assert_eq!(processing.method, Some(PaymentMethod::External));
    assert_eq!(
        db.get_account_by_id(acc.id).await.unwrap().unwrap().balance,
        0
    );

    // Admin confirm → SUCCEEDED, account credited.
    let confirmed = db
        .transition_payment(payment.id, PaymentStatus::Succeeded, Some("ok".into()))
        .await
        .unwrap();
    assert_eq!(confirmed.status, PaymentStatus::Succeeded);
    assert!(confirmed.operation_id.is_some());
    assert_eq!(
        db.get_account_by_id(acc.id).await.unwrap().unwrap().balance,
        200
    );

    // Re-confirming a terminal payment is illegal.
    let err = db
        .transition_payment(payment.id, PaymentStatus::Succeeded, None)
        .await
        .unwrap_err();
    assert!(matches!(err, AccountingError::InvalidTransition { .. }));

    // Full trail: NULL→PENDING, PENDING→PROCESSING, PROCESSING→SUCCEEDED.
    let events = db.list_payment_events(payment.id).await.unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].to_status, PaymentStatus::Pending);
    assert_eq!(events[1].to_status, PaymentStatus::Processing);
    assert_eq!(events[2].to_status, PaymentStatus::Succeeded);
}

#[sqlx::test]
async fn account_search_filters(pool: PgPool) {
    let db = DatabaseClient::new(pool);
    let rich = db.create_account(new_account("rich", 0)).await.unwrap();
    db.create_operation(rich.id, OperationKind::Credit, 500, HashMap::new())
        .await
        .unwrap();
    let poor = db.create_account(new_account("poor", 100)).await.unwrap();
    db.create_operation(poor.id, OperationKind::Debit, 80, HashMap::new())
        .await
        .unwrap();

    let in_debt = db
        .list_accounts(&AccountFilter {
            in_debt: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(in_debt.iter().any(|a| a.id == poor.id));
    assert!(!in_debt.iter().any(|a| a.id == rich.id));

    let high = db
        .list_accounts(&AccountFilter {
            min_balance: Some(200),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(high.iter().any(|a| a.id == rich.id));
    assert!(!high.iter().any(|a| a.id == poor.id));
}
