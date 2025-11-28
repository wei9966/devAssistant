use crate::db::connection::DbConnection;
use crate::models::weekly_plan::WeeklyPlan;
use crate::services::weekly_plan_service::WeeklyPlanService;
use tauri::State;

/// 保存周计划
/// 如果指定周的计划已存在,则更新;否则创建新记录
#[tauri::command]
pub fn save_weekly_plan(
    db: State<DbConnection>,
    week_key: String,
    content: String,
    task_ids: Vec<i64>,
    status: Option<String>,
) -> Result<WeeklyPlan, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let status_str = status.unwrap_or_else(|| "draft".to_string());

    WeeklyPlanService::save_weekly_plan(&conn, &week_key, &content, &task_ids, &status_str)
        .map_err(|e| e.to_string())
}

/// 获取指定周的计划
#[tauri::command]
pub fn get_weekly_plan(
    db: State<DbConnection>,
    week_key: String,
) -> Result<Option<WeeklyPlan>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    WeeklyPlanService::get_weekly_plan(&conn, &week_key).map_err(|e| e.to_string())
}

/// 获取所有周计划
#[tauri::command]
pub fn get_all_weekly_plans(db: State<DbConnection>) -> Result<Vec<WeeklyPlan>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    WeeklyPlanService::get_all_weekly_plans(&conn).map_err(|e| e.to_string())
}

/// 获取最近N周的计划
#[tauri::command]
pub fn get_recent_weekly_plans(
    db: State<DbConnection>,
    limit: i32,
) -> Result<Vec<WeeklyPlan>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    WeeklyPlanService::get_recent_weekly_plans(&conn, limit).map_err(|e| e.to_string())
}

/// 更新周计划的任务列表
#[tauri::command]
pub fn update_weekly_plan_tasks(
    db: State<DbConnection>,
    week_key: String,
    task_ids: Vec<i64>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    WeeklyPlanService::update_task_ids(&conn, &week_key, &task_ids).map_err(|e| e.to_string())
}

/// 更新周计划状态
#[tauri::command]
pub fn update_weekly_plan_status(
    db: State<DbConnection>,
    week_key: String,
    status: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    WeeklyPlanService::update_status(&conn, &week_key, &status).map_err(|e| e.to_string())
}

/// 删除指定周的计划
#[tauri::command]
pub fn delete_weekly_plan(db: State<DbConnection>, week_key: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    WeeklyPlanService::delete_weekly_plan(&conn, &week_key).map_err(|e| e.to_string())
}
