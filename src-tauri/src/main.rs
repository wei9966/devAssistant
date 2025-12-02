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
use services::clipboard_history_service::ClipboardHistoryService;
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tauri::Manager;
use tauri::PhysicalPosition;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[cfg(windows)]
use winapi::um::winuser::{GetCursorPos, GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
#[cfg(windows)]
use winapi::shared::windef::POINT;

/// 获取鼠标当前所在屏幕的中心位置
#[cfg(windows)]
fn get_mouse_monitor_center(window_width: i32, window_height: i32) -> Option<(i32, i32)> {
    unsafe {
        let mut cursor_pos = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut cursor_pos) == 0 {
            return None;
        }

        let monitor = MonitorFromPoint(cursor_pos, MONITOR_DEFAULTTONEAREST);
        if monitor.is_null() {
            return None;
        }

        let mut monitor_info: MONITORINFO = std::mem::zeroed();
        monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut monitor_info) == 0 {
            return None;
        }

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
    None
}

/// 将窗口移动到鼠标所在屏幕的中央
fn center_window_on_mouse_screen<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) {
    if let Ok(size) = window.outer_size() {
        let width = size.width as i32;
        let height = size.height as i32;

        if let Some((x, y)) = get_mouse_monitor_center(width, height) {
            let _ = window.set_position(PhysicalPosition::new(x, y));
        } else {
            let _ = window.center();
        }
    } else {
        let _ = window.center();
    }
}

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

    // 启动剪贴板监控服务（SQL 历史）
    let clipboard_service = ClipboardService::new();
    clipboard_service.start_monitoring(db_path.to_str().expect("无法转换数据库路径").to_string());
    println!("剪贴板SQL监控服务已启动");

    // 启动剪切板历史监控服务
    let clipboard_history_service = ClipboardHistoryService::new();
    clipboard_history_service.start_monitoring(db_path.to_str().expect("无法转换数据库路径").to_string());
    println!("剪贴板历史监控服务已启动");

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
            // 使用独立透明窗口
            let sql_history_shortcut = loaded_config.sql_history.clone();
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
                eprintln!("警告: 无法注册快捷键 {}: {}", sql_history_shortcut, e);
            }

            // 注册全局快捷键: 应用启动器
            // 使用独立透明窗口，类似 Mac Spotlight
            let app_launcher_shortcut = loaded_config.app_launcher.clone();
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
                eprintln!("警告: 无法注册快捷键 {}: {}", app_launcher_shortcut, e);
            }

            // 注册全局快捷键: 快速任务
            // 使用独立透明窗口
            let quick_task_shortcut = loaded_config.quick_task.clone();
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
                eprintln!("警告: 无法注册快捷键 {}: {}", quick_task_shortcut, e);
            }

            // 注册全局快捷键: 剪切板历史
            // 使用独立透明窗口
            let clipboard_history_shortcut = loaded_config.clipboard_history.clone();
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
                eprintln!("警告: 无法注册快捷键 {}: {}", clipboard_history_shortcut, e);
            }

            // 注册全局快捷键: 复制最近图片路径
            let copy_image_path_shortcut = loaded_config.copy_image_path.clone();
            let db_path_for_shortcut = db_path.clone();
            if let Err(e) = app.global_shortcut().on_shortcut(
                copy_image_path_shortcut.as_str(),
                move |_app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        // 直接查询数据库获取最近图片路径
                        if let Ok(conn) = rusqlite::Connection::open(&db_path_for_shortcut) {
                            let result: Result<String, _> = conn.query_row(
                                "SELECT COALESCE(image_path, content) FROM clipboard_history WHERE content_type = 'image' ORDER BY created_at DESC LIMIT 1",
                                [],
                                |row| row.get(0),
                            );

                            if let Ok(path) = result {
                                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                    if clipboard.set_text(&path).is_ok() {
                                        println!("✓ 已复制最近图片路径: {}", path);
                                    }
                                }
                            } else {
                                println!("没有找到图片记录");
                            }
                        }
                    }
                },
            ) {
                eprintln!("警告: 无法注册快捷键 {}: {}", copy_image_path_shortcut, e);
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
            commands::task_commands::get_tasks_by_date_range,
            commands::task_commands::update_task_display_date,
            commands::task_commands::move_task_to_today,
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
            commands::ai_commands::ai_chat,
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
            // 周计划相关命令
            commands::weekly_plan_commands::save_weekly_plan,
            commands::weekly_plan_commands::get_weekly_plan,
            commands::weekly_plan_commands::get_all_weekly_plans,
            commands::weekly_plan_commands::get_recent_weekly_plans,
            commands::weekly_plan_commands::update_weekly_plan_tasks,
            commands::weekly_plan_commands::update_weekly_plan_status,
            commands::weekly_plan_commands::delete_weekly_plan,
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
            commands::app_launcher_commands::save_workflow,
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
            // 剪切板历史相关命令
            commands::clipboard_commands::get_clipboard_history,
            commands::clipboard_commands::copy_from_clipboard_history,
            commands::clipboard_commands::delete_clipboard_history_item,
            commands::clipboard_commands::clear_clipboard_history,
            commands::clipboard_commands::toggle_clipboard_pin,
            commands::clipboard_commands::search_clipboard_history,
            commands::clipboard_commands::get_clipboard_config,
            commands::clipboard_commands::update_clipboard_config,
            commands::clipboard_commands::get_clipboard_history_count,
            commands::clipboard_commands::copy_image_path,
            commands::clipboard_commands::copy_latest_image_path,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
