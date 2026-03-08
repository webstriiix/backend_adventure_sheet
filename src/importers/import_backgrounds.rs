use super::import_helpers::{get_source_id, upsert_source};
use serde_json::Value;
use sqlx::PgPool;

pub async fn import_backgrounds(pool: &PgPool, data: &Value) -> anyhow::Result<()> {
    let bgs = match data["background"].as_array() {
        Some(b) => b,
        None => return Ok(()),
    };

    for bg in bgs {
        let source_slug = bg["source"].as_str().unwrap_or("PHB");
        upsert_source(pool, source_slug, false).await?;
        let source_id = get_source_id(pool, source_slug).await?;

        let ability_bonuses = bg.get("ability").cloned();
        let grants_bonus_feat = bg.get("feat").is_some();

        sqlx::query!(
            r#"
            INSERT INTO backgrounds (
                name, source_id, skill_proficiencies, tool_proficiencies,
                language_count, starting_equipment, ability_bonuses, grants_bonus_feat, entries
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (name, source_id) DO UPDATE SET
                ability_bonuses = EXCLUDED.ability_bonuses,
                grants_bonus_feat = EXCLUDED.grants_bonus_feat,
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
            bg["entries"],
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
