use tauri::State;
use crate::db::connection::DbConnection;
use crate::services::sql_service::{SqlService, SqlRecord};

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
