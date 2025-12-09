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

/// 使用 AI 自动分类 SQL
#[tauri::command]
pub async fn ai_classify_sqls(
    db: State<'_, DbConnection>,
    #[allow(non_snake_case)]
    sqlIds: Option<Vec<i64>>,
    limit: Option<usize>,
) -> Result<Vec<SqlClassifyResult>, String> {
    println!("=== ai_classify_sqls 命令被调用 ===");
    println!("参数: sqlIds={:?}, limit={:?}", sqlIds, limit);

    // 从数据库加载全局 AI 配置
    let config = {
        let conn = db.0.lock().map_err(|e| {
            println!("获取数据库连接失败: {}", e);
            e.to_string()
        })?;

        println!("正在加载 AI 配置...");
        let ai_config = AiService::load_config(&conn).map_err(|e| {
            println!("加载 AI 配置失败: {}", e);
            e.to_string()
        })?;
        println!("AI 配置加载成功: enabled={}, provider={:?}", ai_config.enabled, ai_config.provider);

        if !ai_config.enabled {
            println!("AI 功能未启用");
            return Err("AI 功能未启用，请在设置中启用 AI".to_string());
        }

        if ai_config.api_key.is_empty() {
            println!("AI API Key 为空");
            return Err("AI 未配置，请先在设置中配置 AI".to_string());
        }

        println!("AI 配置验证通过");

        // 转换为 SqlAiConfig
        SqlAiConfig {
            provider: match ai_config.provider {
                crate::services::ai_service::AiProvider::DeepSeek => AiProvider::DeepSeek,
                crate::services::ai_service::AiProvider::Qwen => AiProvider::Qwen,
                crate::services::ai_service::AiProvider::Custom => AiProvider::Custom,
            },
            api_key: ai_config.api_key,
            base_url: ai_config.base_url,
            model: ai_config.model,
        }
    };

    // 获取待分类的 SQL 列表
    println!("开始获取待分类的 SQL 列表...");
    let sqls: Vec<(i64, String)> = {
        let conn = db.0.lock().map_err(|e| {
            println!("获取数据库连接失败(sqls): {}", e);
            e.to_string()
        })?;

        if let Some(ids) = sqlIds {
            println!("获取指定 ID 的 SQL: {:?}", ids);
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
            println!("获取未分类的 SQL, limit={}", limit);
            let uncategorized = SqlService::get_uncategorized_sqls(&conn, limit)
                .map_err(|e| {
                    println!("获取未分类SQL失败: {}", e);
                    e.to_string()
                })?;
            println!("获取到 {} 条未分类SQL", uncategorized.len());
            uncategorized
                .into_iter()
                .filter_map(|r| r.id.map(|id| (id, r.sql_text)))
                .collect()
        }
    };

    println!("待分类SQL数量: {}", sqls.len());
    if sqls.is_empty() {
        println!("没有待分类的SQL，返回空结果");
        return Ok(vec![]);
    }

    // 获取所有分类（包含 AI 提示词）
    println!("开始获取分类列表...");
    let categories: Vec<CategoryWithPrompt> = {
        let conn = db.0.lock().map_err(|e| {
            println!("获取数据库连接失败(categories): {}", e);
            e.to_string()
        })?;
        let cats = SqlService::get_all_categories(&conn)
            .map_err(|e| {
                println!("获取分类列表失败: {}", e);
                e.to_string()
            })?;
        println!("获取到 {} 个分类", cats.len());
        cats.into_iter()
            .filter_map(|c| {
                c.id.map(|id| CategoryWithPrompt {
                    id,
                    name: c.name,
                    ai_prompt: c.ai_prompt,
                })
            })
            .collect()
    };
    println!("有效分类数量: {}", categories.len());

    // 调用 AI 进行分类
    println!("开始调用 AI 进行分类...");
    let temp_service = SqlAiService::new(config);
    let results = temp_service
        .classify_sqls(sqls, categories.clone())
        .await
        .map_err(|e| {
            println!("AI 分类失败: {}", e);
            e.to_string()
        })?;
    println!("AI 分类完成，结果数量: {}", results.len());

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
    #[allow(non_snake_case)]
    sqlId: i64,
    name: Option<String>,
    #[allow(non_snake_case)]
    categoryIds: Option<Vec<i64>>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 更新名称
    if let Some(n) = &name {
        SqlService::update_sql_name(&conn, sqlId, Some(n)).map_err(|e| e.to_string())?;
    }

    // 设置分类标签
    if let Some(ids) = categoryIds {
        SqlService::set_sql_categories(&conn, sqlId, &ids).map_err(|e| e.to_string())?;
    }

    Ok(())
}
