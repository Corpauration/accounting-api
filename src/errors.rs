use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;
use tracing::error;

use crate::db::errors::RepositoryError;

/// Application-wide error type. Implements `IntoResponse` so handlers can simply
/// return `Result<T, AccountingError>` and get a consistent JSON error body.
#[derive(Debug, Error)]
pub enum AccountingError {
    #[error("resource not found")]
    NotFound,

    #[error("insufficient funds: balance plus allowed debt cannot cover an amount of {amount}")]
    InsufficientFunds { amount: i32 },

    #[error("invalid status transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    #[error("invalid payment combination: {0}")]
    InvalidCombination(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error("database error")]
    Db(#[from] sqlx::Error),
}

impl AccountingError {
    fn status(&self) -> StatusCode {
        match self {
            AccountingError::NotFound => StatusCode::NOT_FOUND,
            AccountingError::InsufficientFunds { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            AccountingError::InvalidTransition { .. } => StatusCode::CONFLICT,
            AccountingError::InvalidCombination(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AccountingError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AccountingError::Repository(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AccountingError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AccountingError {
    fn into_response(self) -> Response {
        let status = self.status();
        // Log full detail server-side; never leak internal DB errors to clients.
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            error!("internal error: {self:?}");
        }
        let message = match status {
            StatusCode::INTERNAL_SERVER_ERROR => "internal server error".to_string(),
            _ => self.to_string(),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
