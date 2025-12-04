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

    // 创建 AppLauncher 表
    create_apps_table(conn)?;
    create_apps_indexes(conn)?;

    create_categories_table(conn)?;
    create_categories_indexes(conn)?;

    create_workflows_table(conn)?;
    create_workflows_indexes(conn)?;

    create_launch_history_table(conn)?;
    create_launch_history_indexes(conn)?;

    // 创建 app_settings 表
    create_app_settings_table(conn)?;

    // 创建标签相关表
    create_tags_table(conn)?;
    create_tags_indexes(conn)?;

    create_task_tags_table(conn)?;
    create_task_tags_indexes(conn)?;

    // 迁移 tasks 表：添加 quadrant 字段
    migrate_tasks_add_quadrant(conn)?;

    // 创建 SQL 分类表
    create_sql_categories_table(conn)?;
    create_sql_categories_indexes(conn)?;

    // 创建 SQL 与分类的多对多关联表
    create_sql_category_mappings_table(conn)?;

    // 迁移 sql_history 表：添加 name 字段
    migrate_sql_history_add_name_category(conn)?;

    // 迁移 sql_categories 表：添加新字段
    migrate_sql_categories_add_fields(conn)?;

    // 创建 AI 全局配置表
    create_ai_config_table(conn)?;

    // 创建 AI 调用日志表
    create_ai_logs_table(conn)?;

    // 迁移 apps 表:添加 item_type 字段
    migrate_apps_add_item_type(conn)?;

    // 创建应用启动器设置表
    create_app_launcher_settings_table(conn)?;

    // 迁移 tasks 表：添加 due_date 和 registered_at 字段
    migrate_tasks_add_date_fields(conn)?;

    // 迁移 tasks 表：添加 current_date 字段（日历显示日期）
    migrate_tasks_add_current_date(conn)?;

    // 创建周计划表
    create_weekly_plans_table(conn)?;
    create_weekly_plans_indexes(conn)?;

    // 创建剪切板历史表
    create_clipboard_history_table(conn)?;
    create_clipboard_history_indexes(conn)?;

    // 创建屏幕上下文相关表
    create_screen_contexts_table(conn)?;
    create_screen_contexts_indexes(conn)?;
    create_vlm_config_table(conn)?;
    create_daily_summaries_table(conn)?;
    migrate_work_logs_add_context_ids(conn)?;

    // 创建通知中心相关表
    create_notifications_table(conn)?;
    create_notifications_indexes(conn)?;
    create_notification_settings_table(conn)?;

    // 创建日报表
    create_daily_reports_table(conn)?;
    create_daily_reports_indexes(conn)?;

    // 创建活动总结表
    create_activity_summaries_table(conn)?;
    create_activity_summaries_indexes(conn)?;

    // 创建智能提示表
    create_tips_table(conn)?;
    create_tips_indexes(conn)?;

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

/// 创建 apps 表（应用管理）
fn create_apps_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS apps (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            icon TEXT,
            category TEXT,
            tags TEXT,
            launch_count INTEGER DEFAULT 0,
            last_launched_at INTEGER,
            is_pinned INTEGER DEFAULT 0,
            is_hidden INTEGER DEFAULT 0,
            launch_args TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(())
}

/// 创建 apps 表索引
fn create_apps_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_apps_category ON apps(category)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_apps_launch_count ON apps(launch_count DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_apps_last_launched ON apps(last_launched_at DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_apps_is_pinned ON apps(is_pinned)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_apps_is_hidden ON apps(is_hidden)",
        [],
    )?;

    Ok(())
}

/// 创建 categories 表（应用分类）
fn create_categories_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT,
            icon TEXT,
            sort_order INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(())
}

/// 创建 categories 表索引
fn create_categories_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_categories_sort_order ON categories(sort_order ASC)",
        [],
    )?;

    Ok(())
}

/// 创建 workflows 表（工作流）
fn create_workflows_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workflows (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            app_ids TEXT NOT NULL,
            launch_delay INTEGER,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(())
}

/// 创建 workflows 表索引
fn create_workflows_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_workflows_created_at ON workflows(created_at DESC)",
        [],
    )?;

    Ok(())
}

/// 创建 launch_history 表（启动历史）
fn create_launch_history_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS launch_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app_id TEXT NOT NULL,
            launched_at INTEGER NOT NULL,
            FOREIGN KEY (app_id) REFERENCES apps(id)
        )",
        [],
    )?;
    Ok(())
}

/// 创建 launch_history 表索引
fn create_launch_history_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_launch_history_app_id ON launch_history(app_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_launch_history_launched_at ON launch_history(launched_at DESC)",
        [],
    )?;

    Ok(())
}

/// 创建 daily_reports 表（日报）
fn create_daily_reports_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS daily_reports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL UNIQUE,
            summary_text TEXT NOT NULL,
            highlights TEXT NOT NULL,
            insights TEXT NOT NULL,
            total_screenshots INTEGER NOT NULL,
            activity_breakdown TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now', 'localtime'))
        )",
        [],
    )?;
    Ok(())
}

/// 创建 daily_reports 表索引
fn create_daily_reports_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_daily_reports_date ON daily_reports(date DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_daily_reports_created_at ON daily_reports(created_at DESC)",
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

        // 验证 apps 表已创建
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='apps'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // 验证 categories 表已创建
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='categories'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // 验证 workflows 表已创建
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workflows'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // 验证 launch_history 表已创建
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='launch_history'",
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

    #[test]
    fn test_apps_table_structure() {
        let conn = Connection::open_in_memory().unwrap();
        create_apps_table(&conn).unwrap();
        create_apps_indexes(&conn).unwrap();

        // 测试插入数据
        let result = conn.execute(
            "INSERT INTO apps (id, name, path, category, launch_count, is_pinned, is_hidden, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                "vscode",
                "Visual Studio Code",
                "C:\\Program Files\\Microsoft VS Code\\Code.exe",
                "dev",
                "10",
                "1",
                "0",
                "1732435200",
                "1732435200",
            ],
        );
        assert!(result.is_ok());

        // 验证数据已插入
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM apps", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        // 验证数据正确性
        let name: String = conn
            .query_row("SELECT name FROM apps WHERE id = ?", ["vscode"], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(name, "Visual Studio Code");
    }

    #[test]
    fn test_categories_table_structure() {
        let conn = Connection::open_in_memory().unwrap();
        create_categories_table(&conn).unwrap();
        create_categories_indexes(&conn).unwrap();

        // 测试插入数据
        let result = conn.execute(
            "INSERT INTO categories (id, name, color, sort_order, created_at)
             VALUES (?, ?, ?, ?, ?)",
            ["dev", "开发工具", "#6366f1", "1", "1732435200"],
        );
        assert!(result.is_ok());

        // 验证数据已插入
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_workflows_table_structure() {
        let conn = Connection::open_in_memory().unwrap();
        create_workflows_table(&conn).unwrap();
        create_workflows_indexes(&conn).unwrap();

        // 测试插入数据
        let result = conn.execute(
            "INSERT INTO workflows (id, name, app_ids, launch_delay, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            [
                "frontend-dev",
                "前端开发环境",
                "[\"vscode\",\"chrome\",\"terminal\"]",
                "500",
                "1732435200",
                "1732435200",
            ],
        );
        assert!(result.is_ok());

        // 验证数据已插入
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM workflows", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_launch_history_table_structure() {
        let conn = Connection::open_in_memory().unwrap();
        create_apps_table(&conn).unwrap();
        create_launch_history_table(&conn).unwrap();
        create_launch_history_indexes(&conn).unwrap();

        // 先插入一个应用
        conn.execute(
            "INSERT INTO apps (id, name, path, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?)",
            [
                "vscode",
                "Visual Studio Code",
                "C:\\Code.exe",
                "1732435200",
                "1732435200",
            ],
        )
        .unwrap();

        // 测试插入启动历史
        let result = conn.execute(
            "INSERT INTO launch_history (app_id, launched_at)
             VALUES (?, ?)",
            ["vscode", "1732435200"],
        );
        assert!(result.is_ok());

        // 验证数据已插入
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM launch_history", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_app_launcher_indexes() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 验证 apps 表索引
        let index_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='apps'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_count >= 5); // 至少有5个索引

        // 验证 launch_history 表索引
        let index_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='launch_history'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_count >= 2); // 至少有2个索引
    }

    #[test]
    fn test_tags_table_created() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 验证 tags 表已创建
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tags'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // 测试插入标签
        let result = conn.execute(
            "INSERT INTO tags (name, color) VALUES (?, ?)",
            ["测试标签", "#FF5733"],
        );
        assert!(result.is_ok());

        // 验证数据已插入
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tags", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_task_tags_table_created() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 验证 task_tags 表已创建
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='task_tags'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // 先创建一个任务和一个标签
        conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status) VALUES (?, ?, ?, ?, ?)",
            ["测试任务", "测试描述", "dev", "1", "todo"],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tags (name, color) VALUES (?, ?)",
            ["测试标签", "#FF5733"],
        )
        .unwrap();

        // 测试插入任务-标签关联
        let result = conn.execute(
            "INSERT INTO task_tags (task_id, tag_id) VALUES (?, ?)",
            ["1", "1"],
        );
        assert!(result.is_ok());

        // 验证数据已插入
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM task_tags", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_tasks_quadrant_field() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 验证 quadrant 字段存在
        let has_quadrant: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('tasks') WHERE name='quadrant'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(has_quadrant, 1);

        // 测试插入带 quadrant 的任务
        let result = conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status, quadrant)
             VALUES (?, ?, ?, ?, ?, ?)",
            [
                "测试任务",
                "测试描述",
                "dev",
                "1",
                "todo",
                "urgent_important",
            ],
        );
        assert!(result.is_ok());

        // 验证 quadrant 值正确
        let quadrant: String = conn
            .query_row(
                "SELECT quadrant FROM tasks WHERE title = ?",
                ["测试任务"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(quadrant, "urgent_important");
    }

    #[test]
    fn test_quadrant_default_value() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 插入不指定 quadrant 的任务
        conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status)
             VALUES (?, ?, ?, ?, ?)",
            ["测试任务", "测试描述", "dev", "1", "todo"],
        )
        .unwrap();

        // 验证默认值为 urgent_not_important
        let quadrant: String = conn
            .query_row(
                "SELECT quadrant FROM tasks WHERE title = ?",
                ["测试任务"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(quadrant, "urgent_not_important");
    }
}

/// 创建 app_settings 表（应用设置）
fn create_app_settings_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;

    Ok(())
}

/// 创建 tags 表（标签）
fn create_tags_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT '#6366f1',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

/// 创建 tags 表索引
fn create_tags_indexes(conn: &Connection) -> Result<()> {
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name)", [])?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tags_created_at ON tags(created_at DESC)",
        [],
    )?;

    Ok(())
}

/// 创建 task_tags 关联表（任务-标签多对多关系）
fn create_task_tags_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
            UNIQUE(task_id, tag_id)
        )",
        [],
    )?;
    Ok(())
}

/// 创建 task_tags 表索引
fn create_task_tags_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_task_tags_task_id ON task_tags(task_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_task_tags_tag_id ON task_tags(tag_id)",
        [],
    )?;

    Ok(())
}

/// 创建 SQL 分类表（支持多标签和规则提示词）
fn create_sql_categories_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sql_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            color TEXT DEFAULT '#6366f1',
            icon TEXT,
            ai_prompt TEXT,
            sort_order INTEGER DEFAULT 0,
            is_system INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 插入默认分类（带有 AI 提示词）
    let default_categories = [
        ("MySQL", "MySQL 数据库", "#00758f", "mysql",
         "识别规则：表名和字段名使用小写字母和下划线命名（snake_case），如 user_info, order_detail。常见关键字：LIMIT, AUTO_INCREMENT, IFNULL, DATE_FORMAT 等", 1, 1),
        ("SQLServer", "SQL Server 数据库", "#cc2927", "sqlserver",
         "识别规则：表名和字段名使用大驼峰命名（PascalCase）且无下划线，如 UserInfo, OrderDetail。常见关键字：TOP, GETDATE(), ISNULL, CONVERT, DATEPART 等", 2, 1),
        ("Oracle", "Oracle 数据库", "#f80000", "oracle",
         "识别规则：表名通常全大写，使用 ROWNUM, NVL, TO_DATE, TO_CHAR, SYSDATE, DUAL 等 Oracle 特有语法", 3, 1),
        ("PostgreSQL", "PostgreSQL 数据库", "#336791", "postgresql",
         "识别规则：使用 :: 类型转换，COALESCE, NULLIF, NOW(), CURRENT_DATE 等，支持 JSON 操作符 ->, ->>", 4, 1),
        ("查询", "数据查询类SQL", "#10b981", "search",
         "SELECT 语句，用于数据检索和查询", 10, 1),
        ("更新", "数据更新类SQL", "#f59e0b", "edit",
         "UPDATE, INSERT, DELETE 等数据修改语句", 11, 1),
        ("DDL", "表结构定义", "#8b5cf6", "build",
         "CREATE, ALTER, DROP 等表结构操作", 12, 1),
    ];

    for (name, desc, color, icon, prompt, order, is_system) in default_categories {
        conn.execute(
            "INSERT OR IGNORE INTO sql_categories (name, description, color, icon, ai_prompt, sort_order, is_system)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![name, desc, color, icon, prompt, order, is_system],
        )?;
    }

    Ok(())
}

/// 创建 SQL 分类表索引
fn create_sql_categories_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sql_categories_name ON sql_categories(name)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sql_categories_sort_order ON sql_categories(sort_order ASC)",
        [],
    )?;

    Ok(())
}

/// 创建 SQL 与分类的多对多关联表
fn create_sql_category_mappings_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sql_category_mappings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sql_id INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (sql_id) REFERENCES sql_history(id) ON DELETE CASCADE,
            FOREIGN KEY (category_id) REFERENCES sql_categories(id) ON DELETE CASCADE,
            UNIQUE(sql_id, category_id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sql_category_mappings_sql_id ON sql_category_mappings(sql_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sql_category_mappings_category_id ON sql_category_mappings(category_id)",
        [],
    )?;

    Ok(())
}

/// 迁移 sql_history 表：添加 name 字段
fn migrate_sql_history_add_name_category(conn: &Connection) -> Result<()> {
    // 检查 name 列是否存在
    let has_name: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('sql_history') WHERE name='name'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if has_name == 0 {
        conn.execute("ALTER TABLE sql_history ADD COLUMN name TEXT", [])?;
        println!("✓ 已添加 name 列到 sql_history 表");

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sql_name ON sql_history(name)",
            [],
        )?;
    }

    // 兼容旧的 category_id 字段（如果存在则保留，新数据使用多对多关系）
    let has_category_id: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('sql_history') WHERE name='category_id'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if has_category_id == 0 {
        conn.execute("ALTER TABLE sql_history ADD COLUMN category_id INTEGER", [])?;
    }

    Ok(())
}

/// 迁移 sql_categories 表：添加新字段
fn migrate_sql_categories_add_fields(conn: &Connection) -> Result<()> {
    // 检查 ai_prompt 列是否存在
    let has_ai_prompt: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('sql_categories') WHERE name='ai_prompt'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if has_ai_prompt == 0 {
        conn.execute("ALTER TABLE sql_categories ADD COLUMN ai_prompt TEXT", [])?;
        conn.execute("ALTER TABLE sql_categories ADD COLUMN icon TEXT", [])?;
        conn.execute(
            "ALTER TABLE sql_categories ADD COLUMN is_system INTEGER DEFAULT 0",
            [],
        )?;
        println!("✓ 已添加 ai_prompt, icon, is_system 列到 sql_categories 表");

        // 更新现有的默认分类，添加 AI 提示词
        conn.execute(
            "UPDATE sql_categories SET ai_prompt = '识别规则：表名和字段名使用小写字母和下划线命名（snake_case）', is_system = 1 WHERE name = 'MySQL'",
            [],
        )?;
    }

    Ok(())
}

/// 迁移 tasks 表：添加 quadrant 字段（四象限）
fn migrate_tasks_add_quadrant(conn: &Connection) -> Result<()> {
    // 检查 quadrant 列是否存在
    let has_quadrant: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('tasks') WHERE name='quadrant'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if has_quadrant == 0 {
        // 添加 quadrant 列
        conn.execute(
            "ALTER TABLE tasks ADD COLUMN quadrant TEXT DEFAULT 'urgent_not_important'",
            [],
        )?;

        // 为所有现有任务设置默认值
        conn.execute(
            "UPDATE tasks SET quadrant = 'urgent_not_important' WHERE quadrant IS NULL",
            [],
        )?;

        println!("✓ 已添加 quadrant 列到 tasks 表，默认值为 'urgent_not_important'");

        // 创建索引以提高查询性能
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_quadrant ON tasks(quadrant)",
            [],
        )?;
    }

    Ok(())
}

/// 创建 AI 全局配置表
fn create_ai_config_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_config (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider TEXT NOT NULL,
            api_key TEXT NOT NULL,
            base_url TEXT,
            model TEXT,
            enabled INTEGER DEFAULT 1,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 确保只有一条配置记录的唯一索引
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_config_single ON ai_config(id) WHERE id = 1",
        [],
    )?;

    Ok(())
}

/// 创建 AI 调用日志表
fn create_ai_logs_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            module TEXT NOT NULL,
            action TEXT NOT NULL,
            provider TEXT NOT NULL,
            model TEXT,
            prompt TEXT NOT NULL,
            response TEXT,
            tokens_used INTEGER,
            duration_ms INTEGER,
            status TEXT NOT NULL DEFAULT 'success',
            error_message TEXT,
            created_at INTEGER NOT NULL
        )",
        [],
    )?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ai_logs_module ON ai_logs(module)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ai_logs_created_at ON ai_logs(created_at)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ai_logs_status ON ai_logs(status)",
        [],
    )?;

    Ok(())
}

/// 迁移 apps 表:添加 item_type 字段
fn migrate_apps_add_item_type(conn: &Connection) -> Result<()> {
    // 检查字段是否已存在
    let mut stmt = conn.prepare("PRAGMA table_info(apps)")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !columns.contains(&"item_type".to_string()) {
        // 添加 item_type 字段,默认为 'Application'
        conn.execute(
            "ALTER TABLE apps ADD COLUMN item_type TEXT DEFAULT 'Application'",
            [],
        )?;
        println!("Added item_type column to apps table");
    }

    Ok(())
}

/// 创建应用启动器设置表
fn create_app_launcher_settings_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_launcher_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            allowed_extensions TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;

    // 插入默认设置(如果不存在)
    conn.execute(
        "INSERT OR IGNORE INTO app_launcher_settings (id, allowed_extensions, created_at, updated_at)
         VALUES (1, '[\"exe\",\"lnk\"]', strftime('%s', 'now'), strftime('%s', 'now'))",
        [],
    )?;

    Ok(())
}

/// 迁移 tasks 表：添加 due_date 和 registered_at 字段
fn migrate_tasks_add_date_fields(conn: &Connection) -> Result<()> {
    // 检查 due_date 列是否存在
    let has_due_date: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('tasks') WHERE name='due_date'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if has_due_date == 0 {
        conn.execute("ALTER TABLE tasks ADD COLUMN due_date TEXT", [])?;
        println!("✓ 已添加 due_date 列到 tasks 表");
    }

    // 检查 registered_at 列是否存在
    let has_registered_at: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('tasks') WHERE name='registered_at'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if has_registered_at == 0 {
        conn.execute("ALTER TABLE tasks ADD COLUMN registered_at TEXT", [])?;
        println!("✓ 已添加 registered_at 列到 tasks 表");
    }

    Ok(())
}

/// 迁移 tasks 表：添加 display_date 字段（日历显示日期）
fn migrate_tasks_add_current_date(conn: &Connection) -> Result<()> {
    // 检查 display_date 列是否存在
    let has_display_date: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('tasks') WHERE name='display_date'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if has_display_date == 0 {
        conn.execute("ALTER TABLE tasks ADD COLUMN display_date TEXT", [])?;
        println!("✓ 已添加 display_date 列到 tasks 表");

        // 创建索引以优化日期查询
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_display_date ON tasks(display_date)",
            [],
        )?;
    }

    Ok(())
}

/// 创建周计划表
fn create_weekly_plans_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS weekly_plans (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            week_key TEXT NOT NULL UNIQUE,
            content TEXT NOT NULL DEFAULT '',
            task_ids TEXT NOT NULL DEFAULT '[]',
            original_task_ids TEXT NOT NULL DEFAULT '[]',
            status TEXT NOT NULL DEFAULT 'draft',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

/// 创建周计划表索引
fn create_weekly_plans_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_weekly_plans_week_key ON weekly_plans(week_key)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_weekly_plans_status ON weekly_plans(status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_weekly_plans_created_at ON weekly_plans(created_at DESC)",
        [],
    )?;

    Ok(())
}

/// 创建剪切板历史表
fn create_clipboard_history_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS clipboard_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content_type TEXT NOT NULL,
            content TEXT NOT NULL,
            preview TEXT,
            image_path TEXT,
            source_app TEXT,
            is_pinned INTEGER DEFAULT 0,
            created_at TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

/// 创建剪切板历史表索引
fn create_clipboard_history_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_clipboard_history_created_at ON clipboard_history(created_at DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_clipboard_history_pinned ON clipboard_history(is_pinned)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_clipboard_history_content_type ON clipboard_history(content_type)",
        [],
    )?;

    Ok(())
}

/// 创建 screen_contexts 表（屏幕上下文）
fn create_screen_contexts_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS screen_contexts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            captured_at TEXT NOT NULL,
            app_name TEXT,
            window_title TEXT,
            activity_type TEXT NOT NULL,
            description TEXT NOT NULL,
            key_content TEXT,
            screenshot_hash TEXT,
            screenshot_path TEXT,
            processing_time_ms INTEGER
        )",
        [],
    )?;
    Ok(())
}

/// 创建 screen_contexts 表索引
fn create_screen_contexts_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_screen_contexts_captured_at ON screen_contexts(captured_at DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_screen_contexts_app_name ON screen_contexts(app_name)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_screen_contexts_activity_type ON screen_contexts(activity_type)",
        [],
    )?;

    Ok(())
}

/// 创建 VLM 配置表
fn create_vlm_config_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS vlm_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            provider TEXT NOT NULL,
            api_key TEXT NOT NULL,
            base_url TEXT,
            model TEXT,
            enabled INTEGER DEFAULT 1,
            max_image_size INTEGER DEFAULT 10240,
            image_quality INTEGER DEFAULT 80,
            timeout INTEGER DEFAULT 30,
            created_at TEXT DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT DEFAULT (datetime('now', 'localtime'))
        )",
        [],
    )?;
    Ok(())
}

/// 创建 daily_summaries 表（每日摘要）
fn create_daily_summaries_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS daily_summaries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            summary_date TEXT NOT NULL UNIQUE,
            total_contexts INTEGER NOT NULL,
            app_stats TEXT NOT NULL,
            activity_timeline TEXT NOT NULL,
            ai_summary TEXT,
            created_at TEXT DEFAULT (datetime('now', 'localtime'))
        )",
        [],
    )?;
    Ok(())
}

/// 迁移 work_logs 表：添加 context_ids 字段
fn migrate_work_logs_add_context_ids(conn: &Connection) -> Result<()> {
    // 检查 context_ids 列是否存在
    let has_context_ids: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('work_logs') WHERE name='context_ids'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if has_context_ids == 0 {
        conn.execute("ALTER TABLE work_logs ADD COLUMN context_ids TEXT", [])?;
        println!("✓ 已添加 context_ids 列到 work_logs 表");
    }

    Ok(())
}

/// 创建 notifications 表（通知记录）
fn create_notifications_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            notification_type TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            is_read INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now', 'localtime'))
        )",
        [],
    )?;
    Ok(())
}

/// 创建 notifications 表索引
fn create_notifications_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_notifications_type ON notifications(notification_type)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_notifications_is_read ON notifications(is_read)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at DESC)",
        [],
    )?;

    Ok(())
}

/// 创建 notification_settings 表（通知设置）
fn create_notification_settings_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notification_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            tips_enabled INTEGER DEFAULT 1,
            tips_interval_minutes INTEGER DEFAULT 60,
            tips_max_per_day INTEGER DEFAULT 5,
            daily_report_enabled INTEGER DEFAULT 1,
            daily_report_time TEXT DEFAULT '18:00',
            weekly_report_enabled INTEGER DEFAULT 1,
            weekly_report_day INTEGER DEFAULT 0,
            weekly_report_time TEXT DEFAULT '20:00',
            updated_at TEXT DEFAULT (datetime('now', 'localtime'))
        )",
        [],
    )?;

    // 插入默认设置(如果不存在)
    conn.execute(
        "INSERT OR IGNORE INTO notification_settings (id) VALUES (1)",
        [],
    )?;

    Ok(())
}

/// 创建 activity_summaries 表（活动总结）
fn create_activity_summaries_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS activity_summaries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            summary_text TEXT NOT NULL,
            activity_type TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now', 'localtime'))
        )",
        [],
    )?;
    Ok(())
}

/// 创建 activity_summaries 表索引
fn create_activity_summaries_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_activity_summaries_start_time ON activity_summaries(start_time DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_activity_summaries_activity_type ON activity_summaries(activity_type)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_activity_summaries_created_at ON activity_summaries(created_at DESC)",
        [],
    )?;

    Ok(())
}

/// 创建 tips 表（智能提示）
fn create_tips_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tips (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            category TEXT NOT NULL,
            priority TEXT NOT NULL,
            is_read INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now', 'localtime'))
        )",
        [],
    )?;
    Ok(())
}

/// 创建 tips 表索引
fn create_tips_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tips_category ON tips(category)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tips_priority ON tips(priority)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tips_is_read ON tips(is_read)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tips_created_at ON tips(created_at DESC)",
        [],
    )?;

    Ok(())
}
