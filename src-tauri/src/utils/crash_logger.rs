use std::fs::{self, OpenOptions};
use std::io::Write;
use std::panic;
use std::path::PathBuf;
use chrono::Local;

/// 获取崩溃日志目录
pub fn get_crash_log_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dev-assistant")
        .join("crash_logs")
}

/// 获取运行日志文件路径
pub fn get_runtime_log_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dev-assistant")
        .join("runtime.log")
}

/// 写入运行时日志
pub fn log_runtime(message: &str) {
    let log_path = get_runtime_log_path();

    // 确保目录存在
    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_line = format!("[{}] {}\n", timestamp, message);

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = file.write_all(log_line.as_bytes());
    }
}

/// 写入崩溃日志
fn write_crash_log(info: &str) {
    let crash_dir = get_crash_log_dir();

    // 确保目录存在
    if let Err(e) = fs::create_dir_all(&crash_dir) {
        eprintln!("无法创建崩溃日志目录: {}", e);
        return;
    }

    // 使用时间戳作为文件名
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let crash_file = crash_dir.join(format!("crash_{}.log", timestamp));

    // 构建崩溃报告
    let report = format!(
        "=== DevAssistant 崩溃报告 ===\n\
        时间: {}\n\
        版本: {}\n\
        平台: {}\n\
        \n\
        === 崩溃信息 ===\n\
        {}\n\
        \n\
        === 系统信息 ===\n\
        OS: {} {}\n\
        架构: {}\n\
        ",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        info,
        std::env::consts::OS,
        std::env::consts::FAMILY,
        std::env::consts::ARCH,
    );

    // 写入文件
    match fs::write(&crash_file, &report) {
        Ok(_) => {
            eprintln!("崩溃日志已保存到: {:?}", crash_file);
        }
        Err(e) => {
            eprintln!("无法写入崩溃日志: {}", e);
            eprintln!("崩溃信息:\n{}", report);
        }
    }
}

/// 设置全局 panic 处理器
pub fn setup_panic_handler() {
    let default_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        // 获取 panic 信息
        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "未知错误".to_string()
        };

        // 获取位置信息
        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "未知位置".to_string()
        };

        // 获取当前线程信息
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unnamed");

        // 尝试获取回溯信息
        let backtrace = std::backtrace::Backtrace::capture();
        let backtrace_str = format!("{}", backtrace);

        // 构建完整的崩溃信息
        let crash_info = format!(
            "Panic 发生!\n\
            线程: {}\n\
            位置: {}\n\
            信息: {}\n\
            \n\
            === 堆栈回溯 ===\n\
            {}",
            thread_name,
            location,
            payload,
            if backtrace_str.is_empty() || backtrace_str.contains("disabled") {
                "堆栈回溯不可用 (设置 RUST_BACKTRACE=1 环境变量以启用)".to_string()
            } else {
                backtrace_str
            }
        );

        // 写入崩溃日志
        write_crash_log(&crash_info);

        // 同时写入运行时日志
        log_runtime(&format!("CRASH: {} at {}", payload, location));

        // 调用默认处理器
        default_hook(panic_info);
    }));

    log_runtime("Panic 处理器已设置");
}

/// 清理过期的崩溃日志（保留最近30天的）
pub fn cleanup_old_crash_logs() {
    let crash_dir = get_crash_log_dir();

    if !crash_dir.exists() {
        return;
    }

    let cutoff = Local::now() - chrono::Duration::days(30);

    if let Ok(entries) = fs::read_dir(&crash_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified_time: chrono::DateTime<Local> = modified.into();
                    if modified_time < cutoff {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
}

/// 获取最近的崩溃日志
pub fn get_recent_crash_logs(limit: usize) -> Vec<(String, String)> {
    let crash_dir = get_crash_log_dir();
    let mut logs = Vec::new();

    if !crash_dir.exists() {
        return logs;
    }

    if let Ok(mut entries) = fs::read_dir(&crash_dir)
        .map(|entries| entries.flatten().collect::<Vec<_>>())
    {
        // 按修改时间排序（最新的在前）
        entries.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).ok();
            let b_time = b.metadata().and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        for entry in entries.into_iter().take(limit) {
            let filename = entry.file_name().to_string_lossy().to_string();
            if let Ok(content) = fs::read_to_string(entry.path()) {
                logs.push((filename, content));
            }
        }
    }

    logs
}
