use crate::models::account::Account;
use uuid::Uuid;
use super::DatabaseClient;

impl DatabaseClient {
    pub async fn accounts_handler_get(&self) -> Result<Vec<Account>, sqlx::Error> {
        let accounts = sqlx::query_as!(Account, "SELECT * FROM accounts",).fetch_all(&self.pool).await?;
        Ok(accounts)
    }
    pub async fn account_handler_get_by_id(&self, account_id: Uuid) -> Result<Vec<Account>, sqlx::Error> {
        let account = sqlx::query_as!(Account, "SELECT * FROM accounts WHERE id=$1", account_id).fetch_all(&self.pool).await?;
        Ok(account)
    }
    pub async fn create_account_handler(&self, account: Account) -> Result<Account, sqlx::Error> {
        let new_account = sqlx::query_as!(Account, "
            INSERT INTO account (
                id,
                owner_id,
                name,
                description,
                tags,
                labels,
                max_allowed_debt,
                balance,
                status
            ) 
            VALUES ($2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING id, owner_id, name, description, tags, labels, max_allowed_debt, balance, status",
            account.id,
            account.owner_id,
            account.name,
            account.description,
            account.tags,
            account.labels,
            accoun.max_allowed_debt,
            account.balance,
            account.status
        ).fetch_all(&self.pool).await?;
        Ok(account)
    }
    pub async fn account_handler_delete(&self, account_id: Uuid) -> Result<Vec<Account>, sqlx::Error> {
        let account = sqlx::query_as!(Account, "
            UPDATE account
            SET status = DELETED
            WHERE id = $1
            RETURNING id, owner_id, name, description, tags, labels, max_allowed_debt, balance, status",
            account_id
        ).fetch_all(&self.pool).await?;
        Ok(account)
    }
    pub async fn account_handler_patch(&self, account: Account) -> Result<Account, sqlx::Error> {
        let updated_account = sqlx::query_as!(Account, "
            UPDATE account
            SET name = $2,
                description = $3,
                tags = $4,
                labels = $5,
                max_allowed_debt = $6,
                balance = $7,
                status = $8
            WHERE id = $1 RETURNING id, owner_id, name, description, tags, labels, max_allowed_debt, balance, status",
            account.id,
            account.name,
            account.description,
            account.tags,
            account.labels,
            account.max_allowed_debt,
            account.balance,
            account.status
        )
        .fetch_all(&self.pool).await?;
        Ok(updated_account)
    }
    // pub async fn put_account_metadata(&self, account_id: Uuid, metadata: serde_json::Value) -> Result<Account, sqlx::Error> {
    // }
}