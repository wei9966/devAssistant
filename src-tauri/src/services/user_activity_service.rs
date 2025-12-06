// User Activity Service
// 用户活动检测服务 - 检测用户空闲时间

use anyhow::Result;

#[cfg(windows)]
use winapi::um::sysinfoapi::GetTickCount;
#[cfg(windows)]
use winapi::um::winuser::{GetLastInputInfo, LASTINPUTINFO};

/// 用户活动检测服务
pub struct UserActivityService;

impl UserActivityService {
    /// 创建用户活动检测服务实例
    pub fn new() -> Self {
        Self
    }

    /// 获取用户空闲时间（秒）
    /// 返回自上次鼠标/键盘输入以来的秒数
    #[cfg(windows)]
    pub fn get_idle_seconds(&self) -> Result<u64> {
        unsafe {
            let mut last_input_info = LASTINPUTINFO {
                cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                dwTime: 0,
            };

            // 获取最后输入时间
            if GetLastInputInfo(&mut last_input_info) == 0 {
                return Err(anyhow::anyhow!("获取最后输入时间失败"));
            }

            // 获取系统运行时间（毫秒）
            let tick_count = GetTickCount();

            // 计算空闲时间（毫秒）
            let idle_time_ms = tick_count.saturating_sub(last_input_info.dwTime);

            // 转换为秒
            Ok((idle_time_ms / 1000) as u64)
        }
    }

    /// 非Windows平台返回0（始终活跃）
    #[cfg(not(windows))]
    pub fn get_idle_seconds(&self) -> Result<u64> {
        // TODO: 实现其他平台的空闲检测
        Ok(0)
    }

    /// 检查用户是否空闲超过指定秒数
    pub fn is_idle(&self, timeout_secs: u64) -> Result<bool> {
        let idle_secs = self.get_idle_seconds()?;
        Ok(idle_secs >= timeout_secs)
    }

    /// 检查用户是否活跃（未超过指定空闲时间）
    pub fn is_active(&self, timeout_secs: u64) -> Result<bool> {
        Ok(!self.is_idle(timeout_secs)?)
    }
}

impl Default for UserActivityService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_idle_seconds() {
        let service = UserActivityService::new();
        let result = service.get_idle_seconds();

        // 应该成功
        assert!(result.is_ok());

        if let Ok(seconds) = result {
            println!("当前空闲秒数: {} 秒", seconds);
            // 空闲时间应该是合理的（小于7天）
            assert!(seconds < 86400 * 7);
        }
    }

    #[test]
    fn test_is_idle() {
        let service = UserActivityService::new();

        // 测试5分钟阈值
        let result = service.is_idle(300);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_active() {
        let service = UserActivityService::new();

        // 如果空闲时间小于1秒，应该是活跃的
        let result = service.is_active(1);

        if let Ok(is_active) = result {
            println!("用户当前是否活跃: {}", is_active);
        }
    }
}
