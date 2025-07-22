pub mod account;

use axum::routing::get;
use axum::Router;

pub async fn api() -> Router<()> {
    Router::new()
    .route("/", get("nothing to see here..."))
    .nest("/accounts", account::account_router().await)
}