use super::import_helpers::{get_source_id, upsert_source};
use serde_json::Value;
use sqlx::PgPool;

pub async fn import_monsters(pool: &PgPool, data: &Value) -> anyhow::Result<()> {
    let monsters = match data["monster"].as_array() {
        Some(m) => m,
        None => return Ok(()),
    };

    for m in monsters {
        let source_slug = m["source"].as_str().unwrap_or("PHB");
        upsert_source(pool, source_slug, false).await?;
        let source_id = get_source_id(pool, source_slug).await?;

        let hp_avg = m["hp"]["average"].as_i64().map(|v| v as i32);
        let hp_formula = m["hp"]["formula"].as_str().map(String::from);

        sqlx::query!(
            r#"
            INSERT INTO monsters (
                name, source_id, size, type, alignment, ac, hp_average, hp_formula,
                speed, str, dex, con, int, wis, cha, skills, senses, passive,
                cr, traits, actions, reactions
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            ON CONFLICT (name, source_id) DO UPDATE SET
                ac = EXCLUDED.ac,
                hp_average = EXCLUDED.hp_average
            "#,
            m["name"].as_str().unwrap_or(""),
            source_id,
            &m["size"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>()).unwrap_or_default(),
            m["type"].as_str().or_else(|| m["type"]["type"].as_str()),
            &m["alignment"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>()).unwrap_or_default(),
            m.get("ac"),
            hp_avg,
            hp_formula,
            m.get("speed"),
            m["str"].as_i64().map(|v| v as i32),
            m["dex"].as_i64().map(|v| v as i32),
            m["con"].as_i64().map(|v| v as i32),
            m["int"].as_i64().map(|v| v as i32),
            m["wis"].as_i64().map(|v| v as i32),
            m["cha"].as_i64().map(|v| v as i32),
            m.get("skill"),
            &m["senses"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>()).unwrap_or_default(),
            m["passive"].as_i64().map(|v| v as i32),
            m["cr"].as_str().or_else(|| m["cr"]["cr"].as_str()),
            m.get("trait"),
            m.get("action"),
            m.get("reaction"),
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
