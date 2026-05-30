use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::db::DatabaseClient;
use crate::errors::AccountingError;
use crate::models::operation::{CreateOperationRequest, Operation};

#[utoipa::path(
    get, path = "/accounts/{account_id}/operations", tag = "operations",
    params(("account_id" = Uuid, Path)),
    responses((status = 200, body = Vec<Operation>))
)]
pub async fn list_operations(
    State(db): State<DatabaseClient>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Vec<Operation>>, AccountingError> {
    Ok(Json(db.list_operations(account_id).await?))
}

#[utoipa::path(
    post, path = "/accounts/{account_id}/operations", tag = "operations",
    params(("account_id" = Uuid, Path)),
    request_body = CreateOperationRequest,
    responses(
        (status = 201, body = Operation),
        (status = 422, description = "Insufficient funds"),
        (status = 404, description = "Account not found")
    )
)]
pub async fn create_operation(
    State(db): State<DatabaseClient>,
    Path(account_id): Path<Uuid>,
    Json(req): Json<CreateOperationRequest>,
) -> Result<(StatusCode, Json<Operation>), AccountingError> {
    let operation = db
        .create_operation(account_id, req.kind, req.amount, req.labels)
        .await?;
    Ok((StatusCode::CREATED, Json(operation)))
}

#[utoipa::path(
    get, path = "/accounts/{account_id}/operations/{operation_id}", tag = "operations",
    params(("account_id" = Uuid, Path), ("operation_id" = Uuid, Path)),
    responses((status = 200, body = Operation), (status = 404))
)]
pub async fn get_operation(
    State(db): State<DatabaseClient>,
    Path((account_id, operation_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Operation>, AccountingError> {
    db.get_operation(account_id, operation_id)
        .await?
        .map(Json)
        .ok_or(AccountingError::NotFound)
}

pub fn routes() -> OpenApiRouter<crate::db::DatabaseClient> {
    OpenApiRouter::new()
        .routes(routes!(list_operations, create_operation))
        .routes(routes!(get_operation))
}
