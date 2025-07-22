pub mod models;
pub mod routes;
pub mod db;
pub mod errors;

#[tokio::main]
async fn main() {
    let app = routes::api().await.into_make_service();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}