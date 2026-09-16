use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionItem {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_bonus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_uses: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
