use super::import_helpers::{get_source_id, upsert_source};
use bigdecimal::FromPrimitive;
use serde_json::{Value, json};
use sqlx::PgPool;

pub async fn import_items(pool: &PgPool, data: &Value) -> anyhow::Result<()> {
    let mut all_items = Vec::new();
    if let Some(items) = data["item"].as_array() {
        all_items.extend(items.iter());
    }
    if let Some(base_items) = data["baseitem"].as_array() {
        all_items.extend(base_items.iter());
    }

    for i in all_items {
        let source_slug = i["source"].as_str().unwrap_or("PHB");
        upsert_source(pool, source_slug, false).await?;
        let source_id = get_source_id(pool, source_slug).await?;

        sqlx::query!(
            r#"
            INSERT INTO items (
                name, source_id, type, rarity, weight, value_cp, damage,
                armor_class, properties, requires_attune, entries, is_magic
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (name, source_id) DO UPDATE SET
                entries = EXCLUDED.entries
            "#,
            i["name"].as_str().unwrap_or(""),
            source_id,
            i["type"].as_str(),
            i["rarity"].as_str(),
            i["weight"]
                .as_f64()
                .map(bigdecimal::BigDecimal::from_f64)
                .flatten(),
            i["value"].as_i64().map(|v| v as i32),
            i.get("dmg1")
                .map(|d| json!({"dmg1": d, "dmgType": i["dmgType"]})),
            i["ac"].as_i64().map(|v| v as i32),
            &i["property"]
                .as_array()
                .map(|a| a
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>())
                .unwrap_or_default(),
            i["reqAttune"].as_bool().unwrap_or(false),
            i["entries"],
            i.get("wondrous")
                .map(|w| w.as_bool().unwrap_or(false))
                .unwrap_or(false)
                || i["rarity"]
                    .as_str()
                    .map(|r| r != "none" && r != "unknown")
                    .unwrap_or(false),
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
