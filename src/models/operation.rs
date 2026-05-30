use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::Labels;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "operation_kind", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationKind {
    Debit,
    Credit,
}

/// An append-only ledger entry.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Operation {
    pub id: Uuid,
    pub account_id: Uuid,
    pub amount: i32,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub timestamp: OffsetDateTime,
    #[schema(value_type = HashMap<String, String>)]
    pub labels: Labels,
    pub kind: OperationKind,
}

/// Body for `POST /accounts/{id}/operations`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateOperationRequest {
    pub kind: OperationKind,
    pub amount: i32,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}
