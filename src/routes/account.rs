use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::db::DatabaseClient;
use crate::errors::AccountingError;
use crate::filters::AccountFilter;
use crate::models::account::{Account, CreateAccountRequest, MetadataRequest, PatchAccountRequest};

#[utoipa::path(
    get, path = "/accounts", tag = "accounts",
    params(AccountFilter),
    responses((status = 200, description = "Matching accounts", body = Vec<Account>))
)]
pub async fn list_accounts(
    State(db): State<DatabaseClient>,
    Query(filter): Query<AccountFilter>,
) -> Result<Json<Vec<Account>>, AccountingError> {
    Ok(Json(db.list_accounts(&filter).await?))
}

#[utoipa::path(
    post, path = "/accounts", tag = "accounts",
    request_body = CreateAccountRequest,
    responses((status = 201, description = "Account created", body = Account))
)]
pub async fn create_account(
    State(db): State<DatabaseClient>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<Account>), AccountingError> {
    let account = db.create_account(req).await?;
    Ok((StatusCode::CREATED, Json(account)))
}

#[utoipa::path(
    get, path = "/accounts/{id}", tag = "accounts",
    params(("id" = Uuid, Path, description = "Account id")),
    responses((status = 200, body = Account), (status = 404, description = "Not found"))
)]
pub async fn get_account(
    State(db): State<DatabaseClient>,
    Path(id): Path<Uuid>,
) -> Result<Json<Account>, AccountingError> {
    db.get_account_by_id(id)
        .await?
        .map(Json)
        .ok_or(AccountingError::NotFound)
}

#[utoipa::path(
    patch, path = "/accounts/{id}", tag = "accounts",
    params(("id" = Uuid, Path, description = "Account id")),
    request_body = PatchAccountRequest,
    responses((status = 200, body = Account), (status = 404, description = "Not found"))
)]
pub async fn patch_account(
    State(db): State<DatabaseClient>,
    Path(id): Path<Uuid>,
    Json(req): Json<PatchAccountRequest>,
) -> Result<Json<Account>, AccountingError> {
    db.patch_account(id, req)
        .await?
        .map(Json)
        .ok_or(AccountingError::NotFound)
}

#[utoipa::path(
    delete, path = "/accounts/{id}", tag = "accounts",
    params(("id" = Uuid, Path, description = "Account id")),
    responses((status = 200, description = "Soft-deleted account", body = Account), (status = 404, description = "Not found"))
)]
pub async fn delete_account(
    State(db): State<DatabaseClient>,
    Path(id): Path<Uuid>,
) -> Result<Json<Account>, AccountingError> {
    db.delete_account(id)
        .await?
        .map(Json)
        .ok_or(AccountingError::NotFound)
}

#[utoipa::path(
    put, path = "/accounts/{id}/metadata", tag = "accounts",
    params(("id" = Uuid, Path, description = "Account id")),
    request_body = MetadataRequest,
    responses((status = 200, body = Account), (status = 404, description = "Not found"))
)]
pub async fn put_metadata(
    State(db): State<DatabaseClient>,
    Path(id): Path<Uuid>,
    Json(req): Json<MetadataRequest>,
) -> Result<Json<Account>, AccountingError> {
    db.set_account_metadata(id, req)
        .await?
        .map(Json)
        .ok_or(AccountingError::NotFound)
}

/// Absolute-path routes for this slice, generic over `DatabaseClient` state.
/// State is applied centrally in `routes::api`.
pub fn routes() -> OpenApiRouter<DatabaseClient> {
    OpenApiRouter::new()
        .routes(routes!(list_accounts, create_account))
        .routes(routes!(get_account, patch_account, delete_account))
        .routes(routes!(put_metadata))
}
