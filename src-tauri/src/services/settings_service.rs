use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // 通用设置
    pub theme: String,
    pub language: String,

    // 窗口设置
    pub always_on_top: bool,
    pub start_minimized: bool,
    pub minimize_to_tray: bool,

    // 任务设置
    pub auto_save_context: bool,
    pub stale_task_days: i32,
    pub completed_tasks_retention_days: i32,

    // SQL 设置
    pub enable_clipboard_monitoring: bool,
    pub clipboard_interval: i32,
    pub sql_history_limit: i32,
    pub auto_detect_sql_type: bool,

    // 通知设置
    pub enable_notifications: bool,
    pub notify_on_task_complete: bool,
    pub notify_on_sql_detected: bool,
    pub task_notification: bool,
    pub worklog_reminder: bool,

    // AI 设置
    pub enable_ai: bool,
    pub ai_model: String,

    // Git 设置
    pub enable_git_integration: bool,
    pub auto_detect_branch: bool,
    pub git_enabled: bool,
    pub git_path: String,

    // 开机自启动
    pub auto_start: bool,

    // 日志设置
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "auto".to_string(),
            language: "zh-CN".to_string(),
            always_on_top: false,
            start_minimized: false,
            minimize_to_tray: true,
            auto_save_context: true,
            stale_task_days: 3,
            completed_tasks_retention_days: 7,
            enable_clipboard_monitoring: true,
            clipboard_interval: 2,
            sql_history_limit: 100,
            auto_detect_sql_type: true,
            enable_notifications: true,
            notify_on_task_complete: true,
            notify_on_sql_detected: false,
            task_notification: true,
            worklog_reminder: true,
            enable_ai: false,
            ai_model: "claude-3-sonnet-20240229".to_string(),
            enable_git_integration: true,
            auto_detect_branch: true,
            git_enabled: false,
            git_path: String::new(),
            auto_start: false,
            log_level: "info".to_string(),
        }
    }
}

pub struct SettingsService {
    conn: Arc<Mutex<Connection>>,
}

impl SettingsService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// 确保设置表存在
    pub fn ensure_table(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // 使用已有的 app_settings 表
        // 表结构: (key TEXT PRIMARY KEY, value TEXT)

        Ok(())
    }

    /// 获取所有设置
    pub fn get_settings(&self) -> Result<AppSettings> {
        let conn = self.conn.lock().unwrap();

        // 尝试从数据库读取设置
        let settings_json: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'app_settings'",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(json) = settings_json {
            // 解析JSON,如果失败则返回默认值
            serde_json::from_str(&json).or(Ok(AppSettings::default()))
        } else {
            Ok(AppSettings::default())
        }
    }

    /// 保存所有设置
    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let settings_json = serde_json::to_string(settings)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params!["app_settings", settings_json, timestamp],
        )?;

        Ok(())
    }

    /// 更新单个设置项
    pub fn update_setting(&self, key: &str, value: serde_json::Value) -> Result<()> {
        let mut settings = self.get_settings()?;

        // 根据key更新对应的字段
        match key {
            "theme" => {
                if let Some(s) = value.as_str() {
                    settings.theme = s.to_string();
                }
            }
            "language" => {
                if let Some(s) = value.as_str() {
                    settings.language = s.to_string();
                }
            }
            "always_on_top" => {
                if let Some(b) = value.as_bool() {
                    settings.always_on_top = b;
                }
            }
            "start_minimized" => {
                if let Some(b) = value.as_bool() {
                    settings.start_minimized = b;
                }
            }
            "minimize_to_tray" => {
                if let Some(b) = value.as_bool() {
                    settings.minimize_to_tray = b;
                }
            }
            "auto_save_context" => {
                if let Some(b) = value.as_bool() {
                    settings.auto_save_context = b;
                }
            }
            "stale_task_days" => {
                if let Some(n) = value.as_i64() {
                    settings.stale_task_days = n as i32;
                }
            }
            "completed_tasks_retention_days" => {
                if let Some(n) = value.as_i64() {
                    settings.completed_tasks_retention_days = n as i32;
                }
            }
            "enable_clipboard_monitoring" => {
                if let Some(b) = value.as_bool() {
                    settings.enable_clipboard_monitoring = b;
                }
            }
            "clipboard_interval" => {
                if let Some(n) = value.as_i64() {
                    settings.clipboard_interval = n as i32;
                }
            }
            "sql_history_limit" => {
                if let Some(n) = value.as_i64() {
                    settings.sql_history_limit = n as i32;
                }
            }
            "auto_detect_sql_type" => {
                if let Some(b) = value.as_bool() {
                    settings.auto_detect_sql_type = b;
                }
            }
            "enable_notifications" => {
                if let Some(b) = value.as_bool() {
                    settings.enable_notifications = b;
                }
            }
            "notify_on_task_complete" => {
                if let Some(b) = value.as_bool() {
                    settings.notify_on_task_complete = b;
                }
            }
            "notify_on_sql_detected" => {
                if let Some(b) = value.as_bool() {
                    settings.notify_on_sql_detected = b;
                }
            }
            "task_notification" => {
                if let Some(b) = value.as_bool() {
                    settings.task_notification = b;
                }
            }
            "worklog_reminder" => {
                if let Some(b) = value.as_bool() {
                    settings.worklog_reminder = b;
                }
            }
            "enable_ai" => {
                if let Some(b) = value.as_bool() {
                    settings.enable_ai = b;
                }
            }
            "ai_model" => {
                if let Some(s) = value.as_str() {
                    settings.ai_model = s.to_string();
                }
            }
            "enable_git_integration" => {
                if let Some(b) = value.as_bool() {
                    settings.enable_git_integration = b;
                }
            }
            "auto_detect_branch" => {
                if let Some(b) = value.as_bool() {
                    settings.auto_detect_branch = b;
                }
            }
            "git_enabled" => {
                if let Some(b) = value.as_bool() {
                    settings.git_enabled = b;
                }
            }
            "git_path" => {
                if let Some(s) = value.as_str() {
                    settings.git_path = s.to_string();
                }
            }
            "auto_start" => {
                if let Some(b) = value.as_bool() {
                    settings.auto_start = b;
                }
            }
            "log_level" => {
                if let Some(s) = value.as_str() {
                    settings.log_level = s.to_string();
                }
            }
            _ => {
                return Err(rusqlite::Error::InvalidQuery);
            }
        }

        self.save_settings(&settings)
    }

    /// 重置为默认设置
    pub fn reset_settings(&self) -> Result<()> {
        let default_settings = AppSettings::default();
        self.save_settings(&default_settings)
    }

    /// 导出设置为JSON字符串
    pub fn export_settings(&self) -> Result<String> {
        let settings = self.get_settings()?;
        serde_json::to_string_pretty(&settings)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    }

    /// 从JSON字符串导入设置
    pub fn import_settings(&self, json: &str) -> Result<()> {
        let settings: AppSettings = serde_json::from_str(json)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        self.save_settings(&settings)
    }
}
