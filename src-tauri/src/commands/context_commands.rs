// Context Commands
// 屏幕上下文相关的 Tauri 命令

use crate::db::connection::DbConnection;
use crate::models::screen_context::{DayStats, ScreenContext};
use crate::services::context_manager_service::{ContextConfig, ContextManager};
use crate::services::context_store_service::ContextStoreService;
use crate::services::screen_capture_service::CaptureStatus;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// 上下文管理器状态
pub struct ContextManagerState {
    manager: Arc<Mutex<Option<ContextManager>>>,
}

impl ContextManagerState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取或初始化管理器
    async fn get_or_init(&self) -> ContextManager {
        let mut guard = self.manager.lock().await;
        if guard.is_none() {
            *guard = Some(ContextManager::new());
        }
        guard.as_ref().unwrap().clone()
    }
}

impl Clone for ContextManagerState {
    fn clone(&self) -> Self {
        Self {
            manager: Arc::clone(&self.manager),
        }
    }
}

impl Default for ContextManagerState {
    fn default() -> Self {
        Self::new()
    }
}

/// 启动屏幕上下文采集
#[tauri::command]
pub async fn context_start_capture(
    state: State<'_, ContextManagerState>,
) -> Result<(), String> {
    let manager = state.get_or_init().await;

    manager
        .start_capture(|context| {
            // 这里可以添加保存到数据库的逻辑
            println!(
                "截图采集成功: {} - {:?}",
                context.captured_at, context.app_name
            );
        })
        .await
        .map_err(|e| e.to_string())
}

/// 停止屏幕上下文采集
#[tauri::command]
pub async fn context_stop_capture(
    state: State<'_, ContextManagerState>,
) -> Result<(), String> {
    let manager = state.get_or_init().await;
    manager.stop_capture().await.map_err(|e| e.to_string())
}

/// 获取当前采集状态
#[tauri::command]
pub async fn context_get_status(
    state: State<'_, ContextManagerState>,
) -> Result<CaptureStatus, String> {
    let manager = state.get_or_init().await;
    Ok(manager.get_status().await)
}

/// 执行一次手动截图
#[tauri::command]
pub async fn context_capture_once(
    state: State<'_, ContextManagerState>,
) -> Result<ScreenContext, String> {
    let manager = state.get_or_init().await;
    manager
        .capture_once_manual()
        .await
        .map_err(|e| e.to_string())
}

/// 更新上下文采集配置
#[tauri::command]
pub async fn context_update_config(
    config: ContextConfig,
    state: State<'_, ContextManagerState>,
) -> Result<(), String> {
    let manager = state.get_or_init().await;
    manager.update_config(config).await.map_err(|e| e.to_string())
}

/// 获取当前配置
#[tauri::command]
pub async fn context_get_config(
    state: State<'_, ContextManagerState>,
) -> Result<ContextConfig, String> {
    let manager = state.get_or_init().await;
    Ok(manager.get_config().await)
}

/// 重置今日统计
#[tauri::command]
pub async fn context_reset_daily_stats(
    state: State<'_, ContextManagerState>,
) -> Result<(), String> {
    let manager = state.get_or_init().await;
    manager.reset_daily_stats().await;
    Ok(())
}

/// 上下文设置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSettings {
    pub capture_enabled: bool,
    pub capture_interval: u64,  // 秒
    pub similarity_threshold: f32,
    pub retention_days: i32,
    pub excluded_apps: Vec<String>,
    pub save_screenshots: bool,
}

/// 获取上下文设置
#[tauri::command]
pub async fn context_get_settings(
    state: State<'_, ContextManagerState>,
) -> Result<ContextSettings, String> {
    let manager = state.get_or_init().await;
    let config = manager.get_config().await;
    let status = manager.get_status().await;

    Ok(ContextSettings {
        capture_enabled: status.is_running,
        capture_interval: config.capture_interval_secs,
        similarity_threshold: config.similarity_threshold,
        retention_days: 30, // 默认值，后续可以从配置读取
        excluded_apps: vec![], // 默认为空，后续可以从配置读取
        save_screenshots: true, // 默认保存截图
    })
}

/// 更新上下文设置
#[tauri::command]
pub async fn context_update_settings(
    settings: ContextSettings,
    state: State<'_, ContextManagerState>,
) -> Result<(), String> {
    let manager = state.get_or_init().await;

    let config = ContextConfig {
        capture_interval_secs: settings.capture_interval,
        similarity_threshold: settings.similarity_threshold,
    };

    manager.update_config(config).await.map_err(|e| e.to_string())
}

/// 获取统计数据
#[tauri::command]
pub async fn context_get_stats(
    date: String,
    db: State<'_, DbConnection>,
) -> Result<DayStats, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    ContextStoreService::get_day_stats(&conn, &date).map_err(|e| e.to_string())
}

/// 按日期查询上下文列表
#[tauri::command]
pub async fn context_list_by_date(
    date: String,
    db: State<'_, DbConnection>,
) -> Result<Vec<ScreenContext>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    ContextStoreService::list_by_date(&conn, &date).map_err(|e| e.to_string())
}

/// 生成摘要
#[tauri::command]
pub async fn context_generate_summary(
    date: String,
    db: State<'_, DbConnection>,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 获取当日统计
    let stats = ContextStoreService::get_day_stats(&conn, &date)
        .map_err(|e| e.to_string())?;

    if stats.total_count == 0 {
        return Ok(format!("# {} 摘要\n\n今日无活动记录。", date));
    }

    // 构建简单的文本摘要
    let mut summary = format!("# {} 活动摘要\n\n", date);

    summary.push_str(&format!("## 总览\n\n- 总记录数: {}\n", stats.total_count));

    if let Some((start, end)) = &stats.time_range {
        summary.push_str(&format!("- 活动时间: {} 至 {}\n\n", start, end));
    }

    if !stats.app_distribution.is_empty() {
        summary.push_str("## 应用使用情况\n\n");
        let mut apps: Vec<_> = stats.app_distribution.iter().collect();
        apps.sort_by(|a, b| b.1.cmp(a.1));
        for (app, count) in apps.iter().take(5) {
            summary.push_str(&format!("- {}: {} 次\n", app, count));
        }
        summary.push('\n');
    }

    if !stats.activity_distribution.is_empty() {
        summary.push_str("## 活动类型分布\n\n");
        let mut activities: Vec<_> = stats.activity_distribution.iter().collect();
        activities.sort_by(|a, b| b.1.cmp(a.1));
        for (activity, count) in activities {
            summary.push_str(&format!("- {}: {} 次\n", activity, count));
        }
    }

    Ok(summary)
}

/// 删除指定日期的上下文数据
#[tauri::command]
pub async fn context_delete_by_date(
    date: String,
    db: State<'_, DbConnection>,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let deleted = ContextStoreService::delete_by_date(&conn, &date)
        .map_err(|e| e.to_string())?;
    Ok(deleted as i64)
}

/// 清理旧数据
#[tauri::command]
pub async fn context_cleanup_old(
    retention_days: Option<u32>,
    db: State<'_, DbConnection>,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let days = retention_days.unwrap_or(30);
    let deleted = ContextStoreService::cleanup_old(&conn, days)
        .map_err(|e| e.to_string())?;
    Ok(deleted as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_manager_state() {
        let state = ContextManagerState::new();
        let manager = state.get_or_init().await;

        let status = manager.get_status().await;
        assert!(!status.is_running);
    }
}
