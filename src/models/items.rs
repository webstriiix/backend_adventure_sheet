use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::BigDecimal;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub r#type: Option<String>, // "type" is a reserved keyword
    pub rarity: Option<String>,
    pub weight: Option<BigDecimal>,
    pub value_cp: Option<i32>,
    pub damage: Option<Value>, // JSONB
    pub armor_class: Option<i32>,
    pub properties: Vec<String>,
    pub requires_attune: bool,
    pub entries: Option<Value>, // JSONB
    pub is_magic: bool,
}
