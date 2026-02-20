use axum::{Router, extract::DefaultBodyLimit};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod handlers;
mod importers;
mod models;
mod routes;
mod services;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env();
    let pool = db::create_pool(&config.database_url).await;

    // Run migrations automatically
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migrations failed!!!");

    let state = db::AppState { db: pool, config };

    let app = Router::new()
        .nest("/api/v1", routes::all_routes())
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024)); // 5mb

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("Listening on {addr}");

    axum::serve(listener, app).await.unwrap();
}
