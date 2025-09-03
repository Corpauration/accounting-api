pub mod account;

use axum::extract::State;
use axum::routing::get;
use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use axum::http::header;


use crate::db;

pub async fn api(State(db): State<db::DatabaseClient>) -> Router<()> {
    Router::new()
    .nest("/accounts", account::account_router(db))
    .layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers([header::CONTENT_TYPE])
    )
}