// Sedentary Reminder Commands
// 久坐提醒相关的 Tauri 命令

use crate::db::connection::DbConnection;
use crate::services::sedentary_reminder_service::{
    load_config_from_db, save_config_to_db, SedentaryReminderConfig, SedentaryReminderService,
    SedentaryStatus,
};
use std::sync::Mutex;
use tauri::{Emitter, State};

/// 久坐提醒服务状态
pub struct SedentaryState(pub Mutex<Option<SedentaryReminderService>>);

impl SedentaryState {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

impl Default for SedentaryState {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取久坐提醒配置
#[tauri::command]
pub fn get_sedentary_config(db: State<DbConnection>) -> Result<SedentaryReminderConfig, String> {
    let conn = db.0.lock().map_err(|e| format!("获取数据库连接失败: {}", e))?;

    load_config_from_db(&conn).map_err(|e| format!("获取久坐提醒配置失败: {}", e))
}

/// 保存久坐提醒配置
#[tauri::command]
pub fn save_sedentary_config(
    config: SedentaryReminderConfig,
    db: State<DbConnection>,
    sedentary_state: State<SedentaryState>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| format!("获取数据库连接失败: {}", e))?;

    // 保存到数据库
    save_config_to_db(&conn, &config).map_err(|e| format!("保存久坐提醒配置失败: {}", e))?;

    // 更新运行中的服务配置
    if let Ok(mut service_guard) = sedentary_state.0.lock() {
        if let Some(ref service) = *service_guard {
            service.set_config(config);
        }
    }

    Ok(())
}

/// 启动久坐提醒
#[tauri::command]
pub fn start_sedentary_reminder(
    app: tauri::AppHandle,
    db: State<DbConnection>,
    sedentary_state: State<SedentaryState>,
) -> Result<(), String> {
    let config = {
        let conn = db.0.lock().map_err(|e| format!("获取数据库连接失败: {}", e))?;
        load_config_from_db(&conn).map_err(|e| format!("加载配置失败: {}", e))?
    };

    let mut service_guard = sedentary_state
        .0
        .lock()
        .map_err(|e| format!("获取服务状态失败: {}", e))?;

    // 如果服务已存在且正在运行，先停止
    if let Some(ref service) = *service_guard {
        if service.is_running() {
            service.stop();
        }
    }

    // 创建新服务
    let service = SedentaryReminderService::with_config(config);

    // 克隆 app handle 用于发送事件
    let app_handle = app.clone();

    // 启动服务，设置回调发送事件到前端
    service
        .start(move |tip| {
            // 发送久坐提醒事件到前端
            if let Err(e) = app_handle.emit("sedentary-reminder", SedentaryReminderEvent {
                tip,
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            }) {
                log::error!("[久坐提醒] 发送事件失败: {}", e);
            }
        })
        .map_err(|e| format!("启动久坐提醒服务失败: {}", e))?;

    *service_guard = Some(service);

    log::info!("[久坐提醒] 服务已启动");
    Ok(())
}

/// 停止久坐提醒
#[tauri::command]
pub fn stop_sedentary_reminder(sedentary_state: State<SedentaryState>) -> Result<(), String> {
    let service_guard = sedentary_state
        .0
        .lock()
        .map_err(|e| format!("获取服务状态失败: {}", e))?;

    if let Some(ref service) = *service_guard {
        service.stop();
        log::info!("[久坐提醒] 服务已停止");
    }

    Ok(())
}

/// 获取久坐提醒状态
#[tauri::command]
pub fn get_sedentary_status(sedentary_state: State<SedentaryState>) -> Result<SedentaryStatus, String> {
    let service_guard = sedentary_state
        .0
        .lock()
        .map_err(|e| format!("获取服务状态失败: {}", e))?;

    if let Some(ref service) = *service_guard {
        Ok(service.get_status())
    } else {
        // 服务未初始化，返回默认状态
        Ok(SedentaryStatus {
            is_running: false,
            continuous_work_seconds: 0,
            last_reminder_time: None,
            today_reminder_count: 0,
        })
    }
}

/// 重置久坐计时器（用户关闭提醒后调用）
#[tauri::command]
pub fn reset_sedentary_timer(sedentary_state: State<SedentaryState>) -> Result<(), String> {
    let service_guard = sedentary_state
        .0
        .lock()
        .map_err(|e| format!("获取服务状态失败: {}", e))?;

    if let Some(ref service) = *service_guard {
        service.reset_timer();
        log::info!("[久坐提醒] 计时器已重置");
    }

    Ok(())
}

/// 久坐提醒事件结构
#[derive(Clone, serde::Serialize)]
pub struct SedentaryReminderEvent {
    pub tip: String,
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sedentary_state_creation() {
        let state = SedentaryState::new();
        let guard = state.0.lock().unwrap();
        assert!(guard.is_none());
    }
}
