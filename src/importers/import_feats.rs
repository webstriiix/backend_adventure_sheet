use super::import_helpers::{get_source_id, upsert_source};
use serde_json::Value;
use sqlx::PgPool;

pub async fn import_feats(pool: &PgPool, data: &Value) -> anyhow::Result<()> {
    let feats = match data["feat"].as_array() {
        Some(f) => f,
        None => return Ok(()),
    };

    for f in feats {
        let source_slug = f["source"].as_str().unwrap_or("PHB");
        upsert_source(pool, source_slug, false).await?;
        let source_id = get_source_id(pool, source_slug).await?;

        sqlx::query!(
            r#"
            INSERT INTO feats (
                name, source_id, page, prerequisite, ability, skill_proficiencies,
                resist, additional_spells, has_uses, uses_formula, recharge_on, entries
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (name, source_id) DO UPDATE SET
                entries = EXCLUDED.entries
            "#,
            f["name"].as_str().unwrap_or(""),
            source_id,
            f["page"].as_i64().map(|v| v as i32),
            f.get("prerequisite"),
            f.get("ability"),
            f.get("skillProficiencies"),
            &f["resist"]
                .as_array()
                .map(|a| a
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>())
                .unwrap_or_default(),
            f.get("additionalSpells"),
            f.get("has_uses")
                .map(|v| v.as_bool().unwrap_or(false))
                .unwrap_or(false),
            f["uses"]
                .as_array()
                .and_then(|a| a.get(0))
                .and_then(|v| v["formula"].as_str()),
            f["recharge"].as_str(),
            f["entries"],
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
