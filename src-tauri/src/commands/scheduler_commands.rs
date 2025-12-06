// Scheduler Commands
// 定时任务调度器相关命令

use crate::db::connection::DbConnection;
use crate::services::scheduler_service::SchedulerConfig;
use tauri::State;

/// 获取调度器配置
#[tauri::command]
pub fn get_scheduler_config(db: State<DbConnection>) -> Result<SchedulerConfig, String> {
    let conn = db.0.lock().map_err(|e| format!("获取数据库连接失败: {}", e))?;

    // 从数据库读取配置
    let config_json: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'scheduler_config'",
            [],
            |row| row.get(0),
        )
        .ok();

    if let Some(json) = config_json {
        serde_json::from_str(&json).map_err(|e| format!("解析调度器配置失败: {}", e))
    } else {
        // 返回默认配置
        Ok(SchedulerConfig::default())
    }
}

/// 保存调度器配置
#[tauri::command]
pub fn save_scheduler_config(
    db: State<DbConnection>,
    config: SchedulerConfig,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| format!("获取数据库连接失败: {}", e))?;

    let config_json = serde_json::to_string(&config)
        .map_err(|e| format!("序列化调度器配置失败: {}", e))?;

    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('scheduler_config', ?1, datetime('now', 'localtime'))",
        rusqlite::params![config_json],
    )
    .map_err(|e| format!("保存调度器配置失败: {}", e))?;

    log::info!("调度器配置已保存");
    Ok(())
}

/// 获取调度器运行状态
#[tauri::command]
pub fn get_scheduler_status() -> Result<bool, String> {
    // 目前调度器是在应用启动时自动运行的，所以始终返回 true
    // 如果需要动态控制，可以将 SchedulerService 作为 Tauri State 管理
    Ok(true)
}
