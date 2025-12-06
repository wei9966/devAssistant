use serde::{Deserialize, Serialize};
use sysinfo::System;
use crate::utils::crash_logger;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
}

#[tauri::command]
pub fn get_system_info() -> Result<SystemInfo, String> {
    let mut sys = System::new_all();

    // 需要刷新两次CPU才能获取准确的使用率
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu();

    // 刷新内存信息
    sys.refresh_memory();

    // 获取全局CPU使用率
    let cpu_usage = sys.global_cpu_info().cpu_usage();

    // 获取内存信息
    let memory_total = sys.total_memory();
    let memory_used = sys.used_memory();
    let memory_usage = if memory_total > 0 {
        (memory_used as f32 / memory_total as f32) * 100.0
    } else {
        0.0
    };

    Ok(SystemInfo {
        cpu_usage,
        memory_usage,
        memory_total,
        memory_used,
    })
}

/// 获取数据库文件路径
#[tauri::command]
pub fn get_database_path() -> Result<String, String> {
    let db_path = dirs::data_local_dir()
        .ok_or("无法获取应用数据目录".to_string())?
        .join("dev-assistant")
        .join("dev_assistant.db");

    Ok(db_path.to_string_lossy().to_string())
}

/// 打开数据文件夹
#[tauri::command]
pub fn open_data_folder() -> Result<(), String> {
    let data_dir = dirs::data_local_dir()
        .ok_or("无法获取应用数据目录".to_string())?
        .join("dev-assistant");

    // 确保目录存在
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("创建数据目录失败: {}", e))?;
    }

    // 根据操作系统打开文件夹
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(data_dir)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(data_dir)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(data_dir)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    Ok(())
}

/// 导出数据库
#[tauri::command]
pub fn export_database(export_path: String) -> Result<(), String> {
    let db_path = dirs::data_local_dir()
        .ok_or("无法获取应用数据目录".to_string())?
        .join("dev-assistant")
        .join("dev_assistant.db");

    if !db_path.exists() {
        return Err("数据库文件不存在".to_string());
    }

    // 复制数据库文件
    std::fs::copy(&db_path, &export_path)
        .map_err(|e| format!("导出数据库失败: {}", e))?;

    Ok(())
}

/// 清空所有数据
#[tauri::command]
pub fn clear_all_data() -> Result<(), String> {
    let db_path = dirs::data_local_dir()
        .ok_or("无法获取应用数据目录".to_string())?
        .join("dev-assistant")
        .join("dev_assistant.db");

    if !db_path.exists() {
        return Ok(()); // 数据库不存在,认为已清空
    }

    // 打开数据库连接
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("打开数据库失败: {}", e))?;

    // 清空所有表的数据(保留表结构)
    let tables = vec![
        "tasks",
        "work_contexts",
        "sql_records",
        "sql_categories",
        "sql_record_categories",
        "work_logs",
        "tags",
        "task_tags",
        "ai_logs",
        "app_launcher_apps",
        "app_launcher_categories",
        "app_launcher_workflows",
        "app_launcher_workflow_apps",
        "app_launcher_launch_history",
    ];

    for table in tables {
        conn.execute(&format!("DELETE FROM {}", table), [])
            .map_err(|e| format!("清空表 {} 失败: {}", table, e))?;
    }

    // 不清空 app_settings 表,保留用户设置

    Ok(())
}

/// 崩溃日志条目
#[derive(Debug, Serialize, Deserialize)]
pub struct CrashLogEntry {
    pub filename: String,
    pub content: String,
}

/// 获取崩溃日志列表
#[tauri::command]
pub fn get_crash_logs(limit: Option<usize>) -> Result<Vec<CrashLogEntry>, String> {
    let logs = crash_logger::get_recent_crash_logs(limit.unwrap_or(10));
    Ok(logs
        .into_iter()
        .map(|(filename, content)| CrashLogEntry { filename, content })
        .collect())
}

/// 获取崩溃日志目录路径
#[tauri::command]
pub fn get_crash_log_path() -> Result<String, String> {
    let path = crash_logger::get_crash_log_dir();
    Ok(path.to_string_lossy().to_string())
}

/// 获取运行时日志内容（最近N行）
#[tauri::command]
pub fn get_runtime_log(lines: Option<usize>) -> Result<String, String> {
    let log_path = crash_logger::get_runtime_log_path();

    if !log_path.exists() {
        return Ok("暂无运行日志".to_string());
    }

    let content = std::fs::read_to_string(&log_path)
        .map_err(|e| format!("读取日志失败: {}", e))?;

    let limit = lines.unwrap_or(100);
    let log_lines: Vec<&str> = content.lines().collect();
    let start = if log_lines.len() > limit {
        log_lines.len() - limit
    } else {
        0
    };

    Ok(log_lines[start..].join("\n"))
}

/// 设置日志级别
#[tauri::command]
pub fn set_log_level(level: String) -> Result<(), String> {
    let log_level = match level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        "off" => log::LevelFilter::Off,
        _ => return Err(format!("无效的日志级别: {}", level)),
    };

    log::set_max_level(log_level);
    crash_logger::log_runtime(&format!("日志级别已设置为: {}", level));
    log::info!("日志级别已动态更改为: {}", level);

    Ok(())
}

/// 获取当前日志级别
#[tauri::command]
pub fn get_log_level() -> String {
    log::max_level().to_string().to_lowercase()
}

/// 记录更新日志
#[tauri::command]
pub fn log_update_info(message: String) {
    crash_logger::log_runtime(&format!("[Updater] {}", message));
    log::info!("[Updater] {}", message);
}
