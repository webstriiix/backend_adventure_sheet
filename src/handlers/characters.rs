use crate::{
    db::AppState,
    error::{AppError, Result},
    models::character::{
        Character, CharacterHitDice, CharacterSpellSlot, CreateCharacter, ShortRestRequest,
        UpdateCharacter, UpdateHitDice, UpdateSpellSlot,
    },
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
        r#"
        SELECT c.*, cc.class_id
        FROM characters c
        LEFT JOIN character_classes cc ON cc.character_id = c.id AND cc.is_primary = true
        WHERE c.user_id = $1
        ORDER BY c.updated_at DESC
        "#,
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
        r#"
        SELECT c.*, cc.class_id
        FROM characters c
        LEFT JOIN character_classes cc ON cc.character_id = c.id AND cc.is_primary = true
        WHERE c.id = $1 AND c.user_id = $2
        "#,
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

    let row = sqlx::query!(
        r#"
        INSERT INTO characters (
            user_id, name, race_id, subrace_id, background_id,
            str, dex, con, int, wis, cha, max_hp, current_hp, temp_hp,
            death_saves_successes, death_saves_failures,
            cp, sp, ep, gp, pp
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12, 0,
                0, 0, 0, 0, 0, 0, 0)
        RETURNING id
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

    let char_id = row.id;

    // Insert class (starting at level 1)
    sqlx::query!(
        "INSERT INTO character_classes (character_id, class_id, level, is_primary) VALUES ($1, $2, 1, true)",
        char_id,
        payload.class_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let character = sqlx::query_as!(
        Character,
        r#"
        SELECT c.*, cc.class_id
        FROM characters c
        LEFT JOIN character_classes cc ON cc.character_id = c.id AND cc.is_primary = true
        WHERE c.id = $1
        "#,
        char_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(character))
}

// PUT /characters/:id
pub async fn update_character(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCharacter>,
) -> Result<Json<Character>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;

    let updated = sqlx::query!(
        r#"
        UPDATE characters SET
            name = $1, race_id = $2, subrace_id = $3, background_id = $4,
            str = $5, dex = $6, con = $7, int = $8, wis = $9, cha = $10,
            max_hp = $11, experience_pts = $12, current_hp = $13, temp_hp = $14,
            inspiration = COALESCE($15, inspiration), notes = COALESCE($16, notes),
            death_saves_successes = COALESCE($17, death_saves_successes),
            death_saves_failures = COALESCE($18, death_saves_failures),
            cp = COALESCE($19, cp),
            sp = COALESCE($20, sp),
            ep = COALESCE($21, ep),
            gp = COALESCE($22, gp),
            pp = COALESCE($23, pp),
            updated_at = now()
        WHERE id = $24 AND user_id = $25
        RETURNING id
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
        payload.experience_pts,
        payload.current_hp,
        payload.temp_hp,
        payload.inspiration,
        payload.notes,
        payload.death_saves_successes,
        payload.death_saves_failures,
        payload.cp,
        payload.sp,
        payload.ep,
        payload.gp,
        payload.pp,
        id,
        user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound(
        "Character not found or access denied".into(),
    ))?;

    // Update primary class if provided
    if let Some(class_id) = payload.class_id {
        sqlx::query!(
            r#"
            INSERT INTO character_classes (character_id, class_id, level, is_primary)
            VALUES ($1, $2, 1, true)
            ON CONFLICT (character_id, class_id) DO NOTHING
            "#,
            updated.id,
            class_id
        )
        .execute(&state.db)
        .await?;

        // Mark this class as primary, others as non-primary
        sqlx::query!(
            "UPDATE character_classes SET is_primary = (class_id = $2) WHERE character_id = $1",
            updated.id,
            class_id
        )
        .execute(&state.db)
        .await?;
    }

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

// ── Rests ──────────────────────────────────────────────────────────────

// POST /characters/:id/short-rest
pub async fn short_rest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
    Json(payload): Json<ShortRestRequest>,
) -> Result<Json<Character>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let mut tx = state.db.begin().await?;

    // Roll/spend hit dice based on payload
    for (die_size, spent) in payload.hit_dice_spent {
        sqlx::query!(
            r#"
            INSERT INTO character_hit_dice (character_id, die_size, expended)
            VALUES ($1, $2, $3)
            ON CONFLICT (character_id, die_size) 
            DO UPDATE SET expended = character_hit_dice.expended + $3
            "#,
            character_id,
            die_size,
            spent
        )
        .execute(&mut *tx)
        .await?;
    }

    // Refresh Short Rest features
    sqlx::query!(
        r#"
        UPDATE character_feats 
        SET uses_remaining = uses_max 
        WHERE character_id = $1 AND recharge_on = 'short_rest'
        "#,
        character_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let character = sqlx::query_as!(
        Character,
        r#"
        SELECT c.*, cc.class_id
        FROM characters c
        LEFT JOIN character_classes cc ON cc.character_id = c.id AND cc.is_primary = true
        WHERE c.id = $1
        "#,
        character_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(character))
}

// POST /characters/:id/long-rest
pub async fn long_rest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
) -> Result<Json<Character>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let mut tx = state.db.begin().await?;

    sqlx::query!(
        r#"
        UPDATE characters 
        SET current_hp = max_hp, death_saves_successes = 0, death_saves_failures = 0
        WHERE id = $1
        "#,
        character_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE character_spell_slots SET expended = 0 WHERE character_id = $1",
        character_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        UPDATE character_feats 
        SET uses_remaining = uses_max 
        WHERE character_id = $1 AND (recharge_on = 'long_rest' OR recharge_on = 'short_rest' OR recharge_on = 'dawn')
        "#,
        character_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE character_hit_dice SET expended = 0 WHERE character_id = $1",
        character_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let character = sqlx::query_as!(
        Character,
        r#"
        SELECT c.*, cc.class_id
        FROM characters c
        LEFT JOIN character_classes cc ON cc.character_id = c.id AND cc.is_primary = true
        WHERE c.id = $1
        "#,
        character_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(character))
}
