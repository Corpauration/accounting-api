// temporary file to hold the code for the accounts module

use axum::extract::Path;
use axum::Router;
use axum::routing::{get, post, delete, patch, put};
use crate::db::DatabaseClient;

pub async fn get_accounts() -> String {
    
    format!("Get all accounts")
}

pub async fn create_account() -> String {
    format!("Creating account")
}

pub async fn get_account_by_id(Path(id): Path<u64>) -> String {
    format!("Get account {id}")
}

pub async fn delete_account(Path(id): Path<u64>) -> String {
    format!("Delete account {id}")
}

pub async fn patch_account(Path(id): Path<u64>) -> String {
    format!("Patch account {id}")
}

pub async fn put_account_metadata(Path(id): Path<u64>) -> String {
    format!("Put metadata for account {id}")
}

pub async fn account_router() -> Router<()> {
    Router::new()
        .route("/", get(get_accounts))
        .route("/", post(create_account))
        .route("/{id}", get(get_account_by_id))
        .route("/{id}", delete(delete_account))
        .route("/{id}", patch(patch_account))
        .route("/{id}/metadata", put(put_account_metadata))
}
