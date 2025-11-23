use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLog {
    pub id: Option<i64>,
    pub date: String,
    pub log_type: String,
    pub content: String,
    pub ai_generated: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
