use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyPlan {
    pub id: Option<i64>,
    pub week_key: String,              // 周标识，如 "2025-W48"
    pub content: String,               // 润色后的计划内容
    pub task_ids: Vec<i64>,            // 当前关联的任务ID列表
    pub original_task_ids: Vec<i64>,   // 初次创建时的任务ID（用于对比新增项）
    pub status: WeeklyPlanStatus,      // 状态
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WeeklyPlanStatus {
    Draft,
    Confirmed,
}

impl Default for WeeklyPlanStatus {
    fn default() -> Self {
        WeeklyPlanStatus::Draft
    }
}

impl From<String> for WeeklyPlanStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "confirmed" => WeeklyPlanStatus::Confirmed,
            _ => WeeklyPlanStatus::Draft,
        }
    }
}

impl ToString for WeeklyPlanStatus {
    fn to_string(&self) -> String {
        match self {
            WeeklyPlanStatus::Draft => "draft".to_string(),
            WeeklyPlanStatus::Confirmed => "confirmed".to_string(),
        }
    }
}
