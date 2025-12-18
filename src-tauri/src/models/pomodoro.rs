use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// 番茄钟会话状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PomodoroStatus {
    Pending,   // 待开始
    Focusing,  // 专注中
    Paused,    // 已暂停
    Completed, // 已完成
    Cancelled, // 已取消
}

impl Default for PomodoroStatus {
    fn default() -> Self {
        PomodoroStatus::Pending
    }
}

impl fmt::Display for PomodoroStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PomodoroStatus::Pending => write!(f, "pending"),
            PomodoroStatus::Focusing => write!(f, "focusing"),
            PomodoroStatus::Paused => write!(f, "paused"),
            PomodoroStatus::Completed => write!(f, "completed"),
            PomodoroStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl FromStr for PomodoroStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(PomodoroStatus::Pending),
            "focusing" => Ok(PomodoroStatus::Focusing),
            "paused" => Ok(PomodoroStatus::Paused),
            "completed" => Ok(PomodoroStatus::Completed),
            "cancelled" => Ok(PomodoroStatus::Cancelled),
            _ => Err(format!("Invalid PomodoroStatus: {}", s)),
        }
    }
}

impl PomodoroStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PomodoroStatus::Pending => "pending",
            PomodoroStatus::Focusing => "focusing",
            PomodoroStatus::Paused => "paused",
            PomodoroStatus::Completed => "completed",
            PomodoroStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "pending" => PomodoroStatus::Pending,
            "focusing" => PomodoroStatus::Focusing,
            "paused" => PomodoroStatus::Paused,
            "completed" => PomodoroStatus::Completed,
            "cancelled" => PomodoroStatus::Cancelled,
            _ => PomodoroStatus::Pending,
        }
    }
}

/// 番茄钟阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PomodoroPhase {
    Prep,       // 准备阶段
    Focusing,   // 专注中
    Distracted, // 分心提醒
    Report,     // 结束报告
    Stats,      // 统计看板
}

impl Default for PomodoroPhase {
    fn default() -> Self {
        PomodoroPhase::Prep
    }
}

impl fmt::Display for PomodoroPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PomodoroPhase::Prep => write!(f, "prep"),
            PomodoroPhase::Focusing => write!(f, "focusing"),
            PomodoroPhase::Distracted => write!(f, "distracted"),
            PomodoroPhase::Report => write!(f, "report"),
            PomodoroPhase::Stats => write!(f, "stats"),
        }
    }
}

impl FromStr for PomodoroPhase {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "prep" => Ok(PomodoroPhase::Prep),
            "focusing" => Ok(PomodoroPhase::Focusing),
            "distracted" => Ok(PomodoroPhase::Distracted),
            "report" => Ok(PomodoroPhase::Report),
            "stats" => Ok(PomodoroPhase::Stats),
            _ => Err(format!("Invalid PomodoroPhase: {}", s)),
        }
    }
}

impl PomodoroPhase {
    pub fn as_str(&self) -> &str {
        match self {
            PomodoroPhase::Prep => "prep",
            PomodoroPhase::Focusing => "focusing",
            PomodoroPhase::Distracted => "distracted",
            PomodoroPhase::Report => "report",
            PomodoroPhase::Stats => "stats",
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "prep" => PomodoroPhase::Prep,
            "focusing" => PomodoroPhase::Focusing,
            "distracted" => PomodoroPhase::Distracted,
            "report" => PomodoroPhase::Report,
            "stats" => PomodoroPhase::Stats,
            _ => PomodoroPhase::Prep,
        }
    }
}

/// 番茄钟会话
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroSession {
    pub id: Option<i64>,
    pub task_id: Option<i64>,
    pub duration_minutes: i32,
    pub status: PomodoroStatus,
    pub phase: PomodoroPhase,
    pub focus_goal: Option<String>,
    pub ai_suggestion: Option<String>,
    pub actual_focus_seconds: i32,
    pub distraction_count: i32,
    pub focus_rate: f64,
    pub feedback: Option<String>,
    pub progress_update: Option<String>,
    pub started_at: Option<String>,
    pub paused_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
}

/// 专注应用（白名单）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusApp {
    pub id: Option<i64>,
    pub name: String,
    pub process_name: Option<String>,
    pub is_default: bool,
    pub created_at: Option<String>,
}

/// 每日统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroDailyStats {
    pub id: Option<i64>,
    pub date: String,
    pub total_sessions: i32,
    pub completed_sessions: i32,
    pub total_focus_minutes: i32,
    pub avg_focus_rate: f64,
    pub total_distractions: i32,
    pub app_usage: Option<String>,
    pub ai_insight: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 创建番茄钟会话的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePomodoroRequest {
    pub task_id: Option<i64>,
    pub duration_minutes: Option<i32>,
    pub focus_goal: Option<String>,
    pub focus_apps: Option<Vec<String>>,
}

/// 完成番茄钟的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletePomodoroRequest {
    pub feedback: Option<String>,
    pub progress_update: Option<String>,
}

/// AI任务拆解建议
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiFocusSuggestion {
    pub suggested_goal: String,
    pub sub_tasks: Vec<String>,
    pub estimated_pomodoros: i32,
    pub tips: Option<String>,
}

/// AI专注力分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiFocusAnalysis {
    pub focus_rate: f64,
    pub productivity_score: i32,
    pub summary: String,
    pub suggestions: Vec<String>,
    pub next_session_tip: Option<String>,
}
