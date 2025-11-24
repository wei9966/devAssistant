use tauri::State;
use crate::db::connection::DbConnection;
use crate::models::task::Tag;
use crate::services::tag_service::TagService;
use serde::{Deserialize, Serialize};

/// 标签使用统计结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagWithUsage {
    pub tag: Tag,
    pub usage_count: i64,
}

// ==================== 标签管理命令 ====================

/// 创建标签
#[tauri::command]
pub fn create_tag(db: State<DbConnection>, name: String, color: String) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::create_tag(&conn, &name, &color).map_err(|e| e.to_string())
}

/// 获取所有标签
#[tauri::command]
pub fn get_all_tags(db: State<DbConnection>) -> Result<Vec<Tag>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::get_all_tags(&conn).map_err(|e| e.to_string())
}

/// 根据ID获取标签
#[tauri::command]
pub fn get_tag_by_id(db: State<DbConnection>, id: i64) -> Result<Tag, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::get_tag_by_id(&conn, id).map_err(|e| e.to_string())
}

/// 更新标签
#[tauri::command]
pub fn update_tag(
    db: State<DbConnection>,
    id: i64,
    name: String,
    color: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::update_tag(&conn, id, &name, &color).map_err(|e| e.to_string())
}

/// 删除标签
#[tauri::command]
pub fn delete_tag(db: State<DbConnection>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::delete_tag(&conn, id).map_err(|e| e.to_string())
}

// ==================== 任务标签关联命令 ====================

/// 为任务添加标签
#[tauri::command]
pub fn add_tag_to_task(
    db: State<DbConnection>,
    task_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::add_tag_to_task(&conn, task_id, tag_id).map_err(|e| e.to_string())
}

/// 从任务移除标签
#[tauri::command]
pub fn remove_tag_from_task(
    db: State<DbConnection>,
    task_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::remove_tag_from_task(&conn, task_id, tag_id).map_err(|e| e.to_string())
}

/// 获取任务的所有标签
#[tauri::command]
pub fn get_task_tags(db: State<DbConnection>, task_id: i64) -> Result<Vec<Tag>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::get_task_tags(&conn, task_id).map_err(|e| e.to_string())
}

/// 获取使用某标签的所有任务ID
#[tauri::command]
pub fn get_tasks_by_tag(db: State<DbConnection>, tag_id: i64) -> Result<Vec<i64>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::get_tasks_by_tag(&conn, tag_id).map_err(|e| e.to_string())
}

/// 批量为任务添加标签
#[tauri::command]
pub fn add_tags_to_task(
    db: State<DbConnection>,
    task_id: i64,
    tag_ids: Vec<i64>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::add_tags_to_task(&conn, task_id, &tag_ids).map_err(|e| e.to_string())
}

/// 移除任务的所有标签
#[tauri::command]
pub fn remove_all_tags_from_task(db: State<DbConnection>, task_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::remove_all_tags_from_task(&conn, task_id).map_err(|e| e.to_string())
}

// ==================== 高级查询命令 ====================

/// 获取标签的使用统计
#[tauri::command]
pub fn get_tag_usage_count(db: State<DbConnection>, tag_id: i64) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::get_tag_usage_count(&conn, tag_id).map_err(|e| e.to_string())
}

/// 获取所有标签及其使用次数
#[tauri::command]
pub fn get_tags_with_usage_count(db: State<DbConnection>) -> Result<Vec<TagWithUsage>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let results = TagService::get_tags_with_usage_count(&conn).map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .map(|(tag, usage_count)| TagWithUsage { tag, usage_count })
        .collect())
}

/// 搜索标签（按名称模糊搜索）
#[tauri::command]
pub fn search_tags(db: State<DbConnection>, keyword: String) -> Result<Vec<Tag>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TagService::search_tags(&conn, &keyword).map_err(|e| e.to_string())
}
