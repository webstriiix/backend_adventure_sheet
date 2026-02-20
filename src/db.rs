use sqlx::PgPool;
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config
}

pub async fn create_pool(database_url: &str) -> PgPool{
    sqlx::PgPool::connect(database_url)
        .await
        .expect("Failed to connect to Database!!!")
}
