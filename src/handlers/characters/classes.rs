use super::{get_user_id, verify_character_ownership};
use crate::{
    db::AppState,
    error::{AppError, Result},
    models::character::{AddClassRequest, Character, UpdateClassLevelRequest},
};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use uuid::Uuid;

// POST /characters/:id/classes
pub async fn add_character_class(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
    Json(payload): Json<AddClassRequest>,
) -> Result<Json<Character>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

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

    // Verify Prereqs
    let target_class = sqlx::query!(
        "SELECT multiclass_requirements FROM classes WHERE id = $1",
        payload.class_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Class not found".into()))?;

    if let Some(reqs) = target_class.multiclass_requirements {
        let reqs_map: std::collections::HashMap<String, i32> = serde_json::from_value(reqs)
            .map_err(|_| {
                AppError::Internal(anyhow::anyhow!("Failed to parse class requirements"))
            })?;

        for (ability, min_score) in reqs_map {
            let passed = match ability.as_str() {
                "str" => character.str >= min_score,
                "dex" => character.dex >= min_score,
                "con" => character.con >= min_score,
                "int" => character.int >= min_score,
                "wis" => character.wis >= min_score,
                "cha" => character.cha >= min_score,
                _ => true,
            };

            if !passed {
                return Err(AppError::BadRequest(format!(
                    "Does not meet multiclass requirement: {} {}",
                    ability, min_score
                )));
            }
        }
    }

    sqlx::query!(
        r#"
        INSERT INTO character_classes (character_id, class_id, level, is_primary)
        VALUES ($1, $2, 1, false)
        ON CONFLICT (character_id, class_id) DO NOTHING
        "#,
        character_id,
        payload.class_id
    )
    .execute(&state.db)
    .await?;

    // Refetch in case they needed to see updated logic
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

// PATCH /characters/:id/classes/:class_id
pub async fn update_character_class(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, class_id)): Path<(Uuid, i32)>,
    Json(payload): Json<UpdateClassLevelRequest>,
) -> Result<Json<Character>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    // Prevent leveling past 20 total
    let total_level_record = sqlx::query!(
        "SELECT SUM(level) as total_level FROM character_classes WHERE character_id = $1 AND class_id != $2",
        character_id,
        class_id
    )
    .fetch_one(&state.db)
    .await?;

    let other_levels: i64 = total_level_record.total_level.unwrap_or(0);
    if other_levels + (payload.level as i64) > 20 {
        return Err(AppError::BadRequest(
            "Total character level cannot exceed 20".into(),
        ));
    }

    // Verify subclass if provided
    if let Some(sub_id) = payload.subclass_id {
        let subclass = sqlx::query!(
            "SELECT unlock_level FROM subclasses WHERE id = $1 AND class_id = $2",
            sub_id,
            class_id
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound(
            "Subclass not found for this class".into(),
        ))?;

        if payload.level < subclass.unlock_level {
            return Err(AppError::BadRequest(format!(
                "This subclass unlocks at level {}",
                subclass.unlock_level
            )));
        }
    }

    let result = sqlx::query!(
        r#"
        UPDATE character_classes 
        SET level = $1, subclass_id = $2
        WHERE character_id = $3 AND class_id = $4
        "#,
        payload.level,
        payload.subclass_id,
        character_id,
        class_id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Character does not have this class".into(),
        ));
    }

    // Return the updated character
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
