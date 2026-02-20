use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Spell {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub level: i32,
    pub school: String,
    pub casting_time: Value,               // JSONB
    pub range: Value,                      // JSONB
    pub components: Value,                 // JSONB
    pub duration: Value,                   // JSONB
    pub entries: Value,                    // JSONB
    pub entries_higher_lvl: Option<Value>, // JSONB
    pub ritual: bool,
    pub concentration: bool,
}
