use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OptionalFeature {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub feature_type: String,
    pub prerequisite: Option<Value>,
    pub entries: Value,
}
