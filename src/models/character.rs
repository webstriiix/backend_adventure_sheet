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
    pub class_id: Option<i32>,
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
    pub death_saves_successes: i32,
    pub death_saves_failures: i32,
    pub cp: i32,
    pub sp: i32,
    pub ep: i32,
    pub gp: i32,
    pub pp: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CharacterSpellSlot {
    pub character_id: Uuid,
    pub slot_level: i32,
    pub expended: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CharacterHitDice {
    pub character_id: Uuid,
    pub die_size: i32,
    pub expended: i32,
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
    pub bonus_feat_id: Option<i32>,
    pub background_feat_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCharacter {
    pub name: String,
    pub class_id: Option<i32>,
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
    pub inspiration: Option<bool>,
    pub notes: Option<String>,
    pub death_saves_successes: Option<i32>,
    pub death_saves_failures: Option<i32>,
    pub cp: Option<i32>,
    pub sp: Option<i32>,
    pub ep: Option<i32>,
    pub gp: Option<i32>,
    pub pp: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSpellSlot {
    pub expended: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHitDice {
    pub expended: i32,
}

#[derive(Debug, Deserialize)]
pub struct ShortRestRequest {
    pub hit_dice_spent: std::collections::HashMap<i32, i32>, // mapping die_size to amount spent
}

#[derive(Debug, Deserialize)]
pub struct AddClassRequest {
    pub class_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClassLevelRequest {
    pub level: i32,
    pub subclass_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AsiChoiceRequest {
    pub bump_str: Option<i32>,
    pub bump_dex: Option<i32>,
    pub bump_con: Option<i32>,
    pub bump_int: Option<i32>,
    pub bump_wis: Option<i32>,
    pub bump_cha: Option<i32>,
    pub feat_id: Option<i32>,
    pub source_type: Option<String>,
}
