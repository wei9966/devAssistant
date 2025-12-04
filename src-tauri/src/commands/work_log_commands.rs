use crate::db::connection::DbConnection;
use crate::models::work_log::WorkLog;
use crate::models::screen_context::{DayContextSummary, WeekContextSummary};
use crate::services::work_log_service::WorkLogService;
use tauri::State;

/// 保存工作日志
/// 如果指定日期的日志已存在,则更新;否则创建新记录
#[tauri::command]
pub fn save_work_log(
    db: State<DbConnection>,
    date: String,
    log_type: String,
    content: String,
    ai_generated: Option<bool>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let ai_gen = ai_generated.unwrap_or(false);

    WorkLogService::save_work_log(&conn, &date, &log_type, &content, ai_gen)
        .map_err(|e| e.to_string())
}

/// 获取指定日期的工作日志
#[tauri::command]
pub fn get_work_log(db: State<DbConnection>, date: String) -> Result<Option<WorkLog>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    WorkLogService::get_work_log(&conn, &date).map_err(|e| e.to_string())
}

/// 获取最近N天的工作日志
#[tauri::command]
pub fn get_recent_work_logs(db: State<DbConnection>, days: i32) -> Result<Vec<WorkLog>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    WorkLogService::get_recent_work_logs(&conn, days).map_err(|e| e.to_string())
}

/// 删除指定日期的工作日志
#[tauri::command]
pub fn delete_work_log(db: State<DbConnection>, date: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    WorkLogService::delete_work_log(&conn, &date).map_err(|e| e.to_string())
}

/// 获取所有工作日志
#[tauri::command]
pub fn get_all_work_logs(db: State<DbConnection>) -> Result<Vec<WorkLog>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    WorkLogService::get_all_work_logs(&conn).map_err(|e| e.to_string())
}

/// 获取当日上下文摘要
#[tauri::command]
pub fn context_get_day_summary(
    db: State<DbConnection>,
    date: String,
) -> Result<DayContextSummary, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    WorkLogService::get_day_context_summary(&conn, &date).map_err(|e| e.to_string())
}

/// 获取周上下文摘要
#[tauri::command]
pub fn context_get_week_summary(
    db: State<DbConnection>,
    start_date: String,
    end_date: String,
) -> Result<WeekContextSummary, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    WorkLogService::get_week_context_summary(&conn, &start_date, &end_date)
        .map_err(|e| e.to_string())
}

/// 基于上下文生成日报
/// 注意：实际的AI生成逻辑需要在AI service中实现，这里先返回占位符
#[tauri::command]
pub fn work_log_generate_with_context(
    db: State<DbConnection>,
    date: String,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 获取上下文摘要
    let summary = WorkLogService::get_day_context_summary(&conn, &date)
        .map_err(|e| e.to_string())?;

    // 生成简单的文本描述（后续可以集成AI服务）
    let mut content = format!("# {} 工作日报\n\n", date);

    if summary.active_time_range.total_minutes > 0 {
        content.push_str(&format!(
            "## 工作时长\n\n- 工作时段: {} - {}\n- 总时长: {} 分钟 ({:.1} 小时)\n\n",
            summary.active_time_range.start,
            summary.active_time_range.end,
            summary.active_time_range.total_minutes,
            summary.active_time_range.total_minutes as f32 / 60.0
        ));

        if !summary.app_usage.is_empty() {
            content.push_str("## 应用使用情况\n\n");
            for app in summary.app_usage.iter().take(5) {
                content.push_str(&format!(
                    "- {}: {} 分钟 ({:.1}%)\n",
                    app.app_name, app.minutes, app.percentage
                ));
            }
            content.push_str("\n");
        }

        content.push_str("## 活动类型分布\n\n");
        content.push_str(&format!("- 编码开发: {} 分钟\n", summary.activity_distribution.coding));
        content.push_str(&format!("- 网页浏览: {} 分钟\n", summary.activity_distribution.browsing));
        content.push_str(&format!("- 文档处理: {} 分钟\n", summary.activity_distribution.document));
        content.push_str(&format!("- 会议沟通: {} 分钟\n", summary.activity_distribution.meeting));
        content.push_str(&format!("- 即时通讯: {} 分钟\n", summary.activity_distribution.communication));
        content.push_str(&format!("- 其他活动: {} 分钟\n\n", summary.activity_distribution.other));

        if !summary.key_activities.is_empty() {
            content.push_str("## 主要工作内容\n\n");
            for activity in summary.key_activities.iter().take(10) {
                content.push_str(&format!(
                    "- [{}] {}: {}\n",
                    activity.time, activity.app_name, activity.description
                ));
            }
        }
    } else {
        content.push_str("今日暂无屏幕上下文数据。\n");
    }

    Ok(content)
}

/// 基于上下文生成周报
#[tauri::command]
pub fn work_log_generate_weekly_with_context(
    db: State<DbConnection>,
    start_date: String,
    end_date: String,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 获取周上下文摘要
    let summary = WorkLogService::get_week_context_summary(&conn, &start_date, &end_date)
        .map_err(|e| e.to_string())?;

    // 生成周报文本
    let mut content = format!("# {} 至 {} 周报\n\n", start_date, end_date);

    content.push_str("## 本周工作时长统计\n\n");
    for day in &summary.daily_active_hours {
        content.push_str(&format!("- {}: {:.1} 小时\n", day.date, day.hours));
    }
    content.push_str("\n");

    if !summary.weekly_app_ranking.is_empty() {
        content.push_str("## 本周应用使用排行\n\n");
        for (idx, app) in summary.weekly_app_ranking.iter().enumerate() {
            content.push_str(&format!(
                "{}. {}: {} 分钟 ({:.1} 小时)\n",
                idx + 1,
                app.app_name,
                app.total_minutes,
                app.total_minutes as f32 / 60.0
            ));
        }
        content.push_str("\n");
    }

    if !summary.weekly_activity_summary.is_empty() {
        content.push_str("## 本周活动类型汇总\n\n");
        for activity in &summary.weekly_activity_summary {
            content.push_str(&format!(
                "- {}: {} 分钟，日均 {:.1} 分钟\n",
                activity.r#type, activity.total_minutes, activity.daily_average
            ));
        }
        content.push_str("\n");
    }

    content.push_str("## 工作节奏分析\n\n");
    content.push_str(&format!(
        "- 最高效时段: {}:00\n",
        summary.work_patterns.most_productive_hour
    ));
    content.push_str(&format!(
        "- 平均开始时间: {}\n",
        summary.work_patterns.average_start_time
    ));
    content.push_str(&format!(
        "- 平均结束时间: {}\n",
        summary.work_patterns.average_end_time
    ));

    Ok(content)
}
