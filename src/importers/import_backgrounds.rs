use super::import_helpers::{get_source_id, upsert_source};
use serde_json::Value;
use sqlx::PgPool;
use tracing;

fn extract_feat_name_from_entries(entries: &Value) -> Option<String> {
    fn walk(val: &Value) -> Option<String> {
        match val {
            Value::String(s) => {
                if let Some(start) = s.find("{@feat ") {
                    let after_prefix = &s[start + 7..];
                    if let Some(end) = after_prefix.find('}') {
                        let inner = &after_prefix[..end];
                        let name = inner.split('|').next().unwrap_or(inner);
                        return Some(name.to_string());
                    }
                }
                None
            }
            Value::Array(arr) => {
                for item in arr {
                    if let Some(r) = walk(item) {
                        return Some(r);
                    }
                }
                None
            }
            Value::Object(map) => {
                for v in map.values() {
                    if let Some(r) = walk(v) {
                        return Some(r);
                    }
                }
                None
            }
            _ => None,
        }
    }
    walk(entries)
}

pub async fn import_backgrounds(pool: &PgPool, data: &Value) -> anyhow::Result<()> {
    tracing::info!("Starting import of backgrounds from JSON data");
    let bgs = match data["background"].as_array() {
        Some(b) => b,
        None => return Ok(()),
    };

    tracing::info!(count = bgs.len(), "Importing backgrounds");

    for bg in bgs {
        let source_slug = bg["source"].as_str().unwrap_or("PHB");
        upsert_source(pool, source_slug, false).await?;
        let source_id = get_source_id(pool, source_slug).await?;

        let ability_bonuses = bg.get("ability").cloned();
        let has_structured_feat = bg.get("feat").is_some();
        let has_entry_feat = extract_feat_name_from_entries(&bg["entries"]).is_some();
        let grants_bonus_feat = has_structured_feat || has_entry_feat;

        let granted_feat_id: Option<i32> = if let Some(feat_val) = bg.get("feat") {
            let feat_name = feat_val
                .as_array()
                .and_then(|a| a.first())
                .or_else(|| {
                    if feat_val.is_object() {
                        Some(feat_val)
                    } else {
                        None
                    }
                })
                .and_then(|o| o.get("name"))
                .and_then(|n| n.as_str());

            match feat_name {
                Some(name) => sqlx::query_scalar!(
                    "SELECT id FROM feats WHERE name = $1 LIMIT 1",
                    name
                )
                .fetch_optional(pool)
                .await?,
                None => None,
            }
        } else if let Some(feat_name) = extract_feat_name_from_entries(&bg["entries"]) {
            sqlx::query_scalar!(
                "SELECT id FROM feats WHERE name = $1 LIMIT 1",
                feat_name
            )
            .fetch_optional(pool)
            .await?
        } else {
            None
        };

        sqlx::query!(
            r#"
            INSERT INTO backgrounds (
                name, source_id, skill_proficiencies, tool_proficiencies,
                language_count, starting_equipment, ability_bonuses,
                grants_bonus_feat, granted_feat_id, entries
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (name, source_id) DO UPDATE SET
                ability_bonuses = EXCLUDED.ability_bonuses,
                grants_bonus_feat = EXCLUDED.grants_bonus_feat,
                granted_feat_id = EXCLUDED.granted_feat_id,
                entries = EXCLUDED.entries
            "#,
            bg["name"].as_str().unwrap_or(""),
            source_id,
            bg.get("skillProficiencies"),
            bg.get("toolProficiencies"),
            bg.get("languageProficiencies")
                .and_then(|l| l.as_array())
                .map(|a| a.len() as i32)
                .or(Some(0)),
            bg.get("startingEquipment"),
            ability_bonuses,
            grants_bonus_feat,
            granted_feat_id,
            bg["entries"],
        )
        .execute(pool)
        .await?;
    }

    tracing::info!(count = bgs.len(), "Successfully imported {} background records.", bgs.len());

    Ok(())
}
