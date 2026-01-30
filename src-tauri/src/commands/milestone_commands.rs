use crate::db::connection::DbConnection;
use crate::models::task::TaskMilestone;
use crate::services::milestone_service::MilestoneService;
use tauri::State;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneWithTask {
    #[serde(flatten)]
    pub milestone: TaskMilestone,
    pub task_title: String,
}

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

#[tauri::command]
pub fn get_milestones_by_date_range(
    db: State<DbConnection>,
    start_date: String,
    end_date: String,
) -> Result<Vec<MilestoneWithTask>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let results = MilestoneService::get_milestones_by_date_range(&conn, &start_date, &end_date)
        .map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .map(|(milestone, task_title)| MilestoneWithTask {
            milestone,
            task_title,
        })
        .collect())
}
