use crate::db::connection::DbConnection;
use crate::models::work_log::WorkLog;
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
