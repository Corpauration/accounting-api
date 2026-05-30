use axum::extract::{Path, Query, State};
use axum::Json;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::db::DatabaseClient;
use crate::errors::AccountingError;
use crate::filters::AdminActionFilter;
use crate::models::admin_action::AdminAction;

#[utoipa::path(
    get, path = "/audit/actions", tag = "audit",
    params(AdminActionFilter),
    responses((status = 200, description = "Matching admin actions", body = Vec<AdminAction>))
)]
pub async fn list_actions(
    State(db): State<DatabaseClient>,
    Query(filter): Query<AdminActionFilter>,
) -> Result<Json<Vec<AdminAction>>, AccountingError> {
    Ok(Json(db.list_audit_actions(&filter).await?))
}

#[utoipa::path(
    get, path = "/audit/actions/{id}", tag = "audit",
    params(("id" = Uuid, Path, description = "Admin action id")),
    responses((status = 200, body = AdminAction), (status = 404, description = "Not found"))
)]
pub async fn get_action(
    State(db): State<DatabaseClient>,
    Path(id): Path<Uuid>,
) -> Result<Json<AdminAction>, AccountingError> {
    db.get_audit_action(id)
        .await?
        .map(Json)
        .ok_or(AccountingError::NotFound)
}

pub fn routes() -> OpenApiRouter<DatabaseClient> {
    OpenApiRouter::new()
        .routes(routes!(list_actions))
        .routes(routes!(get_action))
}
