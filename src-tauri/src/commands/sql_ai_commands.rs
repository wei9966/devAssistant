use crate::db::connection::DbConnection;
use crate::services::ai_service::AiService;
use crate::services::sql_ai_service::{
    AiProvider, CategoryWithPrompt, SqlAiConfig, SqlAiService, SqlClassifyResult,
};
use crate::services::sql_service::SqlService;
use std::sync::Mutex;
use tauri::State;

/// AI 服务状态
pub struct SqlAiState(pub Mutex<Option<SqlAiService>>);

impl SqlAiState {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

/// 配置 AI 服务
///
/// # 废弃说明
/// 此命令已废弃，请使用全局 AI 配置（通过 ai_commands 模块）
#[deprecated(note = "请使用全局 AI 配置，通过 ai_commands::save_ai_config")]
#[tauri::command]
pub fn configure_sql_ai(
    ai_state: State<SqlAiState>,
    provider: String,
    api_key: String,
    base_url: Option<String>,
    model: Option<String>,
) -> Result<(), String> {
    let provider = match provider.to_lowercase().as_str() {
        "deepseek" => AiProvider::DeepSeek,
        "qwen" => AiProvider::Qwen,
        _ => return Err(format!("不支持的 AI 提供商: {}", provider)),
    };

    let config = SqlAiConfig {
        provider,
        api_key,
        base_url,
        model,
    };

    let service = SqlAiService::new(config);
    let mut state = ai_state.0.lock().map_err(|e| e.to_string())?;
    *state = Some(service);

    Ok(())
}

/// 获取当前 AI 配置状态
///
/// # 废弃说明
/// 此命令已废弃，请使用全局 AI 配置（通过 ai_commands 模块）
#[deprecated(note = "请使用全局 AI 配置，通过 ai_commands::is_ai_enabled")]
#[tauri::command]
pub fn get_sql_ai_status(ai_state: State<SqlAiState>) -> Result<bool, String> {
    let state = ai_state.0.lock().map_err(|e| e.to_string())?;
    Ok(state.is_some())
}

/// 测试 AI 连接
///
/// # 废弃说明
/// 此命令已废弃，请使用全局 AI 配置（通过 ai_commands 模块）
#[deprecated(note = "请使用全局 AI 配置，通过 ai_commands::test_ai_connection")]
#[tauri::command]
pub async fn test_sql_ai_connection(ai_state: State<'_, SqlAiState>) -> Result<bool, String> {
    // 在单独的作用域中获取配置，确保 MutexGuard 在 await 之前被释放
    let config = {
        let state = ai_state.0.lock().map_err(|e| e.to_string())?;
        let service = state.as_ref().ok_or("AI 服务未配置")?;

        SqlAiConfig {
            provider: match service.config.provider {
                AiProvider::DeepSeek => AiProvider::DeepSeek,
                AiProvider::Qwen => AiProvider::Qwen,
            },
            api_key: service.config.api_key.clone(),
            base_url: service.config.base_url.clone(),
            model: service.config.model.clone(),
        }
    }; // MutexGuard 在这里被释放

    let temp_service = SqlAiService::new(config);
    temp_service
        .test_connection()
        .await
        .map_err(|e| e.to_string())
}

/// 使用 AI 自动分类 SQL
#[tauri::command]
pub async fn ai_classify_sqls(
    db: State<'_, DbConnection>,
    sql_ids: Option<Vec<i64>>,
    limit: Option<usize>,
) -> Result<Vec<SqlClassifyResult>, String> {
    // 从数据库加载全局 AI 配置
    let config = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let ai_config = AiService::load_config(&conn).map_err(|e| e.to_string())?;

        if !ai_config.enabled {
            return Err("AI 功能未启用，请在设置中启用 AI".to_string());
        }

        if ai_config.api_key.is_empty() {
            return Err("AI 未配置，请先在设置中配置 AI".to_string());
        }

        // 转换为 SqlAiConfig
        SqlAiConfig {
            provider: match ai_config.provider {
                crate::services::ai_service::AiProvider::DeepSeek => AiProvider::DeepSeek,
                crate::services::ai_service::AiProvider::Qwen => AiProvider::Qwen,
            },
            api_key: ai_config.api_key,
            base_url: ai_config.base_url,
            model: ai_config.model,
        }
    };

    // 获取待分类的 SQL 列表
    let sqls: Vec<(i64, String)> = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;

        if let Some(ids) = sql_ids {
            // 获取指定 ID 的 SQL
            let mut result = Vec::new();
            for id in ids {
                if let Ok(records) = SqlService::get_recent_sqls(&conn, 1000) {
                    if let Some(record) = records.iter().find(|r| r.id == Some(id)) {
                        result.push((id, record.sql_text.clone()));
                    }
                }
            }
            result
        } else {
            // 获取未分类的 SQL
            let limit = limit.unwrap_or(20);
            SqlService::get_uncategorized_sqls(&conn, limit)
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter_map(|r| r.id.map(|id| (id, r.sql_text)))
                .collect()
        }
    };

    if sqls.is_empty() {
        return Ok(vec![]);
    }

    // 获取所有分类（包含 AI 提示词）
    let categories: Vec<CategoryWithPrompt> = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        SqlService::get_all_categories(&conn)
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter_map(|c| {
                c.id.map(|id| CategoryWithPrompt {
                    id,
                    name: c.name,
                    ai_prompt: c.ai_prompt,
                })
            })
            .collect()
    };

    // 调用 AI 进行分类
    let temp_service = SqlAiService::new(config);
    let results = temp_service
        .classify_sqls(sqls, categories.clone())
        .await
        .map_err(|e| e.to_string())?;

    // 更新数据库（支持多标签）
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;

        // 获取分类 ID 映射
        let category_map: std::collections::HashMap<String, i64> =
            categories.iter().map(|c| (c.name.clone(), c.id)).collect();

        for result in &results {
            // 更新名称
            SqlService::update_sql_name(&conn, result.sql_id, Some(&result.name))
                .map_err(|e| e.to_string())?;

            // 设置多个分类标签
            let category_ids: Vec<i64> = result
                .categories
                .iter()
                .filter_map(|name| category_map.get(name).copied())
                .collect();

            if !category_ids.is_empty() {
                SqlService::set_sql_categories(&conn, result.sql_id, &category_ids)
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(results)
}

/// 手动更新单条 SQL 的分类（支持多标签）
#[tauri::command]
pub fn manual_classify_sql(
    db: State<DbConnection>,
    sql_id: i64,
    name: Option<String>,
    category_ids: Option<Vec<i64>>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 更新名称
    if let Some(n) = &name {
        SqlService::update_sql_name(&conn, sql_id, Some(n)).map_err(|e| e.to_string())?;
    }

    // 设置分类标签
    if let Some(ids) = category_ids {
        SqlService::set_sql_categories(&conn, sql_id, &ids).map_err(|e| e.to_string())?;
    }

    Ok(())
}
