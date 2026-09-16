#![allow(dead_code)]

use adventure_sheets::{
    config::Config,
    db::{AppState, create_pool, run_migrations},
    routes::all_routes,
    services::auth::{create_token, hash_password},
};
use axum::Router;
use sqlx::PgPool;
use uuid::Uuid;

pub const TEST_JWT_SECRET: &str = "test-secret-key-for-unit-tests-12345";

pub struct TestApp {
    pub pool: PgPool,
    pub router: Router,
    pub config: Config,
}

impl TestApp {
    pub async fn new() -> Self {
        dotenvy::dotenv().ok();
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:password@localhost:5432/adventure_sheet_DB".to_string()
        });

        let pool = create_pool(&db_url).await;
        run_migrations(&pool).await;

        let config = Config {
            database_url: db_url,
            jwt_secret: TEST_JWT_SECRET.to_string(),
            port: 8080,
        };

        let state = AppState {
            db: pool.clone(),
            config: config.clone(),
        };

        let router = all_routes().with_state(state);

        Self {
            pool,
            router,
            config,
        }
    }
}

pub async fn create_test_user(pool: &PgPool, username: &str, secret: &str) -> (Uuid, String) {
    let password_hash = hash_password("password123").expect("Failed to hash password");
    let email = format!("{}@test.com", username);

    let user_id = sqlx::query_scalar!(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        ON CONFLICT (email) DO UPDATE SET username = EXCLUDED.username
        RETURNING id
        "#,
        username,
        email,
        password_hash,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert test user");

    let token = create_token(user_id, secret).expect("Failed to create token");
    (user_id, token)
}

pub async fn create_test_character(pool: &PgPool, user_id: Uuid, name: &str) -> Uuid {
    let char_id = sqlx::query_scalar!(
        r#"
        INSERT INTO characters (
            user_id, name, str, dex, con, int, wis, cha, max_hp, current_hp
        )
        VALUES ($1, $2, 10, 10, 10, 10, 10, 10, 20, 20)
        RETURNING id
        "#,
        user_id,
        name,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert test character");

    char_id
}

pub fn bearer(token: &str) -> String {
    format!("Bearer {}", token)
}
