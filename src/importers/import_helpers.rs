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

pub async fn upsert_class(pool: &PgPool, cls: &Value, source_id: i32) -> anyhow::Result<i32> {
    let name = cls["name"].as_str().unwrap_or("");
    let asi_levels = match name {
        "Fighter" => vec![4, 6, 8, 12, 14, 16, 19],
        "Rogue" => vec![4, 8, 10, 12, 16, 18],
        _ => vec![4, 8, 12, 16, 19],
    };

    let hit_die = cls["hd"]["faces"].as_i64().unwrap_or(-1) as i32;
    let spellcasting_ability = cls["spellcastingAbility"].as_str();
    let caster_progression = cls["casterProgression"].as_str();
    let editon = cls["edition"].as_str();

    let row = sqlx::query!(
        r#"
        INSERT INTO classes (
            name, source_id, hit_die, proficiency_saves,
            spellcasting_ability, caster_progression,
            skill_choices, starting_equipment, multiclass_requirements,
            class_table, subclass_title, edition, asi_levels,
            weapon_proficiencies, armor_proficiencies
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
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
                edition = COALESCE(EXCLUDED.edition, classes.edition)
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
        editon,
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
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}
