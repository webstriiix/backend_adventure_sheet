use super::get_user_id;
use crate::{
    db::AppState,
    error::{AppError, Result},
    models::character::{Character, CreateCharacter, UpdateCharacter},
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

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

    if let (Some(race_id), Some(bonus_feat_id)) = (payload.race_id, payload.bonus_feat_id) {
        let race = sqlx::query!("SELECT grants_bonus_feat FROM races WHERE id = $1", race_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(crate::error::AppError::NotFound("Race not found".into()))?;

        if race.grants_bonus_feat.unwrap_or(false) {
            let feat = sqlx::query!(
                "SELECT has_uses, recharge_on FROM feats WHERE id = $1",
                bonus_feat_id
            )
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(crate::error::AppError::NotFound(
                "Bonus feat not found".into(),
            ))?;

            let max_uses = if feat.has_uses { 1 } else { 0 };

            sqlx::query!(
                r#"
                INSERT INTO character_feats 
                    (character_id, feat_id, uses_remaining, uses_max, recharge_on, source_type)
                VALUES ($1, $2, $3, $4, $5, 'race')
                "#,
                char_id,
                bonus_feat_id,
                max_uses,
                max_uses,
                feat.recharge_on,
            )
            .execute(&mut *tx)
            .await?;
        } else {
            return Err(crate::error::AppError::BadRequest(
                "Selected race does not grant a bonus feat".into(),
            ));
        }
    }

    // Insert background feat if granted by background and provided
    if let (Some(background_id), Some(bg_feat_id)) =
        (payload.background_id, payload.background_feat_id)
    {
        let bg = sqlx::query!(
            "SELECT grants_bonus_feat FROM backgrounds WHERE id = $1",
            background_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(crate::error::AppError::NotFound(
            "Background not found".into(),
        ))?;

        if bg.grants_bonus_feat.unwrap_or(false) {
            let feat = sqlx::query!(
                "SELECT has_uses, recharge_on FROM feats WHERE id = $1",
                bg_feat_id
            )
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(crate::error::AppError::NotFound(
                "Background feat not found".into(),
            ))?;

            let max_uses = if feat.has_uses { 1 } else { 0 };

            sqlx::query!(
                r#"
                INSERT INTO character_feats 
                    (character_id, feat_id, uses_remaining, uses_max, recharge_on, source_type)
                VALUES ($1, $2, $3, $4, $5, 'background')
                "#,
                char_id,
                bg_feat_id,
                max_uses,
                max_uses,
                feat.recharge_on,
            )
            .execute(&mut *tx)
            .await?;
        } else {
            return Err(crate::error::AppError::BadRequest(
                "Selected background does not grant a bonus feat".into(),
            ));
        }
    }

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
