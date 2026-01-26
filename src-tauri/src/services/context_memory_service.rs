use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// 上下文记忆数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMemory {
    pub id: i64,
    pub context_type: String,
    pub key: String,
    pub value: serde_json::Value,
    pub importance: i32,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

/// 上下文类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextType {
    Entity,     // 实体信息（项目、人名、常用任务）
    Preference, // 用户偏好（工作时间、番茄钟时长偏好）
    Fact,       // 事实（"周五要提交报告"）
}

impl ContextType {
    /// 转换为字符串
    pub fn as_str(&self) -> &str {
        match self {
            ContextType::Entity => "entity",
            ContextType::Preference => "preference",
            ContextType::Fact => "fact",
        }
    }

    /// 从字符串转换
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "entity" => ContextType::Entity,
            "preference" => ContextType::Preference,
            "fact" => ContextType::Fact,
            _ => ContextType::Entity, // 默认值
        }
    }
}

/// 上下文记忆服务
pub struct ContextMemoryService;

impl ContextMemoryService {
    /// 保存上下文
    /// 如果 key 已存在，更新 value 和 last_used_at
    /// 如果不存在，插入新记录
    pub fn save_context(
        conn: &Connection,
        context_type: &str,
        key: &str,
        value: &serde_json::Value,
        importance: i32,
    ) -> Result<i64> {
        // 验证 importance 范围 (1-10)
        let importance = importance.clamp(1, 10);

        let value_str = serde_json::to_string(value)?;

        // 检查是否已存在相同的 key
        let existing_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM ai_context_memory WHERE context_type = ? AND key = ?",
                params![context_type, key],
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = existing_id {
            // 更新现有记录
            conn.execute(
                "UPDATE ai_context_memory
                 SET value = ?, importance = ?, last_used_at = datetime('now', 'localtime')
                 WHERE id = ?",
                params![value_str, importance, id],
            )?;
            Ok(id)
        } else {
            // 插入新记录
            conn.execute(
                "INSERT INTO ai_context_memory (context_type, key, value, importance, last_used_at, created_at)
                 VALUES (?, ?, ?, ?, datetime('now', 'localtime'), datetime('now', 'localtime'))",
                params![context_type, key, value_str, importance],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }

    /// 查询相关上下文
    /// 根据关键词匹配 key 或 value 中的内容
    /// 按 importance 和 last_used_at 排序
    pub fn get_relevant_contexts(
        conn: &Connection,
        keywords: &[String],
        limit: usize,
    ) -> Result<Vec<ContextMemory>> {
        let limit = if limit == 0 { 10 } else { limit };

        // 构建 WHERE 子句
        let mut where_clauses = Vec::new();
        let mut params_vec: Vec<String> = Vec::new();

        for keyword in keywords {
            where_clauses.push("(key LIKE ? OR value LIKE ?)");
            let pattern = format!("%{}%", keyword);
            params_vec.push(pattern.clone());
            params_vec.push(pattern);
        }

        let where_clause = if where_clauses.is_empty() {
            "1=1".to_string()
        } else {
            where_clauses.join(" OR ")
        };

        let sql = format!(
            "SELECT id, context_type, key, value, importance, last_used_at, created_at
             FROM ai_context_memory
             WHERE {}
             ORDER BY importance DESC, last_used_at DESC
             LIMIT ?",
            where_clause
        );

        let mut stmt = conn.prepare(&sql)?;

        // 构建参数
        let mut sql_params: Vec<&dyn rusqlite::ToSql> = params_vec
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();
        sql_params.push(&limit);

        let contexts = stmt
            .query_map(sql_params.as_slice(), |row| {
                let value_str: String = row.get(3)?;
                let value = serde_json::from_str(&value_str).unwrap_or(serde_json::Value::Null);

                Ok(ContextMemory {
                    id: row.get(0)?,
                    context_type: row.get(1)?,
                    key: row.get(2)?,
                    value,
                    importance: row.get(4)?,
                    last_used_at: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(contexts)
    }

    /// 获取最近上下文
    /// 获取指定类型的最近使用的上下文
    pub fn get_recent_contexts(
        conn: &Connection,
        context_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ContextMemory>> {
        let limit_i32 = if limit == 0 { 10i32 } else { limit as i32 };

        // 定义行映射闭包
        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<ContextMemory> {
            let value_str: String = row.get(3)?;
            let value = serde_json::from_str(&value_str).unwrap_or(serde_json::Value::Null);

            Ok(ContextMemory {
                id: row.get(0)?,
                context_type: row.get(1)?,
                key: row.get(2)?,
                value,
                importance: row.get(4)?,
                last_used_at: row.get(5)?,
                created_at: row.get(6)?,
            })
        };

        if let Some(ctx_type) = context_type {
            // 带类型过滤的查询
            let sql = "SELECT id, context_type, key, value, importance, last_used_at, created_at
                       FROM ai_context_memory
                       WHERE context_type = ?
                       ORDER BY last_used_at DESC
                       LIMIT ?";

            let mut stmt = conn.prepare(sql)?;
            let contexts = stmt
                .query_map(params![ctx_type, limit_i32], map_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(contexts)
        } else {
            // 不带类型过滤的查询
            let sql = "SELECT id, context_type, key, value, importance, last_used_at, created_at
                       FROM ai_context_memory
                       ORDER BY last_used_at DESC
                       LIMIT ?";

            let mut stmt = conn.prepare(sql)?;
            let contexts = stmt
                .query_map(params![limit_i32], map_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(contexts)
        }
    }

    /// 更新使用时间
    /// 更新 last_used_at 为当前时间
    pub fn touch_context(conn: &Connection, id: i64) -> Result<()> {
        conn.execute(
            "UPDATE ai_context_memory
             SET last_used_at = datetime('now', 'localtime')
             WHERE id = ?",
            params![id],
        )?;
        Ok(())
    }

    /// 清理过期上下文
    /// 删除 N 天未使用且重要性低的上下文
    pub fn cleanup_old_contexts(conn: &Connection, days: i64) -> Result<usize> {
        let affected = conn.execute(
            "DELETE FROM ai_context_memory
             WHERE last_used_at < datetime('now', 'localtime', '-' || ? || ' days')
             AND importance < 5",
            params![days],
        )?;
        Ok(affected)
    }

    /// 删除上下文
    pub fn delete_context(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM ai_context_memory WHERE id = ?", params![id])?;
        Ok(())
    }

    /// 根据 key 获取上下文
    pub fn get_context_by_key(
        conn: &Connection,
        context_type: &str,
        key: &str,
    ) -> Result<Option<ContextMemory>> {
        let result = conn
            .query_row(
                "SELECT id, context_type, key, value, importance, last_used_at, created_at
                 FROM ai_context_memory
                 WHERE context_type = ? AND key = ?",
                params![context_type, key],
                |row| {
                    let value_str: String = row.get(3)?;
                    let value = serde_json::from_str(&value_str).unwrap_or(serde_json::Value::Null);

                    Ok(ContextMemory {
                        id: row.get(0)?,
                        context_type: row.get(1)?,
                        key: row.get(2)?,
                        value,
                        importance: row.get(4)?,
                        last_used_at: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                },
            )
            .ok();

        Ok(result)
    }

    /// 获取所有上下文（用于调试）
    pub fn get_all_contexts(conn: &Connection) -> Result<Vec<ContextMemory>> {
        let mut stmt = conn.prepare(
            "SELECT id, context_type, key, value, importance, last_used_at, created_at
             FROM ai_context_memory
             ORDER BY importance DESC, last_used_at DESC",
        )?;

        let contexts = stmt
            .query_map([], |row| {
                let value_str: String = row.get(3)?;
                let value = serde_json::from_str(&value_str).unwrap_or(serde_json::Value::Null);

                Ok(ContextMemory {
                    id: row.get(0)?,
                    context_type: row.get(1)?,
                    key: row.get(2)?,
                    value,
                    importance: row.get(4)?,
                    last_used_at: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(contexts)
    }

    /// 更新上下文重要性
    pub fn update_importance(conn: &Connection, id: i64, importance: i32) -> Result<()> {
        let importance = importance.clamp(1, 10);
        conn.execute(
            "UPDATE ai_context_memory SET importance = ? WHERE id = ?",
            params![importance, id],
        )?;
        Ok(())
    }

    /// 清理过期的低重要性记忆
    /// - 删除超过 days 天未使用且重要性 < min_importance 的记忆
    pub fn cleanup_expired_memories(
        conn: &Connection,
        days: i64,
        min_importance: i32,
    ) -> Result<i32> {
        let affected = conn.execute(
            "DELETE FROM ai_context_memory
             WHERE last_used_at < datetime('now', 'localtime', '-' || ? || ' days')
             AND importance < ?",
            params![days, min_importance],
        )?;
        Ok(affected as i32)
    }

    /// 清理重复的记忆（相同 context_type 和 key 的记录，只保留最新的）
    pub fn cleanup_duplicate_memories(conn: &Connection) -> Result<i32> {
        let affected = conn.execute(
            "DELETE FROM ai_context_memory
             WHERE id NOT IN (
                 SELECT MAX(id)
                 FROM ai_context_memory
                 GROUP BY context_type, key
             )",
            [],
        )?;
        Ok(affected as i32)
    }

    /// 限制记忆总数（删除最旧的低重要性记忆，保留前 max_count 条）
    pub fn limit_memory_count(conn: &Connection, max_count: i32) -> Result<i32> {
        // 首先获取当前记忆总数
        let total_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM ai_context_memory",
            [],
            |row| row.get(0),
        )?;

        if total_count <= max_count {
            return Ok(0);
        }

        // 删除最旧的低重要性记忆
        let to_delete = total_count - max_count;
        let affected = conn.execute(
            "DELETE FROM ai_context_memory
             WHERE id IN (
                 SELECT id FROM ai_context_memory
                 ORDER BY importance ASC, last_used_at ASC
                 LIMIT ?
             )",
            params![to_delete],
        )?;
        Ok(affected as i32)
    }

    /// 执行完整的记忆维护（组合以上清理操作）
    pub fn perform_memory_maintenance(conn: &Connection) -> Result<MemoryMaintenanceResult> {
        // 1. 清理过期的低重要性记忆（30天未使用且重要性 < 5）
        let expired_deleted = Self::cleanup_expired_memories(conn, 30, 5)?;

        // 2. 清理重复记忆
        let duplicates_deleted = Self::cleanup_duplicate_memories(conn)?;

        // 3. 限制记忆总数（保留前 1000 条）
        let overflow_deleted = Self::limit_memory_count(conn, 1000)?;

        // 4. 获取剩余记忆总数
        let total_remaining: i32 = conn.query_row(
            "SELECT COUNT(*) FROM ai_context_memory",
            [],
            |row| row.get(0),
        )?;

        Ok(MemoryMaintenanceResult {
            expired_deleted,
            duplicates_deleted,
            overflow_deleted,
            total_remaining,
        })
    }
}

/// 记忆维护结果
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryMaintenanceResult {
    pub expired_deleted: i32,
    pub duplicates_deleted: i32,
    pub overflow_deleted: i32,
    pub total_remaining: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;
    use serde_json::json;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_save_and_get_context() {
        let conn = setup_test_db();

        let value = json!({"project": "DevAssistant", "role": "developer"});
        let id = ContextMemoryService::save_context(
            &conn,
            "entity",
            "current_project",
            &value,
            8,
        )
        .unwrap();

        assert!(id > 0);

        let context = ContextMemoryService::get_context_by_key(&conn, "entity", "current_project")
            .unwrap()
            .unwrap();

        assert_eq!(context.key, "current_project");
        assert_eq!(context.context_type, "entity");
        assert_eq!(context.importance, 8);
        assert_eq!(context.value["project"], "DevAssistant");
    }

    #[test]
    fn test_update_existing_context() {
        let conn = setup_test_db();

        let value1 = json!({"status": "active"});
        let id1 = ContextMemoryService::save_context(&conn, "entity", "project_status", &value1, 5)
            .unwrap();

        let value2 = json!({"status": "completed"});
        let id2 = ContextMemoryService::save_context(&conn, "entity", "project_status", &value2, 7)
            .unwrap();

        // 应该返回相同的 ID（更新而非插入）
        assert_eq!(id1, id2);

        let context = ContextMemoryService::get_context_by_key(&conn, "entity", "project_status")
            .unwrap()
            .unwrap();

        assert_eq!(context.value["status"], "completed");
        assert_eq!(context.importance, 7);
    }

    #[test]
    fn test_get_relevant_contexts() {
        let conn = setup_test_db();

        ContextMemoryService::save_context(
            &conn,
            "entity",
            "project_rust",
            &json!({"name": "Rust Project"}),
            8,
        )
        .unwrap();

        ContextMemoryService::save_context(
            &conn,
            "entity",
            "project_python",
            &json!({"name": "Python Project"}),
            6,
        )
        .unwrap();

        ContextMemoryService::save_context(
            &conn,
            "preference",
            "language_preference",
            &json!({"favorite": "Rust"}),
            9,
        )
        .unwrap();

        let keywords = vec!["rust".to_string(), "project".to_string()];
        let contexts = ContextMemoryService::get_relevant_contexts(&conn, &keywords, 10).unwrap();

        assert!(contexts.len() >= 2);
        // 应该按重要性排序
        assert!(contexts[0].importance >= contexts[1].importance);
    }

    #[test]
    fn test_get_recent_contexts() {
        let conn = setup_test_db();

        ContextMemoryService::save_context(
            &conn,
            "entity",
            "task1",
            &json!({"title": "Task 1"}),
            5,
        )
        .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        ContextMemoryService::save_context(
            &conn,
            "entity",
            "task2",
            &json!({"title": "Task 2"}),
            5,
        )
        .unwrap();

        let contexts = ContextMemoryService::get_recent_contexts(&conn, Some("entity"), 10).unwrap();

        assert_eq!(contexts.len(), 2);
        // 最近的应该在前面
        assert_eq!(contexts[0].key, "task2");
    }

    #[test]
    fn test_touch_context() {
        let conn = setup_test_db();

        let id = ContextMemoryService::save_context(
            &conn,
            "entity",
            "test_key",
            &json!({"data": "test"}),
            5,
        )
        .unwrap();

        let context1 = ContextMemoryService::get_context_by_key(&conn, "entity", "test_key")
            .unwrap()
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        ContextMemoryService::touch_context(&conn, id).unwrap();

        let context2 = ContextMemoryService::get_context_by_key(&conn, "entity", "test_key")
            .unwrap()
            .unwrap();

        // last_used_at 应该已更新
        assert!(context2.last_used_at > context1.last_used_at);
    }

    #[test]
    fn test_cleanup_old_contexts() {
        let conn = setup_test_db();

        // 创建一个低重要性的上下文
        let id = ContextMemoryService::save_context(
            &conn,
            "entity",
            "old_task",
            &json!({"title": "Old Task"}),
            3,
        )
        .unwrap();

        // 手动设置为很久以前
        conn.execute(
            "UPDATE ai_context_memory SET last_used_at = datetime('now', 'localtime', '-40 days') WHERE id = ?",
            params![id],
        )
        .unwrap();

        // 清理 30 天未使用的低重要性上下文
        let deleted = ContextMemoryService::cleanup_old_contexts(&conn, 30).unwrap();

        assert_eq!(deleted, 1);

        let context = ContextMemoryService::get_context_by_key(&conn, "entity", "old_task").unwrap();
        assert!(context.is_none());
    }

    #[test]
    fn test_delete_context() {
        let conn = setup_test_db();

        let id = ContextMemoryService::save_context(
            &conn,
            "entity",
            "to_delete",
            &json!({"data": "test"}),
            5,
        )
        .unwrap();

        ContextMemoryService::delete_context(&conn, id).unwrap();

        let context = ContextMemoryService::get_context_by_key(&conn, "entity", "to_delete").unwrap();
        assert!(context.is_none());
    }

    #[test]
    fn test_importance_clamping() {
        let conn = setup_test_db();

        // 测试超出范围的重要性值
        let id1 = ContextMemoryService::save_context(
            &conn,
            "entity",
            "test1",
            &json!({"data": "test"}),
            15, // 超过最大值 10
        )
        .unwrap();

        let context1 = ContextMemoryService::get_context_by_key(&conn, "entity", "test1")
            .unwrap()
            .unwrap();
        assert_eq!(context1.importance, 10);

        let id2 = ContextMemoryService::save_context(
            &conn,
            "entity",
            "test2",
            &json!({"data": "test"}),
            -5, // 低于最小值 1
        )
        .unwrap();

        let context2 = ContextMemoryService::get_context_by_key(&conn, "entity", "test2")
            .unwrap()
            .unwrap();
        assert_eq!(context2.importance, 1);
    }

    #[test]
    fn test_update_importance() {
        let conn = setup_test_db();

        let id = ContextMemoryService::save_context(
            &conn,
            "entity",
            "test_key",
            &json!({"data": "test"}),
            5,
        )
        .unwrap();

        ContextMemoryService::update_importance(&conn, id, 9).unwrap();

        let context = ContextMemoryService::get_context_by_key(&conn, "entity", "test_key")
            .unwrap()
            .unwrap();

        assert_eq!(context.importance, 9);
    }

    #[test]
    fn test_cleanup_expired_memories() {
        let conn = setup_test_db();

        // 创建高重要性的旧记忆（应该保留）
        let id1 = ContextMemoryService::save_context(
            &conn,
            "entity",
            "important_old",
            &json!({"data": "important"}),
            8,
        )
        .unwrap();

        // 创建低重要性的旧记忆（应该被删除）
        let id2 = ContextMemoryService::save_context(
            &conn,
            "entity",
            "unimportant_old",
            &json!({"data": "unimportant"}),
            3,
        )
        .unwrap();

        // 创建低重要性的新记忆（应该保留）
        ContextMemoryService::save_context(
            &conn,
            "entity",
            "unimportant_new",
            &json!({"data": "recent"}),
            2,
        )
        .unwrap();

        // 手动设置前两个记忆为 40 天前
        conn.execute(
            "UPDATE ai_context_memory SET last_used_at = datetime('now', 'localtime', '-40 days') WHERE id IN (?, ?)",
            params![id1, id2],
        )
        .unwrap();

        // 清理 30 天未使用且重要性 < 5 的记忆
        let deleted = ContextMemoryService::cleanup_expired_memories(&conn, 30, 5).unwrap();

        assert_eq!(deleted, 1); // 只删除了低重要性的旧记忆

        // 验证高重要性的旧记忆仍然存在
        let context1 = ContextMemoryService::get_context_by_key(&conn, "entity", "important_old").unwrap();
        assert!(context1.is_some());

        // 验证低重要性的旧记忆已被删除
        let context2 = ContextMemoryService::get_context_by_key(&conn, "entity", "unimportant_old").unwrap();
        assert!(context2.is_none());

        // 验证低重要性的新记忆仍然存在
        let context3 = ContextMemoryService::get_context_by_key(&conn, "entity", "unimportant_new").unwrap();
        assert!(context3.is_some());
    }

    #[test]
    fn test_cleanup_duplicate_memories() {
        let conn = setup_test_db();

        // 创建第一个记忆
        let id1 = ContextMemoryService::save_context(
            &conn,
            "entity",
            "duplicate_key",
            &json!({"version": 1}),
            5,
        )
        .unwrap();

        // 手动插入重复记忆（绕过 save_context 的更新逻辑）
        conn.execute(
            "INSERT INTO ai_context_memory (context_type, key, value, importance, last_used_at, created_at)
             VALUES (?, ?, ?, ?, datetime('now', 'localtime'), datetime('now', 'localtime'))",
            params!["entity", "duplicate_key", json!({"version": 2}).to_string(), 6],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO ai_context_memory (context_type, key, value, importance, last_used_at, created_at)
             VALUES (?, ?, ?, ?, datetime('now', 'localtime'), datetime('now', 'localtime'))",
            params!["entity", "duplicate_key", json!({"version": 3}).to_string(), 7],
        )
        .unwrap();

        // 清理重复记忆
        let deleted = ContextMemoryService::cleanup_duplicate_memories(&conn).unwrap();

        assert_eq!(deleted, 2); // 删除了 2 个重复的旧记忆

        // 验证只保留了最新的一个
        let all_contexts = ContextMemoryService::get_all_contexts(&conn).unwrap();
        let duplicates: Vec<_> = all_contexts
            .iter()
            .filter(|c| c.key == "duplicate_key")
            .collect();
        assert_eq!(duplicates.len(), 1);
    }

    #[test]
    fn test_limit_memory_count() {
        let conn = setup_test_db();

        // 创建 15 个记忆，重要性和时间各不相同
        for i in 1..=15 {
            let id = ContextMemoryService::save_context(
                &conn,
                "entity",
                &format!("memory_{}", i),
                &json!({"index": i}),
                if i <= 5 { 8 } else { 3 }, // 前 5 个高重要性，后 10 个低重要性
            )
            .unwrap();

            // 为了确保时间不同，稍微调整时间
            if i > 10 {
                conn.execute(
                    "UPDATE ai_context_memory SET last_used_at = datetime('now', 'localtime', '-' || ? || ' hours') WHERE id = ?",
                    params![i, id],
                )
                .unwrap();
            }
        }

        // 限制为 10 条记忆
        let deleted = ContextMemoryService::limit_memory_count(&conn, 10).unwrap();

        assert_eq!(deleted, 5); // 删除了 5 条

        // 验证剩余 10 条
        let remaining = ContextMemoryService::get_all_contexts(&conn).unwrap();
        assert_eq!(remaining.len(), 10);

        // 验证高重要性的记忆都被保留
        let high_importance: Vec<_> = remaining.iter().filter(|c| c.importance >= 8).collect();
        assert_eq!(high_importance.len(), 5);
    }

    #[test]
    fn test_limit_memory_count_below_max() {
        let conn = setup_test_db();

        // 只创建 5 个记忆
        for i in 1..=5 {
            ContextMemoryService::save_context(
                &conn,
                "entity",
                &format!("memory_{}", i),
                &json!({"index": i}),
                5,
            )
            .unwrap();
        }

        // 尝试限制为 10 条（当前只有 5 条）
        let deleted = ContextMemoryService::limit_memory_count(&conn, 10).unwrap();

        assert_eq!(deleted, 0); // 不应该删除任何记忆

        let remaining = ContextMemoryService::get_all_contexts(&conn).unwrap();
        assert_eq!(remaining.len(), 5);
    }

    #[test]
    fn test_perform_memory_maintenance() {
        let conn = setup_test_db();

        // 创建各种类型的记忆

        // 1. 过期的低重要性记忆
        let expired_id = ContextMemoryService::save_context(
            &conn,
            "entity",
            "expired_memory",
            &json!({"data": "old"}),
            3,
        )
        .unwrap();
        conn.execute(
            "UPDATE ai_context_memory SET last_used_at = datetime('now', 'localtime', '-40 days') WHERE id = ?",
            params![expired_id],
        )
        .unwrap();

        // 2. 重复的记忆
        ContextMemoryService::save_context(
            &conn,
            "entity",
            "duplicate_key",
            &json!({"version": 1}),
            5,
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_context_memory (context_type, key, value, importance, last_used_at, created_at)
             VALUES (?, ?, ?, ?, datetime('now', 'localtime'), datetime('now', 'localtime'))",
            params!["entity", "duplicate_key", json!({"version": 2}).to_string(), 5],
        )
        .unwrap();

        // 3. 正常的记忆
        for i in 1..=10 {
            ContextMemoryService::save_context(
                &conn,
                "entity",
                &format!("normal_memory_{}", i),
                &json!({"index": i}),
                6,
            )
            .unwrap();
        }

        // 执行维护
        let result = ContextMemoryService::perform_memory_maintenance(&conn).unwrap();

        // 验证结果
        assert_eq!(result.expired_deleted, 1); // 删除了 1 个过期记忆
        assert_eq!(result.duplicates_deleted, 1); // 删除了 1 个重复记忆
        assert!(result.total_remaining > 0); // 还有剩余记忆

        // 验证过期记忆已被删除
        let expired = ContextMemoryService::get_context_by_key(&conn, "entity", "expired_memory").unwrap();
        assert!(expired.is_none());

        // 验证重复记忆只保留一个
        let all = ContextMemoryService::get_all_contexts(&conn).unwrap();
        let duplicates: Vec<_> = all.iter().filter(|c| c.key == "duplicate_key").collect();
        assert_eq!(duplicates.len(), 1);
    }
}
