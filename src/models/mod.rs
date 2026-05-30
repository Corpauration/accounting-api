pub mod account;
pub mod admin_action;
pub mod operation;
pub mod payment;

use std::collections::HashMap;

/// Labels are stored as a `jsonb` column and mapped to a string→string map.
/// sqlx requires the `Json` wrapper to decode/encode `jsonb`; it (de)serializes
/// transparently as the inner map, so API bodies see a plain object.
pub type Labels = sqlx::types::Json<HashMap<String, String>>;

/// Convenience constructor for an empty label set.
pub fn empty_labels() -> Labels {
    sqlx::types::Json(HashMap::new())
}
