use crate::db::connection::DbConnection;
use crate::services::ai_service::AiService;
use crate::services::sql_ai_service::{AiProvider, SqlAiConfig, SqlAiService};
use crate::services::sql_template_service::{ConsolidateResult, SqlRecord, SqlTemplate, SqlTemplateService};
use serde::{Deserialize, Serialize};
use tauri::State;

/// 分组统计数据
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupCount {
    pub name: String,
    pub count: i32,
}

/// 模板分组统计信息
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateGroupStats {
    pub by_scene: Vec<GroupCount>,
    pub by_type: Vec<GroupCount>,
    pub by_table: Vec<GroupCount>,
    pub total_templates: i32,
    pub total_variants: i32,
}

/// 1. 智能整合SQL历史，生成模板
#[tauri::command]
pub fn consolidate_sql_templates(db: State<DbConnection>) -> Result<ConsolidateResult, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlTemplateService::consolidate_sql_history(&conn).map_err(|e| e.to_string())
}

/// 2. 获取模板列表（支持分组）
#[tauri::command]
pub fn get_sql_templates(
    db: State<DbConnection>,
    group_by: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<SqlTemplate>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let group_by_str = group_by.as_deref();
    let limit_value = limit.unwrap_or(100);
    SqlTemplateService::get_templates(&conn, group_by_str, limit_value).map_err(|e| e.to_string())
}

/// 3. 获取热门模板
#[tauri::command]
pub fn get_hot_sql_templates(
    db: State<DbConnection>,
    limit: Option<i32>,
) -> Result<Vec<SqlTemplate>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let limit_value = limit.unwrap_or(50);
    SqlTemplateService::get_hot_templates(&conn, limit_value).map_err(|e| e.to_string())
}

/// 4. 获取模板详情及其变体SQL
#[tauri::command]
pub fn get_template_variants(
    db: State<DbConnection>,
    template_id: i64,
) -> Result<Vec<SqlRecord>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlTemplateService::get_template_variants(&conn, template_id).map_err(|e| e.to_string())
}

/// 5. 按表名获取模板
#[tauri::command]
pub fn get_templates_by_table(
    db: State<DbConnection>,
    table_name: String,
) -> Result<Vec<SqlTemplate>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlTemplateService::get_templates_by_table(&conn, &table_name).map_err(|e| e.to_string())
}

/// 6. AI识别业务场景
#[tauri::command]
pub async fn identify_template_scenes(
    db: State<'_, DbConnection>,
    template_ids: Option<Vec<i64>>,
) -> Result<i32, String> {
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
                crate::services::ai_service::AiProvider::Custom => AiProvider::Custom,
            },
            api_key: ai_config.api_key,
            base_url: ai_config.base_url,
            model: ai_config.model,
        }
    };

    // 获取待识别的模板列表
    let templates: Vec<(i64, String)> = if let Some(ids) = template_ids {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        // 获取指定 ID 的模板
        let mut result = Vec::new();
        for id in ids {
            let template = conn
                .query_row(
                    "SELECT template_text FROM sql_templates WHERE id = ?",
                    rusqlite::params![id],
                    |row| row.get::<_, String>(0),
                )
                .map_err(|e| e.to_string())?;
            result.push((id, template));
        }
        result
    } else {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        // 获取所有未设置业务场景的模板（最多100个）
        let mut stmt = conn
            .prepare(
                "SELECT id, template_text FROM sql_templates
                 WHERE business_scene IS NULL OR business_scene = ''
                 ORDER BY usage_count DESC LIMIT 100",
            )
            .map_err(|e| e.to_string())?;

        let result: Vec<(i64, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        result
    };

    if templates.is_empty() {
        return Ok(0);
    }

    // 调用 AI 进行业务场景识别
    let ai_service = SqlAiService::new(config);
    let results = ai_service
        .identify_business_scenes(templates)
        .await
        .map_err(|e| e.to_string())?;

    // 更新数据库中的业务场景
    let updated_count = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let mut count = 0;

        for result in results {
            if let Err(e) = SqlTemplateService::update_template_scene(
                &conn,
                result.template_id,
                &result.scene,
            ) {
                log::warn!(
                    "更新模板 {} 的业务场景失败: {}",
                    result.template_id,
                    e
                );
            } else {
                count += 1;
            }
        }

        count
    };

    Ok(updated_count)
}

/// 7. 切换模板收藏状态
#[tauri::command]
pub fn toggle_template_favorite(
    db: State<DbConnection>,
    template_id: i64,
) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    SqlTemplateService::toggle_template_favorite(&conn, template_id).map_err(|e| e.to_string())
}

/// 8. 获取分组统计信息
#[tauri::command]
pub fn get_template_group_stats(db: State<DbConnection>) -> Result<TemplateGroupStats, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 按业务场景分组统计
    let by_scene: Vec<GroupCount> = {
        let mut stmt = conn
            .prepare(
                "SELECT business_scene, COUNT(*) as count
                 FROM sql_templates
                 WHERE business_scene IS NOT NULL AND business_scene != ''
                 GROUP BY business_scene
                 ORDER BY count DESC",
            )
            .map_err(|e| e.to_string())?;

        let result: Vec<GroupCount> = stmt
            .query_map([], |row| {
                Ok(GroupCount {
                    name: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        result
    };

    // 按SQL类型分组统计
    let by_type: Vec<GroupCount> = {
        let mut stmt = conn
            .prepare(
                "SELECT sql_type, COUNT(*) as count
                 FROM sql_templates
                 WHERE sql_type IS NOT NULL
                 GROUP BY sql_type
                 ORDER BY count DESC",
            )
            .map_err(|e| e.to_string())?;

        let result: Vec<GroupCount> = stmt
            .query_map([], |row| {
                Ok(GroupCount {
                    name: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        result
    };

    // 按表名分组统计（提取表名列表中的每个表）
    let all_templates: Vec<(i64, String)> = {
        let mut stmt = conn
            .prepare("SELECT id, table_names FROM sql_templates")
            .map_err(|e| e.to_string())?;

        let result: Vec<(i64, String)> = stmt
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        result
    };

    // 统计每个表名的出现次数
    let mut table_counts = std::collections::HashMap::new();
    for (_id, table_names_json) in all_templates {
        if let Ok(tables) = serde_json::from_str::<Vec<String>>(&table_names_json) {
            for table in tables {
                *table_counts.entry(table).or_insert(0) += 1;
            }
        }
    }

    let mut by_table: Vec<GroupCount> = table_counts
        .into_iter()
        .map(|(name, count)| GroupCount { name, count })
        .collect();
    by_table.sort_by(|a, b| b.count.cmp(&a.count));
    by_table.truncate(20); // 只取前20个最常用的表

    // 总模板数
    let total_templates: i32 = conn
        .query_row("SELECT COUNT(*) FROM sql_templates", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    // 总变体数
    let total_variants: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sql_history WHERE template_id IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(TemplateGroupStats {
        by_scene,
        by_type,
        by_table,
        total_templates,
        total_variants,
    })
}
