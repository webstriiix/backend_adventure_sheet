use axum::{extract::{Path, State}, Json, http::HeaderMap};
use crate::{db::AppState, error::{AppError, Result}};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;
use crate::models::race_options::RaceOption;

#[derive(Debug, Deserialize)]
pub struct CreateCharacterRaceOption {
    pub race_option_id: i32,
    pub selection: Value,
}

// POST /characters/{id}/race-options
pub async fn add_character_race_option(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateCharacterRaceOption>,
) -> Result<Json<crate::models::race_options::CharacterRaceOption>> {
    let user_id = super::get_user_id(&headers, &state.config.jwt_secret)?;

    // verify ownership
    super::verify_character_ownership(&state.db, id, user_id).await?;

    // fetch race_option and validate
    let ro = sqlx::query_as!(
        RaceOption,
        "SELECT * FROM race_options WHERE id = $1",
        payload.race_option_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Race option not found".into()))?;

    // If race_id or subrace_id set, ensure they match the character's race/subrace
    if ro.race_id.is_some() || ro.subrace_id.is_some() {
        let char_row = sqlx::query!(
            "SELECT race_id, subrace_id FROM characters WHERE id = $1",
            id
        )
        .fetch_one(&state.db)
        .await?;

        if let Some(rid) = ro.race_id {
            if char_row.race_id != Some(rid) {
                return Err(AppError::BadRequest("Race option does not belong to this character's race".into()));
            }
        }
        if let Some(sid) = ro.subrace_id {
            if char_row.subrace_id != Some(sid) {
                return Err(AppError::BadRequest("Race option does not belong to this character's subrace".into()));
            }
        }
    }

    // Determine selection count (single object -> 1, array -> length)
    let selection_count: i32 = if let Some(arr) = payload.selection.as_array() {
        arr.len() as i32
    } else {
        1
    };

    // Enforce min_choose / max_choose
    if selection_count < ro.min_choose || selection_count > ro.max_choose {
        return Err(AppError::BadRequest(format!(
            "Selection must contain between {} and {} items",
            ro.min_choose, ro.max_choose
        )));
    }

    // Validate selection against choices if provided
    if let Some(choices) = &ro.choices {
        // choices expected to be an array; ensure payload.selection items are in choices
        if !choices.is_null() {
            if let Some(choices_arr) = choices.as_array() {
                if let Some(sel_arr) = payload.selection.as_array() {
                    for sel in sel_arr {
                        if !choices_arr.iter().any(|c| c == sel) {
                            return Err(AppError::BadRequest("Selection not in allowed choices".into()));
                        }
                    }
                } else {
                    if !choices_arr.iter().any(|c| c == &payload.selection) {
                        return Err(AppError::BadRequest("Selection not in allowed choices".into()));
                    }
                }
            }
        }
    }

    // Upsert into character_race_options and return the persisted row
    let cro = sqlx::query_as!(
        crate::models::race_options::CharacterRaceOption,
        r#"
        INSERT INTO character_race_options (character_id, race_option_id, selection)
        VALUES ($1, $2, $3)
        ON CONFLICT (character_id, race_option_id) DO UPDATE SET selection = EXCLUDED.selection
        RETURNING id, character_id, race_option_id, selection
        "#,
        id,
        payload.race_option_id,
        payload.selection
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(cro))
}
