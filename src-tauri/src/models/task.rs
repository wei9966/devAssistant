use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    pub due_date: Option<String>,             // 截止日期
    pub registered_at: Option<String>,        // 登记日期
    pub display_date: Option<String>,         // 日历显示日期
    pub scheduled_start_time: Option<String>, // 计划开始时间（精确到秒，用于提醒）
    pub estimated_hours: Option<f32>,
    pub actual_hours: Option<f32>,
    pub context: Option<WorkContext>,
    pub notes: Option<String>,
    pub quadrant: Option<TaskQuadrant>,
    pub tags: Option<Vec<Tag>>,
    pub progress: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Todo,      // 待办
    Active,    // 进行中
    Done,      // 已完成
    Deferred,  // 延后
    Cancelled, // 已取消
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::Active => "active",
            TaskStatus::Done => "done",
            TaskStatus::Deferred => "deferred",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "todo" => TaskStatus::Todo,
            "active" => TaskStatus::Active,
            "done" => TaskStatus::Done,
            "deferred" => TaskStatus::Deferred,
            "cancelled" => TaskStatus::Cancelled,
            _ => TaskStatus::Todo,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskCategory {
    Backend,  // 后端开发
    Database, // 数据库
    Feature,  // 功能开发
    Docs,     // 文档
    Other,    // 其他
    Custom(String),
}

impl TaskCategory {
    pub fn as_str(&self) -> &str {
        match self {
            TaskCategory::Backend => "backend",
            TaskCategory::Database => "database",
            TaskCategory::Feature => "feature",
            TaskCategory::Docs => "docs",
            TaskCategory::Other => "other",
            TaskCategory::Custom(value) => value.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "backend" => TaskCategory::Backend,
            "database" => TaskCategory::Database,
            "feature" => TaskCategory::Feature,
            "docs" => TaskCategory::Docs,
            "other" | "" => TaskCategory::Other,
            value => TaskCategory::Custom(value.to_string()),
        }
    }
}

impl Serialize for TaskCategory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TaskCategory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(TaskCategory::from_str(&value))
    }
}

#[derive(Debug, Clone, PartialEq)]
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

// 自定义序列化：将 TaskPriority 序列化为数字
impl Serialize for TaskPriority {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(self.as_i32())
    }
}

// 自定义反序列化：从数字反序列化为 TaskPriority
impl<'de> Deserialize<'de> for TaskPriority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let n = i32::deserialize(deserializer)?;
        Ok(TaskPriority::from_i32(n))
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

/// 导入任务的数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportTask {
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub priority: Option<i32>,
    pub status: Option<String>,
    pub git_branch: Option<String>,
    pub created_at: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub estimated_hours: Option<f32>,
    pub actual_hours: Option<f32>,
    pub notes: Option<String>,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub success: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// 四象限枚举（基于紧急和重要程度）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskQuadrant {
    UrgentImportant,       // 紧急且重要
    UrgentNotImportant,    // 紧急不重要
    NotUrgentImportant,    // 不紧急但重要
    NotUrgentNotImportant, // 不紧急不重要
}

impl TaskQuadrant {
    pub fn as_str(&self) -> &str {
        match self {
            TaskQuadrant::UrgentImportant => "urgent_important",
            TaskQuadrant::UrgentNotImportant => "urgent_not_important",
            TaskQuadrant::NotUrgentImportant => "not_urgent_important",
            TaskQuadrant::NotUrgentNotImportant => "not_urgent_not_important",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "urgent_important" => TaskQuadrant::UrgentImportant,
            "urgent_not_important" => TaskQuadrant::UrgentNotImportant,
            "not_urgent_important" => TaskQuadrant::NotUrgentImportant,
            "not_urgent_not_important" => TaskQuadrant::NotUrgentNotImportant,
            _ => TaskQuadrant::UrgentNotImportant, // 默认值
        }
    }
}

/// 标签结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub color: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub is_favorite: Option<bool>,
}

/// 四象限统计结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuadrantStatistics {
    pub quadrant: String,
    pub count: i64,
}

/// 任务里程碑结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskMilestone {
    pub id: Option<i64>,
    pub task_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub progress_snapshot: Option<i32>,
    pub created_at: Option<String>,
}
