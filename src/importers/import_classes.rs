use super::import_helpers::{
    get_class_id, get_source_id, get_subclass_id, upsert_class, upsert_source,
};
use crate::importers::pipe_parser::{parse_class_feature_entry, parse_feature_ref};
use serde_json::Value;
use sqlx::PgPool;

pub async fn import_classes(pool: &PgPool, data: &Value) -> anyhow::Result<()> {
    // Classes + class-level feature gates
    if let Some(classes) = data["class"].as_array() {
        for cls in classes {
            let source_slug = cls["source"].as_str().unwrap_or("PHB");
            upsert_source(pool, source_slug, false).await?;
            let source_id = get_source_id(pool, source_slug).await?;
            let class_id = upsert_class(pool, cls, source_id).await?;

            if let Some(features) = cls["classFeatures"].as_array() {
                for entry in features {
                    if let Some(parsed) = parse_class_feature_entry(entry) {
                        sqlx::query!(
                            r#"
                            UPDATE class_features
                            SET is_subclass_gate = $1
                            WHERE name = $2 AND class_id = $3 AND level = $4
                            "#,
                            parsed.gain_subclass,
                            parsed.feature_ref.name,
                            class_id,
                            parsed.feature_ref.level as i32,
                        )
                        .execute(pool)
                        .await?;
                    }
                }
            }
        }
    }

    // Class features
    if let Some(features) = data["classFeature"].as_array() {
        for feat in features {
            let source_slug = feat["source"].as_str().unwrap_or("PHB");
            upsert_source(pool, source_slug, false).await?;
            let source_id = get_source_id(pool, source_slug).await?;

            if let Ok(class_id) = get_class_id(
                pool,
                feat["className"].as_str().unwrap_or(""),
                feat["classSource"].as_str().unwrap_or("PHB"),
            )
            .await
            {
                sqlx::query!(
                    r#"
                    INSERT INTO class_features
                        (name, source_id, class_id, level, entries, is_subclass_gate)
                    VALUES ($1,$2,$3,$4,$5, false)
                    ON CONFLICT (name, source_id, class_id)
                    DO UPDATE SET 
                        entries = CASE 
                            WHEN EXCLUDED.entries IS NULL OR EXCLUDED.entries = 'null'::jsonb THEN class_features.entries 
                            ELSE EXCLUDED.entries 
                        END,
                        level = EXCLUDED.level
                    "#,
                    feat["name"].as_str().unwrap_or(""),
                    source_id,
                    class_id,
                    feat["level"].as_i64().unwrap_or(1) as i32,
                    feat["entries"],
                )
                .execute(pool)
                .await?;
            }
        }
    }

    // Subclasses
    if let Some(subclasses) = data["subclass"].as_array() {
        for sc in subclasses {
            let source_slug = sc["source"].as_str().unwrap_or("PHB");
            let class_source = sc["classSource"].as_str().unwrap_or("PHB");
            upsert_source(pool, source_slug, false).await?;
            let source_id = get_source_id(pool, source_slug).await?;

            if let Ok(class_id) =
                get_class_id(pool, sc["className"].as_str().unwrap_or(""), class_source).await
            {
                let unlock_level = sc["subclassFeatures"]
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .and_then(|s| parse_feature_ref(s))
                    .map(|r| r.level as i32)
                    .unwrap_or(3);

                let fluff_image = sc["fluff"]
                    .get("images")
                    .and_then(|i| i.as_array())
                    .and_then(|a| a.first())
                    .and_then(|o| o["href"]["url"].as_str())
                    .map(String::from);

                sqlx::query!(
                    r#"
                    INSERT INTO subclasses
                        (name, short_name, source_id, class_id, unlock_level, fluff_image_url)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    ON CONFLICT (short_name, source_id, class_id)
                    DO UPDATE SET name = EXCLUDED.name, fluff_image_url = EXCLUDED.fluff_image_url
                    "#,
                    sc["name"].as_str().unwrap_or(""),
                    sc["shortName"].as_str().unwrap_or(""),
                    source_id,
                    class_id,
                    unlock_level,
                    fluff_image,
                )
                .execute(pool)
                .await?;
            }
        }
    }

    // Subclass features
    if let Some(sc_features) = data["subclassFeature"].as_array() {
        for feat in sc_features {
            let source_slug = feat["source"].as_str().unwrap_or("PHB");
            let class_source = feat["classSource"].as_str().unwrap_or("PHB");
            let sc_source = feat["subclassSource"].as_str().unwrap_or("PHB");
            let sc_short_name = feat["subclassShortName"].as_str().unwrap_or("");

            upsert_source(pool, source_slug, sc_source != "PHB").await?;
            let source_id = get_source_id(pool, source_slug).await?;

            if let Ok(class_id) =
                get_class_id(pool, feat["className"].as_str().unwrap_or(""), class_source).await
            {
                if let Ok(subclass_id) =
                    get_subclass_id(pool, sc_short_name, sc_source, class_id).await
                {
                    sqlx::query!(
                        r#"
                        INSERT INTO subclass_features
                            (name, source_id, subclass_id, level, header, entries)
                        VALUES ($1,$2,$3,$4,$5,$6)
                        ON CONFLICT (name, source_id, subclass_id)
                        DO UPDATE SET 
                            entries = CASE 
                                WHEN EXCLUDED.entries IS NULL OR EXCLUDED.entries = 'null'::jsonb THEN subclass_features.entries 
                                ELSE EXCLUDED.entries 
                            END,
                            level = EXCLUDED.level
                        "#,
                        feat["name"].as_str().unwrap_or(""),
                        source_id,
                        subclass_id,
                        feat["level"].as_i64().unwrap_or(1) as i32,
                        feat["header"].as_i64().map(|h| h as i32),
                        feat["entries"],
                    )
                    .execute(pool)
                    .await?;
                }
            }
        }
    }

    Ok(())
}
