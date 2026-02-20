use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Monster {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub size: Option<Vec<String>>,
    pub r#type: Option<String>, // "type" is a reserved keyword
    pub alignment: Option<Vec<String>>,
    pub ac: Option<Value>, // JSONB
    pub hp_average: Option<i32>,
    pub hp_formula: Option<String>,
    pub speed: Option<Value>, // JSONB
    pub str: Option<i32>,
    pub dex: Option<i32>,
    pub con: Option<i32>,
    pub int: Option<i32>,
    pub wis: Option<i32>,
    pub cha: Option<i32>,
    pub skills: Option<Value>, // JSONB
    pub senses: Option<Vec<String>>,
    pub passive: Option<i32>,
    pub cr: Option<String>,
    pub traits: Option<Value>,    // JSONB
    pub actions: Option<Value>,   // JSONB
    pub reactions: Option<Value>, // JSONB
}
