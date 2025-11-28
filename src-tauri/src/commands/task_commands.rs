use crate::db::connection::DbConnection;
use crate::models::task::{
    ImportResult, ImportTask, QuadrantStatistics, Task, TaskCategory, TaskPriority, TaskQuadrant,
    WorkContext,
};
use crate::services::task_service::TaskService;
use tauri::State;

#[tauri::command]
pub fn get_all_tasks(db: State<DbConnection>) -> Result<Vec<Task>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TaskService::get_all_tasks(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_completed_tasks(db: State<DbConnection>, days: i64) -> Result<Vec<Task>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TaskService::get_completed_tasks(&conn, days).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_task(
    db: State<DbConnection>,
    title: String,
    description: Option<String>,
    category: String,
    priority: i32,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let cat = TaskCategory::from_str(&category);
    let pri = TaskPriority::from_i32(priority);
    TaskService::create_task(&conn, &title, description.as_deref(), cat, pri)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_task(db: State<DbConnection>, task_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TaskService::start_task(&conn, task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pause_task(
    db: State<DbConnection>,
    task_id: i64,
    context: Option<WorkContext>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TaskService::pause_task(&conn, task_id, context).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn defer_task(db: State<DbConnection>, task_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TaskService::defer_task(&conn, task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn complete_task(db: State<DbConnection>, task_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TaskService::complete_task(&conn, task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task(
    db: State<DbConnection>,
    task_id: i64,
    title: Option<String>,
    description: Option<String>,
    category: Option<String>,
    priority: Option<i32>,
    git_branch: Option<String>,
    notes: Option<String>,
    quadrant: Option<String>,
    due_date: Option<String>,
    registered_at: Option<String>,
    display_date: Option<String>,
) -> Result<(), String> {
    // 添加调试日志
    eprintln!("=== update_task called ===");
    eprintln!("task_id: {}", task_id);
    eprintln!("title: {:?}", title);
    eprintln!("description: {:?}", description);
    eprintln!("category: {:?}", category);
    eprintln!("priority: {:?}", priority);
    eprintln!("git_branch: {:?}", git_branch);
    eprintln!("notes: {:?}", notes);
    eprintln!("quadrant: {:?}", quadrant);
    eprintln!("due_date: {:?}", due_date);
    eprintln!("registered_at: {:?}", registered_at);
    eprintln!("display_date: {:?}", display_date);

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let cat = category.map(|c| TaskCategory::from_str(&c));
    let pri = priority.map(TaskPriority::from_i32);
    let quad = quadrant.map(|q| TaskQuadrant::from_str(&q));

    eprintln!("Converted category: {:?}", cat);
    eprintln!("Converted priority: {:?}", pri);
    eprintln!("Converted quadrant: {:?}", quad);

    TaskService::update_task(
        &conn,
        task_id,
        title.as_deref(),
        description.as_deref(),
        cat,
        pri,
        git_branch.as_deref(),
        notes.as_deref(),
        quad,
        due_date.as_deref(),
        registered_at.as_deref(),
        display_date.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task(db: State<DbConnection>, task_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TaskService::delete_task(&conn, task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_stale_tasks(db: State<DbConnection>, days: i64) -> Result<Vec<Task>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TaskService::get_stale_tasks(&conn, days).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_current_branch() -> Result<String, String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .map_err(|e| format!("Failed to execute git command: {}", e))?;

    if !output.status.success() {
        return Err("Not a git repository or git command not available".to_string());
    }

    let branch = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to parse git output: {}", e))?
        .trim()
        .to_string();

    Ok(branch)
}

#[tauri::command]
pub fn import_tasks(
    db: State<DbConnection>,
    tasks: Vec<ImportTask>,
) -> Result<ImportResult, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TaskService::import_tasks(&conn, tasks).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_import_template() -> Result<String, String> {
    Ok(TaskService::generate_import_template())
}

#[tauri::command]
pub fn get_tasks_by_quadrant(
    db: State<DbConnection>,
    quadrant: String,
) -> Result<Vec<Task>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let quad = TaskQuadrant::from_str(&quadrant);
    TaskService::get_tasks_by_quadrant(&conn, quad).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_quadrant_statistics(db: State<DbConnection>) -> Result<Vec<QuadrantStatistics>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let stats = TaskService::get_quadrant_statistics(&conn).map_err(|e| e.to_string())?;

    // 将 (String, i64) 转换为 QuadrantStatistics
    let result = stats
        .into_iter()
        .map(|(quadrant, count)| QuadrantStatistics { quadrant, count })
        .collect();

    Ok(result)
}

#[tauri::command]
pub fn get_tasks_by_date_range(
    db: State<DbConnection>,
    start_date: String,
    end_date: String,
) -> Result<Vec<Task>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TaskService::get_tasks_by_date_range(&conn, &start_date, &end_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task_display_date(
    db: State<DbConnection>,
    task_id: i64,
    display_date: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TaskService::update_task_display_date(&conn, task_id, &display_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_task_to_today(db: State<DbConnection>, task_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    TaskService::move_task_to_today(&conn, task_id).map_err(|e| e.to_string())
}
