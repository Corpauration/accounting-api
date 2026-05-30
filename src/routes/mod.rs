pub mod account;
pub mod admin_action;
pub mod audit;
pub mod operation;
pub mod payment;

use axum::http::header;
use axum::routing::get;
use axum::{Json, Router};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable as _};

use crate::db::DatabaseClient;
use crate::docs::ApiDoc;

pub fn api(db: DatabaseClient) -> Router {
    // Every slice contributes its paths + schemas to the generated spec.
    let (router, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(account::routes())
        .merge(operation::routes())
        .merge(admin_action::routes())
        .merge(audit::routes())
        .merge(payment::routes())
        .split_for_parts();

    let openapi_json = openapi.clone();

    router
        .merge(Scalar::with_url("/api/docs/ui", openapi))
        .route(
            "/api/docs/openapi.json",
            get(move || {
                let spec = openapi_json.clone();
                async move { Json(spec) }
            }),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers([header::CONTENT_TYPE]),
        )
        .with_state(db)
}
