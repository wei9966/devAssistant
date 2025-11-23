use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlRecord {
    pub id: Option<i64>,
    pub sql_text: String,
    pub sql_type: Option<String>,
    pub database_name: Option<String>,
    pub executed_at: Option<String>,
    pub execution_source: Option<String>,
    pub is_favorite: bool,
    pub tags: Option<String>,
    pub description: Option<String>,
}
