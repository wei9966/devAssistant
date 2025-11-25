use tauri::State;
use crate::db::connection::DbConnection;
use crate::services::sql_service::{SqlService, SqlRecord, SqlCategory, SqlCategoryInfo};

#[tauri::command]
pub fn save_sql(db: State<DbConnection>, sql_text: String, source: String) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::save_sql(&conn, &sql_text, &source).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recent_sqls(db: State<DbConnection>, limit: usize) -> Result<Vec<SqlRecord>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::get_recent_sqls(&conn, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_favorite_sqls(db: State<DbConnection>) -> Result<Vec<SqlRecord>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::get_favorite_sqls(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_favorite_sql(db: State<DbConnection>, sql_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::toggle_favorite(&conn, sql_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_sql(db: State<DbConnection>, sql_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::delete_sql(&conn, sql_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_sql_name_category(
    db: State<DbConnection>,
    sql_id: i64,
    name: Option<String>,
    category_id: Option<i64>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::update_sql_name_category(&conn, sql_id, name.as_deref(), category_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_update_sql_name_category(
    db: State<DbConnection>,
    updates: Vec<(i64, Option<String>, Option<i64>)>,
) -> Result<usize, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::batch_update_sql_name_category(&conn, updates).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sql_categories(db: State<DbConnection>) -> Result<Vec<SqlCategory>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::get_all_categories(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_sql_category(
    db: State<DbConnection>,
    name: String,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
    ai_prompt: Option<String>,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::add_category(&conn, &name, description.as_deref(), color.as_deref(), icon.as_deref(), ai_prompt.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_sql_category(
    db: State<DbConnection>,
    category_id: i64,
    name: String,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
    ai_prompt: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::update_category(&conn, category_id, &name, description.as_deref(), color.as_deref(), icon.as_deref(), ai_prompt.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_sql_category(db: State<DbConnection>, category_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::delete_category(&conn, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_uncategorized_sqls(db: State<DbConnection>, limit: usize) -> Result<Vec<SqlRecord>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::get_uncategorized_sqls(&conn, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sqls_by_category(
    db: State<DbConnection>,
    category_id: i64,
    limit: usize,
) -> Result<Vec<SqlRecord>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::get_sqls_by_category(&conn, category_id, limit).map_err(|e| e.to_string())
}

// === 多标签操作命令 ===

#[tauri::command]
pub fn set_sql_categories(
    db: State<DbConnection>,
    sql_id: i64,
    category_ids: Vec<i64>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::set_sql_categories(&conn, sql_id, &category_ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_sql_category_tag(
    db: State<DbConnection>,
    sql_id: i64,
    category_id: i64,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::add_sql_category(&conn, sql_id, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_sql_category_tag(
    db: State<DbConnection>,
    sql_id: i64,
    category_id: i64,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::remove_sql_category(&conn, sql_id, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_sql_name_and_categories(
    db: State<DbConnection>,
    sql_id: i64,
    name: Option<String>,
    category_ids: Vec<i64>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 更新名称
    if let Some(n) = &name {
        SqlService::update_sql_name(&conn, sql_id, Some(n)).map_err(|e| e.to_string())?;
    }

    // 设置分类
    SqlService::set_sql_categories(&conn, sql_id, &category_ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_update_sql_name_categories(
    db: State<DbConnection>,
    updates: Vec<(i64, Option<String>, Vec<i64>)>,
) -> Result<usize, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::batch_update_sql_name_categories(&conn, updates).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_category_sql_counts(db: State<DbConnection>) -> Result<Vec<(i64, i64)>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlService::get_category_sql_counts(&conn).map_err(|e| e.to_string())
}
