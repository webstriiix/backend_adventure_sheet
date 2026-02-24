use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Character {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub experience_pts: i32,
    pub race_id: Option<i32>,
    pub subrace_id: Option<i32>,
    pub background_id: Option<i32>,
    pub str: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wis: i32,
    pub cha: i32,
    pub max_hp: i32,
    pub current_hp: i32,
    pub temp_hp: i32,
    pub inspiration: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCharacter {
    pub name: String,
    pub class_id: i32,
    pub race_id: Option<i32>,
    pub subrace_id: Option<i32>,
    pub background_id: Option<i32>,
    pub str: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wis: i32,
    pub cha: i32,
    pub max_hp: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCharacter {
    pub name: String,
    pub experience_pts: i32,
    pub race_id: Option<i32>,
    pub subrace_id: Option<i32>,
    pub background_id: Option<i32>,
    pub str: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wis: i32,
    pub cha: i32,
    pub max_hp: i32,
    pub current_hp: i32,
    pub temp_hp: i32,
    pub inspiration: bool,
    pub notes: Option<String>,
}
