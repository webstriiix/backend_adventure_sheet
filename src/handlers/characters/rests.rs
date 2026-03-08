use super::{get_user_id, verify_character_ownership};
use crate::{
    db::AppState,
    error::Result,
    models::character::{Character, ShortRestRequest},
};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use uuid::Uuid;

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
