// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;
mod services;
mod utils;

use commands::ai_commands::AiState;
use commands::shortcut_commands::ShortcutState;
use commands::sql_ai_commands::SqlAiState;
use db::connection::{init_database, DbConnection};
use services::clipboard_service::ClipboardService;
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

fn main() {
    // 初始化环境日志
    env_logger::init();

    // 初始化数据库
    let conn = init_database().expect("无法初始化数据库");

    // 尝试从数据库加载快捷键配置（在将 conn 移动到 Arc 之前）
    let saved_config = {
        use commands::shortcut_commands::ShortcutConfig;

        let config_json: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'shortcuts'",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(json) = config_json {
            serde_json::from_str::<ShortcutConfig>(&json).ok()
        } else {
            None
        }
    };

    let loaded_config =
        saved_config.unwrap_or_else(|| commands::shortcut_commands::ShortcutConfig::default());

    // 将 conn 移动到 Arc 中
    let db_state = DbConnection(Arc::new(Mutex::new(conn)));

    // 初始化快捷键状态
    let shortcut_state = ShortcutState::new();
    shortcut_state.set_config(loaded_config.clone());

    // 初始化 SQL AI 状态
    let sql_ai_state = SqlAiState::new();

    // 初始化全局 AI 状态
    let ai_state = AiState::new();

    // 获取数据库路径
    let db_path = dirs::data_local_dir()
        .expect("无法获取应用数据目录")
        .join("dev-assistant")
        .join("dev_assistant.db");

    // 启动剪贴板监控服务
    let clipboard_service = ClipboardService::new();
    clipboard_service.start_monitoring(db_path.to_str().expect("无法转换数据库路径").to_string());
    println!("剪贴板监控服务已启动");

    tauri::Builder::default()
        .manage(db_state)
        .manage(shortcut_state)
        .manage(sql_ai_state)
        .manage(ai_state)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(move |app| {
            // 注册全局快捷键: 任务看板
            let task_board_shortcut = loaded_config.task_board.clone();
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
                eprintln!("警告: 无法注册快捷键 {}: {}", task_board_shortcut, e);
            }

            // 注册全局快捷键: SQL历史
            let sql_history_shortcut = loaded_config.sql_history.clone();
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
                eprintln!("警告: 无法注册快捷键 {}: {}", sql_history_shortcut, e);
            }

            // 注册全局快捷键: 应用启动器
            let app_launcher_shortcut = loaded_config.app_launcher.clone();
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
                eprintln!("警告: 无法注册快捷键 {}: {}", app_launcher_shortcut, e);
            }

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
            commands::task_commands::import_tasks,
            commands::task_commands::get_import_template,
            commands::task_commands::get_tasks_by_quadrant,
            commands::task_commands::get_quadrant_statistics,
            // SQL 相关命令
            commands::sql_commands::save_sql,
            commands::sql_commands::get_recent_sqls,
            commands::sql_commands::get_favorite_sqls,
            commands::sql_commands::toggle_favorite_sql,
            commands::sql_commands::delete_sql,
            commands::sql_commands::update_sql_name_category,
            commands::sql_commands::batch_update_sql_name_category,
            commands::sql_commands::get_sql_categories,
            commands::sql_commands::add_sql_category,
            commands::sql_commands::update_sql_category,
            commands::sql_commands::delete_sql_category,
            commands::sql_commands::get_uncategorized_sqls,
            commands::sql_commands::get_sqls_by_category,
            // SQL 多标签相关命令
            commands::sql_commands::set_sql_categories,
            commands::sql_commands::add_sql_category_tag,
            commands::sql_commands::remove_sql_category_tag,
            commands::sql_commands::update_sql_name_and_categories,
            commands::sql_commands::batch_update_sql_name_categories,
            commands::sql_commands::get_category_sql_counts,
            // SQL AI 相关命令
            commands::sql_ai_commands::configure_sql_ai,
            commands::sql_ai_commands::get_sql_ai_status,
            commands::sql_ai_commands::test_sql_ai_connection,
            commands::sql_ai_commands::ai_classify_sqls,
            commands::sql_ai_commands::manual_classify_sql,
            // AI 相关命令
            commands::ai_commands::save_ai_config,
            commands::ai_commands::get_ai_config,
            commands::ai_commands::test_ai_connection,
            commands::ai_commands::is_ai_enabled,
            commands::ai_commands::ai_generate_work_log,
            commands::ai_commands::ai_polish_work_log,
            commands::ai_commands::ai_generate_weekly_report,
            commands::ai_commands::ai_classify_task,
            commands::ai_commands::ai_enhance_task_description,
            commands::ai_commands::ai_generate_subtasks,
            commands::ai_commands::ai_summarize_tasks,
            commands::ai_commands::ai_classify_apps,
            commands::ai_commands::ai_recommend_workflows,
            commands::ai_commands::ai_generate_app_description,
            // AI 日志相关命令
            commands::ai_commands::get_ai_logs,
            commands::ai_commands::get_ai_log_stats,
            commands::ai_commands::clear_ai_logs,
            commands::ai_commands::save_ai_log,
            // 窗口相关命令
            commands::window_commands::show_window,
            commands::window_commands::hide_window,
            commands::window_commands::toggle_window,
            commands::window_commands::show_window_with_route,
            commands::window_commands::set_always_on_top,
            commands::window_commands::get_always_on_top,
            commands::window_commands::toggle_always_on_top,
            // 工作日志相关命令
            commands::work_log_commands::save_work_log,
            commands::work_log_commands::get_work_log,
            commands::work_log_commands::get_recent_work_logs,
            commands::work_log_commands::delete_work_log,
            commands::work_log_commands::get_all_work_logs,
            // 数据库修复命令
            commands::db_repair_commands::repair_database,
            commands::db_repair_commands::get_database_stats,
            // AppLauncher 相关命令
            commands::app_launcher_commands::scan_installed_apps,
            commands::app_launcher_commands::add_manual_app,
            commands::app_launcher_commands::get_all_apps,
            commands::app_launcher_commands::get_app_by_id,
            commands::app_launcher_commands::search_apps,
            commands::app_launcher_commands::add_app,
            commands::app_launcher_commands::update_app,
            commands::app_launcher_commands::delete_app,
            commands::app_launcher_commands::launch_app,
            commands::app_launcher_commands::launch_apps,
            commands::app_launcher_commands::launch_workflow,
            commands::app_launcher_commands::get_categories,
            commands::app_launcher_commands::add_category,
            commands::app_launcher_commands::update_category,
            commands::app_launcher_commands::save_category,
            commands::app_launcher_commands::delete_category,
            commands::app_launcher_commands::get_workflows,
            commands::app_launcher_commands::get_workflow_by_id,
            commands::app_launcher_commands::add_workflow,
            commands::app_launcher_commands::update_workflow,
            commands::app_launcher_commands::delete_workflow,
            commands::app_launcher_commands::get_launch_history,
            commands::app_launcher_commands::clear_launch_history,
            commands::app_launcher_commands::toggle_app_pin,
            commands::app_launcher_commands::toggle_app_hidden,
            commands::app_launcher_commands::validate_path,
            commands::app_launcher_commands::export_config,
            commands::app_launcher_commands::import_config,
            commands::app_launcher_commands::refresh_all_icons,
            commands::app_launcher_commands::update_app_icon,
            commands::app_launcher_commands::get_launcher_settings,
            commands::app_launcher_commands::update_launcher_settings,
            // 快捷键相关命令
            commands::shortcut_commands::get_shortcut_config,
            commands::shortcut_commands::update_shortcut_config,
            commands::shortcut_commands::reset_shortcut_config,
            commands::shortcut_commands::validate_shortcut,
            commands::shortcut_commands::get_available_shortcuts,
            // 系统监控相关命令
            commands::system_commands::get_system_info,
            commands::system_commands::get_database_path,
            commands::system_commands::open_data_folder,
            commands::system_commands::export_database,
            commands::system_commands::clear_all_data,
            // 开机自启动相关命令
            commands::autostart_commands::get_autostart_status,
            commands::autostart_commands::set_autostart,
            commands::autostart_commands::toggle_autostart,
            // 设置相关命令
            commands::settings_commands::get_app_settings,
            commands::settings_commands::save_app_settings,
            commands::settings_commands::update_app_setting,
            commands::settings_commands::reset_app_settings,
            commands::settings_commands::export_app_settings,
            commands::settings_commands::import_app_settings,
            // 标签相关命令
            commands::tag_commands::create_tag,
            commands::tag_commands::get_all_tags,
            commands::tag_commands::get_tag_by_id,
            commands::tag_commands::update_tag,
            commands::tag_commands::delete_tag,
            commands::tag_commands::add_tag_to_task,
            commands::tag_commands::remove_tag_from_task,
            commands::tag_commands::get_task_tags,
            commands::tag_commands::get_tasks_by_tag,
            commands::tag_commands::add_tags_to_task,
            commands::tag_commands::remove_all_tags_from_task,
            commands::tag_commands::get_tag_usage_count,
            commands::tag_commands::get_tags_with_usage_count,
            commands::tag_commands::search_tags,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
