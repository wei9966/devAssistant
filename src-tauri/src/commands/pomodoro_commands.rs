use crate::db::connection::DbConnection;
use crate::models::pomodoro::{
    CompletePomodoroRequest, CreatePomodoroRequest, FocusApp, PomodoroDailyStats,
    PomodoroSession,
};
use crate::services::pomodoro_service::{PomodoroService, PomodoroAiService, SessionActivity};
use crate::services::prompt_db_service::PromptDbService;
use crate::services::ai_service::ChatMessage;
use crate::commands::ai_commands::AiState;
use tauri::State;
use std::collections::HashMap;

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

/// 更新会话的AI分析结果
#[tauri::command]
pub fn update_pomodoro_ai_analysis(
    db: State<DbConnection>,
    session_id: i64,
    ai_analysis: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::update_session_ai_analysis(&conn, session_id, &ai_analysis).map_err(|e| e.to_string())
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

/// 获取日期范围内的会话列表
#[tauri::command]
pub fn get_pomodoro_sessions_range(
    db: State<DbConnection>,
    start_date: String,
    end_date: String,
) -> Result<Vec<PomodoroSession>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::get_sessions_by_date_range(&conn, &start_date, &end_date).map_err(|e| e.to_string())
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
    db: State<'_, DbConnection>,
    task_id: Option<i64>,
    task_title: String,
    task_description: Option<String>,
    current_progress: Option<i32>,
) -> Result<String, String> {
    // 克隆 AI 服务以避免持有 MutexGuard 跨越 await
    let ai_service = {
        let guard = ai_state.0.lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("AI 服务未初始化")?.clone()
    };

    // 如果提供了 task_id，从数据库获取里程碑信息
    let milestones_text = if let Some(tid) = task_id {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        match crate::services::milestone_service::MilestoneService::get_task_milestones(&conn, tid) {
            Ok(milestones) => {
                if milestones.is_empty() {
                    None
                } else {
                    // 格式化里程碑信息
                    let formatted = milestones
                        .iter()
                        .map(|m| {
                            let progress = m.progress_snapshot
                                .map(|p| format!("{}%", p))
                                .unwrap_or_else(|| "未记录".to_string());
                            format!("- {} (进度: {}) - {}",
                                m.title,
                                progress,
                                m.description.as_deref().unwrap_or(""))
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    Some(formatted)
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };

    PomodoroAiService::get_task_breakdown(
        &ai_service,
        &task_title,
        task_description.as_deref(),
        current_progress,
        milestones_text.as_deref(),
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

/// 获取番茄钟期间的活动记录
#[tauri::command]
pub fn pomodoro_get_session_activities(
    db: State<DbConnection>,
    session_id: i64,
) -> Result<Vec<SessionActivity>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::get_session_activities(&conn, session_id).map_err(|e| e.to_string())
}

/// 获取番茄钟期间的应用使用统计
#[tauri::command]
pub fn pomodoro_get_session_app_usage(
    db: State<DbConnection>,
    session_id: i64,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PomodoroService::get_session_app_usage(&conn, session_id).map_err(|e| e.to_string())
}

/// AI 会话分析 - 分析番茄钟期间的任务相关性
#[tauri::command]
pub async fn pomodoro_ai_analyze_session(
    ai_state: State<'_, AiState>,
    db: State<'_, DbConnection>,
    session_id: i64,
) -> Result<String, String> {
    // 获取 session 信息
    let session = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        PomodoroService::get_session_by_id(&conn, session_id).map_err(|e| e.to_string())?
    };

    // 获取任务信息（如果有关联任务）
    let (task_title, task_description) = if let Some(task_id) = session.task_id {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        // 直接查询任务信息
        let result = conn.query_row(
            "SELECT title, description FROM tasks WHERE id = ?",
            [task_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                ))
            },
        );
        match result {
            Ok((title, description)) => (title, description),
            Err(_) => ("未关联任务".to_string(), None),
        }
    } else {
        ("未关联任务".to_string(), None)
    };

    // 获取会话活动记录
    let activities = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        PomodoroService::get_session_activities(&conn, session_id).map_err(|e| e.to_string())?
    };

    // 生成活动摘要
    let activities_summary = if activities.is_empty() {
        "暂无活动记录".to_string()
    } else {
        // 统计活动中出现的应用和窗口标题
        let mut app_activities = std::collections::HashMap::new();
        for activity in &activities {
            if let Some(ref app_name) = activity.app_name {
                let entry = app_activities.entry(app_name.clone()).or_insert_with(Vec::new);
                if let Some(ref window_title) = activity.window_title {
                    entry.push(window_title.clone());
                }
            }
        }

        // 格式化为摘要文本
        let mut summary_parts = Vec::new();
        for (app_name, windows) in app_activities.iter().take(10) {
            if windows.is_empty() {
                summary_parts.push(format!("- {}", app_name));
            } else {
                let unique_windows: std::collections::HashSet<_> = windows.iter().collect();
                let window_list = unique_windows.iter().take(3)
                    .map(|w| w.as_str())
                    .collect::<Vec<_>>()
                    .join("、");
                summary_parts.push(format!("- {}：{}", app_name, window_list));
            }
        }

        if summary_parts.is_empty() {
            "暂无详细活动记录".to_string()
        } else {
            summary_parts.join("\n")
        }
    };

    // 获取应用使用统计
    let app_usage_json = session.app_usage.as_deref().unwrap_or("[]");
    let app_usage_display = if app_usage_json == "[]" || app_usage_json.is_empty() {
        "暂无应用使用记录".to_string()
    } else {
        app_usage_json.to_string()
    };

    // 获取专注目标
    let focus_goal = session.focus_goal.as_deref().unwrap_or("未设置专注目标");

    // 计算专注率
    let focus_rate = if session.duration_minutes > 0 {
        (session.actual_focus_seconds as f64 / (session.duration_minutes * 60) as f64) * 100.0
    } else {
        0.0
    };

    // 从数据库获取prompt
    let rendered_prompt = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;

        let mut vars = HashMap::new();
        vars.insert("task_title".to_string(), task_title.clone());
        vars.insert("focus_goal".to_string(), focus_goal.to_string());
        vars.insert("task_description".to_string(), task_description.clone().unwrap_or_else(|| "无".to_string()));
        vars.insert("duration_minutes".to_string(), session.duration_minutes.to_string());
        vars.insert("actual_focus_minutes".to_string(), format!("{:.1}", session.actual_focus_seconds as f64 / 60.0));
        vars.insert("focus_rate".to_string(), format!("{:.1}", focus_rate));
        vars.insert("app_usage".to_string(), app_usage_display);
        vars.insert("activities_summary".to_string(), activities_summary.clone());

        PromptDbService::render_prompt(&conn, "pomodoro_session_analysis", &vars)
            .map_err(|e| e.to_string())?
    };

    // 克隆 AI 服务以避免持有 MutexGuard 跨越 await
    let ai_service = {
        let guard = ai_state.0.lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("AI 服务未初始化")?.clone()
    };

    // 调用 AI 分析
    let mut messages = Vec::new();
    if let Some(system_prompt) = rendered_prompt.system {
        messages.push(ChatMessage::system(system_prompt));
    }
    messages.push(ChatMessage::user(rendered_prompt.user));

    ai_service.chat(messages).await.map_err(|e| e.to_string())
}
