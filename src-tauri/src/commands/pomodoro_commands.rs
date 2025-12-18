use crate::db::connection::DbConnection;
use crate::models::pomodoro::{
    CompletePomodoroRequest, CreatePomodoroRequest, FocusApp, PomodoroDailyStats,
    PomodoroSession,
};
use crate::services::pomodoro_service::{PomodoroService, PomodoroAiService};
use crate::commands::ai_commands::AiState;
use tauri::State;

// ==================== 会话管理命令 ====================

/// 创建新的番茄钟会话
#[tauri::command]
pub fn create_pomodoro_session(
    db: State<DbConnection>,
    request: CreatePomodoroRequest,
) -> Result<PomodoroSession, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::create_session(&conn, request).map_err(|e| e.to_string())
}

/// 开始番茄钟
#[tauri::command]
pub fn start_pomodoro(
    db: State<DbConnection>,
    session_id: i64,
) -> Result<PomodoroSession, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::start_session(&conn, session_id).map_err(|e| e.to_string())
}

/// 暂停番茄钟
#[tauri::command]
pub fn pause_pomodoro(
    db: State<DbConnection>,
    session_id: i64,
) -> Result<PomodoroSession, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::pause_session(&conn, session_id).map_err(|e| e.to_string())
}

/// 恢复番茄钟
#[tauri::command]
pub fn resume_pomodoro(
    db: State<DbConnection>,
    session_id: i64,
) -> Result<PomodoroSession, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::resume_session(&conn, session_id).map_err(|e| e.to_string())
}

/// 取消番茄钟
#[tauri::command]
pub fn cancel_pomodoro(
    db: State<DbConnection>,
    session_id: i64,
) -> Result<PomodoroSession, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::cancel_session(&conn, session_id).map_err(|e| e.to_string())
}

/// 完成番茄钟
#[tauri::command]
pub fn complete_pomodoro(
    db: State<DbConnection>,
    session_id: i64,
    request: CompletePomodoroRequest,
) -> Result<PomodoroSession, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::complete_session(&conn, session_id, request).map_err(|e| e.to_string())
}

/// 记录分心
#[tauri::command]
pub fn record_pomodoro_distraction(
    db: State<DbConnection>,
    session_id: i64,
) -> Result<PomodoroSession, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::record_distraction(&conn, session_id).map_err(|e| e.to_string())
}

/// 更新专注时间
#[tauri::command]
pub fn update_pomodoro_focus_time(
    db: State<DbConnection>,
    session_id: i64,
    seconds: i32,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::update_focus_time(&conn, session_id, seconds).map_err(|e| e.to_string())
}

/// 获取会话详情
#[tauri::command]
pub fn get_pomodoro_session(
    db: State<DbConnection>,
    session_id: i64,
) -> Result<Option<PomodoroSession>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    match PomodoroService::get_session_by_id(&conn, session_id) {
        Ok(session) => Ok(Some(session)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// 获取当前活跃会话
#[tauri::command]
pub fn get_active_pomodoro_session(
    db: State<DbConnection>,
) -> Result<Option<PomodoroSession>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::get_active_session(&conn).map_err(|e| e.to_string())
}

/// 获取今日会话列表
#[tauri::command]
pub fn get_today_pomodoro_sessions(
    db: State<DbConnection>,
) -> Result<Vec<PomodoroSession>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::get_today_sessions(&conn).map_err(|e| e.to_string())
}

/// 获取指定任务的会话历史
#[tauri::command]
pub fn get_task_pomodoro_sessions(
    db: State<DbConnection>,
    task_id: i64,
) -> Result<Vec<PomodoroSession>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::get_task_sessions(&conn, task_id).map_err(|e| e.to_string())
}

// ==================== 白名单应用管理命令 ====================

/// 获取所有白名单应用
#[tauri::command]
pub fn get_pomodoro_focus_apps(db: State<DbConnection>) -> Result<Vec<FocusApp>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::get_all_focus_apps(&conn).map_err(|e| e.to_string())
}

/// 添加白名单应用
#[tauri::command]
pub fn add_pomodoro_focus_app(
    db: State<DbConnection>,
    name: String,
    process_name: Option<String>,
) -> Result<FocusApp, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::add_focus_app(&conn, name, process_name).map_err(|e| e.to_string())
}

/// 删除白名单应用
#[tauri::command]
pub fn remove_pomodoro_focus_app(db: State<DbConnection>, app_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::remove_focus_app(&conn, app_id).map_err(|e| e.to_string())
}

// ==================== 统计数据命令 ====================

/// 获取今日统计
#[tauri::command]
pub fn get_pomodoro_today_stats(
    db: State<DbConnection>,
) -> Result<PomodoroDailyStats, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::get_today_stats(&conn).map_err(|e| e.to_string())
}

/// 获取指定日期统计
#[tauri::command]
pub fn get_pomodoro_stats_by_date(
    db: State<DbConnection>,
    date: String,
) -> Result<Option<PomodoroDailyStats>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::get_stats_by_date(&conn, &date).map_err(|e| e.to_string())
}

/// 获取日期范围统计
#[tauri::command]
pub fn get_pomodoro_stats_range(
    db: State<DbConnection>,
    start_date: String,
    end_date: String,
) -> Result<Vec<PomodoroDailyStats>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::get_stats_range(&conn, &start_date, &end_date).map_err(|e| e.to_string())
}

// ==================== AI 功能命令 ====================

/// AI 任务拆解建议 - 在开始专注前调用，帮助拆解任务为25分钟可完成的目标
#[tauri::command]
pub async fn pomodoro_ai_task_breakdown(
    ai_state: State<'_, AiState>,
    task_title: String,
    task_description: Option<String>,
) -> Result<String, String> {
    // 克隆 AI 服务以避免持有 MutexGuard 跨越 await
    let ai_service = {
        let guard = ai_state.0.lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("AI 服务未初始化")?.clone()
    };

    PomodoroAiService::get_task_breakdown(
        &ai_service,
        &task_title,
        task_description.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// AI 专注力分析 - 在完成番茄钟后调用，分析专注表现
#[tauri::command]
pub async fn pomodoro_ai_focus_analysis(
    ai_state: State<'_, AiState>,
    focus_goal: String,
    duration_minutes: i32,
    actual_focus_seconds: i32,
    distraction_count: i32,
    user_feedback: Option<String>,
) -> Result<String, String> {
    // 克隆 AI 服务以避免持有 MutexGuard 跨越 await
    let ai_service = {
        let guard = ai_state.0.lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("AI 服务未初始化")?.clone()
    };

    PomodoroAiService::get_focus_analysis(
        &ai_service,
        &focus_goal,
        duration_minutes,
        actual_focus_seconds,
        distraction_count,
        user_feedback.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// AI 每日复盘 - 在查看统计时调用，生成每日总结和建议
#[tauri::command]
pub async fn pomodoro_ai_daily_review(
    ai_state: State<'_, AiState>,
    db: State<'_, DbConnection>,
    date: String,
) -> Result<String, String> {
    // 获取当日统计数据
    let stats = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        PomodoroService::get_stats_by_date(&conn, &date)
            .map_err(|e| e.to_string())?
            .ok_or("当日无统计数据")?
    };

    // 克隆 AI 服务以避免持有 MutexGuard 跨越 await
    let ai_service = {
        let guard = ai_state.0.lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("AI 服务未初始化")?.clone()
    };

    let review = PomodoroAiService::get_daily_review(
        &ai_service,
        &date,
        stats.total_sessions,
        stats.completed_sessions,
        stats.total_focus_minutes,
        stats.avg_focus_rate,
        stats.app_usage.as_deref().unwrap_or("暂无数据"),
    )
    .await
    .map_err(|e| e.to_string())?;

    // 保存 AI 洞察到数据库
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        PomodoroService::update_daily_ai_insight(&conn, &date, &review)
            .map_err(|e| e.to_string())?;
    }

    Ok(review)
}

/// AI 进度评估 - 在提交反馈后调用，评估任务完成进度
#[tauri::command]
pub async fn pomodoro_ai_progress_eval(
    ai_state: State<'_, AiState>,
    task_title: String,
    focus_goal: String,
    user_feedback: String,
    previous_progress: Option<i32>,
) -> Result<String, String> {
    // 克隆 AI 服务以避免持有 MutexGuard 跨越 await
    let ai_service = {
        let guard = ai_state.0.lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("AI 服务未初始化")?.clone()
    };

    PomodoroAiService::get_progress_evaluation(
        &ai_service,
        &task_title,
        &focus_goal,
        &user_feedback,
        previous_progress,
    )
    .await
    .map_err(|e| e.to_string())
}

// ==================== 任务中断与恢复命令 ====================

/// AI 中断活动分析 - 分析中断期间的屏幕活动
/// 需要前端传入中断期间的活动摘要（从 activity_summaries 表获取）
#[tauri::command]
pub async fn pomodoro_ai_analyze_interruption(
    ai_state: State<'_, AiState>,
    original_task: String,
    focus_goal: String,
    interruption_duration_minutes: i32,
    activity_summaries: String,
) -> Result<String, String> {
    let ai_service = {
        let guard = ai_state.0.lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("AI 服务未初始化")?.clone()
    };

    PomodoroAiService::analyze_interruption(
        &ai_service,
        &original_task,
        &focus_goal,
        interruption_duration_minutes,
        &activity_summaries,
    )
    .await
    .map_err(|e| e.to_string())
}

/// AI 任务恢复建议 - 基于中断分析生成恢复建议
#[tauri::command]
pub async fn pomodoro_ai_resume_suggestion(
    ai_state: State<'_, AiState>,
    original_task: String,
    focus_goal: String,
    progress_before_interruption: Option<i32>,
    interruption_analysis: String,
    elapsed_focus_seconds: i32,
    remaining_seconds: i32,
) -> Result<String, String> {
    let ai_service = {
        let guard = ai_state.0.lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("AI 服务未初始化")?.clone()
    };

    PomodoroAiService::get_resume_suggestion(
        &ai_service,
        &original_task,
        &focus_goal,
        progress_before_interruption,
        &interruption_analysis,
        elapsed_focus_seconds,
        remaining_seconds,
    )
    .await
    .map_err(|e| e.to_string())
}

/// AI 快速恢复提示 - 轻量级恢复提示，无需完整分析
#[tauri::command]
pub async fn pomodoro_ai_quick_resume(
    ai_state: State<'_, AiState>,
    focus_goal: String,
    last_activity: String,
) -> Result<String, String> {
    let ai_service = {
        let guard = ai_state.0.lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("AI 服务未初始化")?.clone()
    };

    PomodoroAiService::get_quick_resume_tip(
        &ai_service,
        &focus_goal,
        &last_activity,
    )
    .await
    .map_err(|e| e.to_string())
}
