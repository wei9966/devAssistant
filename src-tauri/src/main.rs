// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;
mod prompts;
mod services;
mod utils;

use utils::crash_logger::{setup_panic_handler, cleanup_old_crash_logs, log_runtime};
use commands::ai_commands::AiState;
use commands::context_commands::{ContextManagerState, BatchProcessorState};
use commands::shortcut_commands::ShortcutState;
use commands::sql_ai_commands::SqlAiState;
use commands::vlm_commands::VlmState;
use db::connection::{init_database, DbConnection};
use services::clipboard_history_service::ClipboardHistoryService;
use services::scheduler_service::SchedulerService;
use services::activity_summary_service::ActivitySummaryService;
use services::vlm_service::VlmService;
use services::prompt_manager_service::PromptManager;
use services::notification_service::NotificationService;
use services::tips_service::TipsService;
use services::ai_service::AiService;
use services::todo_prediction_service::TodoPredictionService;
use services::tool_service::ToolService;
use commands::file_index_commands::FileIndexState;
use chrono::{Duration, Local, Timelike};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tauri::Manager;
use tauri::PhysicalPosition;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use tauri::menu::{Menu, MenuItem};

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
    // 设置 Windows 错误模式，禁用系统错误对话框（如"损坏的映像"等 DLL 加载错误）
    // 必须在最开始设置，避免任何 DLL 加载时弹出错误框
    #[cfg(windows)]
    {
        use winapi::um::errhandlingapi::SetErrorMode;
        use winapi::um::winbase::{SEM_FAILCRITICALERRORS, SEM_NOGPFAULTERRORBOX, SEM_NOOPENFILEERRORBOX};
        unsafe {
            // SEM_FAILCRITICALERRORS: 禁用严重错误对话框
            // SEM_NOGPFAULTERRORBOX: 禁用 GPF 错误框
            // SEM_NOOPENFILEERRORBOX: 禁用文件打开错误框
            SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
        }
    }

    // 设置 panic 处理器（必须在最开始）
    setup_panic_handler();

    // 初始化环境日志
    env_logger::init();

    // 清理过期的崩溃日志
    cleanup_old_crash_logs();

    log_runtime("应用启动中...");

    // 初始化数据库
    let conn = match init_database() {
        Ok(c) => {
            log_runtime("数据库初始化成功");
            c
        }
        Err(e) => {
            log_runtime(&format!("数据库初始化失败: {}", e));
            panic!("无法初始化数据库: {}", e);
        }
    };

    // 初始化 AI 提示词缓存
    {
        use services::prompt_db_service::PromptDbService;
        if let Err(e) = PromptDbService::refresh_cache(&conn) {
            log_runtime(&format!("AI 提示词缓存初始化失败: {}, 将在首次使用时加载", e));
        } else {
            log_runtime("AI 提示词缓存初始化成功");
        }
    }

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

    // 从设置加载日志级别
    {
        let settings_json: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'app_settings'",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(json) = settings_json {
            if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&json) {
                if let Some(log_level) = settings.get("log_level").and_then(|v| v.as_str()) {
                    let level = match log_level.to_lowercase().as_str() {
                        "trace" => log::LevelFilter::Trace,
                        "debug" => log::LevelFilter::Debug,
                        "info" => log::LevelFilter::Info,
                        "warn" => log::LevelFilter::Warn,
                        "error" => log::LevelFilter::Error,
                        "off" => log::LevelFilter::Off,
                        _ => log::LevelFilter::Info,
                    };
                    log::set_max_level(level);
                    log_runtime(&format!("已从设置加载日志级别: {}", log_level));
                }
            }
        }
    }

    // 将 conn 移动到 Arc 中
    let db_state = DbConnection(Arc::new(Mutex::new(conn)));

    // 初始化快捷键状态
    let shortcut_state = ShortcutState::new();
    shortcut_state.set_config(loaded_config.clone());

    // 初始化 SQL AI 状态
    let sql_ai_state = SqlAiState::new();

    // 初始化全局 AI 状态，并从数据库加载已保存的配置
    let ai_state = AiState::new();
    {
        let conn = db_state.0.lock().expect("获取数据库连接失败");
        match ai_state.load_from_db(&conn) {
            Ok(true) => log_runtime("AI 服务已从数据库配置初始化"),
            Ok(false) => log_runtime("AI 服务未配置或未启用"),
            Err(e) => log_runtime(&format!("加载 AI 配置失败: {}", e)),
        }
    }

    // 初始化 VLM 状态
    let vlm_state = VlmState::new();

    // 初始化上下文管理器状态
    let context_manager_state = ContextManagerState::new();

    // 初始化批量处理器状态
    let batch_processor_state = BatchProcessorState::new();

    // 初始化工具服务
    let tool_service = Mutex::new(ToolService::new());

    // 初始化文件索引状态
    let file_index_state = FileIndexState::new();
    log_runtime("文件索引服务已初始化");

    // 获取数据库路径
    let db_path = match dirs::data_local_dir() {
        Some(dir) => dir.join("dev-assistant").join("dev_assistant.db"),
        None => {
            log_runtime("无法获取应用数据目录，使用当前目录");
            std::path::PathBuf::from("dev_assistant.db")
        }
    };

    let db_path_str = db_path.to_str().unwrap_or("dev_assistant.db").to_string();
    log_runtime(&format!("数据库路径: {}", db_path_str));

    // 启动剪切板历史监控服务（统一监控，包含 SQL 检测）
    let clipboard_history_service = ClipboardHistoryService::new();
    clipboard_history_service.start_monitoring(db_path_str.clone());
    log_runtime("剪贴板历史监控服务已启动");

    // 初始化 PromptManager
    match PromptManager::get_instance() {
        Ok(_) => {
            log_runtime("PromptManager 初始化成功");
        }
        Err(e) => {
            log_runtime(&format!("PromptManager 初始化失败: {}, 将在首次使用时加载", e));
        }
    }

    // 初始化定时任务调度器 - 从数据库加载配置
    let scheduler_config = {
        let conn = db_state.0.lock().expect("获取数据库连接失败");
        let config_json: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'scheduler_config'",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(json) = config_json {
            match serde_json::from_str::<services::scheduler_service::SchedulerConfig>(&json) {
                Ok(config) => {
                    log_runtime(&format!(
                        "从数据库加载定时任务配置: Activity总结={}({}分钟), Tips={}({}分钟), TODO预测={}({}分钟), 小时总结={}({}分钟)",
                        config.enable_activity_summary, config.activity_summary_interval_minutes,
                        config.enable_tips, config.tips_interval_minutes,
                        config.enable_todo_prediction, config.todo_prediction_interval_minutes,
                        config.enable_hourly_summary, config.hourly_summary_interval_minutes
                    ));
                    config
                }
                Err(e) => {
                    log_runtime(&format!("解析定时任务配置失败，使用默认值: {}", e));
                    services::scheduler_service::SchedulerConfig::default()
                }
            }
        } else {
            log_runtime("未找到定时任务配置，使用默认值");
            services::scheduler_service::SchedulerConfig::default()
        }
    };

    let scheduler_service = SchedulerService::with_config(scheduler_config);
    let scheduler_clone = scheduler_service.clone();
    let db_path_for_scheduler = db_path_str.clone();

    // 启动任务提醒定时检查（每分钟检查一次）
    let db_path_for_task_reminder = db_path_str.clone();
    tauri::async_runtime::spawn(async move {
        use tokio::time::{interval, Duration as TokioDuration};
        let mut ticker = interval(TokioDuration::from_secs(60)); // 每分钟检查一次

        loop {
            ticker.tick().await;

            // 检查即将开始的任务（5分钟内）
            if let Ok(conn) = rusqlite::Connection::open(&db_path_for_task_reminder) {
                match services::task_service::TaskService::get_upcoming_tasks(&conn, 5) {
                    Ok(tasks) => {
                        for task in tasks {
                            if let Some(task_id) = task.id {
                                // 检查是否已经为该任务创建过提醒（通过标题唯一性）
                                let reminder_title = format!("任务即将开始: {}", task.title);
                                let exists: i64 = conn.query_row(
                                    "SELECT COUNT(*) FROM notifications WHERE title = ? AND notification_type = 'task_reminder' AND created_at > datetime('now', 'localtime', '-1 hour')",
                                    rusqlite::params![&reminder_title],
                                    |row| row.get(0)
                                ).unwrap_or(0);

                                if exists > 0 {
                                    // 已经提醒过，跳过
                                    continue;
                                }

                                let content = if let Some(scheduled_time) = &task.scheduled_start_time {
                                    format!("计划开始时间: {}\n{}", scheduled_time, task.description.unwrap_or_default())
                                } else {
                                    task.description.unwrap_or_else(|| "请准备开始此任务".to_string())
                                };

                                if let Err(e) = NotificationService::create(
                                    &conn,
                                    "task_reminder",
                                    &reminder_title,
                                    &content
                                ) {
                                    log::error!("创建任务提醒通知失败: {}", e);
                                } else {
                                    log::info!("任务提醒通知已创建: {}", task.title);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("获取即将开始的任务失败: {}", e);
                    }
                }
            }
        }
    });

    // 启动定时任务（使用 Tauri 的异步运行时）
    tauri::async_runtime::spawn(async move {
        // Activity 总结回调
        let on_activity_summary = {
            let db_path = db_path_for_scheduler.clone();
            move || {
                log::info!("执行 Activity 总结任务");

                // 使用 Tauri 的异步运行时执行异步任务
                let db_path = db_path.clone();
                tauri::async_runtime::spawn(async move {
                    let summary_service = ActivitySummaryService::new(db_path.clone());
                    let vlm_service = VlmService::new(db_path.clone());

                    // 聚合过去15分钟的截图
                    let end_time = Local::now();
                    let start_time = end_time - Duration::minutes(15);

                    match summary_service.generate_summary(start_time, end_time, &vlm_service).await {
                        Ok(summary) => {
                            log::info!("Activity 总结已生成: {}", summary.summary_text);

                            // 创建通知推送到通知中心
                            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                                let title = format!("{} - {} 活动总结", summary.start_time, summary.end_time);
                                if let Err(e) = NotificationService::create(
                                    &conn,
                                    "activity_summary",
                                    &title,
                                    &summary.summary_text
                                ) {
                                    log::error!("创建活动总结通知失败: {}", e);
                                } else {
                                    log::info!("活动总结通知已创建");
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("生成 Activity 总结失败: {}", e);
                        }
                    }
                });
            }
        };

        // Tips 回调 - 基于活动分析的智能提示
        let on_tips = {
            let db_path = db_path_for_scheduler.clone();
            move || {
                log_runtime("[Tips] 回调被触发，准备执行异步任务");

                let db_path = db_path.clone();
                // 使用 Tauri 的异步运行时执行异步任务
                let handle = tauri::async_runtime::spawn(async move {
                    log_runtime("[Tips] 异步任务开始执行");
                    // 第一步：检查设置和准备数据（使用 Connection 但不跨 await）
                    let (pattern, ai_config, use_ai) = {
                        let conn = match rusqlite::Connection::open(&db_path) {
                            Ok(c) => c,
                            Err(e) => {
                                log_runtime(&format!("[Tips] 打开数据库失败: {}", e));
                                return;
                            }
                        };

                        // 检查设置
                        let settings = match NotificationService::get_settings(&conn) {
                            Ok(s) => s,
                            Err(e) => {
                                log_runtime(&format!("[Tips] 获取通知设置失败: {}", e));
                                return;
                            }
                        };

                        if !settings.tips_enabled {
                            log_runtime("[Tips] Tips 功能未启用，跳过");
                            return;
                        }

                        // 检查今日数量限制
                        if let Ok(today_count) = NotificationService::get_today_tips_count(&conn) {
                            if today_count >= settings.tips_max_per_day {
                                log_runtime(&format!("[Tips] 今日 Tips 已达上限 ({}/{})", today_count, settings.tips_max_per_day));
                                return;
                            }
                        }

                        // 检查是否应该生成提示（避免重复生成）
                        let interval_minutes = settings.tips_interval_minutes as i64;
                        if let Ok(should_generate) = TipsService::should_generate_tip(&conn, interval_minutes) {
                            if !should_generate {
                                log_runtime("[Tips] 距离上次提示时间不足，跳过生成");
                                return;
                            }
                        }

                        // 分析最近1小时的活动模式
                        let pattern = match TipsService::analyze_recent_pattern(&conn, 1) {
                            Ok(p) => p,
                            Err(e) => {
                                log_runtime(&format!("[Tips] 分析活动模式失败: {}", e));
                                return;
                            }
                        };

                        // 如果没有活动数据，跳过生成
                        if pattern.total_activities == 0 {
                            log_runtime("[Tips] 没有活动数据，跳过 Tips 生成");
                            return;
                        }

                        log_runtime(&format!("[Tips] 活动分析: 连续工作 {} 分钟, 主要活动: {:?}, 主要应用: {:?}, 活动数: {}",
                            pattern.continuous_work_minutes,
                            pattern.dominant_activity_type,
                            pattern.dominant_app,
                            pattern.total_activities
                        ));

                        // 加载 AI 配置
                        let ai_config = AiService::load_config(&conn).ok();
                        let use_ai = ai_config.is_some();
                        log_runtime(&format!("[Tips] AI配置: use_ai={}", use_ai));

                        // Connection 在这里被 drop，不会跨越 await 边界
                        (pattern, ai_config, use_ai)
                    };

                    // 第二步：使用 AI 生成提示（使用 spawn_blocking 避免 Connection 跨 await）
                    log_runtime("[Tips] 开始生成提示内容...");
                    let (tip_content, category, priority) = if use_ai {
                        let ai_config_owned = ai_config.unwrap();
                        let ai_service = AiService::new(ai_config_owned.clone());

                        if !ai_service.is_configured() {
                            log_runtime("[Tips] AI 未配置，使用默认提示生成");
                            TipsService::generate_default_tip(&pattern)
                        } else {
                            log_runtime("[Tips] 使用 AI 生成提示...");
                            let db_path_for_ai = db_path.clone();
                            let pattern_for_ai = pattern.clone();

                            // 使用 spawn_blocking 在单独的线程中运行包含 Connection 的异步代码
                            match tokio::task::spawn_blocking(move || {
                                let ai_service_inner = AiService::new(ai_config_owned);
                                tokio::runtime::Handle::current().block_on(async move {
                                    let conn = match rusqlite::Connection::open(&db_path_for_ai) {
                                        Ok(c) => c,
                                        Err(e) => {
                                            return Err(anyhow::anyhow!("打开数据库失败: {}", e));
                                        }
                                    };

                                    TipsService::generate_tip_with_ai(&conn, &ai_service_inner, &pattern_for_ai).await
                                })
                            }).await {
                                Ok(Ok(result)) => {
                                    log_runtime("[Tips] AI 智能提示生成成功");
                                    result
                                }
                                Ok(Err(e)) => {
                                    log_runtime(&format!("[Tips] AI 生成失败，使用默认方式: {}", e));
                                    TipsService::generate_default_tip(&pattern)
                                }
                                Err(e) => {
                                    log_runtime(&format!("[Tips] spawn_blocking 失败: {}", e));
                                    TipsService::generate_default_tip(&pattern)
                                }
                            }
                        }
                    } else {
                        log_runtime("[Tips] AI 配置未加载，使用默认提示生成");
                        TipsService::generate_default_tip(&pattern)
                    };

                    // 第三步：保存结果（重新打开连接）
                    // 安全截断UTF-8字符串，避免在字符边界中间截断
                    let truncated_tip: String = tip_content.chars().take(50).collect();
                    log_runtime(&format!("[Tips] 生成的提示: [{}] {}", category.as_str(), truncated_tip));
                    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                        // 保存提示到 tips 表
                        if let Err(e) = TipsService::save_tip(&conn, &tip_content, &category, &priority) {
                            log_runtime(&format!("[Tips] 保存 Tips 失败: {}", e));
                            return;
                        }

                        // 同时创建通知推送到通知中心
                        let title = match category {
                            services::tips_service::TipCategory::Health => "💪 健康提醒",
                            services::tips_service::TipCategory::Productivity => "📈 生产力建议",
                            services::tips_service::TipCategory::Focus => "🎯 专注力提醒",
                        };

                        if let Err(e) = NotificationService::create(&conn, "tip", title, &tip_content) {
                            log_runtime(&format!("[Tips] 创建通知失败: {}", e));
                        } else {
                            log_runtime(&format!("[Tips] 智能提示已创建通知: {}", title));
                        }
                    } else {
                        log_runtime("[Tips] 打开数据库失败，无法保存提示");
                    }
                    log_runtime("[Tips] 异步任务执行完成");
                });
                log_runtime("[Tips] 异步任务已提交");
                // 不需要等待 handle，让它在后台运行
                drop(handle);
            }
        };

        // TODO 预测回调 - 从截图上下文智能推断任务
        let on_todo_prediction = {
            let db_path = db_path_for_scheduler.clone();
            move || {
                log::info!("执行 TODO 预测任务");

                let db_path = db_path.clone();
                tauri::async_runtime::spawn(async move {
                    // 使用 spawn_blocking 避免 Connection 跨越 await 边界
                    match tokio::task::spawn_blocking(move || {
                        tokio::runtime::Handle::current().block_on(async move {
                            // 1. 加载 AI 配置
                            let conn = match rusqlite::Connection::open(&db_path) {
                                Ok(c) => c,
                                Err(e) => {
                                    log::error!("打开数据库失败: {}", e);
                                    return;
                                }
                            };

                            let ai_config = match AiService::load_config(&conn) {
                                Ok(config) => config,
                                Err(e) => {
                                    log::warn!("加载 AI 配置失败，跳过 TODO 预测: {}", e);
                                    return;
                                }
                            };

                            // 2. 检查 AI 是否配置
                            let ai_service = AiService::new(ai_config);
                            if !ai_service.is_configured() {
                                log::info!("AI 未配置，跳过 TODO 预测");
                                return;
                            }

                            // 2.5 读取调度器配置获取预测间隔时间
                            let interval_minutes: Option<i64> = conn
                                .query_row(
                                    "SELECT value FROM app_settings WHERE key = 'scheduler_config'",
                                    [],
                                    |row| row.get::<_, String>(0),
                                )
                                .ok()
                                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                                .and_then(|v| v.get("todo_prediction_interval_minutes")?.as_i64());

                            log::info!(
                                "[TODO预测] 使用间隔时间: {} 分钟",
                                interval_minutes.unwrap_or(120)
                            );

                            // Connection 在这里被 drop
                            drop(conn);

                            // 3. 创建 TodoPredictionService 实例
                            let prediction_service = TodoPredictionService::new(db_path.clone());

                            // 4. 调用 predict_and_save() 方法（保存到 predicted_tasks 表，不直接创建任务）
                            // 传入从配置读取的间隔时间
                            match prediction_service.predict_and_save(&ai_service, interval_minutes).await {
                                Ok(result) => {
                                    log::info!("TODO 预测完成: 分析了 {} 条上下文，预测了 {} 个任务",
                                        result.analyzed_contexts, result.tasks.len());

                                    // 5. 如果有预测结果，创建通知提醒用户查看
                                    if !result.tasks.is_empty() {
                                        let conn = match rusqlite::Connection::open(&db_path) {
                                            Ok(c) => c,
                                            Err(e) => {
                                                log::error!("打开数据库失败: {}", e);
                                                return;
                                            }
                                        };

                                        // 构建通知内容
                                        let task_list: Vec<String> = result.tasks.iter()
                                            .take(3)  // 最多显示前3个任务
                                            .enumerate()
                                            .map(|(i, task)| format!("{}. {} ({})", i + 1, task.description, task.priority))
                                            .collect();

                                        let notification_content = if result.tasks.len() > 3 {
                                            format!("{}\n\n...还有 {} 个任务建议，点击查看完整列表",
                                                task_list.join("\n"),
                                                result.tasks.len() - 3)
                                        } else {
                                            format!("{}\n\n点击查看详情并选择是否添加到任务面板", task_list.join("\n"))
                                        };

                                        let title = format!("AI 发现 {} 个待办任务建议", result.tasks.len());

                                        // 6. 通知用户查看预测任务列表
                                        if let Err(e) = NotificationService::create(
                                            &conn,
                                            "todo_prediction",
                                            &title,
                                            &notification_content
                                        ) {
                                            log::error!("创建 TODO 预测通知失败: {}", e);
                                        } else {
                                            log::info!("TODO 预测通知已创建，共 {} 个待处理建议", result.tasks.len());
                                        }
                                    } else {
                                        log::info!("暂无待办任务建议");
                                    }
                                }
                                Err(e) => {
                                    log::warn!("TODO 预测失败: {}", e);
                                }
                            }
                        })
                    }).await {
                        Ok(_) => {},
                        Err(e) => {
                            log::error!("spawn_blocking 失败: {}", e);
                        }
                    }
                });
            }
        };

        // 小时总结回调
        let on_hourly_summary = {
            let db_path = db_path_for_scheduler.clone();
            move || {
                log::info!("执行小时总结任务");
                let db_path = db_path.clone();
                tauri::async_runtime::spawn(async move {
                    // 获取上一个小时的时间
                    let now = Local::now();
                    let last_hour = now - Duration::hours(1);
                    let date = last_hour.format("%Y-%m-%d").to_string();
                    let hour = last_hour.hour() as i32;

                    let summary_service = ActivitySummaryService::new(db_path);
                    match summary_service.generate_hourly_summary(&date, hour).await {
                        Ok(summary) => {
                            log::info!("小时总结已生成: {} {}点 - {}", date, hour, summary.summary_text);
                        }
                        Err(e) => {
                            log::warn!("生成小时总结失败: {}", e);
                        }
                    }
                });
            }
        };

        // 启动调度器
        if let Err(e) = scheduler_clone.start(on_activity_summary, on_tips, on_todo_prediction, on_hourly_summary).await {
            log::error!("启动定时任务调度器失败: {}", e);
        } else {
            log::info!("定时任务调度器已启动");
        }
    });

    tauri::Builder::default()
        .manage(db_state)
        .manage(shortcut_state)
        .manage(sql_ai_state)
        .manage(ai_state)
        .manage(vlm_state)
        .manage(context_manager_state)
        .manage(batch_processor_state)
        .manage(tool_service)
        .manage(file_index_state)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_http::init())
        .setup(move |app| {
            // ========== 系统托盘设置 ==========
            let app_handle = app.handle().clone();

            // 创建托盘菜单
            let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出程序", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            // 创建系统托盘
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(false)
                .tooltip("DevAssistant")
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            // 退出整个程序
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键单击托盘图标时显示主窗口
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 监听主窗口关闭事件，改为隐藏而不是关闭
            let main_window = app.get_webview_window("main").unwrap();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    // 阻止默认关闭行为
                    api.prevent_close();
                    // 隐藏窗口
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
            });

            log_runtime("系统托盘初始化完成");

            // ========== 全局快捷键设置 ==========
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

            log_runtime("全局快捷键注册完成");

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

            log_runtime("应用 setup 完成");

            // 自动恢复屏幕采集（如果之前开启了）
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // 延迟一点启动，确保所有服务初始化完成
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                // 获取数据库连接检查设置
                let db_path = dirs::data_local_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("dev-assistant")
                    .join("dev_assistant.db");

                if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                    // 读取设置
                    let settings_json: Option<String> = conn
                        .query_row(
                            "SELECT value FROM app_settings WHERE key = 'context_settings'",
                            [],
                            |row| row.get(0),
                        )
                        .ok();

                    if let Some(json) = settings_json {
                        if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&json) {
                            let capture_enabled = settings.get("captureEnabled")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);

                            if capture_enabled {
                                log::info!("检测到屏幕采集已启用，正在自动恢复...");

                                // 通过调用命令来启动采集
                                if let Some(context_state) = app_handle.try_state::<ContextManagerState>() {
                                    if let Some(batch_state) = app_handle.try_state::<BatchProcessorState>() {
                                        if let Some(db_state) = app_handle.try_state::<DbConnection>() {
                                            // 加载配置
                                            let config = {
                                                if let Ok(conn) = db_state.0.lock() {
                                                    let settings = commands::context_commands::load_context_settings_internal(&conn);
                                                    services::context_manager_service::ContextConfig {
                                                        capture_interval_secs: settings.capture_interval,
                                                        similarity_threshold: settings.similarity_threshold,
                                                        save_screenshots: settings.save_screenshots,
                                                        screenshot_dir: settings.screenshot_dir,
                                                        idle_timeout_secs: settings.idle_timeout_secs,
                                                    }
                                                } else {
                                                    services::context_manager_service::ContextConfig::default()
                                                }
                                            };

                                            let manager = context_state.get_or_init().await;
                                            let _ = manager.update_config(config).await;

                                            // 准备回调所需的状态
                                            let db_arc = db_state.0.clone();
                                            let batch_state_clone = batch_state.inner().clone();
                                            let db_path_str = db_path.to_string_lossy().to_string();

                                            // 自动启动批处理器
                                            let batch_processor = batch_state_clone.get_or_init(db_path_str.clone()).await;
                                            let processor_state = batch_processor.get_state().await;
                                            if !processor_state.is_running {
                                                if let Err(e) = batch_processor.start().await {
                                                    log::warn!("启动批处理器失败: {}", e);
                                                } else {
                                                    let processor_clone = batch_processor.clone();
                                                    tauri::async_runtime::spawn(async move {
                                                        if let Err(e) = processor_clone.run_processing_loop().await {
                                                            log::error!("批量处理循环出错: {}", e);
                                                        }
                                                    });
                                                }
                                            }

                                            // 启动采集
                                            let db_path_for_batch = db_path_str.clone();
                                            if let Err(e) = manager.start_capture(move |context| {
                                                // 保存到数据库
                                                let saved_id = match db_arc.lock() {
                                                    Ok(conn) => {
                                                        match services::context_store_service::ContextStoreService::save_context(&conn, &context) {
                                                            Ok(id) => {
                                                                println!("截图采集成功并已保存: {} - {:?}, ID: {}",
                                                                    context.captured_at, context.app_name, id);
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

                                                // 保存成功后，加入批量处理队列
                                                if let Some(id) = saved_id {
                                                    if let Some(ref path) = context.screenshot_path {
                                                        let batch_state_inner = batch_state_clone.clone();
                                                        let db_path_inner = db_path_for_batch.clone();
                                                        let screenshot_path = path.clone();

                                                        tokio::spawn(async move {
                                                            let processor = batch_state_inner.get_or_init(db_path_inner).await;
                                                            let batch_item = services::screenshot_batch_processor_service::BatchItem {
                                                                context_id: id,
                                                                screenshot_path,
                                                            };

                                                            if let Err(e) = processor.enqueue(batch_item).await {
                                                                eprintln!("加入批量处理队列失败: {}", e);
                                                            }
                                                        });
                                                    }
                                                }
                                            }).await {
                                                log::error!("自动恢复屏幕采集失败: {}", e);
                                            } else {
                                                log::info!("屏幕采集已自动恢复");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });

            // 自动初始化文件索引
            let app_handle_for_index = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // 延迟启动，确保其他服务先初始化完成
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                log_runtime("[文件索引] 启动自动初始化...");

                // 从 app_handle 获取 FileIndexState
                if let Some(file_index_state) = app_handle_for_index.try_state::<FileIndexState>() {
                    match commands::file_index_commands::auto_init_file_index(
                        app_handle_for_index.clone(),
                        &file_index_state,
                    ).await {
                        Ok(msg) => {
                            log_runtime(&format!("[文件索引] 自动初始化完成: {}", msg));
                        }
                        Err(e) => {
                            log_runtime(&format!("[文件索引] 自动初始化失败: {}", e));
                        }
                    }
                } else {
                    log_runtime("[文件索引] 无法获取 FileIndexState，跳过自动初始化");
                }
            });

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
            commands::task_commands::get_upcoming_tasks,
            commands::task_commands::update_task_progress,
            // 里程碑相关命令
            commands::milestone_commands::create_task_milestone,
            commands::milestone_commands::get_task_milestones,
            commands::milestone_commands::delete_task_milestone,
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
            commands::sql_ai_commands::ai_classify_sqls,
            commands::sql_ai_commands::manual_classify_sql,
            // SQL 模板相关命令
            commands::sql_template_commands::consolidate_sql_templates,
            commands::sql_template_commands::get_sql_templates,
            commands::sql_template_commands::get_hot_sql_templates,
            commands::sql_template_commands::get_template_variants,
            commands::sql_template_commands::get_templates_by_table,
            commands::sql_template_commands::identify_template_scenes,
            commands::sql_template_commands::toggle_template_favorite,
            commands::sql_template_commands::get_template_group_stats,
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
            // AI Chat 相关命令
            commands::ai_chat_commands::ai_assistant_chat,
            commands::ai_chat_commands::ai_get_dashboard_stats,
            commands::ai_chat_commands::ai_save_chat_session,
            commands::ai_chat_commands::ai_get_chat_sessions,
            commands::ai_chat_commands::ai_delete_chat_session,
            commands::ai_chat_commands::ai_get_session_messages,
            commands::ai_chat_commands::ai_save_chat_message,
            commands::ai_chat_commands::ai_clear_session_messages,
            commands::ai_chat_commands::ai_get_process_logs,
            commands::ai_chat_commands::ai_clear_process_logs,
            commands::ai_chat_commands::ai_reset_conversation_context,
            commands::ai_chat_commands::ai_get_conversation_stats,
            // VLM 相关命令
            commands::vlm_commands::vlm_save_config,
            commands::vlm_commands::vlm_get_config,
            commands::vlm_commands::vlm_test_connection,
            commands::vlm_commands::vlm_analyze_image,
            commands::vlm_commands::vlm_is_enabled,
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
            commands::work_log_commands::context_get_day_summary,
            commands::work_log_commands::context_get_week_summary,
            commands::work_log_commands::work_log_generate_with_context,
            commands::work_log_commands::work_log_generate_weekly_with_context,
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
            commands::app_launcher_commands::show_in_folder,
            commands::app_launcher_commands::export_config,
            commands::app_launcher_commands::import_config,
            commands::app_launcher_commands::refresh_all_icons,
            commands::app_launcher_commands::update_app_icon,
            commands::app_launcher_commands::get_launcher_settings,
            commands::app_launcher_commands::update_launcher_settings,
            commands::app_launcher_commands::full_scan_apps,
            commands::app_launcher_commands::sync_apps_to_db,
            commands::app_launcher_commands::scan_and_sync_apps,
            commands::app_launcher_commands::start_app_monitor,
            commands::app_launcher_commands::stop_app_monitor,
            commands::app_launcher_commands::get_monitor_status,
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
            // 屏幕上下文相关命令
            commands::context_commands::context_start_capture,
            commands::context_commands::context_stop_capture,
            commands::context_commands::context_get_status,
            commands::context_commands::context_capture_once,
            commands::context_commands::context_update_config,
            commands::context_commands::context_get_config,
            commands::context_commands::context_reset_daily_stats,
            commands::context_commands::context_get_settings,
            commands::context_commands::context_update_settings,
            commands::context_commands::context_get_default_screenshot_dir,
            commands::context_commands::context_get_stats,
            commands::context_commands::context_list_by_date,
            commands::context_commands::context_generate_summary,
            commands::context_commands::context_delete_by_date,
            commands::context_commands::context_cleanup_old,
            // 批量截图处理器相关命令
            commands::context_commands::context_start_batch_processor,
            commands::context_commands::context_stop_batch_processor,
            commands::context_commands::context_get_batch_processor_status,
            commands::context_commands::context_process_screenshots_manually,
            commands::context_commands::context_count_unprocessed_screenshots,
            commands::context_commands::context_get_unprocessed_screenshot_ids,
            // 截图回顾相关命令
            commands::screenshot_commands::screenshot_get_image,
            commands::screenshot_commands::screenshot_list,
            commands::screenshot_commands::screenshot_get_detail,
            commands::screenshot_commands::screenshot_get_activities,
            commands::screenshot_commands::screenshot_debug_dates,
            commands::screenshot_commands::get_active_window_info,
            commands::screenshot_commands::get_running_apps,
            // 时间线相关命令
            commands::timeline_commands::get_timeline,
            commands::timeline_commands::generate_daily_report,
            commands::timeline_commands::merge_timeline_items,
            commands::timeline_commands::get_activities_by_date,
            commands::timeline_commands::auto_merge_by_time_window,
            // 崩溃日志相关命令
            commands::system_commands::get_crash_logs,
            commands::system_commands::get_crash_log_path,
            commands::system_commands::get_runtime_log,
            commands::system_commands::set_log_level,
            commands::system_commands::get_log_level,
            commands::system_commands::log_update_info,
            commands::system_commands::get_device_info,
            commands::system_commands::get_app_version,
            // 提示词相关命令
            commands::prompt_commands::get_prompt_config,
            commands::prompt_commands::save_prompt_config,
            commands::prompt_commands::reset_prompt_config,
            commands::prompt_commands::get_prompt_template,
            commands::prompt_commands::update_prompt_template,
            // 提示词数据库相关命令
            commands::prompt_db_commands::get_all_prompts,
            commands::prompt_db_commands::get_prompt,
            commands::prompt_db_commands::get_prompts_by_module,
            commands::prompt_db_commands::update_prompt,
            commands::prompt_db_commands::reset_prompt,
            commands::prompt_db_commands::reset_all_prompts,
            commands::prompt_db_commands::render_prompt_preview,
            commands::prompt_db_commands::get_prompt_variables,
            commands::prompt_db_commands::refresh_prompt_cache,
            // 通知中心相关命令
            commands::notification_commands::notification_list,
            commands::notification_commands::notification_get_unread_count,
            commands::notification_commands::notification_mark_read,
            commands::notification_commands::notification_mark_all_read,
            commands::notification_commands::notification_delete,
            commands::notification_commands::notification_clear_all,
            commands::notification_commands::notification_get_settings,
            commands::notification_commands::notification_update_settings,
            commands::notification_commands::notification_generate_daily_report,
            commands::notification_commands::notification_generate_weekly_report,
            commands::notification_commands::notification_generate_tip,
            commands::notification_commands::notification_check_and_generate,
            // 统计相关命令
            commands::statistics_commands::statistics_get_heatmap,
            commands::statistics_commands::statistics_get_daily_trend,
            commands::statistics_commands::statistics_get_hourly_distribution,
            commands::statistics_commands::statistics_get_app_usage,
            commands::statistics_commands::statistics_get_activity_types,
            // 日报相关命令
            commands::report_commands::report_generate,
            commands::report_commands::report_get,
            commands::report_commands::report_list,
            commands::report_commands::report_delete,
            commands::report_commands::report_exists,
            // 智能提示相关命令
            commands::tips_commands::tips_generate,
            commands::tips_commands::tips_list,
            commands::tips_commands::tips_mark_read,
            commands::tips_commands::tips_get_unread_count,
            commands::tips_commands::tips_analyze_pattern,
            commands::tips_commands::tips_should_generate,
            commands::tips_commands::tips_cleanup_old,
            // 调度器相关命令
            commands::scheduler_commands::get_scheduler_config,
            commands::scheduler_commands::save_scheduler_config,
            commands::scheduler_commands::get_scheduler_status,
            // 预测任务相关命令
            commands::prediction_commands::get_pending_predictions,
            commands::prediction_commands::get_all_predictions,
            commands::prediction_commands::accept_prediction,
            commands::prediction_commands::ignore_prediction,
            commands::prediction_commands::accept_predictions,
            commands::prediction_commands::ignore_predictions,
            commands::prediction_commands::get_pending_prediction_count,
            commands::prediction_commands::cleanup_old_predictions,
            // 番茄钟相关命令
            commands::pomodoro_commands::create_pomodoro_session,
            commands::pomodoro_commands::start_pomodoro,
            commands::pomodoro_commands::pause_pomodoro,
            commands::pomodoro_commands::resume_pomodoro,
            commands::pomodoro_commands::cancel_pomodoro,
            commands::pomodoro_commands::complete_pomodoro,
            commands::pomodoro_commands::record_pomodoro_distraction,
            commands::pomodoro_commands::update_pomodoro_focus_time,
            commands::pomodoro_commands::update_pomodoro_ai_analysis,
            commands::pomodoro_commands::get_pomodoro_session,
            commands::pomodoro_commands::get_active_pomodoro_session,
            commands::pomodoro_commands::get_today_pomodoro_sessions,
            commands::pomodoro_commands::get_task_pomodoro_sessions,
            commands::pomodoro_commands::get_pomodoro_focus_apps,
            commands::pomodoro_commands::add_pomodoro_focus_app,
            commands::pomodoro_commands::remove_pomodoro_focus_app,
            commands::pomodoro_commands::get_pomodoro_today_stats,
            commands::pomodoro_commands::get_pomodoro_stats_by_date,
            commands::pomodoro_commands::get_pomodoro_stats_range,
            // 番茄钟 AI 功能
            commands::pomodoro_commands::pomodoro_ai_task_breakdown,
            commands::pomodoro_commands::pomodoro_ai_focus_analysis,
            commands::pomodoro_commands::pomodoro_ai_daily_review,
            commands::pomodoro_commands::pomodoro_ai_progress_eval,
            // 番茄钟任务中断与恢复
            commands::pomodoro_commands::pomodoro_ai_analyze_interruption,
            commands::pomodoro_commands::pomodoro_ai_resume_suggestion,
            commands::pomodoro_commands::pomodoro_ai_quick_resume,
            commands::pomodoro_commands::pomodoro_get_session_activities,
            commands::pomodoro_commands::pomodoro_get_session_app_usage,
            // 番茄钟会话分析
            commands::pomodoro_commands::pomodoro_ai_analyze_session,
            // 工具箱相关命令
            commands::tool_commands::check_port_usage,
            commands::tool_commands::kill_process_by_pid,
            commands::tool_commands::open_tool_window,
            commands::tool_commands::get_all_tools,
            commands::tool_commands::get_tool_by_id,
            commands::tool_commands::set_tool_pinned,
            commands::tool_commands::pin_tool,
            commands::tool_commands::unpin_tool,
            commands::tool_commands::get_pinned_tools,
            commands::tool_commands::record_tool_usage,
            commands::tool_commands::get_recent_tools,
            commands::tool_commands::open_tool_container,
            // 文件搜索相关命令
            commands::file_search_commands::search_files,
            commands::file_search_commands::open_file,
            commands::file_search_commands::open_file_in_folder,
            commands::file_search_commands::get_all_drives,
            // 文件索引相关命令
            commands::file_index_commands::check_file_index_admin_privilege,
            commands::file_index_commands::get_file_index_status,
            commands::file_index_commands::start_file_indexing,
            commands::file_index_commands::search_indexed_files,
            commands::file_index_commands::get_file_index_stats,
            commands::file_index_commands::clear_file_index,
            commands::file_index_commands::start_file_watching,
            commands::file_index_commands::stop_file_watching,
            commands::file_index_commands::get_available_drives,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
