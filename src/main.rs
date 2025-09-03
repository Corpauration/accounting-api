pub mod models;
pub mod routes;
pub mod db;
pub mod errors;
use std::path::Path;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};
use crate::db::DatabaseClient;
use dotenv::dotenv;

pub fn env_get(env: &'static str) -> String {
    let env_panic = |e| {
        error!("{env} is not set ({})", e);
        std::process::exit(1);
    };

    std::env::var(env).map_err(env_panic).unwrap()
}


#[tokio::main]
async fn main() {
    dotenv().ok();
    let postgresql_uri = env_get("POSTGRESQL_ADDON_URI");
    info!("Connecting to database");

    // Create a connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgresql_uri)
        .await
        .expect("Failed to create pool");

    // Run migrations
    let migrator = Migrator::new(Path::new("migrations")).await.unwrap();
    migrator.run(&pool).await.unwrap();
    info!("Database migrations applied");

    
    let db = match DatabaseClient::connect(&postgresql_uri).await {
        Some(db) => {
            info!("Connected to database!");
            db
        }
        None => {
            error!("Failed to connect to database");
            std::process::exit(1);
        }
    };
    let app = routes::api(axum::extract::State(db)).await.into_make_service();
    // let app = api::app(
    //     ApiHandlerState::new(ApiHandler {
    //         db,
    //         sessions: Arc::new(RwLock::new(HashMap::new())),
    //     })
    // );
    // _ = db.clean_invitations().await;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}