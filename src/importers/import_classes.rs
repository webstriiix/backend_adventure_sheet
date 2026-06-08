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
                        // Prefer explicit target from the feature ref; fall back to the current class
                        let feat = parsed.feature_ref;
                        let target_class_id = if !feat.class_name.is_empty() {
                            match get_class_id(pool, &feat.class_name, &feat.class_source).await {
                                Ok(id) => id,
                                Err(_) => class_id,
                            }
                        } else {
                            class_id
                        };

                        // We only update the class_features is_subclass_gate flag here.
                        // If the featureRef targets a different class, resolve it; otherwise use the current class.
                        sqlx::query!(
                            r#"
                            UPDATE class_features
                            SET is_subclass_gate = $1
                            WHERE name = $2 AND class_id = $3 AND level = $4
                            "#,
                            parsed.gain_subclass,
                            feat.name,
                            target_class_id,
                            feat.level as i32,
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

                let sc_additional_spells = sc.get("additionalSpells");

                sqlx::query!(
                    r#"
                    INSERT INTO subclasses
                        (name, short_name, source_id, class_id, unlock_level, fluff_image_url, additional_spells)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    ON CONFLICT (short_name, source_id, class_id)
                    DO UPDATE SET
                        name = EXCLUDED.name,
                        fluff_image_url = EXCLUDED.fluff_image_url,
                        additional_spells = CASE WHEN EXCLUDED.additional_spells IS NULL OR EXCLUDED.additional_spells = 'null'::jsonb THEN subclasses.additional_spells ELSE EXCLUDED.additional_spells END
                    "#,
                    sc["name"].as_str().unwrap_or(""),
                    sc["shortName"].as_str().unwrap_or(""),
                    source_id,
                    class_id,
                    unlock_level,
                    fluff_image,
                    sc_additional_spells,
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

    // --- Map class-feature gates to subclass features ---
    if let Some(classes) = data["class"].as_array() {
        for cls in classes {
            let class_source = cls["source"].as_str().unwrap_or("PHB");
            if let Ok(class_id) = get_class_id(pool, cls["name"].as_str().unwrap_or(""), class_source).await {
                if let Some(features) = cls["classFeatures"].as_array() {
                    for entry in features {
                        if let Some(parsed) = parse_class_feature_entry(entry) {
                            if !parsed.gain_subclass { continue; }

                            // find the class_feature id referenced by this entry
                            let feature_name = parsed.feature_ref.name;
                            let feature_level = parsed.feature_ref.level as i32;
                            let feature_source = parsed.feature_ref.class_source;

                            if let Ok(source_id) = get_source_id(pool, &feature_source).await {
                                if let Some(cf_row) = sqlx::query!(
                                    "SELECT cf.id FROM class_features cf JOIN sources s ON s.id = cf.source_id WHERE cf.name = $1 AND cf.class_id = $2 AND s.slug = $3",
                                    feature_name,
                                    class_id,
                                    feature_source
                                ).fetch_optional(pool).await?
                                {
                                    let class_feature_id = cf_row.id;

                                    // find subclass_features for this class that occur at the same level
                                    let scf_rows = sqlx::query!(
                                        "SELECT scf.id FROM subclass_features scf JOIN subclasses sc ON sc.id = scf.subclass_id WHERE sc.class_id = $1 AND scf.level = $2",
                                        class_id,
                                        feature_level
                                    ).fetch_all(pool).await?;

                                    for scf in scf_rows {
                                        sqlx::query!(
                                            "INSERT INTO class_feature_subclass_map (class_feature_id, subclass_feature_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
                                            class_feature_id,
                                            scf.id,
                                        )
                                        .execute(pool)
                                        .await?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
