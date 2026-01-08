// Context Commands
// 屏幕上下文相关的 Tauri 命令

use crate::db::connection::DbConnection;
use crate::models::screen_context::{DayStats, ScreenContext};
use crate::services::context_manager_service::{ContextConfig, ContextManager};
use crate::services::context_store_service::ContextStoreService;
use crate::services::screen_capture_service::CaptureStatus;
use crate::services::vlm_service::VlmService;
use crate::services::screenshot_batch_processor_service::{ScreenshotBatchProcessor, BatchItem};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
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
    pub async fn get_or_init(&self) -> ContextManager {
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

/// 批量处理器状态
pub struct BatchProcessorState {
    processor: Arc<Mutex<Option<ScreenshotBatchProcessor>>>,
}

impl BatchProcessorState {
    pub fn new() -> Self {
        Self {
            processor: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取或初始化处理器
    pub async fn get_or_init(&self, db_path: String) -> ScreenshotBatchProcessor {
        let mut guard = self.processor.lock().await;
        if guard.is_none() {
            *guard = Some(ScreenshotBatchProcessor::new(db_path));
        }
        guard.as_ref().unwrap().clone()
    }
}

impl Clone for BatchProcessorState {
    fn clone(&self) -> Self {
        Self {
            processor: Arc::clone(&self.processor),
        }
    }
}

impl Default for BatchProcessorState {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取数据库路径
fn get_db_path() -> String {
    let db_path = dirs::data_local_dir()
        .expect("无法获取应用数据目录")
        .join("dev-assistant")
        .join("dev_assistant.db");
    db_path.to_string_lossy().to_string()
}

/// 异步分析截图并更新数据库（保留供手动分析使用）
#[allow(dead_code)]
async fn analyze_screenshot_with_vlm(
    context_id: i64,
    screenshot_path: String,
    db_path: String,
) {
    // 检查VLM是否启用
    let vlm_service = VlmService::new(db_path.clone());
    let vlm_enabled = vlm_service.is_enabled().unwrap_or(false);

    if !vlm_enabled {
        return;
    }

    // 读取截图文件并转换为base64
    let image_data = match std::fs::read(&screenshot_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("读取截图文件失败: {} - {}", screenshot_path, e);
            return;
        }
    };

    let image_base64 = BASE64.encode(&image_data);

    // 构建分析提示词
    let prompt = r#"请分析这张屏幕截图，描述用户正在进行的活动。请用中文简洁地描述：
1. 用户正在使用什么应用或网站
2. 用户正在做什么具体任务
3. 屏幕上显示的关键内容（如代码片段、文档标题、网页内容等）

请以简洁的一段话描述，不超过100字。"#;

    // 调用VLM分析
    match vlm_service.analyze_image(&image_base64, prompt).await {
        Ok(description) => {
            // 更新数据库
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                if let Err(e) = ContextStoreService::update_description(&conn, context_id, &description, None, None) {
                    eprintln!("更新VLM分析结果失败: {}", e);
                } else {
                    println!("VLM分析完成, ID: {}, 描述: {}", context_id, description);
                }
            }
        }
        Err(e) => {
            eprintln!("VLM分析失败: {}", e);
        }
    }
}

/// 启动屏幕上下文采集
#[tauri::command]
pub async fn context_start_capture(
    state: State<'_, ContextManagerState>,
    batch_state: State<'_, BatchProcessorState>,
    db: State<'_, DbConnection>,
) -> Result<(), String> {
    // 从数据库加载配置（在单独作用域内释放锁）
    let config = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let settings = load_context_settings_from_db(&conn);

        ContextConfig {
            capture_interval_secs: settings.capture_interval,
            similarity_threshold: settings.similarity_threshold,
            save_screenshots: settings.save_screenshots,
            screenshot_dir: settings.screenshot_dir,
            idle_timeout_secs: settings.idle_timeout_secs,  // 新增
        }
    };

    let manager = state.get_or_init().await;

    // 尝试更新配置（如果采集未运行）
    let _ = manager.update_config(config).await;

    // 克隆数据库连接的 Arc，以便在回调中使用
    let db_arc = db.0.clone();
    let db_path = get_db_path();

    // 自动启动批处理器（如果尚未运行）
    let batch_processor = batch_state.get_or_init(db_path.clone()).await;
    let processor_state = batch_processor.get_state().await;
    if !processor_state.is_running {
        // 启动处理器
        if let Err(e) = batch_processor.start().await {
            log::warn!("启动批处理器失败: {}", e);
        } else {
            // 在后台运行处理循环
            let processor_clone = batch_processor.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = processor_clone.run_processing_loop().await {
                    log::error!("批量处理循环出错: {}", e);
                }
            });
            log::info!("批量截图处理器已自动启动");
        }
    }

    // 克隆批量处理器状态
    let batch_state_clone = batch_state.inner().clone();
    let db_path_for_batch = db_path.clone();

    manager
        .start_capture(move |context| {
            // 保存到数据库
            let saved_id = match db_arc.lock() {
                Ok(conn) => {
                    match ContextStoreService::save_context(&conn, &context) {
                        Ok(id) => {
                            println!(
                                "截图采集成功并已保存: {} - {:?}, ID: {}",
                                context.captured_at, context.app_name, id
                            );
                            Some(id)
                        }
                        Err(e) => {
                            eprintln!("保存截图上下文失败: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("获取数据库连接失败: {}", e);
                    None
                }
            };

            // 保存成功后，将截图加入批量处理队列
            if let Some(id) = saved_id {
                if let Some(ref path) = context.screenshot_path {
                    let batch_state_inner = batch_state_clone.clone();
                    let db_path_inner = db_path_for_batch.clone();
                    let screenshot_path = path.clone();

                    // 使用 tokio::spawn 异步发送到批量处理队列
                    tokio::spawn(async move {
                        let processor = batch_state_inner.get_or_init(db_path_inner).await;
                        let batch_item = BatchItem {
                            context_id: id,
                            screenshot_path,
                        };

                        if let Err(e) = processor.enqueue(batch_item).await {
                            eprintln!("加入批量处理队列失败: {}", e);
                        } else {
                            println!("截图已加入批量处理队列, ID: {}", id);
                        }
                    });
                }
            }
        })
        .await
        .map_err(|e| e.to_string())
}

/// 停止屏幕上下文采集
#[tauri::command]
pub async fn context_stop_capture(
    state: State<'_, ContextManagerState>,
    batch_state: State<'_, BatchProcessorState>,
) -> Result<(), String> {
    let manager = state.get_or_init().await;
    manager.stop_capture().await.map_err(|e| e.to_string())?;

    // 同时停止批处理器
    let db_path = get_db_path();
    let batch_processor = batch_state.get_or_init(db_path).await;
    let processor_state = batch_processor.get_state().await;
    if processor_state.is_running {
        if let Err(e) = batch_processor.stop().await {
            log::warn!("停止批处理器失败: {}", e);
        } else {
            log::info!("批量截图处理器已停止");
        }
    }

    Ok(())
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

/// 执行手动截图并保存到数据库（供快捷键调用）
#[tauri::command]
pub async fn context_manual_capture_and_save(
    state: State<'_, ContextManagerState>,
    batch_state: State<'_, BatchProcessorState>,
    db: State<'_, DbConnection>,
) -> Result<i64, String> {
    // 执行截图
    let manager = state.get_or_init().await;
    let context = manager
        .capture_once_manual()
        .await
        .map_err(|e| e.to_string())?;

    // 保存到数据库
    let saved_id = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        ContextStoreService::save_context(&conn, &context)
            .map_err(|e| format!("保存截图失败: {}", e))?
    };

    println!("✓ 手动截图已保存: {} - {:?}, ID: {}", context.captured_at, context.app_name, saved_id);

    // 加入批量处理队列（用于VLM分析）
    if let Some(ref path) = context.screenshot_path {
        let db_path = get_db_path();
        let processor = batch_state.get_or_init(db_path).await;
        let batch_item = BatchItem {
            context_id: saved_id,
            screenshot_path: path.clone(),
        };

        if let Err(e) = processor.enqueue(batch_item).await {
            eprintln!("加入批量处理队列失败: {}", e);
        } else {
            println!("✓ 手动截图已加入VLM分析队列, ID: {}", saved_id);
        }
    }

    Ok(saved_id)
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
    pub screenshot_dir: Option<String>,  // 截图保存目录
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,  // 空闲超时时间（秒），默认300秒（5分钟）
}

/// 空闲超时默认值：300秒（5分钟）
fn default_idle_timeout() -> u64 {
    300
}

/// 从数据库加载上下文设置（内部使用）
fn load_context_settings_from_db(conn: &rusqlite::Connection) -> ContextSettings {
    let settings_json: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'context_settings'",
            [],
            |row| row.get(0),
        )
        .ok();

    if let Some(json) = settings_json {
        serde_json::from_str(&json).unwrap_or_else(|_| ContextSettings::default())
    } else {
        ContextSettings::default()
    }
}

/// 从数据库加载上下文设置（公开接口，供 main.rs 自动恢复采集使用）
pub fn load_context_settings_internal(conn: &rusqlite::Connection) -> ContextSettings {
    load_context_settings_from_db(conn)
}

/// 保存上下文设置到数据库
fn save_context_settings_to_db(conn: &rusqlite::Connection, settings: &ContextSettings) -> Result<(), String> {
    let settings_json = serde_json::to_string(settings)
        .map_err(|e| format!("序列化设置失败: {}", e))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
        rusqlite::params!["context_settings", settings_json, timestamp],
    ).map_err(|e| format!("保存设置失败: {}", e))?;

    Ok(())
}

impl Default for ContextSettings {
    fn default() -> Self {
        Self {
            capture_enabled: false,
            capture_interval: 10,
            similarity_threshold: 0.95,
            retention_days: 30,
            excluded_apps: vec![],
            save_screenshots: true,
            screenshot_dir: None,
            idle_timeout_secs: 300,  // 默认5分钟
        }
    }
}

/// 获取上下文设置
#[tauri::command]
pub async fn context_get_settings(
    state: State<'_, ContextManagerState>,
    db: State<'_, DbConnection>,
) -> Result<ContextSettings, String> {
    // 从数据库加载设置（在单独的作用域内释放锁）
    let mut settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        load_context_settings_from_db(&conn)
    };

    // 更新运行状态（运行状态不持久化，从内存获取）
    let manager = state.get_or_init().await;
    let status = manager.get_status().await;
    settings.capture_enabled = status.is_running;

    Ok(settings)
}

/// 更新上下文设置
#[tauri::command]
pub async fn context_update_settings(
    settings: ContextSettings,
    state: State<'_, ContextManagerState>,
    db: State<'_, DbConnection>,
) -> Result<(), String> {
    // 保存到数据库
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        save_context_settings_to_db(&conn, &settings)?;
    }

    // 更新内存中的配置
    let manager = state.get_or_init().await;
    let config = ContextConfig {
        capture_interval_secs: settings.capture_interval,
        similarity_threshold: settings.similarity_threshold,
        save_screenshots: settings.save_screenshots,
        screenshot_dir: settings.screenshot_dir,
        idle_timeout_secs: settings.idle_timeout_secs,  // 新增
    };

    manager.update_config(config).await.map_err(|e| e.to_string())
}

/// 获取默认截图目录
#[tauri::command]
pub fn context_get_default_screenshot_dir() -> String {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("dev-assistant")
        .join("screenshots")
        .to_string_lossy()
        .to_string()
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

/// 启动批量截图处理器
#[tauri::command]
pub async fn context_start_batch_processor(
    batch_state: State<'_, BatchProcessorState>,
) -> Result<(), String> {
    let db_path = get_db_path();
    let processor = batch_state.get_or_init(db_path).await;

    // 启动处理器
    processor.start().await.map_err(|e| e.to_string())?;

    // 在后台运行处理循环
    let processor_clone = processor.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = processor_clone.run_processing_loop().await {
            log::error!("批量处理循环出错: {}", e);
        }
    });

    Ok(())
}

/// 停止批量截图处理器
#[tauri::command]
pub async fn context_stop_batch_processor(
    batch_state: State<'_, BatchProcessorState>,
) -> Result<(), String> {
    let db_path = get_db_path();
    let processor = batch_state.get_or_init(db_path).await;
    processor.stop().await.map_err(|e| e.to_string())
}

/// 获取批量处理器状态
#[tauri::command]
pub async fn context_get_batch_processor_status(
    batch_state: State<'_, BatchProcessorState>,
) -> Result<serde_json::Value, String> {
    let db_path = get_db_path();
    let processor = batch_state.get_or_init(db_path).await;
    let state = processor.get_state().await;

    Ok(serde_json::json!({
        "is_running": state.is_running,
        "batch_size": state.batch_size,
        "batch_timeout_secs": state.batch_timeout_secs
    }))
}

/// 手动触发批量处理指定的截图
#[tauri::command]
pub async fn context_process_screenshots_manually(
    screenshot_ids: Vec<i64>,
    batch_state: State<'_, BatchProcessorState>,
) -> Result<serde_json::Value, String> {
    let db_path = get_db_path();
    let processor = batch_state.get_or_init(db_path).await;

    let result = processor.process_screenshots_manually(screenshot_ids)
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "total_processed": result.total_processed,
        "successful": result.successful,
        "failed": result.failed,
        "processing_time_ms": result.processing_time_ms
    }))
}

/// 获取未处理的截图数量
#[tauri::command]
pub async fn context_count_unprocessed_screenshots(
    batch_state: State<'_, BatchProcessorState>,
) -> Result<usize, String> {
    let db_path = get_db_path();
    let processor = batch_state.get_or_init(db_path).await;
    processor.count_unprocessed_screenshots()
        .await
        .map_err(|e| e.to_string())
}

/// 获取未处理的截图ID列表
#[tauri::command]
pub async fn context_get_unprocessed_screenshot_ids(
    limit: Option<usize>,
    batch_state: State<'_, BatchProcessorState>,
) -> Result<Vec<i64>, String> {
    let db_path = get_db_path();
    let processor = batch_state.get_or_init(db_path).await;
    processor.get_unprocessed_screenshot_ids(limit)
        .await
        .map_err(|e| e.to_string())
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
