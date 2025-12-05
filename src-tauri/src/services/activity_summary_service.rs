// Activity Summary Service
// 活动总结服务 - 聚合截图记录并生成活动总结

use crate::models::screen_context::ScreenContext;
use crate::services::vlm_service::VlmService;
use crate::services::ai_service::{AiService, ChatMessage};
use crate::prompts::activity_prompts::ACTIVITY_SUMMARY_PROMPT;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Local};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// 活动总结类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityType {
    Coding,
    Browsing,
    Document,
    Meeting,
    Communication,
    Design,
    Other,
}

impl ActivityType {
    pub fn as_str(&self) -> &str {
        match self {
            ActivityType::Coding => "coding",
            ActivityType::Browsing => "browsing",
            ActivityType::Document => "document",
            ActivityType::Meeting => "meeting",
            ActivityType::Communication => "communication",
            ActivityType::Design => "design",
            ActivityType::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "coding" => ActivityType::Coding,
            "browsing" => ActivityType::Browsing,
            "document" => ActivityType::Document,
            "meeting" => ActivityType::Meeting,
            "communication" => ActivityType::Communication,
            "design" => ActivityType::Design,
            _ => ActivityType::Other,
        }
    }
}

/// 活动总结数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivitySummary {
    pub id: Option<i64>,
    pub start_time: String,
    pub end_time: String,
    pub summary_text: String,
    pub activity_type: String,
    pub created_at: Option<String>,
    /// 关联的截图数量
    pub screenshot_count: Option<i32>,
    /// 主要应用
    pub main_apps: Option<String>,
}

/// 活动总结服务
pub struct ActivitySummaryService {
    db_path: String,
}

impl ActivitySummaryService {
    /// 创建新的活动总结服务实例
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }

    /// 获取数据库连接
    fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path).map_err(|e| anyhow!("Failed to open database: {}", e))
    }

    /// 生成活动总结（聚合指定时间段的截图）
    ///
    /// 修改说明：优先基于已有的截图描述（VLM已分析）生成总结，
    /// 而不是重新调用VLM分析第一张图片。这样可以利用所有已分析的截图信息。
    pub async fn generate_summary(
        &self,
        start_time: DateTime<Local>,
        end_time: DateTime<Local>,
        vlm_service: &VlmService,
    ) -> Result<ActivitySummary> {
        let conn = self.get_connection()?;

        // 查询时间段内的截图记录
        let screenshots = self.get_screenshots_in_range(&conn, &start_time, &end_time)?;

        if screenshots.is_empty() {
            log::warn!(
                "时间段 {} - {} 没有截图记录",
                start_time.format("%H:%M:%S"),
                end_time.format("%H:%M:%S")
            );
            return Err(anyhow!("时间段内没有截图记录"));
        }

        log::info!(
            "找到 {} 条截图记录，准备生成活动总结",
            screenshots.len()
        );

        // 统计主要应用
        let main_apps = self.extract_main_apps(&screenshots);

        // 推断主要活动类型
        let activity_type = self.infer_activity_type(&screenshots);

        // 收集已有的截图描述（description 非空的截图）
        let descriptions: Vec<String> = screenshots
            .iter()
            .filter_map(|s| {
                if !s.description.is_empty() {
                    Some(s.description.clone())
                } else {
                    None
                }
            })
            .collect();

        log::info!(
            "找到 {} 条已分析的截图描述",
            descriptions.len()
        );

        // 基于已有描述生成总结
        let summary_text = if !descriptions.is_empty() {
            // 优先尝试使用AI智能总结
            match self.generate_ai_summary(&screenshots, &descriptions, &start_time, &end_time).await {
                Ok(ai_summary) => {
                    log::info!("使用AI生成了智能活动总结");
                    ai_summary
                }
                Err(e) => {
                    log::warn!("AI总结失败，回退到简单拼接: {}", e);
                    self.merge_descriptions(&descriptions)
                }
            }
        } else {
            // 如果没有已有描述，尝试调用VLM分析（作为回退方案）
            log::info!("没有已分析的描述，尝试使用VLM分析第一张截图");

            if let Some(first_screenshot) = screenshots.first() {
                if let Some(ref screenshot_path) = first_screenshot.screenshot_path {
                    let prompt = self.build_summary_prompt(&screenshots, &activity_type);

                    // 读取截图文件并转换为base64
                    match self.load_screenshot_as_base64(screenshot_path) {
                        Ok(image_base64) => {
                            match vlm_service.analyze_image(&image_base64, &prompt).await {
                                Ok(summary) => summary,
                                Err(e) => {
                                    log::error!("VLM分析失败: {}", e);
                                    // 回退到基于规则的总结
                                    self.generate_rule_based_summary(&screenshots, &activity_type)
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("无法加载截图文件: {}", e);
                            // 回退到基于规则的总结
                            self.generate_rule_based_summary(&screenshots, &activity_type)
                        }
                    }
                } else {
                    // 没有截图路径，使用基于规则的总结
                    self.generate_rule_based_summary(&screenshots, &activity_type)
                }
            } else {
                return Err(anyhow!("没有可用的截图"));
            }
        };

        // 创建活动总结
        let summary = ActivitySummary {
            id: None,
            start_time: start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            end_time: end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            summary_text,
            activity_type: activity_type.as_str().to_string(),
            created_at: Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            screenshot_count: Some(screenshots.len() as i32),
            main_apps: Some(main_apps),
        };

        // 保存到数据库
        self.save_summary(&conn, &summary)?;

        log::info!("活动总结已生成并保存");
        Ok(summary)
    }

    /// 查询指定时间范围内的截图记录
    fn get_screenshots_in_range(
        &self,
        conn: &Connection,
        start_time: &DateTime<Local>,
        end_time: &DateTime<Local>,
    ) -> Result<Vec<ScreenContext>> {
        let mut stmt = conn.prepare(
            "SELECT id, captured_at, app_name, window_title, activity_type,
                    description, key_content, screenshot_hash, screenshot_path,
                    processing_time_ms
             FROM screen_contexts
             WHERE captured_at >= ? AND captured_at < ?
             ORDER BY captured_at ASC",
        )?;

        let start_str = start_time.format("%Y-%m-%d %H:%M:%S").to_string();
        let end_str = end_time.format("%Y-%m-%d %H:%M:%S").to_string();

        let screenshots = stmt
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

        Ok(screenshots)
    }

    /// 提取主要应用列表
    fn extract_main_apps(&self, screenshots: &[ScreenContext]) -> String {
        let mut app_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for screenshot in screenshots {
            if let Some(ref app_name) = screenshot.app_name {
                *app_counts.entry(app_name.clone()).or_insert(0) += 1;
            }
        }

        // 按使用次数排序，取前3个
        let mut apps: Vec<_> = app_counts.into_iter().collect();
        apps.sort_by(|a, b| b.1.cmp(&a.1));

        apps.iter()
            .take(3)
            .map(|(app, _)| app.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// 推断主要活动类型
    fn infer_activity_type(&self, screenshots: &[ScreenContext]) -> ActivityType {
        let mut type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for screenshot in screenshots {
            *type_counts
                .entry(screenshot.activity_type.clone())
                .or_insert(0) += 1;
        }

        // 找到出现次数最多的活动类型
        let most_common_type = type_counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(activity_type, _)| activity_type)
            .unwrap_or_else(|| "other".to_string());

        ActivityType::from_str(&most_common_type)
    }

    /// 构建VLM分析提示词
    fn build_summary_prompt(&self, screenshots: &[ScreenContext], activity_type: &ActivityType) -> String {
        let time_range = format!(
            "{} - {}",
            screenshots.first().map(|s| s.captured_at.clone()).unwrap_or_default(),
            screenshots.last().map(|s| s.captured_at.clone()).unwrap_or_default()
        );

        let apps = self.extract_main_apps(screenshots);

        format!(
            "请分析这张截图，这是用户在 {} 时段的工作活动记录。\n\
            主要使用的应用: {}\n\
            活动类型: {}\n\
            总共有 {} 次屏幕变化。\n\n\
            请用1-2句话简洁总结用户在这段时间的主要工作内容，突出关键任务和成果。",
            time_range,
            apps,
            activity_type.as_str(),
            screenshots.len()
        )
    }

    /// 基于规则生成总结（VLM不可用时的回退方案）
    fn generate_rule_based_summary(&self, screenshots: &[ScreenContext], activity_type: &ActivityType) -> String {
        let main_apps = self.extract_main_apps(screenshots);
        let count = screenshots.len();

        let activity_desc = match activity_type {
            ActivityType::Coding => "进行代码开发",
            ActivityType::Browsing => "浏览网页",
            ActivityType::Document => "编辑文档",
            ActivityType::Meeting => "参加会议",
            ActivityType::Communication => "进行沟通交流",
            ActivityType::Design => "进行设计工作",
            ActivityType::Other => "进行工作",
        };

        format!(
            "在这段时间内，主要使用 {} {}，共产生 {} 次屏幕变化。",
            main_apps, activity_desc, count
        )
    }

    /// 使用AI生成智能活动总结
    ///
    /// 调用AiService使用ACTIVITY_SUMMARY_PROMPT分析截图记录，生成智能总结
    async fn generate_ai_summary(
        &self,
        screenshots: &[ScreenContext],
        _descriptions: &[String],
        start_time: &DateTime<Local>,
        end_time: &DateTime<Local>,
    ) -> Result<String> {
        // 从数据库加载AI配置
        let conn = self.get_connection()?;
        let ai_config = AiService::load_config(&conn)?;

        // 创建AI服务实例
        let ai_service = AiService::new(ai_config);

        // 检查AI服务是否已配置
        if !ai_service.is_configured() {
            return Err(anyhow!("AI服务未配置"));
        }

        // 构建输入数据
        let input_data = serde_json::json!({
            "timeRange": {
                "start": start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                "end": end_time.format("%Y-%m-%d %H:%M:%S").to_string()
            },
            "screenshots": screenshots.iter().map(|s| {
                serde_json::json!({
                    "capturedAt": s.captured_at,
                    "appName": s.app_name,
                    "windowTitle": s.window_title,
                    "activityType": s.activity_type,
                    "description": s.description
                })
            }).collect::<Vec<_>>()
        });

        // 构建消息
        let messages = vec![
            ChatMessage::system(ACTIVITY_SUMMARY_PROMPT.to_string()),
            ChatMessage::user(input_data.to_string()),
        ];

        // 调用AI
        let response = ai_service.chat(messages).await?;

        // 解析JSON响应，提取有用字段
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
            // 优先提取description
            if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
                // 如果有title，组合使用
                if let Some(title) = json.get("title").and_then(|v| v.as_str()) {
                    return Ok(format!("**{}**\n{}", title, desc));
                }
                return Ok(desc.to_string());
            }
            // 尝试提取title
            if let Some(title) = json.get("title").and_then(|v| v.as_str()) {
                return Ok(title.to_string());
            }
        }

        // 尝试清理被```json```包裹的响应
        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(cleaned) {
            if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
                return Ok(desc.to_string());
            }
        }

        // 如果响应看起来像JSON，不要直接显示原始JSON
        if response.trim().starts_with('{') || response.trim().starts_with('[') {
            Ok("活动总结生成中...".to_string())
        } else {
            // 如果不是JSON，可能是普通文本，直接返回
            Ok(response)
        }
    }

    /// 合并多个截图描述生成时间段总结
    ///
    /// 该方法从多个已分析的截图描述中提取关键信息并生成一个连贯的总结。
    /// 策略：
    /// 1. 去重相似的描述，避免重复
    /// 2. 按时间顺序或重要性合并描述
    /// 3. 生成简洁的总结文本
    fn merge_descriptions(&self, descriptions: &[String]) -> String {
        use std::collections::HashSet;

        if descriptions.is_empty() {
            return "在这段时间内进行了工作活动。".to_string();
        }

        // 去除完全重复的描述
        let mut unique_descriptions: Vec<String> = descriptions
            .iter()
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // 按长度排序，将更详细的描述放在前面
        unique_descriptions.sort_by(|a, b| b.len().cmp(&a.len()));

        // 限制描述数量，避免总结过长
        let max_descriptions = 5;
        let selected_descriptions: Vec<String> = unique_descriptions
            .into_iter()
            .take(max_descriptions)
            .collect();

        // 如果只有一个描述，直接返回
        if selected_descriptions.len() == 1 {
            return selected_descriptions[0].clone();
        }

        // 合并多个描述，使用分号分隔以保持清晰度
        let merged = selected_descriptions.join("；");

        // 如果合并后的文本过长，截断并添加省略号
        // 使用字符边界安全截断，避免在多字节字符中间切片
        // AI智能总结返回的内容通常是合理长度，因此增大限制到2000字符
        const MAX_CHARS: usize = 2000;  // 按字符数限制，不是字节数
        let char_count = merged.chars().count();
        if char_count > MAX_CHARS {
            let truncated: String = merged.chars().take(MAX_CHARS).collect();
            format!("{}...", truncated)
        } else {
            merged
        }
    }

    /// 加载截图文件并转换为base64
    fn load_screenshot_as_base64(&self, screenshot_path: &str) -> Result<String> {
        use base64::{engine::general_purpose, Engine as _};
        use std::fs;

        let image_data = fs::read(screenshot_path)
            .map_err(|e| anyhow!("读取截图文件失败: {}", e))?;

        Ok(general_purpose::STANDARD.encode(&image_data))
    }

    /// 保存活动总结到数据库
    fn save_summary(&self, conn: &Connection, summary: &ActivitySummary) -> Result<i64> {
        conn.execute(
            "INSERT INTO activity_summaries (start_time, end_time, summary_text, activity_type, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                summary.start_time,
                summary.end_time,
                summary.summary_text,
                summary.activity_type,
                summary.created_at,
            ],
        )?;

        let id = conn.last_insert_rowid();
        Ok(id)
    }

    /// 获取最近的活动总结列表
    pub fn get_recent_summaries(&self, limit: i32) -> Result<Vec<ActivitySummary>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, start_time, end_time, summary_text, activity_type, created_at
             FROM activity_summaries
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;

        let summaries = stmt
            .query_map([limit], |row| {
                Ok(ActivitySummary {
                    id: Some(row.get(0)?),
                    start_time: row.get(1)?,
                    end_time: row.get(2)?,
                    summary_text: row.get(3)?,
                    activity_type: row.get(4)?,
                    created_at: Some(row.get(5)?),
                    screenshot_count: None,
                    main_apps: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(summaries)
    }

    /// 获取指定日期的活动总结
    pub fn get_summaries_by_date(&self, date: &str) -> Result<Vec<ActivitySummary>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, start_time, end_time, summary_text, activity_type, created_at
             FROM activity_summaries
             WHERE DATE(start_time) = ?1
             ORDER BY start_time ASC",
        )?;

        let summaries = stmt
            .query_map([date], |row| {
                Ok(ActivitySummary {
                    id: Some(row.get(0)?),
                    start_time: row.get(1)?,
                    end_time: row.get(2)?,
                    summary_text: row.get(3)?,
                    activity_type: row.get(4)?,
                    created_at: Some(row.get(5)?),
                    screenshot_count: None,
                    main_apps: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(summaries)
    }

    /// 删除指定日期之前的活动总结
    pub fn cleanup_old_summaries(&self, days: i64) -> Result<usize> {
        let conn = self.get_connection()?;

        let cutoff_date = Local::now() - Duration::days(days);
        let cutoff_str = cutoff_date.format("%Y-%m-%d").to_string();

        let deleted = conn.execute(
            "DELETE FROM activity_summaries WHERE DATE(start_time) < ?1",
            [&cutoff_str],
        )?;

        log::info!("清理了 {} 条超过 {} 天的活动总结", deleted, days);
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_type_conversion() {
        assert_eq!(ActivityType::from_str("coding").as_str(), "coding");
        assert_eq!(ActivityType::from_str("browsing").as_str(), "browsing");
        assert_eq!(ActivityType::from_str("unknown").as_str(), "other");
    }

    #[test]
    fn test_extract_main_apps() {
        let service = ActivitySummaryService::new("test.db".to_string());

        let screenshots = vec![
            ScreenContext {
                id: Some(1),
                captured_at: "2025-12-04 10:00:00".to_string(),
                app_name: Some("VS Code".to_string()),
                window_title: None,
                activity_type: "coding".to_string(),
                description: "".to_string(),
                key_content: None,
                screenshot_hash: None,
                screenshot_path: None,
                processing_time_ms: None,
            },
            ScreenContext {
                id: Some(2),
                captured_at: "2025-12-04 10:05:00".to_string(),
                app_name: Some("VS Code".to_string()),
                window_title: None,
                activity_type: "coding".to_string(),
                description: "".to_string(),
                key_content: None,
                screenshot_hash: None,
                screenshot_path: None,
                processing_time_ms: None,
            },
            ScreenContext {
                id: Some(3),
                captured_at: "2025-12-04 10:10:00".to_string(),
                app_name: Some("Chrome".to_string()),
                window_title: None,
                activity_type: "browsing".to_string(),
                description: "".to_string(),
                key_content: None,
                screenshot_hash: None,
                screenshot_path: None,
                processing_time_ms: None,
            },
        ];

        let main_apps = service.extract_main_apps(&screenshots);
        assert!(main_apps.contains("VS Code"));
    }
}
