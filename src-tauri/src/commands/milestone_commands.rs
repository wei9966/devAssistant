use crate::db::connection::DbConnection;
use crate::models::task::TaskMilestone;
use crate::services::milestone_service::MilestoneService;
use tauri::State;

#[tauri::command]
pub fn create_task_milestone(
    db: State<DbConnection>,
    task_id: i64,
    title: String,
    description: Option<String>,
    progress_snapshot: Option<i32>,
) -> Result<TaskMilestone, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    MilestoneService::create_milestone(
        &conn,
        task_id,
        &title,
        description.as_deref(),
        progress_snapshot,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_task_milestones(
    db: State<DbConnection>,
    task_id: i64,
) -> Result<Vec<TaskMilestone>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    MilestoneService::get_task_milestones(&conn, task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task_milestone(
    db: State<DbConnection>,
    milestone_id: i64,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    MilestoneService::delete_milestone(&conn, milestone_id).map_err(|e| e.to_string())
}
