// Activity Summary Service
// 活动总结服务 - 聚合截图记录并生成活动总结

use crate::models::screen_context::ScreenContext;
use crate::services::vlm_service::VlmService;
use crate::services::ai_service::{AiService, ChatMessage};
use crate::services::prompt_db_service::PromptDbService;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Local};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

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

/// 小时总结数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlySummary {
    pub id: Option<i64>,
    pub date: String,
    pub hour: i32,
    pub summary_text: String,
    pub activity_type: String,
    pub source_summary_ids: Option<String>,
    pub screenshot_count: Option<i32>,
    pub created_at: Option<String>,
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
        // 查询时间段内的截图记录（连接不跨 await）
        let screenshots = {
            let conn = self.get_connection()?;
            self.get_screenshots_in_range(&conn, &start_time, &end_time)?
        };

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
                    let prompt = self.build_summary_prompt(&screenshots, &activity_type).await?;

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

        // 保存到数据库（重新打开连接，避免跨 await 持有连接）
        {
            let conn = self.get_connection()?;
            self.save_summary(&conn, &summary)?;
        }

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

    /// 构建VLM分析提示词 - 使用数据库中的提示词
    async fn build_summary_prompt(&self, screenshots: &[ScreenContext], activity_type: &ActivityType) -> Result<String> {
        let time_range = format!(
            "{} - {}",
            screenshots.first().map(|s| s.captured_at.clone()).unwrap_or_default(),
            screenshots.last().map(|s| s.captured_at.clone()).unwrap_or_default()
        );

        let apps = self.extract_main_apps(screenshots);

        // 准备变量
        let mut vars = HashMap::new();
        vars.insert("time_range".to_string(), time_range);
        vars.insert("main_apps".to_string(), apps);
        vars.insert("activity_type".to_string(), activity_type.as_str().to_string());
        vars.insert("screenshot_count".to_string(), screenshots.len().to_string());

        // 优先从数据库获取提示词
        match PromptDbService::render_prompt_cached("screenshot_single", &vars) {
            Ok(rendered) => {
                log::info!("[活动总结] 使用数据库提示词");
                Ok(rendered.user)
            }
            Err(e) => {
                log::warn!("[活动总结] 加载数据库提示词失败，使用默认提示词: {}", e);
                // 回退到默认提示词
                Ok(format!(
                    "请分析这张截图，这是用户在 {} 时段的工作活动记录。\n\
                    主要使用的应用: {}\n\
                    活动类型: {}\n\
                    总共有 {} 次屏幕变化。\n\n\
                    请用1-2句话简洁总结用户在这段时间的主要工作内容，突出关键任务和成果。",
                    vars["time_range"],
                    vars["main_apps"],
                    vars["activity_type"],
                    vars["screenshot_count"]
                ))
            }
        }
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
    /// 优先从数据库加载 activity_summary 提示词，如果失败则回退到硬编码提示词
    async fn generate_ai_summary(
        &self,
        screenshots: &[ScreenContext],
        _descriptions: &[String],
        start_time: &DateTime<Local>,
        end_time: &DateTime<Local>,
    ) -> Result<String> {
        let api_start_time = Instant::now();

        // 从数据库加载AI配置（不跨 await 持有连接）
        let ai_config = {
            let conn = self.get_connection()?;
            AiService::load_config(&conn)?
        };

        // 创建AI服务实例
        let ai_service = AiService::new(ai_config.clone());

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

        // 优先从数据库加载提示词，如果失败则使用硬编码提示词
        let mut vars = HashMap::new();
        vars.insert("input_data".to_string(), input_data.to_string());

        let messages = match PromptDbService::render_prompt_cached("activity_summary", &vars) {
            Ok(rendered) => {
                log::info!("[活动总结] 使用数据库提示词");
                if let Some(system) = rendered.system {
                    vec![
                        ChatMessage::system(system),
                        ChatMessage::user(rendered.user),
                    ]
                } else {
                    vec![ChatMessage::user(rendered.user)]
                }
            }
            Err(e) => {
                log::warn!("[活动总结] 加载数据库提示词失败，使用简单提示词: {}", e);
                // 使用简单的硬编码 fallback 提示词
                let fallback_system = "你是一个专业的活动分析助手。请分析用户活动并生成JSON格式的总结。";
                let fallback_user = format!(
                    "请分析以下活动记录并生成JSON格式总结：\n{}\n\n返回格式：{{\"title\":\"标题\",\"description\":\"描述\",\"keywords\":[],\"importance\":1-5}}",
                    input_data
                );
                vec![
                    ChatMessage::system(fallback_system.to_string()),
                    ChatMessage::user(fallback_user),
                ]
            }
        };

        let prompt_summary = format!(
            "[活动总结] {} - {}, {} 条截图",
            start_time.format("%H:%M"),
            end_time.format("%H:%M"),
            screenshots.len()
        );

        // 调用AI
        let response_result = ai_service.chat(messages).await;
        let duration_ms = api_start_time.elapsed().as_millis() as u64;

        // 记录日志（重新打开连接，不跨 await）
        let provider_str = format!("{:?}", ai_config.provider);
        let model_str = ai_config.model.clone().unwrap_or_default();
        match &response_result {
            Ok(resp) => {
                let response_summary = if resp.chars().count() > 300 {
                    let truncated: String = resp.chars().take(300).collect();
                    format!("{}...", truncated)
                } else {
                    resp.clone()
                };
                if let Ok(conn) = self.get_connection() {
                    let _ = AiService::save_log(
                        &conn,
                        "activity_summary",
                        "generate_summary",
                        &provider_str,
                        Some(model_str.as_str()),
                        &prompt_summary,
                        Some(response_summary.as_str()),
                        None,
                        Some(duration_ms as i64),
                        "success",
                        None,
                    );
                }
            }
            Err(e) => {
                let err_msg = e.to_string();
                if let Ok(conn) = self.get_connection() {
                    let _ = AiService::save_log(
                        &conn,
                        "activity_summary",
                        "generate_summary",
                        &provider_str,
                        Some(model_str.as_str()),
                        &prompt_summary,
                        None,
                        None,
                        Some(duration_ms as i64),
                        "error",
                        Some(err_msg.as_str()),
                    );
                }
            }
        }

        let response = response_result?;

        // 解析JSON响应，提取有用字段
        // 优化：包含 potentialTodos 信息，用于后续 TODO 预测
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
            return Ok(self.format_summary_from_json(&json));
        }

        // 尝试清理被```json```包裹的响应
        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(cleaned) {
            return Ok(self.format_summary_from_json(&json));
        }

        // 如果响应看起来像JSON，不要直接显示原始JSON
        if response.trim().starts_with('{') || response.trim().starts_with('[') {
            Ok("活动总结生成中...".to_string())
        } else {
            // 如果不是JSON，可能是普通文本，直接返回
            Ok(response)
        }
    }

    /// 从 JSON 响应中格式化活动总结
    ///
    /// 将 AI 返回的结构化 JSON 转换为易于阅读和分析的文本格式
    /// 支持多种 JSON 格式：
    /// 1. 标准格式：title, description, keyInsights, potentialTodos
    /// 2. 备用格式：activity, details, context
    fn format_summary_from_json(&self, json: &serde_json::Value) -> String {
        let mut parts = Vec::new();

        // 1. 标题和描述 - 支持多种字段名
        if let Some(title) = json.get("title").and_then(|v| v.as_str()) {
            parts.push(format!("**{}**", title));
        }

        if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
            parts.push(desc.to_string());
        }

        // 2. 备用格式：activity + details（兼容旧格式的 AI 响应）
        if parts.is_empty() {
            if let Some(activity) = json.get("activity").and_then(|v| v.as_str()) {
                parts.push(format!("**{}**", activity));
            }

            // 处理 details 数组
            if let Some(details) = json.get("details").and_then(|v| v.as_array()) {
                let detail_texts: Vec<String> = details
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| format!("• {}", s))
                    .collect();

                if !detail_texts.is_empty() {
                    parts.push(detail_texts.join("\n"));
                }
            }

            // 处理 context 对象
            if let Some(context) = json.get("context").and_then(|v| v.as_object()) {
                let mut context_parts = Vec::new();

                if let Some(app) = context.get("application").and_then(|v| v.as_str()) {
                    context_parts.push(format!("应用: {}", app));
                }
                if let Some(task) = context.get("primary_task").and_then(|v| v.as_str()) {
                    context_parts.push(format!("主要任务: {}", task));
                }
                if let Some(env) = context.get("environment").and_then(|v| v.as_str()) {
                    context_parts.push(format!("环境: {}", env));
                }

                // 处理 secondary_tasks 数组
                if let Some(secondary) = context.get("secondary_tasks").and_then(|v| v.as_array()) {
                    let tasks: Vec<&str> = secondary
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect();
                    if !tasks.is_empty() {
                        context_parts.push(format!("其他任务: {}", tasks.join(", ")));
                    }
                }

                if !context_parts.is_empty() {
                    parts.push(format!("\n📋 上下文：\n{}", context_parts.join("\n")));
                }
            }
        }

        // 3. 关键洞察（可选）
        if let Some(insights) = json.get("keyInsights").and_then(|v| v.as_array()) {
            let insight_texts: Vec<String> = insights
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| format!("• {}", s))
                .collect();

            if !insight_texts.is_empty() {
                parts.push(format!("\n📌 关键洞察：\n{}", insight_texts.join("\n")));
            }
        }

        // 4. 潜在待办任务（用于 TODO 预测）
        if let Some(todos) = json.get("potentialTodos").and_then(|v| v.as_array()) {
            if !todos.is_empty() {
                let todo_texts: Vec<String> = todos
                    .iter()
                    .filter_map(|todo| {
                        let task = todo.get("task").and_then(|v| v.as_str())?;
                        let priority = todo.get("priority").and_then(|v| v.as_str()).unwrap_or("medium");
                        let reason = todo.get("reason").and_then(|v| v.as_str()).unwrap_or("");

                        let priority_icon = match priority {
                            "high" => "🔴",
                            "medium" => "🟡",
                            "low" => "🟢",
                            _ => "⚪",
                        };

                        if reason.is_empty() {
                            Some(format!("{} {}", priority_icon, task))
                        } else {
                            Some(format!("{} {} ({})", priority_icon, task, reason))
                        }
                    })
                    .collect();

                if !todo_texts.is_empty() {
                    parts.push(format!("\n📋 潜在待办：\n{}", todo_texts.join("\n")));
                }
            }
        }

        if parts.is_empty() {
            // 如果所有已知字段都没有匹配到，尝试提取任意字符串字段作为描述
            let mut fallback_parts = Vec::new();
            if let Some(obj) = json.as_object() {
                for (key, value) in obj {
                    if let Some(s) = value.as_str() {
                        if s.len() > 10 && s.len() < 500 {
                            fallback_parts.push(format!("**{}**: {}", key, s));
                        }
                    }
                }
            }
            if !fallback_parts.is_empty() {
                fallback_parts.join("\n")
            } else {
                "活动总结生成中...".to_string()
            }
        } else {
            parts.join("\n")
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

    /// 获取指定日期和小时的所有15分钟总结
    pub fn get_summaries_for_hour(&self, date: &str, hour: i32) -> Result<Vec<ActivitySummary>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, start_time, end_time, summary_text, activity_type, created_at, screenshot_count, main_apps
             FROM activity_summaries
             WHERE DATE(start_time) = ?1 AND CAST(strftime('%H', start_time) AS INTEGER) = ?2
             ORDER BY start_time ASC",
        )?;

        let summaries = stmt
            .query_map(rusqlite::params![date, hour], |row| {
                Ok(ActivitySummary {
                    id: Some(row.get(0)?),
                    start_time: row.get(1)?,
                    end_time: row.get(2)?,
                    summary_text: row.get(3)?,
                    activity_type: row.get(4)?,
                    created_at: Some(row.get(5)?),
                    screenshot_count: row.get(6)?,
                    main_apps: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(summaries)
    }

    /// 从 hourly_summaries 表查询已存在的小时总结
    pub fn get_hourly_summary(&self, date: &str, hour: i32) -> Result<Option<HourlySummary>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, date, hour, summary_text, activity_type, source_summary_ids, screenshot_count, created_at
             FROM hourly_summaries
             WHERE date = ?1 AND hour = ?2",
        )?;

        let result = stmt.query_row(rusqlite::params![date, hour], |row| {
            Ok(HourlySummary {
                id: Some(row.get(0)?),
                date: row.get(1)?,
                hour: row.get(2)?,
                summary_text: row.get(3)?,
                activity_type: row.get(4)?,
                source_summary_ids: row.get(5)?,
                screenshot_count: row.get(6)?,
                created_at: Some(row.get(7)?),
            })
        });

        match result {
            Ok(summary) => Ok(Some(summary)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow!("查询小时总结失败: {}", e)),
        }
    }

    /// 保存小时总结到 hourly_summaries 表
    pub fn save_hourly_summary(&self, summary: &HourlySummary) -> Result<i64> {
        let conn = self.get_connection()?;

        conn.execute(
            "INSERT OR REPLACE INTO hourly_summaries
             (date, hour, summary_text, activity_type, source_summary_ids, screenshot_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                summary.date,
                summary.hour,
                summary.summary_text,
                summary.activity_type,
                summary.source_summary_ids,
                summary.screenshot_count,
                summary.created_at,
            ],
        )?;

        let id = conn.last_insert_rowid();
        Ok(id)
    }

    /// 生成小时总结（拼接多个15分钟总结并调用AI）
    pub async fn generate_hourly_summary(&self, date: &str, hour: i32) -> Result<HourlySummary> {
        let api_start_time = Instant::now();

        // 获取该小时的所有15分钟总结
        let summaries = self.get_summaries_for_hour(date, hour)?;

        if summaries.is_empty() {
            return Err(anyhow!("指定时间段 {} {}:00 没有15分钟总结", date, hour));
        }

        log::info!(
            "找到 {} 条15分钟总结，准备生成小时总结: {} {}:00",
            summaries.len(),
            date,
            hour
        );

        // 从数据库加载AI配置（不跨 await 持有连接）
        let ai_config = {
            let conn = self.get_connection()?;
            AiService::load_config(&conn)?
        };

        // 创建AI服务实例
        let ai_service = AiService::new(ai_config.clone());

        // 检查AI服务是否已配置
        if !ai_service.is_configured() {
            return Err(anyhow!("AI服务未配置"));
        }

        // 拼接总结内容
        let summaries_text: Vec<String> = summaries
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                format!(
                    "{}. 时段: {} - {}\n   总结: {}\n   活动类型: {}",
                    idx + 1,
                    s.start_time,
                    s.end_time,
                    s.summary_text,
                    s.activity_type
                )
            })
            .collect();

        let summaries_content = summaries_text.join("\n\n");

        // 统计截图总数
        let screenshot_count: i32 = summaries
            .iter()
            .filter_map(|s| s.screenshot_count)
            .sum();

        // 收集来源总结ID
        let source_ids: Vec<String> = summaries
            .iter()
            .filter_map(|s| s.id.map(|id| id.to_string()))
            .collect();
        let source_summary_ids = source_ids.join(",");

        // 统计主要活动类型
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for summary in &summaries {
            *type_counts
                .entry(summary.activity_type.clone())
                .or_insert(0) += 1;
        }
        let main_activity_type = type_counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(activity_type, _)| activity_type)
            .unwrap_or_else(|| "other".to_string());

        // 准备提示词变量
        let mut vars = HashMap::new();
        vars.insert("date".to_string(), date.to_string());
        vars.insert("hour".to_string(), hour.to_string());
        vars.insert("summaries".to_string(), summaries_content);
        vars.insert("screenshot_count".to_string(), screenshot_count.to_string());
        vars.insert("summary_count".to_string(), summaries.len().to_string());

        // 使用 PromptDbService 渲染提示词
        let rendered = PromptDbService::render_prompt_cached("hourly_summary", &vars)
            .map_err(|e| anyhow!("获取提示词失败: {}", e))?;

        // 构建消息
        let messages = if let Some(system) = rendered.system {
            vec![
                ChatMessage::system(system),
                ChatMessage::user(rendered.user),
            ]
        } else {
            vec![ChatMessage::user(rendered.user)]
        };

        let prompt_summary = format!(
            "[小时总结] {} {}:00, {} 条15分钟总结, {} 张截图",
            date, hour, summaries.len(), screenshot_count
        );

        // 调用AI
        let response_result = ai_service.chat(messages).await;
        let duration_ms = api_start_time.elapsed().as_millis() as u64;

        // 记录AI日志（重新打开连接，不跨 await）
        let provider_str = format!("{:?}", ai_config.provider);
        let model_str = ai_config.model.clone().unwrap_or_default();
        match &response_result {
            Ok(resp) => {
                let response_summary = if resp.chars().count() > 300 {
                    let truncated: String = resp.chars().take(300).collect();
                    format!("{}...", truncated)
                } else {
                    resp.clone()
                };
                if let Ok(conn) = self.get_connection() {
                    let _ = AiService::save_log(
                        &conn,
                        "hourly_summary",
                        "generate",
                        &provider_str,
                        Some(model_str.as_str()),
                        &prompt_summary,
                        Some(response_summary.as_str()),
                        None,
                        Some(duration_ms as i64),
                        "success",
                        None,
                    );
                }
            }
            Err(e) => {
                let err_msg = e.to_string();
                if let Ok(conn) = self.get_connection() {
                    let _ = AiService::save_log(
                        &conn,
                        "hourly_summary",
                        "generate",
                        &provider_str,
                        Some(model_str.as_str()),
                        &prompt_summary,
                        None,
                        None,
                        Some(duration_ms as i64),
                        "error",
                        Some(err_msg.as_str()),
                    );
                }
            }
        }

        let response = response_result?;

        // 解析AI返回的JSON
        let summary_text = self.parse_hourly_summary_response(&response)?;

        // 创建小时总结对象
        let hourly_summary = HourlySummary {
            id: None,
            date: date.to_string(),
            hour,
            summary_text,
            activity_type: main_activity_type,
            source_summary_ids: Some(source_summary_ids),
            screenshot_count: Some(screenshot_count),
            created_at: Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        };

        // 保存到数据库
        let id = self.save_hourly_summary(&hourly_summary)?;

        log::info!("小时总结已生成并保存，ID: {}", id);

        Ok(HourlySummary {
            id: Some(id),
            ..hourly_summary
        })
    }

    /// 解析小时总结AI响应（从JSON提取title和description）
    fn parse_hourly_summary_response(&self, response: &str) -> Result<String> {
        // 尝试清理被```json```包裹的响应
        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        // 解析JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(cleaned) {
            let mut parts = Vec::new();

            // 提取 title
            if let Some(title) = json.get("title").and_then(|v| v.as_str()) {
                parts.push(format!("**{}**", title));
            }

            // 提取 description
            if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
                parts.push(desc.to_string());
            }

            // 提取 keyActivities
            if let Some(activities) = json.get("keyActivities").and_then(|v| v.as_array()) {
                let activity_texts: Vec<String> = activities
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| format!("• {}", s))
                    .collect();

                if !activity_texts.is_empty() {
                    parts.push(format!("\n关键活动：\n{}", activity_texts.join("\n")));
                }
            }

            if !parts.is_empty() {
                return Ok(parts.join("\n"));
            }
        }

        // 如果JSON解析失败，返回原始响应
        log::warn!("无法解析小时总结JSON响应，使用原始文本");
        Ok(response.to_string())
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
