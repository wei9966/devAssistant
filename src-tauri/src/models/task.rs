use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub category: TaskCategory,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub git_branch: Option<String>,
    pub created_at: Option<String>,
    pub started_at: Option<String>,
    pub last_active_at: Option<String>,
    pub completed_at: Option<String>,
    pub estimated_hours: Option<f32>,
    pub actual_hours: Option<f32>,
    pub context: Option<WorkContext>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Todo,      // 待办
    Active,    // 进行中
    Done,      // 已完成
    Deferred,  // 延后
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::Active => "active",
            TaskStatus::Done => "done",
            TaskStatus::Deferred => "deferred",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "todo" => TaskStatus::Todo,
            "active" => TaskStatus::Active,
            "done" => TaskStatus::Done,
            "deferred" => TaskStatus::Deferred,
            _ => TaskStatus::Todo,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskCategory {
    Dev,      // 开发
    Ops,      // 运维
    Study,    // 学习
    Other,    // 其他
}

impl TaskCategory {
    pub fn as_str(&self) -> &str {
        match self {
            TaskCategory::Dev => "dev",
            TaskCategory::Ops => "ops",
            TaskCategory::Study => "study",
            TaskCategory::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "dev" => TaskCategory::Dev,
            "ops" => TaskCategory::Ops,
            "study" => TaskCategory::Study,
            _ => TaskCategory::Other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    High = 1,
    Medium = 2,
    Low = 3,
}

impl TaskPriority {
    pub fn from_i32(n: i32) -> Self {
        match n {
            1 => TaskPriority::High,
            3 => TaskPriority::Low,
            _ => TaskPriority::Medium,
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            TaskPriority::High => 1,
            TaskPriority::Medium => 2,
            TaskPriority::Low => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkContext {
    pub files: Vec<FileContext>,
    pub last_sql: Option<String>,
    pub browser_tabs: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContext {
    pub path: String,
    pub line: usize,
    pub column: usize,
}
