use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCategoryDefinition {
    pub id: Option<i64>,
    pub key: String,
    pub name: String,
    pub color: String,
    pub icon: String,
    pub is_system: bool,
    pub is_hidden: bool,
    pub sort_order: i32,
    pub usage_count: i64,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
