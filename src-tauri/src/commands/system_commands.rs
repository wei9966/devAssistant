use serde::{Deserialize, Serialize};
use sysinfo::System;

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
