use crate::models::task::Tag;
use rusqlite::{params, Connection, Result};

/// 标签服务
pub struct TagService;

impl TagService {
    /// 创建新标签
    pub fn create_tag(conn: &Connection, name: &str, color: &str) -> Result<i64> {
        conn.execute(
            "INSERT INTO tags (name, color, created_at, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![name, color],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 获取所有标签
    pub fn get_all_tags(conn: &Connection) -> Result<Vec<Tag>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at, updated_at FROM tags ORDER BY created_at DESC",
        )?;

        let tags = stmt
            .query_map([], |row| {
                Ok(Tag {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(tags)
    }

    /// 根据ID获取标签
    pub fn get_tag_by_id(conn: &Connection, id: i64) -> Result<Tag> {
        let tag = conn.query_row(
            "SELECT id, name, color, created_at, updated_at FROM tags WHERE id = ?1",
            params![id],
            |row| {
                Ok(Tag {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )?;

        Ok(tag)
    }

    /// 更新标签
    pub fn update_tag(conn: &Connection, id: i64, name: &str, color: &str) -> Result<()> {
        conn.execute(
            "UPDATE tags SET name = ?1, color = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
            params![name, color, id],
        )?;
        Ok(())
    }

    /// 删除标签
    pub fn delete_tag(conn: &Connection, id: i64) -> Result<()> {
        // 外键约束会自动删除 task_tags 中的关联记录
        conn.execute("DELETE FROM tags WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// 为任务添加标签
    pub fn add_tag_to_task(conn: &Connection, task_id: i64, tag_id: i64) -> Result<()> {
        // 检查任务是否存在
        let task_exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE id = ?1",
            params![task_id],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )?;

        if !task_exists {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        // 检查标签是否存在
        let tag_exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM tags WHERE id = ?1",
            params![tag_id],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )?;

        if !tag_exists {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        // 添加关联（UNIQUE约束会防止重复添加）
        conn.execute(
            "INSERT OR IGNORE INTO task_tags (task_id, tag_id, created_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)",
            params![task_id, tag_id],
        )?;

        Ok(())
    }

    /// 从任务移除标签
    pub fn remove_tag_from_task(conn: &Connection, task_id: i64, tag_id: i64) -> Result<()> {
        conn.execute(
            "DELETE FROM task_tags WHERE task_id = ?1 AND tag_id = ?2",
            params![task_id, tag_id],
        )?;
        Ok(())
    }

    /// 获取任务的所有标签
    pub fn get_task_tags(conn: &Connection, task_id: i64) -> Result<Vec<Tag>> {
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at, t.updated_at
             FROM tags t
             INNER JOIN task_tags tt ON t.id = tt.tag_id
             WHERE tt.task_id = ?1
             ORDER BY t.name ASC",
        )?;

        let tags = stmt
            .query_map(params![task_id], |row| {
                Ok(Tag {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(tags)
    }

    /// 获取使用某标签的所有任务ID
    pub fn get_tasks_by_tag(conn: &Connection, tag_id: i64) -> Result<Vec<i64>> {
        let mut stmt = conn
            .prepare("SELECT task_id FROM task_tags WHERE tag_id = ?1 ORDER BY created_at DESC")?;

        let task_ids = stmt
            .query_map(params![tag_id], |row| row.get(0))?
            .collect::<Result<Vec<_>>>()?;

        Ok(task_ids)
    }

    /// 批量为任务添加标签
    pub fn add_tags_to_task(conn: &Connection, task_id: i64, tag_ids: &[i64]) -> Result<()> {
        for tag_id in tag_ids {
            Self::add_tag_to_task(conn, task_id, *tag_id)?;
        }
        Ok(())
    }

    /// 移除任务的所有标签
    pub fn remove_all_tags_from_task(conn: &Connection, task_id: i64) -> Result<()> {
        conn.execute("DELETE FROM task_tags WHERE task_id = ?1", params![task_id])?;
        Ok(())
    }

    /// 获取标签的使用统计
    pub fn get_tag_usage_count(conn: &Connection, tag_id: i64) -> Result<i64> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM task_tags WHERE tag_id = ?1",
            params![tag_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// 获取所有标签及其使用次数
    pub fn get_tags_with_usage_count(conn: &Connection) -> Result<Vec<(Tag, i64)>> {
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at, t.updated_at,
                    COALESCE(COUNT(tt.task_id), 0) as usage_count
             FROM tags t
             LEFT JOIN task_tags tt ON t.id = tt.tag_id
             GROUP BY t.id, t.name, t.color, t.created_at, t.updated_at
             ORDER BY usage_count DESC, t.created_at DESC",
        )?;

        let results = stmt
            .query_map([], |row| {
                let tag = Tag {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                };
                let usage_count: i64 = row.get(5)?;
                Ok((tag, usage_count))
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(results)
    }

    /// 搜索标签（按名称模糊搜索）
    pub fn search_tags(conn: &Connection, keyword: &str) -> Result<Vec<Tag>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at, updated_at
             FROM tags
             WHERE name LIKE ?1
             ORDER BY created_at DESC",
        )?;

        let search_pattern = format!("%{}%", keyword);
        let tags = stmt
            .query_map(params![search_pattern], |row| {
                Ok(Tag {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(tags)
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
    fn test_create_and_get_tag() {
        let conn = setup_test_db();

        let tag_id = TagService::create_tag(&conn, "测试标签", "#FF5733").unwrap();
        assert!(tag_id > 0);

        let tag = TagService::get_tag_by_id(&conn, tag_id).unwrap();
        assert_eq!(tag.name, "测试标签");
        assert_eq!(tag.color, "#FF5733");
    }

    #[test]
    fn test_get_all_tags() {
        let conn = setup_test_db();

        TagService::create_tag(&conn, "标签1", "#FF5733").unwrap();
        TagService::create_tag(&conn, "标签2", "#33FF57").unwrap();

        let tags = TagService::get_all_tags(&conn).unwrap();
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_update_tag() {
        let conn = setup_test_db();

        let tag_id = TagService::create_tag(&conn, "原标签", "#FF5733").unwrap();
        TagService::update_tag(&conn, tag_id, "新标签", "#33FF57").unwrap();

        let tag = TagService::get_tag_by_id(&conn, tag_id).unwrap();
        assert_eq!(tag.name, "新标签");
        assert_eq!(tag.color, "#33FF57");
    }

    #[test]
    fn test_delete_tag() {
        let conn = setup_test_db();

        let tag_id = TagService::create_tag(&conn, "待删除标签", "#FF5733").unwrap();
        TagService::delete_tag(&conn, tag_id).unwrap();

        let result = TagService::get_tag_by_id(&conn, tag_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_tag_to_task() {
        let conn = setup_test_db();

        // 创建任务
        conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status) VALUES (?, ?, ?, ?, ?)",
            params!["测试任务", "描述", "dev", 1, "todo"],
        ).unwrap();
        let task_id = conn.last_insert_rowid();

        // 创建标签
        let tag_id = TagService::create_tag(&conn, "测试标签", "#FF5733").unwrap();

        // 添加标签到任务
        TagService::add_tag_to_task(&conn, task_id, tag_id).unwrap();

        // 验证
        let tags = TagService::get_task_tags(&conn, task_id).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "测试标签");
    }

    #[test]
    fn test_remove_tag_from_task() {
        let conn = setup_test_db();

        // 创建任务
        conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status) VALUES (?, ?, ?, ?, ?)",
            params!["测试任务", "描述", "dev", 1, "todo"],
        ).unwrap();
        let task_id = conn.last_insert_rowid();

        // 创建并添加标签
        let tag_id = TagService::create_tag(&conn, "测试标签", "#FF5733").unwrap();
        TagService::add_tag_to_task(&conn, task_id, tag_id).unwrap();

        // 移除标签
        TagService::remove_tag_from_task(&conn, task_id, tag_id).unwrap();

        // 验证
        let tags = TagService::get_task_tags(&conn, task_id).unwrap();
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_get_tasks_by_tag() {
        let conn = setup_test_db();

        // 创建两个任务
        conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status) VALUES (?, ?, ?, ?, ?)",
            params!["任务1", "描述1", "dev", 1, "todo"],
        ).unwrap();
        let task_id1 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status) VALUES (?, ?, ?, ?, ?)",
            params!["任务2", "描述2", "dev", 1, "todo"],
        ).unwrap();
        let task_id2 = conn.last_insert_rowid();

        // 创建标签并添加到两个任务
        let tag_id = TagService::create_tag(&conn, "共同标签", "#FF5733").unwrap();
        TagService::add_tag_to_task(&conn, task_id1, tag_id).unwrap();
        TagService::add_tag_to_task(&conn, task_id2, tag_id).unwrap();

        // 获取使用该标签的任务
        let task_ids = TagService::get_tasks_by_tag(&conn, tag_id).unwrap();
        assert_eq!(task_ids.len(), 2);
        assert!(task_ids.contains(&task_id1));
        assert!(task_ids.contains(&task_id2));
    }

    #[test]
    fn test_get_tags_with_usage_count() {
        let conn = setup_test_db();

        // 创建任务
        conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status) VALUES (?, ?, ?, ?, ?)",
            params!["任务1", "描述", "dev", 1, "todo"],
        ).unwrap();
        let task_id = conn.last_insert_rowid();

        // 创建标签
        let tag_id1 = TagService::create_tag(&conn, "常用标签", "#FF5733").unwrap();
        let tag_id2 = TagService::create_tag(&conn, "未使用标签", "#33FF57").unwrap();

        // 只为一个标签添加任务关联
        TagService::add_tag_to_task(&conn, task_id, tag_id1).unwrap();

        // 获取使用统计
        let results = TagService::get_tags_with_usage_count(&conn).unwrap();
        assert_eq!(results.len(), 2);

        // 验证排序（使用次数多的在前）
        assert_eq!(results[0].0.name, "常用标签");
        assert_eq!(results[0].1, 1);
        assert_eq!(results[1].0.name, "未使用标签");
        assert_eq!(results[1].1, 0);
    }

    #[test]
    fn test_search_tags() {
        let conn = setup_test_db();

        TagService::create_tag(&conn, "前端开发", "#FF5733").unwrap();
        TagService::create_tag(&conn, "后端开发", "#33FF57").unwrap();
        TagService::create_tag(&conn, "测试", "#5733FF").unwrap();

        let results = TagService::search_tags(&conn, "开发").unwrap();
        assert_eq!(results.len(), 2);

        let results = TagService::search_tags(&conn, "测试").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "测试");
    }
}
