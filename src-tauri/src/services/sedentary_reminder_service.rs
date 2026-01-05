// Sedentary Reminder Service
// 久坐提醒服务 - 检测用户连续工作时间并发送提醒

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use chrono::Local;

use super::user_activity_service::UserActivityService;

/// 久坐提醒配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SedentaryReminderConfig {
    /// 是否启用久坐提醒
    pub enabled: bool,
    /// 提醒间隔（分钟），默认45分钟
    pub reminder_interval_minutes: u32,
    /// 提醒文案列表
    pub tips: Vec<String>,
    /// 判断用户空闲的阈值（秒），超过此时间认为用户休息了
    #[serde(default = "default_idle_threshold")]
    pub idle_threshold_seconds: u64,
}

fn default_idle_threshold() -> u64 {
    60 // 默认60秒
}

impl Default for SedentaryReminderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            reminder_interval_minutes: 45,
            idle_threshold_seconds: 60,
            tips: vec![
                "站起来活动一下身体吧！".to_string(),
                "喝杯水，让眼睛休息一下。".to_string(),
                "做几个简单的伸展运动。".to_string(),
                "走动几分钟，促进血液循环。".to_string(),
                "眺望远方，放松眼部肌肉。".to_string(),
                "深呼吸几次，放松身心。".to_string(),
                "检查一下坐姿，保持脊椎挺直。".to_string(),
                "转动一下脖子和肩膀。".to_string(),
            ],
        }
    }
}

/// 久坐提醒状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SedentaryStatus {
    /// 是否正在运行
    pub is_running: bool,
    /// 连续工作时间（秒）
    pub continuous_work_seconds: u64,
    /// 上次提醒时间
    pub last_reminder_time: Option<String>,
    /// 今日提醒次数
    pub today_reminder_count: u32,
}

/// 久坐提醒服务
pub struct SedentaryReminderService {
    /// 配置
    config: Arc<Mutex<SedentaryReminderConfig>>,
    /// 连续工作时间（秒）
    continuous_work_seconds: Arc<Mutex<u64>>,
    /// 上次检查时间
    last_check_time: Arc<Mutex<Option<Instant>>>,
    /// 上次提醒时间
    last_reminder_time: Arc<Mutex<Option<String>>>,
    /// 今日提醒次数
    today_reminder_count: Arc<Mutex<u32>>,
    /// 是否正在运行
    is_running: Arc<AtomicBool>,
    /// 用户活动检测服务
    activity_service: UserActivityService,
}

impl SedentaryReminderService {
    /// 创建久坐提醒服务实例
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(SedentaryReminderConfig::default())),
            continuous_work_seconds: Arc::new(Mutex::new(0)),
            last_check_time: Arc::new(Mutex::new(None)),
            last_reminder_time: Arc::new(Mutex::new(None)),
            today_reminder_count: Arc::new(Mutex::new(0)),
            is_running: Arc::new(AtomicBool::new(false)),
            activity_service: UserActivityService::new(),
        }
    }

    /// 从数据库加载配置创建服务
    pub fn with_config(config: SedentaryReminderConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            continuous_work_seconds: Arc::new(Mutex::new(0)),
            last_check_time: Arc::new(Mutex::new(None)),
            last_reminder_time: Arc::new(Mutex::new(None)),
            today_reminder_count: Arc::new(Mutex::new(0)),
            is_running: Arc::new(AtomicBool::new(false)),
            activity_service: UserActivityService::new(),
        }
    }

    /// 获取配置
    pub fn get_config(&self) -> SedentaryReminderConfig {
        self.config.lock().unwrap().clone()
    }

    /// 设置配置
    pub fn set_config(&self, config: SedentaryReminderConfig) {
        let mut cfg = self.config.lock().unwrap();
        *cfg = config;
    }

    /// 获取状态
    pub fn get_status(&self) -> SedentaryStatus {
        SedentaryStatus {
            is_running: self.is_running.load(Ordering::SeqCst),
            continuous_work_seconds: *self.continuous_work_seconds.lock().unwrap(),
            last_reminder_time: self.last_reminder_time.lock().unwrap().clone(),
            today_reminder_count: *self.today_reminder_count.lock().unwrap(),
        }
    }

    /// 重置计时器
    pub fn reset_timer(&self) {
        let mut work_seconds = self.continuous_work_seconds.lock().unwrap();
        *work_seconds = 0;

        let mut last_check = self.last_check_time.lock().unwrap();
        *last_check = Some(Instant::now());

        log::info!("[久坐提醒] 计时器已重置");
    }

    /// 检查并发送提醒
    /// 返回是否需要发送提醒以及提醒内容
    pub fn check_and_remind(&self) -> Result<Option<String>> {
        let config = self.config.lock().unwrap().clone();

        if !config.enabled {
            return Ok(None);
        }

        // 获取用户空闲时间
        let idle_seconds = self.activity_service.get_idle_seconds()?;

        let mut work_seconds = self.continuous_work_seconds.lock().unwrap();
        let mut last_check = self.last_check_time.lock().unwrap();

        let now = Instant::now();

        // 计算自上次检查以来的时间
        let elapsed = if let Some(last) = *last_check {
            now.duration_since(last).as_secs()
        } else {
            0
        };

        *last_check = Some(now);

        // 如果用户空闲超过阈值，重置连续工作时间
        if idle_seconds >= config.idle_threshold_seconds {
            if *work_seconds > 0 {
                log::info!("[久坐提醒] 检测到用户休息 {} 秒，重置连续工作时间", idle_seconds);
            }
            *work_seconds = 0;
            return Ok(None);
        }

        // 用户活跃，累加工作时间
        *work_seconds += elapsed;

        // 检查是否需要提醒
        let reminder_interval_seconds = (config.reminder_interval_minutes as u64) * 60;

        if *work_seconds >= reminder_interval_seconds {
            // 需要提醒
            let tip = self.get_random_tip(&config.tips);

            // 更新上次提醒时间
            let mut last_reminder = self.last_reminder_time.lock().unwrap();
            *last_reminder = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

            // 增加今日提醒次数
            let mut count = self.today_reminder_count.lock().unwrap();
            *count += 1;

            // 重置连续工作时间
            *work_seconds = 0;

            log::info!("[久坐提醒] 发送提醒: {}", tip);

            return Ok(Some(tip));
        }

        Ok(None)
    }

    /// 获取随机提示文案
    fn get_random_tip(&self, tips: &[String]) -> String {
        if tips.is_empty() {
            return "站起来活动一下吧！".to_string();
        }

        use std::time::SystemTime;
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as usize;

        let index = seed % tips.len();
        tips[index].clone()
    }

    /// 启动后台监控
    pub fn start<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(String) + Send + 'static,
    {
        if self.is_running.load(Ordering::SeqCst) {
            log::warn!("[久坐提醒] 服务已在运行中");
            return Ok(());
        }

        self.is_running.store(true, Ordering::SeqCst);
        self.reset_timer();

        let config = self.config.clone();
        let continuous_work_seconds = self.continuous_work_seconds.clone();
        let last_check_time = self.last_check_time.clone();
        let last_reminder_time = self.last_reminder_time.clone();
        let today_reminder_count = self.today_reminder_count.clone();
        let is_running = self.is_running.clone();

        // 启动后台线程
        std::thread::spawn(move || {
            let activity_service = UserActivityService::new();

            log::info!("[久坐提醒] 后台监控线程已启动");

            while is_running.load(Ordering::SeqCst) {
                // 每30秒检查一次
                std::thread::sleep(Duration::from_secs(30));

                if !is_running.load(Ordering::SeqCst) {
                    break;
                }

                let cfg = config.lock().unwrap().clone();

                if !cfg.enabled {
                    continue;
                }

                // 获取用户空闲时间
                let idle_seconds = match activity_service.get_idle_seconds() {
                    Ok(s) => s,
                    Err(e) => {
                        log::warn!("[久坐提醒] 获取空闲时间失败: {}", e);
                        continue;
                    }
                };

                let mut work_seconds = continuous_work_seconds.lock().unwrap();
                let mut last_check = last_check_time.lock().unwrap();

                let now = Instant::now();

                // 计算自上次检查以来的时间
                let elapsed = if let Some(last) = *last_check {
                    now.duration_since(last).as_secs()
                } else {
                    30 // 默认30秒
                };

                *last_check = Some(now);

                // 如果用户空闲超过阈值，重置连续工作时间
                if idle_seconds >= cfg.idle_threshold_seconds {
                    if *work_seconds > 0 {
                        log::debug!("[久坐提醒] 用户休息中，空闲 {} 秒", idle_seconds);
                    }
                    *work_seconds = 0;
                    continue;
                }

                // 用户活跃，累加工作时间
                *work_seconds += elapsed;

                log::debug!("[久坐提醒] 连续工作 {} 秒 / {} 秒",
                    *work_seconds,
                    (cfg.reminder_interval_minutes as u64) * 60
                );

                // 检查是否需要提醒
                let reminder_interval_seconds = (cfg.reminder_interval_minutes as u64) * 60;

                if *work_seconds >= reminder_interval_seconds {
                    // 获取随机提示
                    let tip = if cfg.tips.is_empty() {
                        "站起来活动一下吧！".to_string()
                    } else {
                        use std::time::SystemTime;
                        let seed = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as usize;
                        let index = seed % cfg.tips.len();
                        cfg.tips[index].clone()
                    };

                    // 更新上次提醒时间
                    let mut last_reminder = last_reminder_time.lock().unwrap();
                    *last_reminder = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

                    // 增加今日提醒次数
                    let mut count = today_reminder_count.lock().unwrap();
                    *count += 1;

                    log::info!("[久坐提醒] 连续工作 {} 分钟，发送提醒: {}",
                        *work_seconds / 60,
                        tip
                    );

                    // 重置连续工作时间
                    *work_seconds = 0;

                    // 调用回调发送提醒
                    callback(tip);
                }
            }

            log::info!("[久坐提醒] 后台监控线程已停止");
        });

        log::info!("[久坐提醒] 服务已启动");
        Ok(())
    }

    /// 停止监控
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        log::info!("[久坐提醒] 服务已停止");
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

impl Default for SedentaryReminderService {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== 数据库操作 ====================

/// 从数据库加载久坐提醒配置
pub fn load_config_from_db(conn: &Connection) -> Result<SedentaryReminderConfig> {
    let config_json: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'sedentary_config'",
            [],
            |row| row.get(0),
        )
        .ok();

    if let Some(json) = config_json {
        let config: SedentaryReminderConfig = serde_json::from_str(&json)?;
        Ok(config)
    } else {
        Ok(SedentaryReminderConfig::default())
    }
}

/// 保存久坐提醒配置到数据库
pub fn save_config_to_db(conn: &Connection, config: &SedentaryReminderConfig) -> Result<()> {
    let config_json = serde_json::to_string(config)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
        params!["sedentary_config", config_json, timestamp],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SedentaryReminderConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.reminder_interval_minutes, 45);
        assert_eq!(config.idle_threshold_seconds, 60);
        assert!(!config.tips.is_empty());
    }

    #[test]
    fn test_service_creation() {
        let service = SedentaryReminderService::new();
        assert!(!service.is_running());

        let status = service.get_status();
        assert!(!status.is_running);
        assert_eq!(status.continuous_work_seconds, 0);
    }

    #[test]
    fn test_reset_timer() {
        let service = SedentaryReminderService::new();

        // 模拟一些工作时间
        {
            let mut work_seconds = service.continuous_work_seconds.lock().unwrap();
            *work_seconds = 1000;
        }

        // 重置
        service.reset_timer();

        let status = service.get_status();
        assert_eq!(status.continuous_work_seconds, 0);
    }

    #[test]
    fn test_config_serialization() {
        let config = SedentaryReminderConfig {
            enabled: true,
            reminder_interval_minutes: 30,
            idle_threshold_seconds: 120,
            tips: vec!["测试提醒".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: SedentaryReminderConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.enabled, config.enabled);
        assert_eq!(parsed.reminder_interval_minutes, config.reminder_interval_minutes);
        assert_eq!(parsed.idle_threshold_seconds, config.idle_threshold_seconds);
    }
}
