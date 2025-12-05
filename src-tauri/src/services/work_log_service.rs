use crate::models::work_log::WorkLog;
use crate::models::screen_context::{
    ActiveTimeRange, ActivityDistribution, AppUsage, DailyActiveHours, DayContextSummary,
    KeyActivity, ScreenContext, WeekContextSummary, WeeklyActivitySummary, WeeklyAppRanking,
    WorkPatterns,
};
use crate::services::context_store_service::ContextStoreService;
use anyhow::Result;
use chrono::{DateTime, Duration, Local, NaiveDateTime, Timelike};
use rusqlite::{params, Connection};
use std::collections::HashMap;

pub struct WorkLogService;

impl WorkLogService {
    /// 保存或更新工作日志
    /// 如果指定日期的日志已存在,则更新;否则创建新记录
    pub fn save_work_log(
        conn: &Connection,
        date: &str,
        log_type: &str,
        content: &str,
        ai_generated: bool,
    ) -> Result<()> {
        // 检查是否已存在该日期的记录
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM work_logs WHERE date = ?",
            params![date],
            |row| row.get(0),
        )?;

        if exists > 0 {
            // 更新现有记录
            conn.execute(
                "UPDATE work_logs
                 SET log_type = ?, content = ?, ai_generated = ?, updated_at = datetime('now', 'localtime')
                 WHERE date = ?",
                params![log_type, content, ai_generated, date],
            )?;
        } else {
            // 创建新记录
            conn.execute(
                "INSERT INTO work_logs (date, log_type, content, ai_generated, created_at)
                 VALUES (?, ?, ?, ?, datetime('now', 'localtime'))",
                params![date, log_type, content, ai_generated],
            )?;
        }

        Ok(())
    }

    /// 获取指定日期的工作日志
    pub fn get_work_log(conn: &Connection, date: &str) -> Result<Option<WorkLog>> {
        let mut stmt = conn.prepare(
            "SELECT id, date, log_type, content, ai_generated, created_at, updated_at
             FROM work_logs
             WHERE date = ?",
        )?;

        let mut rows = stmt.query(params![date])?;

        if let Some(row) = rows.next()? {
            Ok(Some(WorkLog {
                id: Some(row.get(0)?),
                date: row.get(1)?,
                log_type: row.get(2)?,
                content: row.get(3)?,
                ai_generated: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 获取最近N天的工作日志
    pub fn get_recent_work_logs(conn: &Connection, days: i32) -> Result<Vec<WorkLog>> {
        let mut stmt = conn.prepare(
            "SELECT id, date, log_type, content, ai_generated, created_at, updated_at
             FROM work_logs
             WHERE date >= date('now', '-' || ? || ' days')
             ORDER BY date DESC",
        )?;

        let logs = stmt
            .query_map(params![days], |row| {
                Ok(WorkLog {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    log_type: row.get(2)?,
                    content: row.get(3)?,
                    ai_generated: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// 删除指定日期的工作日志
    pub fn delete_work_log(conn: &Connection, date: &str) -> Result<()> {
        let affected = conn.execute("DELETE FROM work_logs WHERE date = ?", params![date])?;

        if affected == 0 {
            Err(anyhow::anyhow!("未找到指定日期的工作日志"))
        } else {
            Ok(())
        }
    }

    /// 获取所有工作日志
    pub fn get_all_work_logs(conn: &Connection) -> Result<Vec<WorkLog>> {
        let mut stmt = conn.prepare(
            "SELECT id, date, log_type, content, ai_generated, created_at, updated_at
             FROM work_logs
             ORDER BY date DESC",
        )?;

        let logs = stmt
            .query_map([], |row| {
                Ok(WorkLog {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    log_type: row.get(2)?,
                    content: row.get(3)?,
                    ai_generated: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }
    // ====== 为了符合任务要求,添加简化的方法别名 ======

    /// 保存日志的简化方法 (与 save_work_log 功能相同)
    /// 默认 log_type='daily', ai_generated=false
    pub fn save_log(conn: &Connection, date: &str, content: &str) -> Result<()> {
        Self::save_work_log(conn, date, "daily", content, false)
    }

    /// 获取日志的简化方法 (与 get_work_log 功能相同)
    pub fn get_log(conn: &Connection, date: &str) -> Result<Option<WorkLog>> {
        Self::get_work_log(conn, date)
    }

    /// 获取最近日志的简化方法 (与 get_recent_work_logs 功能相同)
    pub fn get_recent_logs(conn: &Connection, days: i32) -> Result<Vec<WorkLog>> {
        Self::get_recent_work_logs(conn, days)
    }

    /// 删除日志的简化方法 (与 delete_work_log 功能相同)
    pub fn delete_log(conn: &Connection, date: &str) -> Result<()> {
        Self::delete_work_log(conn, date)
    }

    /// 获取当日上下文摘要
    pub fn get_day_context_summary(
        conn: &Connection,
        date: &str,
    ) -> Result<DayContextSummary> {
        // 获取当日所有上下文
        let contexts = ContextStoreService::list_by_date(conn, date)?;

        if contexts.is_empty() {
            return Ok(DayContextSummary {
                active_time_range: ActiveTimeRange {
                    start: String::new(),
                    end: String::new(),
                    total_minutes: 0,
                },
                app_usage: vec![],
                activity_distribution: ActivityDistribution {
                    coding: 0,
                    browsing: 0,
                    document: 0,
                    meeting: 0,
                    communication: 0,
                    other: 0,
                },
                key_activities: vec![],
            });
        }

        // 计算时间范围
        let timestamps: Vec<_> = contexts
            .iter()
            .filter_map(|c| {
                NaiveDateTime::parse_from_str(&c.captured_at, "%Y-%m-%d %H:%M:%S").ok()
            })
            .collect();

        let (start, end, total_minutes) = if !timestamps.is_empty() {
            let min_time = timestamps.iter().min().unwrap();
            let max_time = timestamps.iter().max().unwrap();
            let duration = max_time.signed_duration_since(*min_time);
            (
                min_time.format("%H:%M").to_string(),
                max_time.format("%H:%M").to_string(),
                duration.num_minutes() as i32,
            )
        } else {
            (String::new(), String::new(), 0)
        };

        // 统计应用使用情况（按次数统计）
        let mut app_counts: HashMap<String, i32> = HashMap::new();
        for context in &contexts {
            if let Some(app_name) = &context.app_name {
                *app_counts.entry(app_name.clone()).or_insert(0) += 1;
            }
        }

        // 根据比例分配总时长到各应用
        let total_count = contexts.len() as f32;
        let mut app_usage: Vec<AppUsage> = app_counts
            .into_iter()
            .map(|(app_name, count)| {
                // 按比例计算该应用的使用时长
                let minutes = ((count as f32 / total_count) * total_minutes as f32).round() as i32;
                AppUsage {
                    app_name,
                    minutes,
                    percentage: (count as f32 / total_count * 100.0),
                }
            })
            .collect();
        app_usage.sort_by(|a, b| b.minutes.cmp(&a.minutes));

        // 统计活动类型分布（按次数统计）
        let mut activity_type_counts: HashMap<String, i32> = HashMap::new();
        for context in &contexts {
            *activity_type_counts
                .entry(context.activity_type.clone())
                .or_insert(0) += 1;
        }

        // 根据比例分配总时长到各活动类型
        let get_activity_minutes = |activity_type: &str| -> i32 {
            let count = *activity_type_counts.get(activity_type).unwrap_or(&0);
            ((count as f32 / total_count) * total_minutes as f32).round() as i32
        };

        let activity_distribution = ActivityDistribution {
            coding: get_activity_minutes("coding"),
            browsing: get_activity_minutes("browsing"),
            document: get_activity_minutes("document"),
            meeting: get_activity_minutes("meeting"),
            communication: get_activity_minutes("communication"),
            other: get_activity_minutes("other"),
        };

        // 提取关键活动（取前10个最重要的）
        let key_activities: Vec<KeyActivity> = contexts
            .iter()
            .take(10)
            .map(|c| KeyActivity {
                time: c
                    .captured_at
                    .split(' ')
                    .nth(1)
                    .unwrap_or("")
                    .to_string(),
                description: c.description.clone(),
                app_name: c.app_name.clone().unwrap_or_else(|| "Unknown".to_string()),
            })
            .collect();

        Ok(DayContextSummary {
            active_time_range: ActiveTimeRange {
                start,
                end,
                total_minutes,
            },
            app_usage,
            activity_distribution,
            key_activities,
        })
    }

    /// 获取周上下文摘要
    pub fn get_week_context_summary(
        conn: &Connection,
        start_date: &str,
        end_date: &str,
    ) -> Result<WeekContextSummary> {
        // 解析日期范围
        let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")?;
        let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")?;

        // 收集每日活动时长（基于实际时间范围计算）
        let mut daily_active_hours = vec![];
        let mut current_date = start;
        while current_date <= end {
            let date_str = current_date.format("%Y-%m-%d").to_string();
            let contexts = ContextStoreService::list_by_date(conn, &date_str)?;

            // 根据实际时间戳计算活动时长
            let hours = if contexts.len() > 1 {
                let timestamps: Vec<_> = contexts
                    .iter()
                    .filter_map(|c| {
                        NaiveDateTime::parse_from_str(&c.captured_at, "%Y-%m-%d %H:%M:%S").ok()
                    })
                    .collect();
                if let (Some(min_time), Some(max_time)) = (timestamps.iter().min(), timestamps.iter().max()) {
                    let duration = max_time.signed_duration_since(*min_time);
                    duration.num_minutes() as f32 / 60.0
                } else {
                    0.0
                }
            } else if contexts.len() == 1 {
                1.0 / 60.0 // 单次采集按1分钟计
            } else {
                0.0
            };

            daily_active_hours.push(DailyActiveHours {
                date: date_str,
                hours,
            });
            current_date = current_date.succ_opt().unwrap();
        }

        // 收集本周所有上下文用于统计
        let mut all_contexts = vec![];
        let mut current_date = start;
        while current_date <= end {
            let date_str = current_date.format("%Y-%m-%d").to_string();
            let mut contexts = ContextStoreService::list_by_date(conn, &date_str)?;
            all_contexts.append(&mut contexts);
            current_date = current_date.succ_opt().unwrap();
        }

        // 计算周总活动时长（基于每日时长汇总）
        let week_total_minutes: f32 = daily_active_hours.iter().map(|d| d.hours * 60.0).sum();
        let week_total_count = all_contexts.len() as f32;

        // 统计应用使用排行（按次数统计，然后按比例分配时长）
        let mut app_counts: HashMap<String, i32> = HashMap::new();
        for context in &all_contexts {
            if let Some(app_name) = &context.app_name {
                *app_counts.entry(app_name.clone()).or_insert(0) += 1;
            }
        }

        let mut weekly_app_ranking: Vec<WeeklyAppRanking> = app_counts
            .into_iter()
            .map(|(app_name, count)| {
                let total_minutes = ((count as f32 / week_total_count) * week_total_minutes).round() as i32;
                WeeklyAppRanking {
                    app_name,
                    total_minutes,
                    trend: "stable".to_string(), // 简化处理，默认为stable
                }
            })
            .collect();
        weekly_app_ranking.sort_by(|a, b| b.total_minutes.cmp(&a.total_minutes));
        weekly_app_ranking.truncate(10); // 只保留前10名

        // 统计活动类型（按次数统计，然后按比例分配时长）
        let mut activity_type_counts: HashMap<String, i32> = HashMap::new();
        for context in &all_contexts {
            *activity_type_counts
                .entry(context.activity_type.clone())
                .or_insert(0) += 1;
        }

        let working_days = (end - start).num_days() + 1;
        let weekly_activity_summary: Vec<WeeklyActivitySummary> = activity_type_counts
            .into_iter()
            .map(|(activity_type, count)| {
                let total_minutes = ((count as f32 / week_total_count) * week_total_minutes).round() as i32;
                WeeklyActivitySummary {
                    r#type: activity_type,
                    total_minutes,
                    daily_average: total_minutes as f32 / working_days as f32,
                }
            })
            .collect();

        // 分析工作模式
        let mut hour_counts: HashMap<i32, i32> = HashMap::new();
        let mut start_hours = vec![];
        let mut end_hours = vec![];

        for date in &daily_active_hours {
            let contexts = ContextStoreService::list_by_date(conn, &date.date)?;
            if !contexts.is_empty() {
                // 统计每小时活动
                for context in &contexts {
                    if let Ok(dt) =
                        NaiveDateTime::parse_from_str(&context.captured_at, "%Y-%m-%d %H:%M:%S")
                    {
                        let hour = dt.hour() as i32;
                        *hour_counts.entry(hour).or_insert(0) += 1;
                    }
                }

                // 记录开始和结束时间
                if let Some(first) = contexts.last() {
                    if let Ok(dt) =
                        NaiveDateTime::parse_from_str(&first.captured_at, "%Y-%m-%d %H:%M:%S")
                    {
                        start_hours.push(dt.hour());
                    }
                }
                if let Some(last) = contexts.first() {
                    if let Ok(dt) =
                        NaiveDateTime::parse_from_str(&last.captured_at, "%Y-%m-%d %H:%M:%S")
                    {
                        end_hours.push(dt.hour());
                    }
                }
            }
        }

        let most_productive_hour = hour_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(hour, _)| hour)
            .unwrap_or(10);

        let average_start_hour = if !start_hours.is_empty() {
            start_hours.iter().sum::<u32>() as f32 / start_hours.len() as f32
        } else {
            9.0
        };

        let average_end_hour = if !end_hours.is_empty() {
            end_hours.iter().sum::<u32>() as f32 / end_hours.len() as f32
        } else {
            18.0
        };

        let work_patterns = WorkPatterns {
            most_productive_hour,
            average_start_time: format!("{:02}:00", average_start_hour as i32),
            average_end_time: format!("{:02}:00", average_end_hour as i32),
        };

        Ok(WeekContextSummary {
            daily_active_hours,
            weekly_app_ranking,
            weekly_activity_summary,
            work_patterns,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    #[test]
    fn test_save_and_get_log() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 测试简化方法
        WorkLogService::save_log(&conn, "2024-01-15", "完成了任务A和任务B").unwrap();

        let log = WorkLogService::get_log(&conn, "2024-01-15").unwrap();
        assert!(log.is_some());
        let log = log.unwrap();
        assert_eq!(log.date, "2024-01-15");
        assert_eq!(log.content, "完成了任务A和任务B");
        assert_eq!(log.log_type, "daily");
        assert!(!log.ai_generated);
    }

    #[test]
    fn test_save_work_log_full() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 测试完整方法
        WorkLogService::save_work_log(&conn, "2024-01-16", "weekly", "本周工作总结", true).unwrap();

        let log = WorkLogService::get_work_log(&conn, "2024-01-16")
            .unwrap()
            .unwrap();
        assert_eq!(log.log_type, "weekly");
        assert_eq!(log.content, "本周工作总结");
        assert!(log.ai_generated);
    }

    #[test]
    fn test_update_existing_log() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 保存初始日志
        WorkLogService::save_log(&conn, "2024-01-15", "初始内容").unwrap();

        // 更新日志
        WorkLogService::save_log(&conn, "2024-01-15", "更新后的内容").unwrap();

        // 验证更新
        let log = WorkLogService::get_log(&conn, "2024-01-15")
            .unwrap()
            .unwrap();
        assert_eq!(log.content, "更新后的内容");
    }

    #[test]
    fn test_get_recent_logs() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 插入测试数据
        conn.execute(
            "INSERT INTO work_logs (date, log_type, content, ai_generated, created_at)
             VALUES (date('now'), 'daily', '今天的日志', 0, datetime('now'))",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO work_logs (date, log_type, content, ai_generated, created_at)
             VALUES (date('now', '-1 days'), 'daily', '昨天的日志', 0, datetime('now'))",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO work_logs (date, log_type, content, ai_generated, created_at)
             VALUES (date('now', '-5 days'), 'daily', '5天前的日志', 0, datetime('now'))",
            [],
        )
        .unwrap();

        // 获取最近3天的日志
        let logs = WorkLogService::get_recent_logs(&conn, 3).unwrap();
        assert_eq!(logs.len(), 2); // 应该只有今天和昨天的日志
    }

    #[test]
    fn test_delete_log() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 保存日志
        WorkLogService::save_log(&conn, "2024-01-15", "要删除的日志").unwrap();

        // 确认日志存在
        assert!(WorkLogService::get_log(&conn, "2024-01-15")
            .unwrap()
            .is_some());

        // 删除日志
        WorkLogService::delete_log(&conn, "2024-01-15").unwrap();

        // 确认日志已删除
        assert!(WorkLogService::get_log(&conn, "2024-01-15")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_delete_nonexistent_log() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 删除不存在的日志应该返回错误
        let result = WorkLogService::delete_log(&conn, "2024-01-15");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_logs() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 插入多条日志
        WorkLogService::save_log(&conn, "2024-01-10", "日志1").unwrap();
        WorkLogService::save_log(&conn, "2024-01-15", "日志2").unwrap();
        WorkLogService::save_log(&conn, "2024-01-20", "日志3").unwrap();

        // 获取所有日志
        let logs = WorkLogService::get_all_work_logs(&conn).unwrap();
        assert_eq!(logs.len(), 3);

        // 验证按日期倒序
        assert_eq!(logs[0].date, "2024-01-20");
        assert_eq!(logs[1].date, "2024-01-15");
        assert_eq!(logs[2].date, "2024-01-10");
    }
}
