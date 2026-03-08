use super::{get_user_id, verify_character_ownership};
use crate::{
    db::AppState,
    error::{AppError, Result},
    models::character::{
        Character, CharacterHitDice, CharacterSpellSlot, UpdateHitDice, UpdateSpellSlot,
    },
};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Resource Tracking ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateDeathSaves {
    pub successes: Option<i32>,
    pub failures: Option<i32>,
}

// PATCH /characters/:id/death-saves
pub async fn update_death_saves(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateDeathSaves>,
) -> Result<Json<Character>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;

    let updated = sqlx::query!(
        r#"
        UPDATE characters SET
            death_saves_successes = COALESCE($1, death_saves_successes),
            death_saves_failures = COALESCE($2, death_saves_failures),
            updated_at = now()
        WHERE id = $3 AND user_id = $4
        RETURNING id
        "#,
        payload.successes,
        payload.failures,
        id,
        user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Character not found".into()))?;

    let character = sqlx::query_as!(
        Character,
        r#"
        SELECT c.*, cc.class_id
        FROM characters c
        LEFT JOIN character_classes cc ON cc.character_id = c.id AND cc.is_primary = true
        WHERE c.id = $1
        "#,
        updated.id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(character))
}

// GET /characters/:id/spell-slots
pub async fn get_spell_slots(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
) -> Result<Json<Vec<CharacterSpellSlot>>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let slots = sqlx::query_as!(
        CharacterSpellSlot,
        r#"
        SELECT character_id, slot_level, expended
        FROM character_spell_slots
        WHERE character_id = $1 AND expended > 0
        ORDER BY slot_level ASC
        "#,
        character_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(slots))
}

// GET /characters/:id/spell-slots/:level
pub async fn get_spell_slot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, level)): Path<(Uuid, i32)>,
) -> Result<Json<CharacterSpellSlot>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let slot = sqlx::query_as!(
        CharacterSpellSlot,
        r#"
        SELECT character_id, slot_level, expended
        FROM character_spell_slots
        WHERE character_id = $1 AND slot_level = $2
        "#,
        character_id,
        level
    )
    .fetch_optional(&state.db)
    .await?
    .unwrap_or(CharacterSpellSlot {
        character_id,
        slot_level: level,
        expended: 0,
    });

    Ok(Json(slot))
}

// PATCH /characters/:id/spell-slots/:level
pub async fn update_spell_slots(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, level)): Path<(Uuid, i32)>,
    Json(payload): Json<UpdateSpellSlot>,
) -> Result<Json<CharacterSpellSlot>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let row = sqlx::query_as!(
        CharacterSpellSlot,
        r#"
        INSERT INTO character_spell_slots (character_id, slot_level, expended)
        VALUES ($1, $2, $3)
        ON CONFLICT (character_id, slot_level) 
        DO UPDATE SET expended = $3
        RETURNING *
        "#,
        character_id,
        level,
        payload.expended
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(row))
}

// PATCH /characters/:id/hit-dice/:size
pub async fn update_hit_dice(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, size)): Path<(Uuid, i32)>,
    Json(payload): Json<UpdateHitDice>,
) -> Result<Json<CharacterHitDice>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let row = sqlx::query_as!(
        CharacterHitDice,
        r#"
        INSERT INTO character_hit_dice (character_id, die_size, expended)
        VALUES ($1, $2, $3)
        ON CONFLICT (character_id, die_size) 
        DO UPDATE SET expended = $3
        RETURNING *
        "#,
        character_id,
        size,
        payload.expended
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(row))
}

// ── Generic Dynamic Resources ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateResourceUses {
    pub uses_remaining: i32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CharacterResourcePoolRow {
    pub character_id: Uuid,
    pub resource_name: String,
    pub uses_remaining: i32,
}

// PATCH /characters/:id/resources/:resource_name
pub async fn update_resource_uses(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, resource_name)): Path<(Uuid, String)>,
    Json(payload): Json<UpdateResourceUses>,
) -> Result<Json<CharacterResourcePoolRow>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let row = sqlx::query_as!(
        CharacterResourcePoolRow,
        r#"
        INSERT INTO character_resource_pools (character_id, resource_name, uses_remaining)
        VALUES ($1, $2, $3)
        ON CONFLICT (character_id, resource_name)
        DO UPDATE SET uses_remaining = EXCLUDED.uses_remaining
        RETURNING character_id, resource_name, uses_remaining
        "#,
        character_id,
        resource_name,
        payload.uses_remaining
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(row))
}
