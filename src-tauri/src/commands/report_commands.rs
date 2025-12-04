// Report Commands
// 日报相关命令

use crate::db::connection::DbConnection;
use crate::services::ai_service::AiService;
use crate::services::report_service::{DailyReport, ReportService};
use tauri::State;

/// 生成日报（手动或定时触发）
#[tauri::command]
pub async fn report_generate(
    db: State<'_, DbConnection>,
    date: String,
) -> Result<DailyReport, String> {
    // 验证日期格式
    if date.len() != 10 || !date.contains('-') {
        return Err("日期格式无效，应为 YYYY-MM-DD".to_string());
    }

    // 第1步：同步收集数据（在作用域内获取锁并释放）
    let (ai_config, input_data) = {
        let conn = db.0.lock().map_err(|e| format!("数据库锁定失败: {}", e))?;

        let ai_config = AiService::load_config(&conn)
            .map_err(|e| format!("加载AI配置失败: {}", e))?;

        let input_data = ReportService::collect_report_data(&conn, &date)
            .map_err(|e| format!("收集日报数据失败: {}", e))?;

        (ai_config, input_data)
    };

    if !ai_config.is_valid() {
        return Err("AI 配置无效或未启用".to_string());
    }

    let ai_service = AiService::new(ai_config);

    // 第2步：异步生成日报（不使用Connection）
    let report = ReportService::generate_report_async(input_data, &ai_service)
        .await
        .map_err(|e| format!("生成日报失败: {}", e))?;

    // 第3步：同步保存日报
    let conn = db.0.lock().map_err(|e| format!("数据库锁定失败: {}", e))?;
    ReportService::save_and_get_report(&conn, &report)
        .map_err(|e| format!("保存日报失败: {}", e))
}

/// 获取指定日期的日报
#[tauri::command]
pub fn report_get(db: State<'_, DbConnection>, date: String) -> Result<Option<DailyReport>, String> {
    let conn = db.0.lock().map_err(|e| format!("数据库锁定失败: {}", e))?;

    ReportService::get_report(&conn, &date).map_err(|e| format!("获取日报失败: {}", e))
}

/// 获取日报列表
#[tauri::command]
pub fn report_list(
    db: State<'_, DbConnection>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<DailyReport>, String> {
    let conn = db.0.lock().map_err(|e| format!("数据库锁定失败: {}", e))?;

    ReportService::list_reports(&conn, limit, offset)
        .map_err(|e| format!("获取日报列表失败: {}", e))
}

/// 删除日报
#[tauri::command]
pub fn report_delete(db: State<'_, DbConnection>, date: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| format!("数据库锁定失败: {}", e))?;

    ReportService::delete_report(&conn, &date).map_err(|e| format!("删除日报失败: {}", e))
}

/// 检查指定日期是否已有日报
#[tauri::command]
pub fn report_exists(db: State<'_, DbConnection>, date: String) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| format!("数据库锁定失败: {}", e))?;

    let report = ReportService::get_report(&conn, &date)
        .map_err(|e| format!("检查日报失败: {}", e))?;

    Ok(report.is_some())
}
