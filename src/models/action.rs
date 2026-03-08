use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionItem {
    pub name: String,
    pub source: Option<String>,
    pub description: Option<String>,
    pub range: Option<String>,
    pub hit_bonus: Option<String>,
    pub damage: Option<String>,
    pub max_uses: Option<i32>,
    pub current_uses: Option<i32>,
    pub reset_type: Option<String>,
    pub time: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ActionPayload {
    pub all: Vec<ActionItem>,
    pub attack: Vec<ActionItem>,
    pub action: Vec<ActionItem>,
    pub bonus_action: Vec<ActionItem>,
    pub reaction: Vec<ActionItem>,
    pub other: Vec<ActionItem>,
    pub limited_use: Vec<ActionItem>,
}
