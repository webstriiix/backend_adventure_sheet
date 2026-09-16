use serde_json::Value;
use sqlx::PgPool;

pub async fn upsert_source(pool: &PgPool, slug: &str, is_homebrew: bool) -> anyhow::Result<()> {
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

pub async fn get_source_id(pool: &PgPool, slug: &str) -> anyhow::Result<i32> {
    let row = sqlx::query!("SELECT id FROM sources WHERE slug = $1", slug)
        .fetch_one(pool)
        .await?;
    Ok(row.id)
}

pub async fn get_class_id(pool: &PgPool, name: &str, source_slug: &str) -> anyhow::Result<i32> {
    let row = sqlx::query!(
        "SELECT c.id FROM classes c JOIN sources s ON s.id=c.source_id WHERE c.name=$1 AND s.slug=$2",
        name,
        source_slug
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

pub async fn get_subclass_id(
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

pub async fn get_race_id(pool: &PgPool, name: &str, source_slug: &str) -> anyhow::Result<i32> {
    let row = sqlx::query!(
        "SELECT r.id FROM races r JOIN sources s ON s.id=r.source_id WHERE r.name=$1 AND s.slug=$2",
        name,
        source_slug
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

pub async fn get_spell_id(pool: &PgPool, name: &str, source_slug: &str) -> anyhow::Result<i32> {
    let row = sqlx::query!(
        "SELECT sp.id FROM spells sp JOIN sources s ON s.id=sp.source_id WHERE sp.name=$1 AND s.slug=$2",
        name,
        source_slug
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

fn extract_spell_slots(class_table_groups: &Value) -> Value {
    if let Some(groups) = class_table_groups.as_array() {
        for group in groups {
            if let Some(rows) = group.get("rowsSpellProgression") {
                return rows.clone();
            }
        }
    }
    Value::Null
}

/// Extract ASI/Feat levels from classFeatures entries like "Ability Score Improvement|Class|Source|4".
fn extract_asi_levels(class_features: &Value) -> Vec<i32> {
    let mut levels = Vec::new();
    if let Some(arr) = class_features.as_array() {
        for entry in arr {
            let feature_ref = match entry {
                Value::String(s) => Some(s.as_str()),
                Value::Object(obj) => obj.get("classFeature").and_then(|v| v.as_str()),
                _ => None,
            };
            if let Some(s) = feature_ref {
                if s.starts_with("Ability Score Improvement") {
                    // Format: "Ability Score Improvement|Class|Source|Level" or
                    //        "Ability Score Improvement|Class|Source|Level|Source2|Level2"
                    let parts: Vec<&str> = s.split('|').collect();
                    if let Some(level_str) = parts.get(3) {
                        if let Ok(level) = level_str.parse::<i32>() {
                            levels.push(level);
                        }
                    }
                }
            }
        }
    }
    levels.sort();
    levels.dedup();
    levels
}

/// Extract primary abilities from `primaryAbility` JSON.
/// Handles both `["int"]` and `[{"int": true}]` shapes.
fn extract_primary_ability(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    if let Some(s) = v.as_str() {
                        return Some(s.to_lowercase());
                    }
                    if let Some(obj) = v.as_object() {
                        return obj
                            .iter()
                            .find(|(_, val)| val.as_bool() == Some(true))
                            .map(|(key, _)| key.to_lowercase());
                    }
                    None
                })
                .collect()
        })
        .unwrap_or_default()
}

fn int_array(value: &Value) -> Vec<i32> {
    value
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().map(|n| n as i32))
                .collect()
        })
        .unwrap_or_default()
}

pub async fn upsert_class(pool: &PgPool, cls: &Value, source_id: i32) -> anyhow::Result<i32> {
    let name = cls["name"].as_str().unwrap_or("");
    let asi_levels = extract_asi_levels(&cls["classFeatures"]);

    let hit_die = cls["hd"]["faces"].as_i64().unwrap_or(-1) as i32;
    let spellcasting_ability = cls["spellcastingAbility"].as_str();
    let caster_progression = cls["casterProgression"].as_str();
    let edition = cls["edition"].as_str();
    let spell_slots = extract_spell_slots(&cls["classTableGroups"]);
    let additional_spells = cls.get("additionalSpells");

    // XPHB / 2024 fields
    let primary_ability = extract_primary_ability(&cls["primaryAbility"]);
    let prepared_spells_progression = int_array(&cls["preparedSpellsProgression"]);
    let prepared_spells_change = cls["preparedSpellsChange"].as_str();
    let cantrip_progression = int_array(&cls["cantripProgression"]);
    let spells_known_progression_fixed = int_array(&cls["spellsKnownProgressionFixed"]);
    let feat_progression = cls.get("featProgression");

    let row = sqlx::query!(
        r#"
        INSERT INTO classes (
            name, source_id, hit_die, proficiency_saves,
            spellcasting_ability, caster_progression,
            skill_choices, starting_equipment, multiclass_requirements,
            class_table, subclass_title, edition, asi_levels,
            weapon_proficiencies, armor_proficiencies,
            spell_slots, additional_spells,
            primary_ability, prepared_spells_progression, prepared_spells_change,
            cantrip_progression, spells_known_progression_fixed, feat_progression
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23)
        ON CONFLICT (name, source_id) DO UPDATE
            SET hit_die = CASE WHEN EXCLUDED.hit_die = -1 THEN classes.hit_die ELSE EXCLUDED.hit_die END,
                asi_levels = CASE WHEN EXCLUDED.asi_levels IS NULL THEN classes.asi_levels ELSE EXCLUDED.asi_levels END,
                spellcasting_ability = COALESCE(EXCLUDED.spellcasting_ability, classes.spellcasting_ability),
                caster_progression = COALESCE(EXCLUDED.caster_progression, classes.caster_progression),
                weapon_proficiencies = CASE WHEN cardinality(EXCLUDED.weapon_proficiencies) = 0 THEN classes.weapon_proficiencies ELSE EXCLUDED.weapon_proficiencies END,
                armor_proficiencies = CASE WHEN cardinality(EXCLUDED.armor_proficiencies) = 0 THEN classes.armor_proficiencies ELSE EXCLUDED.armor_proficiencies END,
                skill_choices = CASE WHEN EXCLUDED.skill_choices IS NULL OR EXCLUDED.skill_choices = 'null'::jsonb THEN classes.skill_choices ELSE EXCLUDED.skill_choices END,
                starting_equipment = CASE WHEN EXCLUDED.starting_equipment IS NULL OR EXCLUDED.starting_equipment = 'null'::jsonb THEN classes.starting_equipment ELSE EXCLUDED.starting_equipment END,
                multiclass_requirements = CASE WHEN EXCLUDED.multiclass_requirements IS NULL OR EXCLUDED.multiclass_requirements = 'null'::jsonb THEN classes.multiclass_requirements ELSE EXCLUDED.multiclass_requirements END,
                class_table = CASE WHEN EXCLUDED.class_table IS NULL OR EXCLUDED.class_table = 'null'::jsonb THEN classes.class_table ELSE EXCLUDED.class_table END,
                edition = COALESCE(EXCLUDED.edition, classes.edition),
                spell_slots = CASE WHEN EXCLUDED.spell_slots IS NULL OR EXCLUDED.spell_slots = 'null'::jsonb THEN classes.spell_slots ELSE EXCLUDED.spell_slots END,
                additional_spells = CASE WHEN EXCLUDED.additional_spells IS NULL OR EXCLUDED.additional_spells = 'null'::jsonb THEN classes.additional_spells ELSE EXCLUDED.additional_spells END,
                primary_ability = CASE WHEN EXCLUDED.primary_ability IS NULL THEN classes.primary_ability ELSE EXCLUDED.primary_ability END,
                prepared_spells_progression = CASE WHEN EXCLUDED.prepared_spells_progression IS NULL THEN classes.prepared_spells_progression ELSE EXCLUDED.prepared_spells_progression END,
                prepared_spells_change = COALESCE(EXCLUDED.prepared_spells_change, classes.prepared_spells_change),
                cantrip_progression = CASE WHEN EXCLUDED.cantrip_progression IS NULL THEN classes.cantrip_progression ELSE EXCLUDED.cantrip_progression END,
                spells_known_progression_fixed = CASE WHEN EXCLUDED.spells_known_progression_fixed IS NULL THEN classes.spells_known_progression_fixed ELSE EXCLUDED.spells_known_progression_fixed END,
                feat_progression = CASE WHEN EXCLUDED.feat_progression IS NULL OR EXCLUDED.feat_progression = 'null'::jsonb THEN classes.feat_progression ELSE EXCLUDED.feat_progression END
        RETURNING id
        "#,
        name,
        source_id,
        hit_die,
        &cls["proficiency"]
            .as_array()
            .map(|a| a
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>())
            .unwrap_or_default(),
        spellcasting_ability,
        caster_progression,
        cls["startingProficiencies"]["skills"],
        cls["startingEquipment"],
        cls.get("multiclassing"),
        cls["classTableGroups"],
        cls["subclassTitle"].as_str().unwrap_or("Subclass"),
        edition,
        &asi_levels,
        &cls["startingProficiencies"]["weapons"]
            .as_array()
            .map(|a| a
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>())
            .unwrap_or_default(),
        &cls["startingProficiencies"]["armor"]
            .as_array()
            .map(|a| a
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>())
            .unwrap_or_default(),
        &spell_slots,
        additional_spells,
        &primary_ability,
        &prepared_spells_progression,
        prepared_spells_change,
        &cantrip_progression,
        &spells_known_progression_fixed,
        feat_progression,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}
