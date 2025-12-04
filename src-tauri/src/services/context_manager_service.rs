// Context Manager Service
// 上下文管理服务 - 协调截图采集、变化检测和定时任务

use super::ai_service::{AiService, ChatMessage};
use super::context_store_service::ContextStoreService;
use super::screen_capture_service::{CaptureStatus, ScreenCaptureService};
use crate::models::screen_context::ScreenContext;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration as ChronoDuration, Local, NaiveDateTime};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

/// 合并请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRequest {
    pub items: Vec<ScreenContext>,
}

/// 合并结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeResult {
    pub merge_type: String, // "merged" or "new"
    pub merged_ids: Vec<i64>,
    pub data: MergedData,
}

/// 合并后的数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergedData {
    pub title: String,
    pub summary: String,
    pub keywords: Vec<String>,
    pub importance: i32,
    pub start_time: String,
    pub end_time: String,
}

/// 上下文管理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub capture_interval_secs: u64,
    pub similarity_threshold: f32,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            capture_interval_secs: 10,
            similarity_threshold: 0.95,
        }
    }
}

/// 上下文管理器内部状态
struct ContextManagerState {
    is_running: bool,
    last_hash: Option<String>,
    last_capture_at: Option<String>,
    total_captures_today: u32,
    skipped_count: u32,
    config: ContextConfig,
}

/// 上下文管理器
pub struct ContextManager {
    state: Arc<Mutex<ContextManagerState>>,
    capture_service: Arc<ScreenCaptureService>,
}

impl ContextManager {
    /// 创建新的上下文管理器
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ContextManagerState {
                is_running: false,
                last_hash: None,
                last_capture_at: None,
                total_captures_today: 0,
                skipped_count: 0,
                config: ContextConfig::default(),
            })),
            capture_service: Arc::new(ScreenCaptureService::new()),
        }
    }

    /// 创建带配置的上下文管理器
    pub fn with_config(config: ContextConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(ContextManagerState {
                is_running: false,
                last_hash: None,
                last_capture_at: None,
                total_captures_today: 0,
                skipped_count: 0,
                config,
            })),
            capture_service: Arc::new(ScreenCaptureService::new()),
        }
    }

    /// 启动定时采集任务
    pub async fn start_capture<F>(&self, on_capture: F) -> Result<()>
    where
        F: Fn(ScreenContext) + Send + Sync + 'static,
    {
        let mut state = self.state.lock().await;

        if state.is_running {
            return Err(anyhow::anyhow!("采集任务已在运行中"));
        }

        state.is_running = true;
        let interval_secs = state.config.capture_interval_secs;
        drop(state); // 释放锁

        let state_clone = Arc::clone(&self.state);
        let capture_service = Arc::clone(&self.capture_service);
        let on_capture = Arc::new(on_capture);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            loop {
                ticker.tick().await;

                // 检查是否应该停止
                let should_continue = {
                    let state = state_clone.lock().await;
                    state.is_running
                };

                if !should_continue {
                    break;
                }

                // 执行截图采集
                let result = {
                    let state = state_clone.lock().await;
                    let threshold = state.config.similarity_threshold;
                    let last_hash = state.last_hash.as_deref();

                    capture_service.capture_once(last_hash, threshold)
                };

                match result {
                    Ok(Some(context)) => {
                        // 更新状态
                        let mut state = state_clone.lock().await;
                        state.last_hash = context.screenshot_hash.clone();
                        state.last_capture_at = Some(context.captured_at.clone());
                        state.total_captures_today += 1;
                        drop(state);

                        // 调用回调函数
                        on_capture(context);
                    }
                    Ok(None) => {
                        // 跳过（相似度太高）
                        let mut state = state_clone.lock().await;
                        state.skipped_count += 1;
                    }
                    Err(e) => {
                        eprintln!("截图采集失败: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// 停止采集任务
    pub async fn stop_capture(&self) -> Result<()> {
        let mut state = self.state.lock().await;

        if !state.is_running {
            return Err(anyhow::anyhow!("采集任务未在运行"));
        }

        state.is_running = false;
        Ok(())
    }

    /// 获取当前采集状态
    pub async fn get_status(&self) -> CaptureStatus {
        let state = self.state.lock().await;
        CaptureStatus {
            is_running: state.is_running,
            last_capture_at: state.last_capture_at.clone(),
            total_captures_today: state.total_captures_today,
            skipped_count: state.skipped_count,
        }
    }

    /// 执行一次手动截图
    pub async fn capture_once_manual(&self) -> Result<ScreenContext> {
        let state = self.state.lock().await;
        let threshold = state.config.similarity_threshold;
        let last_hash = state.last_hash.as_deref();

        let result = self
            .capture_service
            .capture_once(last_hash, threshold)?;

        match result {
            Some(context) => Ok(context),
            None => {
                // 如果相似度太高，强制截图（忽略阈值）
                let context = self.capture_service.capture_once(None, 0.0)?;
                context.ok_or_else(|| anyhow::anyhow!("截图失败"))
            }
        }
    }

    /// 更新配置
    pub async fn update_config(&self, config: ContextConfig) -> Result<()> {
        let mut state = self.state.lock().await;

        if state.is_running {
            return Err(anyhow::anyhow!("无法在运行时更新配置，请先停止采集"));
        }

        state.config = config;
        Ok(())
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> ContextConfig {
        let state = self.state.lock().await;
        state.config.clone()
    }

    /// 重置今日统计
    pub async fn reset_daily_stats(&self) {
        let mut state = self.state.lock().await;
        state.total_captures_today = 0;
        state.skipped_count = 0;
    }

    /// 智能合并上下文项
    pub async fn merge_contexts(
        &self,
        items: Vec<ScreenContext>,
        ai_service: &AiService,
    ) -> Result<Vec<MergeResult>> {
        if items.is_empty() {
            return Ok(vec![]);
        }

        // 构建提示词
        let contexts_json = serde_json::to_string_pretty(&items)?;
        let prompt = format!(
            r#"请分析以下屏幕上下文记录，将相似的活动合并在一起。

上下文数据：
{}

请按以下规则进行分析和合并：
1. 如果多个上下文属于同一个应用且活动类型相同，且时间连续（间隔不超过5分钟），应该合并
2. 如果活动内容相似（如都在编辑同一个文件），应该合并
3. 每组合并的活动需要：
   - 生成一个总结标题（title）
   - 生成一个详细摘要（summary）
   - 提取关键词（keywords，3-5个）
   - 评估重要性（importance，1-5分）
   - 记录开始和结束时间

请以JSON数组格式返回结果，每个元素包含：
{{
  "merge_type": "merged" 或 "new",
  "merged_ids": [合并的上下文ID列表],
  "data": {{
    "title": "活动标题",
    "summary": "活动摘要",
    "keywords": ["关键词1", "关键词2"],
    "importance": 3,
    "start_time": "开始时间",
    "end_time": "结束时间"
  }}
}}

只返回JSON数组，不要其他说明文字。"#,
            contexts_json
        );

        // 调用AI服务
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let response = ai_service.chat(messages).await?;

        // 解析AI返回的JSON
        let merge_results: Vec<MergeResult> = serde_json::from_str(&response)
            .map_err(|e| anyhow!("解析AI响应失败: {}", e))?;

        Ok(merge_results)
    }

    /// 基于时间窗口自动合并
    pub async fn auto_merge_by_time_window(
        &self,
        conn: &Connection,
        window_minutes: i32,
        ai_service: &AiService,
    ) -> Result<Vec<MergeResult>> {
        // 计算时间窗口
        let end_time = Local::now();
        let start_time = end_time - ChronoDuration::minutes(window_minutes as i64);

        // 查询时间窗口内的上下文
        let mut stmt = conn.prepare(
            "SELECT id, captured_at, app_name, window_title, activity_type,
                    description, key_content, screenshot_hash, screenshot_path,
                    processing_time_ms
             FROM screen_contexts
             WHERE captured_at >= ? AND captured_at <= ?
             ORDER BY captured_at ASC",
        )?;

        let start_str = start_time.format("%Y-%m-%d %H:%M:%S").to_string();
        let end_str = end_time.format("%Y-%m-%d %H:%M:%S").to_string();

        let contexts = stmt
            .query_map([&start_str, &end_str], |row| {
                Ok(ScreenContext {
                    id: Some(row.get(0)?),
                    captured_at: row.get(1)?,
                    app_name: row.get(2)?,
                    window_title: row.get(3)?,
                    activity_type: row.get(4)?,
                    description: row.get(5)?,
                    key_content: row.get(6)?,
                    screenshot_hash: row.get(7)?,
                    screenshot_path: row.get(8)?,
                    processing_time_ms: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // 调用合并逻辑
        self.merge_contexts(contexts, ai_service).await
    }

    /// 生成日报
    pub async fn generate_daily_report(
        &self,
        conn: &Connection,
        date: &str,
        ai_service: &AiService,
    ) -> Result<String> {
        // 获取指定日期的所有上下文
        let contexts = ContextStoreService::list_by_date(conn, date)?;

        if contexts.is_empty() {
            return Ok(format!("# {} 日报\n\n今日无活动记录。", date));
        }

        // 获取统计信息
        let stats = ContextStoreService::get_day_stats(conn, date)?;

        // 构建日报生成提示词
        let contexts_json = serde_json::to_string_pretty(&contexts)?;
        let stats_json = serde_json::to_string_pretty(&stats)?;

        let prompt = format!(
            r#"请根据以下数据生成一份详细的工作日报：

日期: {}
活动记录: {}
统计信息: {}

请生成包含以下内容的Markdown格式日报：
1. 工作时间概况（开始时间、结束时间、总时长）
2. 主要应用使用情况（列出top 5应用及使用时长）
3. 活动类型分布（编码、浏览、文档等）
4. 关键工作内容总结（提取重要的工作任务和成果）
5. 工作效率分析（专注度、任务切换频率等）
6. 改进建议（如有必要）

请使用清晰的标题层次和列表格式，使日报易于阅读。"#,
            date, contexts_json, stats_json
        );

        // 调用AI服务
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let report = ai_service.chat(messages).await?;
        Ok(report)
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

// 为 ContextManager 实现 Clone，以便在 Tauri State 中使用
impl Clone for ContextManager {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            capture_service: Arc::clone(&self.capture_service),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_manager_lifecycle() {
        let manager = ContextManager::new();

        // 初始状态应该是未运行
        let status = manager.get_status().await;
        assert!(!status.is_running);

        // 测试配置更新
        let new_config = ContextConfig {
            capture_interval_secs: 5,
            similarity_threshold: 0.9,
        };
        manager.update_config(new_config.clone()).await.unwrap();

        let config = manager.get_config().await;
        assert_eq!(config.capture_interval_secs, 5);
        assert!((config.similarity_threshold - 0.9).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_reset_daily_stats() {
        let manager = ContextManager::new();

        // 手动设置一些统计数据
        {
            let mut state = manager.state.lock().await;
            state.total_captures_today = 100;
            state.skipped_count = 50;
        }

        // 重置统计
        manager.reset_daily_stats().await;

        let status = manager.get_status().await;
        assert_eq!(status.total_captures_today, 0);
        assert_eq!(status.skipped_count, 0);
    }
}
