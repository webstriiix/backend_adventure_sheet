use super::{get_user_id, verify_character_ownership};
use crate::{
    db::AppState,
    error::{AppError, Result},
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AddCharacterFeat {
    pub feat_id: i32,
    pub chosen_ability: Option<String>,
    pub source_type: Option<String>,
    pub gained_at_level: Option<i32>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CharacterFeatRow {
    pub id: i32,
    pub character_id: Uuid,
    pub feat_id: i32,
    pub chosen_ability: Option<String>,
    pub uses_remaining: Option<i32>,
    pub uses_max: Option<i32>,
    pub recharge_on: Option<String>,
    pub source_type: String,
    pub gained_at_level: Option<i32>,
}

// GET /characters/:id/feats
pub async fn list_character_feats(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
) -> Result<Json<Vec<CharacterFeatRow>>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let rows = sqlx::query_as!(
        CharacterFeatRow,
        "SELECT * FROM character_feats WHERE character_id = $1",
        character_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

// POST /characters/:id/feats
pub async fn add_character_feat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
    Json(payload): Json<AddCharacterFeat>,
) -> Result<Json<CharacterFeatRow>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    // Look up feat to get uses info
    let feat = sqlx::query!(
        "SELECT has_uses, uses_formula, recharge_on FROM feats WHERE id = $1",
        payload.feat_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Feat not found".into()))?;

    let uses_max = if feat.has_uses {
        feat.uses_formula
            .as_ref()
            .and_then(|f| f.parse::<i32>().ok())
    } else {
        None
    };

    let row = sqlx::query_as!(
        CharacterFeatRow,
        r#"
        INSERT INTO character_feats
            (character_id, feat_id, chosen_ability, uses_remaining, uses_max, recharge_on, source_type, gained_at_level)
        VALUES ($1, $2, $3, $4, $4, $5, $6, $7)
        RETURNING *
        "#,
        character_id,
        payload.feat_id,
        payload.chosen_ability,
        uses_max,
        feat.recharge_on,
        payload.source_type.as_deref().unwrap_or("level"),
        payload.gained_at_level,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(row))
}

// DELETE /characters/:id/feats/:feat_id
pub async fn remove_character_feat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, feat_id)): Path<(Uuid, i32)>,
) -> Result<StatusCode> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let result = sqlx::query!(
        "DELETE FROM character_feats WHERE character_id = $1 AND feat_id = $2",
        character_id,
        feat_id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Character feat not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct UpdateFeatureUses {
    pub uses_remaining: i32,
}

// PATCH /characters/:id/features/:feat_id
pub async fn update_feature_uses(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, feat_id)): Path<(Uuid, i32)>,
    Json(payload): Json<UpdateFeatureUses>,
) -> Result<Json<CharacterFeatRow>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let row = sqlx::query_as!(
        CharacterFeatRow,
        r#"
        UPDATE character_feats SET uses_remaining = $1
        WHERE character_id = $2 AND feat_id = $3
        RETURNING *
        "#,
        payload.uses_remaining,
        character_id,
        feat_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Character feat not found".into()))?;

    Ok(Json(row))
}
