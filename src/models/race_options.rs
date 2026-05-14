use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RaceOption {
    pub id: i32,
    pub race_id: Option<i32>,
    pub subrace_id: Option<i32>,
    pub source_id: i32,
    pub option_type: String,
    pub choices: Option<Value>,
    pub min_choose: i32,
    pub max_choose: i32,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CharacterRaceOption {
    pub id: i32,
    pub character_id: Uuid,
    pub race_option_id: i32,
    pub selection: Value,
}