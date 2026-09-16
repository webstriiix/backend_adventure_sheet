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
use tracing;
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
    let span = tracing::info_span!("wizard_create_character", %user_id);
    let _guard = span.enter();

    tracing::info!(name = %payload.name, class_id = payload.class_id, "Starting character creation wizard");

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

    tracing::info!(character_id = %char_id, "Saving Wizard Step: Basic Info");

    // Insert class (starting at level 1)
    sqlx::query!(
        "INSERT INTO character_classes (character_id, class_id, level, is_primary) VALUES ($1, $2, 1, true)",
        char_id,
        payload.class_id
    )
    .execute(&mut *tx)
    .await?;

    tracing::info!(character_id = %char_id, "Saving Wizard Step: Class Selection");

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

    // Insert background feat if granted by background
    if let Some(background_id) = payload.background_id {
        let bg = sqlx::query!(
            "SELECT grants_bonus_feat, granted_feat_id FROM backgrounds WHERE id = $1",
            background_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(crate::error::AppError::NotFound(
            "Background not found".into(),
        ))?;

        let feat_id_to_grant = payload.background_feat_id.or(bg.granted_feat_id);

        if let Some(feat_id) = feat_id_to_grant {
            let feat = sqlx::query!(
                "SELECT has_uses, recharge_on FROM feats WHERE id = $1",
                feat_id
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
                feat_id,
                max_uses,
                max_uses,
                feat.recharge_on,
            )
            .execute(&mut *tx)
            .await?;
        } else if bg.grants_bonus_feat.unwrap_or(false) {
            return Err(crate::error::AppError::BadRequest(
                "Selected background grants a bonus feat, but no feat was provided".into(),
            ));
        }
    }

    tracing::info!(character_id = %char_id, "Saving Wizard Step: Background & Feats");

    tx.commit().await?;

    tracing::info!(character_id = %char_id, "Character creation wizard completed successfully");

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

// Helper: D&D 5e XP to Level
fn xp_to_level(xp: i32) -> i32 {
    if xp >= 355000 {
        20
    } else if xp >= 305000 {
        19
    } else if xp >= 265000 {
        18
    } else if xp >= 225000 {
        17
    } else if xp >= 195000 {
        16
    } else if xp >= 165000 {
        15
    } else if xp >= 140000 {
        14
    } else if xp >= 120000 {
        13
    } else if xp >= 100000 {
        12
    } else if xp >= 85000 {
        11
    } else if xp >= 64000 {
        10
    } else if xp >= 48000 {
        9
    } else if xp >= 34000 {
        8
    } else if xp >= 23000 {
        7
    } else if xp >= 14000 {
        6
    } else if xp >= 6500 {
        5
    } else if xp >= 2700 {
        4
    } else if xp >= 900 {
        3
    } else if xp >= 300 {
        2
    } else {
        1
    }
}

// PUT /characters/:id
pub async fn update_character(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCharacter>,
) -> Result<Json<Character>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    let span = tracing::info_span!("wizard_update_character", %id, %user_id);
    let _guard = span.enter();

    tracing::info!(character_id = %id, "Starting character update (wizard step)");

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

    tracing::info!(character_id = %updated.id, "Updated Stats for character {}", updated.id);

    // Update primary class if provided
    if let Some(class_id) = payload.class_id {
        tracing::info!(character_id = %updated.id, class_id, "Saving Wizard Step: Class Selection");

        // Calculate new level from XP
        let new_level = xp_to_level(payload.experience_pts);

        sqlx::query!(
            r#"
            INSERT INTO character_classes (character_id, class_id, level, is_primary)
            VALUES ($1, $2, $3, true)
            ON CONFLICT (character_id, class_id) DO UPDATE SET level = EXCLUDED.level
            "#,
            updated.id,
            class_id,
            new_level
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

    // Update subclass on primary class if provided
    if let Some(subclass_id) = payload.subclass_id {
        tracing::info!(character_id = %updated.id, "Saving Wizard Step: Subclass Selection");
        let primary = sqlx::query!(
            "SELECT class_id, level FROM character_classes WHERE character_id = $1 AND is_primary = true",
            updated.id
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound(
            "Character has no primary class".into(),
        ))?;

        let subclass = sqlx::query!(
            "SELECT unlock_level FROM subclasses WHERE id = $1 AND class_id = $2",
            subclass_id,
            primary.class_id
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound(
            "Subclass not found for this class".into(),
        ))?;

        if primary.level < subclass.unlock_level {
            return Err(AppError::BadRequest(format!(
                "This subclass unlocks at level {}",
                subclass.unlock_level
            )));
        }

        sqlx::query!(
            "UPDATE character_classes SET subclass_id = $1 WHERE character_id = $2 AND is_primary = true",
            subclass_id,
            updated.id
        )
        .execute(&state.db)
        .await?;
    }

    tracing::info!(character_id = %updated.id, "Updated Stats for character");

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
