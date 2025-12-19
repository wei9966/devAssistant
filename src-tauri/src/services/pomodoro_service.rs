use crate::models::pomodoro::{
    CompletePomodoroRequest, CreatePomodoroRequest, FocusApp, PomodoroDailyStats,
    PomodoroPhase, PomodoroSession, PomodoroStatus,
};
use crate::services::ai_service::{AiService, ChatMessage};
use crate::services::prompt_db_service::PromptDbService;
use chrono::Local;
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;

/// 番茄钟服务
pub struct PomodoroService;

impl PomodoroService {
    // ==================== 会话管理 ====================

    /// 创建新的番茄钟会话
    pub fn create_session(
        conn: &Connection,
        request: CreatePomodoroRequest,
    ) -> Result<PomodoroSession> {
        let duration = request.duration_minutes.unwrap_or(25);
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            "INSERT INTO pomodoro_sessions (
                task_id, duration_minutes, status, phase, focus_goal, ai_suggestion, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                request.task_id,
                duration,
                PomodoroStatus::Pending.as_str(),
                PomodoroPhase::Prep.as_str(),
                request.focus_goal,
                request.ai_suggestion,
                now,
            ],
        )?;

        let session_id = conn.last_insert_rowid();
        Self::get_session_by_id(conn, session_id)
    }

    /// 开始番茄钟
    pub fn start_session(conn: &Connection, session_id: i64) -> Result<PomodoroSession> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            "UPDATE pomodoro_sessions
             SET status = ?1, phase = ?2, started_at = ?3
             WHERE id = ?4",
            params![
                PomodoroStatus::Focusing.as_str(),
                PomodoroPhase::Focusing.as_str(),
                now,
                session_id,
            ],
        )?;

        Self::get_session_by_id(conn, session_id)
    }

    /// 暂停番茄钟
    pub fn pause_session(conn: &Connection, session_id: i64) -> Result<PomodoroSession> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            "UPDATE pomodoro_sessions
             SET status = ?1, paused_at = ?2
             WHERE id = ?3",
            params![PomodoroStatus::Paused.as_str(), now, session_id],
        )?;

        Self::get_session_by_id(conn, session_id)
    }

    /// 恢复番茄钟
    pub fn resume_session(conn: &Connection, session_id: i64) -> Result<PomodoroSession> {
        conn.execute(
            "UPDATE pomodoro_sessions
             SET status = ?1, phase = ?2, paused_at = NULL
             WHERE id = ?3",
            params![
                PomodoroStatus::Focusing.as_str(),
                PomodoroPhase::Focusing.as_str(),
                session_id,
            ],
        )?;

        Self::get_session_by_id(conn, session_id)
    }

    /// 取消番茄钟
    pub fn cancel_session(conn: &Connection, session_id: i64) -> Result<PomodoroSession> {
        conn.execute(
            "UPDATE pomodoro_sessions
             SET status = ?1, phase = ?2
             WHERE id = ?3",
            params![
                PomodoroStatus::Cancelled.as_str(),
                PomodoroPhase::Report.as_str(),
                session_id,
            ],
        )?;

        Self::get_session_by_id(conn, session_id)
    }

    /// 完成番茄钟
    pub fn complete_session(
        conn: &Connection,
        session_id: i64,
        request: CompletePomodoroRequest,
    ) -> Result<PomodoroSession> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            "UPDATE pomodoro_sessions
             SET status = ?1, phase = ?2, completed_at = ?3, feedback = ?4, progress_update = ?5
             WHERE id = ?6",
            params![
                PomodoroStatus::Completed.as_str(),
                PomodoroPhase::Report.as_str(),
                now,
                request.feedback,
                request.progress_update,
                session_id,
            ],
        )?;

        // 更新每日统计
        Self::update_daily_stats(conn)?;

        Self::get_session_by_id(conn, session_id)
    }

    /// 记录分心
    pub fn record_distraction(conn: &Connection, session_id: i64) -> Result<PomodoroSession> {
        conn.execute(
            "UPDATE pomodoro_sessions
             SET distraction_count = distraction_count + 1
             WHERE id = ?1",
            params![session_id],
        )?;

        Self::get_session_by_id(conn, session_id)
    }

    /// 更新专注时间
    pub fn update_focus_time(
        conn: &Connection,
        session_id: i64,
        seconds: i32,
    ) -> Result<()> {
        // 获取会话信息
        let session = Self::get_session_by_id(conn, session_id)?;
        let total_seconds = session.duration_minutes * 60;
        let focus_rate = if total_seconds > 0 {
            (seconds as f64 / total_seconds as f64) * 100.0
        } else {
            0.0
        };

        conn.execute(
            "UPDATE pomodoro_sessions
             SET actual_focus_seconds = ?1, focus_rate = ?2
             WHERE id = ?3",
            params![seconds, focus_rate, session_id],
        )?;

        Ok(())
    }

    /// 获取会话详情
    pub fn get_session_by_id(conn: &Connection, session_id: i64) -> Result<PomodoroSession> {
        conn.query_row(
            "SELECT id, task_id, duration_minutes, status, phase, focus_goal, ai_suggestion,
                    actual_focus_seconds, distraction_count, focus_rate, feedback, progress_update,
                    started_at, paused_at, completed_at, created_at
             FROM pomodoro_sessions WHERE id = ?1",
            params![session_id],
            |row| {
                Ok(PomodoroSession {
                    id: Some(row.get(0)?),
                    task_id: row.get(1)?,
                    duration_minutes: row.get(2)?,
                    status: PomodoroStatus::from_string(&row.get::<_, String>(3)?),
                    phase: PomodoroPhase::from_string(&row.get::<_, String>(4)?),
                    focus_goal: row.get(5)?,
                    ai_suggestion: row.get(6)?,
                    actual_focus_seconds: row.get(7)?,
                    distraction_count: row.get(8)?,
                    focus_rate: row.get(9)?,
                    feedback: row.get(10)?,
                    progress_update: row.get(11)?,
                    started_at: row.get(12)?,
                    paused_at: row.get(13)?,
                    completed_at: row.get(14)?,
                    created_at: row.get(15)?,
                })
            },
        )
    }

    /// 获取会话详情（别名方法，符合API规范）
    pub fn get_session(conn: &Connection, session_id: i64) -> Result<Option<PomodoroSession>> {
        match Self::get_session_by_id(conn, session_id) {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 获取当前活跃会话
    pub fn get_active_session(conn: &Connection) -> Result<Option<PomodoroSession>> {
        let result = conn.query_row(
            "SELECT id, task_id, duration_minutes, status, phase, focus_goal, ai_suggestion,
                    actual_focus_seconds, distraction_count, focus_rate, feedback, progress_update,
                    started_at, paused_at, completed_at, created_at
             FROM pomodoro_sessions
             WHERE status IN (?1, ?2)
             ORDER BY created_at DESC
             LIMIT 1",
            params![
                PomodoroStatus::Focusing.as_str(),
                PomodoroStatus::Paused.as_str()
            ],
            |row| {
                Ok(PomodoroSession {
                    id: Some(row.get(0)?),
                    task_id: row.get(1)?,
                    duration_minutes: row.get(2)?,
                    status: PomodoroStatus::from_string(&row.get::<_, String>(3)?),
                    phase: PomodoroPhase::from_string(&row.get::<_, String>(4)?),
                    focus_goal: row.get(5)?,
                    ai_suggestion: row.get(6)?,
                    actual_focus_seconds: row.get(7)?,
                    distraction_count: row.get(8)?,
                    focus_rate: row.get(9)?,
                    feedback: row.get(10)?,
                    progress_update: row.get(11)?,
                    started_at: row.get(12)?,
                    paused_at: row.get(13)?,
                    completed_at: row.get(14)?,
                    created_at: row.get(15)?,
                })
            },
        );

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 获取今日会话列表
    pub fn get_today_sessions(conn: &Connection) -> Result<Vec<PomodoroSession>> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        Self::get_sessions_by_date(conn, &today)
    }

    /// 获取指定日期的会话列表
    pub fn get_sessions_by_date(conn: &Connection, date: &str) -> Result<Vec<PomodoroSession>> {
        let start_datetime = format!("{} 00:00:00", date);
        let end_datetime = format!("{} 23:59:59", date);

        let mut stmt = conn.prepare(
            "SELECT id, task_id, duration_minutes, status, phase, focus_goal, ai_suggestion,
                    actual_focus_seconds, distraction_count, focus_rate, feedback, progress_update,
                    started_at, paused_at, completed_at, created_at
             FROM pomodoro_sessions
             WHERE created_at >= ?1 AND created_at <= ?2
             ORDER BY created_at DESC",
        )?;

        let sessions = stmt
            .query_map(params![start_datetime, end_datetime], |row| {
                Ok(PomodoroSession {
                    id: Some(row.get(0)?),
                    task_id: row.get(1)?,
                    duration_minutes: row.get(2)?,
                    status: PomodoroStatus::from_string(&row.get::<_, String>(3)?),
                    phase: PomodoroPhase::from_string(&row.get::<_, String>(4)?),
                    focus_goal: row.get(5)?,
                    ai_suggestion: row.get(6)?,
                    actual_focus_seconds: row.get(7)?,
                    distraction_count: row.get(8)?,
                    focus_rate: row.get(9)?,
                    feedback: row.get(10)?,
                    progress_update: row.get(11)?,
                    started_at: row.get(12)?,
                    paused_at: row.get(13)?,
                    completed_at: row.get(14)?,
                    created_at: row.get(15)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(sessions)
    }

    /// 获取指定任务的会话历史
    pub fn get_task_sessions(conn: &Connection, task_id: i64) -> Result<Vec<PomodoroSession>> {
        let mut stmt = conn.prepare(
            "SELECT id, task_id, duration_minutes, status, phase, focus_goal, ai_suggestion,
                    actual_focus_seconds, distraction_count, focus_rate, feedback, progress_update,
                    started_at, paused_at, completed_at, created_at
             FROM pomodoro_sessions
             WHERE task_id = ?1
             ORDER BY created_at DESC",
        )?;

        let sessions = stmt
            .query_map(params![task_id], |row| {
                Ok(PomodoroSession {
                    id: Some(row.get(0)?),
                    task_id: row.get(1)?,
                    duration_minutes: row.get(2)?,
                    status: PomodoroStatus::from_string(&row.get::<_, String>(3)?),
                    phase: PomodoroPhase::from_string(&row.get::<_, String>(4)?),
                    focus_goal: row.get(5)?,
                    ai_suggestion: row.get(6)?,
                    actual_focus_seconds: row.get(7)?,
                    distraction_count: row.get(8)?,
                    focus_rate: row.get(9)?,
                    feedback: row.get(10)?,
                    progress_update: row.get(11)?,
                    started_at: row.get(12)?,
                    paused_at: row.get(13)?,
                    completed_at: row.get(14)?,
                    created_at: row.get(15)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(sessions)
    }

    /// 获取指定任务的会话历史（别名方法，符合API规范）
    pub fn get_sessions_by_task(conn: &Connection, task_id: i64) -> Result<Vec<PomodoroSession>> {
        Self::get_task_sessions(conn, task_id)
    }

    // ==================== 白名单应用管理 ====================

    /// 获取所有白名单应用
    pub fn get_all_focus_apps(conn: &Connection) -> Result<Vec<FocusApp>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, process_name, is_default, created_at
             FROM pomodoro_focus_apps
             ORDER BY is_default DESC, created_at DESC",
        )?;

        let apps = stmt
            .query_map([], |row| {
                Ok(FocusApp {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    process_name: row.get(2)?,
                    is_default: row.get::<_, i32>(3)? == 1,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(apps)
    }

    /// 获取所有白名单应用（别名方法，符合API规范）
    pub fn get_focus_apps(conn: &Connection) -> Result<Vec<FocusApp>> {
        Self::get_all_focus_apps(conn)
    }

    /// 添加白名单应用
    pub fn add_focus_app(
        conn: &Connection,
        name: String,
        process_name: Option<String>,
    ) -> Result<FocusApp> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            "INSERT INTO pomodoro_focus_apps (name, process_name, is_default, created_at)
             VALUES (?1, ?2, 0, ?3)",
            params![name, process_name, now],
        )?;

        let app_id = conn.last_insert_rowid();
        Self::get_focus_app_by_id(conn, app_id)
    }

    /// 删除白名单应用
    pub fn remove_focus_app(conn: &Connection, app_id: i64) -> Result<()> {
        conn.execute(
            "DELETE FROM pomodoro_focus_apps WHERE id = ?1",
            params![app_id],
        )?;
        Ok(())
    }

    /// 根据ID获取白名单应用
    fn get_focus_app_by_id(conn: &Connection, app_id: i64) -> Result<FocusApp> {
        conn.query_row(
            "SELECT id, name, process_name, is_default, created_at
             FROM pomodoro_focus_apps
             WHERE id = ?1",
            params![app_id],
            |row| {
                Ok(FocusApp {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    process_name: row.get(2)?,
                    is_default: row.get::<_, i32>(3)? == 1,
                    created_at: row.get(4)?,
                })
            },
        )
    }

    // ==================== 统计数据 ====================

    /// 获取今日统计
    pub fn get_today_stats(conn: &Connection) -> Result<PomodoroDailyStats> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        Self::get_or_create_stats_by_date(conn, &today)
    }

    /// 获取指定日期统计
    pub fn get_stats_by_date(conn: &Connection, date: &str) -> Result<Option<PomodoroDailyStats>> {
        let result = conn.query_row(
            "SELECT id, date, total_sessions, completed_sessions, total_focus_minutes,
                    avg_focus_rate, total_distractions, app_usage, ai_insight, created_at, updated_at
             FROM pomodoro_daily_stats
             WHERE date = ?1",
            params![date],
            |row| {
                Ok(PomodoroDailyStats {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    total_sessions: row.get(2)?,
                    completed_sessions: row.get(3)?,
                    total_focus_minutes: row.get(4)?,
                    avg_focus_rate: row.get(5)?,
                    total_distractions: row.get(6)?,
                    app_usage: row.get(7)?,
                    ai_insight: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            },
        );

        match result {
            Ok(stats) => Ok(Some(stats)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 获取或创建指定日期的统计
    fn get_or_create_stats_by_date(conn: &Connection, date: &str) -> Result<PomodoroDailyStats> {
        // 先尝试获取
        if let Some(stats) = Self::get_stats_by_date(conn, date)? {
            return Ok(stats);
        }

        // 不存在则创建
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO pomodoro_daily_stats (date, created_at, updated_at)
             VALUES (?1, ?2, ?3)",
            params![date, now, now],
        )?;

        Self::get_stats_by_date(conn, date)?
            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
    }

    /// 获取日期范围统计
    pub fn get_stats_range(
        conn: &Connection,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<PomodoroDailyStats>> {
        let mut stmt = conn.prepare(
            "SELECT id, date, total_sessions, completed_sessions, total_focus_minutes,
                    avg_focus_rate, total_distractions, app_usage, ai_insight, created_at, updated_at
             FROM pomodoro_daily_stats
             WHERE date >= ?1 AND date <= ?2
             ORDER BY date DESC",
        )?;

        let stats = stmt
            .query_map(params![start_date, end_date], |row| {
                Ok(PomodoroDailyStats {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    total_sessions: row.get(2)?,
                    completed_sessions: row.get(3)?,
                    total_focus_minutes: row.get(4)?,
                    avg_focus_rate: row.get(5)?,
                    total_distractions: row.get(6)?,
                    app_usage: row.get(7)?,
                    ai_insight: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(stats)
    }

    /// 更新每日统计
    fn update_daily_stats(conn: &Connection) -> Result<()> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // 计算今日统计数据
        let sessions = Self::get_sessions_by_date(conn, &today)?;
        let total_sessions = sessions.len() as i32;
        let completed_sessions = sessions
            .iter()
            .filter(|s| s.status == PomodoroStatus::Completed)
            .count() as i32;
        let total_focus_minutes: i32 = sessions
            .iter()
            .filter(|s| s.status == PomodoroStatus::Completed)
            .map(|s| s.actual_focus_seconds / 60)
            .sum();
        let avg_focus_rate = if completed_sessions > 0 {
            sessions
                .iter()
                .filter(|s| s.status == PomodoroStatus::Completed)
                .map(|s| s.focus_rate)
                .sum::<f64>()
                / completed_sessions as f64
        } else {
            0.0
        };
        let total_distractions: i32 = sessions.iter().map(|s| s.distraction_count).sum();

        // 确保记录存在
        Self::get_or_create_stats_by_date(conn, &today)?;

        // 更新统计
        conn.execute(
            "UPDATE pomodoro_daily_stats
             SET total_sessions = ?1, completed_sessions = ?2, total_focus_minutes = ?3,
                 avg_focus_rate = ?4, total_distractions = ?5, updated_at = ?6
             WHERE date = ?7",
            params![
                total_sessions,
                completed_sessions,
                total_focus_minutes,
                avg_focus_rate,
                total_distractions,
                now,
                today
            ],
        )?;

        Ok(())
    }

    /// 更新每日AI洞察
    pub fn update_daily_ai_insight(conn: &Connection, date: &str, insight: &str) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "UPDATE pomodoro_daily_stats SET ai_insight = ?1, updated_at = ?2 WHERE date = ?3",
            params![insight, now, date],
        )?;
        Ok(())
    }
}

/// AI 相关的番茄钟服务（异步方法）
/// 使用数据库存储的提示词模板，支持用户自定义
pub struct PomodoroAiService;

impl PomodoroAiService {
    /// AI 任务拆解建议 - 在开始专注前调用
    pub async fn get_task_breakdown(
        ai_service: &AiService,
        task_title: &str,
        task_description: Option<&str>,
        current_progress: Option<i32>,
        milestones: Option<&str>,
    ) -> anyhow::Result<String> {
        let mut vars = HashMap::new();
        vars.insert("task_title".to_string(), task_title.to_string());
        vars.insert("task_description".to_string(), task_description.unwrap_or("无").to_string());

        // 添加任务进度信息
        let progress_text = if let Some(progress) = current_progress {
            format!("当前进度：{}%", progress)
        } else {
            "当前进度：未开始（0%）".to_string()
        };
        vars.insert("current_progress".to_string(), progress_text);

        // 添加里程碑信息
        let milestone_text = if let Some(ms) = milestones {
            if ms.trim().is_empty() {
                "暂无里程碑记录".to_string()
            } else {
                format!("历史里程碑：\n{}", ms)
            }
        } else {
            "暂无里程碑记录".to_string()
        };
        vars.insert("milestones".to_string(), milestone_text);

        let rendered = PromptDbService::render_prompt_cached("pomodoro_task_breakdown", &vars)?;
        let messages = vec![
            ChatMessage::system(rendered.system.unwrap_or_default()),
            ChatMessage::user(rendered.user),
        ];
        ai_service.chat(messages).await
    }

    /// AI 专注力分析 - 在完成番茄钟后调用
    pub async fn get_focus_analysis(
        ai_service: &AiService,
        focus_goal: &str,
        duration_minutes: i32,
        actual_focus_seconds: i32,
        distraction_count: i32,
        user_feedback: Option<&str>,
    ) -> anyhow::Result<String> {
        let mut vars = HashMap::new();
        vars.insert("focus_goal".to_string(), focus_goal.to_string());
        vars.insert("duration_minutes".to_string(), duration_minutes.to_string());
        vars.insert("actual_focus_seconds".to_string(), actual_focus_seconds.to_string());
        vars.insert("actual_focus_minutes".to_string(), format!("{:.1}", actual_focus_seconds as f64 / 60.0));
        vars.insert("distraction_count".to_string(), distraction_count.to_string());
        vars.insert("user_feedback".to_string(), user_feedback.unwrap_or("用户未填写反馈").to_string());

        let rendered = PromptDbService::render_prompt_cached("pomodoro_focus_analysis", &vars)?;
        let messages = vec![
            ChatMessage::system(rendered.system.unwrap_or_default()),
            ChatMessage::user(rendered.user),
        ];
        ai_service.chat(messages).await
    }

    /// AI 每日复盘 - 在查看统计时调用
    pub async fn get_daily_review(
        ai_service: &AiService,
        date: &str,
        total_sessions: i32,
        completed_sessions: i32,
        total_focus_minutes: i32,
        avg_focus_rate: f64,
        app_usage: &str,
    ) -> anyhow::Result<String> {
        let mut vars = HashMap::new();
        vars.insert("date".to_string(), date.to_string());
        vars.insert("total_sessions".to_string(), total_sessions.to_string());
        vars.insert("completed_sessions".to_string(), completed_sessions.to_string());
        vars.insert("total_focus_minutes".to_string(), total_focus_minutes.to_string());
        vars.insert("avg_focus_rate".to_string(), format!("{:.1}", avg_focus_rate));
        vars.insert("app_usage".to_string(), app_usage.to_string());

        let rendered = PromptDbService::render_prompt_cached("pomodoro_daily_review", &vars)?;
        let messages = vec![
            ChatMessage::system(rendered.system.unwrap_or_default()),
            ChatMessage::user(rendered.user),
        ];
        ai_service.chat(messages).await
    }

    /// AI 进度评估 - 在提交反馈后调用
    pub async fn get_progress_evaluation(
        ai_service: &AiService,
        task_title: &str,
        focus_goal: &str,
        user_feedback: &str,
        previous_progress: Option<i32>,
    ) -> anyhow::Result<String> {
        let mut vars = HashMap::new();
        vars.insert("task_title".to_string(), task_title.to_string());
        vars.insert("focus_goal".to_string(), focus_goal.to_string());
        vars.insert("user_feedback".to_string(), user_feedback.to_string());
        let prev_text = previous_progress
            .map(|p| format!("之前进度：{}%", p))
            .unwrap_or_else(|| "首次专注".to_string());
        vars.insert("previous_progress_text".to_string(), prev_text);

        let rendered = PromptDbService::render_prompt_cached("pomodoro_progress_eval", &vars)?;
        let messages = vec![
            ChatMessage::system(rendered.system.unwrap_or_default()),
            ChatMessage::user(rendered.user),
        ];
        ai_service.chat(messages).await
    }

    // ==================== 任务中断与恢复相关 ====================

    /// AI 中断活动分析 - 分析中断期间的屏幕活动，判断与原任务的关联性
    pub async fn analyze_interruption(
        ai_service: &AiService,
        original_task: &str,
        focus_goal: &str,
        interruption_duration_minutes: i32,
        activity_summaries: &str,
    ) -> anyhow::Result<String> {
        let mut vars = HashMap::new();
        vars.insert("original_task".to_string(), original_task.to_string());
        vars.insert("focus_goal".to_string(), focus_goal.to_string());
        vars.insert("interruption_duration_minutes".to_string(), interruption_duration_minutes.to_string());
        vars.insert("activity_summaries".to_string(), activity_summaries.to_string());

        let rendered = PromptDbService::render_prompt_cached("pomodoro_interruption_analysis", &vars)?;
        let messages = vec![
            ChatMessage::system(rendered.system.unwrap_or_default()),
            ChatMessage::user(rendered.user),
        ];
        ai_service.chat(messages).await
    }

    /// AI 任务恢复建议 - 基于中断分析结果，生成恢复工作的建议
    pub async fn get_resume_suggestion(
        ai_service: &AiService,
        original_task: &str,
        focus_goal: &str,
        progress_before_interruption: Option<i32>,
        interruption_analysis: &str,
        elapsed_focus_seconds: i32,
        remaining_seconds: i32,
    ) -> anyhow::Result<String> {
        let mut vars = HashMap::new();
        vars.insert("original_task".to_string(), original_task.to_string());
        vars.insert("focus_goal".to_string(), focus_goal.to_string());
        let progress_text = progress_before_interruption
            .map(|p| format!("中断前进度：{}%", p))
            .unwrap_or_else(|| "进度：未记录".to_string());
        vars.insert("progress_text".to_string(), progress_text);
        vars.insert("elapsed_focus_minutes".to_string(), (elapsed_focus_seconds / 60).to_string());
        vars.insert("remaining_minutes".to_string(), (remaining_seconds / 60).to_string());
        vars.insert("interruption_analysis".to_string(), interruption_analysis.to_string());

        let rendered = PromptDbService::render_prompt_cached("pomodoro_resume_suggestion", &vars)?;
        let messages = vec![
            ChatMessage::system(rendered.system.unwrap_or_default()),
            ChatMessage::user(rendered.user),
        ];
        ai_service.chat(messages).await
    }

    /// AI 快速恢复提示 - 轻量级的恢复提示，不需要完整分析
    pub async fn get_quick_resume_tip(
        ai_service: &AiService,
        focus_goal: &str,
        last_activity: &str,
    ) -> anyhow::Result<String> {
        let mut vars = HashMap::new();
        vars.insert("focus_goal".to_string(), focus_goal.to_string());
        vars.insert("last_activity".to_string(), last_activity.to_string());

        let rendered = PromptDbService::render_prompt_cached("pomodoro_quick_resume", &vars)?;
        let messages = vec![
            ChatMessage::system(rendered.system.unwrap_or_default()),
            ChatMessage::user(rendered.user),
        ];
        ai_service.chat(messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_session() {
        let conn = setup_test_db();

        let request = CreatePomodoroRequest {
            task_id: None,
            duration_minutes: Some(25),
            focus_goal: Some("测试专注目标".to_string()),
            focus_apps: None,
        };

        let session = PomodoroService::create_session(&conn, request).unwrap();
        assert!(session.id.is_some());
        assert_eq!(session.duration_minutes, 25);
        assert_eq!(session.status, PomodoroStatus::Pending);
        assert_eq!(session.phase, PomodoroPhase::Prep);
    }

    #[test]
    fn test_start_session() {
        let conn = setup_test_db();

        let request = CreatePomodoroRequest {
            task_id: None,
            duration_minutes: Some(25),
            focus_goal: None,
            focus_apps: None,
        };

        let session = PomodoroService::create_session(&conn, request).unwrap();
        let session_id = session.id.unwrap();

        let started_session = PomodoroService::start_session(&conn, session_id).unwrap();
        assert_eq!(started_session.status, PomodoroStatus::Focusing);
        assert_eq!(started_session.phase, PomodoroPhase::Focusing);
        assert!(started_session.started_at.is_some());
    }

    #[test]
    fn test_add_and_get_focus_apps() {
        let conn = setup_test_db();

        let app = PomodoroService::add_focus_app(
            &conn,
            "Visual Studio Code".to_string(),
            Some("code.exe".to_string()),
        )
        .unwrap();

        assert!(app.id.is_some());
        assert_eq!(app.name, "Visual Studio Code");

        let apps = PomodoroService::get_all_focus_apps(&conn).unwrap();
        assert_eq!(apps.len(), 1);
    }

    #[test]
    fn test_get_today_stats() {
        let conn = setup_test_db();

        let stats = PomodoroService::get_today_stats(&conn).unwrap();
        assert!(stats.id.is_some());
        assert_eq!(stats.total_sessions, 0);
    }
}
