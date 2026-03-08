use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Race {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub size: Vec<String>,
    pub speed: Value,           // JSONB
    pub ability_bonuses: Value, // JSONB
    pub age_description: Option<String>,
    pub alignment_description: Option<String>,
    pub skill_proficiencies: Option<Value>,    // JSONB
    pub language_proficiencies: Option<Value>, // JSONB
    pub trait_tags: Vec<String>,
    pub grants_bonus_feat: Option<bool>,
    pub entries: Value, // JSONB
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Subrace {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub race_id: i32,
    pub speed: Option<Value>,           // JSONB
    pub ability_bonuses: Option<Value>, // JSONB
    pub entries: Value,                 // JSONB
}
