//! Query-parameter filters for the list endpoints. These build dynamic `WHERE`
//! clauses, which sqlx's compile-time macros cannot express, so they are the one
//! place we use the runtime `QueryBuilder` + `push_bind` (values are always
//! bound, never interpolated; sort columns come from a fixed allow-list).

use serde::Deserialize;
use sqlx::{Postgres, QueryBuilder};
use utoipa::IntoParams;

use uuid::Uuid;

use crate::models::account::AccountStatus;
use crate::models::payment::{PaymentMethod, PaymentPurpose, PaymentStatus};

const DEFAULT_LIMIT: i64 = 100;
const MAX_LIMIT: i64 = 500;

fn clamp_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

fn offset(offset: Option<i64>) -> i64 {
    offset.unwrap_or(0).max(0)
}

/// Resolve a user-supplied sort direction to `ASC`/`DESC` (defaults to `ASC`).
fn direction(order: Option<&str>) -> &'static str {
    match order {
        Some(o) if o.eq_ignore_ascii_case("desc") => "DESC",
        _ => "ASC",
    }
}

#[derive(Debug, Default, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AccountFilter {
    /// Only accounts with `balance >= min_balance`.
    pub min_balance: Option<i32>,
    /// Only accounts with `balance <= max_balance`.
    pub max_balance: Option<i32>,
    /// Only accounts in debt (`balance < 0`).
    pub in_debt: Option<bool>,
    pub status: Option<AccountStatus>,
    pub owner_id: Option<String>,
    /// Case-insensitive substring match on the account name.
    pub name: Option<String>,
    /// Match accounts carrying this tag.
    pub tag: Option<String>,
    /// Sort column: one of `balance`, `name`, `created_at` (default `created_at`).
    pub sort_by: Option<String>,
    /// `asc` or `desc` (default `asc`).
    pub order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl AccountFilter {
    pub fn build(&self) -> QueryBuilder<Postgres> {
        let mut qb = QueryBuilder::new("SELECT * FROM accounts WHERE TRUE");
        if let Some(v) = self.min_balance {
            qb.push(" AND balance >= ").push_bind(v);
        }
        if let Some(v) = self.max_balance {
            qb.push(" AND balance <= ").push_bind(v);
        }
        if self.in_debt == Some(true) {
            qb.push(" AND balance < 0");
        }
        if let Some(status) = &self.status {
            qb.push(" AND status = ").push_bind(status.clone());
        }
        if let Some(owner) = &self.owner_id {
            qb.push(" AND owner_id = ").push_bind(owner.clone());
        }
        if let Some(name) = &self.name {
            qb.push(" AND name ILIKE ").push_bind(format!("%{name}%"));
        }
        if let Some(tag) = &self.tag {
            qb.push(" AND ").push_bind(tag.clone()).push(" = ANY(tags)");
        }

        let sort_col = match self.sort_by.as_deref() {
            Some("balance") => "balance",
            Some("name") => "name",
            _ => "created_at",
        };
        qb.push(format!(
            " ORDER BY {sort_col} {}",
            direction(self.order.as_deref())
        ));
        qb.push(" LIMIT ").push_bind(clamp_limit(self.limit));
        qb.push(" OFFSET ").push_bind(offset(self.offset));
        qb
    }
}

#[derive(Debug, Default, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaymentFilter {
    pub account_id: Option<uuid::Uuid>,
    pub status: Option<PaymentStatus>,
    pub method: Option<PaymentMethod>,
    pub purpose: Option<PaymentPurpose>,
    pub min_amount: Option<i32>,
    pub max_amount: Option<i32>,
    /// RFC3339 timestamp lower bound (inclusive) on `created_at`.
    #[serde(default, with = "time::serde::rfc3339::option")]
    #[param(value_type = Option<String>, format = DateTime)]
    pub created_after: Option<time::OffsetDateTime>,
    /// RFC3339 timestamp upper bound (inclusive) on `created_at`.
    #[serde(default, with = "time::serde::rfc3339::option")]
    #[param(value_type = Option<String>, format = DateTime)]
    pub created_before: Option<time::OffsetDateTime>,
    /// Sort column: one of `created_at`, `amount` (default `created_at`).
    pub sort_by: Option<String>,
    /// `asc` or `desc` (default `desc`).
    pub order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl PaymentFilter {
    pub fn build(&self) -> QueryBuilder<Postgres> {
        let mut qb = QueryBuilder::new("SELECT * FROM payments WHERE TRUE");
        if let Some(id) = self.account_id {
            qb.push(" AND account_id = ").push_bind(id);
        }
        if let Some(status) = self.status {
            qb.push(" AND status = ").push_bind(status);
        }
        if let Some(method) = self.method {
            qb.push(" AND method = ").push_bind(method);
        }
        if let Some(purpose) = self.purpose {
            qb.push(" AND purpose = ").push_bind(purpose);
        }
        if let Some(v) = self.min_amount {
            qb.push(" AND amount >= ").push_bind(v);
        }
        if let Some(v) = self.max_amount {
            qb.push(" AND amount <= ").push_bind(v);
        }
        if let Some(ts) = self.created_after {
            qb.push(" AND created_at >= ").push_bind(ts);
        }
        if let Some(ts) = self.created_before {
            qb.push(" AND created_at <= ").push_bind(ts);
        }

        let sort_col = match self.sort_by.as_deref() {
            Some("amount") => "amount",
            _ => "created_at",
        };
        // Payments default to most-recent-first.
        let dir = match self.order.as_deref() {
            Some(o) if o.eq_ignore_ascii_case("asc") => "ASC",
            _ => "DESC",
        };
        qb.push(format!(" ORDER BY {sort_col} {dir}"));
        qb.push(" LIMIT ").push_bind(clamp_limit(self.limit));
        qb.push(" OFFSET ").push_bind(offset(self.offset));
        qb
    }
}

#[derive(Debug, Default, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AdminActionFilter {
    pub account_id: Option<Uuid>,
    /// Action kind, e.g. `AccountDeactivated`, `AccountReactivated`, `MaxDebtUpdated`.
    pub kind: Option<String>,
    pub actor: Option<String>,
    /// RFC3339 timestamp lower bound (inclusive).
    #[serde(default, with = "time::serde::rfc3339::option")]
    #[param(value_type = Option<String>, format = DateTime)]
    pub after: Option<time::OffsetDateTime>,
    /// RFC3339 timestamp upper bound (inclusive).
    #[serde(default, with = "time::serde::rfc3339::option")]
    #[param(value_type = Option<String>, format = DateTime)]
    pub before: Option<time::OffsetDateTime>,
    /// `asc` or `desc` (default `desc`, i.e. newest first).
    pub order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl AdminActionFilter {
    pub fn build(&self) -> QueryBuilder<Postgres> {
        let mut qb = QueryBuilder::new("SELECT * FROM admin_actions WHERE TRUE");
        if let Some(id) = self.account_id {
            qb.push(" AND account_id = ").push_bind(id);
        }
        if let Some(kind) = &self.kind {
            qb.push(" AND kind = ").push_bind(kind.clone());
        }
        if let Some(actor) = &self.actor {
            qb.push(" AND actor = ").push_bind(actor.clone());
        }
        if let Some(ts) = self.after {
            qb.push(" AND timestamp >= ").push_bind(ts);
        }
        if let Some(ts) = self.before {
            qb.push(" AND timestamp <= ").push_bind(ts);
        }
        let dir = match self.order.as_deref() {
            Some(o) if o.eq_ignore_ascii_case("asc") => "ASC",
            _ => "DESC",
        };
        qb.push(format!(" ORDER BY timestamp {dir}"));
        qb.push(" LIMIT ").push_bind(clamp_limit(self.limit));
        qb.push(" OFFSET ").push_bind(offset(self.offset));
        qb
    }
}
