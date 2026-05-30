use sqlx::types::Json;
use uuid::Uuid;

use crate::db::DatabaseClient;
use crate::errors::AccountingError;
use crate::models::account::{
    Account, AccountStatus, CreateAccountRequest, MetadataRequest, PatchAccountRequest,
};

impl DatabaseClient {
    /// Filtered listing. Dynamic `WHERE` clauses require the runtime
    /// `QueryBuilder` (sqlx's compile-time macros can't express them).
    pub async fn list_accounts(
        &self,
        filter: &crate::filters::AccountFilter,
    ) -> Result<Vec<Account>, AccountingError> {
        let accounts = filter
            .build()
            .build_query_as::<Account>()
            .fetch_all(&self.pool)
            .await?;
        Ok(accounts)
    }

    pub async fn get_account_by_id(
        &self,
        account_id: Uuid,
    ) -> Result<Option<Account>, AccountingError> {
        let account = sqlx::query_as!(
            Account,
            r#"
            SELECT id, owner_id, name, description, tags,
                   labels as "labels: Json<std::collections::HashMap<String, String>>",
                   max_allowed_debt, balance,
                   status as "status: AccountStatus",
                   created_at, updated_at
            FROM accounts
            WHERE id = $1
            "#,
            account_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(account)
    }

    pub async fn create_account(
        &self,
        req: CreateAccountRequest,
    ) -> Result<Account, AccountingError> {
        let account = sqlx::query_as!(
            Account,
            r#"
            INSERT INTO accounts (id, owner_id, name, description, tags, labels, max_allowed_debt, balance)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 0)
            RETURNING id, owner_id, name, description, tags,
                      labels as "labels: Json<std::collections::HashMap<String, String>>",
                      max_allowed_debt, balance,
                      status as "status: AccountStatus",
                      created_at, updated_at
            "#,
            Uuid::new_v4(),
            req.owner_id,
            req.name,
            req.description,
            &req.tags,
            Json(req.labels) as _,
            req.max_allowed_debt,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(account)
    }

    pub async fn patch_account(
        &self,
        account_id: Uuid,
        req: PatchAccountRequest,
    ) -> Result<Option<Account>, AccountingError> {
        // Static query with COALESCE: balance, status and max_allowed_debt are
        // intentionally not patchable here (operations / admin-actions own them).
        let account = sqlx::query_as!(
            Account,
            r#"
            UPDATE accounts
            SET name = COALESCE($2::text, name),
                description = COALESCE($3::text, description),
                tags = COALESCE($4::text[], tags),
                updated_at = now()
            WHERE id = $1
            RETURNING id, owner_id, name, description, tags,
                      labels as "labels: Json<std::collections::HashMap<String, String>>",
                      max_allowed_debt, balance,
                      status as "status: AccountStatus",
                      created_at, updated_at
            "#,
            account_id,
            req.name,
            req.description,
            req.tags.as_deref(),
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(account)
    }

    pub async fn set_account_metadata(
        &self,
        account_id: Uuid,
        req: MetadataRequest,
    ) -> Result<Option<Account>, AccountingError> {
        let account = sqlx::query_as!(
            Account,
            r#"
            UPDATE accounts
            SET labels = $2, tags = $3, updated_at = now()
            WHERE id = $1
            RETURNING id, owner_id, name, description, tags,
                      labels as "labels: Json<std::collections::HashMap<String, String>>",
                      max_allowed_debt, balance,
                      status as "status: AccountStatus",
                      created_at, updated_at
            "#,
            account_id,
            Json(req.labels) as _,
            &req.tags,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(account)
    }

    pub async fn delete_account(
        &self,
        account_id: Uuid,
    ) -> Result<Option<Account>, AccountingError> {
        let account = sqlx::query_as!(
            Account,
            r#"
            UPDATE accounts
            SET status = 'DELETED', updated_at = now()
            WHERE id = $1
            RETURNING id, owner_id, name, description, tags,
                      labels as "labels: Json<std::collections::HashMap<String, String>>",
                      max_allowed_debt, balance,
                      status as "status: AccountStatus",
                      created_at, updated_at
            "#,
            account_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(account)
    }
}
