use anyhow::{anyhow, Result};
use reqwest::Client;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// AI 提供商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    DeepSeek,
    Qwen,
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
        }
    }
}

impl std::str::FromStr for AiProvider {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "deepseek" => Ok(AiProvider::DeepSeek),
            "qwen" => Ok(AiProvider::Qwen),
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
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::DeepSeek,
            api_key: String::new(),
            base_url: None,
            model: None,
            enabled: true,
        }
    }
}

impl AiConfig {
    /// 获取 API 基础 URL
    pub fn get_base_url(&self) -> &str {
        if let Some(url) = &self.base_url {
            url.as_str()
        } else {
            match self.provider {
                AiProvider::DeepSeek => "https://api.deepseek.com",
                AiProvider::Qwen => "https://dashscope.aliyuncs.com/compatible-mode",
            }
        }
    }

    /// 获取模型名称
    pub fn get_model(&self) -> &str {
        if let Some(model) = &self.model {
            model.as_str()
        } else {
            match self.provider {
                AiProvider::DeepSeek => "deepseek-chat",
                AiProvider::Qwen => "qwen-turbo",
            }
        }
    }

    /// 验证配置是否有效
    pub fn is_valid(&self) -> bool {
        !self.api_key.is_empty() && self.enabled
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
            max_tokens: 4096,
        };

        let url = format!("{}/v1/chat/completions", base_url);

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

        let url = format!("{}/v1/chat/completions", base_url);

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
        let prompt = format!(
            r#"请根据以下信息生成今日工作日志：

日期：{}

完成的任务：
{}

执行的 SQL 操作：
{}

Git 提交记录：
{}

要求：
1. 使用简洁的语言描述工作内容
2. 按工作类型分组（开发、数据库、文档等）
3. 突出重要的成果和进展
4. 使用 Markdown 格式

生成格式：
## {} 工作日志

### 开发工作
- ...

### 数据库工作
- ...

### 其他
- ..."#,
            input.date,
            if input.completed_tasks.is_empty() {
                "无".to_string()
            } else {
                input.completed_tasks.join("\n")
            },
            if input.executed_sqls.is_empty() {
                "无".to_string()
            } else {
                input
                    .executed_sqls
                    .iter()
                    .take(10)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            if input.git_commits.is_empty() {
                "无".to_string()
            } else {
                input.git_commits.join("\n")
            },
            input.date
        );

        let response = self.chat(vec![ChatMessage::user(prompt)]).await?;
        Ok(response.trim().to_string())
    }

    /// 日志润色
    pub async fn polish_work_log(&self, content: &str) -> Result<String> {
        let prompt = format!(
            r#"请优化以下工作日志，使其更专业、条理更清晰：

{}

要求：
1. 保持原有内容的核心信息
2. 改善语言表达
3. 优化格式结构
4. 使用 Markdown 格式

直接返回优化后的日志内容。"#,
            content
        );

        let response = self.chat(vec![ChatMessage::user(prompt)]).await?;
        Ok(response.trim().to_string())
    }

    /// 生成周报
    pub async fn generate_weekly_report(&self, logs: Vec<String>) -> Result<String> {
        let prompt = format!(
            r#"请根据以下工作日志生成周报：

日志内容：
{}

要求：
1. 总结本周主要工作成果
2. 列出遇到的问题和解决方案
3. 规划下周工作重点
4. 使用 Markdown 格式

生成格式：
## 周报

### 本周工作成果
- ...

### 遇到的问题与解决方案
- ...

### 下周工作计划
- ..."#,
            logs.join("\n\n---\n\n")
        );

        let response = self.chat(vec![ChatMessage::user(prompt)]).await?;
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

        // 精简提示词，减少token消耗
        let prompt = format!(
            r#"任务分类。标题：{}{}{}

返回JSON：{{"category":"backend|database|feature|docs|other","priority":1-3,"quadrant":"urgent_important|urgent_not_important|not_urgent_important|not_urgent_not_important","suggestedTags":["标签"],"confidence":0-1}}
只返回JSON。任务描述保持100字符以下。"#,
            title,
            if desc.is_empty() { String::new() } else { format!("\n描述：{}", desc) },
            tags_hint
        );

        let response = self.chat(vec![ChatMessage::user(prompt)]).await?;
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
        let prompt = format!(
            r#"请帮我完善以下任务的描述，使其更加清晰、具体、可执行：

任务标题：{}
当前描述：{}

请补充：
1. 具体的实现步骤
2. 需要注意的事项
3. 可能的技术难点
4. 验收标准

直接返回优化后的描述文本，使用 Markdown 格式。"#,
            title, desc
        );

        let response = self.chat(vec![ChatMessage::user(prompt)]).await?;
        Ok(response.trim().to_string())
    }

    /// 生成子任务
    pub async fn generate_subtasks(
        &self,
        title: &str,
        description: Option<&str>,
    ) -> Result<Vec<String>> {
        let desc = description.unwrap_or("无");
        let prompt = format!(
            r#"请将以下任务拆分为具体的子任务：

任务标题：{}
任务描述：{}

要求：
1. 每个子任务应该是可独立完成的
2. 子任务粒度适中（2-4小时可完成）
3. 子任务之间有清晰的先后顺序

返回 JSON 数组格式，只返回子任务标题：
["子任务1", "子任务2", "子任务3"]

只返回 JSON 数组，不要其他内容。"#,
            title, desc
        );

        let response = self.chat(vec![ChatMessage::user(prompt)]).await?;
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

        let prompt = format!(
            r#"请根据以下任务列表生成工作总结：

任务列表：
{}

要求：
1. 总结完成的主要工作
2. 分析工作重点和效率
3. 提出改进建议
4. 使用 Markdown 格式"#,
            tasks_str
        );

        let response = self.chat(vec![ChatMessage::user(prompt)]).await?;
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

        let prompt = format!(
            r#"你是一个应用分类助手。请根据应用名称，为以下应用分配分类。

可用分类：{}

应用列表：
{}

返回 JSON 数组格式（只返回JSON，不要其他内容）：
[{{"appId": "id值", "category": "分类名", "tags": ["标签"], "confidence": 0.9}}]"#,
            categories.join("、"),
            serde_json::to_string(&simplified_apps).unwrap_or_default()
        );

        let response = self.chat(vec![ChatMessage::user(prompt)]).await?;
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
        let prompt = format!(
            r#"根据以下应用启动历史，推荐可能的工作流组合：

应用列表：
{}

启动历史（应用ID和启动次数）：
{}

请分析用户的使用习惯，推荐 3-5 个工作流组合，返回 JSON：
[
  {{
    "name": "工作流名称",
    "appIds": ["app1", "app2"],
    "reason": "推荐理由"
  }}
]"#,
            serde_json::to_string_pretty(&apps).unwrap_or_default(),
            launch_history.iter()
                .map(|(id, count)| format!("{}: {} 次", id, count))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let response = self.chat(vec![ChatMessage::user(prompt)]).await?;
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
        let prompt = format!(
            r#"请为以下应用生成一个简短的中文描述（不超过50字）：

应用名称：{}
应用路径：{}

只返回描述文本，不要其他内容。"#,
            app_name, app_path
        );

        let response = self.chat(vec![ChatMessage::user(prompt)]).await?;
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
    }

    #[test]
    fn test_ai_provider_from_str() {
        assert_eq!(
            "deepseek".parse::<AiProvider>().unwrap(),
            AiProvider::DeepSeek
        );
        assert_eq!("qwen".parse::<AiProvider>().unwrap(), AiProvider::Qwen);
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
