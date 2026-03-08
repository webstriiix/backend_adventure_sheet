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
            }
        }
    }

    Ok(())
}
