use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 屏幕上下文数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenContext {
    pub id: Option<i64>,
    pub captured_at: String,
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub activity_type: String,
    pub description: String,
    pub key_content: Option<String>,
    pub screenshot_hash: Option<String>,
    pub screenshot_path: Option<String>,
    pub processing_time_ms: Option<i64>,
}

/// 当日统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayStats {
    pub total_count: u32,
    pub app_distribution: HashMap<String, u32>,
    pub activity_distribution: HashMap<String, u32>,
    pub time_range: Option<(String, String)>,
}

/// 每日摘要数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailySummary {
    pub id: Option<i64>,
    pub summary_date: String,
    pub total_contexts: i32,
    pub app_stats: String,
    pub activity_timeline: String,
    pub ai_summary: Option<String>,
    pub created_at: Option<String>,
}

/// 活动时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTimeRange {
    pub start: String,
    pub end: String,
    pub total_minutes: i32,
}

/// 应用使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUsage {
    pub app_name: String,
    pub minutes: i32,
    pub percentage: f32,
}

/// 活动类型分布
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDistribution {
    pub coding: i32,
    pub browsing: i32,
    pub document: i32,
    pub meeting: i32,
    pub communication: i32,
    pub other: i32,
}

/// 关键活动
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyActivity {
    pub time: String,
    pub description: String,
    pub app_name: String,
}

/// 当日上下文摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayContextSummary {
    pub active_time_range: ActiveTimeRange,
    pub app_usage: Vec<AppUsage>,
    pub activity_distribution: ActivityDistribution,
    pub key_activities: Vec<KeyActivity>,
}

/// 每日活动时长
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyActiveHours {
    pub date: String,
    pub hours: f32,
}

/// 周应用使用排行
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyAppRanking {
    pub app_name: String,
    pub total_minutes: i32,
    pub trend: String, // "up" | "down" | "stable"
}

/// 周活动类型摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyActivitySummary {
    pub r#type: String,
    pub total_minutes: i32,
    pub daily_average: f32,
}

/// 工作模式分析
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkPatterns {
    pub most_productive_hour: i32,
    pub average_start_time: String,
    pub average_end_time: String,
}

/// 周上下文摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekContextSummary {
    pub daily_active_hours: Vec<DailyActiveHours>,
    pub weekly_app_ranking: Vec<WeeklyAppRanking>,
    pub weekly_activity_summary: Vec<WeeklyActivitySummary>,
    pub work_patterns: WorkPatterns,
}
