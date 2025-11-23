use rusqlite::{Connection, Result};

/// 执行数据库迁移，创建所有表和索引
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // 创建 tasks 表
    create_tasks_table(conn)?;
    create_tasks_indexes(conn)?;

    // 创建 sql_history 表
    create_sql_history_table(conn)?;
    create_sql_history_indexes(conn)?;

    // 创建 work_logs 表
    create_work_logs_table(conn)?;
    create_work_logs_indexes(conn)?;

    // 创建 git_commits 表
    create_git_commits_table(conn)?;
    create_git_commits_indexes(conn)?;

    Ok(())
}

/// 创建 tasks 表
fn create_tasks_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT,
            category TEXT DEFAULT 'other',
            priority INTEGER DEFAULT 2,
            status TEXT NOT NULL DEFAULT 'todo',
            git_branch TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            started_at TIMESTAMP,
            last_active_at TIMESTAMP,
            completed_at TIMESTAMP,
            estimated_hours REAL,
            actual_hours REAL,
            context_json TEXT,
            notes TEXT
        )",
        [],
    )?;
    Ok(())
}

/// 创建 tasks 表索引
fn create_tasks_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_category ON tasks(category)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at DESC)",
        [],
    )?;

    Ok(())
}

/// 创建 sql_history 表
fn create_sql_history_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sql_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sql_text TEXT NOT NULL,
            sql_type TEXT,
            database_name TEXT,
            executed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            execution_source TEXT,
            is_favorite BOOLEAN DEFAULT 0,
            tags TEXT,
            description TEXT,
            usage_count INTEGER DEFAULT 1,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

/// 创建 sql_history 表索引
fn create_sql_history_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sql_executed_at ON sql_history(executed_at DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sql_favorite ON sql_history(is_favorite)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sql_type ON sql_history(sql_type)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sql_text ON sql_history(sql_text)",
        [],
    )?;

    // 迁移已存在的表：添加新列（如果不存在）
    migrate_sql_history_add_columns(conn)?;

    Ok(())
}

/// 迁移已存在的 sql_history 表，添加新列
fn migrate_sql_history_add_columns(conn: &Connection) -> Result<()> {
    // 检查 usage_count 列是否存在
    let has_usage_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('sql_history') WHERE name='usage_count'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if has_usage_count == 0 {
        // 添加列（使用常量默认值）
        conn.execute(
            "ALTER TABLE sql_history ADD COLUMN usage_count INTEGER DEFAULT 1",
            [],
        )?;
        println!("✓ 已添加 usage_count 列到 sql_history 表");
    }

    // 检查 created_at 列是否存在
    let has_created_at: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('sql_history') WHERE name='created_at'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if has_created_at == 0 {
        // 先添加列（SQLite不支持非常量默认值，所以先用NULL）
        conn.execute(
            "ALTER TABLE sql_history ADD COLUMN created_at TIMESTAMP",
            [],
        )?;

        // 然后更新已存在记录的值（使用executed_at作为created_at的初始值）
        conn.execute(
            "UPDATE sql_history SET created_at = executed_at WHERE created_at IS NULL",
            [],
        )?;

        println!("✓ 已添加 created_at 列到 sql_history 表");
    }

    Ok(())
}

/// 创建 work_logs 表
fn create_work_logs_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS work_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date DATE NOT NULL UNIQUE,
            log_type TEXT NOT NULL,
            content TEXT NOT NULL,
            ai_generated BOOLEAN DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

/// 创建 work_logs 表索引
fn create_work_logs_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_logs_date ON work_logs(date DESC)",
        [],
    )?;

    Ok(())
}

/// 创建 git_commits 表
fn create_git_commits_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS git_commits (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            commit_hash TEXT UNIQUE NOT NULL,
            branch TEXT NOT NULL,
            message TEXT NOT NULL,
            author TEXT,
            committed_at TIMESTAMP NOT NULL,
            synced_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

/// 创建 git_commits 表索引
fn create_git_commits_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_commits_branch ON git_commits(branch)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_commits_date ON git_commits(committed_at DESC)",
        [],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_run_migrations() {
        // 创建内存数据库进行测试
        let conn = Connection::open_in_memory().unwrap();

        // 执行迁移
        let result = run_migrations(&conn);
        assert!(result.is_ok());

        // 验证 tasks 表已创建
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tasks'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // 验证 sql_history 表已创建
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sql_history'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // 验证 work_logs 表已创建
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='work_logs'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // 验证 git_commits 表已创建
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='git_commits'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);
    }

    #[test]
    fn test_tasks_table_structure() {
        let conn = Connection::open_in_memory().unwrap();
        create_tasks_table(&conn).unwrap();
        create_tasks_indexes(&conn).unwrap();

        // 测试插入数据
        let result = conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status)
             VALUES (?, ?, ?, ?, ?)",
            ["测试任务", "这是一个测试任务", "dev", "1", "todo"],
        );
        assert!(result.is_ok());

        // 验证数据已插入
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sql_history_table_structure() {
        let conn = Connection::open_in_memory().unwrap();
        create_sql_history_table(&conn).unwrap();
        create_sql_history_indexes(&conn).unwrap();

        // 测试插入数据
        let result = conn.execute(
            "INSERT INTO sql_history (sql_text, sql_type, execution_source)
             VALUES (?, ?, ?)",
            ["SELECT * FROM users", "SELECT", "clipboard"],
        );
        assert!(result.is_ok());

        // 验证数据已插入
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sql_history", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_indexes_created() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 验证 tasks 表索引
        let index_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='tasks'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_count >= 4); // 至少有4个索引

        // 验证 sql_history 表索引
        let index_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='sql_history'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_count >= 2); // 至少有2个索引
    }
}
