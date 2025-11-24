use rusqlite::{params, Connection};
use anyhow::Result;
use crate::models::task::{Task, TaskStatus, TaskCategory, TaskPriority, TaskQuadrant, WorkContext, ImportTask, ImportResult};

pub struct TaskService;

impl TaskService {
    /// 获取所有任务（按优先级和最后活跃时间排序）
    pub fn get_all_tasks(conn: &Connection) -> Result<Vec<Task>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, category, priority, status, git_branch,
                    created_at, started_at, last_active_at, completed_at,
                    estimated_hours, actual_hours, context_json, notes, quadrant
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
                    estimated_hours, actual_hours, context_json, notes, quadrant
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
        // 验证标题长度
        if title.trim().is_empty() {
            return Err(anyhow::anyhow!("任务标题不能为空"));
        }
        if title.len() > 200 {
            return Err(anyhow::anyhow!("任务标题不能超过200个字符"));
        }

        // 验证描述长度
        if let Some(desc) = description {
            if desc.len() > 10000 {
                return Err(anyhow::anyhow!("任务描述不能超过10000个字符"));
            }
        }

        let trimmed_title = title.trim();
        let trimmed_description = description.map(|d| d.trim()).filter(|d| !d.is_empty());

        conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status, created_at)
             VALUES (?, ?, ?, ?, 'todo', datetime('now'))",
            params![trimmed_title, trimmed_description, category.as_str(), priority.as_i32()],
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
        quadrant: Option<TaskQuadrant>,
    ) -> Result<()> {
        // 验证标题
        if let Some(t) = title {
            if t.trim().is_empty() {
                return Err(anyhow::anyhow!("任务标题不能为空"));
            }
            if t.len() > 200 {
                return Err(anyhow::anyhow!("任务标题不能超过200个字符"));
            }
        }

        // 验证描述
        if let Some(d) = description {
            if d.len() > 10000 {
                return Err(anyhow::anyhow!("任务描述不能超过10000个字符"));
            }
        }

        // 验证备注
        if let Some(n) = notes {
            if n.len() > 5000 {
                return Err(anyhow::anyhow!("任务备注不能超过5000个字符"));
            }
        }

        let mut updates = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(t) = title {
            updates.push("title = ?");
            params_vec.push(Box::new(t.trim().to_string()));
        }
        if let Some(d) = description {
            updates.push("description = ?");
            let trimmed = d.trim();
            params_vec.push(Box::new(if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }));
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
            let trimmed = n.trim();
            params_vec.push(Box::new(if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }));
        }
        if let Some(q) = quadrant {
            updates.push("quadrant = ?");
            params_vec.push(Box::new(q.as_str().to_string()));
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
                    estimated_hours, actual_hours, context_json, notes, quadrant
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

    /// 批量导入任务
    pub fn import_tasks(conn: &Connection, tasks: Vec<ImportTask>) -> Result<ImportResult> {
        let mut success = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for (index, task) in tasks.iter().enumerate() {
            match Self::import_single_task(conn, task) {
                Ok(_) => success += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(format!("第{}行: {}", index + 1, e));
                }
            }
        }

        Ok(ImportResult {
            success,
            failed,
            errors,
        })
    }

    /// 导入单个任务
    fn import_single_task(conn: &Connection, task: &ImportTask) -> Result<()> {
        // 验证标题
        if task.title.trim().is_empty() {
            return Err(anyhow::anyhow!("任务标题不能为空"));
        }
        if task.title.len() > 200 {
            return Err(anyhow::anyhow!("任务标题不能超过200个字符"));
        }

        // 解析分类
        let category = task.category.as_deref()
            .map(|c| TaskCategory::from_str(c))
            .unwrap_or(TaskCategory::Other);

        // 解析优先级
        let priority = task.priority
            .map(|p| TaskPriority::from_i32(p))
            .unwrap_or(TaskPriority::Medium);

        // 解析状态
        let status = task.status.as_deref()
            .map(|s| TaskStatus::from_str(s))
            .unwrap_or(TaskStatus::Todo);

        // 处理时间字段
        let created_at = task.created_at.as_deref().unwrap_or("datetime('now')");
        let started_at = task.started_at.as_deref();
        let completed_at = task.completed_at.as_deref();

        // 如果有创建时间，使用提供的值；否则使用当前时间
        let sql = if task.created_at.is_some() {
            "INSERT INTO tasks (
                title, description, category, priority, status,
                git_branch, created_at, started_at, completed_at,
                estimated_hours, actual_hours, notes, last_active_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO tasks (
                title, description, category, priority, status,
                git_branch, created_at, started_at, completed_at,
                estimated_hours, actual_hours, notes, last_active_at
            ) VALUES (?, ?, ?, ?, ?, ?, datetime('now'), ?, ?, ?, ?, ?, ?)"
        };

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(task.title.trim().to_string()),
            Box::new(task.description.clone()),
            Box::new(category.as_str().to_string()),
            Box::new(priority.as_i32()),
            Box::new(status.as_str().to_string()),
            Box::new(task.git_branch.clone()),
        ];

        if task.created_at.is_some() {
            params_vec.push(Box::new(created_at.to_string()));
        }

        params_vec.push(Box::new(started_at.map(|s| s.to_string())));
        params_vec.push(Box::new(completed_at.map(|s| s.to_string())));
        params_vec.push(Box::new(task.estimated_hours));
        params_vec.push(Box::new(task.actual_hours));
        params_vec.push(Box::new(task.notes.clone()));

        // last_active_at 使用 completed_at 或 started_at 或 created_at
        let last_active = completed_at
            .or(started_at)
            .or(Some(created_at))
            .map(|s| s.to_string());
        params_vec.push(Box::new(last_active));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec
            .iter()
            .map(|b| b.as_ref())
            .collect();

        conn.execute(sql, params_refs.as_slice())?;

        Ok(())
    }

    /// 生成导入模板（JSON格式）
    pub fn generate_import_template() -> String {
        let template = vec![
            ImportTask {
                title: "示例任务1".to_string(),
                description: Some("这是一个示例任务的描述".to_string()),
                category: Some("dev".to_string()),
                priority: Some(2),
                status: Some("todo".to_string()),
                git_branch: None,
                created_at: Some("2024-01-15 10:00:00".to_string()),
                started_at: None,
                completed_at: None,
                estimated_hours: Some(4.0),
                actual_hours: None,
                notes: Some("备注信息".to_string()),
            },
            ImportTask {
                title: "示例任务2（已完成）".to_string(),
                description: Some("这是一个已完成的任务".to_string()),
                category: Some("ops".to_string()),
                priority: Some(1),
                status: Some("done".to_string()),
                git_branch: Some("feature/example".to_string()),
                created_at: Some("2024-01-10 09:00:00".to_string()),
                started_at: Some("2024-01-10 09:30:00".to_string()),
                completed_at: Some("2024-01-11 18:00:00".to_string()),
                estimated_hours: Some(8.0),
                actual_hours: Some(9.5),
                notes: None,
            },
        ];

        serde_json::to_string_pretty(&template).unwrap()
    }

    /// 按四象限筛选任务
    pub fn get_tasks_by_quadrant(conn: &Connection, quadrant: TaskQuadrant) -> Result<Vec<Task>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, category, priority, status, git_branch,
                    created_at, started_at, last_active_at, completed_at,
                    estimated_hours, actual_hours, context_json, notes, quadrant
             FROM tasks
             WHERE status != 'done' AND quadrant = ?
             ORDER BY priority ASC, last_active_at DESC"
        )?;

        let tasks = stmt.query_map(params![quadrant.as_str()], |row| {
            Self::map_row_to_task(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// 统计各象限任务数量
    pub fn get_quadrant_statistics(conn: &Connection) -> Result<Vec<(String, i64)>> {
        let mut stmt = conn.prepare(
            "SELECT quadrant, COUNT(*) as count
             FROM tasks
             WHERE status != 'done'
             GROUP BY quadrant"
        )?;

        let stats = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(stats)
    }

    /// 辅助方法：将数据库行映射为 Task 对象
    fn map_row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
        // 读取描述字段，如果超过限制则截断
        let description: Option<String> = row.get(2)?;
        let safe_description = description.map(|d| {
            if d.len() > 10000 {
                format!("{}...[内容过长已截断]", &d[..10000])
            } else {
                d
            }
        });

        // 读取context_json，如果解析失败则记录错误并设为None
        let context = row.get::<_, Option<String>>(13)?
            .and_then(|json| {
                if json.len() > 100000 {
                    // context_json过大，直接忽略
                    eprintln!("警告: context_json过大 ({}字节)，已忽略", json.len());
                    None
                } else {
                    match serde_json::from_str(&json) {
                        Ok(ctx) => Some(ctx),
                        Err(e) => {
                            eprintln!("警告: 解析context_json失败: {}", e);
                            None
                        }
                    }
                }
            });

        // 读取备注字段，如果超过限制则截断
        let notes: Option<String> = row.get(14)?;
        let safe_notes = notes.map(|n| {
            if n.len() > 5000 {
                format!("{}...[内容过长已截断]", &n[..5000])
            } else {
                n
            }
        });

        // 读取 quadrant 字段
        let quadrant_str: Option<String> = row.get(15)?;
        let quadrant = quadrant_str.map(|s| TaskQuadrant::from_str(&s));

        Ok(Task {
            id: Some(row.get(0)?),
            title: row.get(1)?,
            description: safe_description,
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
            context,
            notes: safe_notes,
            quadrant,
            tags: None, // 标签需要单独查询，暂时设为 None
        })
    }
}
