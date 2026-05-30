use utoipa::OpenApi;

/// Root OpenAPI document. Path items and their component schemas are collected
/// automatically from the per-slice `OpenApiRouter`s in `routes::api`.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Accounting API",
        version = "0.1.0",
        description = "Accounts, append-only ledger operations, admin actions and payments."
    ),
    tags(
        (name = "accounts", description = "Account lifecycle and metadata"),
        (name = "operations", description = "Append-only ledger (CREDIT/DEBIT)"),
        (name = "account management", description = "Account administrative actions and shortcuts"),
        (name = "audit", description = "Audit log of admin actions, filterable"),
        (name = "payments", description = "Payments, their state machine and audit log")
    )
)]
pub struct ApiDoc;
