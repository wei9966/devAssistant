use rusqlite::{params, Connection};
use anyhow::Result;
use crate::models::task::{Task, TaskStatus, TaskCategory, TaskPriority, WorkContext};

pub struct TaskService;

impl TaskService {
    /// 获取所有任务（按优先级和最后活跃时间排序）
    pub fn get_all_tasks(conn: &Connection) -> Result<Vec<Task>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, category, priority, status, git_branch,
                    created_at, started_at, last_active_at, completed_at,
                    estimated_hours, actual_hours, context_json, notes
             FROM tasks
             WHERE status != 'done'
             ORDER BY priority ASC, last_active_at DESC"
        )?;

        let tasks = stmt.query_map([], |row| {
            Self::map_row_to_task(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// 获取已完成的任务（最近 7 天）
    pub fn get_completed_tasks(conn: &Connection, days: i64) -> Result<Vec<Task>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, category, priority, status, git_branch,
                    created_at, started_at, last_active_at, completed_at,
                    estimated_hours, actual_hours, context_json, notes
             FROM tasks
             WHERE status = 'done'
               AND completed_at >= datetime('now', ? || ' days')
             ORDER BY completed_at DESC"
        )?;

        let tasks = stmt.query_map(params![format!("-{}", days)], |row| {
            Self::map_row_to_task(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// 创建新任务
    pub fn create_task(
        conn: &Connection,
        title: &str,
        description: Option<&str>,
        category: TaskCategory,
        priority: TaskPriority,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status, created_at)
             VALUES (?, ?, ?, ?, 'todo', datetime('now'))",
            params![title, description, category.as_str(), priority.as_i32()],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// 开始任务（将状态改为 active）
    pub fn start_task(conn: &Connection, task_id: i64) -> Result<()> {
        // 直接开始目标任务，允许多个任务同时进行
        conn.execute(
            "UPDATE tasks
             SET status = 'active',
                 started_at = COALESCE(started_at, datetime('now')),
                 last_active_at = datetime('now')
             WHERE id = ?",
            params![task_id],
        )?;

        Ok(())
    }

    /// 暂停任务（将状态改回 todo，保存上下文）
    pub fn pause_task(conn: &Connection, task_id: i64, context: Option<WorkContext>) -> Result<()> {
        let context_json = context.map(|c| serde_json::to_string(&c).unwrap());

        conn.execute(
            "UPDATE tasks
             SET status = 'todo',
                 last_active_at = datetime('now'),
                 context_json = COALESCE(?, context_json)
             WHERE id = ?",
            params![context_json, task_id],
        )?;

        Ok(())
    }

    /// 延后任务（将状态改为 deferred）
    pub fn defer_task(conn: &Connection, task_id: i64) -> Result<()> {
        conn.execute(
            "UPDATE tasks
             SET status = 'deferred',
                 last_active_at = datetime('now')
             WHERE id = ?",
            params![task_id],
        )?;

        Ok(())
    }

    /// 标记任务完成
    pub fn complete_task(conn: &Connection, task_id: i64) -> Result<()> {
        conn.execute(
            "UPDATE tasks
             SET status = 'done',
                 completed_at = datetime('now'),
                 last_active_at = datetime('now')
             WHERE id = ?",
            params![task_id],
        )?;

        Ok(())
    }

    /// 更新任务信息
    pub fn update_task(
        conn: &Connection,
        task_id: i64,
        title: Option<&str>,
        description: Option<&str>,
        category: Option<TaskCategory>,
        priority: Option<TaskPriority>,
        git_branch: Option<&str>,
        notes: Option<&str>,
    ) -> Result<()> {
        let mut updates = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(t) = title {
            updates.push("title = ?");
            params_vec.push(Box::new(t.to_string()));
        }
        if let Some(d) = description {
            updates.push("description = ?");
            params_vec.push(Box::new(d.to_string()));
        }
        if let Some(c) = category {
            updates.push("category = ?");
            params_vec.push(Box::new(c.as_str().to_string()));
        }
        if let Some(p) = priority {
            updates.push("priority = ?");
            params_vec.push(Box::new(p.as_i32()));
        }
        if let Some(b) = git_branch {
            updates.push("git_branch = ?");
            params_vec.push(Box::new(b.to_string()));
        }
        if let Some(n) = notes {
            updates.push("notes = ?");
            params_vec.push(Box::new(n.to_string()));
        }

        if updates.is_empty() {
            return Ok(());
        }

        params_vec.push(Box::new(task_id));

        let sql = format!(
            "UPDATE tasks SET {} WHERE id = ?",
            updates.join(", ")
        );

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec
            .iter()
            .map(|b| b.as_ref())
            .collect();

        conn.execute(&sql, params_refs.as_slice())?;

        Ok(())
    }

    /// 删除任务
    pub fn delete_task(conn: &Connection, task_id: i64) -> Result<()> {
        conn.execute("DELETE FROM tasks WHERE id = ?", params![task_id])?;
        Ok(())
    }

    /// 获取长时间未处理的待办任务（超过 N 天）
    pub fn get_stale_tasks(conn: &Connection, days: i64) -> Result<Vec<Task>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, category, priority, status, git_branch,
                    created_at, started_at, last_active_at, completed_at,
                    estimated_hours, actual_hours, context_json, notes
             FROM tasks
             WHERE status = 'todo'
               AND created_at < datetime('now', ? || ' days')
               AND (last_active_at IS NULL OR last_active_at < datetime('now', ? || ' days'))
             ORDER BY priority ASC, created_at ASC"
        )?;

        let days_str = format!("-{}", days);
        let tasks = stmt.query_map(params![&days_str, &days_str], |row| {
            Self::map_row_to_task(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// 辅助方法：将数据库行映射为 Task 对象
    fn map_row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
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
            context: row.get::<_, Option<String>>(13)?
                .and_then(|json| serde_json::from_str(&json).ok()),
            notes: row.get(14)?,
        })
    }
}
