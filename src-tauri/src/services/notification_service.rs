// Notification Service
// 通知服务 - 管理通知的CRUD操作和自动生成逻辑

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use chrono::{Local, Datelike};

/// 通知类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    Tip,           // 休息提醒
    DailyReport,   // 日报
    WeeklyReport,  // 周报
}

impl NotificationType {
    pub fn as_str(&self) -> &str {
        match self {
            NotificationType::Tip => "tip",
            NotificationType::DailyReport => "daily_report",
            NotificationType::WeeklyReport => "weekly_report",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "tip" => Some(NotificationType::Tip),
            "daily_report" => Some(NotificationType::DailyReport),
            "weekly_report" => Some(NotificationType::WeeklyReport),
            _ => None,
        }
    }
}

/// 通知记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: Option<i64>,
    pub notification_type: String,
    pub title: String,
    pub content: String,
    pub is_read: bool,
    pub created_at: String,
}

/// 通知设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub tips_enabled: bool,
    pub tips_interval_minutes: i32,
    pub tips_max_per_day: i32,
    pub daily_report_enabled: bool,
    pub daily_report_time: String,
    pub weekly_report_enabled: bool,
    pub weekly_report_day: i32,
    pub weekly_report_time: String,
    pub updated_at: String,
}

/// 通知服务
pub struct NotificationService;

impl NotificationService {
    /// 创建新通知
    pub fn create(
        conn: &Connection,
        notification_type: &str,
        title: &str,
        content: &str,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO notifications (notification_type, title, content) VALUES (?1, ?2, ?3)",
            params![notification_type, title, content],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 获取通知列表
    pub fn list(
        conn: &Connection,
        notification_type: Option<&str>,
        is_read: Option<bool>,
        limit: Option<i32>,
    ) -> Result<Vec<Notification>> {
        let mut sql = String::from(
            "SELECT id, notification_type, title, content, is_read, created_at FROM notifications WHERE 1=1"
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ntype) = notification_type {
            sql.push_str(" AND notification_type = ?");
            params_vec.push(Box::new(ntype.to_string()));
        }

        if let Some(read) = is_read {
            sql.push_str(" AND is_read = ?");
            params_vec.push(Box::new(if read { 1 } else { 0 }));
        }

        sql.push_str(" ORDER BY created_at DESC");

        if let Some(l) = limit {
            sql.push_str(&format!(" LIMIT {}", l));
        }

        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let notifications = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(Notification {
                    id: Some(row.get(0)?),
                    notification_type: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    is_read: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(notifications)
    }

    /// 获取未读通知数量
    pub fn get_unread_count(conn: &Connection) -> Result<i32> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM notifications WHERE is_read = 0",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// 标记已读
    pub fn mark_read(conn: &Connection, id: i64) -> Result<()> {
        conn.execute(
            "UPDATE notifications SET is_read = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// 全部标记已读
    pub fn mark_all_read(conn: &Connection) -> Result<()> {
        conn.execute("UPDATE notifications SET is_read = 1", [])?;
        Ok(())
    }

    /// 删除通知
    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM notifications WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// 清空所有通知
    pub fn clear_all(conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM notifications", [])?;
        Ok(())
    }

    /// 获取通知设置
    pub fn get_settings(conn: &Connection) -> Result<NotificationSettings> {
        let settings = conn.query_row(
            "SELECT tips_enabled, tips_interval_minutes, tips_max_per_day,
                    daily_report_enabled, daily_report_time,
                    weekly_report_enabled, weekly_report_day, weekly_report_time,
                    updated_at
             FROM notification_settings WHERE id = 1",
            [],
            |row| {
                Ok(NotificationSettings {
                    tips_enabled: row.get::<_, i32>(0)? != 0,
                    tips_interval_minutes: row.get(1)?,
                    tips_max_per_day: row.get(2)?,
                    daily_report_enabled: row.get::<_, i32>(3)? != 0,
                    daily_report_time: row.get(4)?,
                    weekly_report_enabled: row.get::<_, i32>(5)? != 0,
                    weekly_report_day: row.get(6)?,
                    weekly_report_time: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )?;
        Ok(settings)
    }

    /// 更新通知设置
    pub fn update_settings(conn: &Connection, settings: &NotificationSettings) -> Result<()> {
        conn.execute(
            "UPDATE notification_settings SET
                tips_enabled = ?1,
                tips_interval_minutes = ?2,
                tips_max_per_day = ?3,
                daily_report_enabled = ?4,
                daily_report_time = ?5,
                weekly_report_enabled = ?6,
                weekly_report_day = ?7,
                weekly_report_time = ?8,
                updated_at = datetime('now', 'localtime')
             WHERE id = 1",
            params![
                if settings.tips_enabled { 1 } else { 0 },
                settings.tips_interval_minutes,
                settings.tips_max_per_day,
                if settings.daily_report_enabled { 1 } else { 0 },
                settings.daily_report_time,
                if settings.weekly_report_enabled { 1 } else { 0 },
                settings.weekly_report_day,
                settings.weekly_report_time,
            ],
        )?;
        Ok(())
    }

    /// 获取今日Tips数量
    pub fn get_today_tips_count(conn: &Connection) -> Result<i32> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM notifications
             WHERE notification_type = 'tip'
             AND date(created_at) = ?1",
            params![today],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// 生成休息提醒
    pub fn generate_tip(conn: &Connection, work_minutes: i32) -> Result<i64> {
        let title = "休息提醒".to_string();
        let content = format!(
            "您已连续工作 {} 分钟，建议稍作休息，保护视力和身体健康。\n\n💡 建议：\n- 起身活动一下\n- 远眺窗外放松眼睛\n- 喝杯水补充水分",
            work_minutes
        );

        Self::create(conn, NotificationType::Tip.as_str(), &title, &content)
    }

    /// 生成日报通知
    pub fn generate_daily_report_notification(
        conn: &Connection,
        date: &str,
        summary: &str,
    ) -> Result<i64> {
        let title = format!("{} 工作日报", date);
        let content = summary.to_string();

        Self::create(
            conn,
            NotificationType::DailyReport.as_str(),
            &title,
            &content,
        )
    }

    /// 生成周报通知
    pub fn generate_weekly_report_notification(
        conn: &Connection,
        week: &str,
        summary: &str,
    ) -> Result<i64> {
        let title = format!("{} 工作周报", week);
        let content = summary.to_string();

        Self::create(
            conn,
            NotificationType::WeeklyReport.as_str(),
            &title,
            &content,
        )
    }

    /// 检查是否应该生成日报
    pub fn should_generate_daily_report(conn: &Connection) -> Result<bool> {
        let settings = Self::get_settings(conn)?;
        if !settings.daily_report_enabled {
            return Ok(false);
        }

        let now = Local::now();
        let today = now.format("%Y-%m-%d").to_string();

        // 检查今天是否已经生成过日报
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM notifications
             WHERE notification_type = 'daily_report'
             AND date(created_at) = ?1",
            params![today],
            |row| row.get(0),
        )?;

        if count > 0 {
            return Ok(false);
        }

        // 检查当前时间是否达到设置的时间
        let current_time = now.format("%H:%M").to_string();
        Ok(current_time >= settings.daily_report_time)
    }

    /// 检查是否应该生成周报
    pub fn should_generate_weekly_report(conn: &Connection) -> Result<bool> {
        let settings = Self::get_settings(conn)?;
        if !settings.weekly_report_enabled {
            return Ok(false);
        }

        let now = Local::now();
        let weekday = now.weekday().num_days_from_sunday() as i32;

        // 检查今天是否是设置的周报日
        if weekday != settings.weekly_report_day {
            return Ok(false);
        }

        let today = now.format("%Y-%m-%d").to_string();

        // 检查今天是否已经生成过周报
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM notifications
             WHERE notification_type = 'weekly_report'
             AND date(created_at) = ?1",
            params![today],
            |row| row.get(0),
        )?;

        if count > 0 {
            return Ok(false);
        }

        // 检查当前时间是否达到设置的时间
        let current_time = now.format("%H:%M").to_string();
        Ok(current_time >= settings.weekly_report_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_type() {
        assert_eq!(NotificationType::Tip.as_str(), "tip");
        assert_eq!(NotificationType::DailyReport.as_str(), "daily_report");
        assert_eq!(NotificationType::WeeklyReport.as_str(), "weekly_report");

        assert!(matches!(
            NotificationType::from_str("tip"),
            Some(NotificationType::Tip)
        ));
        assert!(matches!(
            NotificationType::from_str("daily_report"),
            Some(NotificationType::DailyReport)
        ));
    }
}
