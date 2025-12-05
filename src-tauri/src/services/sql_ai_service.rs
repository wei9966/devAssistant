use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

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

/// AI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlAiConfig {
    pub provider: AiProvider,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

impl Default for SqlAiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::DeepSeek,
            api_key: String::new(),
            base_url: None,
            model: None,
        }
    }
}

impl SqlAiConfig {
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
}

/// SQL 分类结果（支持多标签）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlClassifyResult {
    pub sql_id: i64,
    pub name: String,
    pub categories: Vec<String>, // 多个分类标签
    pub confidence: f32,
}

/// 分类信息（包含 AI 提示词）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryWithPrompt {
    pub id: i64,
    pub name: String,
    pub ai_prompt: Option<String>,
}

/// AI 请求消息
#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

/// AI 请求体
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
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

/// SQL AI 服务
pub struct SqlAiService {
    client: Client,
    pub config: SqlAiConfig,
}

impl SqlAiService {
    /// 创建新的 SQL AI 服务实例
    pub fn new(config: SqlAiConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// 更新配置
    pub fn update_config(&mut self, config: SqlAiConfig) {
        self.config = config;
    }

    /// 构建分类提示词（支持多标签和分类规则）
    fn build_classify_prompt(sqls: &[(i64, String)], categories: &[CategoryWithPrompt]) -> String {
        // 构建分类名称列表
        let category_names: Vec<&str> = categories.iter().map(|c| c.name.as_str()).collect();
        let categories_str = category_names.join("、");

        // 构建分类规则说明
        let category_rules: Vec<String> = categories
            .iter()
            .filter(|c| c.ai_prompt.is_some())
            .map(|c| format!("- {}：{}", c.name, c.ai_prompt.as_ref().unwrap()))
            .collect();
        let rules_str = if category_rules.is_empty() {
            String::new()
        } else {
            format!("\n\n各分类的识别规则：\n{}", category_rules.join("\n"))
        };

        let sqls_json: Vec<serde_json::Value> = sqls
            .iter()
            .map(|(id, sql)| {
                serde_json::json!({
                    "id": id,
                    "sql": sql.chars().take(500).collect::<String>()
                })
            })
            .collect();

        format!(
            r#"你是一个 SQL 分析专家。请分析以下 SQL 语句，为每条 SQL 生成一个简短的名称和分类。

可用的分类有：{categories}{rules}

请按照以下 JSON 格式返回结果，不要包含任何其他文字：
[
  {{"id": 1, "name": "查询用户列表", "categories": ["MySQL", "用户管理"]}},
  {{"id": 2, "name": "更新订单状态", "categories": ["SQLServer", "订单管理"]}}
]

名称要求：
1. 简短明了，不超过20个字
2. 描述 SQL 的主要功能
3. 使用中文

分类要求：
1. 每条 SQL 可以有多个分类标签（比如：数据库类型 + 业务分类）
2. 必须是给定分类中的一个或多个
3. 根据分类规则提示词进行判断（如果有的话）
4. 数据库类型分类尽量根据 SQL 语法特征判断
5. 如果无法确定某个维度的分类，可以不添加该分类

以下是需要分析的 SQL 语句：
{sqls}
"#,
            categories = categories_str,
            rules = rules_str,
            sqls = serde_json::to_string_pretty(&sqls_json).unwrap_or_default()
        )
    }

    /// 调用 AI API 进行分类（支持多标签和分类规则）
    pub async fn classify_sqls(
        &self,
        sqls: Vec<(i64, String)>,
        categories: Vec<CategoryWithPrompt>,
    ) -> Result<Vec<SqlClassifyResult>> {
        if self.config.api_key.is_empty() {
            return Err(anyhow!("API Key 未配置"));
        }

        if sqls.is_empty() {
            return Ok(vec![]);
        }

        let prompt = Self::build_classify_prompt(&sqls, &categories);
        let base_url = self.config.get_base_url();
        let model = self.config.get_model();

        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.3,
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
            .unwrap_or_default();

        // 解析 AI 返回的 JSON
        let category_names: Vec<String> = categories.iter().map(|c| c.name.clone()).collect();
        self.parse_classify_response(&content, &category_names)
    }

    /// 解析 AI 返回的分类结果（支持多标签）
    fn parse_classify_response(
        &self,
        content: &str,
        categories: &[String],
    ) -> Result<Vec<SqlClassifyResult>> {
        // 尝试提取 JSON 数组
        let json_str = if let Some(start) = content.find('[') {
            if let Some(end) = content.rfind(']') {
                &content[start..=end]
            } else {
                content
            }
        } else {
            content
        };

        #[derive(Deserialize)]
        struct RawResult {
            id: i64,
            name: String,
            #[serde(default)]
            categories: Vec<String>,
            // 兼容旧格式的单个分类
            #[serde(default)]
            category: Option<String>,
        }

        let raw_results: Vec<RawResult> = serde_json::from_str(json_str)
            .map_err(|e| anyhow!("解析 AI 返回结果失败: {} - 原始内容: {}", e, content))?;

        let results = raw_results
            .into_iter()
            .map(|r| {
                // 获取分类列表（兼容旧格式）
                let mut cats = r.categories;
                if cats.is_empty() {
                    if let Some(cat) = r.category {
                        cats.push(cat);
                    }
                }

                // 验证并过滤有效的分类
                let valid_categories: Vec<String> = cats
                    .into_iter()
                    .filter(|c| categories.contains(c))
                    .collect();

                SqlClassifyResult {
                    sql_id: r.id,
                    name: r.name,
                    categories: valid_categories,
                    confidence: 0.8,
                }
            })
            .collect();

        Ok(results)
    }

    /// 测试 API 连接
    pub async fn test_connection(&self) -> Result<bool> {
        if self.config.api_key.is_empty() {
            return Err(anyhow!("API Key 未配置"));
        }

        let base_url = self.config.get_base_url();
        let model = self.config.get_model();

        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "你好".to_string(),
            }],
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
            .map_err(|e| anyhow!("API 连接测试失败: {}", e))?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt() {
        let sqls = vec![
            (1, "SELECT * FROM users WHERE id = 1".to_string()),
            (
                2,
                "UPDATE orders SET status = 'done' WHERE id = 2".to_string(),
            ),
        ];
        let categories = vec![
            CategoryWithPrompt {
                id: 1,
                name: "MySQL".to_string(),
                ai_prompt: Some("snake_case 命名".to_string()),
            },
            CategoryWithPrompt {
                id: 2,
                name: "查询".to_string(),
                ai_prompt: None,
            },
            CategoryWithPrompt {
                id: 3,
                name: "更新".to_string(),
                ai_prompt: None,
            },
        ];

        let prompt = SqlAiService::build_classify_prompt(&sqls, &categories);
        assert!(prompt.contains("MySQL、查询、更新"));
        assert!(prompt.contains("SELECT * FROM users"));
        assert!(prompt.contains("snake_case 命名"));
    }

    #[test]
    fn test_parse_response_multi_categories() {
        let config = SqlAiConfig::default();
        let service = SqlAiService::new(config);

        let content = r#"[
            {"id": 1, "name": "查询用户信息", "categories": ["MySQL", "查询"]},
            {"id": 2, "name": "更新订单状态", "categories": ["SQLServer", "更新"]}
        ]"#;

        let categories = vec![
            "MySQL".to_string(),
            "SQLServer".to_string(),
            "查询".to_string(),
            "更新".to_string(),
        ];
        let results = service
            .parse_classify_response(content, &categories)
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "查询用户信息");
        assert_eq!(results[0].categories, vec!["MySQL", "查询"]);
        assert_eq!(results[1].categories, vec!["SQLServer", "更新"]);
    }

    #[test]
    fn test_parse_response_legacy_format() {
        let config = SqlAiConfig::default();
        let service = SqlAiService::new(config);

        // 测试兼容旧格式（单个 category）
        let content = r#"[
            {"id": 1, "name": "查询用户信息", "category": "查询"}
        ]"#;

        let categories = vec!["查询".to_string(), "更新".to_string()];
        let results = service
            .parse_classify_response(content, &categories)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].categories, vec!["查询"]);
    }
}
