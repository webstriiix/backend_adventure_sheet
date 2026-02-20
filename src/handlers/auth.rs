use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use sqlx::query_as;

use crate::{
    db::AppState,
    error::{AppError, Result},
    models::users::User,
    services::auth,
};

#[derive(Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

pub async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse> {
    // 1. Check if user exists
    let exists = sqlx::query!(
        "SELECT id FROM users WHERE email = $1 OR username = $2",
        payload.email,
        payload.username
    )
    .fetch_optional(&state.db)
    .await?;

    if exists.is_some() {
        return Err(AppError::BadRequest("User already exists".into()));
    }

    // 2. Hash password
    let hash = auth::hash_password(&payload.password)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    // 3. Create user
    let user = query_as!(
        User,
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, username, email, password_hash, created_at
        "#,
        payload.username,
        payload.email,
        hash
    )
    .fetch_one(&state.db)
    .await?;

    // 4. Generate token
    let token = auth::create_token(user.id, &state.config.jwt_secret)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok((StatusCode::CREATED, Json(AuthResponse { token, user })))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse> {
    // 1. Find user
    let user = query_as!(User, "SELECT * FROM users WHERE email = $1", payload.email)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // 2. Verify password
    if !auth::verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::Unauthorized);
    }

    // 3. Generate token
    let token = auth::create_token(user.id, &state.config.jwt_secret)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(Json(AuthResponse { token, user }))
}
