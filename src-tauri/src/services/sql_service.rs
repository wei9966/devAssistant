use rusqlite::{params, Connection};
use anyhow::Result;
use regex::Regex;

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
        let sql_pattern = Regex::new(
            r"(?i)^\s*(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|TRUNCATE)\s+"
        ).unwrap();

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
                 SET executed_at = datetime('now'),
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
                 VALUES (?, ?, ?, datetime('now'), 1, datetime('now'))",
                params![sql_text, sql_type, source],
            )?;
            let new_id = conn.last_insert_rowid();
            println!("✓ 新增SQL记录 ID: {}", new_id);
            Ok(new_id)
        }
    }

    /// 获取最近的 SQL 记录
    pub fn get_recent_sqls(conn: &Connection, limit: usize) -> Result<Vec<SqlRecord>> {
        let mut stmt = conn.prepare(
            "SELECT id, sql_text, sql_type, database_name, executed_at,
                    execution_source, is_favorite, tags, description, usage_count, created_at
             FROM sql_history
             ORDER BY executed_at DESC
             LIMIT ?"
        )?;

        let records = stmt.query_map(params![limit], |row| {
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
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /// 获取收藏的 SQL
    pub fn get_favorite_sqls(conn: &Connection) -> Result<Vec<SqlRecord>> {
        let mut stmt = conn.prepare(
            "SELECT id, sql_text, sql_type, database_name, executed_at,
                    execution_source, is_favorite, tags, description, usage_count, created_at
             FROM sql_history
             WHERE is_favorite = 1
             ORDER BY executed_at DESC"
        )?;

        let records = stmt.query_map([], |row| {
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
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

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
}
