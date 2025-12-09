use crate::models::AppItem;
use crate::services::AppScannerService;
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};

/// 新应用检测事件数据
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAppsDetectedEvent {
    pub count: usize,
    pub apps: Vec<AppItem>,
}

/// 应用监控服务
/// 自动检测新安装的应用并通知前端
pub struct AppMonitorService {
    /// 监控运行状态
    is_running: Arc<AtomicBool>,
    /// 检查间隔（秒）
    check_interval_secs: u64,
}

impl AppMonitorService {
    /// 创建新的应用监控服务实例
    /// 默认检查间隔为 300 秒（5分钟）
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            check_interval_secs: 300,
        }
    }

    /// 创建带自定义检查间隔的服务实例
    pub fn with_interval(check_interval_secs: u64) -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            check_interval_secs,
        }
    }

    /// 启动后台监控
    ///
    /// # 参数
    /// - `db`: 数据库连接
    /// - `app_handle`: Tauri 应用句柄，用于发送事件
    pub fn start(&self, db: Arc<Mutex<Connection>>, app_handle: AppHandle) {
        // 检查是否已经在运行
        if self.is_running.load(Ordering::SeqCst) {
            log::warn!("App monitor service is already running");
            return;
        }

        // 设置运行标志
        self.is_running.store(true, Ordering::SeqCst);

        let is_running = Arc::clone(&self.is_running);
        let check_interval = self.check_interval_secs;

        // 在后台任务中运行监控循环
        tokio::spawn(async move {
            log::info!(
                "App monitor service started with interval: {} seconds",
                check_interval
            );

            while is_running.load(Ordering::SeqCst) {
                // 执行检查
                match Self::check_for_new_apps_internal(Arc::clone(&db)).await {
                    Ok(new_apps) => {
                        if !new_apps.is_empty() {
                            log::info!("Detected {} new apps", new_apps.len());

                            // 发送事件到前端
                            let event = NewAppsDetectedEvent {
                                count: new_apps.len(),
                                apps: new_apps,
                            };

                            if let Err(e) = app_handle.emit("new-apps-detected", event) {
                                log::error!("Failed to emit new-apps-detected event: {}", e);
                            }
                        } else {
                            log::debug!("No new apps detected");
                        }
                    }
                    Err(e) => {
                        log::error!("Error checking for new apps: {}", e);
                    }
                }

                // 等待下一次检查
                sleep(Duration::from_secs(check_interval)).await;
            }

            log::info!("App monitor service stopped");
        });
    }

    /// 停止监控
    pub fn stop(&self) {
        log::info!("Stopping app monitor service...");
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// 检查监控是否正在运行
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// 检查新应用（公共方法）
    ///
    /// 扫描系统并对比数据库，返回新增的应用列表
    pub async fn check_for_new_apps(
        &self,
        db: Arc<Mutex<Connection>>,
    ) -> Result<Vec<AppItem>, String> {
        Self::check_for_new_apps_internal(db).await
    }

    /// 检查新应用的内部实现
    ///
    /// 1. 只扫描开始菜单（最常见的新应用安装位置）
    /// 2. 从数据库获取已存在的应用列表
    /// 3. 对比找出新增的应用
    async fn check_for_new_apps_internal(
        db: Arc<Mutex<Connection>>,
    ) -> Result<Vec<AppItem>, String> {
        // 1. 快速扫描开始菜单（只扫描开始菜单，速度快）
        let scanned_apps = Self::scan_start_menu_only().await?;

        // 2. 获取数据库中已存在的应用ID列表
        let existing_app_ids = Self::get_existing_app_ids(Arc::clone(&db))?;

        // 3. 过滤出新应用
        let new_apps: Vec<AppItem> = scanned_apps
            .into_iter()
            .filter(|app| !existing_app_ids.contains(&app.id))
            .collect();

        Ok(new_apps)
    }

    /// 只扫描开始菜单
    /// 这是最快的扫描方式，因为大多数新安装的应用都会在开始菜单创建快捷方式
    async fn scan_start_menu_only() -> Result<Vec<AppItem>, String> {
        #[cfg(target_os = "windows")]
        {
            use std::path::PathBuf;

            let mut apps = Vec::new();

            // 获取开始菜单路径
            let start_menu_paths = vec![
                Self::get_common_start_menu_path(),
                Self::get_user_start_menu_path(),
            ];

            for start_menu_path in start_menu_paths {
                if let Some(path) = start_menu_path {
                    if path.exists() {
                        // 使用 AppScannerService 扫描指定目录
                        let found_apps = AppScannerService::scan_installed_apps(Some(
                            path.to_string_lossy().to_string(),
                        ))
                        .await?;
                        apps.extend(found_apps);
                    }
                }
            }

            Ok(apps)
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(Vec::new())
        }
    }

    /// 获取公共开始菜单路径
    #[cfg(target_os = "windows")]
    fn get_common_start_menu_path() -> Option<std::path::PathBuf> {
        dirs::data_dir().map(|p| {
            p.parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("ProgramData")
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
        })
    }

    /// 获取用户开始菜单路径
    #[cfg(target_os = "windows")]
    fn get_user_start_menu_path() -> Option<std::path::PathBuf> {
        dirs::data_dir().map(|p| {
            p.join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
        })
    }

    /// 从数据库获取已存在的应用ID集合
    fn get_existing_app_ids(db: Arc<Mutex<Connection>>) -> Result<HashSet<String>, String> {
        let db = db.lock().map_err(|e| e.to_string())?;

        let mut stmt = db
            .prepare("SELECT id FROM apps")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let app_ids = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to query apps: {}", e))?
            .collect::<Result<HashSet<_>, _>>()
            .map_err(|e| format!("Failed to collect app IDs: {}", e))?;

        Ok(app_ids)
    }
}

impl Default for AppMonitorService {
    fn default() -> Self {
        Self::new()
    }
}

// 确保服务在析构时停止监控
impl Drop for AppMonitorService {
    fn drop(&mut self) {
        if self.is_running.load(Ordering::SeqCst) {
            log::info!("AppMonitorService dropped, stopping service...");
            self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let service = AppMonitorService::new();
        assert!(!service.is_running());
        assert_eq!(service.check_interval_secs, 300);
    }

    #[test]
    fn test_service_with_custom_interval() {
        let service = AppMonitorService::with_interval(60);
        assert_eq!(service.check_interval_secs, 60);
    }

    #[test]
    fn test_service_stop() {
        let service = AppMonitorService::new();
        service.is_running.store(true, Ordering::SeqCst);
        assert!(service.is_running());

        service.stop();
        assert!(!service.is_running());
    }

    #[test]
    fn test_default_service() {
        let service = AppMonitorService::default();
        assert!(!service.is_running());
        assert_eq!(service.check_interval_secs, 300);
    }
}
