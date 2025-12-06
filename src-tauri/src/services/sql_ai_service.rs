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

/// 业务场景识别结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessSceneResult {
    pub template_id: i64,
    pub scene: String,           // 业务场景名称
    pub confidence: f32,         // 置信度
    pub description: String,     // 场景描述
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

    /// 识别SQL模板的业务场景
    pub async fn identify_business_scenes(
        &self,
        templates: Vec<(i64, String)>  // (template_id, template_text)
    ) -> Result<Vec<BusinessSceneResult>> {
        if self.config.api_key.is_empty() {
            return Err(anyhow!("API Key 未配置"));
        }

        if templates.is_empty() {
            return Ok(vec![]);
        }

        let prompt = self.build_scene_prompt(&templates);
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

        // 解析 AI 返回的场景识别结果
        Ok(self.parse_scene_response(&content))
    }

    /// 构建场景识别提示词
    fn build_scene_prompt(&self, templates: &[(i64, String)]) -> String {
        let templates_json: Vec<serde_json::Value> = templates
            .iter()
            .map(|(id, sql)| {
                serde_json::json!({
                    "id": id,
                    "sql": sql.chars().take(1000).collect::<String>()
                })
            })
            .collect();

        format!(
            r#"你是一个 SQL 分析专家。请分析以下 SQL 模板，识别每个模板的业务场景。

常见业务场景包括：
- 用户管理：涉及用户、账号、权限、角色、登录、注册等相关操作
- 订单处理：涉及订单、购物、交易、购买、结算等订单相关操作
- 数据统计：包含 COUNT、SUM、AVG、MAX、MIN 等聚合函数，用于统计分析和报表
- 库存管理：涉及库存、商品、产品、仓库、入库、出库等库存相关操作
- 日志记录：涉及日志、操作记录、审计、追踪等记录类操作
- 配置管理：涉及配置、设置、参数、系统配置等配置相关操作
- 支付相关：涉及支付、金额、账单、交易、退款等支付相关操作
- 数据查询：主要是 SELECT 查询操作，不属于上述特定业务场景
- 数据修改：主要是 INSERT、UPDATE、DELETE 等修改操作，不属于上述特定业务场景
- 其他：无法归类到上述场景的 SQL 操作

分析要求：
1. 根据 SQL 中的表名、字段名、操作类型来判断业务场景
2. 置信度范围 0.0-1.0，越接近 1.0 表示越确定
3. 描述要简洁明了，说明该 SQL 的主要功能
4. 如果 SQL 包含多个业务场景特征，选择最主要的场景
5. 对于通用的 CRUD 操作，根据具体表名和字段判断业务场景

请以 JSON 格式返回结果，不要包含任何其他文字：
[
  {{"id": 1, "scene": "用户管理", "confidence": 0.95, "description": "查询用户登录信息"}},
  {{"id": 2, "scene": "订单处理", "confidence": 0.90, "description": "创建新订单并初始化状态"}},
  {{"id": 3, "scene": "数据统计", "confidence": 0.98, "description": "统计每日订单总额"}}
]

以下是需要分析的 SQL 模板：
{templates}
"#,
            templates = serde_json::to_string_pretty(&templates_json).unwrap_or_default()
        )
    }

    /// 解析场景识别响应
    fn parse_scene_response(&self, response: &str) -> Vec<BusinessSceneResult> {
        // 尝试提取 JSON 数组
        let json_str = if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        #[derive(Deserialize)]
        struct RawSceneResult {
            id: i64,
            scene: String,
            #[serde(default = "default_confidence")]
            confidence: f32,
            #[serde(default)]
            description: String,
        }

        fn default_confidence() -> f32 {
            0.5
        }

        match serde_json::from_str::<Vec<RawSceneResult>>(json_str) {
            Ok(raw_results) => {
                raw_results
                    .into_iter()
                    .map(|r| BusinessSceneResult {
                        template_id: r.id,
                        scene: r.scene,
                        confidence: r.confidence.clamp(0.0, 1.0), // 确保置信度在有效范围内
                        description: r.description,
                    })
                    .collect()
            }
            Err(e) => {
                eprintln!("解析场景识别结果失败: {} - 原始内容: {}", e, response);
                vec![] // 返回空结果而不是错误，保证调用方能继续执行
            }
        }
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

    #[test]
    fn test_build_scene_prompt() {
        let config = SqlAiConfig::default();
        let service = SqlAiService::new(config);

        let templates = vec![
            (1, "SELECT * FROM users WHERE username = ?".to_string()),
            (2, "INSERT INTO orders (user_id, total) VALUES (?, ?)".to_string()),
            (3, "SELECT COUNT(*) FROM sales WHERE date > ?".to_string()),
        ];

        let prompt = service.build_scene_prompt(&templates);

        // 验证提示词包含关键信息
        assert!(prompt.contains("SQL 分析专家"));
        assert!(prompt.contains("用户管理"));
        assert!(prompt.contains("订单处理"));
        assert!(prompt.contains("数据统计"));
        assert!(prompt.contains("SELECT * FROM users"));
        assert!(prompt.contains("INSERT INTO orders"));
        assert!(prompt.contains("COUNT"));
    }

    #[test]
    fn test_parse_scene_response() {
        let config = SqlAiConfig::default();
        let service = SqlAiService::new(config);

        let content = r#"[
            {"id": 1, "scene": "用户管理", "confidence": 0.95, "description": "查询用户登录信息"},
            {"id": 2, "scene": "订单处理", "confidence": 0.90, "description": "创建新订单"},
            {"id": 3, "scene": "数据统计", "confidence": 0.98, "description": "统计销售总额"}
        ]"#;

        let results = service.parse_scene_response(content);

        assert_eq!(results.len(), 3);

        assert_eq!(results[0].template_id, 1);
        assert_eq!(results[0].scene, "用户管理");
        assert_eq!(results[0].confidence, 0.95);
        assert_eq!(results[0].description, "查询用户登录信息");

        assert_eq!(results[1].template_id, 2);
        assert_eq!(results[1].scene, "订单处理");
        assert_eq!(results[1].confidence, 0.90);

        assert_eq!(results[2].scene, "数据统计");
        assert_eq!(results[2].confidence, 0.98);
    }

    #[test]
    fn test_parse_scene_response_with_extra_text() {
        let config = SqlAiConfig::default();
        let service = SqlAiService::new(config);

        // 测试 AI 返回的内容包含额外文字的情况
        let content = r#"根据分析，以下是识别结果：
        [
            {"id": 1, "scene": "用户管理", "confidence": 0.95, "description": "查询用户信息"}
        ]
        以上是分析结果。"#;

        let results = service.parse_scene_response(content);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].scene, "用户管理");
    }

    #[test]
    fn test_parse_scene_response_missing_fields() {
        let config = SqlAiConfig::default();
        let service = SqlAiService::new(config);

        // 测试缺少 confidence 和 description 字段的情况
        let content = r#"[
            {"id": 1, "scene": "用户管理"}
        ]"#;

        let results = service.parse_scene_response(content);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].scene, "用户管理");
        assert_eq!(results[0].confidence, 0.5); // 默认值
        assert_eq!(results[0].description, ""); // 默认值
    }

    #[test]
    fn test_parse_scene_response_invalid_json() {
        let config = SqlAiConfig::default();
        let service = SqlAiService::new(config);

        let content = "这不是有效的 JSON";

        let results = service.parse_scene_response(content);

        // 应该返回空数组而不是崩溃
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_parse_scene_response_confidence_clamping() {
        let config = SqlAiConfig::default();
        let service = SqlAiService::new(config);

        // 测试置信度超出范围的情况
        let content = r#"[
            {"id": 1, "scene": "用户管理", "confidence": 1.5, "description": "测试"},
            {"id": 2, "scene": "订单处理", "confidence": -0.5, "description": "测试"}
        ]"#;

        let results = service.parse_scene_response(content);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].confidence, 1.0); // 被限制在 1.0
        assert_eq!(results[1].confidence, 0.0); // 被限制在 0.0
    }
}
