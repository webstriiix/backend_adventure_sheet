use crate::{
    db::AppState,
    error::{AppError, Result},
    models::character::{Character, CreateCharacter},
    services::auth,
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// Helper: Extract user_id from "Authorization: Bearer <token>"
fn get_user_id(headers: &HeaderMap, secret: &str) -> Result<Uuid> {
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

// GET /characters
pub async fn list_characters(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Character>>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;

    let characters = sqlx::query_as!(
        Character,
        "SELECT * FROM characters WHERE user_id = $1 ORDER BY updated_at DESC",
        user_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(characters))
}

// GET /characters/:id
pub async fn get_character(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Character>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;

    let character = sqlx::query_as!(
        Character,
        "SELECT * FROM characters WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Character not found".into()))?;

    Ok(Json(character))
}

// POST /characters
pub async fn create_character(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateCharacter>,
) -> Result<Json<Character>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;

    let mut tx = state.db.begin().await?;

    let character = sqlx::query_as!(
        Character,
        r#"
        INSERT INTO characters (
            user_id, name, race_id, subrace_id, background_id,
            str, dex, con, int, wis, cha, max_hp, current_hp, temp_hp
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12, 0)
        RETURNING *
        "#,
        user_id,
        payload.name,
        payload.race_id,
        payload.subrace_id,
        payload.background_id,
        payload.str,
        payload.dex,
        payload.con,
        payload.int,
        payload.wis,
        payload.cha,
        payload.max_hp
    )
    .fetch_one(&mut *tx)
    .await?;

    // Insert class (starting at level 1)
    sqlx::query!(
        "INSERT INTO character_classes (character_id, class_id, level, is_primary) VALUES ($1, $2, 1, true)",
        character.id,
        payload.class_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(character))
}

// PUT /characters/:id
pub async fn update_character(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateCharacter>,
) -> Result<Json<Character>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;

    // Check ownership first or just include user_id in UPDATE where clause
    let character = sqlx::query_as!(
        Character,
        r#"
        UPDATE characters SET
            name = $1, race_id = $2, subrace_id = $3, background_id = $4,
            str = $5, dex = $6, con = $7, int = $8, wis = $9, cha = $10,
            max_hp = $11,
            updated_at = now()
        WHERE id = $12 AND user_id = $13
        RETURNING *
        "#,
        payload.name,
        payload.race_id,
        payload.subrace_id,
        payload.background_id,
        payload.str,
        payload.dex,
        payload.con,
        payload.int,
        payload.wis,
        payload.cha,
        payload.max_hp,
        id,
        user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound(
        "Character not found or access denied".into(),
    ))?;

    Ok(Json(character))
}

// DELETE /characters/:id
pub async fn delete_character(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;

    let result = sqlx::query!(
        "DELETE FROM characters WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Character not found or access denied".into(),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── Shared helper ──────────────────────────────────────────────────────

async fn verify_character_ownership(
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

// ── Character Feats ────────────────────────────────────────────────────

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
        feat.uses_formula.as_ref().and_then(|f| f.parse::<i32>().ok())
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

// ── Character Spells ───────────────────────────────────────────────────

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

// ── Character Inventory ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddInventoryItem {
    pub item_id: i32,
    pub quantity: Option<i32>,
    pub is_equipped: Option<bool>,
    pub is_attuned: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInventoryItem {
    pub quantity: Option<i32>,
    pub is_equipped: Option<bool>,
    pub is_attuned: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CharacterInventoryRow {
    pub id: i32,
    pub character_id: Uuid,
    pub item_id: i32,
    pub quantity: Option<i32>,
    pub is_equipped: Option<bool>,
    pub is_attuned: Option<bool>,
    pub notes: Option<String>,
}

// GET /characters/:id/inventory
pub async fn list_character_inventory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
) -> Result<Json<Vec<CharacterInventoryRow>>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let rows = sqlx::query_as!(
        CharacterInventoryRow,
        "SELECT * FROM character_inventory WHERE character_id = $1",
        character_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

// POST /characters/:id/inventory
pub async fn add_inventory_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
    Json(payload): Json<AddInventoryItem>,
) -> Result<Json<CharacterInventoryRow>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let row = sqlx::query_as!(
        CharacterInventoryRow,
        r#"
        INSERT INTO character_inventory (character_id, item_id, quantity, is_equipped, is_attuned, notes)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        character_id,
        payload.item_id,
        payload.quantity.unwrap_or(1),
        payload.is_equipped.unwrap_or(false),
        payload.is_attuned.unwrap_or(false),
        payload.notes,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(row))
}

// PUT /characters/:id/inventory/:inventory_id
pub async fn update_inventory_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, inventory_id)): Path<(Uuid, i32)>,
    Json(payload): Json<UpdateInventoryItem>,
) -> Result<Json<CharacterInventoryRow>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let row = sqlx::query_as!(
        CharacterInventoryRow,
        r#"
        UPDATE character_inventory SET
            quantity = COALESCE($1, quantity),
            is_equipped = COALESCE($2, is_equipped),
            is_attuned = COALESCE($3, is_attuned),
            notes = COALESCE($4, notes)
        WHERE id = $5 AND character_id = $6
        RETURNING *
        "#,
        payload.quantity,
        payload.is_equipped,
        payload.is_attuned,
        payload.notes,
        inventory_id,
        character_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Inventory item not found".into()))?;

    Ok(Json(row))
}

// DELETE /characters/:id/inventory/:inventory_id
pub async fn remove_inventory_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, inventory_id)): Path<(Uuid, i32)>,
) -> Result<StatusCode> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let result = sqlx::query!(
        "DELETE FROM character_inventory WHERE id = $1 AND character_id = $2",
        inventory_id,
        character_id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Inventory item not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}
