use super::import_helpers::{get_race_id, get_source_id, upsert_source};
use serde_json::{Value, json};
use sqlx::PgPool;

pub async fn import_races(pool: &PgPool, data: &Value) -> anyhow::Result<()> {
    // Races
    if let Some(races) = data["race"].as_array() {
        for r in races {
            let source_slug = r["source"].as_str().unwrap_or("PHB");
            upsert_source(pool, source_slug, false).await?;
            let source_id = get_source_id(pool, source_slug).await?;

            let race_name = r["name"].as_str().unwrap_or("");
            let race_source = r["source"].as_str().unwrap_or("");
            let grants_bonus_feat = race_name == "Human (Variant)"
                || race_name == "Custom Lineage"
                || (race_name == "Human" && (race_source == "XPHB" || race_source == "PHB"));

            sqlx::query!(
                r#"
                INSERT INTO races (
                    name, source_id, size, speed, ability_bonuses, entries,
                    age_description, alignment_description,
                    skill_proficiencies, language_proficiencies, trait_tags, grants_bonus_feat
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (name, source_id) DO UPDATE SET
                    speed = EXCLUDED.speed,
                    entries = EXCLUDED.entries,
                    grants_bonus_feat = EXCLUDED.grants_bonus_feat
                "#,
                race_name,
                source_id,
                &r["size"]
                    .as_array()
                    .map(|a| a
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>())
                    .unwrap_or_default(),
                r.get("speed").cloned().unwrap_or(json!("30")),
                r.get("ability").cloned().unwrap_or(json!([])),
                r["entries"],
                r["age"]
                    .as_str()
                    .map(|s| s.to_string())
                    .or_else(|| r["age"]["entries"]
                        .as_array()
                        .map(|_| "See entries".to_string())),
                r["alignment"].as_str().map(|s| s.to_string()).or_else(|| {
                    r["alignment"]["entries"]
                        .as_array()
                        .map(|_| "See entries".to_string())
                }),
                r.get("skillProficiencies"),
                r.get("languageProficiencies"),
                &r["traitTags"]
                    .as_array()
                    .map(|a| a
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>())
                    .unwrap_or_default(),
                grants_bonus_feat,
            )
            .execute(pool)
            .await?;
        }
    }

    // Subraces
    if let Some(subraces) = data["subrace"].as_array() {
        for sr in subraces {
            let source_slug = sr["source"].as_str().unwrap_or("PHB");
            upsert_source(pool, source_slug, false).await?;
            let source_id = get_source_id(pool, source_slug).await?;

            let race_name = sr["raceName"]
                .as_str()
                .or_else(|| sr["_copy"]["raceName"].as_str())
                .unwrap_or("");
            let race_source = sr["raceSource"]
                .as_str()
                .or_else(|| sr["_copy"]["raceSource"].as_str())
                .unwrap_or("PHB");

            if let Ok(race_id) = get_race_id(pool, race_name, race_source).await {
                sqlx::query!(
                    r#"
                    INSERT INTO subraces (name, source_id, race_id, speed, ability_bonuses, entries)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    ON CONFLICT (name, source_id, race_id) DO UPDATE SET
                        ability_bonuses = EXCLUDED.ability_bonuses,
                        entries = EXCLUDED.entries
                    "#,
                    sr["name"].as_str().unwrap_or(""),
                    source_id,
                    race_id,
                    sr.get("speed"),
                    sr.get("ability"),
                    sr.get("entries").cloned().unwrap_or(json!([])),
                )
                .execute(pool)
                .await?;

                // --- heuristics: detect options in subrace entries ---
                let entries_text = flatten_entries_text(sr.get("entries").unwrap_or(&json!([])));

                // cantrip option
                if entries_text.to_lowercase().contains("cantrip") {
                    sqlx::query!(
                        r#"INSERT INTO race_options (race_id, subrace_id, source_id, option_type, choices, min_choose, max_choose, note)
                           VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT DO NOTHING"#,
                        race_id,
                        // subrace id unknown at this point in this import pass; we'll resolve by name+source later
                        None::<i32>,
                        source_id,
                        "cantrip",
                        None::<serde_json::Value>,
                        1_i32,
                        1_i32,
                        Some("Detected cantrip option from entries"),
                    )
                    .execute(pool)
                    .await?;
                }

                // variable trait hints
                if entries_text.to_lowercase().contains("one of the following") || entries_text.to_lowercase().contains("one of the following options") {
                    sqlx::query!(
                        r#"INSERT INTO race_options (race_id, subrace_id, source_id, option_type, choices, min_choose, max_choose, note)
                           VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT DO NOTHING"#,
                        race_id,
                        None::<i32>,
                        source_id,
                        "variable_trait",
                        None::<serde_json::Value>,
                        1_i32,
                        1_i32,
                        Some("Detected variable trait choices from entries"),
                    )
                    .execute(pool)
                    .await?;
                }
            }
        }
    }

    // --- heuristics: detect options in races (top-level) ---
    if let Some(races) = data["race"].as_array() {
        for r in races {
            let source_slug = r["source"].as_str().unwrap_or("PHB");
            let source_id = get_source_id(pool, source_slug).await?;
            let race_name = r["name"].as_str().unwrap_or("");
            if let Ok(race_id) = get_race_id(pool, race_name, source_slug).await {
                // grants bonus feat -> create a race_options row for feat choice
                let race_source = r["source"].as_str().unwrap_or("");
                let grants_bonus_feat = race_name == "Human (Variant)"
                    || race_name == "Custom Lineage"
                    || (race_name == "Human" && (race_source == "XPHB" || race_source == "PHB"));
                if grants_bonus_feat {
                    sqlx::query!(
                        r#"INSERT INTO race_options (race_id, subrace_id, source_id, option_type, choices, min_choose, max_choose, note)
                           VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT DO NOTHING"#,
                        race_id,
                        None::<i32>,
                        source_id,
                        "feat",
                        None::<serde_json::Value>,
                        1_i32,
                        1_i32,
                        Some("Human variant or similar grants a bonus feat"),
                    )
                    .execute(pool)
                    .await?;
                }

                // detect cantrip or variable trait in top-level entries
                let entries_text = flatten_entries_text(r.get("entries").unwrap_or(&json!([])));
                if entries_text.to_lowercase().contains("cantrip") {
                    sqlx::query!(
                        r#"INSERT INTO race_options (race_id, subrace_id, source_id, option_type, choices, min_choose, max_choose, note)
                           VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT DO NOTHING"#,
                        race_id,
                        None::<i32>,
                        source_id,
                        "cantrip",
                        None::<serde_json::Value>,
                        1_i32,
                        1_i32,
                        Some("Detected cantrip option from entries"),
                    )
                    .execute(pool)
                    .await?;
                }
                if entries_text.to_lowercase().contains("one of the following") || entries_text.to_lowercase().contains("one of the following options") {
                    sqlx::query!(
                        r#"INSERT INTO race_options (race_id, subrace_id, source_id, option_type, choices, min_choose, max_choose, note)
                           VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT DO NOTHING"#,
                        race_id,
                        None::<i32>,
                        source_id,
                        "variable_trait",
                        None::<serde_json::Value>,
                        1_i32,
                        1_i32,
                        Some("Detected variable trait choices from entries"),
                    )
                    .execute(pool)
                    .await?;
                }
            }
        }
    }

    Ok(())
}


// --- helper: flatten entries into a single searchable string ---
fn flatten_entries_text(val: &serde_json::Value) -> String {
    fn collect(v: &serde_json::Value, out: &mut String) {
        match v {
            serde_json::Value::String(s) => {
                out.push_str(s);
                out.push(' ');
            }
            serde_json::Value::Array(a) => {
                for item in a { collect(item, out); }
            }
            serde_json::Value::Object(o) => {
                if let Some(name) = o.get("name").and_then(|n| n.as_str()) {
                    out.push_str(name);
                    out.push(' ');
                }
                if let Some(entries) = o.get("entries") { collect(entries, out); }
                for (_k, v) in o.iter() { if _k != "entries" { collect(v, out); } }
            }
            _ => {}
        }
    }
    let mut out = String::new();
    collect(val, &mut out);
    out
}