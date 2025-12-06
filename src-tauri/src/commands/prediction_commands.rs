// Prediction Commands
// 预测任务相关命令

use crate::db::connection::DbConnection;
use crate::services::todo_prediction_service::{PredictedTask, TodoPredictionService};
use tauri::State;

/// 获取数据库路径
fn get_db_path(db: &State<DbConnection>) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| format!("获取数据库连接失败: {}", e))?;
    conn.path()
        .map(|p| p.to_string())
        .ok_or_else(|| "无法获取数据库路径".to_string())
}

/// 获取待处理的预测任务列表
#[tauri::command]
pub fn get_pending_predictions(db: State<DbConnection>) -> Result<Vec<PredictedTask>, String> {
    let db_path = get_db_path(&db)?;
    let service = TodoPredictionService::new(db_path);

    service
        .get_pending_predictions()
        .map_err(|e| format!("获取待处理预测任务失败: {}", e))
}

/// 获取所有预测任务列表（包含已处理的）
#[tauri::command]
pub fn get_all_predictions(
    db: State<DbConnection>,
    limit: Option<i32>,
) -> Result<Vec<PredictedTask>, String> {
    let db_path = get_db_path(&db)?;
    let service = TodoPredictionService::new(db_path);

    service
        .get_all_predictions(limit.unwrap_or(50))
        .map_err(|e| format!("获取预测任务列表失败: {}", e))
}

/// 接受预测任务（创建为实际任务）
#[tauri::command]
pub fn accept_prediction(db: State<DbConnection>, prediction_id: i64) -> Result<i64, String> {
    let db_path = get_db_path(&db)?;
    let service = TodoPredictionService::new(db_path);

    service
        .accept_prediction(prediction_id)
        .map_err(|e| format!("接受预测任务失败: {}", e))
}

/// 忽略预测任务
#[tauri::command]
pub fn ignore_prediction(db: State<DbConnection>, prediction_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&db)?;
    let service = TodoPredictionService::new(db_path);

    service
        .ignore_prediction(prediction_id)
        .map_err(|e| format!("忽略预测任务失败: {}", e))
}

/// 批量接受预测任务
#[tauri::command]
pub fn accept_predictions(
    db: State<DbConnection>,
    prediction_ids: Vec<i64>,
) -> Result<Vec<i64>, String> {
    let db_path = get_db_path(&db)?;
    let service = TodoPredictionService::new(db_path);

    service
        .accept_predictions(&prediction_ids)
        .map_err(|e| format!("批量接受预测任务失败: {}", e))
}

/// 批量忽略预测任务
#[tauri::command]
pub fn ignore_predictions(
    db: State<DbConnection>,
    prediction_ids: Vec<i64>,
) -> Result<(), String> {
    let db_path = get_db_path(&db)?;
    let service = TodoPredictionService::new(db_path);

    service
        .ignore_predictions(&prediction_ids)
        .map_err(|e| format!("批量忽略预测任务失败: {}", e))
}

/// 获取待处理的预测任务数量
#[tauri::command]
pub fn get_pending_prediction_count(db: State<DbConnection>) -> Result<i32, String> {
    let db_path = get_db_path(&db)?;
    let service = TodoPredictionService::new(db_path);

    service
        .get_pending_count()
        .map_err(|e| format!("获取待处理预测任务数量失败: {}", e))
}

/// 清理过期的预测任务
#[tauri::command]
pub fn cleanup_old_predictions(db: State<DbConnection>, days: Option<i32>) -> Result<i32, String> {
    let db_path = get_db_path(&db)?;
    let service = TodoPredictionService::new(db_path);

    service
        .cleanup_old_predictions(days.unwrap_or(7))
        .map_err(|e| format!("清理过期预测任务失败: {}", e))
}
