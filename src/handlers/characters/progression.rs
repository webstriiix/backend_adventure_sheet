use super::{get_user_id, verify_character_ownership};
use crate::{
    db::AppState,
    error::Result,
    models::character::{
        AsiChoice, ChoiceDetail, DecisionPoint, ProgressionManifest, WeaponMasteryChoice,
    },
};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde_json::Value;
use uuid::Uuid;

// GET /characters/:id/progression
pub async fn get_progression(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(character_id): Path<Uuid>,
) -> Result<Json<ProgressionManifest>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, character_id, user_id).await?;

    // Fetch primary class info
    let class_info = sqlx::query!(
        r#"
        SELECT
            c.id as class_id,
            c.name as class_name,
            s.slug as class_source,
            c.asi_levels,
            c.class_table,
            cc.subclass_id
        FROM character_classes cc
        JOIN classes c ON c.id = cc.class_id
        JOIN sources s ON s.id = c.source_id
        WHERE cc.character_id = $1 AND cc.is_primary = true
        "#,
        character_id
    )
    .fetch_optional(&state.db)
    .await?;

    let (class_id, class_name, class_source, asi_levels, class_table, subclass_id) =
        match class_info {
            Some(r) => (
                r.class_id,
                r.class_name,
                r.class_source,
                r.asi_levels,
                r.class_table,
                r.subclass_id,
            ),
            None => {
                return Ok(Json(ProgressionManifest {
                    character_id,
                    total_level: 0,
                    class_name: String::new(),
                    class_source: String::new(),
                    decision_points: vec![],
                }));
            }
        };

    // Total character level
    let total_level = sqlx::query_scalar!(
        "SELECT COALESCE(SUM(level), 0) FROM character_classes WHERE character_id = $1",
        character_id
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0) as i32;

    // Existing ASI choices
    let asi_choices = sqlx::query_as!(
        AsiChoice,
        r#"SELECT * FROM character_asi_choices WHERE character_id = $1 ORDER BY level"#,
        character_id
    )
    .fetch_all(&state.db)
    .await?;

    // Existing feats (ASI-sourced)
    let feats = sqlx::query_as!(
        super::feats::CharacterFeatRow,
        r#"
        SELECT id, character_id, feat_id, chosen_ability, uses_remaining, uses_max,
               recharge_on, source_type, gained_at_level
        FROM character_feats
        WHERE character_id = $1 AND (source_type = 'asi' OR source_type IS NULL)
        ORDER BY gained_at_level
        "#,
        character_id
    )
    .fetch_all(&state.db)
    .await?;

    // Existing weapon masteries
    let weapon_masteries = sqlx::query_as!(
        WeaponMasteryChoice,
        r#"SELECT id, character_id, mastery_name, source_level, created_at
           FROM character_weapon_masteries WHERE character_id = $1"#,
        character_id
    )
    .fetch_all(&state.db)
    .await?;
    // ── Parse weapon mastery levels from class_table ──
    let weapon_mastery_levels = parse_weapon_mastery_levels(&class_table);

    // Get subclass unlock level
    let subclass_unlock_level = sqlx::query_scalar!(
        "SELECT MIN(level) FROM class_features WHERE class_id = $1 AND is_subclass_gate = true",
        class_id
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(3);

    // Build a set of ASI levels for quick lookup
    let asi_set: std::collections::HashSet<i32> = asi_levels
        .as_ref()
        .map(|v| v.iter().copied().collect())
        .unwrap_or_default();

    // Group asi_choices by level
    let mut asi_by_level: std::collections::HashMap<i32, Vec<&AsiChoice>> =
        std::collections::HashMap::new();
    for ac in &asi_choices {
        asi_by_level.entry(ac.level).or_default().push(ac);
    }

    // Group feats by gained_at_level
    let mut feats_by_level: std::collections::HashMap<i32, Vec<&super::feats::CharacterFeatRow>> =
        std::collections::HashMap::new();
    for f in &feats {
        if let Some(lvl) = f.gained_at_level {
            feats_by_level.entry(lvl).or_default().push(f);
        }
    }

    let mut decision_points: Vec<DecisionPoint> = Vec::new();

    for level in 1..=total_level {
        // ── ASI / Feat decision point ──
        if asi_set.contains(&level) {
            let choices_for_level = asi_by_level.get(&level).into_iter().flatten();
            let feats_for_level = feats_by_level.get(&level).into_iter().flatten();

            let mut current_choices: Vec<ChoiceDetail> = Vec::new();

            for ac in choices_for_level {
                let mut parts = Vec::new();
                if ac.bump_str > 0 {
                    parts.push(format!("Str +{}", ac.bump_str));
                }
                if ac.bump_dex > 0 {
                    parts.push(format!("Dex +{}", ac.bump_dex));
                }
                if ac.bump_con > 0 {
                    parts.push(format!("Con +{}", ac.bump_con));
                }
                if ac.bump_int > 0 {
                    parts.push(format!("Int +{}", ac.bump_int));
                }
                if ac.bump_wis > 0 {
                    parts.push(format!("Wis +{}", ac.bump_wis));
                }
                if ac.bump_cha > 0 {
                    parts.push(format!("Cha +{}", ac.bump_cha));
                }

                if let Some(feat_id) = ac.feat_id {
                    current_choices.push(ChoiceDetail {
                        id: format!("feat:{}", feat_id),
                        description: format!("Feat #{}", feat_id),
                    });
                } else if !parts.is_empty() {
                    current_choices.push(ChoiceDetail {
                        id: format!("asi:{}", ac.id),
                        description: parts.join(", "),
                    });
                }
            }

            for f in feats_for_level {
                current_choices.push(ChoiceDetail {
                    id: format!("character_feat:{}", f.id),
                    description: format!("Feat #{}", f.feat_id),
                });
            }

            decision_points.push(DecisionPoint {
                level,
                name: "Ability Score Improvement".into(),
                r#type: "decision_slot".into(),
                choice_type: "asi".into(),
                required_count: 1,
                status: if !current_choices.is_empty() {
                    "complete".into()
                } else {
                    "pending".into()
                },
                current_choices,
            });
        }

        // ── Subclass Selection decision point ──
        if level == subclass_unlock_level {
            let status = if subclass_id.is_some() {
                "complete".into()
            } else {
                "pending".into()
            };
            let mut sc_choices = Vec::new();
            if let Some(sc_id) = subclass_id {
                let sc_name =
                    sqlx::query_scalar!("SELECT name FROM subclasses WHERE id = $1", sc_id)
                        .fetch_one(&state.db)
                        .await?;
                sc_choices.push(ChoiceDetail {
                    id: format!("subclass:{}", sc_id),
                    description: sc_name,
                });
            }
            decision_points.push(DecisionPoint {
                level,
                name: "Subclass Selection".into(),
                r#type: "decision_slot".into(),
                choice_type: "subclass".into(),
                required_count: 1,
                status,
                current_choices: sc_choices,
            });
        }

        // ── Weapon Mastery decision point ──
        if weapon_mastery_levels.contains_key(&level) {
            let required = weapon_mastery_levels[&level];
            let chosen: Vec<&WeaponMasteryChoice> = weapon_masteries
                .iter()
                .filter(|wm| wm.source_level == level)
                .collect();

            decision_points.push(DecisionPoint {
                level,
                name: "Weapon Mastery".into(),
                r#type: "decision_slot".into(),
                choice_type: "weapon_mastery".into(),
                required_count: required,
                status: if chosen.len() as i32 >= required {
                    "complete".into()
                } else if chosen.is_empty() {
                    "pending".into()
                } else {
                    "partial".into()
                },
                current_choices: chosen
                    .iter()
                    .map(|wm| ChoiceDetail {
                        id: format!("weapon_mastery:{}", wm.id),
                        description: wm.mastery_name.clone(),
                    })
                    .collect(),
            });
        }
    }

    Ok(Json(ProgressionManifest {
        character_id,
        total_level,
        class_name,
        class_source,
        decision_points,
    }))
}

/// Parse the class_table JSONB to find Weapon Mastery column and detect levels
/// where the required count changes.
/// Returns a map of level -> required_count (number of masteries to choose).
fn parse_weapon_mastery_levels(class_table: &Value) -> std::collections::HashMap<i32, i32> {
    let mut result = std::collections::HashMap::new();

    let arr = match class_table.as_array() {
        Some(a) => a,
        None => return result,
    };

    let first = match arr.first().and_then(|v| v.as_object()) {
        Some(o) => o,
        None => return result,
    };

    let col_labels = match first.get("colLabels").and_then(|v| v.as_array()) {
        Some(l) => l,
        None => return result,
    };

    let mastery_idx = col_labels.iter().position(|label| {
        label
            .as_str()
            .map(|s| s.contains("Weapon Mastery"))
            .unwrap_or(false)
    });

    let mastery_idx = match mastery_idx {
        Some(i) => i,
        None => return result,
    };

    let rows = match first.get("rows").and_then(|v| v.as_array()) {
        Some(r) => r,
        None => return result,
    };

    let mut prev_count: Option<i32> = None;

    for (i, row) in rows.iter().enumerate() {
        let level = (i + 1) as i32;
        let cell = row.get(mastery_idx).and_then(|v| v.as_str());
        let count: i32 = cell.and_then(|s| s.parse().ok()).unwrap_or(0);

        if count > 0 && Some(count) != prev_count {
            result.insert(level, count);
            prev_count = Some(count);
        }
    }

    result
}
