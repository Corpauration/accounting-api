pub mod models;
pub mod routes;
pub mod db;
pub mod errors;
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
    let app = routes::api().await.into_make_service();
    let postgresql_uri = env_get("POSTGRESQL_ADDON_URI");
    info!("Connecting to database");
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}