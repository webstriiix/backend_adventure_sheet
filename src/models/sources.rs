use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub id: i32,
    pub slug: String,
    pub full_name: String,
    pub is_hombrew: bool,
    pub publisher: Option<String>,
}
