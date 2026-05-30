pub mod account;
pub mod admin_actions;
pub mod errors;
pub mod operation;
pub mod payment;

use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct DatabaseClient {
    pub(crate) pool: PgPool,
}

impl DatabaseClient {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
