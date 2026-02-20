use serde::{Deserialize, Serialize};
use serde_json::Value;

/// class row from the DB
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Class {
    pub id: i32,
    pub name: String,
    pub source_slug: String,
    pub hit_die: i32,
    pub proficiency_saves: Vec<String>,
    pub spellcasting_ability: Option<String>,
    pub caster_progression: Option<String>,
    pub weapon_proficiencies: Vec<String>,
    pub armor_proficiencies: Vec<String>,
    pub skill_choices: Value, // JSONB
    pub starting_equipment: Value,
    pub multiclass_requirements: Option<Value>,
    pub class_table: Value, // JSONB — level/slot progression
    pub subclass_title: String,
    pub edition: Option<String>,
}

/// One row of class_feature table
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClassFeature {
    pub id: i32,
    pub name: String,
    pub source_slug: String,
    pub class_name: String,
    pub level: i32,
    pub entries: Value,
    pub is_subclass_gate: bool, // true = "pick subclass at this level"
}

/// subclass table
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Subclass {
    pub id: i32,
    pub name: String,
    pub short_name: String,
    pub source_slug: String,
    pub class_name: String,
    pub class_source: String,
    pub unlock_level: i32,
    pub fluff_text: Option<String>,
    pub fluff_image_url: Option<String>,
}

/// Feature belonging to specific class
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct SubclassFeature {
    pub id: i32,
    pub name: String,
    pub source_slug: String,
    pub subclass_short_name: String,
    pub subclass_source: String,
    pub class_name: String,
    pub level: i32,
    pub header: Option<i32>, // display indent level
    pub entries: Value,
}

/// Class detail view for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct ClassDetail {
    pub class: Class,
    pub features: Vec<ClassFeature>, // sorted by level
    pub subclasses: Vec<SubclassWithFeatures>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubclassWithFeatures {
    pub subclass: Subclass,
    pub features: Vec<SubclassFeature>, // sorted by level
}
