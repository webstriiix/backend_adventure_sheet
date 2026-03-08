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
pub struct AddCharacterSpell {
    pub spell_id: i32,
    pub is_prepared: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCharacterSpell {
    pub is_prepared: bool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CharacterSpellRow {
    pub character_id: Uuid,
    pub spell_id: i32,
    pub is_prepared: Option<bool>,
}

// GET /characters/:id/spells
pub async fn list_character_spells(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
) -> Result<Json<Vec<CharacterSpellRow>>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let rows = sqlx::query_as!(
        CharacterSpellRow,
        "SELECT * FROM character_spells WHERE character_id = $1",
        character_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

// POST /characters/:id/spells
pub async fn add_character_spell(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
    Json(payload): Json<AddCharacterSpell>,
) -> Result<Json<CharacterSpellRow>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let row = sqlx::query_as!(
        CharacterSpellRow,
        r#"
        INSERT INTO character_spells (character_id, spell_id, is_prepared)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        character_id,
        payload.spell_id,
        payload.is_prepared.unwrap_or(false),
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(row))
}

// PUT /characters/:id/spells/:spell_id
pub async fn update_character_spell(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, spell_id)): Path<(Uuid, i32)>,
    Json(payload): Json<UpdateCharacterSpell>,
) -> Result<Json<CharacterSpellRow>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let row = sqlx::query_as!(
        CharacterSpellRow,
        r#"
        UPDATE character_spells SET is_prepared = $1
        WHERE character_id = $2 AND spell_id = $3
        RETURNING *
        "#,
        payload.is_prepared,
        character_id,
        spell_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Character spell not found".into()))?;

    Ok(Json(row))
}

// DELETE /characters/:id/spells/:spell_id
pub async fn remove_character_spell(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, spell_id)): Path<(Uuid, i32)>,
) -> Result<StatusCode> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let result = sqlx::query!(
        "DELETE FROM character_spells WHERE character_id = $1 AND spell_id = $2",
        character_id,
        spell_id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Character spell not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}
