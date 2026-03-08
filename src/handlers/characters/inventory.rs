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
