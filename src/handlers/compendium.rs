use axum::{
    Json,
    extract::{Query, State, Path},
};
use crate::error::AppError;
use serde::Deserialize;

use crate::{
    db::AppState,
    error::Result,
    models::{
        backgrounds::Background, feats::Feat, items::Item, monsters::Monster,
        optional_features::OptionalFeature, races::{Race, Subrace}, spells::Spell,
    },
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub name: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OptionalFeatureQuery {
    pub name: Option<String>,
    pub source: Option<String>,
    pub feature_type: Option<String>,
}

// Spells
pub async fn list_spells(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Spell>>> {
    let rows = sqlx::query_as!(
        Spell,
        r#"
        SELECT s.* FROM spells s
        JOIN sources src ON src.id = s.source_id
        WHERE ($1::text IS NULL OR s.name ILIKE $1)
          AND ($2::text IS NULL OR src.slug = $2)
        ORDER BY s.name
        "#,
        q.name.as_ref().map(|n| format!("%{}%", n)),
        q.source,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

//  Items
pub async fn list_items(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Item>>> {
    // Note: We cast weight to f64 compatibility if needed, but sqlx should handle BigDecimal -> BigDecimal
    let rows = sqlx::query_as!(
        Item,
        r#"
        SELECT i.* FROM items i
        JOIN sources src ON src.id = i.source_id
        WHERE ($1::text IS NULL OR i.name ILIKE $1)
          AND ($2::text IS NULL OR src.slug = $2)
        ORDER BY i.name
        "#,
        q.name.as_ref().map(|n| format!("%{}%", n)),
        q.source,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

//  Monsters

pub async fn list_monsters(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Monster>>> {
    let rows = sqlx::query_as!(
        Monster,
        r#"
        SELECT m.* FROM monsters m
        JOIN sources src ON src.id = m.source_id
        WHERE ($1::text IS NULL OR m.name ILIKE $1)
          AND ($2::text IS NULL OR src.slug = $2)
        ORDER BY m.name
        LIMIT 50
        "#,
        q.name.as_ref().map(|n| format!("%{}%", n)),
        q.source,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

//  Races

pub async fn list_races(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Race>>> {
    let rows = sqlx::query_as!(
        Race,
        r#"
        SELECT r.* FROM races r
        JOIN sources src ON src.id = r.source_id
        WHERE ($1::text IS NULL OR r.name ILIKE $1)
          AND ($2::text IS NULL OR src.slug = $2)
        ORDER BY r.name
        "#,
        q.name.as_ref().map(|n| format!("%{}%", n)),
        q.source,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

//  Subraces

pub async fn list_subraces(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Subrace>>> {
    let rows = sqlx::query_as!(
        Subrace,
        r#"
        SELECT s.* FROM subraces s
        JOIN sources src ON src.id = s.source_id
        JOIN races r ON r.id = s.race_id
        WHERE ($1::text IS NULL OR s.name ILIKE $1)
          AND ($2::text IS NULL OR src.slug = $2)
        ORDER BY s.name
        "#,
        q.name.as_ref().map(|n| format!("%{}%", n)),
        q.source,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

// Race options

pub async fn list_race_options(
    State(state): State<AppState>,
    Path((name, source)): Path<(String, String)>,
) -> Result<Json<Vec<crate::models::race_options::RaceOption>>> {
    // find race id
    let row = sqlx::query!(
        "SELECT r.id FROM races r JOIN sources s ON s.id = r.source_id WHERE r.name = $1 AND s.slug = $2",
        name,
        source,
    )
    .fetch_optional(&state.db)
    .await?;

    let race_id = match row { Some(r) => r.id, None => return Err(AppError::NotFound(format!("Race {}/{} not found", name, source))) };

    let rows = sqlx::query_as!(
        crate::models::race_options::RaceOption,
        r#"
        SELECT ro.* FROM race_options ro
        WHERE (ro.race_id = $1 OR ro.race_id IS NULL) AND (ro.source_id = (SELECT id FROM sources WHERE slug = $2))
        ORDER BY ro.id
        "#,
        race_id,
        source
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

//  Backgrounds

pub async fn list_backgrounds(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Background>>> {
    let rows = sqlx::query_as!(
        Background,
        r#"
        SELECT b.* FROM backgrounds b
        JOIN sources src ON src.id = b.source_id
        WHERE ($1::text IS NULL OR b.name ILIKE $1)
          AND ($2::text IS NULL OR src.slug = $2)
        ORDER BY b.name
        "#,
        q.name.as_ref().map(|n| format!("%{}%", n)),
        q.source,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

// Feats

pub async fn list_feats(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Feat>>> {
    let rows = sqlx::query_as!(
        Feat,
        r#"
        SELECT f.* FROM feats f
        JOIN sources src ON src.id = f.source_id
        WHERE ($1::text IS NULL OR f.name ILIKE $1)
          AND ($2::text IS NULL OR src.slug = $2)
        ORDER BY f.name
        "#,
        q.name.as_ref().map(|n| format!("%{}%", n)),
        q.source,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

// Optional Features

pub async fn list_optional_features(
    State(state): State<AppState>,
    Query(q): Query<OptionalFeatureQuery>,
) -> Result<Json<Vec<OptionalFeature>>> {
    let rows = sqlx::query_as!(
        OptionalFeature,
        r#"
        SELECT of.* FROM optional_features of
        JOIN sources src ON src.id = of.source_id
        WHERE ($1::text IS NULL OR of.name ILIKE $1)
          AND ($2::text IS NULL OR src.slug = $2)
          AND ($3::text IS NULL OR of.feature_type = $3)
        ORDER BY of.name
        LIMIT 100
        "#,
        q.name.as_ref().map(|n| format!("%{}%", n)),
        q.source,
        q.feature_type,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}
