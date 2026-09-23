use crate::db::connection::DbConnection;
use crate::models::task_category::TaskCategoryDefinition;
use crate::services::task_category_service::TaskCategoryService;
use tauri::State;

#[tauri::command]
pub fn get_task_categories(db: State<DbConnection>) -> Result<Vec<TaskCategoryDefinition>, String> {
    let conn = db.0.lock().map_err(|error| error.to_string())?;
    TaskCategoryService::get_all(&conn)
}

#[tauri::command]
pub fn create_task_category(
    db: State<DbConnection>,
    name: String,
    color: String,
) -> Result<TaskCategoryDefinition, String> {
    let conn = db.0.lock().map_err(|error| error.to_string())?;
    TaskCategoryService::create(&conn, &name, &color)
}

#[tauri::command]
pub fn update_task_category(
    db: State<DbConnection>,
    id: i64,
    name: String,
    color: String,
) -> Result<TaskCategoryDefinition, String> {
    let conn = db.0.lock().map_err(|error| error.to_string())?;
    TaskCategoryService::update(&conn, id, &name, &color)
}

#[tauri::command]
pub fn set_task_category_hidden(
    db: State<DbConnection>,
    id: i64,
    is_hidden: bool,
) -> Result<TaskCategoryDefinition, String> {
    let conn = db.0.lock().map_err(|error| error.to_string())?;
    TaskCategoryService::set_hidden(&conn, id, is_hidden)
}

#[tauri::command]
pub fn delete_task_category(db: State<DbConnection>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|error| error.to_string())?;
    TaskCategoryService::delete(&conn, id)
}
