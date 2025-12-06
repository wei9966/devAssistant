use crate::models::screen_context::{DailySummary, DayStats, ScreenContext};
use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::HashMap;

pub struct ContextStoreService;

impl ContextStoreService {
    /// 保存屏幕上下文
    pub fn save_context(conn: &Connection, context: &ScreenContext) -> Result<i64> {
        conn.execute(
            "INSERT INTO screen_contexts (
                captured_at, app_name, window_title, activity_type,
                description, key_content, screenshot_hash, screenshot_path,
                processing_time_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                context.captured_at,
                context.app_name,
                context.window_title,
                context.activity_type,
                context.description,
                context.key_content,
                context.screenshot_hash,
                context.screenshot_path,
                context.processing_time_ms,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// 按日期查询上下文列表
    pub fn list_by_date(conn: &Connection, date: &str) -> Result<Vec<ScreenContext>> {
        let mut stmt = conn.prepare(
            "SELECT id, captured_at, app_name, window_title, activity_type,
                    description, key_content, screenshot_hash, screenshot_path,
                    processing_time_ms
             FROM screen_contexts
             WHERE date(captured_at) = ?
             ORDER BY captured_at DESC",
        )?;

        let contexts = stmt
            .query_map(params![date], |row| {
                Ok(ScreenContext {
                    id: Some(row.get(0)?),
                    captured_at: row.get(1)?,
                    app_name: row.get(2)?,
                    window_title: row.get(3)?,
                    activity_type: row.get(4)?,
                    description: row.get(5)?,
                    key_content: row.get(6)?,
                    screenshot_hash: row.get(7)?,
                    screenshot_path: row.get(8)?,
                    processing_time_ms: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(contexts)
    }

    /// 获取当日统计
    pub fn get_day_stats(conn: &Connection, date: &str) -> Result<DayStats> {
        // 获取总数
        let total_count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM screen_contexts WHERE date(captured_at) = ?",
            params![date],
            |row| row.get(0),
        )?;

        // 获取应用分布
        let mut app_distribution = HashMap::new();
        let mut stmt = conn.prepare(
            "SELECT app_name, COUNT(*) as count
             FROM screen_contexts
             WHERE date(captured_at) = ? AND app_name IS NOT NULL
             GROUP BY app_name",
        )?;

        let rows = stmt.query_map(params![date], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })?;

        for row in rows {
            let (app_name, count) = row?;
            app_distribution.insert(app_name, count);
        }

        // 获取活动类型分布
        let mut activity_distribution = HashMap::new();
        let mut stmt = conn.prepare(
            "SELECT activity_type, COUNT(*) as count
             FROM screen_contexts
             WHERE date(captured_at) = ?
             GROUP BY activity_type",
        )?;

        let rows = stmt.query_map(params![date], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })?;

        for row in rows {
            let (activity_type, count) = row?;
            activity_distribution.insert(activity_type, count);
        }

        // 获取时间范围
        let time_range: Option<(String, String)> = conn
            .query_row(
                "SELECT MIN(captured_at), MAX(captured_at)
                 FROM screen_contexts
                 WHERE date(captured_at) = ?",
                params![date],
                |row| {
                    let min: Option<String> = row.get(0)?;
                    let max: Option<String> = row.get(1)?;
                    match (min, max) {
                        (Some(min_time), Some(max_time)) => Ok(Some((min_time, max_time))),
                        _ => Ok(None),
                    }
                },
            )
            .unwrap_or(None);

        Ok(DayStats {
            total_count,
            app_distribution,
            activity_distribution,
            time_range,
        })
    }

    /// 删除指定日期的上下文
    pub fn delete_by_date(conn: &Connection, date: &str) -> Result<u32> {
        let affected = conn.execute(
            "DELETE FROM screen_contexts WHERE date(captured_at) = ?",
            params![date],
        )? as u32;

        Ok(affected)
    }

    /// 清理过期数据
    pub fn cleanup_old(conn: &Connection, retention_days: u32) -> Result<u32> {
        let affected = conn.execute(
            "DELETE FROM screen_contexts
             WHERE date(captured_at) < date('now', '-' || ? || ' days')",
            params![retention_days],
        )? as u32;

        Ok(affected)
    }

    /// 更新上下文描述（用于VLM分析后更新）
    /// 同时更新 activity_type（如果VLM分析返回了更准确的分类）
    pub fn update_description(
        conn: &Connection,
        id: i64,
        description: &str,
        key_content: Option<&str>,
        activity_type: Option<&str>,
    ) -> Result<()> {
        if let Some(activity) = activity_type {
            // 如果提供了 activity_type，同时更新
            conn.execute(
                "UPDATE screen_contexts SET description = ?, key_content = ?, activity_type = ? WHERE id = ?",
                params![description, key_content, activity, id],
            )?;
        } else {
            // 保持原有逻辑，只更新 description 和 key_content
            conn.execute(
                "UPDATE screen_contexts SET description = ?, key_content = ? WHERE id = ?",
                params![description, key_content, id],
            )?;
        }
        Ok(())
    }

    /// 保存每日摘要
    pub fn save_daily_summary(conn: &Connection, summary: &DailySummary) -> Result<()> {
        // 检查是否已存在该日期的摘要
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM daily_summaries WHERE summary_date = ?",
            params![summary.summary_date],
            |row| row.get(0),
        )?;

        if exists > 0 {
            // 更新现有记录
            conn.execute(
                "UPDATE daily_summaries
                 SET total_contexts = ?, app_stats = ?, activity_timeline = ?, ai_summary = ?
                 WHERE summary_date = ?",
                params![
                    summary.total_contexts,
                    summary.app_stats,
                    summary.activity_timeline,
                    summary.ai_summary,
                    summary.summary_date,
                ],
            )?;
        } else {
            // 创建新记录
            conn.execute(
                "INSERT INTO daily_summaries (summary_date, total_contexts, app_stats, activity_timeline, ai_summary)
                 VALUES (?, ?, ?, ?, ?)",
                params![
                    summary.summary_date,
                    summary.total_contexts,
                    summary.app_stats,
                    summary.activity_timeline,
                    summary.ai_summary,
                ],
            )?;
        }

        Ok(())
    }

    /// 获取每日摘要
    pub fn get_daily_summary(conn: &Connection, date: &str) -> Result<Option<DailySummary>> {
        let mut stmt = conn.prepare(
            "SELECT id, summary_date, total_contexts, app_stats, activity_timeline, ai_summary, created_at
             FROM daily_summaries
             WHERE summary_date = ?",
        )?;

        let mut rows = stmt.query(params![date])?;

        if let Some(row) = rows.next()? {
            Ok(Some(DailySummary {
                id: Some(row.get(0)?),
                summary_date: row.get(1)?,
                total_contexts: row.get(2)?,
                app_stats: row.get(3)?,
                activity_timeline: row.get(4)?,
                ai_summary: row.get(5)?,
                created_at: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_save_and_list_contexts() {
        let conn = setup_test_db();

        // 创建测试上下文
        let context = ScreenContext {
            id: None,
            captured_at: "2024-12-04 10:30:00".to_string(),
            app_name: Some("VS Code".to_string()),
            window_title: Some("main.rs - DevAssistant".to_string()),
            activity_type: "coding".to_string(),
            description: "编写 Rust 代码".to_string(),
            key_content: Some("impl ContextStoreService".to_string()),
            screenshot_hash: Some("abc123".to_string()),
            screenshot_path: None,
            processing_time_ms: Some(150),
        };

        // 保存上下文
        let id = ContextStoreService::save_context(&conn, &context).unwrap();
        assert!(id > 0);

        // 查询上下文列表
        let contexts = ContextStoreService::list_by_date(&conn, "2024-12-04").unwrap();
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].activity_type, "coding");
        assert_eq!(contexts[0].app_name, Some("VS Code".to_string()));
    }

    #[test]
    fn test_get_day_stats() {
        let conn = setup_test_db();

        // 插入多条测试数据
        for i in 0..5 {
            let context = ScreenContext {
                id: None,
                captured_at: format!("2024-12-04 10:{:02}:00", i * 10),
                app_name: Some(if i < 3 { "VS Code" } else { "Chrome" }.to_string()),
                window_title: None,
                activity_type: if i < 3 { "coding" } else { "browsing" }.to_string(),
                description: format!("测试活动 {}", i),
                key_content: None,
                screenshot_hash: None,
                screenshot_path: None,
                processing_time_ms: None,
            };
            ContextStoreService::save_context(&conn, &context).unwrap();
        }

        // 获取统计
        let stats = ContextStoreService::get_day_stats(&conn, "2024-12-04").unwrap();
        assert_eq!(stats.total_count, 5);
        assert_eq!(stats.app_distribution.get("VS Code"), Some(&3));
        assert_eq!(stats.app_distribution.get("Chrome"), Some(&2));
        assert_eq!(stats.activity_distribution.get("coding"), Some(&3));
        assert_eq!(stats.activity_distribution.get("browsing"), Some(&2));
        assert!(stats.time_range.is_some());
    }

    #[test]
    fn test_delete_by_date() {
        let conn = setup_test_db();

        // 插入测试数据
        let context = ScreenContext {
            id: None,
            captured_at: "2024-12-04 10:30:00".to_string(),
            app_name: Some("VS Code".to_string()),
            window_title: None,
            activity_type: "coding".to_string(),
            description: "测试".to_string(),
            key_content: None,
            screenshot_hash: None,
            screenshot_path: None,
            processing_time_ms: None,
        };
        ContextStoreService::save_context(&conn, &context).unwrap();

        // 删除
        let deleted = ContextStoreService::delete_by_date(&conn, "2024-12-04").unwrap();
        assert_eq!(deleted, 1);

        // 验证已删除
        let contexts = ContextStoreService::list_by_date(&conn, "2024-12-04").unwrap();
        assert_eq!(contexts.len(), 0);
    }

    #[test]
    fn test_save_and_get_daily_summary() {
        let conn = setup_test_db();

        // 创建摘要
        let summary = DailySummary {
            id: None,
            summary_date: "2024-12-04".to_string(),
            total_contexts: 50,
            app_stats: r#"{"VS Code": 30, "Chrome": 20}"#.to_string(),
            activity_timeline: r#"[{"time": "10:00", "activity": "coding"}]"#.to_string(),
            ai_summary: Some("今天主要进行代码开发".to_string()),
            created_at: None,
        };

        // 保存摘要
        ContextStoreService::save_daily_summary(&conn, &summary).unwrap();

        // 获取摘要
        let retrieved = ContextStoreService::get_daily_summary(&conn, "2024-12-04")
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.total_contexts, 50);
        assert_eq!(retrieved.ai_summary, Some("今天主要进行代码开发".to_string()));

        // 测试更新
        let updated_summary = DailySummary {
            id: None,
            summary_date: "2024-12-04".to_string(),
            total_contexts: 60,
            app_stats: r#"{"VS Code": 35, "Chrome": 25}"#.to_string(),
            activity_timeline: r#"[{"time": "10:00", "activity": "coding"}]"#.to_string(),
            ai_summary: Some("更新后的摘要".to_string()),
            created_at: None,
        };

        ContextStoreService::save_daily_summary(&conn, &updated_summary).unwrap();

        let retrieved = ContextStoreService::get_daily_summary(&conn, "2024-12-04")
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.total_contexts, 60);
        assert_eq!(retrieved.ai_summary, Some("更新后的摘要".to_string()));
    }

    #[test]
    fn test_cleanup_old() {
        let conn = setup_test_db();

        // 插入旧数据（使用 SQL 直接插入，绕过时间限制）
        conn.execute(
            "INSERT INTO screen_contexts (captured_at, activity_type, description)
             VALUES (date('now', '-10 days'), 'coding', '旧数据')",
            [],
        )
        .unwrap();

        // 插入新数据
        let context = ScreenContext {
            id: None,
            captured_at: "2024-12-04 10:30:00".to_string(),
            app_name: Some("VS Code".to_string()),
            window_title: None,
            activity_type: "coding".to_string(),
            description: "新数据".to_string(),
            key_content: None,
            screenshot_hash: None,
            screenshot_path: None,
            processing_time_ms: None,
        };
        ContextStoreService::save_context(&conn, &context).unwrap();

        // 清理 7 天前的数据
        let deleted = ContextStoreService::cleanup_old(&conn, 7).unwrap();
        assert!(deleted >= 1);
    }
}
