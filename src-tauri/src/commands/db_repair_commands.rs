use crate::db::DbConnection;
use rusqlite::{params, Connection};
use tauri::State;

/// 修复数据库中的超长字段
#[tauri::command]
pub fn repair_database(db: State<DbConnection>) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut repaired_count = 0;
    let mut errors = Vec::new();

    // 查询所有任务
    let mut stmt = conn
        .prepare("SELECT id, description, notes, context_json FROM tasks")
        .map_err(|e| e.to_string())?;

    let tasks: Vec<(i64, Option<String>, Option<String>, Option<String>)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    drop(stmt); // 释放查询语句

    // 修复每个任务
    for (task_id, description, notes, context_json) in tasks {
        let mut need_update = false;
        let mut updates = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // 安全截断辅助函数（按字符数，不是字节数）
        fn safe_truncate(s: &str, max_chars: usize) -> String {
            s.chars().take(max_chars).collect()
        }

        // 检查并截断description
        if let Some(desc) = description {
            let char_count = desc.chars().count();
            if char_count > 3000 {
                updates.push("description = ?");
                let truncated = format!(
                    "{}...[已自动截断，原长度:{}字符]",
                    safe_truncate(&desc, 3000),
                    char_count
                );
                params_vec.push(Box::new(truncated));
                need_update = true;
                eprintln!("任务 {} 的描述过长 ({}字符)，已截断", task_id, char_count);
            }
        }

        // 检查并截断notes
        if let Some(n) = notes {
            let char_count = n.chars().count();
            if char_count > 1500 {
                updates.push("notes = ?");
                let truncated = format!("{}...[已自动截断，原长度:{}字符]", safe_truncate(&n, 1500), char_count);
                params_vec.push(Box::new(truncated));
                need_update = true;
                eprintln!("任务 {} 的备注过长 ({}字符)，已截断", task_id, char_count);
            }
        }

        // 检查并清理context_json
        if let Some(ctx) = context_json {
            if ctx.len() > 100000 {
                updates.push("context_json = NULL");
                need_update = true;
                eprintln!("任务 {} 的上下文过大 ({}字节)，已清空", task_id, ctx.len());
            }
        }

        if need_update {
            params_vec.push(Box::new(task_id));
            let sql = format!("UPDATE tasks SET {} WHERE id = ?", updates.join(", "));

            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|b| b.as_ref()).collect();

            match conn.execute(&sql, params_refs.as_slice()) {
                Ok(_) => repaired_count += 1,
                Err(e) => errors.push(format!("修复任务 {} 失败: {}", task_id, e)),
            }
        }
    }

    let mut result = format!("数据库修复完成！修复了 {} 个任务", repaired_count);
    if !errors.is_empty() {
        result.push_str(&format!(
            "\n遇到 {} 个错误:\n{}",
            errors.len(),
            errors.join("\n")
        ));
    }

    Ok(result)
}

/// 获取数据库统计信息
#[tauri::command]
pub fn get_database_stats(db: State<DbConnection>) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT
            COUNT(*) as total_tasks,
            COUNT(CASE WHEN length(description) > 10000 THEN 1 END) as large_descriptions,
            COUNT(CASE WHEN length(notes) > 5000 THEN 1 END) as large_notes,
            COUNT(CASE WHEN length(context_json) > 100000 THEN 1 END) as large_contexts,
            MAX(length(description)) as max_desc_len,
            MAX(length(notes)) as max_notes_len,
            MAX(length(context_json)) as max_context_len
         FROM tasks",
        )
        .map_err(|e| e.to_string())?;

    let stats = stmt
        .query_row([], |row| {
            Ok(format!(
                "数据库统计信息:\n\
            总任务数: {}\n\
            超长描述(>10000): {}\n\
            超长备注(>5000): {}\n\
            超大上下文(>100000): {}\n\
            最大描述长度: {} 字符\n\
            最大备注长度: {} 字符\n\
            最大上下文长度: {} 字节",
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<i64>>(4)?.unwrap_or(0),
                row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                row.get::<_, Option<i64>>(6)?.unwrap_or(0),
            ))
        })
        .map_err(|e| e.to_string())?;

    Ok(stats)
}
