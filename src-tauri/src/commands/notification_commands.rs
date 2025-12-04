// Notification Commands
// 通知命令 - 通知中心的所有 Tauri 命令

use crate::db::connection::DbConnection;
use crate::services::context_store_service::ContextStoreService;
use crate::services::notification_service::{Notification, NotificationService, NotificationSettings};
use crate::services::work_log_service::WorkLogService;
use chrono::{Local, Duration as ChronoDuration};
use tauri::State;

/// 获取通知列表
#[tauri::command]
pub fn notification_list(
    db: State<DbConnection>,
    notification_type: Option<String>,
    is_read: Option<bool>,
    limit: Option<i32>,
) -> Result<Vec<Notification>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::list(
        &conn,
        notification_type.as_deref(),
        is_read,
        limit,
    )
    .map_err(|e| e.to_string())
}

/// 获取未读通知数量
#[tauri::command]
pub fn notification_get_unread_count(db: State<DbConnection>) -> Result<i32, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::get_unread_count(&conn).map_err(|e| e.to_string())
}

/// 标记通知已读
#[tauri::command]
pub fn notification_mark_read(db: State<DbConnection>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::mark_read(&conn, id).map_err(|e| e.to_string())
}

/// 标记所有通知已读
#[tauri::command]
pub fn notification_mark_all_read(db: State<DbConnection>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::mark_all_read(&conn).map_err(|e| e.to_string())
}

/// 删除通知
#[tauri::command]
pub fn notification_delete(db: State<DbConnection>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::delete(&conn, id).map_err(|e| e.to_string())
}

/// 清空所有通知
#[tauri::command]
pub fn notification_clear_all(db: State<DbConnection>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::clear_all(&conn).map_err(|e| e.to_string())
}

/// 获取通知设置
#[tauri::command]
pub fn notification_get_settings(db: State<DbConnection>) -> Result<NotificationSettings, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::get_settings(&conn).map_err(|e| e.to_string())
}

/// 更新通知设置
#[tauri::command]
pub fn notification_update_settings(
    db: State<DbConnection>,
    settings: NotificationSettings,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::update_settings(&conn, &settings).map_err(|e| e.to_string())
}

/// 生成日报（手动触发）
#[tauri::command]
pub async fn notification_generate_daily_report(
    db: State<'_, DbConnection>,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 获取昨天的日期
    let yesterday = (Local::now() - ChronoDuration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    // 尝试从 work_logs 获取现有日志
    let log = WorkLogService::get_work_log(&conn, &yesterday)
        .map_err(|e| e.to_string())?;

    let summary = if let Some(log) = log {
        // 如果有日志，使用日志内容
        log.content
    } else {
        // 否则尝试从上下文生成简单摘要
        let contexts = ContextStoreService::list_by_date(&conn, &yesterday)
            .map_err(|e| e.to_string())?;

        if contexts.is_empty() {
            format!("{} 无工作记录", yesterday)
        } else {
            let stats = ContextStoreService::get_day_stats(&conn, &yesterday)
                .map_err(|e| e.to_string())?;

            format!(
                "# {} 工作简报\n\n共记录 {} 次活动\n主要应用：{}",
                yesterday,
                contexts.len(),
                serde_json::to_string_pretty(&stats).unwrap_or_default()
            )
        }
    };

    NotificationService::generate_daily_report_notification(&conn, &yesterday, &summary)
        .map_err(|e| e.to_string())
}

/// 生成周报（手动触发）
#[tauri::command]
pub async fn notification_generate_weekly_report(
    db: State<'_, DbConnection>,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now = Local::now();
    let week_start = now - ChronoDuration::days(7);
    let week_str = format!(
        "{} 至 {}",
        week_start.format("%Y-%m-%d"),
        now.format("%Y-%m-%d")
    );

    // 尝试从最近7天的日志生成周报
    let mut summary = format!("# {} 工作周报\n\n", week_str);

    for i in 0..7 {
        let date = (week_start + ChronoDuration::days(i))
            .format("%Y-%m-%d")
            .to_string();

        if let Ok(Some(log)) = WorkLogService::get_work_log(&conn, &date) {
            summary.push_str(&format!("\n## {}\n{}\n", date, log.content));
        }
    }

    NotificationService::generate_weekly_report_notification(&conn, &week_str, &summary)
        .map_err(|e| e.to_string())
}

/// 生成休息提醒（手动触发）
#[tauri::command]
pub fn notification_generate_tip(
    db: State<DbConnection>,
    work_minutes: Option<i32>,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let minutes = work_minutes.unwrap_or(60);
    NotificationService::generate_tip(&conn, minutes).map_err(|e| e.to_string())
}

/// 检查并生成定时通知（由后台任务调用）
#[tauri::command]
pub async fn notification_check_and_generate(db: State<'_, DbConnection>) -> Result<(), String> {
    let should_daily = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        NotificationService::should_generate_daily_report(&conn).map_err(|e| e.to_string())?
    };

    if should_daily {
        notification_generate_daily_report(db.clone()).await?;
        return Ok(());
    }

    let should_weekly = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        NotificationService::should_generate_weekly_report(&conn).map_err(|e| e.to_string())?
    };

    if should_weekly {
        notification_generate_weekly_report(db.clone()).await?;
        return Ok(());
    }

    Ok(())
}
