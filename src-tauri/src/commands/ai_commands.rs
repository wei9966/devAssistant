use crate::db::connection::DbConnection;
use crate::services::ai_service::{
    AiConfig, AiLog, AiLogQuery, AiLogStats, AiProvider as ServiceAiProvider, AiService,
    AppClassifyInput, AppClassifyResult, TaskClassifyResult, TaskSummaryInput, WorkflowRecommendation,
    ActivitySummaryInput,
};
use crate::services::work_log_service::WorkLogService;
use std::sync::Mutex;
use tauri::State;

/// AI 状态管理
pub struct AiState(pub Mutex<Option<AiService>>);

impl AiState {
    pub fn new() -> Self {
        AiState(Mutex::new(None))
    }
}

/// AI 配置响应（不返回完整 API Key）
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfigResponse {
    pub provider: String,
    pub api_key_preview: String, // 只显示部分 API Key
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub enabled: bool,
    pub max_tokens: Option<u32>,
}

/// AI 提供商枚举
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AiProvider {
    Claude,
    DeepSeek,
    Qwen,
    Custom,
}

impl AiProvider {
    fn from_string(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(AiProvider::Claude),
            "deepseek" => Ok(AiProvider::DeepSeek),
            "qwen" => Ok(AiProvider::Qwen),
            "custom" => Ok(AiProvider::Custom),
            _ => Err(format!("不支持的 AI 提供商: {}", s)),
        }
    }

    fn to_string(&self) -> String {
        match self {
            AiProvider::Claude => "claude".to_string(),
            AiProvider::DeepSeek => "deepseek".to_string(),
            AiProvider::Qwen => "qwen".to_string(),
            AiProvider::Custom => "custom".to_string(),
        }
    }
}

/// 保存 AI 配置
#[tauri::command]
pub async fn save_ai_config(
    db: State<'_, DbConnection>,
    ai_state: State<'_, AiState>,
    provider: String,
    api_key: String,
    base_url: Option<String>,
    model: Option<String>,
    enabled: bool,
    max_tokens: Option<u32>,
) -> Result<(), String> {
    // 验证 provider
    let provider_enum = AiProvider::from_string(&provider)?;

    // 构建配置 JSON
    let config_json = serde_json::json!({
        "provider": provider,
        "api_key": api_key,
        "base_url": base_url,
        "model": model,
        "enabled": enabled,
        "max_tokens": max_tokens,
    });

    // 保存到数据库
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('ai_config', ?1, ?2)",
            rusqlite::params![config_json.to_string(), now],
        )
        .map_err(|e| format!("保存 AI 配置失败: {}", e))?;
    }

    // 如果启用，更新内存中的 AI 服务
    if enabled {
        let service_provider = match provider_enum {
            AiProvider::DeepSeek => ServiceAiProvider::DeepSeek,
            AiProvider::Qwen => ServiceAiProvider::Qwen,
            AiProvider::Claude => ServiceAiProvider::DeepSeek, // Claude 暂不支持，使用 DeepSeek
            AiProvider::Custom => ServiceAiProvider::Custom,
        };

        let ai_config = AiConfig {
            provider: service_provider,
            api_key: api_key.clone(),
            base_url: base_url.clone(),
            model: model.clone(),
            enabled: true,
            max_tokens,
        };

        let service = AiService::new(ai_config);
        let mut state = ai_state.0.lock().map_err(|e| e.to_string())?;
        *state = Some(service);
    } else {
        // 如果禁用，清空内存中的服务
        let mut state = ai_state.0.lock().map_err(|e| e.to_string())?;
        *state = None;
    }

    Ok(())
}

/// 获取 AI 配置（不返回完整 API Key，只返回部分）
#[tauri::command]
pub fn get_ai_config(db: State<'_, DbConnection>) -> Result<Option<AiConfigResponse>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let config_json: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'ai_config'",
            [],
            |row| row.get(0),
        )
        .ok();

    if let Some(json) = config_json {
        let config: serde_json::Value =
            serde_json::from_str(&json).map_err(|e| format!("解析 AI 配置失败: {}", e))?;

        let api_key = config["api_key"].as_str().unwrap_or("");
        let api_key_preview = if api_key.len() > 8 {
            format!("{}...{}", &api_key[..4], &api_key[api_key.len() - 4..])
        } else {
            "***".to_string()
        };

        Ok(Some(AiConfigResponse {
            provider: config["provider"].as_str().unwrap_or("claude").to_string(),
            api_key_preview,
            base_url: config["base_url"].as_str().map(|s| s.to_string()),
            model: config["model"].as_str().map(|s| s.to_string()),
            enabled: config["enabled"].as_bool().unwrap_or(false),
            max_tokens: config["max_tokens"].as_u64().map(|n| n as u32),
        }))
    } else {
        Ok(None)
    }
}

/// 测试 AI 连接
#[tauri::command]
pub async fn test_ai_connection(ai_state: State<'_, AiState>) -> Result<bool, String> {
    // 在单独的作用域中获取配置，确保 MutexGuard 在 await 之前被释放
    let config = {
        let state = ai_state.0.lock().map_err(|e| e.to_string())?;
        let service = state.as_ref().ok_or("AI 服务未配置")?;
        service.config.clone()
    }; // MutexGuard 在这里被释放

    // 创建临时服务进行测试
    let temp_service = AiService::new(config);
    temp_service
        .test_connection()
        .await
        .map_err(|e| e.to_string())
}

/// 检查 AI 是否已配置且启用
#[tauri::command]
pub fn is_ai_enabled(db: State<'_, DbConnection>) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let config_json: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'ai_config'",
            [],
            |row| row.get(0),
        )
        .ok();

    if let Some(json) = config_json {
        let config: serde_json::Value =
            serde_json::from_str(&json).map_err(|e| format!("解析 AI 配置失败: {}", e))?;

        Ok(config["enabled"].as_bool().unwrap_or(false))
    } else {
        Ok(false)
    }
}

/// AI 生成工作日志
#[tauri::command]
pub async fn ai_generate_work_log(
    db: State<'_, DbConnection>,
    date: String,
    completed_tasks: Vec<String>,
    executed_sqls: Vec<String>,
    git_commits: Vec<String>,
) -> Result<String, String> {
    use crate::services::ai_service::WorkLogInput;

    // 尝试获取屏幕活动摘要
    let activity_summary = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        match WorkLogService::get_day_context_summary(&conn, &date) {
            Ok(summary) => {
                // 检查是否有有效的活动数据
                if summary.active_time_range.total_minutes > 0 {
                    // 转换为 ActivitySummaryInput
                    let time_range = format!(
                        "{} - {}",
                        summary.active_time_range.start,
                        summary.active_time_range.end
                    );

                    // 获取前5个应用
                    let top_apps: Vec<String> = summary.app_usage
                        .iter()
                        .take(5)
                        .map(|app| app.app_name.clone())
                        .collect();

                    // 构建活动类型分布描述
                    let dist = &summary.activity_distribution;
                    let mut dist_parts = vec![];
                    if dist.coding > 0 {
                        dist_parts.push(format!("编程{}分钟", dist.coding));
                    }
                    if dist.browsing > 0 {
                        dist_parts.push(format!("浏览{}分钟", dist.browsing));
                    }
                    if dist.document > 0 {
                        dist_parts.push(format!("文档{}分钟", dist.document));
                    }
                    if dist.meeting > 0 {
                        dist_parts.push(format!("会议{}分钟", dist.meeting));
                    }
                    if dist.communication > 0 {
                        dist_parts.push(format!("沟通{}分钟", dist.communication));
                    }
                    if dist.other > 0 {
                        dist_parts.push(format!("其他{}分钟", dist.other));
                    }
                    let activity_distribution = dist_parts.join("、");

                    // 获取关键活动描述
                    let key_activities: Vec<String> = summary.key_activities
                        .iter()
                        .take(5)
                        .map(|act| format!("[{}] {} ({})", act.time, act.description, act.app_name))
                        .collect();

                    Some(ActivitySummaryInput {
                        time_range,
                        total_minutes: summary.active_time_range.total_minutes,
                        top_apps,
                        activity_distribution,
                        key_activities,
                    })
                } else {
                    None
                }
            }
            Err(_) => None, // 如果获取失败，继续使用原有逻辑
        }
    };

    let (config, provider_str, model_str) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let cfg = AiService::load_config(&conn).map_err(|e| e.to_string())?;
        let provider = cfg.provider.to_string();
        let model = cfg.get_model().to_string();
        (cfg, provider, model)
    };

    if !config.enabled {
        return Err("AI 功能未启用".to_string());
    }

    let service = AiService::new(config);
    let input = WorkLogInput {
        date: date.clone(),
        completed_tasks: completed_tasks.clone(),
        executed_sqls: executed_sqls.clone(),
        git_commits: git_commits.clone(),
        activity_summary: activity_summary.clone(),
    };

    let has_activity = activity_summary.is_some();
    let prompt = format!(
        "生成工作日志 - 日期: {}, 任务数: {}, SQL数: {}, 提交数: {}, 含活动摘要: {}",
        date, completed_tasks.len(), executed_sqls.len(), git_commits.len(), has_activity
    );

    let start = std::time::Instant::now();
    let result = service.generate_work_log(input).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // 记录日志
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let (status, response, error_msg) = match &result {
            Ok(resp) => ("success", Some(resp.as_str()), None),
            Err(e) => ("error", None, Some(e.to_string())),
        };
        let _ = AiService::save_log(
            &conn,
            "work_log",
            "generate",
            &provider_str,
            Some(&model_str),
            &prompt,
            response,
            None,
            Some(duration_ms),
            status,
            error_msg.as_deref(),
        );
    }

    result.map_err(|e| e.to_string())
}

/// AI 润色工作日志
#[tauri::command]
pub async fn ai_polish_work_log(
    db: State<'_, DbConnection>,
    content: String,
) -> Result<String, String> {
    let (config, provider_str, model_str) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let cfg = AiService::load_config(&conn).map_err(|e| e.to_string())?;
        let provider = cfg.provider.to_string();
        let model = cfg.get_model().to_string();
        (cfg, provider, model)
    };

    if !config.enabled {
        return Err("AI 功能未启用".to_string());
    }

    let service = AiService::new(config);
    let prompt = format!("润色工作日志 (内容长度: {} 字符)", content.len());

    let start = std::time::Instant::now();
    let result = service.polish_work_log(&content).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // 记录日志
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let (status, response, error_msg) = match &result {
            Ok(resp) => ("success", Some(resp.as_str()), None),
            Err(e) => ("error", None, Some(e.to_string())),
        };
        let _ = AiService::save_log(
            &conn,
            "work_log",
            "polish",
            &provider_str,
            Some(&model_str),
            &prompt,
            response,
            None,
            Some(duration_ms),
            status,
            error_msg.as_deref(),
        );
    }

    result.map_err(|e| e.to_string())
}

/// 通用 AI 聊天接口
#[tauri::command]
pub async fn ai_chat(
    db: State<'_, DbConnection>,
    prompt: String,
    module: Option<String>,
) -> Result<String, String> {
    use crate::services::ai_service::ChatMessage;

    let (config, provider_str, model_str) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let cfg = AiService::load_config(&conn).map_err(|e| e.to_string())?;
        let provider = cfg.provider.to_string();
        let model = cfg.get_model().to_string();
        (cfg, provider, model)
    };

    if !config.enabled {
        return Err("AI 功能未启用".to_string());
    }

    let service = AiService::new(config);
    let messages = vec![ChatMessage::user(prompt.clone())];

    let start = std::time::Instant::now();
    let result = service.chat(messages).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // 记录日志
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let (status, response, error_msg) = match &result {
            Ok(resp) => ("success", Some(resp.as_str()), None),
            Err(e) => ("error", None, Some(e.to_string())),
        };
        let module_name = module.as_deref().unwrap_or("general");
        let _ = AiService::save_log(
            &conn,
            module_name,
            "chat",
            &provider_str,
            Some(&model_str),
            &prompt,
            response,
            None,
            Some(duration_ms),
            status,
            error_msg.as_deref(),
        );
    }

    result.map_err(|e| e.to_string())
}

/// AI 生成周报
#[tauri::command]
pub async fn ai_generate_weekly_report(
    db: State<'_, DbConnection>,
    logs: Vec<String>,
) -> Result<String, String> {
    let (config, provider_str, model_str) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let cfg = AiService::load_config(&conn).map_err(|e| e.to_string())?;
        let provider = cfg.provider.to_string();
        let model = cfg.get_model().to_string();
        (cfg, provider, model)
    };

    if !config.enabled {
        return Err("AI 功能未启用".to_string());
    }

    let service = AiService::new(config);
    let prompt = format!("生成周报 (日志数: {})", logs.len());

    let start = std::time::Instant::now();
    let result = service.generate_weekly_report(logs).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // 记录日志
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let (status, response, error_msg) = match &result {
            Ok(resp) => ("success", Some(resp.as_str()), None),
            Err(e) => ("error", None, Some(e.to_string())),
        };
        let _ = AiService::save_log(
            &conn,
            "work_log",
            "weekly_report",
            &provider_str,
            Some(&model_str),
            &prompt,
            response,
            None,
            Some(duration_ms),
            status,
            error_msg.as_deref(),
        );
    }

    result.map_err(|e| e.to_string())
}

/// AI 分类任务
#[tauri::command]
pub async fn ai_classify_task(
    db: State<'_, DbConnection>,
    title: String,
    description: Option<String>,
    existing_tags: Option<Vec<String>>,
) -> Result<TaskClassifyResult, String> {
    let (config, provider_str, model_str) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let cfg = AiService::load_config(&conn).map_err(|e| e.to_string())?;
        let provider = cfg.provider.to_string();
        let model = cfg.get_model().to_string();
        (cfg, provider, model)
    };

    if !config.enabled {
        return Err("AI 功能未启用".to_string());
    }

    let service = AiService::new(config);
    let prompt = format!("分类任务: {}", title);

    let start = std::time::Instant::now();
    let result = service.classify_task(&title, description.as_deref(), existing_tags.as_deref()).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // 记录日志
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let (status, response, error_msg) = match &result {
            Ok(resp) => ("success", Some(serde_json::to_string(resp).unwrap_or_default()), None),
            Err(e) => ("error", None, Some(e.to_string())),
        };
        let _ = AiService::save_log(
            &conn,
            "task",
            "classify",
            &provider_str,
            Some(&model_str),
            &prompt,
            response.as_deref(),
            None,
            Some(duration_ms),
            status,
            error_msg.as_deref(),
        );
    }

    result.map_err(|e| e.to_string())
}

/// AI 增强任务描述
#[tauri::command]
pub async fn ai_enhance_task_description(
    db: State<'_, DbConnection>,
    title: String,
    description: Option<String>,
) -> Result<String, String> {
    let (config, provider_str, model_str) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let cfg = AiService::load_config(&conn).map_err(|e| e.to_string())?;
        let provider = cfg.provider.to_string();
        let model = cfg.get_model().to_string();
        (cfg, provider, model)
    };

    if !config.enabled {
        return Err("AI 功能未启用".to_string());
    }

    let service = AiService::new(config);
    let prompt = format!("增强任务描述: {}", title);

    let start = std::time::Instant::now();
    let result = service.enhance_task_description(&title, description.as_deref()).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // 记录日志
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let (status, response, error_msg) = match &result {
            Ok(resp) => ("success", Some(resp.as_str()), None),
            Err(e) => ("error", None, Some(e.to_string())),
        };
        let _ = AiService::save_log(
            &conn,
            "task",
            "enhance_description",
            &provider_str,
            Some(&model_str),
            &prompt,
            response,
            None,
            Some(duration_ms),
            status,
            error_msg.as_deref(),
        );
    }

    result.map_err(|e| e.to_string())
}

/// AI 生成子任务
#[tauri::command]
pub async fn ai_generate_subtasks(
    db: State<'_, DbConnection>,
    title: String,
    description: Option<String>,
) -> Result<Vec<String>, String> {
    let (config, provider_str, model_str) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let cfg = AiService::load_config(&conn).map_err(|e| e.to_string())?;
        let provider = cfg.provider.to_string();
        let model = cfg.get_model().to_string();
        (cfg, provider, model)
    };

    if !config.enabled {
        return Err("AI 功能未启用".to_string());
    }

    let service = AiService::new(config);
    let prompt = format!("生成子任务: {}", title);

    let start = std::time::Instant::now();
    let result = service.generate_subtasks(&title, description.as_deref()).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // 记录日志
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let (status, response, error_msg) = match &result {
            Ok(resp) => ("success", Some(resp.join(", ")), None),
            Err(e) => ("error", None, Some(e.to_string())),
        };
        let _ = AiService::save_log(
            &conn,
            "task",
            "generate_subtasks",
            &provider_str,
            Some(&model_str),
            &prompt,
            response.as_deref(),
            None,
            Some(duration_ms),
            status,
            error_msg.as_deref(),
        );
    }

    result.map_err(|e| e.to_string())
}

/// AI 任务总结
#[tauri::command]
pub async fn ai_summarize_tasks(
    db: State<'_, DbConnection>,
    tasks: Vec<TaskSummaryInput>,
) -> Result<String, String> {
    let (config, provider_str, model_str) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let cfg = AiService::load_config(&conn).map_err(|e| e.to_string())?;
        let provider = cfg.provider.to_string();
        let model = cfg.get_model().to_string();
        (cfg, provider, model)
    };

    if !config.enabled {
        return Err("AI 功能未启用".to_string());
    }

    let service = AiService::new(config);
    let prompt = format!("任务总结 (任务数: {})", tasks.len());

    let start = std::time::Instant::now();
    let result = service.summarize_tasks(tasks).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // 记录日志
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let (status, response, error_msg) = match &result {
            Ok(resp) => ("success", Some(resp.as_str()), None),
            Err(e) => ("error", None, Some(e.to_string())),
        };
        let _ = AiService::save_log(
            &conn,
            "task",
            "summarize",
            &provider_str,
            Some(&model_str),
            &prompt,
            response,
            None,
            Some(duration_ms),
            status,
            error_msg.as_deref(),
        );
    }

    result.map_err(|e| e.to_string())
}

/// AI 分类应用
#[tauri::command]
pub async fn ai_classify_apps(
    db: State<'_, DbConnection>,
    apps: Vec<AppClassifyInput>,
    categories: Vec<String>,
) -> Result<Vec<AppClassifyResult>, String> {
    let (config, provider_str, model_str) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let cfg = AiService::load_config(&conn).map_err(|e| e.to_string())?;
        let provider = cfg.provider.to_string();
        let model = cfg.get_model().to_string();
        (cfg, provider, model)
    };

    if !config.enabled {
        return Err("AI 功能未启用".to_string());
    }

    let service = AiService::new(config);
    let prompt = format!("应用分类 (应用数: {}, 分类数: {})", apps.len(), categories.len());

    let start = std::time::Instant::now();
    let result = service.classify_apps(apps, categories).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // 记录日志
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let (status, response, error_msg) = match &result {
            Ok(resp) => ("success", Some(serde_json::to_string(resp).unwrap_or_default()), None),
            Err(e) => ("error", None, Some(e.to_string())),
        };
        let _ = AiService::save_log(
            &conn,
            "app_launcher",
            "classify_apps",
            &provider_str,
            Some(&model_str),
            &prompt,
            response.as_deref(),
            None,
            Some(duration_ms),
            status,
            error_msg.as_deref(),
        );
    }

    result.map_err(|e| e.to_string())
}

/// AI 推荐工作流
#[tauri::command]
pub async fn ai_recommend_workflows(
    db: State<'_, DbConnection>,
    apps: Vec<AppClassifyInput>,
    launch_history: Vec<(String, i64)>,
) -> Result<Vec<WorkflowRecommendation>, String> {
    let (config, provider_str, model_str) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let cfg = AiService::load_config(&conn).map_err(|e| e.to_string())?;
        let provider = cfg.provider.to_string();
        let model = cfg.get_model().to_string();
        (cfg, provider, model)
    };

    if !config.enabled {
        return Err("AI 功能未启用".to_string());
    }

    let service = AiService::new(config);
    let prompt = format!("推荐工作流 (应用数: {}, 历史记录数: {})", apps.len(), launch_history.len());

    let start = std::time::Instant::now();
    let result = service.recommend_workflows(apps, launch_history).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // 记录日志
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let (status, response, error_msg) = match &result {
            Ok(resp) => ("success", Some(serde_json::to_string(resp).unwrap_or_default()), None),
            Err(e) => ("error", None, Some(e.to_string())),
        };
        let _ = AiService::save_log(
            &conn,
            "app_launcher",
            "recommend_workflows",
            &provider_str,
            Some(&model_str),
            &prompt,
            response.as_deref(),
            None,
            Some(duration_ms),
            status,
            error_msg.as_deref(),
        );
    }

    result.map_err(|e| e.to_string())
}

/// AI 生成应用描述
#[tauri::command]
pub async fn ai_generate_app_description(
    db: State<'_, DbConnection>,
    app_name: String,
    app_path: String,
) -> Result<String, String> {
    let (config, provider_str, model_str) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let cfg = AiService::load_config(&conn).map_err(|e| e.to_string())?;
        let provider = cfg.provider.to_string();
        let model = cfg.get_model().to_string();
        (cfg, provider, model)
    };

    if !config.enabled {
        return Err("AI 功能未启用".to_string());
    }

    let service = AiService::new(config);
    let prompt = format!("生成应用描述: {}", app_name);

    let start = std::time::Instant::now();
    let result = service.generate_app_description(&app_name, &app_path).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // 记录日志
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let (status, response, error_msg) = match &result {
            Ok(resp) => ("success", Some(resp.as_str()), None),
            Err(e) => ("error", None, Some(e.to_string())),
        };
        let _ = AiService::save_log(
            &conn,
            "app_launcher",
            "generate_description",
            &provider_str,
            Some(&model_str),
            &prompt,
            response,
            None,
            Some(duration_ms),
            status,
            error_msg.as_deref(),
        );
    }

    result.map_err(|e| e.to_string())
}

// ========== AI 日志相关命令 ==========

/// 查询 AI 调用日志
#[tauri::command]
pub fn get_ai_logs(
    db: State<'_, DbConnection>,
    module: Option<String>,
    status: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<AiLog>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let query = AiLogQuery {
        module,
        status,
        limit,
        offset,
    };

    AiService::query_logs(&conn, &query).map_err(|e| e.to_string())
}

/// 获取 AI 调用统计
#[tauri::command]
pub fn get_ai_log_stats(db: State<'_, DbConnection>) -> Result<AiLogStats, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    AiService::get_log_stats(&conn).map_err(|e| e.to_string())
}

/// 清空 AI 日志
#[tauri::command]
pub fn clear_ai_logs(
    db: State<'_, DbConnection>,
    before_days: Option<i32>,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let before_timestamp = before_days.map(|days| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        now - (days as i64 * 24 * 60 * 60)
    });

    AiService::clear_logs(&conn, before_timestamp).map_err(|e| e.to_string())
}

/// 记录 AI 调用日志（手动记录）
#[tauri::command]
pub fn save_ai_log(
    db: State<'_, DbConnection>,
    module: String,
    action: String,
    provider: String,
    model: Option<String>,
    prompt: String,
    response: Option<String>,
    tokens_used: Option<i32>,
    duration_ms: Option<i64>,
    status: String,
    error_message: Option<String>,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    AiService::save_log(
        &conn,
        &module,
        &action,
        &provider,
        model.as_deref(),
        &prompt,
        response.as_deref(),
        tokens_used,
        duration_ms,
        &status,
        error_message.as_deref(),
    )
    .map_err(|e| e.to_string())
}
