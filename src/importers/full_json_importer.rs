use crate::importers::pipe_parser::{parse_class_feature_entry, parse_feature_ref};
use bigdecimal::FromPrimitive;
use serde_json::{Value, json};
use sqlx::PgPool;

pub async fn import_everything(pool: &PgPool, raw: &str) -> anyhow::Result<()> {
    let data: Value = serde_json::from_str(raw)?;

    // 0. Import sources first (from meta or deduced)
    // For now we rely on the loops below to upsert sources as needed.

    // 1. Classes & Subclasses
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

    if let Some(features) = data["classFeature"].as_array() {
        for feat in features {
            let source_slug = feat["source"].as_str().unwrap_or("PHB");
            upsert_source(pool, source_slug, false).await?;
            let source_id = get_source_id(pool, source_slug).await?;
            // class might not exist if we only have the feature file, but we assume dependencies are loaded
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
                    DO UPDATE SET entries = EXCLUDED.entries, level = EXCLUDED.level
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

                // Store fluff if available
                let fluff_image = sc["fluff"]
                    .get("images")
                    .and_then(|i| i.as_array())
                    .and_then(|a| a.first())
                    .and_then(|o| o["href"]["url"].as_str())
                    .map(String::from);

                // For now we don't have fluff_text logic easily available from this JSON structure
                // without cross-referencing, but let's save what we can.

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
                        DO UPDATE SET entries = EXCLUDED.entries, level = EXCLUDED.level
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

    // 2. Races
    if let Some(races) = data["race"].as_array() {
        for r in races {
            let source_slug = r["source"].as_str().unwrap_or("PHB");
            upsert_source(pool, source_slug, false).await?;
            let source_id = get_source_id(pool, source_slug).await?;

            sqlx::query!(
                r#"
                INSERT INTO races (
                    name, source_id, size, speed, ability_bonuses, entries,
                    age_description, alignment_description,
                    skill_proficiencies, language_proficiencies, trait_tags
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (name, source_id) DO UPDATE SET
                    speed = EXCLUDED.speed,
                    entries = EXCLUDED.entries
                "#,
                r["name"].as_str().unwrap_or(""),
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
                        .map(|_| "See entries".to_string())), // Simplification
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
            )
            .execute(pool)
            .await?;
        }
    }

    // 2b. Subraces
    if let Some(subraces) = data["subrace"].as_array() {
        for sr in subraces {
            let source_slug = sr["source"].as_str().unwrap_or("PHB");
            upsert_source(pool, source_slug, false).await?;
            let source_id = get_source_id(pool, source_slug).await?;

            // raceName/raceSource can be at top level or inside _copy
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

    // 3. Backgrounds
    if let Some(bgs) = data["background"].as_array() {
        for bg in bgs {
            let source_slug = bg["source"].as_str().unwrap_or("PHB");
            upsert_source(pool, source_slug, false).await?;
            let source_id = get_source_id(pool, source_slug).await?;

            sqlx::query!(
                r#"
                INSERT INTO backgrounds (
                    name, source_id, skill_proficiencies, tool_proficiencies,
                    language_count, starting_equipment, entries
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (name, source_id) DO UPDATE SET
                    entries = EXCLUDED.entries
                "#,
                bg["name"].as_str().unwrap_or(""),
                source_id,
                bg.get("skillProficiencies"),
                bg.get("toolProficiencies"),
                bg.get("languageProficiencies")
                    .and_then(|l| l.as_array())
                    .map(|a| a.len() as i32)
                    .or(Some(0)), // simplified logic
                bg.get("startingEquipment"),
                bg["entries"],
            )
            .execute(pool)
            .await?;
        }
    }

    // 4. Spells
    if let Some(spells) = data["spell"].as_array() {
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
    }

    // 5. Items
    if let Some(items) = data["item"].as_array() {
        for i in items {
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
    }

    // 6. Monsters
    if let Some(monsters) = data["monster"].as_array() {
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
    }
    // 7. Optional Features
    if let Some(opt_features) = data["optionalfeature"].as_array() {
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
    }

    // 8. Feats
    if let Some(feats) = data["feat"].as_array() {
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
    }
    Ok(())
}

// ── Helper DB functions ────────────────────────────────────────────────

async fn upsert_source(pool: &PgPool, slug: &str, is_homebrew: bool) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO sources (slug, full_name, is_homebrew)
        VALUES ($1, $1, $2)
        ON CONFLICT (slug) DO NOTHING
        "#,
        slug,
        is_homebrew,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn get_source_id(pool: &PgPool, slug: &str) -> anyhow::Result<i32> {
    let row = sqlx::query!("SELECT id FROM sources WHERE slug = $1", slug)
        .fetch_one(pool)
        .await?;
    Ok(row.id)
}

async fn get_class_id(pool: &PgPool, name: &str, source_slug: &str) -> anyhow::Result<i32> {
    let row = sqlx::query!(
        "SELECT c.id FROM classes c JOIN sources s ON s.id=c.source_id WHERE c.name=$1 AND s.slug=$2",
        name, source_slug
    ).fetch_one(pool).await?;
    Ok(row.id)
}

async fn get_subclass_id(
    pool: &PgPool,
    short_name: &str,
    source_slug: &str,
    class_id: i32,
) -> anyhow::Result<i32> {
    let row = sqlx::query!(
        r#"
        SELECT sc.id FROM subclasses sc
        JOIN sources s ON s.id = sc.source_id
        WHERE sc.short_name = $1 AND s.slug = $2 AND sc.class_id = $3
        "#,
        short_name,
        source_slug,
        class_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

async fn get_race_id(pool: &PgPool, name: &str, source_slug: &str) -> anyhow::Result<i32> {
    let row = sqlx::query!(
        "SELECT r.id FROM races r JOIN sources s ON s.id=r.source_id WHERE r.name=$1 AND s.slug=$2",
        name, source_slug
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

async fn get_spell_id(pool: &PgPool, name: &str, source_slug: &str) -> anyhow::Result<i32> {
    let row = sqlx::query!(
        "SELECT sp.id FROM spells sp JOIN sources s ON s.id=sp.source_id WHERE sp.name=$1 AND s.slug=$2",
        name, source_slug
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

async fn upsert_class(pool: &PgPool, cls: &Value, source_id: i32) -> anyhow::Result<i32> {
    let row = sqlx::query!(
        r#"
        INSERT INTO classes (
            name, source_id, hit_die, proficiency_saves,
            spellcasting_ability, caster_progression,
            skill_choices, starting_equipment, multiclass_requirements,
            class_table, subclass_title, edition
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
        ON CONFLICT (name, source_id) DO UPDATE
            SET hit_die = EXCLUDED.hit_die
        RETURNING id
        "#,
        cls["name"].as_str().unwrap_or(""),
        source_id,
        cls["hd"]["faces"].as_i64().unwrap_or(8) as i32,
        &cls["proficiency"]
            .as_array()
            .map(|a| a
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>())
            .unwrap_or_default(),
        cls["spellcastingAbility"].as_str(),
        cls["casterProgression"].as_str(),
        cls["startingProficiencies"]["skills"],
        cls["startingEquipment"],
        cls.get("multiclassing"),
        cls["classTableGroups"],
        cls["subclassTitle"].as_str().unwrap_or("Subclass"),
        cls["edition"].as_str(),
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

/// Import spell-to-class mappings from spells/sources.json
/// Structure: { "PHB": { "SpellName": { "class": [{"name":"Wizard","source":"PHB"}], "classVariant": [...] } } }
pub async fn import_spell_classes(pool: &PgPool, raw: &str) -> anyhow::Result<()> {
    let data: Value = serde_json::from_str(raw)?;

    let obj = data.as_object().ok_or_else(|| anyhow::anyhow!("Expected top-level object"))?;

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
