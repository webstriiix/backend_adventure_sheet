use super::import_helpers::{get_class_id, get_source_id, get_spell_id, upsert_source};
use serde_json::{Value, json};
use sqlx::PgPool;

pub async fn import_spells(pool: &PgPool, data: &Value) -> anyhow::Result<()> {
    let spells = match data["spell"].as_array() {
        Some(s) => s,
        None => return Ok(()),
    };

    for s in spells {
        let source_slug = s["source"].as_str().unwrap_or("PHB");
        upsert_source(pool, source_slug, false).await?;
        let source_id = get_source_id(pool, source_slug).await?;

        sqlx::query!(
            r#"
            INSERT INTO spells (
                name, source_id, level, school, casting_time, range, components,
                duration, entries, entries_higher_lvl, ritual, concentration
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (name, source_id) DO UPDATE SET
                level = EXCLUDED.level,
                entries = EXCLUDED.entries
            "#,
            s["name"].as_str().unwrap_or(""),
            source_id,
            s["level"].as_i64().unwrap_or(0) as i32,
            s["school"].as_str().unwrap_or(""),
            s.get("time").cloned().unwrap_or(json!([])),
            s.get("range").cloned().unwrap_or(json!({})),
            s.get("components").cloned().unwrap_or(json!({})),
            s.get("duration").cloned().unwrap_or(json!([])),
            s["entries"],
            s.get("entriesHigherLevel"),
            s["meta"]["ritual"].as_bool().unwrap_or(false),
            s["concentration"].as_bool().unwrap_or(false)
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Import spell-to-class mappings from spells/sources.json
/// Structure: { "PHB": { "SpellName": { "class": [{"name":"Wizard","source":"PHB"}], "classVariant": [...] } } }
pub async fn import_spell_classes(pool: &PgPool, raw: &str) -> anyhow::Result<()> {
    let data: Value = serde_json::from_str(raw)?;

    let obj = data
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Expected top-level object"))?;

    for (source_slug, spells_map) in obj {
        let spells = match spells_map.as_object() {
            Some(m) => m,
            None => continue,
        };

        for (spell_name, spell_data) in spells {
            let spell_id = match get_spell_id(pool, spell_name, source_slug).await {
                Ok(id) => id,
                Err(_) => continue,
            };

            // Process "class" array
            if let Some(classes) = spell_data["class"].as_array() {
                for cls in classes {
                    let class_name = cls["name"].as_str().unwrap_or("");
                    let class_source = cls["source"].as_str().unwrap_or("PHB");
                    if let Ok(class_id) = get_class_id(pool, class_name, class_source).await {
                        sqlx::query!(
                            "INSERT INTO spell_classes (spell_id, class_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                            spell_id,
                            class_id,
                        )
                        .execute(pool)
                        .await?;
                    }
                }
            }

            // Process "classVariant" array (expanded spell lists from supplements)
            if let Some(variants) = spell_data["classVariant"].as_array() {
                for cls in variants {
                    let class_name = cls["name"].as_str().unwrap_or("");
                    let class_source = cls["source"].as_str().unwrap_or("PHB");
                    if let Ok(class_id) = get_class_id(pool, class_name, class_source).await {
                        sqlx::query!(
                            "INSERT INTO spell_classes (spell_id, class_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                            spell_id,
                            class_id,
                        )
                        .execute(pool)
                        .await?;
                    }
                }
            }
        }
    }

    Ok(())
}
