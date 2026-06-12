use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Background {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub skill_proficiencies: Option<Value>,
    pub tool_proficiencies: Option<Value>,
    pub language_count: Option<i32>,
    pub starting_equipment: Option<Value>,
    pub ability_bonuses: Option<Value>,
    pub grants_bonus_feat: Option<bool>,
    pub granted_feat_id: Option<i32>,
    pub entries: Value,
}
