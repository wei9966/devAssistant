use crate::db::connection::DbConnection;
use crate::models::clipboard::{ClipboardConfig, ClipboardHistory};
use crate::services::clipboard_history_service::ClipboardHistoryService;
use tauri::State;

/// 获取剪切板历史记录
#[tauri::command]
pub fn get_clipboard_history(
    db: State<DbConnection>,
    limit: Option<i32>,
    offset: Option<i32>,
    content_type: Option<String>,
    keyword: Option<String>,
    pinned_only: Option<bool>,
) -> Result<Vec<ClipboardHistory>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    ClipboardHistoryService::get_history(
        &conn,
        limit.unwrap_or(50),
        offset.unwrap_or(0),
        content_type.as_deref(),
        keyword.as_deref(),
        pinned_only.unwrap_or(false),
    )
}

/// 从历史记录中复制内容
#[tauri::command]
pub fn copy_from_clipboard_history(
    db: State<DbConnection>,
    id: i64,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    ClipboardHistoryService::copy_from_history(&conn, id)
}

/// 删除剪切板历史记录
#[tauri::command]
pub fn delete_clipboard_history_item(
    db: State<DbConnection>,
    id: i64,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    ClipboardHistoryService::delete_record(&conn, id)
}

/// 清空剪切板历史
#[tauri::command]
pub fn clear_clipboard_history(
    db: State<DbConnection>,
    keep_pinned: Option<bool>,
) -> Result<i32, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    ClipboardHistoryService::clear_history(&conn, keep_pinned.unwrap_or(true))
}

/// 切换置顶状态
#[tauri::command]
pub fn toggle_clipboard_pin(
    db: State<DbConnection>,
    id: i64,
) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    ClipboardHistoryService::toggle_pin(&conn, id)
}

/// 搜索剪切板历史
#[tauri::command]
pub fn search_clipboard_history(
    db: State<DbConnection>,
    keyword: String,
    limit: Option<i32>,
) -> Result<Vec<ClipboardHistory>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    ClipboardHistoryService::search_history(&conn, &keyword, limit.unwrap_or(50))
}

/// 获取剪切板配置
#[tauri::command]
pub fn get_clipboard_config(
    db: State<DbConnection>,
) -> Result<ClipboardConfig, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    ClipboardHistoryService::get_config(&conn)
}

/// 更新剪切板配置
#[tauri::command]
pub fn update_clipboard_config(
    db: State<DbConnection>,
    config: ClipboardConfig,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    ClipboardHistoryService::update_config(&conn, config)
}

/// 获取历史记录数量
#[tauri::command]
pub fn get_clipboard_history_count(
    db: State<DbConnection>,
) -> Result<i32, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let count: i32 = conn
        .query_row("SELECT COUNT(*) FROM clipboard_history", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    Ok(count)
}

/// 复制图片路径到剪切板
#[tauri::command]
pub fn copy_image_path(
    db: State<DbConnection>,
    id: i64,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let path: String = conn
        .query_row(
            "SELECT COALESCE(image_path, content) FROM clipboard_history WHERE id = ? AND content_type = 'image'",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|_| "图片记录不存在".to_string())?;

    // 复制路径到剪切板
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(&path).map_err(|e| e.to_string())?;

    Ok(path)
}

/// 复制最近一张图片的路径到剪切板
#[tauri::command]
pub fn copy_latest_image_path(
    db: State<DbConnection>,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let path: String = conn
        .query_row(
            "SELECT COALESCE(image_path, content) FROM clipboard_history WHERE content_type = 'image' ORDER BY created_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "没有图片记录".to_string())?;

    // 复制路径到剪切板
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(&path).map_err(|e| e.to_string())?;

    Ok(path)
}
