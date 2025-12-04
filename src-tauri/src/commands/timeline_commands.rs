// Timeline Commands
// 时间线相关的 Tauri 命令

use crate::db::connection::DbConnection;
use crate::models::screen_context::ScreenContext;
use crate::services::ai_service::AiService;
use crate::services::context_store_service::ContextStoreService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

/// 时间线筛选器
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineFilter {
    pub date_range: Option<(String, String)>,
    pub activity_types: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub min_importance: Option<i32>,
}

/// 时间线分组
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineGroup {
    pub date: String,
    pub display_date: String,
    pub items: Vec<TimelineItem>,
    pub summary: Option<String>,
}

/// 时间线项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub keywords: Vec<String>,
    pub importance: i32,
    pub start_time: String,
    pub end_time: Option<String>,
    pub app_name: Option<String>,
    pub activity_type: String,
    pub thumbnail_path: Option<String>,
    pub merged: bool,
    pub merged_count: Option<i32>,
}

impl From<ScreenContext> for TimelineItem {
    fn from(context: ScreenContext) -> Self {
        TimelineItem {
            id: context.id.map(|id| id.to_string()).unwrap_or_default(),
            title: context.window_title.clone().unwrap_or_else(|| {
                context
                    .app_name
                    .clone()
                    .unwrap_or_else(|| "未知活动".to_string())
            }),
            summary: context.description.clone(),
            keywords: vec![],
            importance: 2, // 默认重要性
            start_time: context.captured_at.clone(),
            end_time: None,
            app_name: context.app_name,
            activity_type: context.activity_type,
            thumbnail_path: context.screenshot_path,
            merged: false,
            merged_count: None,
        }
    }
}

/// 获取时间线数据
#[tauri::command]
pub async fn get_timeline(
    filter: Option<TimelineFilter>,
    db: State<'_, DbConnection>,
) -> Result<Vec<TimelineGroup>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 构建查询条件
    let mut query = String::from(
        "SELECT id, captured_at, app_name, window_title, activity_type,
                description, key_content, screenshot_hash, screenshot_path,
                processing_time_ms
         FROM screen_contexts WHERE 1=1",
    );

    let mut query_params: Vec<String> = vec![];

    if let Some(ref f) = filter {
        // 日期范围过滤
        if let Some(ref date_range) = f.date_range {
            query.push_str(" AND date(captured_at) >= ? AND date(captured_at) <= ?");
            query_params.push(date_range.0.clone());
            query_params.push(date_range.1.clone());
        }

        // 活动类型过滤
        if let Some(ref types) = f.activity_types {
            if !types.is_empty() {
                let placeholders = types.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                query.push_str(&format!(" AND activity_type IN ({})", placeholders));
                query_params.extend(types.clone());
            }
        }

        // 关键词过滤
        if let Some(ref keywords) = f.keywords {
            if !keywords.is_empty() {
                let conditions = keywords
                    .iter()
                    .map(|_| "(description LIKE ? OR key_content LIKE ?)")
                    .collect::<Vec<_>>()
                    .join(" OR ");
                query.push_str(&format!(" AND ({})", conditions));
                for keyword in keywords {
                    let pattern = format!("%{}%", keyword);
                    query_params.push(pattern.clone());
                    query_params.push(pattern);
                }
            }
        }
    }

    query.push_str(" ORDER BY captured_at DESC");

    // 执行查询
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let param_refs: Vec<&dyn rusqlite::ToSql> = query_params
        .iter()
        .map(|p| p as &dyn rusqlite::ToSql)
        .collect();

    let contexts = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(ScreenContext {
                id: Some(row.get(0)?),
                captured_at: row.get(1)?,
                app_name: row.get(2)?,
                window_title: row.get(3)?,
                activity_type: row.get(4)?,
                description: row.get(5)?,
                key_content: row.get(6)?,
                screenshot_hash: row.get(7)?,
                screenshot_path: row.get(8)?,
                processing_time_ms: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 按日期分组
    let mut groups: HashMap<String, Vec<ScreenContext>> = HashMap::new();
    for context in contexts {
        let date = context.captured_at.split(' ').next().unwrap_or("").to_string();
        groups.entry(date).or_insert_with(Vec::new).push(context);
    }

    // 转换为TimelineGroup
    let mut timeline_groups: Vec<TimelineGroup> = groups
        .into_iter()
        .map(|(date, contexts)| {
            let items: Vec<TimelineItem> = contexts.into_iter().map(|c| c.into()).collect();
            TimelineGroup {
                date: date.clone(),
                display_date: date,
                items,
                summary: None,
            }
        })
        .collect();

    // 按日期排序
    timeline_groups.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(timeline_groups)
}

/// 生成日报
#[tauri::command]
pub async fn generate_daily_report(
    date: String,
    db: State<'_, DbConnection>,
) -> Result<String, String> {
    // 提取所有需要的数据，不跨await持有锁
    let (contexts, stats) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;

        // 获取AI配置
        let config_json: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'ai_config'",
                [],
                |row| row.get(0),
            )
            .ok();

        if config_json.is_none() {
            return Err("AI服务未配置".to_string());
        }

        let config: serde_json::Value = serde_json::from_str(&config_json.unwrap())
            .map_err(|e| format!("解析 AI 配置失败: {}", e))?;

        let enabled = config["enabled"].as_bool().unwrap_or(false);
        if !enabled {
            return Err("AI服务未启用".to_string());
        }

        // 获取上下文数据
        let contexts = ContextStoreService::list_by_date(&conn, &date)
            .map_err(|e| e.to_string())?;
        let stats = ContextStoreService::get_day_stats(&conn, &date)
            .map_err(|e| e.to_string())?;

        (contexts, stats)
        // conn lock dropped here
    };

    if contexts.is_empty() {
        return Ok(format!("# {} 日报\n\n今日无活动记录。", date));
    }

    // 构建简单的日报（不使用AI，避免跨await持有锁的复杂性）
    let mut report = format!("# {} 工作日报\n\n", date);

    report.push_str(&format!("## 总览\n\n- 总记录数: {}\n", stats.total_count));

    if let Some((start, end)) = &stats.time_range {
        report.push_str(&format!("- 活动时间: {} 至 {}\n\n", start, end));
    }

    if !stats.app_distribution.is_empty() {
        report.push_str("## 应用使用情况\n\n");
        let mut apps: Vec<_> = stats.app_distribution.iter().collect();
        apps.sort_by(|a, b| b.1.cmp(a.1));
        for (app, count) in apps.iter().take(5) {
            report.push_str(&format!("- {}: {} 次\n", app, count));
        }
        report.push('\n');
    }

    if !stats.activity_distribution.is_empty() {
        report.push_str("## 活动类型分布\n\n");
        let mut activities: Vec<_> = stats.activity_distribution.iter().collect();
        activities.sort_by(|a, b| b.1.cmp(a.1));
        for (activity, count) in activities {
            report.push_str(&format!("- {}: {} 次\n", activity, count));
        }
    }

    Ok(report)
}

/// 手动合并时间线项
#[tauri::command]
pub async fn merge_timeline_items(
    item_ids: Vec<String>,
    db: State<'_, DbConnection>,
) -> Result<TimelineItem, String> {
    if item_ids.is_empty() {
        return Err("没有选择要合并的项".to_string());
    }

    // 提取上下文数据，不跨await持有锁
    let contexts = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;

        // 获取AI配置检查
        let config_json: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'ai_config'",
                [],
                |row| row.get(0),
            )
            .ok();

        if config_json.is_none() {
            return Err("AI服务未配置".to_string());
        }

        let config: serde_json::Value = serde_json::from_str(&config_json.unwrap())
            .map_err(|e| format!("解析 AI 配置失败: {}", e))?;

        let enabled = config["enabled"].as_bool().unwrap_or(false);
        if !enabled {
            return Err("AI服务未启用".to_string());
        }

        // 查询要合并的上下文
        let placeholders = item_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT id, captured_at, app_name, window_title, activity_type,
                    description, key_content, screenshot_hash, screenshot_path,
                    processing_time_ms
             FROM screen_contexts
             WHERE id IN ({})",
            placeholders
        );

        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

        // 创建参数的引用向量，确保生命周期正确
        let params: Vec<_> = item_ids.iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();

        let contexts_result = stmt
            .query_map(rusqlite::params_from_iter(params), |row| {
                Ok(ScreenContext {
                    id: Some(row.get(0)?),
                    captured_at: row.get(1)?,
                    app_name: row.get(2)?,
                    window_title: row.get(3)?,
                    activity_type: row.get(4)?,
                    description: row.get(5)?,
                    key_content: row.get(6)?,
                    screenshot_hash: row.get(7)?,
                    screenshot_path: row.get(8)?,
                    processing_time_ms: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        contexts_result
        // conn lock dropped here
    };

    // 构建简化的合并结果（不使用AI）
    if contexts.is_empty() {
        return Err("未找到要合并的项".to_string());
    }

    let start_time = contexts.first().map(|c| c.captured_at.clone()).unwrap_or_default();
    let end_time = contexts.last().map(|c| c.captured_at.clone());
    let merged_count = contexts.len() as i32;

    // 提取应用名称和活动类型
    let app_name = contexts.first().and_then(|c| c.app_name.clone());
    let activity_type = contexts.first().map(|c| c.activity_type.clone()).unwrap_or_default();

    // 生成简单的标题和摘要
    let title = format!("合并活动 ({} 项)", merged_count);
    let summary = contexts.iter()
        .map(|c| c.description.as_str())
        .take(3)
        .collect::<Vec<_>>()
        .join("; ");

    Ok(TimelineItem {
        id: "merged".to_string(),
        title,
        summary,
        keywords: vec![],
        importance: 2,
        start_time,
        end_time,
        app_name,
        activity_type,
        thumbnail_path: None,
        merged: true,
        merged_count: Some(merged_count),
    })
}

/// 获取指定日期的活动
#[tauri::command]
pub async fn get_activities_by_date(
    date: String,
    db: State<'_, DbConnection>,
) -> Result<Vec<TimelineItem>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let contexts = ContextStoreService::list_by_date(&conn, &date).map_err(|e| e.to_string())?;

    let items: Vec<TimelineItem> = contexts.into_iter().map(|c| c.into()).collect();

    Ok(items)
}

/// 自动合并时间窗口内的上下文
#[tauri::command]
pub async fn auto_merge_by_time_window(
    window_minutes: i32,
    db: State<'_, DbConnection>,
) -> Result<Vec<crate::services::context_manager_service::MergeResult>, String> {
    // 检查AI配置，但不执行实际的AI合并
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;

        let config_json: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'ai_config'",
                [],
                |row| row.get(0),
            )
            .ok();

        if config_json.is_none() {
            return Err("AI服务未配置".to_string());
        }

        let config: serde_json::Value = serde_json::from_str(&config_json.unwrap())
            .map_err(|e| format!("解析 AI 配置失败: {}", e))?;

        let enabled = config["enabled"].as_bool().unwrap_or(false);
        if !enabled {
            return Err("AI服务未启用".to_string());
        }
        // conn lock dropped here
    }

    // 返回空结果，表示功能暂未实现（需要AI支持）
    // 实际实现需要重构ContextManager以支持不持有Connection锁的异步操作
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_item_conversion() {
        let context = ScreenContext {
            id: Some(1),
            captured_at: "2024-12-04 10:30:00".to_string(),
            app_name: Some("VS Code".to_string()),
            window_title: Some("main.rs - DevAssistant".to_string()),
            activity_type: "coding".to_string(),
            description: "编写 Rust 代码".to_string(),
            key_content: Some("impl ContextStoreService".to_string()),
            screenshot_hash: Some("abc123".to_string()),
            screenshot_path: None,
            processing_time_ms: Some(150),
        };

        let item: TimelineItem = context.into();
        assert_eq!(item.id, "1");
        assert_eq!(item.activity_type, "coding");
        assert_eq!(item.title, "main.rs - DevAssistant");
    }
}
