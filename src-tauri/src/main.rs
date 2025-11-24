// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod services;
mod db;
mod utils;

use db::connection::{DbConnection, init_database};
use services::clipboard_service::ClipboardService;
use std::sync::Mutex;
use tauri::Emitter;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri::Manager;

fn main() {
    // 初始化环境日志
    env_logger::init();

    // 初始化数据库
    let conn = init_database().expect("无法初始化数据库");
    let db_state = DbConnection(Mutex::new(conn));

    // 获取数据库路径
    let db_path = dirs::data_local_dir()
        .expect("无法获取应用数据目录")
        .join("dev-assistant")
        .join("dev_assistant.db");

    // 启动剪贴板监控服务
    let clipboard_service = ClipboardService::new();
    clipboard_service.start_monitoring(
        db_path.to_str().expect("无法转换数据库路径").to_string()
    );
    println!("剪贴板监控服务已启动");

    tauri::Builder::default()
        .manage(db_state)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // 注册全局快捷键: Ctrl+Shift+N - 显示窗口并导航到任务看板
            app.global_shortcut().on_shortcut("Ctrl+Shift+N", move |app, _shortcut, event| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.emit("navigate-to", "/task-board");
                    }
                }
            })?;

            // 注册全局快捷键: Ctrl+Shift+S - 显示窗口并导航到SQL历史
            app.global_shortcut().on_shortcut("Ctrl+Shift+S", move |app, _shortcut, event| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.emit("navigate-to", "/sql-history");
                    }
                }
            })?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 任务相关命令
            commands::task_commands::get_all_tasks,
            commands::task_commands::get_completed_tasks,
            commands::task_commands::create_task,
            commands::task_commands::start_task,
            commands::task_commands::pause_task,
            commands::task_commands::defer_task,
            commands::task_commands::complete_task,
            commands::task_commands::update_task,
            commands::task_commands::delete_task,
            commands::task_commands::get_stale_tasks,
            commands::task_commands::get_current_branch,
            // SQL 相关命令
            commands::sql_commands::save_sql,
            commands::sql_commands::get_recent_sqls,
            commands::sql_commands::get_favorite_sqls,
            commands::sql_commands::toggle_favorite_sql,
            commands::sql_commands::delete_sql,
            // 窗口相关命令
            commands::window_commands::show_window,
            commands::window_commands::hide_window,
            commands::window_commands::toggle_window,
            commands::window_commands::show_window_with_route,
            // 工作日志相关命令
            commands::work_log_commands::save_work_log,
            commands::work_log_commands::get_work_log,
            commands::work_log_commands::get_recent_work_logs,
            commands::work_log_commands::delete_work_log,
            commands::work_log_commands::get_all_work_logs,
            // 数据库修复命令
            commands::db_repair_commands::repair_database,
            commands::db_repair_commands::get_database_stats,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
