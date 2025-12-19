use crate::models::task::TaskMilestone;
use anyhow::Result;
use rusqlite::{params, Connection};

pub struct MilestoneService;

impl MilestoneService {
    /// 创建任务里程碑
    pub fn create_milestone(
        conn: &Connection,
        task_id: i64,
        title: &str,
        description: Option<&str>,
        progress_snapshot: Option<i32>,
    ) -> Result<TaskMilestone> {
        // 验证标题
        if title.trim().is_empty() {
            return Err(anyhow::anyhow!("里程碑标题不能为空"));
        }
        if title.len() > 200 {
            return Err(anyhow::anyhow!("里程碑标题不能超过200个字符"));
        }

        // 如果没有传 progress_snapshot，获取当前任务的 progress 值
        let progress_snapshot = if let Some(snapshot) = progress_snapshot {
            // 验证进度值范围
            if !(0..=100).contains(&snapshot) {
                return Err(anyhow::anyhow!("进度快照值必须在 0-100 之间"));
            }
            Some(snapshot)
        } else {
            // 从任务表中获取当前进度
            conn.query_row(
                "SELECT progress FROM tasks WHERE id = ?",
                params![task_id],
                |row| row.get::<_, i32>(0),
            )
            .ok()
        };

        // 插入里程碑
        conn.execute(
            "INSERT INTO task_milestones (task_id, title, description, progress_snapshot)
             VALUES (?, ?, ?, ?)",
            params![
                task_id,
                title.trim(),
                description.map(|d| d.trim()).filter(|d| !d.is_empty()),
                progress_snapshot
            ],
        )?;

        let milestone_id = conn.last_insert_rowid();

        // 查询并返回创建的里程碑
        let milestone = conn.query_row(
            "SELECT id, task_id, title, description, progress_snapshot, created_at
             FROM task_milestones
             WHERE id = ?",
            params![milestone_id],
            |row| {
                Ok(TaskMilestone {
                    id: Some(row.get(0)?),
                    task_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    progress_snapshot: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )?;

        Ok(milestone)
    }

    /// 获取任务的所有里程碑
    pub fn get_task_milestones(conn: &Connection, task_id: i64) -> Result<Vec<TaskMilestone>> {
        let mut stmt = conn.prepare(
            "SELECT id, task_id, title, description, progress_snapshot, created_at
             FROM task_milestones
             WHERE task_id = ?
             ORDER BY created_at DESC",
        )?;

        let milestones = stmt
            .query_map(params![task_id], |row| {
                Ok(TaskMilestone {
                    id: Some(row.get(0)?),
                    task_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    progress_snapshot: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(milestones)
    }

    /// 删除里程碑
    pub fn delete_milestone(conn: &Connection, milestone_id: i64) -> Result<()> {
        conn.execute(
            "DELETE FROM task_milestones WHERE id = ?",
            params![milestone_id],
        )?;
        Ok(())
    }
}
