use anyhow::{anyhow, Result};
use reqwest::Client;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use crate::services::prompt_db_service::PromptDbService;

/// AI 提供商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    DeepSeek,
    Qwen,
    Custom,
}

impl Default for AiProvider {
    fn default() -> Self {
        AiProvider::DeepSeek
    }
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::DeepSeek => write!(f, "deepseek"),
            AiProvider::Qwen => write!(f, "qwen"),
            AiProvider::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for AiProvider {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "deepseek" => Ok(AiProvider::DeepSeek),
            "qwen" => Ok(AiProvider::Qwen),
            "custom" => Ok(AiProvider::Custom),
            _ => Err(anyhow!("未知的 AI 提供商: {}", s)),
        }
    }
}

/// AI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    pub provider: AiProvider,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub enabled: bool,
    /// 最大输出 token 数，默认 4096
    pub max_tokens: Option<u32>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::DeepSeek,
            api_key: String::new(),
            base_url: None,
            model: None,
            enabled: true,
            max_tokens: Some(4096),
        }
    }
}

impl AiConfig {
    /// 获取 API 基础 URL
    pub fn get_base_url(&self) -> &str {
        if let Some(url) = &self.base_url {
            if !url.is_empty() {
                return url.as_str();
            }
        }
        match self.provider {
            AiProvider::DeepSeek => "https://api.deepseek.com",
            AiProvider::Qwen => "https://dashscope.aliyuncs.com/compatible-mode",
            AiProvider::Custom => "", // 自定义提供商必须设置 base_url
        }
    }

    /// 获取模型名称
    pub fn get_model(&self) -> &str {
        if let Some(model) = &self.model {
            if !model.is_empty() {
                return model.as_str();
            }
        }
        match self.provider {
            AiProvider::DeepSeek => "deepseek-chat",
            AiProvider::Qwen => "qwen-turbo",
            AiProvider::Custom => "", // 自定义提供商必须设置 model
        }
    }

    /// 验证配置是否有效
    pub fn is_valid(&self) -> bool {
        if self.api_key.is_empty() || !self.enabled {
            return false;
        }
        // 自定义提供商必须配置 base_url 和 model
        if self.provider == AiProvider::Custom {
            let has_base_url = self.base_url.as_ref().map_or(false, |u| !u.is_empty());
            let has_model = self.model.as_ref().map_or(false, |m| !m.is_empty());
            return has_base_url && has_model;
        }
        true
    }

    /// 获取最大输出 token 数
    pub fn get_max_tokens(&self) -> u32 {
        self.max_tokens.unwrap_or(4096)
    }
}

/// 工作日志生成输入
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLogInput {
    pub date: String,
    pub completed_tasks: Vec<String>,
    pub executed_sqls: Vec<String>,
    pub git_commits: Vec<String>,
    /// 可选的屏幕活动摘要（来自截图回顾）
    pub activity_summary: Option<ActivitySummaryInput>,
    /// 里程碑进展更新
    pub progress_updates: Vec<String>,
}

/// 活动摘要输入（简化版，用于工作日志生成）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivitySummaryInput {
    /// 活动时间范围 (如 "09:00 - 18:30")
    pub time_range: String,
    /// 总活动时长（分钟）
    pub total_minutes: i32,
    /// 主要应用使用（前5个）
    pub top_apps: Vec<String>,
    /// 活动类型分布描述
    pub activity_distribution: String,
    /// 关键活动描述
    pub key_activities: Vec<String>,
}

/// 应用分类输入
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppClassifyInput {
    pub id: String,
    pub name: String,
    pub path: String,
}

/// 应用分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppClassifyResult {
    pub app_id: String,
    pub category: String,
    pub tags: Vec<String>,
    pub confidence: f32,
}

/// 工作流推荐
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRecommendation {
    pub name: String,
    pub app_ids: Vec<String>,
    pub reason: String,
}

/// 任务分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskClassifyResult {
    pub category: String,
    pub priority: i32,
    pub quadrant: String,
    pub suggested_tags: Vec<String>,
    pub confidence: f32,
}

/// 任务总结输入
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSummaryInput {
    pub title: String,
    pub status: String,
    pub completed_at: Option<String>,
}

/// AI 调用日志
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiLog {
    pub id: i64,
    pub module: String,
    pub action: String,
    pub provider: String,
    pub model: Option<String>,
    pub prompt: String,
    pub response: Option<String>,
    pub tokens_used: Option<i32>,
    pub duration_ms: Option<i64>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: i64,
}

/// AI 日志查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiLogQuery {
    pub module: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// AI 日志统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiLogStats {
    pub total_calls: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub total_tokens: i64,
    pub avg_duration_ms: f64,
    pub calls_by_module: Vec<(String, i64)>,
}

/// AI 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
        }
    }

    pub fn system(content: String) -> Self {
        Self {
            role: "system".to_string(),
            content,
        }
    }
}

/// AI 请求体
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: i32,
}

/// AI 响应体
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: String,
}

/// 全局 AI 服务
#[derive(Clone)]
pub struct AiService {
    client: Client,
    pub config: AiConfig,
}

impl AiService {
    /// 创建新的 AI 服务实例
    pub fn new(config: AiConfig) -> Self {
        // 创建带超时的 HTTP 客户端
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120)) // 2 分钟超时
            .connect_timeout(std::time::Duration::from_secs(30)) // 30 秒连接超时
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            config,
        }
    }

    /// 从数据库加载配置（从 app_settings 表读取）
    pub fn load_config(conn: &Connection) -> Result<AiConfig> {
        let config_json: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'ai_config'",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(json) = config_json {
            let value: serde_json::Value = serde_json::from_str(&json)
                .map_err(|e| anyhow!("解析 AI 配置失败: {}", e))?;

            let provider_str = value["provider"].as_str().unwrap_or("deepseek");
            let provider = provider_str.parse::<AiProvider>().unwrap_or_default();

            Ok(AiConfig {
                provider,
                api_key: value["api_key"].as_str().unwrap_or("").to_string(),
                base_url: value["base_url"].as_str().map(|s| s.to_string()),
                model: value["model"].as_str().map(|s| s.to_string()),
                enabled: value["enabled"].as_bool().unwrap_or(false),
                max_tokens: value["max_tokens"].as_u64().map(|n| n as u32),
            })
        } else {
            // 如果没有配置记录，返回默认配置
            Ok(AiConfig::default())
        }
    }

    /// 保存配置到数据库（保存到 app_settings 表）
    pub fn save_config(conn: &Connection, config: &AiConfig) -> Result<()> {
        let config_json = serde_json::json!({
            "provider": config.provider.to_string(),
            "api_key": config.api_key,
            "base_url": config.base_url,
            "model": config.model,
            "enabled": config.enabled,
            "max_tokens": config.max_tokens,
        });

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| anyhow!("获取时间戳失败: {}", e))?
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('ai_config', ?1, ?2)",
            rusqlite::params![config_json.to_string(), now],
        )?;

        Ok(())
    }

    /// 更新配置
    pub fn update_config(&mut self, config: AiConfig) {
        self.config = config;
    }

    /// 保存 AI 调用日志
    pub fn save_log(
        conn: &Connection,
        module: &str,
        action: &str,
        provider: &str,
        model: Option<&str>,
        prompt: &str,
        response: Option<&str>,
        tokens_used: Option<i32>,
        duration_ms: Option<i64>,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<i64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| anyhow!("获取时间戳失败: {}", e))?
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO ai_logs (module, action, provider, model, prompt, response, tokens_used, duration_ms, status, error_message, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                module,
                action,
                provider,
                model,
                prompt,
                response,
                tokens_used,
                duration_ms,
                status,
                error_message,
                now
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// 查询 AI 调用日志
    pub fn query_logs(conn: &Connection, query: &AiLogQuery) -> Result<Vec<AiLog>> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let mut sql = String::from(
            "SELECT id, module, action, provider, model, prompt, response, tokens_used, duration_ms, status, error_message, created_at
             FROM ai_logs WHERE 1=1"
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(module) = &query.module {
            sql.push_str(" AND module = ?");
            params.push(Box::new(module.clone()));
        }

        if let Some(status) = &query.status {
            sql.push_str(" AND status = ?");
            params.push(Box::new(status.clone()));
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit));
        params.push(Box::new(offset));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let logs = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(AiLog {
                    id: row.get(0)?,
                    module: row.get(1)?,
                    action: row.get(2)?,
                    provider: row.get(3)?,
                    model: row.get(4)?,
                    prompt: row.get(5)?,
                    response: row.get(6)?,
                    tokens_used: row.get(7)?,
                    duration_ms: row.get(8)?,
                    status: row.get(9)?,
                    error_message: row.get(10)?,
                    created_at: row.get(11)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// 获取 AI 调用统计
    pub fn get_log_stats(conn: &Connection) -> Result<AiLogStats> {
        let (total_calls, success_count, error_count, total_tokens, avg_duration): (i64, i64, i64, i64, f64) = conn
            .query_row(
                "SELECT
                    COUNT(*) as total,
                    COALESCE(SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END), 0) as success,
                    COALESCE(SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END), 0) as error,
                    COALESCE(SUM(tokens_used), 0) as tokens,
                    COALESCE(AVG(duration_ms), 0.0) as avg_duration
                 FROM ai_logs",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )?;

        let mut stmt = conn.prepare(
            "SELECT module, COUNT(*) FROM ai_logs GROUP BY module ORDER BY COUNT(*) DESC"
        )?;
        let calls_by_module = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(AiLogStats {
            total_calls,
            success_count,
            error_count,
            total_tokens,
            avg_duration_ms: avg_duration,
            calls_by_module,
        })
    }

    /// 清空日志
    pub fn clear_logs(conn: &Connection, before_timestamp: Option<i64>) -> Result<i64> {
        let deleted = if let Some(ts) = before_timestamp {
            conn.execute("DELETE FROM ai_logs WHERE created_at < ?1", [ts])?
        } else {
            conn.execute("DELETE FROM ai_logs", [])?
        };
        Ok(deleted as i64)
    }

    /// 带日志记录的聊天接口
    pub async fn chat_with_log(
        &self,
        conn: &Connection,
        module: &str,
        action: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<String> {
        let start = Instant::now();
        let prompt = messages.iter().map(|m| m.content.clone()).collect::<Vec<_>>().join("\n");
        let provider = self.config.provider.to_string();
        let model = self.config.get_model().to_string();

        match self.chat(messages).await {
            Ok(response) => {
                let duration_ms = start.elapsed().as_millis() as i64;
                // 记录成功日志
                let _ = Self::save_log(
                    conn,
                    module,
                    action,
                    &provider,
                    Some(&model),
                    &prompt,
                    Some(&response),
                    None, // tokens_used 暂时没有获取
                    Some(duration_ms),
                    "success",
                    None,
                );
                Ok(response)
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as i64;
                // 记录错误日志
                let _ = Self::save_log(
                    conn,
                    module,
                    action,
                    &provider,
                    Some(&model),
                    &prompt,
                    None,
                    None,
                    Some(duration_ms),
                    "error",
                    Some(&e.to_string()),
                );
                Err(e)
            }
        }
    }

    /// 通用聊天接口
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        if !self.config.is_valid() {
            return Err(anyhow!("AI 配置无效或未启用"));
        }

        let base_url = self.config.get_base_url();
        let model = self.config.get_model();

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            temperature: 0.7,
            max_tokens: self.config.get_max_tokens() as i32,
        };

        // 自定义提供商：如果 base_url 已包含版本路径（如 /v3、/v1），则直接追加 /chat/completions
        // 否则按标准 OpenAI 格式追加 /v1/chat/completions
        let url = if self.config.provider == AiProvider::Custom {
            let trimmed_url = base_url.trim_end_matches('/');
            // 检查是否已包含版本路径（如 /v1, /v2, /v3 等）
            if trimmed_url.contains("/v1") || trimmed_url.contains("/v2") || trimmed_url.contains("/v3") {
                format!("{}/chat/completions", trimmed_url)
            } else {
                format!("{}/v1/chat/completions", trimmed_url)
            }
        } else {
            format!("{}/v1/chat/completions", base_url)
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("API 请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API 返回错误 {}: {}", status, error_text));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("解析响应失败: {}", e))?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow!("AI 返回结果为空"))?;

        Ok(content)
    }

    /// 测试连接
    pub async fn test_connection(&self) -> Result<bool> {
        if !self.config.is_valid() {
            return Err(anyhow!("AI 配置无效或未启用"));
        }

        let base_url = self.config.get_base_url();
        let model = self.config.get_model();

        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![ChatMessage::user("你好".to_string())],
            temperature: 0.1,
            max_tokens: 10,
        };

        // 自定义提供商：如果 base_url 已包含版本路径（如 /v3、/v1），则直接追加 /chat/completions
        let url = if self.config.provider == AiProvider::Custom {
            let trimmed_url = base_url.trim_end_matches('/');
            if trimmed_url.contains("/v1") || trimmed_url.contains("/v2") || trimmed_url.contains("/v3") {
                format!("{}/chat/completions", trimmed_url)
            } else {
                format!("{}/v1/chat/completions", trimmed_url)
            }
        } else {
            format!("{}/v1/chat/completions", base_url)
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("连接测试失败: {}", e))?;

        Ok(response.status().is_success())
    }

    /// 检查是否已配置
    pub fn is_configured(&self) -> bool {
        self.config.is_valid()
    }

    /// 自动生成工作日志
    pub async fn generate_work_log(&self, input: WorkLogInput) -> Result<String> {
        // 检查是否有内容
        let has_tasks = !input.completed_tasks.is_empty();
        let has_sqls = !input.executed_sqls.is_empty();
        let has_commits = !input.git_commits.is_empty();
        let has_activity = input.activity_summary.is_some();
        let has_progress = !input.progress_updates.is_empty();

        // 如果没有任何内容，返回简单模板
        if !has_tasks && !has_sqls && !has_commits && !has_activity && !has_progress {
            return Ok(format!("## {} 工作日志\n\n今日暂无记录的工作内容。", input.date));
        }

        // 构建任务列表
        let tasks_section = if has_tasks {
            format!("已完成任务：\n{}", input.completed_tasks.iter()
                .map(|t| format!("- {}", t))
                .collect::<Vec<_>>()
                .join("\n"))
        } else {
            String::new()
        };

        // 构建 SQL 操作列表（限制数量）
        let sqls_section = if has_sqls {
            let sqls: Vec<_> = input.executed_sqls.iter().take(5).cloned().collect();
            format!("\n\nSQL 操作：\n{}", sqls.iter()
                .map(|s| format!("- {}", s))
                .collect::<Vec<_>>()
                .join("\n"))
        } else {
            String::new()
        };

        // 构建 Git 提交列表
        let commits_section = if has_commits {
            format!("\n\nGit 提交：\n{}", input.git_commits.iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n"))
        } else {
            String::new()
        };

        // 构建屏幕活动摘要
        let activity_section = if let Some(activity) = &input.activity_summary {
            let mut section = format!("\n\n屏幕活动记录：\n- 活动时间：{}", activity.time_range);
            if activity.total_minutes > 0 {
                let hours = activity.total_minutes / 60;
                let mins = activity.total_minutes % 60;
                section.push_str(&format!("（共 {}小时{}分钟）", hours, mins));
            }
            if !activity.top_apps.is_empty() {
                section.push_str(&format!("\n- 主要使用应用：{}", activity.top_apps.join("、")));
            }
            if !activity.activity_distribution.is_empty() {
                section.push_str(&format!("\n- 活动类型分布：{}", activity.activity_distribution));
            }
            if !activity.key_activities.is_empty() {
                section.push_str("\n- 关键活动：");
                for act in &activity.key_activities {
                    section.push_str(&format!("\n  - {}", act));
                }
            }
            section
        } else {
            String::new()
        };

        // 构建里程碑进展更新
        let progress_section = if has_progress {
            format!("\n\n里程碑进展更新：\n{}", input.progress_updates.iter()
                .map(|p| format!("- {}", p))
                .collect::<Vec<_>>()
                .join("\n"))
        } else {
            String::new()
        };

        // 从数据库获取提示词并渲染
        let mut vars = HashMap::new();
        vars.insert("date".to_string(), input.date.clone());
        vars.insert("tasks_section".to_string(), tasks_section);
        vars.insert("sqls_section".to_string(), sqls_section);
        vars.insert("commits_section".to_string(), commits_section);
        vars.insert("activity_section".to_string(), activity_section);
        vars.insert("progress_section".to_string(), progress_section);

        let rendered = PromptDbService::render_prompt_cached("work_log_generate", &vars)
            .map_err(|e| anyhow!("获取提示词失败: {}", e))?;

        let response = self.chat(vec![ChatMessage::user(rendered.user)]).await?;
        Ok(response.trim().to_string())
    }

    /// 日志润色
    pub async fn polish_work_log(&self, content: &str) -> Result<String> {
        let mut vars = HashMap::new();
        vars.insert("content".to_string(), content.to_string());

        let rendered = PromptDbService::render_prompt_cached("work_log_polish", &vars)
            .map_err(|e| anyhow!("获取提示词失败: {}", e))?;

        let response = self.chat(vec![ChatMessage::user(rendered.user)]).await?;
        Ok(response.trim().to_string())
    }

    /// 生成周报
    pub async fn generate_weekly_report(&self, logs: Vec<String>) -> Result<String> {
        let mut vars = HashMap::new();
        vars.insert("logs".to_string(), logs.join("\n\n---\n\n"));

        let rendered = PromptDbService::render_prompt_cached("weekly_report_generate", &vars)
            .map_err(|e| anyhow!("获取提示词失败: {}", e))?;

        let response = self.chat(vec![ChatMessage::user(rendered.user)]).await?;
        Ok(response.trim().to_string())
    }

    /// 任务智能分类
    pub async fn classify_task(
        &self,
        title: &str,
        description: Option<&str>,
        existing_tags: Option<&[String]>,
    ) -> Result<TaskClassifyResult> {
        // 截断过长的描述，只保留前300字符，提升响应速度
        let desc = match description {
            Some(d) if !d.is_empty() => {
                let trimmed = d.trim();
                if trimmed.chars().count() > 300 {
                    format!("{}...", trimmed.chars().take(300).collect::<String>())
                } else {
                    trimmed.to_string()
                }
            }
            _ => String::new(),
        };

        // 构建现有标签提示（限制数量）
        let tags_hint = if let Some(tags) = existing_tags {
            let limited_tags: Vec<_> = tags.iter().take(10).collect();
            if !limited_tags.is_empty() {
                format!("\n可选标签：{}", limited_tags.iter().map(|t| t.as_str()).collect::<Vec<_>>().join(","))
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let mut vars = HashMap::new();
        vars.insert("title".to_string(), title.to_string());
        vars.insert("description".to_string(), if desc.is_empty() { String::new() } else { format!("\n描述：{}", desc) });
        vars.insert("tags_hint".to_string(), tags_hint);

        let rendered = PromptDbService::render_prompt_cached("task_classify", &vars)
            .map_err(|e| anyhow!("获取提示词失败: {}", e))?;

        let response = self.chat(vec![ChatMessage::user(rendered.user)]).await?;
        self.parse_task_classify_response(&response)
    }

    fn parse_task_classify_response(&self, content: &str) -> Result<TaskClassifyResult> {
        let json_str = if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                &content[start..=end]
            } else {
                content
            }
        } else {
            content
        };

        serde_json::from_str(json_str)
            .map_err(|e| anyhow!("解析任务分类结果失败: {}", e))
    }

    /// 任务描述增强
    pub async fn enhance_task_description(
        &self,
        title: &str,
        description: Option<&str>,
    ) -> Result<String> {
        let desc = description.unwrap_or("无");

        let mut vars = HashMap::new();
        vars.insert("title".to_string(), title.to_string());
        vars.insert("description".to_string(), desc.to_string());

        let rendered = PromptDbService::render_prompt_cached("task_description_enhance", &vars)
            .map_err(|e| anyhow!("获取提示词失败: {}", e))?;

        let response = self.chat(vec![ChatMessage::user(rendered.user)]).await?;
        Ok(response.trim().to_string())
    }

    /// 生成子任务
    pub async fn generate_subtasks(
        &self,
        title: &str,
        description: Option<&str>,
    ) -> Result<Vec<String>> {
        let desc = description.unwrap_or("无");

        let mut vars = HashMap::new();
        vars.insert("title".to_string(), title.to_string());
        vars.insert("description".to_string(), desc.to_string());

        let rendered = PromptDbService::render_prompt_cached("task_subtasks_generate", &vars)
            .map_err(|e| anyhow!("获取提示词失败: {}", e))?;

        let response = self.chat(vec![ChatMessage::user(rendered.user)]).await?;
        self.parse_subtasks_response(&response)
    }

    fn parse_subtasks_response(&self, content: &str) -> Result<Vec<String>> {
        let json_str = if let Some(start) = content.find('[') {
            if let Some(end) = content.rfind(']') {
                &content[start..=end]
            } else {
                content
            }
        } else {
            content
        };

        serde_json::from_str(json_str)
            .map_err(|e| anyhow!("解析子任务结果失败: {}", e))
    }

    /// 任务总结
    pub async fn summarize_tasks(&self, tasks: Vec<TaskSummaryInput>) -> Result<String> {
        let tasks_str = tasks.iter()
            .map(|t| format!("- {} ({})", t.title, t.status))
            .collect::<Vec<_>>()
            .join("\n");

        let mut vars = HashMap::new();
        vars.insert("tasks".to_string(), tasks_str);

        let rendered = PromptDbService::render_prompt_cached("task_summarize", &vars)
            .map_err(|e| anyhow!("获取提示词失败: {}", e))?;

        let response = self.chat(vec![ChatMessage::user(rendered.user)]).await?;
        Ok(response.trim().to_string())
    }

    /// 应用智能分类
    pub async fn classify_apps(
        &self,
        apps: Vec<AppClassifyInput>,
        categories: Vec<String>,
    ) -> Result<Vec<AppClassifyResult>> {
        // 如果应用数量太多，分批处理
        const BATCH_SIZE: usize = 20;

        if apps.len() > BATCH_SIZE {
            let mut all_results = Vec::new();
            for chunk in apps.chunks(BATCH_SIZE) {
                let batch_results = self.classify_apps_batch(chunk.to_vec(), categories.clone()).await?;
                all_results.extend(batch_results);
            }
            return Ok(all_results);
        }

        self.classify_apps_batch(apps, categories).await
    }

    /// 分类应用的单批次处理
    async fn classify_apps_batch(
        &self,
        apps: Vec<AppClassifyInput>,
        categories: Vec<String>,
    ) -> Result<Vec<AppClassifyResult>> {
        // 简化应用信息，只传递必要字段
        let simplified_apps: Vec<serde_json::Value> = apps.iter().map(|app| {
            serde_json::json!({
                "id": app.id,
                "name": app.name,
                "path": std::path::Path::new(&app.path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&app.path)
            })
        }).collect();

        let mut vars = HashMap::new();
        vars.insert("categories".to_string(), categories.join("、"));
        vars.insert("apps".to_string(), serde_json::to_string(&simplified_apps).unwrap_or_default());

        let rendered = PromptDbService::render_prompt_cached("app_classify", &vars)
            .map_err(|e| anyhow!("获取提示词失败: {}", e))?;

        let response = self.chat(vec![ChatMessage::user(rendered.user)]).await?;
        self.parse_app_classify_response(&response)
    }

    fn parse_app_classify_response(&self, content: &str) -> Result<Vec<AppClassifyResult>> {
        let json_str = if let Some(start) = content.find('[') {
            if let Some(end) = content.rfind(']') {
                &content[start..=end]
            } else {
                content
            }
        } else {
            content
        };

        serde_json::from_str(json_str)
            .map_err(|e| anyhow!("解析应用分类结果失败: {}", e))
    }

    /// 工作流推荐
    pub async fn recommend_workflows(
        &self,
        apps: Vec<AppClassifyInput>,
        launch_history: Vec<(String, i64)>, // (app_id, launch_count)
    ) -> Result<Vec<WorkflowRecommendation>> {
        let mut vars = HashMap::new();
        vars.insert("apps".to_string(), serde_json::to_string_pretty(&apps).unwrap_or_default());
        vars.insert("launch_history".to_string(), launch_history.iter()
            .map(|(id, count)| format!("{}: {} 次", id, count))
            .collect::<Vec<_>>()
            .join("\n"));

        let rendered = PromptDbService::render_prompt_cached("workflow_recommend", &vars)
            .map_err(|e| anyhow!("获取提示词失败: {}", e))?;

        let response = self.chat(vec![ChatMessage::user(rendered.user)]).await?;
        self.parse_workflow_response(&response)
    }

    fn parse_workflow_response(&self, content: &str) -> Result<Vec<WorkflowRecommendation>> {
        let json_str = if let Some(start) = content.find('[') {
            if let Some(end) = content.rfind(']') {
                &content[start..=end]
            } else {
                content
            }
        } else {
            content
        };

        serde_json::from_str(json_str)
            .map_err(|e| anyhow!("解析工作流推荐结果失败: {}", e))
    }

    /// 生成应用描述
    pub async fn generate_app_description(
        &self,
        app_name: &str,
        app_path: &str,
    ) -> Result<String> {
        let mut vars = HashMap::new();
        vars.insert("app_name".to_string(), app_name.to_string());
        vars.insert("app_path".to_string(), app_path.to_string());

        let rendered = PromptDbService::render_prompt_cached("app_description_generate", &vars)
            .map_err(|e| anyhow!("获取提示词失败: {}", e))?;

        let response = self.chat(vec![ChatMessage::user(rendered.user)]).await?;
        Ok(response.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_ai_provider_to_string() {
        assert_eq!(AiProvider::DeepSeek.to_string(), "deepseek");
        assert_eq!(AiProvider::Qwen.to_string(), "qwen");
        assert_eq!(AiProvider::Custom.to_string(), "custom");
    }

    #[test]
    fn test_ai_provider_from_str() {
        assert_eq!(
            "deepseek".parse::<AiProvider>().unwrap(),
            AiProvider::DeepSeek
        );
        assert_eq!("qwen".parse::<AiProvider>().unwrap(), AiProvider::Qwen);
        assert_eq!("custom".parse::<AiProvider>().unwrap(), AiProvider::Custom);
        assert_eq!(
            "DEEPSEEK".parse::<AiProvider>().unwrap(),
            AiProvider::DeepSeek
        );
    }

    #[test]
    fn test_ai_config_default_urls() {
        let deepseek_config = AiConfig {
            provider: AiProvider::DeepSeek,
            ..Default::default()
        };
        assert_eq!(deepseek_config.get_base_url(), "https://api.deepseek.com");
        assert_eq!(deepseek_config.get_model(), "deepseek-chat");

        let qwen_config = AiConfig {
            provider: AiProvider::Qwen,
            ..Default::default()
        };
        assert_eq!(
            qwen_config.get_base_url(),
            "https://dashscope.aliyuncs.com/compatible-mode"
        );
        assert_eq!(qwen_config.get_model(), "qwen-turbo");
    }

    #[test]
    fn test_ai_config_custom_url() {
        let config = AiConfig {
            provider: AiProvider::DeepSeek,
            base_url: Some("https://custom.api.com".to_string()),
            model: Some("custom-model".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_base_url(), "https://custom.api.com");
        assert_eq!(config.get_model(), "custom-model");
    }

    #[test]
    fn test_ai_config_validation() {
        let mut config = AiConfig::default();
        assert!(!config.is_valid()); // 默认 api_key 为空

        config.api_key = "sk-test".to_string();
        assert!(config.is_valid());

        config.enabled = false;
        assert!(!config.is_valid());
    }

    #[test]
    fn test_save_and_load_config() {
        let conn = Connection::open_in_memory().unwrap();

        // 创建 app_settings 表
        conn.execute(
            "CREATE TABLE app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .unwrap();

        // 保存配置
        let config = AiConfig {
            provider: AiProvider::DeepSeek,
            api_key: "sk-test-key".to_string(),
            base_url: Some("https://test.com".to_string()),
            model: Some("test-model".to_string()),
            enabled: true,
        };

        AiService::save_config(&conn, &config).unwrap();

        // 加载配置
        let loaded = AiService::load_config(&conn).unwrap();
        assert_eq!(loaded.provider, AiProvider::DeepSeek);
        assert_eq!(loaded.api_key, "sk-test-key");
        assert_eq!(loaded.base_url, Some("https://test.com".to_string()));
        assert_eq!(loaded.model, Some("test-model".to_string()));
        assert!(loaded.enabled);
    }

    #[test]
    fn test_update_config() {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute(
            "CREATE TABLE app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .unwrap();

        // 首次保存
        let config1 = AiConfig {
            provider: AiProvider::DeepSeek,
            api_key: "sk-key-1".to_string(),
            base_url: None,
            model: None,
            enabled: true,
        };
        AiService::save_config(&conn, &config1).unwrap();

        // 更新配置
        let config2 = AiConfig {
            provider: AiProvider::Qwen,
            api_key: "sk-key-2".to_string(),
            base_url: Some("https://new.com".to_string()),
            model: Some("new-model".to_string()),
            enabled: false,
        };
        AiService::save_config(&conn, &config2).unwrap();

        // 验证更新
        let loaded = AiService::load_config(&conn).unwrap();
        assert_eq!(loaded.provider, AiProvider::Qwen);
        assert_eq!(loaded.api_key, "sk-key-2");
        assert!(!loaded.enabled);
    }

    #[test]
    fn test_chat_message_constructors() {
        let user_msg = ChatMessage::user("Hello".to_string());
        assert_eq!(user_msg.role, "user");
        assert_eq!(user_msg.content, "Hello");

        let assistant_msg = ChatMessage::assistant("Hi".to_string());
        assert_eq!(assistant_msg.role, "assistant");

        let system_msg = ChatMessage::system("You are a helper".to_string());
        assert_eq!(system_msg.role, "system");
    }
}
