use anyhow::Result;
use regex::Regex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// SQL 模板结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlTemplate {
    pub id: i64,
    pub template_text: String,
    pub template_hash: String,
    pub original_sql_sample: Option<String>,
    pub table_names: Vec<String>,
    pub sql_type: Option<String>,
    pub business_scene: Option<String>,
    pub usage_count: i32,
    pub variant_count: i32,
    pub is_favorite: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 整合结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidateResult {
    pub total_processed: i32,
    pub templates_created: i32,
    pub templates_updated: i32,
    pub errors: Vec<String>,
}

/// SQL 记录（用于返回模板变体）
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub template_id: Option<i64>,
}

pub struct SqlTemplateService;

impl SqlTemplateService {
    /// 创建新实例
    pub fn new() -> Self {
        Self
    }

    /// 标准化 SQL - 移除注释、统一空白、替换字面量
    pub fn normalize_sql(sql: &str) -> String {
        let mut normalized = sql.to_string();

        // 1. 移除单行注释 (-- 注释)
        let single_line_comment = Regex::new(r"--[^\n]*").unwrap();
        normalized = single_line_comment.replace_all(&normalized, "").to_string();

        // 2. 移除多行注释 (/* */ 注释)
        let multi_line_comment = Regex::new(r"/\*[\s\S]*?\*/").unwrap();
        normalized = multi_line_comment.replace_all(&normalized, "").to_string();

        // 3. 替换字符串字面量 ('...' 或 "...")
        // 先处理单引号字符串
        let string_literal_single = Regex::new(r"'[^']*'").unwrap();
        normalized = string_literal_single.replace_all(&normalized, "?").to_string();

        // 再处理双引号字符串（非标识符）
        let string_literal_double = Regex::new(r#""[^"]*""#).unwrap();
        normalized = string_literal_double.replace_all(&normalized, "?").to_string();

        // 4. 替换数字字面量（整数、小数、科学计数法）
        let number_literal = Regex::new(r"\b\d+\.?\d*([eE][+-]?\d+)?\b").unwrap();
        normalized = number_literal.replace_all(&normalized, "?").to_string();

        // 5. 统一空白字符（多个空格合并为一个，移除首尾空白）
        let whitespace = Regex::new(r"\s+").unwrap();
        normalized = whitespace.replace_all(&normalized, " ").to_string();
        normalized = normalized.trim().to_string();

        // 6. 统一 SQL 关键字为大写（保持表名和字段名原样）
        normalized = Self::uppercase_keywords(&normalized);

        normalized
    }

    /// 提取 SQL 中涉及的表名
    pub fn extract_table_names(sql: &str) -> Vec<String> {
        let mut tables = Vec::new();
        let sql_upper = sql.to_uppercase();

        // 提取 FROM 子句中的表名
        if let Some(from_match) = Regex::new(r"FROM\s+([a-zA-Z_][\w.]*)")
            .unwrap()
            .captures(&sql_upper)
        {
            if let Some(table) = from_match.get(1) {
                tables.push(table.as_str().to_string());
            }
        }

        // 提取 JOIN 子句中的表名
        let join_pattern = Regex::new(r"JOIN\s+([a-zA-Z_][\w.]*)")
            .unwrap();
        for cap in join_pattern.captures_iter(&sql_upper) {
            if let Some(table) = cap.get(1) {
                tables.push(table.as_str().to_string());
            }
        }

        // 提取 INTO 子句中的表名（INSERT INTO）
        if let Some(into_match) = Regex::new(r"INTO\s+([a-zA-Z_][\w.]*)")
            .unwrap()
            .captures(&sql_upper)
        {
            if let Some(table) = into_match.get(1) {
                tables.push(table.as_str().to_string());
            }
        }

        // 提取 UPDATE 子句中的表名
        if let Some(update_match) = Regex::new(r"UPDATE\s+([a-zA-Z_][\w.]*)")
            .unwrap()
            .captures(&sql_upper)
        {
            if let Some(table) = update_match.get(1) {
                tables.push(table.as_str().to_string());
            }
        }

        // 提取 DELETE FROM 子句中的表名
        if let Some(delete_match) = Regex::new(r"DELETE\s+FROM\s+([a-zA-Z_][\w.]*)")
            .unwrap()
            .captures(&sql_upper)
        {
            if let Some(table) = delete_match.get(1) {
                tables.push(table.as_str().to_string());
            }
        }

        // 去重并排序
        tables.sort();
        tables.dedup();

        tables
    }

    /// 计算模板的 Hash 值（使用 SHA256）
    pub fn calculate_template_hash(normalized_sql: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(normalized_sql.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// 从原始 SQL 生成模板
    pub fn create_template(conn: &Connection, sql_text: &str) -> Result<SqlTemplate> {
        // 标准化 SQL
        let normalized = Self::normalize_sql(sql_text);
        let template_hash = Self::calculate_template_hash(&normalized);
        let table_names = Self::extract_table_names(sql_text);
        let sql_type = Self::detect_sql_type(sql_text);

        // 检查模板是否已存在
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM sql_templates WHERE template_hash = ?",
                params![template_hash],
                |row| row.get(0),
            )
            .ok();

        if let Some(template_id) = existing {
            // 模板已存在，更新使用计数和变体数量
            conn.execute(
                "UPDATE sql_templates
                 SET usage_count = usage_count + 1,
                     updated_at = datetime('now', 'localtime')
                 WHERE id = ?",
                params![template_id],
            )?;

            // 重新查询并返回
            Self::get_template_by_id(conn, template_id)
        } else {
            // 创建新模板
            let table_names_json = serde_json::to_string(&table_names)?;

            conn.execute(
                "INSERT INTO sql_templates (template_text, template_hash, original_sql_sample, table_names, sql_type, usage_count, variant_count)
                 VALUES (?, ?, ?, ?, ?, 1, 1)",
                params![normalized, template_hash, sql_text, table_names_json, sql_type],
            )?;

            let template_id = conn.last_insert_rowid();
            Self::get_template_by_id(conn, template_id)
        }
    }

    /// 智能整合：分析所有未关联模板的 SQL，生成模板
    pub fn consolidate_sql_history(conn: &Connection) -> Result<ConsolidateResult> {
        let mut result = ConsolidateResult {
            total_processed: 0,
            templates_created: 0,
            templates_updated: 0,
            errors: Vec::new(),
        };

        // 查询所有未关联模板的 SQL 记录
        let mut stmt = conn.prepare(
            "SELECT id, sql_text FROM sql_history WHERE template_id IS NULL ORDER BY executed_at DESC"
        )?;

        let sql_records: Vec<(i64, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        for (sql_id, sql_text) in sql_records {
            result.total_processed += 1;

            match Self::process_sql_for_template(conn, sql_id, &sql_text) {
                Ok(is_new) => {
                    if is_new {
                        result.templates_created += 1;
                    } else {
                        result.templates_updated += 1;
                    }
                }
                Err(e) => {
                    result.errors.push(format!("SQL ID {}: {}", sql_id, e));
                }
            }
        }

        Ok(result)
    }

    /// 获取模板列表（支持分组）
    pub fn get_templates(
        conn: &Connection,
        group_by: Option<&str>,
        limit: i32,
    ) -> Result<Vec<SqlTemplate>> {
        let query = match group_by {
            Some("business_scene") => {
                "SELECT id, template_text, template_hash, original_sql_sample, table_names,
                        sql_type, business_scene, usage_count, variant_count, is_favorite,
                        created_at, updated_at
                 FROM sql_templates
                 ORDER BY business_scene, usage_count DESC
                 LIMIT ?"
            }
            Some("sql_type") => {
                "SELECT id, template_text, template_hash, original_sql_sample, table_names,
                        sql_type, business_scene, usage_count, variant_count, is_favorite,
                        created_at, updated_at
                 FROM sql_templates
                 ORDER BY sql_type, usage_count DESC
                 LIMIT ?"
            }
            _ => {
                "SELECT id, template_text, template_hash, original_sql_sample, table_names,
                        sql_type, business_scene, usage_count, variant_count, is_favorite,
                        created_at, updated_at
                 FROM sql_templates
                 ORDER BY usage_count DESC
                 LIMIT ?"
            }
        };

        let mut stmt = conn.prepare(query)?;
        let templates = stmt
            .query_map(params![limit], |row| {
                Ok(SqlTemplate {
                    id: row.get(0)?,
                    template_text: row.get(1)?,
                    template_hash: row.get(2)?,
                    original_sql_sample: row.get(3)?,
                    table_names: serde_json::from_str(row.get::<_, String>(4)?.as_str())
                        .unwrap_or_default(),
                    sql_type: row.get(5)?,
                    business_scene: row.get(6)?,
                    usage_count: row.get(7)?,
                    variant_count: row.get(8)?,
                    is_favorite: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(templates)
    }

    /// 获取模板的所有变体 SQL
    pub fn get_template_variants(conn: &Connection, template_id: i64) -> Result<Vec<SqlRecord>> {
        let mut stmt = conn.prepare(
            "SELECT id, sql_text, sql_type, database_name, executed_at,
                    execution_source, is_favorite, tags, description, usage_count,
                    created_at, name, template_id
             FROM sql_history
             WHERE template_id = ?
             ORDER BY executed_at DESC"
        )?;

        let records = stmt
            .query_map(params![template_id], |row| {
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
                    template_id: row.get(12).ok(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /// 获取热门模板（按使用次数排序）
    pub fn get_hot_templates(conn: &Connection, limit: i32) -> Result<Vec<SqlTemplate>> {
        Self::get_templates(conn, None, limit)
    }

    /// 按表名分组获取模板
    pub fn get_templates_by_table(conn: &Connection, table_name: &str) -> Result<Vec<SqlTemplate>> {
        let mut stmt = conn.prepare(
            "SELECT id, template_text, template_hash, original_sql_sample, table_names,
                    sql_type, business_scene, usage_count, variant_count, is_favorite,
                    created_at, updated_at
             FROM sql_templates
             WHERE table_names LIKE ?
             ORDER BY usage_count DESC"
        )?;

        let search_pattern = format!("%{}%", table_name);
        let templates = stmt
            .query_map(params![search_pattern], |row| {
                Ok(SqlTemplate {
                    id: row.get(0)?,
                    template_text: row.get(1)?,
                    template_hash: row.get(2)?,
                    original_sql_sample: row.get(3)?,
                    table_names: serde_json::from_str(row.get::<_, String>(4)?.as_str())
                        .unwrap_or_default(),
                    sql_type: row.get(5)?,
                    business_scene: row.get(6)?,
                    usage_count: row.get(7)?,
                    variant_count: row.get(8)?,
                    is_favorite: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(templates)
    }

    /// 切换模板收藏状态
    pub fn toggle_template_favorite(conn: &Connection, template_id: i64) -> Result<bool> {
        // 获取当前收藏状态
        let current: bool = conn.query_row(
            "SELECT is_favorite FROM sql_templates WHERE id = ?",
            params![template_id],
            |row| row.get(0),
        )?;

        let new_status = !current;

        // 更新收藏状态
        conn.execute(
            "UPDATE sql_templates SET is_favorite = ?, updated_at = datetime('now', 'localtime') WHERE id = ?",
            params![new_status, template_id],
        )?;

        Ok(new_status)
    }

    /// 更新模板的业务场景
    pub fn update_template_scene(conn: &Connection, template_id: i64, scene: &str) -> Result<()> {
        conn.execute(
            "UPDATE sql_templates SET business_scene = ?, updated_at = datetime('now', 'localtime') WHERE id = ?",
            params![scene, template_id],
        )?;
        Ok(())
    }

    // === 私有辅助方法 ===

    /// 将 SQL 关键字转为大写
    fn uppercase_keywords(sql: &str) -> String {
        let keywords = [
            "SELECT", "FROM", "WHERE", "JOIN", "INNER", "LEFT", "RIGHT", "OUTER", "ON",
            "AND", "OR", "NOT", "IN", "EXISTS", "BETWEEN", "LIKE", "IS", "NULL",
            "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE", "CREATE", "ALTER",
            "DROP", "TABLE", "INDEX", "VIEW", "DATABASE", "SCHEMA", "AS", "ORDER",
            "BY", "GROUP", "HAVING", "LIMIT", "OFFSET", "UNION", "ALL", "DISTINCT",
            "COUNT", "SUM", "AVG", "MAX", "MIN", "CASE", "WHEN", "THEN", "ELSE", "END",
        ];

        let mut result = sql.to_string();
        for keyword in keywords {
            let pattern = format!(r"\b{}\b", keyword);
            let re = Regex::new(&pattern).unwrap();
            result = re
                .replace_all(&result, |_: &regex::Captures| keyword.to_string())
                .to_string();
        }

        result
    }

    /// 识别 SQL 类型
    fn detect_sql_type(sql: &str) -> Option<String> {
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

    /// 根据 ID 获取模板
    fn get_template_by_id(conn: &Connection, template_id: i64) -> Result<SqlTemplate> {
        let template = conn.query_row(
            "SELECT id, template_text, template_hash, original_sql_sample, table_names,
                    sql_type, business_scene, usage_count, variant_count, is_favorite,
                    created_at, updated_at
             FROM sql_templates
             WHERE id = ?",
            params![template_id],
            |row| {
                Ok(SqlTemplate {
                    id: row.get(0)?,
                    template_text: row.get(1)?,
                    template_hash: row.get(2)?,
                    original_sql_sample: row.get(3)?,
                    table_names: serde_json::from_str(row.get::<_, String>(4)?.as_str())
                        .unwrap_or_default(),
                    sql_type: row.get(5)?,
                    business_scene: row.get(6)?,
                    usage_count: row.get(7)?,
                    variant_count: row.get(8)?,
                    is_favorite: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        )?;

        Ok(template)
    }

    /// 处理单个 SQL 记录，生成或更新模板
    fn process_sql_for_template(conn: &Connection, sql_id: i64, sql_text: &str) -> Result<bool> {
        // 标准化 SQL
        let normalized = Self::normalize_sql(sql_text);
        let template_hash = Self::calculate_template_hash(&normalized);
        let table_names = Self::extract_table_names(sql_text);
        let sql_type = Self::detect_sql_type(sql_text);

        // 检查模板是否已存在
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM sql_templates WHERE template_hash = ?",
                params![template_hash],
                |row| row.get(0),
            )
            .ok();

        let (template_id, is_new) = if let Some(tid) = existing {
            // 模板已存在，更新变体计数
            conn.execute(
                "UPDATE sql_templates
                 SET variant_count = variant_count + 1,
                     usage_count = usage_count + 1,
                     updated_at = datetime('now', 'localtime')
                 WHERE id = ?",
                params![tid],
            )?;
            (tid, false)
        } else {
            // 创建新模板
            let table_names_json = serde_json::to_string(&table_names)?;

            conn.execute(
                "INSERT INTO sql_templates (template_text, template_hash, original_sql_sample, table_names, sql_type, usage_count, variant_count)
                 VALUES (?, ?, ?, ?, ?, 1, 1)",
                params![normalized, template_hash, sql_text, table_names_json, sql_type],
            )?;
            (conn.last_insert_rowid(), true)
        };

        // 将 SQL 记录关联到模板
        conn.execute(
            "UPDATE sql_history SET template_id = ? WHERE id = ?",
            params![template_id, sql_id],
        )?;

        Ok(is_new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_sql() {
        // 测试移除注释
        let sql = "SELECT * FROM users -- 查询用户\nWHERE id = 1";
        let normalized = SqlTemplateService::normalize_sql(sql);
        assert!(!normalized.contains("--"));

        // 测试替换字符串字面量
        let sql = "SELECT * FROM users WHERE name = 'John'";
        let normalized = SqlTemplateService::normalize_sql(sql);
        assert!(normalized.contains("WHERE name = ?"));

        // 测试替换数字字面量
        let sql = "SELECT * FROM users WHERE age > 18 AND score = 95.5";
        let normalized = SqlTemplateService::normalize_sql(sql);
        assert!(normalized.contains("age > ?"));
        assert!(normalized.contains("score = ?"));

        // 测试空白字符统一
        let sql = "SELECT   *   FROM    users";
        let normalized = SqlTemplateService::normalize_sql(sql);
        assert_eq!(normalized.split_whitespace().count(), 4);
    }

    #[test]
    fn test_extract_table_names() {
        // 测试 SELECT 语句
        let sql = "SELECT * FROM users";
        let tables = SqlTemplateService::extract_table_names(sql);
        assert!(tables.contains(&"USERS".to_string()));

        // 测试 JOIN 语句
        let sql = "SELECT * FROM users u JOIN orders o ON u.id = o.user_id";
        let tables = SqlTemplateService::extract_table_names(sql);
        assert!(tables.contains(&"USERS".to_string()));
        assert!(tables.contains(&"ORDERS".to_string()));

        // 测试 INSERT 语句
        let sql = "INSERT INTO users (name, age) VALUES ('John', 30)";
        let tables = SqlTemplateService::extract_table_names(sql);
        assert!(tables.contains(&"USERS".to_string()));

        // 测试 UPDATE 语句
        let sql = "UPDATE users SET age = 31 WHERE id = 1";
        let tables = SqlTemplateService::extract_table_names(sql);
        assert!(tables.contains(&"USERS".to_string()));

        // 测试 DELETE 语句
        let sql = "DELETE FROM users WHERE id = 1";
        let tables = SqlTemplateService::extract_table_names(sql);
        assert!(tables.contains(&"USERS".to_string()));
    }

    #[test]
    fn test_calculate_template_hash() {
        let sql1 = "SELECT * FROM users WHERE id = ?";
        let sql2 = "SELECT * FROM users WHERE id = ?";
        let sql3 = "SELECT * FROM orders WHERE id = ?";

        let hash1 = SqlTemplateService::calculate_template_hash(sql1);
        let hash2 = SqlTemplateService::calculate_template_hash(sql2);
        let hash3 = SqlTemplateService::calculate_template_hash(sql3);

        // 相同的 SQL 应该产生相同的 hash
        assert_eq!(hash1, hash2);

        // 不同的 SQL 应该产生不同的 hash
        assert_ne!(hash1, hash3);

        // Hash 应该是 64 位十六进制字符串（SHA256）
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_detect_sql_type() {
        assert_eq!(
            SqlTemplateService::detect_sql_type("SELECT * FROM users"),
            Some("SELECT".to_string())
        );
        assert_eq!(
            SqlTemplateService::detect_sql_type("INSERT INTO users VALUES (1, 'John')"),
            Some("INSERT".to_string())
        );
        assert_eq!(
            SqlTemplateService::detect_sql_type("UPDATE users SET age = 30"),
            Some("UPDATE".to_string())
        );
        assert_eq!(
            SqlTemplateService::detect_sql_type("DELETE FROM users WHERE id = 1"),
            Some("DELETE".to_string())
        );
        assert_eq!(
            SqlTemplateService::detect_sql_type("CREATE TABLE users (id INT)"),
            Some("CREATE".to_string())
        );
    }
}
