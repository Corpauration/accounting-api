use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "account_status")]
pub enum AccountStatus {
    #[sqlx(rename = "ACTIVE")]
    Active,
    #[sqlx(rename = "DEACTIVATED")]
    Deactivated,
    #[sqlx(rename = "DELETED")]
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub owner_id: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub labels: HashMap<String, String>,
    pub max_allowed_debt: i32,
    pub balance: i32,
    pub status: AccountStatus,
}

impl Account {
    pub fn new(
        id: Uuid,
        owner_id: String,
        name: String,
        description: Option<String>,
        labels: HashMap<String, String>,
        tags: Vec<String>,
        max_allowed_debt: i32,
        balance: i32,
    ) -> Self {
        Self {
            id,
            owner_id,
            name,
            description,
            tags,
            labels,
            max_allowed_debt,
            balance,
            status: AccountStatus::Active,
        }
    }

    fn new_with_status(
        id: Uuid,
        owner_id: String,
        name: String,
        description: Option<String>,
        labels: HashMap<String, String>,
        tags: Vec<String>,
        max_allowed_debt: i32,
        balance: i32,
        status: AccountStatus,
    ) -> Self {
        Self {
            id,
            owner_id,
            name,
            description,
            tags,
            labels,
            max_allowed_debt,
            balance,
            status,
        }
    }
}
