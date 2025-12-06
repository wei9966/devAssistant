use anyhow::{anyhow, Result};
use reqwest::Client;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::services::prompt_manager_service::PromptManager;
use crate::services::ai_service::AiService;

/// VLM Provider type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum VlmProvider {
    QwenVl,      // Qwen VL
    DeepseekVl,  // DeepSeek VL
    Openai,      // OpenAI GPT-4V
    Doubao,      // Doubao
    Kimi,        // Kimi/Moonshot
    Claude,      // Claude
    Custom,      // Custom OpenAI compatible API
}

impl std::fmt::Display for VlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VlmProvider::QwenVl => write!(f, "qwen-vl"),
            VlmProvider::DeepseekVl => write!(f, "deepseek-vl"),
            VlmProvider::Openai => write!(f, "openai"),
            VlmProvider::Doubao => write!(f, "doubao"),
            VlmProvider::Kimi => write!(f, "kimi"),
            VlmProvider::Claude => write!(f, "claude"),
            VlmProvider::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for VlmProvider {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "qwen-vl" => Ok(VlmProvider::QwenVl),
            "deepseek-vl" => Ok(VlmProvider::DeepseekVl),
            "openai" => Ok(VlmProvider::Openai),
            "doubao" => Ok(VlmProvider::Doubao),
            "kimi" => Ok(VlmProvider::Kimi),
            "claude" => Ok(VlmProvider::Claude),
            "custom" => Ok(VlmProvider::Custom),
            _ => Err(anyhow!("Unknown VLM provider: {}", s)),
        }
    }
}

/// VLM Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VlmConfig {
    pub provider: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub enabled: bool,
    pub max_image_size: Option<u32>,
    pub image_quality: Option<u32>,
    pub timeout: Option<u32>,
    /// 最大输出 token 数，默认 8192
    pub max_tokens: Option<u32>,
}

impl Default for VlmConfig {
    fn default() -> Self {
        Self {
            provider: "qwen-vl".to_string(),
            api_key: String::new(),
            base_url: None,
            model: None,
            enabled: false,
            max_image_size: Some(10240),  // 10MB
            image_quality: Some(80),
            timeout: Some(30),
            max_tokens: Some(8192),
        }
    }
}

impl VlmConfig {
    /// Get API base URL (removes trailing slashes)
    pub fn get_base_url(&self) -> String {
        let url = if let Some(url) = &self.base_url {
            if !url.is_empty() {
                url.clone()
            } else {
                self.get_default_base_url().to_string()
            }
        } else {
            self.get_default_base_url().to_string()
        };
        // 移除末尾的斜杠，避免 URL 拼接时出现双斜杠
        url.trim_end_matches('/').to_string()
    }

    /// Get default base URL for provider
    fn get_default_base_url(&self) -> &str {
        match self.provider.as_str() {
            "qwen-vl" => "https://dashscope.aliyuncs.com/compatible-mode/v1",
            "deepseek-vl" => "https://api.deepseek.com/v1",
            "openai" => "https://api.openai.com/v1",
            "doubao" => "https://ark.cn-beijing.volces.com/api/v3",
            "kimi" => "https://api.moonshot.cn/v1",
            "claude" => "https://api.anthropic.com/v1",
            "custom" => "", // 自定义提供商必须设置 base_url
            _ => "",
        }
    }

    /// Get model name
    pub fn get_model(&self) -> &str {
        if let Some(model) = &self.model {
            if !model.is_empty() {
                return model.as_str();
            }
        }
        match self.provider.as_str() {
            "qwen-vl" => "qwen-vl-plus",
            "deepseek-vl" => "deepseek-vl",
            "openai" => "gpt-4-vision-preview",
            "doubao" => "doubao-vision-pro-32k",
            "kimi" => "moonshot-v1-8k-vision-preview",
            "claude" => "claude-3-sonnet-20240229",
            "custom" => "", // 自定义提供商必须设置 model
            _ => "",
        }
    }

    /// Validate configuration
    pub fn is_valid(&self) -> bool {
        if self.api_key.is_empty() || !self.enabled {
            return false;
        }
        // 自定义提供商必须配置 base_url 和 model
        if self.provider == "custom" {
            let has_base_url = self.base_url.as_ref().map_or(false, |u| !u.is_empty());
            let has_model = self.model.as_ref().map_or(false, |m| !m.is_empty());
            return has_base_url && has_model;
        }
        true
    }

    /// Get timeout duration
    pub fn get_timeout(&self) -> Duration {
        Duration::from_secs(self.timeout.unwrap_or(30) as u64)
    }

    /// Get max tokens
    pub fn get_max_tokens(&self) -> u32 {
        self.max_tokens.unwrap_or(8192)
    }
}

/// VLM Configuration Response (API Key masked)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VlmConfigResponse {
    pub provider: String,
    pub api_key_preview: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub enabled: bool,
    pub max_image_size: Option<u32>,
    pub image_quality: Option<u32>,
    pub timeout: Option<u32>,
    pub max_tokens: Option<u32>,
}

/// Screenshot Analysis Response from VLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotAnalysisResponse {
    pub title: String,
    pub summary: String,
    pub keywords: Vec<String>,
    pub importance: i32,
    #[serde(alias = "appName", default)]
    pub app_name: String,
    #[serde(alias = "activityType", alias = "context_type", default)]
    pub activity_type: String,
}

/// VLM 批量分析响应结构（来自 PromptManager 的新格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VlmBatchResponse {
    items: Vec<VlmBatchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VlmBatchItem {
    decision: Option<String>,
    #[serde(default)]
    history_id: Option<String>,
    #[serde(default)]
    screen_ids: Vec<i32>,
    analysis: VlmBatchAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VlmBatchAnalysis {
    #[serde(default)]
    context_type: Option<String>,
    title: String,
    summary: String,
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    importance: i32,
    #[serde(default)]
    confidence: Option<i32>,
    #[serde(default)]
    event_time: Option<String>,
}

/// VLM Service
pub struct VlmService {
    db_path: String,
}

/// 安全截取字符串，避免在多字节字符中间截断
fn safe_truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect()
    }
}

impl VlmService {
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }

    /// Get database connection
    fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path).map_err(|e| anyhow!("Failed to open database: {}", e))
    }

    /// 保存 VLM 调用日志
    fn save_vlm_log(
        &self,
        action: &str,
        provider: &str,
        model: &str,
        prompt_summary: &str,
        response_summary: &str,
        duration_ms: u64,
        status: &str,
        error_message: Option<&str>,
    ) {
        if let Ok(conn) = self.get_connection() {
            if let Err(e) = AiService::save_log(
                &conn,
                "vlm",
                action,
                provider,
                Some(model),
                prompt_summary,
                Some(response_summary),
                None, // tokens_used
                Some(duration_ms as i64),
                status,
                error_message,
            ) {
                log::warn!("保存 VLM 日志失败: {}", e);
            }
        }
    }

    /// Save VLM configuration
    pub fn save_config(&self, config: VlmConfig) -> Result<()> {
        let conn = self.get_connection()?;

        // Save to vlm_config table
        conn.execute(
            "INSERT OR REPLACE INTO vlm_config (
                id, provider, api_key, base_url, model, enabled,
                max_image_size, image_quality, timeout, max_tokens, updated_at
            ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now', 'localtime'))",
            rusqlite::params![
                config.provider,
                config.api_key,
                config.base_url,
                config.model,
                config.enabled as i32,
                config.max_image_size,
                config.image_quality,
                config.timeout,
                config.max_tokens,
            ],
        ).map_err(|e| anyhow!("Failed to save VLM config: {}", e))?;

        Ok(())
    }

    /// Get VLM configuration
    pub fn get_config(&self) -> Result<Option<VlmConfigResponse>> {
        let conn = self.get_connection()?;

        let result = conn.query_row(
            "SELECT provider, api_key, base_url, model, enabled,
                    max_image_size, image_quality, timeout, max_tokens
             FROM vlm_config WHERE id = 1",
            [],
            |row| {
                Ok(VlmConfigResponse {
                    provider: row.get(0)?,
                    api_key_preview: Self::mask_api_key(&row.get::<_, String>(1)?),
                    base_url: row.get(2)?,
                    model: row.get(3)?,
                    enabled: row.get::<_, i32>(4)? != 0,
                    max_image_size: row.get(5)?,
                    image_quality: row.get(6)?,
                    timeout: row.get(7)?,
                    max_tokens: row.get(8)?,
                })
            },
        );

        match result {
            Ok(config) => Ok(Some(config)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow!("Failed to get VLM config: {}", e)),
        }
    }


    /// Mask API Key
    fn mask_api_key(api_key: &str) -> String {
        if api_key.len() <= 8 {
            "*".repeat(api_key.len())
        } else {
            format!("{}***{}", &api_key[..4], &api_key[api_key.len() - 4..])
        }
    }

    /// Check if VLM is enabled
    pub fn is_enabled(&self) -> Result<bool> {
        let conn = self.get_connection()?;

        let result = conn.query_row(
            "SELECT enabled FROM vlm_config WHERE id = 1",
            [],
            |row| row.get::<_, i32>(0),
        );

        match result {
            Ok(enabled) => Ok(enabled != 0),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(anyhow!("Failed to check VLM enabled status: {}", e)),
        }
    }

    /// Get full configuration (for internal use, includes unmasked API Key)
    fn get_full_config(&self) -> Result<VlmConfig> {
        let conn = self.get_connection()?;

        let result = conn.query_row(
            "SELECT provider, api_key, base_url, model, enabled,
                    max_image_size, image_quality, timeout, max_tokens
             FROM vlm_config WHERE id = 1",
            [],
            |row| {
                Ok(VlmConfig {
                    provider: row.get(0)?,
                    api_key: row.get(1)?,
                    base_url: row.get(2)?,
                    model: row.get(3)?,
                    enabled: row.get::<_, i32>(4)? != 0,
                    max_image_size: row.get(5)?,
                    image_quality: row.get(6)?,
                    timeout: row.get(7)?,
                    max_tokens: row.get(8)?,
                })
            },
        );

        match result {
            Ok(config) => Ok(config),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(anyhow!("VLM configuration not found. Please configure VLM first."))
            }
            Err(e) => Err(anyhow!("Failed to get VLM config: {}", e)),
        }
    }

    /// Test connection
    pub async fn test_connection(&self) -> Result<bool> {
        let config = self.get_full_config()?;

        if !config.is_valid() {
            return Err(anyhow!("VLM configuration is invalid"));
        }

        // Use simple test prompt
        let test_prompt = "Please reply: Connection successful";
        // 32x32 red PNG image (meets minimum 14x14 requirement for most APIs)
        let test_image = "iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAQ0lEQVR4nO3NQQ0AIAwEwfbfKbwA4eM5MpN9bPcG2/V8fMV/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABwxQVmuwLT69W73AAAAABJRU5ErkJggg==";

        match self.analyze_image_internal(&config, test_image, test_prompt).await {
            Ok(_) => Ok(true),
            Err(e) => Err(anyhow!("Connection test failed: {}", e)),
        }
    }

    /// Analyze image
    pub async fn analyze_image(&self, image_base64: &str, prompt: &str) -> Result<String> {
        let start_time = Instant::now();
        let config = self.get_full_config()?;

        if !config.is_valid() {
            return Err(anyhow!("VLM is not enabled or configuration is invalid"));
        }

        let provider = config.provider.clone();
        let model = config.get_model().to_string();
        let prompt_summary = format!("[图片分析] {}", safe_truncate(prompt, 200));

        let result = self.analyze_image_internal(&config, image_base64, prompt).await;
        let duration_ms = start_time.elapsed().as_millis() as u64;

        match &result {
            Ok(response) => {
                self.save_vlm_log(
                    "image_analyze",
                    &provider,
                    &model,
                    &prompt_summary,
                    safe_truncate(response, 500).as_str(),
                    duration_ms,
                    "success",
                    None,
                );
            }
            Err(e) => {
                self.save_vlm_log(
                    "image_analyze",
                    &provider,
                    &model,
                    &prompt_summary,
                    "",
                    duration_ms,
                    "error",
                    Some(&e.to_string()),
                );
            }
        }

        result
    }

    /// Analyze single screenshot
    /// 单张截图分析使用专门的单张截图提示词
    pub async fn analyze_screenshot(
        &self,
        image_base64: &str,
        history: Option<&str>,
    ) -> Result<ScreenshotAnalysisResponse> {
        let start_time = Instant::now();
        let config = self.get_full_config()?;

        if !config.is_valid() {
            return Err(anyhow!("VLM is not enabled or configuration is invalid"));
        }

        // 尝试从 PromptManager 加载单张截图分析提示词
        let prompt_config = match PromptManager::get_instance() {
            Ok(manager) => manager.get_screenshot_single_prompt().clone(),
            Err(e) => {
                log::warn!("Failed to load PromptManager, using fallback: {}", e);
                return self.analyze_screenshot_fallback(&config, image_base64, history).await;
            }
        };

        // 准备变量
        let now = chrono::Local::now();
        let mut vars = HashMap::new();
        vars.insert(
            "current_timestamp".to_string(),
            now.format("%Y-%m-%d %H:%M:%S").to_string(),
        );
        vars.insert(
            "history".to_string(),
            history.unwrap_or("无历史记录").to_string(),
        );

        // 渲染提示词
        let system_prompt = PromptManager::render_prompt(&prompt_config.system, &vars);
        let user_prompt = PromptManager::render_prompt(&prompt_config.user, &vars);
        let prompt_summary = format!("[截图分析] {}", safe_truncate(&user_prompt, 200));

        // 调用 VLM API
        let api_result = match config.provider.as_str() {
            "claude" => {
                self.analyze_with_claude_messages(&config, image_base64, &system_prompt, &user_prompt)
                    .await
            }
            _ => {
                self.analyze_with_openai_compatible_messages(
                    &config,
                    image_base64,
                    &system_prompt,
                    &user_prompt,
                )
                .await
            }
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let provider = config.provider.clone();
        let model = config.get_model().to_string();

        match api_result {
            Ok(response_text) => {
                // 解析 JSON 响应
                match self.parse_screenshot_response(&response_text) {
                    Ok(result) => {
                        // 记录成功日志
                        let response_summary = format!("[{}] {}", result.title, safe_truncate(&result.summary, 200));
                        self.save_vlm_log(
                            "screenshot_analyze",
                            &provider,
                            &model,
                            &prompt_summary,
                            &response_summary,
                            duration_ms,
                            "success",
                            None,
                        );
                        Ok(result)
                    }
                    Err(e) => {
                        // 记录解析失败日志
                        self.save_vlm_log(
                            "screenshot_analyze",
                            &provider,
                            &model,
                            &prompt_summary,
                            safe_truncate(&response_text, 500).as_str(),
                            duration_ms,
                            "error",
                            Some(&format!("JSON解析失败: {}", e)),
                        );
                        Err(e)
                    }
                }
            }
            Err(e) => {
                // 记录 API 调用失败日志
                self.save_vlm_log(
                    "screenshot_analyze",
                    &provider,
                    &model,
                    &prompt_summary,
                    "",
                    duration_ms,
                    "error",
                    Some(&e.to_string()),
                );
                Err(e)
            }
        }
    }


    /// Parse screenshot analysis response
    /// 支持两种格式：
    /// 1. 新的批量格式：{ "items": [{ "analysis": { ... } }] }
    /// 2. 旧的扁平格式：{ "title": "...", "summary": "...", ... }
    fn parse_screenshot_response(&self, response_text: &str) -> Result<ScreenshotAnalysisResponse> {
        // 记录 VLM 返回的原始文本内容
        log::debug!("VLM 返回的原始文本: {}", response_text);

        // Try to find JSON block in the response
        let json_text = if let Some(start) = response_text.find('{') {
            if let Some(end) = response_text.rfind('}') {
                &response_text[start..=end]
            } else {
                response_text
            }
        } else {
            response_text
        };

        log::debug!("提取的 JSON 文本: {}", json_text);

        // 首先尝试解析为新的批量格式（带 items 数组）
        if let Ok(batch_response) = serde_json::from_str::<VlmBatchResponse>(json_text) {
            if let Some(first_item) = batch_response.items.first() {
                let analysis = &first_item.analysis;
                log::debug!("成功解析为批量格式，提取第一个 item 的 analysis");
                return Ok(ScreenshotAnalysisResponse {
                    title: analysis.title.clone(),
                    summary: analysis.summary.clone(),
                    keywords: analysis.keywords.clone(),
                    importance: analysis.importance,
                    // context_type 映射到 activity_type
                    activity_type: analysis.context_type.clone().unwrap_or_else(|| "other".to_string()),
                    // 批量格式中没有 app_name，设为空字符串
                    app_name: String::new(),
                });
            }
        }

        // 如果批量格式解析失败，尝试解析为扁平格式
        serde_json::from_str::<ScreenshotAnalysisResponse>(json_text).map_err(|e| {
            log::error!("JSON 解析失败: {}，原始文本: {}", e, response_text);
            anyhow!(
                "Failed to parse VLM response as JSON: {}. Response: {}",
                e,
                response_text
            )
        })
    }

    /// Fallback method when PromptService is unavailable
    async fn analyze_screenshot_fallback(
        &self,
        config: &VlmConfig,
        image_base64: &str,
        history: Option<&str>,
    ) -> Result<ScreenshotAnalysisResponse> {
        let default_prompt = format!(
            r#"请分析这张屏幕截图，识别用户正在进行的活动。
当前时间: {}
历史记录: {}

请以JSON格式返回分析结果：
{{
  "title": "简洁的活动标题",
  "summary": "详细的活动描述",
  "keywords": ["关键词1", "关键词2"],
  "importance": 5,
  "app_name": "应用程序名称",
  "activity_type": "coding"
}}"#,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            history.unwrap_or("无")
        );

        let response_text = self
            .analyze_image_internal(config, image_base64, &default_prompt)
            .await?;

        self.parse_screenshot_response(&response_text)
    }

    /// Internal image analysis method
    async fn analyze_image_internal(
        &self,
        config: &VlmConfig,
        image_base64: &str,
        prompt: &str,
    ) -> Result<String> {
        match config.provider.as_str() {
            "claude" => self.analyze_with_claude(config, image_base64, prompt).await,
            _ => self.analyze_with_openai_compatible(config, image_base64, prompt).await,
        }
    }

    /// Analyze image with OpenAI compatible API
    async fn analyze_with_openai_compatible(
        &self,
        config: &VlmConfig,
        image_base64: &str,
        prompt: &str,
    ) -> Result<String> {
        let client = Client::builder()
            .timeout(config.get_timeout())
            .build()?;

        let url = format!("{}/chat/completions", config.get_base_url());

        let request_body = serde_json::json!({
            "model": config.get_model(),
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": prompt
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:image/png;base64,{}", image_base64)
                            }
                        }
                    ]
                }
            ],
            "max_tokens": config.get_max_tokens()
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API request failed: {} - {}", status, error_text));
        }

        let response_json: serde_json::Value = response.json().await?;

        // 记录原始响应用于调试
        log::debug!("VLM API 原始响应: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                log::error!("VLM 响应解析失败，原始响应: {}", serde_json::to_string(&response_json).unwrap_or_default());
                anyhow!("Unable to parse response content")
            })?;

        Ok(content.to_string())
    }

    /// Analyze image with Claude API
    async fn analyze_with_claude(
        &self,
        config: &VlmConfig,
        image_base64: &str,
        prompt: &str,
    ) -> Result<String> {
        let client = Client::builder()
            .timeout(config.get_timeout())
            .build()?;

        let url = format!("{}/messages", config.get_base_url());

        let request_body = serde_json::json!({
            "model": config.get_model(),
            "max_tokens": config.get_max_tokens(),
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": "image/png",
                                "data": image_base64
                            }
                        },
                        {
                            "type": "text",
                            "text": prompt
                        }
                    ]
                }
            ]
        });

        let response = client
            .post(&url)
            .header("x-api-key", &config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API request failed: {} - {}", status, error_text));
        }

        let response_json: serde_json::Value = response.json().await?;

        // 记录原始响应用于调试
        log::debug!("Claude API 原始响应: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());

        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| {
                log::error!("Claude 响应解析失败，原始响应: {}", serde_json::to_string(&response_json).unwrap_or_default());
                anyhow!("Unable to parse response content")
            })?;

        Ok(content.to_string())
    }

    /// Analyze image with Claude API (with system and user prompts)
    async fn analyze_with_claude_messages(
        &self,
        config: &VlmConfig,
        image_base64: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        let client = Client::builder()
            .timeout(config.get_timeout())
            .build()?;

        let url = format!("{}/messages", config.get_base_url());

        let request_body = serde_json::json!({
            "model": config.get_model(),
            "max_tokens": config.get_max_tokens(),
            "system": system_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": "image/png",
                                "data": image_base64
                            }
                        },
                        {
                            "type": "text",
                            "text": user_prompt
                        }
                    ]
                }
            ]
        });

        let response = client
            .post(&url)
            .header("x-api-key", &config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API request failed: {} - {}", status, error_text));
        }

        let response_json: serde_json::Value = response.json().await?;

        // 记录原始响应用于调试
        log::debug!("Claude Messages API 原始响应: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());

        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| {
                log::error!("Claude Messages 响应解析失败，原始响应: {}", serde_json::to_string(&response_json).unwrap_or_default());
                anyhow!("Unable to parse response content")
            })?;

        Ok(content.to_string())
    }

    /// Analyze image with OpenAI compatible API (with system and user prompts)
    async fn analyze_with_openai_compatible_messages(
        &self,
        config: &VlmConfig,
        image_base64: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        let client = Client::builder()
            .timeout(config.get_timeout())
            .build()?;

        let url = format!("{}/chat/completions", config.get_base_url());

        let request_body = serde_json::json!({
            "model": config.get_model(),
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": user_prompt
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:image/png;base64,{}", image_base64)
                            }
                        }
                    ]
                }
            ],
            "max_tokens": config.get_max_tokens()
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API request failed: {} - {}", status, error_text));
        }

        let response_json: serde_json::Value = response.json().await?;

        // 记录原始响应用于调试
        log::debug!("OpenAI Messages API 原始响应: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                log::error!("OpenAI Messages 响应解析失败，原始响应: {}", serde_json::to_string(&response_json).unwrap_or_default());
                anyhow!("Unable to parse response content")
            })?;

        Ok(content.to_string())
    }
}
