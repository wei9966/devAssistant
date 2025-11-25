use crate::models::work_log::WorkLog;
use anyhow::Result;
use rusqlite::{params, Connection};

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
                 SET log_type = ?, content = ?, ai_generated = ?, updated_at = datetime('now')
                 WHERE date = ?",
                params![log_type, content, ai_generated, date],
            )?;
        } else {
            // 创建新记录
            conn.execute(
                "INSERT INTO work_logs (date, log_type, content, ai_generated, created_at)
                 VALUES (?, ?, ?, ?, datetime('now'))",
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
