use crate::models::task_category::TaskCategoryDefinition;
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TaskCategoryService;

impl TaskCategoryService {
    fn map_row(row: &Row<'_>) -> rusqlite::Result<TaskCategoryDefinition> {
        Ok(TaskCategoryDefinition {
            id: Some(row.get(0)?),
            key: row.get(1)?,
            name: row.get(2)?,
            color: row.get(3)?,
            icon: row.get(4)?,
            is_system: row.get::<_, i64>(5)? != 0,
            is_hidden: row.get::<_, i64>(6)? != 0,
            sort_order: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            usage_count: row.get(10)?,
        })
    }

    fn normalize_name(name: &str) -> Result<String, String> {
        let normalized = name.trim();
        let length = normalized.chars().count();
        if length == 0 {
            return Err("分类名称不能为空".to_string());
        }
        if length > 20 {
            return Err("分类名称不能超过20个字符".to_string());
        }
        Ok(normalized.to_string())
    }

    fn normalize_color(color: &str) -> Result<String, String> {
        let normalized = color.trim();
        let is_hex = normalized.len() == 7
            && normalized.starts_with('#')
            && normalized.chars().skip(1).all(|ch| ch.is_ascii_hexdigit());
        if !is_hex {
            return Err("分类颜色必须是 #RRGGBB 格式".to_string());
        }
        Ok(normalized.to_ascii_lowercase())
    }

    pub fn get_all(conn: &Connection) -> Result<Vec<TaskCategoryDefinition>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT c.id, c.category_key, c.name, c.color, c.icon,
                        c.is_system, c.is_hidden, c.sort_order, c.created_at, c.updated_at,
                        COUNT(t.id) AS usage_count
                 FROM task_categories c
                 LEFT JOIN tasks t ON t.category = c.category_key
                 GROUP BY c.id, c.category_key, c.name, c.color, c.icon,
                          c.is_system, c.is_hidden, c.sort_order, c.created_at, c.updated_at
                 ORDER BY c.is_hidden ASC, c.sort_order ASC, c.id ASC",
            )
            .map_err(|error| error.to_string())?;

        let rows = stmt
            .query_map([], Self::map_row)
            .map_err(|error| error.to_string())?;
        let categories = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?;
        Ok(categories)
    }

    pub fn get_by_id(conn: &Connection, id: i64) -> Result<TaskCategoryDefinition, String> {
        conn.query_row(
            "SELECT c.id, c.category_key, c.name, c.color, c.icon,
                    c.is_system, c.is_hidden, c.sort_order, c.created_at, c.updated_at,
                    COUNT(t.id) AS usage_count
             FROM task_categories c
             LEFT JOIN tasks t ON t.category = c.category_key
             WHERE c.id = ?1
             GROUP BY c.id, c.category_key, c.name, c.color, c.icon,
                      c.is_system, c.is_hidden, c.sort_order, c.created_at, c.updated_at",
            params![id],
            Self::map_row,
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "分类不存在".to_string())
    }

    pub fn create(
        conn: &Connection,
        name: &str,
        color: &str,
    ) -> Result<TaskCategoryDefinition, String> {
        let name = Self::normalize_name(name)?;
        let color = Self::normalize_color(color)?;
        let duplicate_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM task_categories WHERE lower(name) = lower(?1)",
                params![name],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if duplicate_count > 0 {
            return Err("分类名称已存在".to_string());
        }

        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        let key = format!("custom_{}", unique_suffix);
        let sort_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM task_categories",
                [],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;

        conn.execute(
            "INSERT INTO task_categories
                (category_key, name, color, icon, is_system, is_hidden, sort_order)
             VALUES (?1, ?2, ?3, 'folder', 0, 0, ?4)",
            params![key, name, color, sort_order],
        )
        .map_err(|error| error.to_string())?;

        Self::get_by_id(conn, conn.last_insert_rowid())
    }

    pub fn update(
        conn: &Connection,
        id: i64,
        name: &str,
        color: &str,
    ) -> Result<TaskCategoryDefinition, String> {
        let current = Self::get_by_id(conn, id)?;
        if current.is_system {
            return Err("系统分类不允许编辑".to_string());
        }

        let name = Self::normalize_name(name)?;
        let color = Self::normalize_color(color)?;
        let duplicate_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM task_categories WHERE lower(name) = lower(?1) AND id <> ?2",
                params![name, id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if duplicate_count > 0 {
            return Err("分类名称已存在".to_string());
        }

        conn.execute(
            "UPDATE task_categories
             SET name = ?1, color = ?2, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?3",
            params![name, color, id],
        )
        .map_err(|error| error.to_string())?;
        Self::get_by_id(conn, id)
    }

    pub fn set_hidden(
        conn: &Connection,
        id: i64,
        is_hidden: bool,
    ) -> Result<TaskCategoryDefinition, String> {
        let current = Self::get_by_id(conn, id)?;
        if is_hidden && !current.is_hidden {
            let visible_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM task_categories WHERE is_hidden = 0",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            if visible_count <= 1 {
                return Err("至少需要保留一个可见分类".to_string());
            }
        }

        conn.execute(
            "UPDATE task_categories
             SET is_hidden = ?1, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?2",
            params![is_hidden as i64, id],
        )
        .map_err(|error| error.to_string())?;
        Self::get_by_id(conn, id)
    }

    pub fn delete(conn: &Connection, id: i64) -> Result<(), String> {
        let current = Self::get_by_id(conn, id)?;
        if current.is_system {
            return Err("系统分类不能删除，可选择隐藏".to_string());
        }

        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "UPDATE tasks SET category = 'other' WHERE category = ?1",
                params![current.key],
            )
            .map_err(|error| error.to_string())?;
        transaction
            .execute("DELETE FROM task_categories WHERE id = ?1", params![id])
            .map_err(|error| error.to_string())?;
        transaction.commit().map_err(|error| error.to_string())
    }
}
