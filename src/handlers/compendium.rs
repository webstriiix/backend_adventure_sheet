use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::{
    db::AppState,
    error::Result,
    models::{
        backgrounds::Background, items::Item, monsters::Monster,
        optional_features::OptionalFeature, races::Race, spells::Spell,
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
        LIMIT 100
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
        LIMIT 100
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
