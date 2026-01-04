// AI Functions Service
// 实现所有 AI Function Call 的执行逻辑

use crate::models::pomodoro::{PomodoroSession, CreatePomodoroRequest};
use crate::models::task::{Task, TaskQuadrant, TaskStatus};
use crate::services::pomodoro_service::PomodoroService;
use crate::services::report_service::{DailyReport, ReportService};
use crate::services::sql_service::{SqlRecord, SqlService};
use crate::services::task_service::TaskService;
use crate::services::time_parser_service::TimeParserService;
use anyhow::{anyhow, Result};
use chrono::{Local, NaiveDate, Datelike};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// ==================== Response Models ====================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksResponse {
    pub total: usize,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskResponse {
    pub success: bool,
    pub task: Option<Task>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlSearchResponse {
    pub total: usize,
    pub sql_list: Vec<SqlRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeStatsResponse {
    pub total_hours: f32,
    pub breakdown: Vec<TimeBreakdownItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBreakdownItem {
    pub period: String,
    pub hours: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroResponse {
    pub total: usize,
    pub records: Vec<PomodoroSession>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPomodoroResponse {
    pub success: bool,
    pub session: Option<PomodoroSession>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyReportResponse {
    pub success: bool,
    pub report: Option<DailyReport>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyReportResponse {
    pub success: bool,
    pub reports: Vec<DailyReport>,
    pub summary: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EfficiencyAnalysis {
    pub overall_score: f32,
    pub focus_time_hours: f32,
    pub task_completion_rate: f32,
    pub productivity_trend: String,
    pub suggestions: Vec<String>,
}

// ==================== AI Functions Service ====================

pub struct AiFunctionsService;

impl AiFunctionsService {
    // ==================== 1. get_tasks ====================

    /// 查询任务
    /// 参数:
    /// - status: 任务状态过滤 (todo, active, done, deferred)
    /// - quadrant: 象限过滤 (urgent_important, urgent_not_important, not_urgent_important, not_urgent_not_important)
    /// - date_range: 日期范围 (today, yesterday, this_week, last_week, this_month)
    /// - keyword: 关键词搜索 (标题/描述)
    /// - limit: 返回数量限制
    pub fn get_tasks(
        conn: &Connection,
        status: Option<&str>,
        quadrant: Option<&str>,
        date_range: Option<&str>,
        keyword: Option<&str>,
        limit: Option<usize>,
    ) -> Result<TasksResponse> {
        let mut tasks = if let Some(status_str) = status {
            // 按状态查询
            match status_str {
                "done" => {
                    let days = if let Some(range) = date_range {
                        Self::parse_date_range_days(range)?
                    } else {
                        7 // 默认最近7天的已完成任务
                    };
                    TaskService::get_completed_tasks(conn, days)?
                }
                _ => TaskService::get_all_tasks(conn)?,
            }
        } else if let Some(quadrant_str) = quadrant {
            // 按象限查询
            let q = TaskQuadrant::from_str(quadrant_str);
            TaskService::get_tasks_by_quadrant(conn, q)?
        } else if let Some(range) = date_range {
            // 按日期范围查询
            let (start_date, end_date) = Self::parse_date_range(range)?;
            TaskService::get_tasks_by_date_range(conn, &start_date, &end_date)?
        } else {
            // 默认查询所有未完成任务
            TaskService::get_all_tasks(conn)?
        };

        // 关键词过滤
        if let Some(kw) = keyword {
            let kw_lower = kw.to_lowercase();
            tasks.retain(|t| {
                t.title.to_lowercase().contains(&kw_lower)
                    || t.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&kw_lower))
                        .unwrap_or(false)
            });
        }

        // 状态过滤（当未指定status但指定了其他条件时）
        if status.is_none() && (quadrant.is_some() || date_range.is_some()) {
            // 保留所有状态
        } else if let Some(status_str) = status {
            let target_status = TaskStatus::from_str(status_str);
            tasks.retain(|t| t.status == target_status);
        }

        // 限制数量
        if let Some(limit_count) = limit {
            tasks.truncate(limit_count);
        }

        let total = tasks.len();

        Ok(TasksResponse { total, tasks })
    }

    // ==================== 2. create_task ====================

    /// 创建任务
    pub fn create_task(
        conn: &Connection,
        title: &str,
        description: Option<&str>,
        quadrant: Option<&str>,
        due_date: Option<&str>,
        tags: Option<Vec<String>>,
    ) -> Result<CreateTaskResponse> {
        use crate::models::task::{TaskCategory, TaskPriority};

        // 根据象限推断优先级
        let priority = if let Some(q) = quadrant {
            match q {
                "urgent_important" => TaskPriority::High,
                "urgent_not_important" => TaskPriority::Medium,
                _ => TaskPriority::Low,
            }
        } else {
            TaskPriority::Medium
        };

        // 创建任务
        let task_id = TaskService::create_task(
            conn,
            title,
            description,
            TaskCategory::Other,
            priority,
        )?;

        // 智能解析截止日期（支持自然语言如"下周五"、"3天后"等）
        let parsed_due_date = due_date.and_then(|d| {
            Self::smart_parse_date(d).or_else(|| {
                // 如果无法解析，仍尝试保留原值（让数据库层处理）
                Some(d.to_string())
            })
        });

        // 更新象限和截止日期
        if quadrant.is_some() || parsed_due_date.is_some() {
            let quad = quadrant.map(TaskQuadrant::from_str);
            TaskService::update_task(
                conn,
                task_id,
                None,
                None,
                None,
                None,
                None,
                None,
                quad,
                parsed_due_date.as_deref(),
                None,
                None,
                None,
            )?;
        }

        // 添加标签
        if let Some(tag_names) = tags {
            use crate::services::tag_service::TagService;
            for tag_name in tag_names {
                // 查找已存在的标签
                let existing_tags = TagService::get_all_tags(conn)?;
                let tag_id = if let Some(tag) = existing_tags.iter().find(|t| t.name == tag_name) {
                    tag.id.unwrap()
                } else {
                    // 创建新标签
                    TagService::create_tag(conn, &tag_name, "#6366f1")?
                };
                let _ = TagService::add_tag_to_task(conn, task_id, tag_id);
            }
        }

        // 获取创建的任务
        let mut stmt = conn.prepare(
            "SELECT id, title, description, category, priority, status, git_branch,
                    created_at, started_at, last_active_at, completed_at,
                    estimated_hours, actual_hours, context_json, notes, quadrant,
                    due_date, registered_at, display_date, scheduled_start_time, progress
             FROM tasks WHERE id = ?",
        )?;

        let task = stmt.query_row([task_id], |row| {
            use crate::models::task::{TaskCategory, TaskPriority, TaskStatus, TaskQuadrant};

            let quadrant_str: Option<String> = row.get(15)?;
            let quadrant = quadrant_str.map(|s| TaskQuadrant::from_str(&s));

            Ok(Task {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                description: row.get(2)?,
                category: TaskCategory::from_str(&row.get::<_, String>(3)?),
                priority: TaskPriority::from_i32(row.get(4)?),
                status: TaskStatus::from_str(&row.get::<_, String>(5)?),
                git_branch: row.get(6)?,
                created_at: row.get(7)?,
                started_at: row.get(8)?,
                last_active_at: row.get(9)?,
                completed_at: row.get(10)?,
                estimated_hours: row.get(11)?,
                actual_hours: row.get(12)?,
                context: None,
                notes: row.get(14)?,
                quadrant,
                tags: None,
                due_date: row.get(16)?,
                registered_at: row.get(17)?,
                display_date: row.get(18)?,
                scheduled_start_time: row.get(19)?,
                progress: row.get(20).unwrap_or(0),
            })
        })?;

        Ok(CreateTaskResponse {
            success: true,
            task: Some(task),
            message: "任务创建成功".to_string(),
        })
    }

    // ==================== 3. update_task ====================

    /// 更新任务
    pub fn update_task(
        conn: &Connection,
        task_id: i64,
        title: Option<&str>,
        status: Option<&str>,
        quadrant: Option<&str>,
        due_date: Option<&str>,
    ) -> Result<UpdateTaskResponse> {
        // 如果更新状态
        if let Some(status_str) = status {
            match status_str {
                "active" => TaskService::start_task(conn, task_id)?,
                "done" => TaskService::complete_task(conn, task_id)?,
                "deferred" => TaskService::defer_task(conn, task_id)?,
                _ => {}
            }
        }

        // 智能解析截止日期（支持自然语言如"下周五"、"3天后"等）
        let parsed_due_date = due_date.and_then(|d| {
            Self::smart_parse_date(d).or_else(|| {
                // 如果无法解析，仍尝试保留原值
                Some(d.to_string())
            })
        });

        // 更新其他字段
        let quad = quadrant.map(TaskQuadrant::from_str);
        TaskService::update_task(
            conn,
            task_id,
            title,
            None,
            None,
            None,
            None,
            None,
            quad,
            parsed_due_date.as_deref(),
            None,
            None,
            None,
        )?;

        Ok(UpdateTaskResponse {
            success: true,
            message: "任务更新成功".to_string(),
        })
    }

    // ==================== 4. search_sql ====================

    /// 搜索 SQL
    pub fn search_sql(
        conn: &Connection,
        keyword: Option<&str>,
        sql_type: Option<&str>,
        favorite_only: Option<bool>,
        limit: Option<usize>,
    ) -> Result<SqlSearchResponse> {
        let limit_count = limit.unwrap_or(50);

        let sql_list = if favorite_only == Some(true) {
            SqlService::get_favorite_sqls(conn)?
        } else {
            SqlService::get_recent_sqls(conn, limit_count)?
        };

        // 过滤
        let mut filtered = sql_list;

        if let Some(kw) = keyword {
            let kw_lower = kw.to_lowercase();
            filtered.retain(|s| {
                s.sql_text.to_lowercase().contains(&kw_lower)
                    || s.name
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(&kw_lower))
                        .unwrap_or(false)
            });
        }

        if let Some(sql_type_str) = sql_type {
            let sql_type_upper = sql_type_str.to_uppercase();
            filtered.retain(|s| {
                s.sql_type
                    .as_ref()
                    .map(|t| t.to_uppercase() == sql_type_upper)
                    .unwrap_or(false)
            });
        }

        filtered.truncate(limit_count);
        let total = filtered.len();

        Ok(SqlSearchResponse {
            total,
            sql_list: filtered,
        })
    }

    // ==================== 5. get_time_stats ====================

    /// 获取时间统计
    pub fn get_time_stats(
        conn: &Connection,
        date_range: Option<&str>,
        group_by: Option<&str>,
    ) -> Result<TimeStatsResponse> {
        let (start_date, end_date) = if let Some(range) = date_range {
            Self::parse_date_range(range)?
        } else {
            Self::parse_date_range("this_week")?
        };

        // 查询时间范围内已完成的任务
        let tasks = TaskService::get_tasks_by_date_range(conn, &start_date, &end_date)?;

        let mut total_hours = 0.0_f32;
        let mut breakdown = Vec::new();

        if group_by == Some("day") {
            // 按天分组
            use std::collections::HashMap;
            let mut day_hours: HashMap<String, f32> = HashMap::new();

            for task in tasks.iter().filter(|t| t.status == TaskStatus::Done) {
                if let Some(hours) = task.actual_hours {
                    if let Some(completed_at) = &task.completed_at {
                        let day = &completed_at[..10]; // YYYY-MM-DD
                        *day_hours.entry(day.to_string()).or_insert(0.0) += hours;
                        total_hours += hours;
                    }
                }
            }

            for (day, hours) in day_hours {
                breakdown.push(TimeBreakdownItem {
                    period: day,
                    hours,
                });
            }
            breakdown.sort_by(|a, b| a.period.cmp(&b.period));
        } else {
            // 默认总计
            for task in tasks.iter().filter(|t| t.status == TaskStatus::Done) {
                if let Some(hours) = task.actual_hours {
                    total_hours += hours;
                }
            }
            breakdown.push(TimeBreakdownItem {
                period: format!("{} to {}", start_date, end_date),
                hours: total_hours,
            });
        }

        Ok(TimeStatsResponse {
            total_hours,
            breakdown,
        })
    }

    // ==================== 6. get_pomodoro_records ====================

    /// 获取番茄钟记录
    pub fn get_pomodoro_records(
        conn: &Connection,
        date_range: Option<&str>,
        task_id: Option<i64>,
    ) -> Result<PomodoroResponse> {
        let records = if let Some(range) = date_range {
            let (start_date, _end_date) = Self::parse_date_range(range)?;
            // 使用 get_sessions_by_date 获取单日记录
            PomodoroService::get_sessions_by_date(conn, &start_date)?
        } else {
            // 默认今天
            PomodoroService::get_today_sessions(conn)?
        };

        let filtered = if let Some(tid) = task_id {
            records
                .into_iter()
                .filter(|r| r.task_id == Some(tid))
                .collect()
        } else {
            records
        };

        let total = filtered.len();

        Ok(PomodoroResponse {
            total,
            records: filtered,
        })
    }

    // ==================== 7. start_pomodoro ====================

    /// 开始番茄钟
    pub fn start_pomodoro(
        conn: &Connection,
        duration_minutes: Option<i32>,
        task_id: Option<i64>,
    ) -> Result<StartPomodoroResponse> {
        let request = CreatePomodoroRequest {
            task_id,
            duration_minutes,
            focus_goal: None,
            focus_apps: None,
            ai_suggestion: None,
        };

        let session = PomodoroService::create_session(conn, request)?;
        let started = PomodoroService::start_session(conn, session.id.unwrap())?;

        Ok(StartPomodoroResponse {
            success: true,
            session: Some(started),
            message: "番茄钟已开始".to_string(),
        })
    }

    // ==================== 8. generate_daily_report ====================

    /// 生成日报（同步部分，需配合异步调用）
    pub fn generate_daily_report(
        conn: &Connection,
        date: Option<&str>,
        _style: Option<&str>,
        _include_time_stats: Option<bool>,
    ) -> Result<DailyReportResponse> {
        // 智能解析日期（支持"昨天"、"前天"等自然语言）
        let target_date = if let Some(d) = date {
            Self::smart_parse_date(d).unwrap_or_else(|| d.to_string())
        } else {
            Local::now().format("%Y-%m-%d").to_string()
        };

        // 检查是否已存在
        if let Ok(Some(existing)) = ReportService::get_report(conn, &target_date) {
            return Ok(DailyReportResponse {
                success: true,
                report: Some(existing),
                message: "日报已存在".to_string(),
            });
        }

        // 返回提示，需要异步生成
        Ok(DailyReportResponse {
            success: false,
            report: None,
            message: "日报需要异步生成，请调用相应的异步接口".to_string(),
        })
    }

    // ==================== 9. generate_weekly_report ====================

    /// 生成周报
    pub fn generate_weekly_report(
        conn: &Connection,
        week: Option<&str>,
        _style: Option<&str>,
        _include_comparison: Option<bool>,
    ) -> Result<WeeklyReportResponse> {
        let (start_date, end_date) = if let Some(w) = week {
            Self::parse_date_range(w)?
        } else {
            Self::parse_date_range("this_week")?
        };

        // 获取这一周的所有日报
        let reports = ReportService::list_reports(conn, Some(7), None)?;

        let week_reports: Vec<DailyReport> = reports
            .into_iter()
            .filter(|r| r.date >= start_date && r.date <= end_date)
            .collect();

        let summary = format!(
            "本周共有 {} 份日报，时间范围：{} 至 {}",
            week_reports.len(),
            start_date,
            end_date
        );

        Ok(WeeklyReportResponse {
            success: true,
            reports: week_reports,
            summary,
            message: "周报生成成功".to_string(),
        })
    }

    // ==================== 10. analyze_efficiency ====================

    /// 效率分析
    pub fn analyze_efficiency(
        conn: &Connection,
        date_range: Option<&str>,
        _focus: Option<&str>,
    ) -> Result<EfficiencyAnalysis> {
        let (start_date, end_date) = if let Some(range) = date_range {
            Self::parse_date_range(range)?
        } else {
            Self::parse_date_range("this_week")?
        };

        // 获取番茄钟记录 - 使用 get_stats_range
        let stats_list = PomodoroService::get_stats_range(conn, &start_date, &end_date)?;

        // 计算总专注时长和平均专注率
        let mut total_focus_minutes = 0;
        let mut total_focus_rate = 0.0;

        for stat in &stats_list {
            total_focus_minutes += stat.total_focus_minutes;
            total_focus_rate += stat.avg_focus_rate;
        }

        let focus_time_hours = total_focus_minutes as f32 / 60.0;
        let avg_focus_rate = if !stats_list.is_empty() {
            total_focus_rate / stats_list.len() as f64
        } else {
            0.0
        };

        // 计算任务完成率
        let tasks = TaskService::get_tasks_by_date_range(conn, &start_date, &end_date)?;
        let total_tasks = tasks.len() as f32;
        let completed_tasks = tasks.iter().filter(|t| t.status == TaskStatus::Done).count() as f32;
        let task_completion_rate = if total_tasks > 0.0 {
            (completed_tasks / total_tasks) * 100.0
        } else {
            0.0
        };

        let overall_score = ((avg_focus_rate * 0.5 + (task_completion_rate as f64 / 100.0) * 0.5) * 100.0) as f32;

        // 生成建议
        let mut suggestions = Vec::new();
        if avg_focus_rate < 0.7 {
            suggestions.push("建议减少分心次数，提高专注度".to_string());
        }
        if task_completion_rate < 70.0 {
            suggestions.push("建议合理安排任务优先级，提高完成率".to_string());
        }
        if focus_time_hours < 4.0 {
            suggestions.push("建议增加每日专注时长".to_string());
        }

        let productivity_trend = if overall_score >= 80.0 {
            "优秀".to_string()
        } else if overall_score >= 60.0 {
            "良好".to_string()
        } else {
            "需改进".to_string()
        };

        Ok(EfficiencyAnalysis {
            overall_score,
            focus_time_hours,
            task_completion_rate,
            productivity_trend,
            suggestions,
        })
    }

    // ==================== Helper Functions ====================

    /// 智能解析日期
    /// 支持标准格式（YYYY-MM-DD）和自然语言（今天、明天、下周五、3天后等）
    fn smart_parse_date(date_str: &str) -> Option<String> {
        let date_str = date_str.trim();

        // 1. 尝试标准日期格式 YYYY-MM-DD
        if NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok() {
            return Some(date_str.to_string());
        }

        // 2. 尝试解析自然语言时间表达
        if let Some(parsed_date) = TimeParserService::parse_natural_time(date_str) {
            let formatted = parsed_date.format("%Y-%m-%d").to_string();
            log::info!("[AI Functions] 🕐 时间解析: \"{}\" → {}", date_str, formatted);
            return Some(formatted);
        }

        // 3. 尝试其他常见格式
        // MM-DD 格式（自动补全年份）
        if let Ok(parsed) = NaiveDate::parse_from_str(
            &format!("{}-{}", Local::now().year(), date_str),
            "%Y-%m-%d"
        ) {
            // 如果日期已过，使用明年
            let today = Local::now().date_naive();
            let result = if parsed < today {
                NaiveDate::from_ymd_opt(today.year() + 1, parsed.month(), parsed.day())
                    .unwrap_or(parsed)
            } else {
                parsed
            };
            return Some(result.format("%Y-%m-%d").to_string());
        }

        // 4. 无法解析，返回 None（调用者可以选择原样保留或报错）
        log::warn!("[AI Functions] ⚠️ 无法解析日期: \"{}\"", date_str);
        None
    }

    /// 解析日期范围
    fn parse_date_range(range: &str) -> Result<(String, String)> {
        let today = Local::now().date_naive();

        match range {
            "today" => {
                let date_str = today.format("%Y-%m-%d").to_string();
                Ok((date_str.clone(), date_str))
            }
            "yesterday" => {
                let yesterday = today - chrono::Duration::days(1);
                let date_str = yesterday.format("%Y-%m-%d").to_string();
                Ok((date_str.clone(), date_str))
            }
            "this_week" => {
                let weekday = today.weekday().num_days_from_monday();
                let start = today - chrono::Duration::days(weekday as i64);
                let end = start + chrono::Duration::days(6);
                Ok((
                    start.format("%Y-%m-%d").to_string(),
                    end.format("%Y-%m-%d").to_string(),
                ))
            }
            "last_week" => {
                let weekday = today.weekday().num_days_from_monday();
                let this_week_start = today - chrono::Duration::days(weekday as i64);
                let start = this_week_start - chrono::Duration::days(7);
                let end = start + chrono::Duration::days(6);
                Ok((
                    start.format("%Y-%m-%d").to_string(),
                    end.format("%Y-%m-%d").to_string(),
                ))
            }
            "this_month" => {
                let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                    .ok_or_else(|| anyhow!("Invalid date"))?;
                let end = if today.month() == 12 {
                    NaiveDate::from_ymd_opt(today.year() + 1, 1, 1)
                } else {
                    NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1)
                }
                .ok_or_else(|| anyhow!("Invalid date"))?
                    - chrono::Duration::days(1);
                Ok((
                    start.format("%Y-%m-%d").to_string(),
                    end.format("%Y-%m-%d").to_string(),
                ))
            }
            _ => Err(anyhow!("不支持的日期范围: {}", range)),
        }
    }

    /// 解析日期范围为天数（用于已完成任务查询）
    fn parse_date_range_days(range: &str) -> Result<i64> {
        match range {
            "today" => Ok(1),
            "yesterday" => Ok(1),
            "this_week" => Ok(7),
            "last_week" => Ok(7),
            "this_month" => Ok(30),
            _ => Ok(7), // 默认7天
        }
    }
}
