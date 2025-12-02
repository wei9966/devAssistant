use crate::db::connection::DbConnection;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State, PhysicalPosition};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[cfg(windows)]
use winapi::um::winuser::{GetCursorPos, GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
#[cfg(windows)]
use winapi::shared::windef::POINT;

/// 获取鼠标当前所在屏幕的中心位置和尺寸
#[cfg(windows)]
fn get_mouse_monitor_center(window_width: i32, window_height: i32) -> Option<(i32, i32)> {
    unsafe {
        // 获取鼠标位置
        let mut cursor_pos = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut cursor_pos) == 0 {
            return None;
        }

        // 获取鼠标所在的显示器
        let monitor = MonitorFromPoint(cursor_pos, MONITOR_DEFAULTTONEAREST);
        if monitor.is_null() {
            return None;
        }

        // 获取显示器信息
        let mut monitor_info: MONITORINFO = std::mem::zeroed();
        monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut monitor_info) == 0 {
            return None;
        }

        // 计算窗口在该显示器中央的位置
        let work_area = monitor_info.rcWork;
        let monitor_width = work_area.right - work_area.left;
        let monitor_height = work_area.bottom - work_area.top;

        let x = work_area.left + (monitor_width - window_width) / 2;
        let y = work_area.top + (monitor_height - window_height) / 2;

        Some((x, y))
    }
}

#[cfg(not(windows))]
fn get_mouse_monitor_center(_window_width: i32, _window_height: i32) -> Option<(i32, i32)> {
    // 非 Windows 平台暂不支持，返回 None 使用默认居中
    None
}

/// 将窗口移动到鼠标所在屏幕的中央
fn center_window_on_mouse_screen<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) {
    // 获取窗口尺寸
    if let Ok(size) = window.outer_size() {
        let width = size.width as i32;
        let height = size.height as i32;

        // 获取鼠标所在屏幕的中心位置
        if let Some((x, y)) = get_mouse_monitor_center(width, height) {
            let _ = window.set_position(PhysicalPosition::new(x, y));
        } else {
            // 如果获取失败，使用默认的居中方法
            let _ = window.center();
        }
    } else {
        let _ = window.center();
    }
}

/// 快捷键配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ShortcutConfig {
    pub task_board: String,
    pub sql_history: String,
    pub app_launcher: String,
    pub quick_task: String,
    pub clipboard_history: String,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            task_board: "Ctrl+Shift+N".to_string(),
            sql_history: "Ctrl+Shift+S".to_string(),
            app_launcher: "Ctrl+Shift+Space".to_string(),
            quick_task: "Ctrl+Shift+T".to_string(),
            clipboard_history: "Ctrl+Shift+C".to_string(),
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
        old_config.quick_task.as_str(),
        old_config.clipboard_history.as_str(),
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

    // SQL面板使用独立透明窗口
    let sql_history_shortcut = config.sql_history.clone();
    if let Err(e) = app.global_shortcut().on_shortcut(
        sql_history_shortcut.as_str(),
        move |app, _shortcut, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                // 获取 SQL 面板窗口
                if let Some(sql_window) = app.get_webview_window("sql-panel") {
                    let is_visible = sql_window.is_visible().unwrap_or(false);

                    if is_visible {
                        // 如果已显示，则隐藏
                        let _ = sql_window.hide();
                    } else {
                        // 显示并移动到鼠标所在屏幕中央
                        center_window_on_mouse_screen(&sql_window);
                        let _ = sql_window.show();
                        let _ = sql_window.set_focus();
                    }
                }
            }
        },
    ) {
        return Err(format!(
            "无法注册SQL历史快捷键 {}: {}",
            sql_history_shortcut, e
        ));
    }

    // 应用启动器使用独立透明窗口
    let app_launcher_shortcut = config.app_launcher.clone();
    if let Err(e) = app.global_shortcut().on_shortcut(
        app_launcher_shortcut.as_str(),
        move |app, _shortcut, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                // 获取启动器窗口
                if let Some(launcher_window) = app.get_webview_window("launcher") {
                    let is_visible = launcher_window.is_visible().unwrap_or(false);

                    if is_visible {
                        // 如果已显示，则隐藏
                        let _ = launcher_window.hide();
                    } else {
                        // 显示并移动到鼠标所在屏幕中央
                        center_window_on_mouse_screen(&launcher_window);
                        let _ = launcher_window.show();
                        let _ = launcher_window.set_focus();
                    }
                }
            }
        },
    ) {
        return Err(format!(
            "无法注册应用启动器快捷键 {}: {}",
            app_launcher_shortcut, e
        ));
    }

    // 快速任务使用独立透明窗口
    let quick_task_shortcut = config.quick_task.clone();
    if let Err(e) = app.global_shortcut().on_shortcut(
        quick_task_shortcut.as_str(),
        move |app, _shortcut, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                // 获取快速任务窗口
                if let Some(task_window) = app.get_webview_window("quick-task") {
                    let is_visible = task_window.is_visible().unwrap_or(false);

                    if is_visible {
                        // 如果已显示，则隐藏
                        let _ = task_window.hide();
                    } else {
                        // 显示并移动到鼠标所在屏幕中央
                        center_window_on_mouse_screen(&task_window);
                        let _ = task_window.show();
                        let _ = task_window.set_focus();
                    }
                }
            }
        },
    ) {
        return Err(format!(
            "无法注册快速任务快捷键 {}: {}",
            quick_task_shortcut, e
        ));
    }

    // 剪切板历史使用独立透明窗口
    let clipboard_history_shortcut = config.clipboard_history.clone();
    if let Err(e) = app.global_shortcut().on_shortcut(
        clipboard_history_shortcut.as_str(),
        move |app, _shortcut, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                // 获取剪切板历史窗口
                if let Some(clipboard_window) = app.get_webview_window("clipboard-history") {
                    let is_visible = clipboard_window.is_visible().unwrap_or(false);

                    if is_visible {
                        // 如果已显示，则隐藏
                        let _ = clipboard_window.hide();
                    } else {
                        // 显示并移动到鼠标所在屏幕中央
                        center_window_on_mouse_screen(&clipboard_window);
                        let _ = clipboard_window.show();
                        let _ = clipboard_window.set_focus();
                    }
                }
            }
        },
    ) {
        return Err(format!(
            "无法注册剪切板历史快捷键 {}: {}",
            clipboard_history_shortcut, e
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
    let now = chrono::Local::now().timestamp();

    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?, ?, ?)",
        [&"shortcuts" as &dyn rusqlite::ToSql, &config_json, &now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
