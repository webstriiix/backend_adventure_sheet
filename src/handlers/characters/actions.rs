use super::{get_user_id, verify_character_ownership};
use crate::models::action::{ActionItem, ActionPayload};
use crate::{db::AppState, error::Result};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct ActionFileData {
    action: Vec<serde_json::Value>,
}

pub async fn get_character_actions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<ActionPayload>> {
    let user_id = get_user_id(&headers, &state.config.jwt_secret)?;
    verify_character_ownership(&state.db, id, user_id).await?;

    let mut payload = ActionPayload::default();

    // 1. Load standard actions from actions.json
    if let Ok(file_content) = std::fs::read_to_string("adventure_sheet_json/data/actions.json") {
        if let Ok(data) = serde_json::from_str::<ActionFileData>(&file_content) {
            for entry in data.action {
                let name = entry["name"].as_str().unwrap_or("Unknown").to_string();
                let source = entry["source"].as_str().map(|s| s.to_string());

                let mut description = None;
                if let Some(entries) = entry["entries"].as_array() {
                    if let Some(first) = entries.first() {
                        if let Some(s) = first.as_str() {
                            description = Some(s.to_string());
                        }
                    }
                }

                let time = entry["time"].clone();
                let mut bucket = "other";

                if let Some(time_arr) = time.as_array() {
                    if let Some(t) = time_arr.first() {
                        if let Some(unit) = t["unit"].as_str() {
                            bucket = match unit {
                                "action" => "action",
                                "bonus" => "bonus_action",
                                "reaction" => "reaction",
                                _ => "other",
                            };
                        } else if let Some(s) = t.as_str() {
                            if s.to_lowercase() == "free" || s.to_lowercase() == "varies" {
                                bucket = "other";
                            }
                        }
                    }
                }

                let action_item = ActionItem {
                    name: name.clone(),
                    source,
                    description,
                    range: None,
                    hit_bonus: None,
                    damage: None,
                    max_uses: None,
                    current_uses: None,
                    reset_type: None,
                    time: Some(time),
                };

                payload.all.push(action_item.clone());

                match bucket {
                    "action" => payload.action.push(action_item),
                    "bonus_action" => payload.bonus_action.push(action_item),
                    "reaction" => payload.reaction.push(action_item),
                    _ => payload.other.push(action_item),
                }
            }
        }
    }

    // Fetch Character stats for weapons
    let character = sqlx::query!(
        "SELECT str, dex, con, int, wis, cha FROM characters WHERE id = $1",
        id
    )
    .fetch_one(&state.db)
    .await?;

    let str_mod = (character.str - 10) / 2;
    let dex_mod = (character.dex - 10) / 2;

    let total_level = sqlx::query!(
        "SELECT COALESCE(SUM(level), 1) as level FROM character_classes WHERE character_id = $1",
        id
    )
    .fetch_one(&state.db)
    .await?
    .level
    .unwrap_or(1);

    let proficiency_bonus = (total_level - 1) / 4 + 2;

    // 2. Fetch Weapons (Attack)
    let inventory = sqlx::query!(
        r#"
        SELECT i.name, i.properties, i.damage, i.type as item_type, i.entries 
        FROM character_inventory ci
        JOIN items i ON ci.item_id = i.id
        WHERE ci.character_id = $1 AND ci.is_equipped = true AND split_part(i.type, '|', 1) IN ('M', 'R')
        "#,
        id
    )
    .fetch_all(&state.db)
    .await?;

    for item in inventory {
        let is_finesse = item
            .properties
            .iter()
            .any(|p| p.to_lowercase() == "finesse");
        let is_ranged = item.item_type.as_deref() == Some("R");

        let stat_mod = if is_ranged {
            dex_mod
        } else if is_finesse {
            str_mod.max(dex_mod)
        } else {
            str_mod
        };

        let hit = stat_mod + proficiency_bonus as i32;
        let hit_str = if hit >= 0 {
            format!("+{}", hit)
        } else {
            hit.to_string()
        };

        let mut dmg_str = String::new();
        if let Some(dmg_json) = item.damage {
            if let Some(dmg1) = dmg_json.get("dmg1").and_then(|d| d.as_str()) {
                let mut base_dmg = dmg1.to_string();
                if stat_mod > 0 {
                    base_dmg.push_str(&format!(" + {}", stat_mod));
                } else if stat_mod < 0 {
                    base_dmg.push_str(&format!(" - {}", stat_mod.abs()));
                }
                dmg_str = base_dmg;
            }
        }

        let action_item = ActionItem {
            name: item.name.clone(),
            source: None,
            description: item.entries.map(|e| e.to_string()),
            range: None,
            hit_bonus: Some(hit_str),
            damage: if dmg_str.is_empty() {
                None
            } else {
                Some(dmg_str)
            },
            max_uses: None,
            current_uses: None,
            reset_type: None,
            time: Some(serde_json::json!([{"number": 1, "unit": "action"}])),
        };

        payload.all.push(action_item.clone());
        payload.attack.push(action_item);
    }

    // Hardcode Unarmed Strike
    let unarmed_hit = str_mod + proficiency_bonus as i32;
    let unarmed_hit_str = if unarmed_hit >= 0 {
        format!("+{}", unarmed_hit)
    } else {
        unarmed_hit.to_string()
    };
    let unarmed = ActionItem {
        name: "Unarmed Strike".to_string(),
        source: None,
        description: Some("You make a melee attack that involves using your body to deal damage or grapple/shove.".to_string()),
        range: Some("5ft.".to_string()),
        hit_bonus: Some(unarmed_hit_str),
        damage: Some(format!("{}", 1 + str_mod)),
        max_uses: None,
        current_uses: None,
        reset_type: None,
        time: Some(serde_json::json!([{"number": 1, "unit": "action"}])),
    };
    payload.all.push(unarmed.clone());
    payload.attack.push(unarmed);

    let resource_pools = sqlx::query!(
        "SELECT resource_name, uses_remaining FROM character_resource_pools WHERE character_id = $1",
        id
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|r| (r.resource_name, r.uses_remaining))
    .collect::<std::collections::HashMap<_, _>>();

    let paladin_level_res = sqlx::query!(
        r#"
        SELECT cc.level 
        FROM character_classes cc 
        JOIN classes c ON cc.class_id = c.id 
        WHERE cc.character_id = $1 AND c.name ILIKE 'Paladin'
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await?;
    let paladin_level = paladin_level_res.map(|r| r.level).unwrap_or(0);

    // 3. Fetch Features (DISTINCT by name so upgrades don't duplicate the entry)
    let class_features = sqlx::query!(
        r#"
        SELECT DISTINCT ON (f.name) f.name, f.entries 
        FROM character_classes cc
        JOIN class_features f ON cc.class_id = f.class_id AND f.level <= cc.level
        WHERE cc.character_id = $1
        ORDER BY f.name, f.level DESC
        "#,
        id
    )
    .fetch_all(&state.db)
    .await?;

    for feature in class_features {
        let entries_str = feature.entries.to_string().to_lowercase();
        let mut time_json = None;
        let mut buckets = Vec::new();

        if entries_str.contains("as an action") || entries_str.contains("magic action") {
            buckets.push("action");
            time_json = Some(serde_json::json!([{"number": 1, "unit": "action"}]));
        }
        if entries_str.contains("bonus action") {
            buckets.push("bonus_action");
            time_json = time_json.or(Some(serde_json::json!([{"number": 1, "unit": "bonus"}])));
        }
        if entries_str.contains("reaction") {
            buckets.push("reaction");
            time_json = time_json.or(Some(serde_json::json!([{"number": 1, "unit": "reaction"}])));
        }

        let feature_name_lc = feature.name.to_lowercase();
        let is_lay_on_hands = feature_name_lc == "lay on hands";
        // Only the canonical "Channel Divinity" row (not sub-variants like "Channel Divinity: Turn Undead")
        let is_channel_divinity = feature_name_lc == "channel divinity";

        let mut feature_max_uses = None;
        let mut feature_current_uses = None;
        let mut reset_type = None;

        if is_lay_on_hands {
            buckets.push("limited_use");
            let max = paladin_level * 5;
            feature_max_uses = Some(max);
            feature_current_uses = Some(resource_pools.get("Lay on Hands").copied().unwrap_or(max));
            reset_type = Some("Long Rest".to_string());
        } else if is_channel_divinity {
            buckets.push("limited_use");
            let max = if paladin_level >= 11 { 3 } else if paladin_level >= 6 { 2 } else { 1 };
            feature_max_uses = Some(max);
            feature_current_uses = Some(resource_pools.get("Channel Divinity").copied().unwrap_or(max));
            reset_type = Some("Short Rest".to_string());
        }

        if buckets.is_empty() {
            continue;
        }

        let action_item = ActionItem {
            name: feature.name.clone(),
            source: None,
            description: feature
                .entries
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            range: None,
            hit_bonus: None,
            damage: None,
            max_uses: feature_max_uses,
            current_uses: feature_current_uses,
            reset_type,
            time: time_json,
        };

        payload.all.push(action_item.clone());
        for bucket in buckets {
            match bucket {
                "action" => payload.action.push(action_item.clone()),
                "bonus_action" => payload.bonus_action.push(action_item.clone()),
                "reaction" => payload.reaction.push(action_item.clone()),
                "limited_use" => payload.limited_use.push(action_item.clone()),
                _ => payload.other.push(action_item.clone()),
            }
        }
    }

    let feats = sqlx::query!(
        r#"
        SELECT f.name, f.entries, cf.uses_max, cf.uses_remaining, cf.recharge_on
        FROM character_feats cf
        JOIN feats f ON cf.feat_id = f.id
        WHERE cf.character_id = $1
        "#,
        id
    )
    .fetch_all(&state.db)
    .await?;

    for feat in feats {
        let entries_str = feat.entries.to_string().to_lowercase();
        let mut time_json = None;
        let mut buckets = Vec::new();

        if entries_str.contains("as an action") || entries_str.contains("magic action") {
            buckets.push("action");
            time_json = Some(serde_json::json!([{"number": 1, "unit": "action"}]));
        }
        if entries_str.contains("bonus action") {
            buckets.push("bonus_action");
            time_json = time_json.or(Some(serde_json::json!([{"number": 1, "unit": "bonus"}])));
        }
        if entries_str.contains("reaction") {
            buckets.push("reaction");
            time_json = time_json.or(Some(serde_json::json!([{"number": 1, "unit": "reaction"}])));
        }

        if feat.uses_max.unwrap_or(0) > 0 {
            buckets.push("limited_use");
        }

        if buckets.is_empty() {
            continue;
        }

        let action_item = ActionItem {
            name: feat.name.clone(),
            source: None,
            description: feat
                .entries
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            range: None,
            hit_bonus: None,
            damage: None,
            max_uses: feat.uses_max,
            current_uses: feat.uses_remaining,
            reset_type: feat.recharge_on,
            time: time_json,
        };

        payload.all.push(action_item.clone());
        for bucket in buckets {
            match bucket {
                "action" => payload.action.push(action_item.clone()),
                "bonus_action" => payload.bonus_action.push(action_item.clone()),
                "reaction" => payload.reaction.push(action_item.clone()),
                "limited_use" => payload.limited_use.push(action_item.clone()),
                _ => payload.other.push(action_item.clone()),
            }
        }
    }

    // 4. Fetch Spells
    let spells = sqlx::query!(
        r#"
        SELECT s.name, s.casting_time, s.entries
        FROM character_spells cs
        JOIN spells s ON cs.spell_id = s.id
        WHERE cs.character_id = $1
        "#,
        id
    )
    .fetch_all(&state.db)
    .await?;

    for spell in spells {
        if let Some(time_arr) = spell.casting_time.as_array() {
            if let Some(t) = time_arr.first() {
                if let Some(unit) = t["unit"].as_str() {
                    let bucket = match unit {
                        "action" => "action",
                        "bonus" => "bonus_action",
                        "reaction" => "reaction",
                        "free" | "varies" => "other",
                        _ => continue, // Do not list minute/hour spells
                    };

                    let action_item = ActionItem {
                        name: spell.name.clone(),
                        source: None,
                        description: spell
                            .entries
                            .as_array()
                            .and_then(|a| a.first())
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        range: None,
                        hit_bonus: None,
                        damage: None,
                        max_uses: None,
                        current_uses: None,
                        reset_type: None,
                        time: Some(spell.casting_time.clone()),
                    };

                    payload.all.push(action_item.clone());
                    match bucket {
                        "action" => payload.action.push(action_item),
                        "bonus_action" => payload.bonus_action.push(action_item),
                        "reaction" => payload.reaction.push(action_item),
                        _ => payload.other.push(action_item),
                    }
                }
            }
        }
    }

    Ok(Json(payload))
}
