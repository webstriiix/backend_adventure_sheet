pub mod actions;
pub mod asi;
pub mod classes;
pub mod core;
pub mod feats;
pub mod inventory;
pub mod resources;
pub mod rests;
pub mod spells;

pub use actions::*;
pub use asi::*;
pub use classes::*;
pub use core::*;
pub use feats::*;
pub use inventory::*;
pub use resources::*;
pub use rests::*;
pub use spells::*;

use crate::{
    error::{AppError, Result},
    services::auth,
};
use axum::http::HeaderMap;
use sqlx::PgPool;
use uuid::Uuid;

/// Extract user_id from "Authorization: Bearer <token>"
pub(super) fn get_user_id(headers: &HeaderMap, secret: &str) -> Result<Uuid> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    let claims = auth::decode_token(token, secret).map_err(|_| AppError::Unauthorized)?;

    Ok(Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?)
}

pub(super) async fn verify_character_ownership(
    pool: &PgPool,
    character_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    let exists = sqlx::query!(
        "SELECT id FROM characters WHERE id = $1 AND user_id = $2",
        character_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    if exists.is_none() {
        return Err(AppError::NotFound(
            "Character not found or access denied".into(),
        ));
    }
    Ok(())
}
