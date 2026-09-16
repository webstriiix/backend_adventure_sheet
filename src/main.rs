use adventure_sheets::{config, db, routes};
use axum::{Router, extract::DefaultBodyLimit};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    db::run_migrations(&pool).await;

    let state = db::AppState { db: pool, config: config.clone() };

    let app = Router::new()
        .nest("/api/v1", routes::all_routes())
        .with_state(state.clone())
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024)) // 5mb
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", state.config.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app).await.unwrap();
}
