use crate::models::weekly_plan::{WeeklyPlan, WeeklyPlanStatus};
use anyhow::Result;
use rusqlite::{params, Connection};

pub struct WeeklyPlanService;

impl WeeklyPlanService {
    /// 保存或更新周计划
    /// 如果指定周的计划已存在,则更新;否则创建新记录
    pub fn save_weekly_plan(
        conn: &Connection,
        week_key: &str,
        content: &str,
        task_ids: &[i64],
        status: &str,
    ) -> Result<WeeklyPlan> {
        let task_ids_json = serde_json::to_string(task_ids)?;

        // 检查是否已存在该周的记录
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM weekly_plans WHERE week_key = ?",
            params![week_key],
            |row| row.get(0),
        )?;

        if exists > 0 {
            // 更新现有记录
            conn.execute(
                "UPDATE weekly_plans
                 SET content = ?, task_ids = ?, status = ?, updated_at = datetime('now', 'localtime')
                 WHERE week_key = ?",
                params![content, task_ids_json, status, week_key],
            )?;
        } else {
            // 创建新记录，original_task_ids 设置为初始的 task_ids
            conn.execute(
                "INSERT INTO weekly_plans (week_key, content, task_ids, original_task_ids, status, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, datetime('now', 'localtime'), datetime('now', 'localtime'))",
                params![week_key, content, task_ids_json.clone(), task_ids_json, status],
            )?;
        }

        // 返回保存后的记录
        Self::get_weekly_plan(conn, week_key)?
            .ok_or_else(|| anyhow::anyhow!("保存周计划后无法获取记录"))
    }

    /// 获取指定周的计划
    pub fn get_weekly_plan(conn: &Connection, week_key: &str) -> Result<Option<WeeklyPlan>> {
        let mut stmt = conn.prepare(
            "SELECT id, week_key, content, task_ids, original_task_ids, status, created_at, updated_at
             FROM weekly_plans
             WHERE week_key = ?",
        )?;

        let mut rows = stmt.query(params![week_key])?;

        if let Some(row) = rows.next()? {
            let task_ids_str: String = row.get(3)?;
            let original_task_ids_str: String = row.get(4)?;
            let status_str: String = row.get(5)?;

            Ok(Some(WeeklyPlan {
                id: Some(row.get(0)?),
                week_key: row.get(1)?,
                content: row.get(2)?,
                task_ids: serde_json::from_str(&task_ids_str).unwrap_or_default(),
                original_task_ids: serde_json::from_str(&original_task_ids_str).unwrap_or_default(),
                status: WeeklyPlanStatus::from(status_str),
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 获取所有周计划（按周倒序）
    pub fn get_all_weekly_plans(conn: &Connection) -> Result<Vec<WeeklyPlan>> {
        let mut stmt = conn.prepare(
            "SELECT id, week_key, content, task_ids, original_task_ids, status, created_at, updated_at
             FROM weekly_plans
             ORDER BY week_key DESC",
        )?;

        let plans = stmt
            .query_map([], |row| {
                let task_ids_str: String = row.get(3)?;
                let original_task_ids_str: String = row.get(4)?;
                let status_str: String = row.get(5)?;

                Ok(WeeklyPlan {
                    id: Some(row.get(0)?),
                    week_key: row.get(1)?,
                    content: row.get(2)?,
                    task_ids: serde_json::from_str(&task_ids_str).unwrap_or_default(),
                    original_task_ids: serde_json::from_str(&original_task_ids_str).unwrap_or_default(),
                    status: WeeklyPlanStatus::from(status_str),
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(plans)
    }

    /// 更新周计划的任务列表（用于添加新任务）
    pub fn update_task_ids(conn: &Connection, week_key: &str, task_ids: &[i64]) -> Result<()> {
        let task_ids_json = serde_json::to_string(task_ids)?;

        let affected = conn.execute(
            "UPDATE weekly_plans
             SET task_ids = ?, updated_at = datetime('now', 'localtime')
             WHERE week_key = ?",
            params![task_ids_json, week_key],
        )?;

        if affected == 0 {
            Err(anyhow::anyhow!("未找到指定周的计划"))
        } else {
            Ok(())
        }
    }

    /// 更新周计划状态
    pub fn update_status(conn: &Connection, week_key: &str, status: &str) -> Result<()> {
        let affected = conn.execute(
            "UPDATE weekly_plans
             SET status = ?, updated_at = datetime('now', 'localtime')
             WHERE week_key = ?",
            params![status, week_key],
        )?;

        if affected == 0 {
            Err(anyhow::anyhow!("未找到指定周的计划"))
        } else {
            Ok(())
        }
    }

    /// 删除指定周的计划
    pub fn delete_weekly_plan(conn: &Connection, week_key: &str) -> Result<()> {
        let affected = conn.execute(
            "DELETE FROM weekly_plans WHERE week_key = ?",
            params![week_key],
        )?;

        if affected == 0 {
            Err(anyhow::anyhow!("未找到指定周的计划"))
        } else {
            Ok(())
        }
    }

    /// 获取最近N周的计划
    pub fn get_recent_weekly_plans(conn: &Connection, limit: i32) -> Result<Vec<WeeklyPlan>> {
        let mut stmt = conn.prepare(
            "SELECT id, week_key, content, task_ids, original_task_ids, status, created_at, updated_at
             FROM weekly_plans
             ORDER BY week_key DESC
             LIMIT ?",
        )?;

        let plans = stmt
            .query_map(params![limit], |row| {
                let task_ids_str: String = row.get(3)?;
                let original_task_ids_str: String = row.get(4)?;
                let status_str: String = row.get(5)?;

                Ok(WeeklyPlan {
                    id: Some(row.get(0)?),
                    week_key: row.get(1)?,
                    content: row.get(2)?,
                    task_ids: serde_json::from_str(&task_ids_str).unwrap_or_default(),
                    original_task_ids: serde_json::from_str(&original_task_ids_str).unwrap_or_default(),
                    status: WeeklyPlanStatus::from(status_str),
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(plans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    #[test]
    fn test_save_and_get_weekly_plan() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let task_ids = vec![1, 2, 3];
        let plan = WeeklyPlanService::save_weekly_plan(
            &conn,
            "2025-W48",
            "本周计划内容",
            &task_ids,
            "draft",
        )
        .unwrap();

        assert_eq!(plan.week_key, "2025-W48");
        assert_eq!(plan.content, "本周计划内容");
        assert_eq!(plan.task_ids, task_ids);
        assert_eq!(plan.original_task_ids, task_ids);
        assert_eq!(plan.status, WeeklyPlanStatus::Draft);
    }

    #[test]
    fn test_update_weekly_plan() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 创建初始计划
        WeeklyPlanService::save_weekly_plan(
            &conn,
            "2025-W48",
            "初始内容",
            &[1, 2],
            "draft",
        )
        .unwrap();

        // 更新计划（注意：original_task_ids 应该保持不变）
        let updated = WeeklyPlanService::save_weekly_plan(
            &conn,
            "2025-W48",
            "更新后的内容",
            &[1, 2, 3, 4],  // 新增了任务
            "confirmed",
        )
        .unwrap();

        assert_eq!(updated.content, "更新后的内容");
        assert_eq!(updated.task_ids, vec![1, 2, 3, 4]);
        assert_eq!(updated.original_task_ids, vec![1, 2]);  // 保持原始任务ID
        assert_eq!(updated.status, WeeklyPlanStatus::Confirmed);
    }

    #[test]
    fn test_delete_weekly_plan() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        WeeklyPlanService::save_weekly_plan(
            &conn,
            "2025-W48",
            "要删除的计划",
            &[1],
            "draft",
        )
        .unwrap();

        // 确认计划存在
        assert!(WeeklyPlanService::get_weekly_plan(&conn, "2025-W48")
            .unwrap()
            .is_some());

        // 删除计划
        WeeklyPlanService::delete_weekly_plan(&conn, "2025-W48").unwrap();

        // 确认已删除
        assert!(WeeklyPlanService::get_weekly_plan(&conn, "2025-W48")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_get_all_weekly_plans() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        WeeklyPlanService::save_weekly_plan(&conn, "2025-W46", "计划1", &[], "draft").unwrap();
        WeeklyPlanService::save_weekly_plan(&conn, "2025-W47", "计划2", &[], "confirmed").unwrap();
        WeeklyPlanService::save_weekly_plan(&conn, "2025-W48", "计划3", &[], "draft").unwrap();

        let plans = WeeklyPlanService::get_all_weekly_plans(&conn).unwrap();
        assert_eq!(plans.len(), 3);

        // 验证按周倒序
        assert_eq!(plans[0].week_key, "2025-W48");
        assert_eq!(plans[1].week_key, "2025-W47");
        assert_eq!(plans[2].week_key, "2025-W46");
    }
}
