use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Feat {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub page: Option<i32>,
    pub prerequisite: Option<Value>,
    pub ability: Option<Value>,
    pub skill_proficiencies: Option<Value>,
    pub resist: Option<Vec<String>>,
    pub additional_spells: Option<Value>,
    pub has_uses: bool,
    pub uses_formula: Option<String>,
    pub recharge_on: Option<String>,
    pub entries: Value,
}
