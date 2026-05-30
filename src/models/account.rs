use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::Labels;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "account_status", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountStatus {
    Active,
    Deactivated,
    Deleted,
}

/// An account row as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct Account {
    pub id: Uuid,
    pub owner_id: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    #[schema(value_type = HashMap<String, String>)]
    pub labels: Labels,
    pub max_allowed_debt: i32,
    pub balance: i32,
    pub status: AccountStatus,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: OffsetDateTime,
}

/// Body for `POST /accounts`. The server owns `id`, `balance` (starts at 0 and
/// only ever moves via operations) and `status` (starts `ACTIVE`).
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateAccountRequest {
    pub owner_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub max_allowed_debt: i32,
}

/// Body for `PATCH /accounts/{id}`. All fields optional; balance, status and
/// max_allowed_debt are intentionally not editable here (operations and
/// admin-actions — e.g. PUT /max-debt-allowed — own those).
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PatchAccountRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Body for `PUT /accounts/{id}/metadata`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct MetadataRequest {
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
}
