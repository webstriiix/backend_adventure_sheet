use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

use crate::{
    db::AppState,
    error::{AppError, Result},
    models::class::*,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub source: Option<String>,  // filter by "PHB", "ObjimaTallGrass"
    pub edition: Option<String>, // classic or one-dnd
}

/// GET   api/v1/classes
pub async fn list_classes(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Class>>> {
    let rows = sqlx::query_as!(
        Class,
        r#"
        SELECT
            c.id,
            c.name,
            s.slug AS source_slug,
            c.hit_die,
            c.proficiency_saves,
            c.spellcasting_ability,
            c.caster_progression,
            c.weapon_proficiencies,
            c.armor_proficiencies,
            c.skill_choices,
            c.starting_equipment,
            c.multiclass_requirements,
            c.class_table,
            c.subclass_title,
            c.edition,
            c.asi_levels
        FROM classes c
        JOIN sources s ON s.id = c.source_id
        WHERE ($1::text IS NULL OR s.slug = $1)
          AND ($2::text IS NULL OR c.edition = $2)
        ORDER BY c.name
        "#,
        q.source,
        q.edition,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

/// GET /api/v1/classes/:name/:source
/// Returns the full class: base info + all features + all subclasses + their features
pub async fn get_class_detail(
    State(state): State<AppState>,
    Path((name, source)): Path<(String, String)>,
) -> Result<Json<ClassDetail>> {
    // 1. Fetch the base class
    let class = sqlx::query_as!(
        Class,
        r#"
        SELECT
            c.id, c.name, s.slug AS source_slug,
            c.hit_die, c.proficiency_saves, c.spellcasting_ability,
            c.caster_progression, c.weapon_proficiencies, c.armor_proficiencies,
            c.skill_choices, c.starting_equipment, c.multiclass_requirements,
            c.class_table, c.subclass_title, c.edition, c.asi_levels
        FROM classes c
        JOIN sources s ON s.id = c.source_id
        WHERE c.name = $1 AND s.slug = $2
        "#,
        name,
        source,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Class {name}/{source} not found")))?;

    // 2. Fetch all class features ordered by level
    let features = sqlx::query_as!(
        ClassFeature,
        r#"
        SELECT
            cf.id, cf.name, s.slug AS source_slug,
            c.name AS class_name, cf.level,
            cf.entries, cf.is_subclass_gate
        FROM class_features cf
        JOIN classes c  ON c.id  = cf.class_id
        JOIN sources s  ON s.id  = cf.source_id
        WHERE c.id = $1
        ORDER BY cf.level, cf.name
        "#,
        class.id,
    )
    .fetch_all(&state.db)
    .await?;

    // 3. Fetch all subclasses for this class
    let subclasses_raw = sqlx::query_as!(
        Subclass,
        r#"
        SELECT
            sc.id, sc.name, sc.short_name,
            s.slug   AS source_slug,
            c.name   AS class_name,
            cs.slug  AS class_source,
            sc.unlock_level, sc.fluff_text, sc.fluff_image_url
        FROM subclasses sc
        JOIN classes c  ON c.id  = sc.class_id
        JOIN sources s  ON s.id  = sc.source_id
        JOIN sources cs ON cs.id = c.source_id
        WHERE sc.class_id = $1
        ORDER BY sc.name
        "#,
        class.id,
    )
    .fetch_all(&state.db)
    .await?;

    // 4. Fetch all subclass features in one query, group in Rust
    let all_scf = sqlx::query_as!(
        SubclassFeature,
        r#"
        SELECT
            scf.id, scf.name, s.slug AS source_slug,
            sc.short_name AS subclass_short_name,
            ss.slug       AS subclass_source,
            c.name        AS class_name,
            scf.level, scf.header, scf.entries
        FROM subclass_features scf
        JOIN subclasses sc ON sc.id  = scf.subclass_id
        JOIN classes    c  ON c.id   = sc.class_id
        JOIN sources    s  ON s.id   = scf.source_id
        JOIN sources    ss ON ss.id  = sc.source_id
        WHERE sc.class_id = $1
        ORDER BY sc.name, scf.level
        "#,
        class.id,
    )
    .fetch_all(&state.db)
    .await?;

    // 5. Group subclass features under their subclass
    let subclasses: Vec<SubclassWithFeatures> = subclasses_raw
        .into_iter()
        .map(|sc| {
            let features = all_scf
                .iter()
                .filter(|f| {
                    f.subclass_short_name == sc.short_name && f.subclass_source == sc.source_slug
                })
                .cloned()
                .collect();
            SubclassWithFeatures {
                subclass: sc,
                features,
            }
        })
        .collect();

    Ok(Json(ClassDetail {
        class,
        features,
        subclasses,
    }))
}
