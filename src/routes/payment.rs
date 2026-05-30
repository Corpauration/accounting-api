use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::db::DatabaseClient;
use crate::errors::AccountingError;
use crate::filters::PaymentFilter;
use crate::models::payment::{
    CreatePaymentRequest, PatchPaymentRequest, PayPaymentRequest, Payment, PaymentEvent,
    PaymentStatus, TransitionRequest,
};

#[utoipa::path(
    get, path = "/payments", tag = "payments",
    params(PaymentFilter),
    responses((status = 200, body = Vec<Payment>))
)]
pub async fn list_payments(
    State(db): State<DatabaseClient>,
    Query(filter): Query<PaymentFilter>,
) -> Result<Json<Vec<Payment>>, AccountingError> {
    Ok(Json(db.list_payments(&filter).await?))
}

#[utoipa::path(
    post, path = "/payments", tag = "payments",
    request_body = CreatePaymentRequest,
    responses(
        (status = 201, body = Payment),
        (status = 422, description = "Invalid combination / insufficient funds"),
        (status = 404, description = "Account not found")
    )
)]
pub async fn create_payment(
    State(db): State<DatabaseClient>,
    Json(req): Json<CreatePaymentRequest>,
) -> Result<(StatusCode, Json<Payment>), AccountingError> {
    let payment = db.create_payment(req).await?;
    Ok((StatusCode::CREATED, Json(payment)))
}

#[utoipa::path(
    post, path = "/payments/{id}/pay", tag = "payments",
    params(("id" = Uuid, Path)),
    request_body = PayPaymentRequest,
    responses(
        (status = 200, body = Payment),
        (status = 422, description = "Invalid combination / insufficient funds"),
        (status = 409, description = "Payment is not payable (not PENDING)"),
        (status = 404)
    )
)]
pub async fn pay_payment(
    State(db): State<DatabaseClient>,
    Path(id): Path<Uuid>,
    Json(req): Json<PayPaymentRequest>,
) -> Result<Json<Payment>, AccountingError> {
    Ok(Json(db.pay_payment(id, req.method).await?))
}

#[utoipa::path(
    get, path = "/payments/{id}", tag = "payments",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = Payment), (status = 404))
)]
pub async fn get_payment(
    State(db): State<DatabaseClient>,
    Path(id): Path<Uuid>,
) -> Result<Json<Payment>, AccountingError> {
    db.get_payment(id)
        .await?
        .map(Json)
        .ok_or(AccountingError::NotFound)
}

#[utoipa::path(
    patch, path = "/payments/{id}", tag = "payments",
    params(("id" = Uuid, Path)),
    request_body = PatchPaymentRequest,
    responses((status = 200, body = Payment), (status = 404))
)]
pub async fn patch_payment(
    State(db): State<DatabaseClient>,
    Path(id): Path<Uuid>,
    Json(req): Json<PatchPaymentRequest>,
) -> Result<Json<Payment>, AccountingError> {
    db.patch_payment(id, req)
        .await?
        .map(Json)
        .ok_or(AccountingError::NotFound)
}

#[utoipa::path(
    post, path = "/payments/{id}/confirm", tag = "payments",
    params(("id" = Uuid, Path)),
    request_body = TransitionRequest,
    responses(
        (status = 200, body = Payment),
        (status = 409, description = "Illegal transition"),
        (status = 404)
    )
)]
pub async fn confirm_payment(
    State(db): State<DatabaseClient>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransitionRequest>,
) -> Result<Json<Payment>, AccountingError> {
    Ok(Json(
        db.transition_payment(id, PaymentStatus::Succeeded, req.reason)
            .await?,
    ))
}

#[utoipa::path(
    post, path = "/payments/{id}/reject", tag = "payments",
    params(("id" = Uuid, Path)),
    request_body = TransitionRequest,
    responses((status = 200, body = Payment), (status = 409), (status = 404))
)]
pub async fn reject_payment(
    State(db): State<DatabaseClient>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransitionRequest>,
) -> Result<Json<Payment>, AccountingError> {
    Ok(Json(
        db.transition_payment(id, PaymentStatus::Failed, req.reason)
            .await?,
    ))
}

#[utoipa::path(
    post, path = "/payments/{id}/cancel", tag = "payments",
    params(("id" = Uuid, Path)),
    request_body = TransitionRequest,
    responses((status = 200, body = Payment), (status = 409), (status = 404))
)]
pub async fn cancel_payment(
    State(db): State<DatabaseClient>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransitionRequest>,
) -> Result<Json<Payment>, AccountingError> {
    Ok(Json(
        db.transition_payment(id, PaymentStatus::Canceled, req.reason)
            .await?,
    ))
}

#[utoipa::path(
    get, path = "/payments/{id}/events", tag = "payments",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = Vec<PaymentEvent>))
)]
pub async fn list_payment_events(
    State(db): State<DatabaseClient>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<PaymentEvent>>, AccountingError> {
    Ok(Json(db.list_payment_events(id).await?))
}

pub fn routes() -> OpenApiRouter<crate::db::DatabaseClient> {
    OpenApiRouter::new()
        .routes(routes!(list_payments, create_payment))
        .routes(routes!(pay_payment))
        .routes(routes!(get_payment, patch_payment))
        .routes(routes!(confirm_payment))
        .routes(routes!(reject_payment))
        .routes(routes!(cancel_payment))
        .routes(routes!(list_payment_events))
}
