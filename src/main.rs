use accounting_api::db::DatabaseClient;
use accounting_api::routes;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};

/// Read a required environment variable, exiting with a clear message if unset.
fn env_get(env: &'static str) -> String {
    std::env::var(env).unwrap_or_else(|e| {
        error!("{env} is not set ({e})");
        std::process::exit(1);
    })
}

/// Read an optional environment variable, falling back to `default`.
fn env_or(env: &str, default: &str) -> String {
    std::env::var(env).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let postgresql_uri = env_get("POSTGRESQL_ADDON_URI");
    let host = env_or("HOST", "0.0.0.0");
    let port = env_or("PORT", "3000");
    let addr = format!("{host}:{port}");

    info!("Connecting to database");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgresql_uri)
        .await
        .expect("Failed to create database pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    info!("Database migrations applied");

    let db = DatabaseClient::new(pool);
    let app = routes::api(db);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind {addr}: {e}"));
    info!("Server running on http://{addr}");
    axum::serve(listener, app).await.expect("Server error");
}
