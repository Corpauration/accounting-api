use uuid::Uuid;
use crate::db::DatabaseClient;
use crate::models::account::Account;

impl DatabaseClient {
    pub async fn get_accounts(&self) -> Result<Vec<Account>, sqlx::Error> {
        let accounts = sqlx::query_as::<_, Account>("SELECT * FROM accounts")
            .fetch_all(&self.pool)
            .await?;
        Ok(accounts)
    }
    pub async fn get_account_by_id(&self, account_id: Uuid) -> Result<Vec<Account>, sqlx::Error> {
        let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(account)
    }
    pub async fn create_account(&self, account: Account) -> Result<Account, sqlx::Error> {
        let new_account = sqlx::query_as::<_, Account>(
            "INSERT INTO accounts (
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
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING id, owner_id, name, description, tags, labels, max_allowed_debt, balance, status"
        )
        .bind(account.id)
        .bind(account.owner_id)
        .bind(account.name)
        .bind(account.description)
        .bind(account.tags)
        .bind(account.labels)
        .bind(account.max_allowed_debt)
        .bind(account.balance)
        .bind(account.status)
        .fetch_one(&self.pool)
        .await?;
        Ok(new_account)
    }
    pub async fn delete_account(&self, account_id: Uuid) -> Result<Vec<Account>, sqlx::Error> {
        let account = sqlx::query_as::<_, Account>(
            "UPDATE accounts
            SET status = 'DELETED'
            WHERE id = $1
            RETURNING id, owner_id, name, description, tags, labels, max_allowed_debt, balance, status"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(account)
    }
    pub async fn patch_account(&self, account: Account) -> Result<Account, sqlx::Error> {
        let updated_account = sqlx::query_as::<_, Account>(
            "UPDATE accounts
            SET name = $2,
                description = $3,
                tags = $4,
                labels = $5,
                max_allowed_debt = $6,
                balance = $7,
                status = $8
            WHERE id = $1 RETURNING id, owner_id, name, description, tags, labels, max_allowed_debt, balance, status"
        )
        .bind(account.id)
        .bind(account.name)
        .bind(account.description)
        .bind(account.tags)
        .bind(account.labels)
        .bind(account.max_allowed_debt)
        .bind(account.balance)
        .bind(account.status)
        .fetch_one(&self.pool)
        .await?;
        Ok(updated_account)
    }

    // pub async fn put_account_metadata(Path(id): Path<u64>) -> String {
    //     format!("Put metadata for account {id}")
    // }

    // pub async fn get_account_operations(Path(id): Path<u64>) -> String {
    //     format!("Get operations for account {id}")
    // }

    // pub async fn put_account_operations(Path(id): Path<u64>) -> String {
    //     format!("Put operations for account {id}")
    // }

    // pub async fn post_account_operations(Path(id): Path<u64>) -> String {
    //     format!("Post operations for account {id}")
    // }

    // `GET /accounts/{id}/operations`
    // - `GET /accounts/{id}/operations/{operation_id}`
    // - `POST /accounts/{id}/operations`
    // - 
    // - `GET /accounts/{id}/admin-actions`
    // - `GET /accounts/{id}/admin-actions/{admin_action_id}`
    // - `POST /accounts/{id}/admin-actions`
}