use anyhow::Result;
use regex::Regex;
use rusqlite::{params, Connection};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlRecord {
    pub id: Option<i64>,
    pub sql_text: String,
    pub sql_type: Option<String>,
    pub database_name: Option<String>,
    pub executed_at: Option<String>,
    pub execution_source: Option<String>,
    pub is_favorite: bool,
    pub tags: Option<String>,
    pub description: Option<String>,
    pub usage_count: Option<i32>,
    pub created_at: Option<String>,
    pub name: Option<String>,
    // 多标签分类
    pub categories: Vec<SqlCategoryInfo>,
}

/// 分类简要信息（用于 SqlRecord）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlCategoryInfo {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlCategory {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub ai_prompt: Option<String>,
    pub sort_order: Option<i32>,
    pub is_system: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

pub struct SqlService;

impl SqlService {
    /// 识别 SQL 类型
    pub fn detect_sql_type(sql: &str) -> Option<String> {
        let sql_upper = sql.trim().to_uppercase();

        if sql_upper.starts_with("SELECT") {
            Some("SELECT".to_string())
        } else if sql_upper.starts_with("INSERT") {
            Some("INSERT".to_string())
        } else if sql_upper.starts_with("UPDATE") {
            Some("UPDATE".to_string())
        } else if sql_upper.starts_with("DELETE") {
            Some("DELETE".to_string())
        } else if sql_upper.starts_with("CREATE") {
            Some("CREATE".to_string())
        } else if sql_upper.starts_with("ALTER") {
            Some("ALTER".to_string())
        } else if sql_upper.starts_with("DROP") {
            Some("DROP".to_string())
        } else {
            None
        }
    }

    /// 检查是否是有效的 SQL 语句
    pub fn is_valid_sql(text: &str) -> bool {
        let sql_pattern =
            Regex::new(r"(?i)^\s*(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|TRUNCATE)\s+")
                .unwrap();

        sql_pattern.is_match(text)
    }

    /// 保存 SQL 记录（带去重）
    pub fn save_sql(conn: &Connection, sql_text: &str, source: &str) -> Result<i64> {
        let sql_type = Self::detect_sql_type(sql_text);

        // 检查是否已存在相同的SQL
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM sql_history WHERE sql_text = ?",
                params![sql_text],
                |row| row.get(0),
            )
            .ok();

        if let Some(existing_id) = existing {
            // 如果存在，更新执行时间和使用次数
            conn.execute(
                "UPDATE sql_history
                 SET executed_at = datetime('now', 'localtime'),
                     usage_count = usage_count + 1,
                     execution_source = ?
                 WHERE id = ?",
                params![source, existing_id],
            )?;
            println!("✓ SQL已存在，更新使用记录 ID: {}", existing_id);
            Ok(existing_id)
        } else {
            // 不存在，插入新记录
            conn.execute(
                "INSERT INTO sql_history (sql_text, sql_type, execution_source, executed_at, usage_count, created_at)
                 VALUES (?, ?, ?, datetime('now', 'localtime'), 1, datetime('now', 'localtime'))",
                params![sql_text, sql_type, source],
            )?;
            let new_id = conn.last_insert_rowid();
            println!("✓ 新增SQL记录 ID: {}", new_id);
            Ok(new_id)
        }
    }

    /// 获取 SQL 的所有分类标签
    fn get_sql_categories(conn: &Connection, sql_id: i64) -> Vec<SqlCategoryInfo> {
        let mut stmt = match conn.prepare(
            "SELECT c.id, c.name, c.color
             FROM sql_categories c
             INNER JOIN sql_category_mappings m ON c.id = m.category_id
             WHERE m.sql_id = ?
             ORDER BY c.sort_order ASC",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        stmt.query_map(params![sql_id], |row| {
            Ok(SqlCategoryInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row
                    .get::<_, Option<String>>(2)?
                    .unwrap_or_else(|| "#6366f1".to_string()),
            })
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    }

    /// 获取最近的 SQL 记录
    pub fn get_recent_sqls(conn: &Connection, limit: usize) -> Result<Vec<SqlRecord>> {
        let mut stmt = conn.prepare(
            "SELECT id, sql_text, sql_type, database_name, executed_at,
                    execution_source, is_favorite, tags, description, usage_count,
                    created_at, name
             FROM sql_history
             ORDER BY executed_at DESC
             LIMIT ?",
        )?;

        let mut records: Vec<SqlRecord> = stmt
            .query_map(params![limit], |row| {
                Ok(SqlRecord {
                    id: Some(row.get(0)?),
                    sql_text: row.get(1)?,
                    sql_type: row.get(2)?,
                    database_name: row.get(3)?,
                    executed_at: row.get(4)?,
                    execution_source: row.get(5)?,
                    is_favorite: row.get(6)?,
                    tags: row.get(7)?,
                    description: row.get(8)?,
                    usage_count: row.get(9).ok(),
                    created_at: row.get(10).ok(),
                    name: row.get(11).ok(),
                    categories: vec![],
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // 为每条记录加载分类标签
        for record in &mut records {
            if let Some(id) = record.id {
                record.categories = Self::get_sql_categories(conn, id);
            }
        }

        Ok(records)
    }

    /// 获取收藏的 SQL
    pub fn get_favorite_sqls(conn: &Connection) -> Result<Vec<SqlRecord>> {
        let mut stmt = conn.prepare(
            "SELECT id, sql_text, sql_type, database_name, executed_at,
                    execution_source, is_favorite, tags, description, usage_count,
                    created_at, name
             FROM sql_history
             WHERE is_favorite = 1
             ORDER BY executed_at DESC",
        )?;

        let mut records: Vec<SqlRecord> = stmt
            .query_map([], |row| {
                Ok(SqlRecord {
                    id: Some(row.get(0)?),
                    sql_text: row.get(1)?,
                    sql_type: row.get(2)?,
                    database_name: row.get(3)?,
                    executed_at: row.get(4)?,
                    execution_source: row.get(5)?,
                    is_favorite: row.get(6)?,
                    tags: row.get(7)?,
                    description: row.get(8)?,
                    usage_count: row.get(9).ok(),
                    created_at: row.get(10).ok(),
                    name: row.get(11).ok(),
                    categories: vec![],
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // 为每条记录加载分类标签
        for record in &mut records {
            if let Some(id) = record.id {
                record.categories = Self::get_sql_categories(conn, id);
            }
        }

        Ok(records)
    }

    /// 标记为收藏
    pub fn toggle_favorite(conn: &Connection, sql_id: i64) -> Result<()> {
        conn.execute(
            "UPDATE sql_history SET is_favorite = NOT is_favorite WHERE id = ?",
            params![sql_id],
        )?;
        Ok(())
    }

    /// 删除 SQL 记录
    pub fn delete_sql(conn: &Connection, sql_id: i64) -> Result<()> {
        conn.execute("DELETE FROM sql_history WHERE id = ?", params![sql_id])?;
        Ok(())
    }

    /// 更新 SQL 名称
    pub fn update_sql_name(conn: &Connection, sql_id: i64, name: Option<&str>) -> Result<()> {
        conn.execute(
            "UPDATE sql_history SET name = ? WHERE id = ?",
            params![name, sql_id],
        )?;
        Ok(())
    }

    /// 设置 SQL 的分类标签（多个）
    pub fn set_sql_categories(conn: &Connection, sql_id: i64, category_ids: &[i64]) -> Result<()> {
        // 先删除原有的分类关联
        conn.execute(
            "DELETE FROM sql_category_mappings WHERE sql_id = ?",
            params![sql_id],
        )?;

        // 添加新的分类关联
        for category_id in category_ids {
            conn.execute(
                "INSERT OR IGNORE INTO sql_category_mappings (sql_id, category_id) VALUES (?, ?)",
                params![sql_id, category_id],
            )?;
        }

        Ok(())
    }

    /// 为 SQL 添加一个分类标签
    pub fn add_sql_category(conn: &Connection, sql_id: i64, category_id: i64) -> Result<()> {
        conn.execute(
            "INSERT OR IGNORE INTO sql_category_mappings (sql_id, category_id) VALUES (?, ?)",
            params![sql_id, category_id],
        )?;
        Ok(())
    }

    /// 从 SQL 移除一个分类标签
    pub fn remove_sql_category(conn: &Connection, sql_id: i64, category_id: i64) -> Result<()> {
        conn.execute(
            "DELETE FROM sql_category_mappings WHERE sql_id = ? AND category_id = ?",
            params![sql_id, category_id],
        )?;
        Ok(())
    }

    /// 更新 SQL 名称和分类（兼容旧接口）
    pub fn update_sql_name_category(
        conn: &Connection,
        sql_id: i64,
        name: Option<&str>,
        category_id: Option<i64>,
    ) -> Result<()> {
        // 更新名称
        if name.is_some() {
            conn.execute(
                "UPDATE sql_history SET name = ? WHERE id = ?",
                params![name, sql_id],
            )?;
        }

        // 如果提供了分类 ID，设置单个分类
        if let Some(cid) = category_id {
            Self::set_sql_categories(conn, sql_id, &[cid])?;
        }

        Ok(())
    }

    /// 批量更新 SQL 名称和分类（支持多标签）
    pub fn batch_update_sql_name_categories(
        conn: &Connection,
        updates: Vec<(i64, Option<String>, Vec<i64>)>,
    ) -> Result<usize> {
        let mut count = 0;
        for (sql_id, name, category_ids) in updates {
            // 更新名称
            if let Some(n) = &name {
                conn.execute(
                    "UPDATE sql_history SET name = ? WHERE id = ?",
                    params![n, sql_id],
                )?;
            }

            // 设置分类
            if !category_ids.is_empty() {
                Self::set_sql_categories(conn, sql_id, &category_ids)?;
            }

            count += 1;
        }
        Ok(count)
    }

    /// 批量更新 SQL 名称和分类（兼容旧接口）
    pub fn batch_update_sql_name_category(
        conn: &Connection,
        updates: Vec<(i64, Option<String>, Option<i64>)>,
    ) -> Result<usize> {
        let new_updates: Vec<(i64, Option<String>, Vec<i64>)> = updates
            .into_iter()
            .map(|(id, name, cat_id)| {
                let cats = cat_id.map(|c| vec![c]).unwrap_or_default();
                (id, name, cats)
            })
            .collect();
        Self::batch_update_sql_name_categories(conn, new_updates)
    }

    /// 获取所有分类（包含 AI 提示词等完整信息）
    pub fn get_all_categories(conn: &Connection) -> Result<Vec<SqlCategory>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, color, icon, ai_prompt, sort_order, is_system, created_at, updated_at
             FROM sql_categories
             ORDER BY sort_order ASC"
        )?;

        let records = stmt
            .query_map([], |row| {
                Ok(SqlCategory {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    icon: row.get(4)?,
                    ai_prompt: row.get(5)?,
                    sort_order: row.get(6).ok(),
                    is_system: row.get::<_, Option<i32>>(7)?.map(|v| v != 0),
                    created_at: row.get(8).ok(),
                    updated_at: row.get(9).ok(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /// 添加分类（支持 AI 提示词）
    pub fn add_category(
        conn: &Connection,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
        icon: Option<&str>,
        ai_prompt: Option<&str>,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO sql_categories (name, description, color, icon, ai_prompt) VALUES (?, ?, ?, ?, ?)",
            params![name, description, color.unwrap_or("#6366f1"), icon, ai_prompt],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 更新分类（支持 AI 提示词）
    pub fn update_category(
        conn: &Connection,
        category_id: i64,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
        icon: Option<&str>,
        ai_prompt: Option<&str>,
    ) -> Result<()> {
        conn.execute(
            "UPDATE sql_categories SET name = ?, description = ?, color = ?, icon = ?, ai_prompt = ?, updated_at = datetime('now', 'localtime') WHERE id = ?",
            params![name, description, color, icon, ai_prompt, category_id],
        )?;
        Ok(())
    }

    /// 删除分类
    pub fn delete_category(conn: &Connection, category_id: i64) -> Result<()> {
        // 先删除关联的映射关系
        conn.execute(
            "DELETE FROM sql_category_mappings WHERE category_id = ?",
            params![category_id],
        )?;
        // 然后删除分类
        conn.execute(
            "DELETE FROM sql_categories WHERE id = ?",
            params![category_id],
        )?;
        Ok(())
    }

    /// 获取未分类的 SQL 记录（没有任何分类标签或没有名称）
    pub fn get_uncategorized_sqls(conn: &Connection, limit: usize) -> Result<Vec<SqlRecord>> {
        let mut stmt = conn.prepare(
            "SELECT h.id, h.sql_text, h.sql_type, h.database_name, h.executed_at,
                    h.execution_source, h.is_favorite, h.tags, h.description, h.usage_count,
                    h.created_at, h.name
             FROM sql_history h
             WHERE (h.name IS NULL OR h.name = '')
                OR NOT EXISTS (SELECT 1 FROM sql_category_mappings m WHERE m.sql_id = h.id)
             ORDER BY h.executed_at DESC
             LIMIT ?",
        )?;

        let mut records: Vec<SqlRecord> = stmt
            .query_map(params![limit], |row| {
                Ok(SqlRecord {
                    id: Some(row.get(0)?),
                    sql_text: row.get(1)?,
                    sql_type: row.get(2)?,
                    database_name: row.get(3)?,
                    executed_at: row.get(4)?,
                    execution_source: row.get(5)?,
                    is_favorite: row.get(6)?,
                    tags: row.get(7)?,
                    description: row.get(8)?,
                    usage_count: row.get(9).ok(),
                    created_at: row.get(10).ok(),
                    name: row.get(11).ok(),
                    categories: vec![],
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // 为每条记录加载分类标签
        for record in &mut records {
            if let Some(id) = record.id {
                record.categories = Self::get_sql_categories(conn, id);
            }
        }

        Ok(records)
    }

    /// 根据分类获取 SQL 记录（通过多对多关系）
    pub fn get_sqls_by_category(
        conn: &Connection,
        category_id: i64,
        limit: usize,
    ) -> Result<Vec<SqlRecord>> {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT h.id, h.sql_text, h.sql_type, h.database_name, h.executed_at,
                    h.execution_source, h.is_favorite, h.tags, h.description, h.usage_count,
                    h.created_at, h.name
             FROM sql_history h
             INNER JOIN sql_category_mappings m ON h.id = m.sql_id
             WHERE m.category_id = ?
             ORDER BY h.executed_at DESC
             LIMIT ?",
        )?;

        let mut records: Vec<SqlRecord> = stmt
            .query_map(params![category_id, limit], |row| {
                Ok(SqlRecord {
                    id: Some(row.get(0)?),
                    sql_text: row.get(1)?,
                    sql_type: row.get(2)?,
                    database_name: row.get(3)?,
                    executed_at: row.get(4)?,
                    execution_source: row.get(5)?,
                    is_favorite: row.get(6)?,
                    tags: row.get(7)?,
                    description: row.get(8)?,
                    usage_count: row.get(9).ok(),
                    created_at: row.get(10).ok(),
                    name: row.get(11).ok(),
                    categories: vec![],
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // 为每条记录加载所有分类标签
        for record in &mut records {
            if let Some(id) = record.id {
                record.categories = Self::get_sql_categories(conn, id);
            }
        }

        Ok(records)
    }

    /// 获取分类的 SQL 数量统计
    pub fn get_category_sql_counts(conn: &Connection) -> Result<Vec<(i64, i64)>> {
        let mut stmt = conn.prepare(
            "SELECT category_id, COUNT(DISTINCT sql_id) as count
             FROM sql_category_mappings
             GROUP BY category_id",
        )?;

        let records = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }
}
