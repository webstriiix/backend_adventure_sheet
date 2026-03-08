use super::{get_user_id, verify_character_ownership};
use crate::{
    db::AppState,
    error::Result,
    models::{
        character::{AsiChoiceRequest, Character},
        feats::Feat,
    },
};
use axum::{
    Json,
    extract::{Path, State},
};

// GET /characters/:id/available-feats
pub async fn list_available_feats(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(character_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<Feat>>> {
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

    let level_record = sqlx::query!(
        "SELECT COALESCE(SUM(level), 0) as total FROM character_classes WHERE character_id = $1",
        character_id
    )
    .fetch_one(&state.db)
    .await?;
    let char_level = level_record.total.unwrap_or(0) as i32;

    let race_record = sqlx::query!(
        "SELECT r.name FROM races r WHERE r.id = $1",
        character.race_id
    )
    .fetch_optional(&state.db)
    .await?;
    let race_name = race_record.map(|r| r.name).unwrap_or_default();

    let spell_check = sqlx::query!(
        "SELECT 1 as has_magic FROM classes c JOIN character_classes cc ON cc.class_id = c.id WHERE cc.character_id = $1 AND c.spellcasting_ability IS NOT NULL LIMIT 1",
        character_id
    )
    .fetch_optional(&state.db)
    .await?;
    let has_spellcasting = spell_check.is_some();

    let all_feats = sqlx::query_as!(Feat, "SELECT * FROM feats")
        .fetch_all(&state.db)
        .await?;

    let mut available = Vec::new();

    for feat in all_feats {
        let mut meets_prereq = true;

        if let Some(prereqs) = &feat.prerequisite {
            if let Some(options) = prereqs.as_array() {
                if !options.is_empty() {
                    let mut option_met = false;
                    for opt in options {
                        let mut this_opt_met = true;
                        if let Some(abilities) = opt.get("ability").and_then(|a| a.as_array()) {
                            for ab in abilities {
                                if let Some(req) = ab.get("str").and_then(|v| v.as_i64()) {
                                    if character.str < req as i32 {
                                        this_opt_met = false;
                                    }
                                }
                                if let Some(req) = ab.get("dex").and_then(|v| v.as_i64()) {
                                    if character.dex < req as i32 {
                                        this_opt_met = false;
                                    }
                                }
                                if let Some(req) = ab.get("con").and_then(|v| v.as_i64()) {
                                    if character.con < req as i32 {
                                        this_opt_met = false;
                                    }
                                }
                                if let Some(req) = ab.get("int").and_then(|v| v.as_i64()) {
                                    if character.int < req as i32 {
                                        this_opt_met = false;
                                    }
                                }
                                if let Some(req) = ab.get("wis").and_then(|v| v.as_i64()) {
                                    if character.wis < req as i32 {
                                        this_opt_met = false;
                                    }
                                }
                                if let Some(req) = ab.get("cha").and_then(|v| v.as_i64()) {
                                    if character.cha < req as i32 {
                                        this_opt_met = false;
                                    }
                                }
                            }
                        }
                        if let Some(level_req) = opt.get("level").and_then(|l| l.as_i64()) {
                            if char_level < level_req as i32 {
                                this_opt_met = false;
                            }
                        }
                        if let Some(race_reqs) = opt.get("race").and_then(|r| r.as_array()) {
                            let mut race_matched = false;
                            for r in race_reqs {
                                if let Some(req_name) = r.get("name").and_then(|n| n.as_str()) {
                                    if race_name.contains(req_name) {
                                        race_matched = true;
                                        break;
                                    }
                                }
                            }
                            if !race_matched {
                                this_opt_met = false;
                            }
                        }
                        if let Some(spell_req) = opt.get("spellcasting").and_then(|s| s.as_bool()) {
                            if spell_req && !has_spellcasting {
                                this_opt_met = false;
                            }
                        }
                        if this_opt_met {
                            option_met = true;
                            break;
                        }
                    }
                    if !option_met {
                        meets_prereq = false;
                    }
                }
            }
        }

        if meets_prereq {
            available.push(feat);
        }
    }

    Ok(Json(available))
}

// POST /characters/:id/asi-choice
pub async fn choose_asi_or_feat(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(character_id): Path<uuid::Uuid>,
    Json(payload): Json<AsiChoiceRequest>,
) -> Result<Json<Character>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    if let Some(feat_id) = payload.feat_id {
        let feat = sqlx::query!(
            "SELECT has_uses, recharge_on FROM feats WHERE id = $1",
            feat_id
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or(crate::error::AppError::NotFound("Feat not found".into()))?;

        let max_uses = if feat.has_uses { 1 } else { 0 };
        let source_type = payload.source_type.unwrap_or_else(|| "asi".to_string());

        sqlx::query!(
            r#"
            INSERT INTO character_feats 
                (character_id, feat_id, uses_remaining, uses_max, recharge_on, source_type)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            character_id,
            feat_id,
            max_uses,
            max_uses,
            feat.recharge_on,
            source_type,
        )
        .execute(&state.db)
        .await?;
    } else {
        let bump_str = payload.bump_str.unwrap_or(0);
        let bump_dex = payload.bump_dex.unwrap_or(0);
        let bump_con = payload.bump_con.unwrap_or(0);
        let bump_int = payload.bump_int.unwrap_or(0);
        let bump_wis = payload.bump_wis.unwrap_or(0);
        let bump_cha = payload.bump_cha.unwrap_or(0);

        let total_bump = bump_str + bump_dex + bump_con + bump_int + bump_wis + bump_cha;

        if total_bump > 3 {
            return Err(crate::error::AppError::BadRequest(
                "Cannot increase ability scores by more than 3".into(),
            ));
        }

        sqlx::query!(
            r#"
            UPDATE characters SET
                str = str + $1, dex = dex + $2, con = con + $3,
                int = int + $4, wis = wis + $5, cha = cha + $6
            WHERE id = $7
            "#,
            bump_str,
            bump_dex,
            bump_con,
            bump_int,
            bump_wis,
            bump_cha,
            character_id
        )
        .execute(&state.db)
        .await?;
    }

    let updated = sqlx::query_as!(
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

    Ok(Json(updated))
}
