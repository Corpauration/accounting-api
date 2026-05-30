use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::db::DatabaseClient;
use crate::errors::AccountingError;
use crate::models::admin_action::{AdminAction, AdminActionKind, MaxDebtRequest, ShortcutRequest};

#[utoipa::path(
    post, path = "/accounts/{account_id}/deactivate", tag = "account management",
    params(("account_id" = Uuid, Path)),
    request_body = ShortcutRequest,
    responses((status = 201, body = AdminAction), (status = 404))
)]
pub async fn deactivate_account(
    State(db): State<DatabaseClient>,
    Path(account_id): Path<Uuid>,
    Json(req): Json<ShortcutRequest>,
) -> Result<(StatusCode, Json<AdminAction>), AccountingError> {
    let action = db
        .apply_admin_action(
            account_id,
            AdminActionKind::AccountDeactivated,
            req.reason,
            req.labels,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(action)))
}

#[utoipa::path(
    post, path = "/accounts/{account_id}/reactivate", tag = "account management",
    params(("account_id" = Uuid, Path)),
    request_body = ShortcutRequest,
    responses((status = 201, body = AdminAction), (status = 404))
)]
pub async fn reactivate_account(
    State(db): State<DatabaseClient>,
    Path(account_id): Path<Uuid>,
    Json(req): Json<ShortcutRequest>,
) -> Result<(StatusCode, Json<AdminAction>), AccountingError> {
    let action = db
        .apply_admin_action(
            account_id,
            AdminActionKind::AccountReactivated,
            req.reason,
            req.labels,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(action)))
}

#[utoipa::path(
    post, path = "/accounts/{account_id}/max-debt-allowed", tag = "account management",
    params(("account_id" = Uuid, Path)),
    request_body = MaxDebtRequest,
    responses((status = 201, body = AdminAction), (status = 404))
)]
pub async fn set_max_debt_allowed(
    State(db): State<DatabaseClient>,
    Path(account_id): Path<Uuid>,
    Json(req): Json<MaxDebtRequest>,
) -> Result<(StatusCode, Json<AdminAction>), AccountingError> {
    let action = db
        .apply_admin_action(
            account_id,
            AdminActionKind::MaxDebtUpdated { value: req.value },
            req.reason,
            req.labels,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(action)))
}

pub fn routes() -> OpenApiRouter<DatabaseClient> {
    OpenApiRouter::new()
        .routes(routes!(deactivate_account))
        .routes(routes!(reactivate_account))
        .routes(routes!(set_max_debt_allowed))
}
