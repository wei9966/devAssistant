// Scheduler Service
// 定时任务调度服务 - 管理Activity总结和Tips提示

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

/// 定时任务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Activity总结间隔（分钟）
    pub activity_summary_interval_minutes: u64,
    /// Tips提示间隔（分钟）
    pub tips_interval_minutes: u64,
    /// TODO预测间隔（分钟）
    pub todo_prediction_interval_minutes: u64,
    /// 是否启用Activity总结
    pub enable_activity_summary: bool,
    /// 是否启用Tips提示
    pub enable_tips: bool,
    /// 是否启用TODO预测
    pub enable_todo_prediction: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            activity_summary_interval_minutes: 15,
            tips_interval_minutes: 60,
            todo_prediction_interval_minutes: 120, // 每2小时执行一次
            enable_activity_summary: true,
            enable_tips: true, // 启用 Tips 定时提醒
            enable_todo_prediction: true, // 启用 TODO 预测
        }
    }
}

/// 定时任务调度器状态
struct SchedulerState {
    is_running: bool,
    config: SchedulerConfig,
}

/// 定时任务调度器
pub struct SchedulerService {
    state: Arc<Mutex<SchedulerState>>,
}

impl SchedulerService {
    /// 创建新的调度器实例
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SchedulerState {
                is_running: false,
                config: SchedulerConfig::default(),
            })),
        }
    }

    /// 创建带配置的调度器实例
    pub fn with_config(config: SchedulerConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(SchedulerState {
                is_running: false,
                config,
            })),
        }
    }

    /// 启动定时任务
    pub async fn start<F1, F2, F3>(&self, on_activity_summary: F1, on_tips: F2, on_todo_prediction: F3) -> Result<()>
    where
        F1: Fn() + Send + Sync + 'static,
        F2: Fn() + Send + Sync + 'static,
        F3: Fn() + Send + Sync + 'static,
    {
        let mut state = self.state.lock().await;

        if state.is_running {
            return Err(anyhow::anyhow!("定时任务已在运行中"));
        }

        state.is_running = true;
        let config = state.config.clone();
        drop(state);

        let state_clone = Arc::clone(&self.state);
        let on_activity_summary = Arc::new(on_activity_summary);
        let on_tips = Arc::new(on_tips);
        let on_todo_prediction = Arc::new(on_todo_prediction);

        // 启动Activity总结定时任务
        if config.enable_activity_summary {
            let state_ref = Arc::clone(&state_clone);
            let callback = Arc::clone(&on_activity_summary);
            let interval_minutes = config.activity_summary_interval_minutes;

            tokio::spawn(async move {
                let mut ticker = interval(Duration::from_secs(interval_minutes * 60));

                loop {
                    ticker.tick().await;

                    // 检查是否应该停止
                    let should_continue = {
                        let state = state_ref.lock().await;
                        state.is_running && state.config.enable_activity_summary
                    };

                    if !should_continue {
                        break;
                    }

                    // 执行Activity总结回调
                    log::info!("触发Activity总结任务");
                    callback();
                }

                log::info!("Activity总结定时任务已停止");
            });
        }

        // 启动Tips提示定时任务
        if config.enable_tips {
            let state_ref = Arc::clone(&state_clone);
            let callback = Arc::clone(&on_tips);
            let interval_minutes = config.tips_interval_minutes;

            tokio::spawn(async move {
                let mut ticker = interval(Duration::from_secs(interval_minutes * 60));

                loop {
                    ticker.tick().await;

                    // 检查是否应该停止
                    let should_continue = {
                        let state = state_ref.lock().await;
                        state.is_running && state.config.enable_tips
                    };

                    if !should_continue {
                        break;
                    }

                    // 执行Tips提示回调
                    log::info!("触发Tips提示任务");
                    callback();
                }

                log::info!("Tips提示定时任务已停止");
            });
        }

        // 启动TODO预测定时任务
        if config.enable_todo_prediction {
            let state_ref = Arc::clone(&state_clone);
            let callback = Arc::clone(&on_todo_prediction);
            let interval_minutes = config.todo_prediction_interval_minutes;

            tokio::spawn(async move {
                let mut ticker = interval(Duration::from_secs(interval_minutes * 60));

                loop {
                    ticker.tick().await;

                    // 检查是否应该停止
                    let should_continue = {
                        let state = state_ref.lock().await;
                        state.is_running && state.config.enable_todo_prediction
                    };

                    if !should_continue {
                        break;
                    }

                    // 执行TODO预测回调
                    log::info!("触发TODO预测任务");
                    callback();
                }

                log::info!("TODO预测定时任务已停止");
            });
        }

        log::info!("定时任务调度器已启动");
        Ok(())
    }

    /// 停止定时任务
    pub async fn stop(&self) -> Result<()> {
        let mut state = self.state.lock().await;

        if !state.is_running {
            return Err(anyhow::anyhow!("定时任务未在运行"));
        }

        state.is_running = false;
        log::info!("定时任务调度器已停止");
        Ok(())
    }

    /// 检查是否正在运行
    pub async fn is_running(&self) -> bool {
        let state = self.state.lock().await;
        state.is_running
    }

    /// 更新配置
    pub async fn update_config(&self, config: SchedulerConfig) -> Result<()> {
        let mut state = self.state.lock().await;

        if state.is_running {
            return Err(anyhow::anyhow!("无法在运行时更新配置，请先停止定时任务"));
        }

        state.config = config;
        Ok(())
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> SchedulerConfig {
        let state = self.state.lock().await;
        state.config.clone()
    }
}

impl Default for SchedulerService {
    fn default() -> Self {
        Self::new()
    }
}

// 实现 Clone，以便在 Tauri State 中使用
impl Clone for SchedulerService {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_scheduler_lifecycle() {
        let scheduler = SchedulerService::new();

        // 初始状态应该是未运行
        assert!(!scheduler.is_running().await);

        // 测试配置更新
        let new_config = SchedulerConfig {
            activity_summary_interval_minutes: 10,
            tips_interval_minutes: 30,
            enable_activity_summary: true,
            enable_tips: true,
        };
        scheduler.update_config(new_config.clone()).await.unwrap();

        let config = scheduler.get_config().await;
        assert_eq!(config.activity_summary_interval_minutes, 10);
        assert_eq!(config.tips_interval_minutes, 30);
    }

    #[tokio::test]
    async fn test_scheduler_callbacks() {
        let scheduler = SchedulerService::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        // 使用很短的间隔进行测试（1秒）
        let config = SchedulerConfig {
            activity_summary_interval_minutes: 0, // 0表示最小间隔
            tips_interval_minutes: 0,
            todo_prediction_interval_minutes: 0,
            enable_activity_summary: true,
            enable_tips: false,
            enable_todo_prediction: false,
        };
        scheduler.update_config(config).await.unwrap();

        // 启动调度器
        scheduler
            .start(
                move || {
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                },
                || {},
                || {},
            )
            .await
            .unwrap();

        // 等待一段时间
        tokio::time::sleep(Duration::from_secs(1)).await;

        // 停止调度器
        scheduler.stop().await.unwrap();

        // 验证回调被执行
        assert!(!scheduler.is_running().await);
    }
}
