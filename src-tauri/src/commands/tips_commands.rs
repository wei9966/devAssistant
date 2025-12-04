// Tips Commands
// 智能提示命令 - 处理所有Tips相关的Tauri命令

use crate::db::connection::DbConnection;
use crate::services::tips_service::{ActivityPattern, Tip, TipsService};
use tauri::State;

/// 生成智能提示
#[tauri::command]
pub fn tips_generate(
    db: State<DbConnection>,
    hours: Option<i64>,
) -> Result<i64, String> {
    let hours = hours.unwrap_or(2); // 默认分析最近2小时

    // 分析活动模式
    let pattern = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        TipsService::analyze_recent_pattern(&conn, hours).map_err(|e| e.to_string())?
    };

    // 生成提示内容
    let (content, category, priority) = TipsService::generate_default_tip(&pattern);

    // 保存到数据库
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TipsService::save_tip(&conn, &content, &category, &priority).map_err(|e| e.to_string())
}

/// 获取提示列表
#[tauri::command]
pub fn tips_list(
    db: State<DbConnection>,
    is_read: Option<bool>,
    limit: Option<i32>,
) -> Result<Vec<Tip>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TipsService::list_tips(&conn, is_read, limit).map_err(|e| e.to_string())
}

/// 标记提示已读
#[tauri::command]
pub fn tips_mark_read(db: State<DbConnection>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TipsService::mark_read(&conn, id).map_err(|e| e.to_string())
}

/// 获取未读提示数量
#[tauri::command]
pub fn tips_get_unread_count(db: State<DbConnection>) -> Result<i32, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TipsService::get_unread_count(&conn).map_err(|e| e.to_string())
}

/// 分析活动模式（不生成提示）
#[tauri::command]
pub fn tips_analyze_pattern(
    db: State<DbConnection>,
    hours: Option<i64>,
) -> Result<ActivityPattern, String> {
    let hours = hours.unwrap_or(2);
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TipsService::analyze_recent_pattern(&conn, hours).map_err(|e| e.to_string())
}

/// 检查是否应该生成新提示
#[tauri::command]
pub fn tips_should_generate(
    db: State<DbConnection>,
    interval_minutes: Option<i64>,
) -> Result<bool, String> {
    let interval = interval_minutes.unwrap_or(60); // 默认60分钟间隔
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TipsService::should_generate_tip(&conn, interval).map_err(|e| e.to_string())
}

/// 清理旧提示
#[tauri::command]
pub fn tips_cleanup_old(db: State<DbConnection>, days: Option<i64>) -> Result<i64, String> {
    let days = days.unwrap_or(30); // 默认清理30天前的已读提示
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TipsService::cleanup_old_tips(&conn, days).map_err(|e| e.to_string())
}
