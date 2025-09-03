// temporary file to hold the code for the accounts module

use axum::Router;
use axum::routing::{get, post, delete, patch, put};
use crate::db::{self};
use axum::extract::{State};
use axum::{Json};
use crate::models::account::Account;
use axum::http::StatusCode;
use axum::extract::rejection::JsonRejection;
use axum::response::IntoResponse;


pub async fn get_accounts(State(db): State<db::DatabaseClient>) -> Result<Json<Vec<Account>>, axum::http::StatusCode> {
    match db.get_accounts().await {
        Ok(accounts) => {
            println!("Fetched accounts: {:?}", accounts);
            Ok(Json(accounts))
        },
        Err(_e) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_account(
    State(db): State<db::DatabaseClient>,
    payload: Result<Json<Account>, JsonRejection>
) -> impl IntoResponse {
    match payload {
        Ok(Json(account)) => match db.create_account(account).await {
            Ok(new_account) => Json(new_account).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)).into_response(),
        },
        Err(rejection) => {
            println!("Deserialization error: {:?}", rejection);
            (StatusCode::UNPROCESSABLE_ENTITY, format!("Invalid payload: {:?}", rejection)).into_response()
        }
    }
}

pub fn account_router(db: db::DatabaseClient) -> Router<()> {
    Router::new()
        .route("/", get(get_accounts))
        .route("/", post(create_account))
        // .route("/{id}", get(get_account_by_id))
        // .route("/{id}", delete(delete_account))
        // .route("/{id}", patch(patch_account))
        // .route("/{id}/metadata", put(put_account_metadata))
        .with_state(db)

}

