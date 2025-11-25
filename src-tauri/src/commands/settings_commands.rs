use crate::db::connection::DbConnection;
use crate::services::settings_service::{AppSettings, SettingsService};
use serde_json::Value;
use tauri::State;

/// 获取所有应用设置
#[tauri::command]
pub fn get_app_settings(db: State<DbConnection>) -> Result<AppSettings, String> {
    let conn = db.0.clone();
    let service = SettingsService::new(conn);

    service
        .get_settings()
        .map_err(|e| format!("获取设置失败: {}", e))
}

/// 保存所有应用设置
#[tauri::command]
pub fn save_app_settings(settings: AppSettings, db: State<DbConnection>) -> Result<(), String> {
    let conn = db.0.clone();
    let service = SettingsService::new(conn);

    service
        .save_settings(&settings)
        .map_err(|e| format!("保存设置失败: {}", e))
}

/// 更新单个设置项
#[tauri::command]
pub fn update_app_setting(key: String, value: Value, db: State<DbConnection>) -> Result<(), String> {
    let conn = db.0.clone();
    let service = SettingsService::new(conn);

    service
        .update_setting(&key, value)
        .map_err(|e| format!("更新设置失败: {}", e))
}

/// 重置为默认设置
#[tauri::command]
pub fn reset_app_settings(db: State<DbConnection>) -> Result<AppSettings, String> {
    let conn = db.0.clone();
    let service = SettingsService::new(conn);

    service
        .reset_settings()
        .map_err(|e| format!("重置设置失败: {}", e))?;

    service
        .get_settings()
        .map_err(|e| format!("获取设置失败: {}", e))
}

/// 导出设置为JSON
#[tauri::command]
pub fn export_app_settings(db: State<DbConnection>) -> Result<String, String> {
    let conn = db.0.clone();
    let service = SettingsService::new(conn);

    service
        .export_settings()
        .map_err(|e| format!("导出设置失败: {}", e))
}

/// 从JSON导入设置
#[tauri::command]
pub fn import_app_settings(json: String, db: State<DbConnection>) -> Result<AppSettings, String> {
    let conn = db.0.clone();
    let service = SettingsService::new(conn);

    service
        .import_settings(&json)
        .map_err(|e| format!("导入设置失败: {}", e))?;

    service
        .get_settings()
        .map_err(|e| format!("获取设置失败: {}", e))
}
