use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::Labels;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "payment_method", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentMethod {
    AccountBalance,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "payment_purpose", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentPurpose {
    Purchase,
    TopUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "payment_status", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Canceled,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PaymentStatus::Pending => "PENDING",
            PaymentStatus::Processing => "PROCESSING",
            PaymentStatus::Succeeded => "SUCCEEDED",
            PaymentStatus::Failed => "FAILED",
            PaymentStatus::Canceled => "CANCELED",
        };
        f.write_str(s)
    }
}

/// A payment row as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub account_id: Uuid,
    pub amount: i32,
    /// Chosen at pay-time; `None` while the payment is an unpaid PENDING intent.
    pub method: Option<PaymentMethod>,
    pub purpose: PaymentPurpose,
    pub status: PaymentStatus,
    pub operation_id: Option<Uuid>,
    #[schema(value_type = HashMap<String, String>)]
    pub labels: Labels,
    pub reason: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: OffsetDateTime,
}

/// An immutable record of a single payment status transition.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaymentEvent {
    pub id: Uuid,
    pub payment_id: Uuid,
    pub from_status: Option<PaymentStatus>,
    pub to_status: PaymentStatus,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub timestamp: OffsetDateTime,
    pub reason: Option<String>,
    pub actor: Option<String>,
}

/// Body for `POST /payments`. A payment is created without a method — it is a
/// pending intent until paid.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreatePaymentRequest {
    pub account_id: Uuid,
    pub amount: i32,
    pub purpose: PaymentPurpose,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Body for `POST /payments/{id}/pay` — choose the method and pay.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PayPaymentRequest {
    pub method: PaymentMethod,
}

/// Body for the confirm / reject / cancel transition endpoints (all optional).
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct TransitionRequest {
    #[serde(default)]
    pub reason: Option<String>,
}

/// Body for `PATCH /payments/{id}` (metadata only).
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PatchPaymentRequest {
    #[serde(default)]
    pub labels: Option<HashMap<String, String>>,
    #[serde(default)]
    pub reason: Option<String>,
}
