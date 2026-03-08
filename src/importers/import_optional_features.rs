use super::import_helpers::{get_source_id, upsert_source};
use serde_json::Value;
use sqlx::PgPool;

pub async fn import_optional_features(pool: &PgPool, data: &Value) -> anyhow::Result<()> {
    let opt_features = match data["optionalfeature"].as_array() {
        Some(o) => o,
        None => return Ok(()),
    };

    for of in opt_features {
        let source_slug = of["source"].as_str().unwrap_or("PHB");
        upsert_source(pool, source_slug, false).await?;
        let source_id = get_source_id(pool, source_slug).await?;

        // featureType is an array — insert one row per type
        if let Some(types) = of["featureType"].as_array() {
            for ft in types {
                if let Some(feature_type) = ft.as_str() {
                    sqlx::query!(
                        r#"
                        INSERT INTO optional_features
                            (name, source_id, feature_type, prerequisite, entries)
                        VALUES ($1, $2, $3, $4, $5)
                        ON CONFLICT (name, source_id, feature_type) DO UPDATE SET
                            entries = EXCLUDED.entries,
                            prerequisite = EXCLUDED.prerequisite
                        "#,
                        of["name"].as_str().unwrap_or(""),
                        source_id,
                        feature_type,
                        of.get("prerequisite"),
                        of["entries"],
                    )
                    .execute(pool)
                    .await?;
                }
            }
        }
    }

    Ok(())
}
