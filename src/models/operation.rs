use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "operation_kind")]
pub enum OperationKind {
    #[sqlx(rename = "DEBIT")]
    Debit,
    #[sqlx(rename = "CREDIT")]
    Credit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: Uuid,
    pub account_id: Uuid,
    pub amount: i32,
    pub timestamp: OffsetDateTime,
    pub labels: HashMap<String, String>,
    pub kind: OperationKind,
}

impl Operation {
    pub fn new(
        id: Uuid,
        account_id: Uuid,
        amount: i32,
        timestamp: OffsetDateTime,
        labels: HashMap<String, String>,
        kind: OperationKind,
    ) -> Self {
        Self {
            id,
            account_id,
            amount,
            timestamp,
            labels,
            kind,
        }
    }
}