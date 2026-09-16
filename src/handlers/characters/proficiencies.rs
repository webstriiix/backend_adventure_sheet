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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CharacterProficiencyRow {
    pub id: i32,
    pub character_id: Uuid,
    pub category: String,
    pub name: String,
    pub proficiency_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AddCharacterProficiency {
    pub category: String,
    pub name: String,
    pub proficiency_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCharacterProficiency {
    pub proficiency_type: String,
}

// GET /characters/:id/proficiencies
pub async fn list_character_proficiencies(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
) -> Result<Json<Vec<CharacterProficiencyRow>>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let rows = sqlx::query_as!(
        CharacterProficiencyRow,
        "SELECT * FROM character_proficiencies WHERE character_id = $1 ORDER BY category, name",
        character_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

// POST /characters/:id/proficiencies
pub async fn add_character_proficiency(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
    Json(payload): Json<AddCharacterProficiency>,
) -> Result<Json<CharacterProficiencyRow>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let category = payload.category.trim().to_lowercase();
    let name = payload.name.trim().to_lowercase();
    let prof_type = payload
        .proficiency_type
        .as_deref()
        .unwrap_or("proficiency")
        .trim()
        .to_lowercase();

    if category != "saving_throw" && category != "skill" {
        return Err(AppError::BadRequest(
            "category must be 'saving_throw' or 'skill'".into(),
        ));
    }

    if name.is_empty() {
        return Err(AppError::BadRequest("name must not be empty".into()));
    }

    if prof_type != "proficiency" && prof_type != "expertise" {
        return Err(AppError::BadRequest(
            "proficiency_type must be 'proficiency' or 'expertise'".into(),
        ));
    }

    // === VALIDASI REPLACEMENT 2024: Cek duplikasi skill ===
    if category == "skill" {
        let existing = sqlx::query!(
            "SELECT id, name, proficiency_type FROM character_proficiencies \
                 WHERE character_id = $1 AND category = 'skill' AND lower(name) = lower($2)",
            character_id,
            &name
        )
        .fetch_optional(&state.db)
        .await?;

        if let Some(existing_row) = existing {
            return Err(AppError::Conflict(format!(
                "DUPLICATE_SKILL_REPLACEMENT_REQUIRED: Skill '{}' sudah dimiliki (type: {}). Pilih skill lain sebagai replacement sesuai aturan 2024.",
                existing_row.name, existing_row.proficiency_type
            )));
        }
    }

    let row = sqlx::query_as!(
        CharacterProficiencyRow,
        r#"
        INSERT INTO character_proficiencies (character_id, category, name, proficiency_type)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (character_id, category, name)
        DO UPDATE SET proficiency_type = EXCLUDED.proficiency_type
        RETURNING id, character_id, category, name, proficiency_type
        "#,
        character_id,
        category,
        name,
        prof_type,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(row))
}

// GET /characters/:id/proficiencies/skills
pub async fn list_character_skills_only(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
) -> Result<Json<Vec<String>>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let skills = sqlx::query_scalar!(
        "SELECT lower(name) as \"name!\" FROM character_proficiencies \
         WHERE character_id = $1 AND category = 'skill' \
         ORDER BY name",
        character_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(skills))
}

// POST /characters/:id/proficiencies/batch
pub async fn add_character_proficiencies_batch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
    Json(payloads): Json<Vec<AddCharacterProficiency>>,
) -> Result<Json<Vec<CharacterProficiencyRow>>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let mut results = Vec::new();

    for payload in payloads {
        let category = payload.category.trim().to_lowercase();
        let name = payload.name.trim().to_lowercase();
        let prof_type = payload
            .proficiency_type
            .as_deref()
            .unwrap_or("proficiency")
            .trim()
            .to_lowercase();

        // Validasi input
        if category != "saving_throw" && category != "skill" {
            continue;
        }

        if name.is_empty() {
            continue;
        }

        if prof_type != "proficiency" && prof_type != "expertise" {
            continue;
        }

        // Skip duplikat skill (replacement ditangani frontend)
        if category == "skill" {
            let existing = sqlx::query!(
                "SELECT id FROM character_proficiencies \
                 WHERE character_id = $1 AND category = 'skill' AND lower(name) = lower($2)",
                character_id,
                &name
            )
            .fetch_optional(&state.db)
            .await?;

            if existing.is_some() {
                continue;
            }
        }

        let row = sqlx::query_as!(
            CharacterProficiencyRow,
            r#"
            INSERT INTO character_proficiencies (character_id, category, name, proficiency_type)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (character_id, category, name)
            DO UPDATE SET proficiency_type = EXCLUDED.proficiency_type
            RETURNING id, character_id, category, name, proficiency_type
            "#,
            character_id,
            category,
            name,
            prof_type,
        )
        .fetch_one(&state.db)
        .await?;

        results.push(row);
    }

    Ok(Json(results))
}

// PATCH /characters/:id/proficiencies/:prof_id
pub async fn update_character_proficiency(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, prof_id)): Path<(Uuid, i32)>,
    Json(payload): Json<UpdateCharacterProficiency>,
) -> Result<Json<CharacterProficiencyRow>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let prof_type = payload.proficiency_type.trim().to_lowercase();

    if prof_type != "proficiency" && prof_type != "expertise" {
        return Err(AppError::BadRequest(
            "proficiency_type must be 'proficiency' or 'expertise'".into(),
        ));
    }

    let row = sqlx::query_as!(
        CharacterProficiencyRow,
        r#"
        UPDATE character_proficiencies SET proficiency_type = $1
        WHERE id = $2 AND character_id = $3
        RETURNING id, character_id, category, name, proficiency_type
        "#,
        prof_type,
        prof_id,
        character_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Character proficiency not found".into()))?;

    Ok(Json(row))
}

// DELETE /characters/:id/proficiencies/:prof_id
pub async fn remove_character_proficiency(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((character_id, prof_id)): Path<(Uuid, i32)>,
) -> Result<StatusCode> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let result = sqlx::query!(
        "DELETE FROM character_proficiencies WHERE id = $1 AND character_id = $2",
        prof_id,
        character_id,
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Character proficiency not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}
