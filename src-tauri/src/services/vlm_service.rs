use anyhow::{anyhow, Result};
use reqwest::Client;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
        }
    }
}

impl VlmConfig {
    /// Get API base URL
    pub fn get_base_url(&self) -> &str {
        if let Some(url) = &self.base_url {
            url.as_str()
        } else {
            match self.provider.as_str() {
                "qwen-vl" => "https://dashscope.aliyuncs.com/compatible-mode/v1",
                "deepseek-vl" => "https://api.deepseek.com/v1",
                "openai" => "https://api.openai.com/v1",
                "doubao" => "https://ark.cn-beijing.volces.com/api/v3",
                "kimi" => "https://api.moonshot.cn/v1",
                "claude" => "https://api.anthropic.com/v1",
                _ => "",
            }
        }
    }

    /// Get model name
    pub fn get_model(&self) -> &str {
        if let Some(model) = &self.model {
            model.as_str()
        } else {
            match self.provider.as_str() {
                "qwen-vl" => "qwen-vl-plus",
                "deepseek-vl" => "deepseek-vl",
                "openai" => "gpt-4-vision-preview",
                "doubao" => "doubao-vision-pro-32k",
                "kimi" => "moonshot-v1-8k-vision-preview",
                "claude" => "claude-3-sonnet-20240229",
                _ => "",
            }
        }
    }

    /// Validate configuration
    pub fn is_valid(&self) -> bool {
        !self.api_key.is_empty() && self.enabled
    }

    /// Get timeout duration
    pub fn get_timeout(&self) -> Duration {
        Duration::from_secs(self.timeout.unwrap_or(30) as u64)
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
}

/// VLM Service
pub struct VlmService {
    db_path: String,
}

impl VlmService {
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }

    /// Get database connection
    fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path).map_err(|e| anyhow!("Failed to open database: {}", e))
    }

    /// Save VLM configuration
    pub fn save_config(&self, config: VlmConfig) -> Result<()> {
        let conn = self.get_connection()?;

        // Save to vlm_config table
        conn.execute(
            "INSERT OR REPLACE INTO vlm_config (
                id, provider, api_key, base_url, model, enabled,
                max_image_size, image_quality, timeout, updated_at
            ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now', 'localtime'))",
            rusqlite::params![
                config.provider,
                config.api_key,
                config.base_url,
                config.model,
                config.enabled as i32,
                config.max_image_size,
                config.image_quality,
                config.timeout,
            ],
        ).map_err(|e| anyhow!("Failed to save VLM config: {}", e))?;

        Ok(())
    }

    /// Get VLM configuration
    pub fn get_config(&self) -> Result<Option<VlmConfigResponse>> {
        let conn = self.get_connection()?;

        let result = conn.query_row(
            "SELECT provider, api_key, base_url, model, enabled,
                    max_image_size, image_quality, timeout
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
                    max_image_size, image_quality, timeout
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
        let test_image = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="; // 1x1 transparent image

        match self.analyze_image_internal(&config, test_image, test_prompt).await {
            Ok(_) => Ok(true),
            Err(e) => Err(anyhow!("Connection test failed: {}", e)),
        }
    }

    /// Analyze image
    pub async fn analyze_image(&self, image_base64: &str, prompt: &str) -> Result<String> {
        let config = self.get_full_config()?;

        if !config.is_valid() {
            return Err(anyhow!("VLM is not enabled or configuration is invalid"));
        }

        self.analyze_image_internal(&config, image_base64, prompt).await
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
            "max_tokens": 1000
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

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Unable to parse response content"))?;

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
            "max_tokens": 1000,
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

        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Unable to parse response content"))?;

        Ok(content.to_string())
    }
}
