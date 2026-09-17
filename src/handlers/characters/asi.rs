use super::{get_user_id, verify_character_ownership};
use crate::{
    db::AppState,
    error::Result,
    models::{
        character::{AsiChoice, AsiChoiceRequest, Character},
        feats::Feat,
    },
};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use tracing;
use uuid::Uuid;

// GET /characters/:id/asi-history
pub async fn list_asi_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
) -> Result<Json<Vec<AsiChoice>>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    let choices = sqlx::query_as!(
        AsiChoice,
        r#"
        SELECT ac.*
        FROM character_asi_choices ac
        WHERE ac.character_id = $1
        ORDER BY ac.level, ac.id
        "#,
        character_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(choices))
}

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

    let all_feats = sqlx::query_as!(
        Feat,
        r#"
        SELECT
            f.id, f.name, f.source_id, src.slug as source_slug, f.page,
            f.prerequisite, f.ability, f.skill_proficiencies, f.resist,
            f.additional_spells, f.has_uses, f.uses_formula, f.recharge_on, f.entries
        FROM feats f
        JOIN sources src ON src.id = f.source_id
        "#
    )
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

    let span = tracing::info_span!("wizard_asi_choice", %character_id);
    let _guard = span.enter();

    tracing::info!(character_id = %character_id, "Saving Wizard Step: ASI or Feat Selection");

    let level = payload.gained_at_level.unwrap_or_else(|| {
        // Use current total level from character_classes if not provided
        0
    });

    // Resolve level from character if not provided
    let resolved_level: i32 = if level == 0 {
        sqlx::query_scalar!(
            "SELECT COALESCE(SUM(level), 1) FROM character_classes WHERE character_id = $1",
            character_id
        )
        .fetch_one(&state.db)
        .await?
        .unwrap_or(1) as i32
    } else {
        level
    };

    // ── Validate against progression rules ──
    let class_info = sqlx::query!(
        r#"
        SELECT c.asi_levels
        FROM character_classes cc
        JOIN classes c ON c.id = cc.class_id
        WHERE cc.character_id = $1 AND cc.is_primary = true
        "#,
        character_id
    )
    .fetch_optional(&state.db)
    .await?;

    let asi_levels = class_info
        .as_ref()
        .and_then(|r| r.asi_levels.as_ref())
        .map(|v| v.clone())
        .unwrap_or_default();

    if !asi_levels.contains(&resolved_level) {
        return Err(crate::error::AppError::BadRequest(format!(
            "Level {} is not a valid ASI level for this class",
            resolved_level
        )));
    }

    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM character_asi_choices WHERE character_id = $1 AND level = $2",
        character_id,
        resolved_level,
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);

    if existing > 0 {
        return Err(crate::error::AppError::BadRequest(format!(
            "ASI choice already exists for level {}",
            resolved_level
        )));
    }

    if let Some(feat_id) = payload.feat_id {
        tracing::info!(%character_id, feat_id, "Character choosing feat");
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
                (character_id, feat_id, uses_remaining, uses_max, recharge_on, source_type, gained_at_level)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            character_id,
            feat_id,
            max_uses,
            max_uses,
            feat.recharge_on,
            source_type,
            resolved_level,
        )
        .execute(&state.db)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO character_asi_choices
                (character_id, level, bump_str, bump_dex, bump_con, bump_int, bump_wis, bump_cha, feat_id)
            VALUES ($1, $2, 0, 0, 0, 0, 0, 0, $3)
            "#,
            character_id,
            resolved_level,
            feat_id,
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

        tracing::info!(
            %character_id,
            str = bump_str,
            dex = bump_dex,
            con = bump_con,
            int = bump_int,
            wis = bump_wis,
            cha = bump_cha,
            "Character choosing ASI"
        );

        sqlx::query!(
            r#"
            INSERT INTO character_asi_choices
                (character_id, level, bump_str, bump_dex, bump_con, bump_int, bump_wis, bump_cha)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            character_id,
            resolved_level,
            bump_str,
            bump_dex,
            bump_con,
            bump_int,
            bump_wis,
            bump_cha,
        )
        .execute(&state.db)
        .await?;

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

    tracing::info!(character_id = %character_id, "Updated Stats for character");

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
