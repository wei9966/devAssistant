use anyhow::Result;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// 提醒类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReminderType {
    DeadlineApproaching,  // 截止日期临近
    HighPriorityStale,    // 高优先级任务长期未处理
    ImportantDate,        // 记忆中的重要日期
}

impl ReminderType {
    /// 转换为字符串
    pub fn as_str(&self) -> &str {
        match self {
            ReminderType::DeadlineApproaching => "deadline_approaching",
            ReminderType::HighPriorityStale => "high_priority_stale",
            ReminderType::ImportantDate => "important_date",
        }
    }

    /// 从字符串转换
    pub fn from_str(s: &str) -> Self {
        match s {
            "deadline_approaching" => ReminderType::DeadlineApproaching,
            "high_priority_stale" => ReminderType::HighPriorityStale,
            "important_date" => ReminderType::ImportantDate,
            _ => ReminderType::DeadlineApproaching, // 默认值
        }
    }
}

/// 紧急程度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Urgency {
    High,
    Medium,
    Low,
}

impl Urgency {
    /// 转换为字符串
    pub fn as_str(&self) -> &str {
        match self {
            Urgency::High => "high",
            Urgency::Medium => "medium",
            Urgency::Low => "low",
        }
    }
}

/// 提醒数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub reminder_type: ReminderType,
    pub title: String,
    pub message: String,
    pub urgency: Urgency,
    pub related_task_id: Option<i64>,
}

/// 智能提醒服务
pub struct ReminderService;

impl ReminderService {
    /// 获取所有需要提醒的事项
    pub fn get_pending_reminders(conn: &Connection) -> Result<Vec<Reminder>> {
        let mut reminders = Vec::new();

        // 检查截止日期提醒
        reminders.extend(Self::check_deadline_reminders(conn)?);

        // 检查高优先级任务长期未处理提醒
        reminders.extend(Self::check_stale_high_priority_reminders(conn)?);

        // 检查记忆中的重要日期提醒
        reminders.extend(Self::check_memory_reminders(conn)?);

        Ok(reminders)
    }

    /// 检查任务截止日期提醒（1天内高紧急、3天内中紧急）
    pub fn check_deadline_reminders(conn: &Connection) -> Result<Vec<Reminder>> {
        let mut reminders = Vec::new();
        let now = Local::now();

        // 查询未完成的任务，且有截止日期
        let mut stmt = conn.prepare(
            "SELECT id, title, due_date, priority
             FROM tasks
             WHERE status != 'completed'
             AND due_date IS NOT NULL
             ORDER BY due_date ASC",
        )?;

        let task_rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,      // id
                row.get::<_, String>(1)?,   // title
                row.get::<_, String>(2)?,   // due_date
                row.get::<_, i32>(3)?,      // priority
            ))
        })?;

        for task_row in task_rows {
            let (task_id, title, due_date_str, _priority) = task_row?;

            // 尝试解析截止日期
            if let Ok(due_date) = Self::parse_datetime(&due_date_str) {
                let days_until_due = (due_date - now).num_days();

                // 1天内：高紧急
                if days_until_due <= 1 && days_until_due >= 0 {
                    let hours_until_due = (due_date - now).num_hours();
                    let message = if hours_until_due <= 0 {
                        format!("任务「{}」已到截止日期！", title)
                    } else if hours_until_due < 24 {
                        format!("任务「{}」将在 {} 小时内到期", title, hours_until_due)
                    } else {
                        format!("任务「{}」将在明天到期", title)
                    };

                    reminders.push(Reminder {
                        reminder_type: ReminderType::DeadlineApproaching,
                        title: "截止日期临近".to_string(),
                        message,
                        urgency: Urgency::High,
                        related_task_id: Some(task_id),
                    });
                }
                // 3天内：中紧急
                else if days_until_due > 1 && days_until_due <= 3 {
                    let message = format!("任务「{}」将在 {} 天内到期", title, days_until_due);

                    reminders.push(Reminder {
                        reminder_type: ReminderType::DeadlineApproaching,
                        title: "即将到期".to_string(),
                        message,
                        urgency: Urgency::Medium,
                        related_task_id: Some(task_id),
                    });
                }
                // 已过期但未完成
                else if days_until_due < 0 {
                    let days_overdue = -days_until_due;
                    let message = format!("任务「{}」已逾期 {} 天", title, days_overdue);

                    reminders.push(Reminder {
                        reminder_type: ReminderType::DeadlineApproaching,
                        title: "任务逾期".to_string(),
                        message,
                        urgency: Urgency::High,
                        related_task_id: Some(task_id),
                    });
                }
            }
        }

        Ok(reminders)
    }

    /// 检查高优先级任务长期未处理提醒（超过3天未更新）
    pub fn check_stale_high_priority_reminders(conn: &Connection) -> Result<Vec<Reminder>> {
        let mut reminders = Vec::new();
        let now = Local::now();

        // 查询高优先级（priority = 1）且未完成的任务
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at, started_at, last_active_at
             FROM tasks
             WHERE priority = 1
             AND status IN ('todo', 'in_progress')
             ORDER BY created_at ASC",
        )?;

        let task_rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,                      // id
                row.get::<_, String>(1)?,                   // title
                row.get::<_, String>(2)?,                   // created_at
                row.get::<_, Option<String>>(3)?,           // started_at
                row.get::<_, Option<String>>(4)?,           // last_active_at
            ))
        })?;

        for task_row in task_rows {
            let (task_id, title, created_at_str, started_at_opt, last_active_at_opt) = task_row?;

            // 确定最后活跃时间：优先使用 last_active_at，其次 started_at，最后 created_at
            let last_update_str = last_active_at_opt
                .or(started_at_opt)
                .unwrap_or(created_at_str);

            if let Ok(last_update) = Self::parse_datetime(&last_update_str) {
                let days_since_update = (now - last_update).num_days();

                // 超过3天未更新
                if days_since_update >= 3 {
                    let message = format!(
                        "高优先级任务「{}」已有 {} 天未更新",
                        title, days_since_update
                    );

                    let urgency = if days_since_update >= 7 {
                        Urgency::High
                    } else {
                        Urgency::Medium
                    };

                    reminders.push(Reminder {
                        reminder_type: ReminderType::HighPriorityStale,
                        title: "高优先级任务长期未处理".to_string(),
                        message,
                        urgency,
                        related_task_id: Some(task_id),
                    });
                }
            }
        }

        Ok(reminders)
    }

    /// 检查记忆中的重要日期提醒
    /// 从 ai_context_memory 表读取 context_type = 'fact' 的日期相关记忆
    pub fn check_memory_reminders(conn: &Connection) -> Result<Vec<Reminder>> {
        let mut reminders = Vec::new();
        let now = Local::now();

        // 查询 fact 类型的记忆
        let mut stmt = conn.prepare(
            "SELECT id, key, value, importance
             FROM ai_context_memory
             WHERE context_type = 'fact'
             ORDER BY importance DESC, created_at DESC",
        )?;

        let memory_rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,      // id
                row.get::<_, String>(1)?,   // key
                row.get::<_, String>(2)?,   // value (JSON string)
                row.get::<_, i32>(3)?,      // importance
            ))
        })?;

        for memory_row in memory_rows {
            let (_id, key, value_str, _importance) = memory_row?;

            // 尝试从 value 中提取日期信息
            if let Some(date_info) = Self::extract_date_from_memory(&key, &value_str) {
                if let Ok(event_date) = Self::parse_date_only(&date_info.date_str) {
                    let event_datetime = event_date
                        .and_hms_opt(9, 0, 0) // 默认早上9点
                        .and_then(|dt| Local.from_local_datetime(&dt).single());

                    if let Some(event_dt) = event_datetime {
                        let days_until_event = (event_dt - now).num_days();

                        // 当天或未来7天内的重要日期
                        if days_until_event >= 0 && days_until_event <= 7 {
                            let urgency = if days_until_event == 0 {
                                Urgency::High
                            } else if days_until_event <= 2 {
                                Urgency::Medium
                            } else {
                                Urgency::Low
                            };

                            let message = if days_until_event == 0 {
                                format!("今天是{}：{}", key, date_info.description)
                            } else if days_until_event == 1 {
                                format!("明天是{}：{}", key, date_info.description)
                            } else {
                                format!(
                                    "{} 天后是{}：{}",
                                    days_until_event, key, date_info.description
                                )
                            };

                            reminders.push(Reminder {
                                reminder_type: ReminderType::ImportantDate,
                                title: "重要日期提醒".to_string(),
                                message,
                                urgency,
                                related_task_id: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(reminders)
    }

    /// 格式化提醒为文本（用于注入系统提示词）
    pub fn format_reminders_for_prompt(reminders: &[Reminder]) -> String {
        if reminders.is_empty() {
            return String::new();
        }

        let mut prompt = String::from("\n\n## 当前待办提醒\n\n");

        // 按紧急程度分组
        let high_urgency: Vec<_> = reminders
            .iter()
            .filter(|r| matches!(r.urgency, Urgency::High))
            .collect();
        let medium_urgency: Vec<_> = reminders
            .iter()
            .filter(|r| matches!(r.urgency, Urgency::Medium))
            .collect();
        let low_urgency: Vec<_> = reminders
            .iter()
            .filter(|r| matches!(r.urgency, Urgency::Low))
            .collect();

        if !high_urgency.is_empty() {
            prompt.push_str("### 🔴 紧急提醒\n");
            for reminder in high_urgency {
                prompt.push_str(&format!("- {}\n", reminder.message));
            }
            prompt.push('\n');
        }

        if !medium_urgency.is_empty() {
            prompt.push_str("### 🟡 重要提醒\n");
            for reminder in medium_urgency {
                prompt.push_str(&format!("- {}\n", reminder.message));
            }
            prompt.push('\n');
        }

        if !low_urgency.is_empty() {
            prompt.push_str("### 🟢 一般提醒\n");
            for reminder in low_urgency {
                prompt.push_str(&format!("- {}\n", reminder.message));
            }
            prompt.push('\n');
        }

        prompt.push_str("请在回复用户时，适时提及这些提醒事项，帮助用户更好地管理任务。\n");

        prompt
    }

    // ========== 辅助方法 ==========

    /// 解析日期时间字符串（支持多种格式）
    fn parse_datetime(datetime_str: &str) -> Result<DateTime<Local>> {
        // 尝试多种日期时间格式
        let formats = [
            "%Y-%m-%d %H:%M:%S",          // 2024-12-29 14:30:00
            "%Y-%m-%dT%H:%M:%S",          // 2024-12-29T14:30:00
            "%Y-%m-%d %H:%M:%S%.f",       // 2024-12-29 14:30:00.123
            "%Y-%m-%dT%H:%M:%S%.f",       // 2024-12-29T14:30:00.123
            "%Y-%m-%d",                   // 2024-12-29 (默认00:00:00)
        ];

        for format in &formats {
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(datetime_str, format) {
                if let Some(dt) = Local.from_local_datetime(&naive_dt).single() {
                    return Ok(dt);
                }
            }
        }

        // 如果是纯日期格式
        if let Ok(naive_date) = NaiveDate::parse_from_str(datetime_str, "%Y-%m-%d") {
            if let Some(naive_dt) = naive_date.and_hms_opt(0, 0, 0) {
                if let Some(dt) = Local.from_local_datetime(&naive_dt).single() {
                    return Ok(dt);
                }
            }
        }

        Err(anyhow::anyhow!("无法解析日期时间: {}", datetime_str))
    }

    /// 解析纯日期字符串
    fn parse_date_only(date_str: &str) -> Result<NaiveDate> {
        let formats = [
            "%Y-%m-%d",      // 2024-12-29
            "%Y/%m/%d",      // 2024/12/29
            "%Y年%m月%d日",  // 2024年12月29日
        ];

        for format in &formats {
            if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
                return Ok(date);
            }
        }

        Err(anyhow::anyhow!("无法解析日期: {}", date_str))
    }

    /// 从记忆中提取日期信息
    /// 返回 (日期字符串, 描述)
    fn extract_date_from_memory(key: &str, value_json: &str) -> Option<DateInfo> {
        // 尝试解析 JSON
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(value_json) {
            // 查找 date 字段
            if let Some(date_value) = value.get("date") {
                if let Some(date_str) = date_value.as_str() {
                    let description = value
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or(key)
                        .to_string();

                    return Some(DateInfo {
                        date_str: date_str.to_string(),
                        description,
                    });
                }
            }

            // 查找 event_date 字段
            if let Some(date_value) = value.get("event_date") {
                if let Some(date_str) = date_value.as_str() {
                    let description = value
                        .get("event")
                        .and_then(|v| v.as_str())
                        .unwrap_or(key)
                        .to_string();

                    return Some(DateInfo {
                        date_str: date_str.to_string(),
                        description,
                    });
                }
            }

            // 如果是字符串值，尝试直接作为日期
            if let Some(text) = value.as_str() {
                // 简单的日期模式匹配（YYYY-MM-DD）
                if text.len() == 10 && text.chars().filter(|c| *c == '-').count() == 2 {
                    return Some(DateInfo {
                        date_str: text.to_string(),
                        description: key.to_string(),
                    });
                }
            }
        }

        None
    }
}

/// 日期信息结构
struct DateInfo {
    date_str: String,
    description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use chrono::{Duration, Local};
    use rusqlite::{params, Connection};

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_deadline_reminders_within_1_day() {
        let conn = setup_test_db();

        // 创建一个明天到期的任务
        let tomorrow = (Local::now() + Duration::days(1))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        conn.execute(
            "INSERT INTO tasks (title, status, priority, due_date, created_at)
             VALUES (?, ?, ?, ?, datetime('now', 'localtime'))",
            params!["明天到期任务", "todo", 2, tomorrow],
        )
        .unwrap();

        let reminders = ReminderService::check_deadline_reminders(&conn).unwrap();

        assert_eq!(reminders.len(), 1);
        assert!(matches!(
            reminders[0].reminder_type,
            ReminderType::DeadlineApproaching
        ));
        assert!(matches!(reminders[0].urgency, Urgency::High));
    }

    #[test]
    fn test_deadline_reminders_within_3_days() {
        let conn = setup_test_db();

        // 创建一个3天后到期的任务
        let in_three_days = (Local::now() + Duration::days(3))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        conn.execute(
            "INSERT INTO tasks (title, status, priority, due_date, created_at)
             VALUES (?, ?, ?, ?, datetime('now', 'localtime'))",
            params!["3天后到期任务", "todo", 2, in_three_days],
        )
        .unwrap();

        let reminders = ReminderService::check_deadline_reminders(&conn).unwrap();

        assert_eq!(reminders.len(), 1);
        assert!(matches!(reminders[0].urgency, Urgency::Medium));
    }

    #[test]
    fn test_stale_high_priority_reminders() {
        let conn = setup_test_db();

        // 创建一个5天前创建的高优先级任务
        conn.execute(
            "INSERT INTO tasks (title, status, priority, created_at)
             VALUES (?, ?, ?, datetime('now', 'localtime', '-5 days'))",
            params!["长期未处理高优任务", "todo", 1],
        )
        .unwrap();

        let reminders = ReminderService::check_stale_high_priority_reminders(&conn).unwrap();

        assert_eq!(reminders.len(), 1);
        assert!(matches!(
            reminders[0].reminder_type,
            ReminderType::HighPriorityStale
        ));
        assert!(reminders[0].message.contains("5 天未更新"));
    }

    #[test]
    fn test_memory_date_reminders() {
        let conn = setup_test_db();

        // 插入一个明天的重要日期记忆
        let tomorrow = (Local::now() + Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        let value = serde_json::json!({
            "date": tomorrow,
            "description": "项目评审会议"
        });

        conn.execute(
            "INSERT INTO ai_context_memory (context_type, key, value, importance)
             VALUES (?, ?, ?, ?)",
            params!["fact", "项目评审", value.to_string(), 8],
        )
        .unwrap();

        let reminders = ReminderService::check_memory_reminders(&conn).unwrap();

        assert_eq!(reminders.len(), 1);
        assert!(matches!(
            reminders[0].reminder_type,
            ReminderType::ImportantDate
        ));
        assert!(reminders[0].message.contains("明天"));
    }

    #[test]
    fn test_format_reminders_for_prompt() {
        let reminders = vec![
            Reminder {
                reminder_type: ReminderType::DeadlineApproaching,
                title: "截止日期临近".to_string(),
                message: "任务「完成报告」将在明天到期".to_string(),
                urgency: Urgency::High,
                related_task_id: Some(1),
            },
            Reminder {
                reminder_type: ReminderType::HighPriorityStale,
                title: "高优先级任务长期未处理".to_string(),
                message: "高优先级任务「修复Bug」已有 5 天未更新".to_string(),
                urgency: Urgency::Medium,
                related_task_id: Some(2),
            },
        ];

        let prompt = ReminderService::format_reminders_for_prompt(&reminders);

        assert!(prompt.contains("🔴 紧急提醒"));
        assert!(prompt.contains("🟡 重要提醒"));
        assert!(prompt.contains("完成报告"));
        assert!(prompt.contains("修复Bug"));
    }

    #[test]
    fn test_overdue_task_reminder() {
        let conn = setup_test_db();

        // 创建一个已经逾期2天的任务
        let two_days_ago = (Local::now() - Duration::days(2))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        conn.execute(
            "INSERT INTO tasks (title, status, priority, due_date, created_at)
             VALUES (?, ?, ?, ?, datetime('now', 'localtime'))",
            params!["逾期任务", "in_progress", 1, two_days_ago],
        )
        .unwrap();

        let reminders = ReminderService::check_deadline_reminders(&conn).unwrap();

        assert_eq!(reminders.len(), 1);
        assert!(matches!(reminders[0].urgency, Urgency::High));
        assert!(reminders[0].message.contains("已逾期"));
        assert!(reminders[0].message.contains("2 天"));
    }

    #[test]
    fn test_no_reminders_for_completed_tasks() {
        let conn = setup_test_db();

        // 创建一个已完成但逾期的任务
        let yesterday = (Local::now() - Duration::days(1))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        conn.execute(
            "INSERT INTO tasks (title, status, priority, due_date, created_at, completed_at)
             VALUES (?, ?, ?, ?, datetime('now', 'localtime'), datetime('now', 'localtime'))",
            params!["已完成任务", "completed", 1, yesterday],
        )
        .unwrap();

        let reminders = ReminderService::check_deadline_reminders(&conn).unwrap();

        // 已完成的任务不应该产生提醒
        assert_eq!(reminders.len(), 0);
    }
}
