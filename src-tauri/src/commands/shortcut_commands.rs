use crate::db::connection::DbConnection;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

/// 快捷键配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShortcutConfig {
    pub task_board: String,
    pub sql_history: String,
    pub app_launcher: String,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            task_board: "Ctrl+Shift+N".to_string(),
            sql_history: "Ctrl+Shift+S".to_string(),
            app_launcher: "Ctrl+Shift+Space".to_string(),
        }
    }
}

/// 快捷键状态管理
pub struct ShortcutState {
    config: Arc<Mutex<ShortcutConfig>>,
}

impl ShortcutState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(ShortcutConfig::default())),
        }
    }

    pub fn get_config(&self) -> ShortcutConfig {
        self.config.lock().unwrap().clone()
    }

    pub fn set_config(&self, config: ShortcutConfig) {
        *self.config.lock().unwrap() = config;
    }
}

/// 获取当前快捷键配置
#[tauri::command]
pub fn get_shortcut_config(
    db: State<DbConnection>,
    state: State<ShortcutState>,
) -> Result<ShortcutConfig, String> {
    // 先尝试从数据库加载
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    if let Some(config) = load_config_from_db(&conn)? {
        // 更新内存中的配置
        state.set_config(config.clone());
        Ok(config)
    } else {
        // 如果数据库中没有，返回当前内存中的配置（或默认配置）
        Ok(state.get_config())
    }
}

/// 更新快捷键配置
#[tauri::command]
pub fn update_shortcut_config(
    app: AppHandle,
    db: State<DbConnection>,
    state: State<ShortcutState>,
    config: ShortcutConfig,
) -> Result<(), String> {
    // 先注销所有旧的快捷键
    let old_config = state.get_config();
    let shortcuts = vec![
        old_config.task_board.as_str(),
        old_config.sql_history.as_str(),
        old_config.app_launcher.as_str(),
    ];

    for shortcut in shortcuts {
        if let Err(e) = app.global_shortcut().unregister(shortcut) {
            eprintln!("警告: 无法注销快捷键 {}: {}", shortcut, e);
        }
    }

    // 注册新的快捷键
    let task_board_shortcut = config.task_board.clone();
    if let Err(e) = app.global_shortcut().on_shortcut(
        task_board_shortcut.as_str(),
        move |app, _shortcut, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("navigate-to", "/task-board");
                }
            }
        },
    ) {
        return Err(format!(
            "无法注册任务看板快捷键 {}: {}",
            task_board_shortcut, e
        ));
    }

    let sql_history_shortcut = config.sql_history.clone();
    if let Err(e) = app.global_shortcut().on_shortcut(
        sql_history_shortcut.as_str(),
        move |app, _shortcut, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("navigate-to", "/sql-history");
                }
            }
        },
    ) {
        return Err(format!(
            "无法注册SQL历史快捷键 {}: {}",
            sql_history_shortcut, e
        ));
    }

    let app_launcher_shortcut = config.app_launcher.clone();
    if let Err(e) = app.global_shortcut().on_shortcut(
        app_launcher_shortcut.as_str(),
        move |app, _shortcut, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("navigate-to", "/app-launcher");
                }
            }
        },
    ) {
        return Err(format!(
            "无法注册应用启动器快捷键 {}: {}",
            app_launcher_shortcut, e
        ));
    }

    // 保存配置到内存
    state.set_config(config.clone());

    // 保存配置到数据库
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    save_config_to_db(&conn, &config)?;

    Ok(())
}

/// 重置快捷键为默认值
#[tauri::command]
pub fn reset_shortcut_config(
    app: AppHandle,
    db: State<DbConnection>,
    state: State<ShortcutState>,
) -> Result<(), String> {
    let default_config = ShortcutConfig::default();
    update_shortcut_config(app, db, state, default_config)
}

/// 验证快捷键格式是否有效
#[tauri::command]
pub fn validate_shortcut(shortcut: String) -> Result<bool, String> {
    // 简单的验证:检查是否包含修饰键和按键
    let parts: Vec<&str> = shortcut.split('+').collect();

    if parts.len() < 2 {
        return Ok(false);
    }

    // 检查是否包含至少一个修饰键
    let modifiers = vec!["Ctrl", "Shift", "Alt", "Super"];
    let has_modifier = parts.iter().any(|p| modifiers.contains(p));

    Ok(has_modifier)
}

/// 获取所有可用的快捷键
#[tauri::command]
pub fn get_available_shortcuts() -> Result<HashMap<String, Vec<String>>, String> {
    let mut shortcuts = HashMap::new();

    shortcuts.insert(
        "modifiers".to_string(),
        vec!["Ctrl".to_string(), "Shift".to_string(), "Alt".to_string()],
    );

    shortcuts.insert(
        "keys".to_string(),
        vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "E".to_string(),
            "F".to_string(),
            "G".to_string(),
            "H".to_string(),
            "I".to_string(),
            "J".to_string(),
            "K".to_string(),
            "L".to_string(),
            "M".to_string(),
            "N".to_string(),
            "O".to_string(),
            "P".to_string(),
            "Q".to_string(),
            "R".to_string(),
            "S".to_string(),
            "T".to_string(),
            "U".to_string(),
            "V".to_string(),
            "W".to_string(),
            "X".to_string(),
            "Y".to_string(),
            "Z".to_string(),
            "Space".to_string(),
            "F1".to_string(),
            "F2".to_string(),
            "F3".to_string(),
            "F4".to_string(),
            "F5".to_string(),
            "F6".to_string(),
            "F7".to_string(),
            "F8".to_string(),
            "F9".to_string(),
            "F10".to_string(),
            "F11".to_string(),
            "F12".to_string(),
        ],
    );

    Ok(shortcuts)
}

/// 从数据库加载快捷键配置
fn load_config_from_db(conn: &Connection) -> Result<Option<ShortcutConfig>, String> {
    let config_json: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'shortcuts'",
            [],
            |row| row.get(0),
        )
        .ok();

    if let Some(json) = config_json {
        serde_json::from_str(&json)
            .map(Some)
            .map_err(|e| e.to_string())
    } else {
        Ok(None)
    }
}

/// 保存快捷键配置到数据库
fn save_config_to_db(conn: &Connection, config: &ShortcutConfig) -> Result<(), String> {
    let config_json = serde_json::to_string(config).map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?, ?, ?)",
        [&"shortcuts" as &dyn rusqlite::ToSql, &config_json, &now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
