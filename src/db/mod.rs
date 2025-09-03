pub mod admin_actions;
pub mod errors;
pub mod account;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

#[derive(Clone, Debug)]
pub struct DatabaseClient {
    pub(crate) pool: PgPool,
}

impl DatabaseClient {
    fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect(uri: &str) -> Option<Self> {
        PgPoolOptions::new().connect(uri).await.map(Self::new).ok()
    }
}
