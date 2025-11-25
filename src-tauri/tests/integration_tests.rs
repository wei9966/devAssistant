// Integration tests for DevAssistant
// 测试数据库初始化、任务CRUD功能、SQL监听功能

use rusqlite::Connection;

mod db_tests {
    use super::*;

    #[test]
    fn test_database_initialization() {
        // 测试数据库能够成功创建
        let conn = Connection::open_in_memory().unwrap();

        // 执行迁移
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 验证所有表都已创建
        let tables = vec!["tasks", "sql_history", "work_logs", "git_commits"];

        for table_name in tables {
            let count: i64 = conn
                .query_row(
                    &format!(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}'",
                        table_name
                    ),
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "Table {} should exist", table_name);
        }
    }

    #[test]
    fn test_database_indexes() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 验证 tasks 表索引
        let index_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='tasks'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            index_count >= 4,
            "Tasks table should have at least 4 indexes"
        );

        // 验证 sql_history 表索引
        let index_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='sql_history'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            index_count >= 2,
            "SQL history table should have at least 2 indexes"
        );
    }
}

mod task_crud_tests {
    use super::*;
    use dev_assistant_lib::models::task::{TaskCategory, TaskPriority, TaskStatus};
    use dev_assistant_lib::services::task_service::TaskService;

    #[test]
    fn test_create_task() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 创建任务
        let task_id = TaskService::create_task(
            &conn,
            "测试任务",
            Some("这是一个测试任务的描述"),
            TaskCategory::Dev,
            TaskPriority::High,
        )
        .unwrap();

        assert!(task_id > 0);

        // 验证任务已创建
        let tasks = TaskService::get_all_tasks(&conn).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "测试任务");
        assert_eq!(
            tasks[0].description,
            Some("这是一个测试任务的描述".to_string())
        );
    }

    #[test]
    fn test_start_and_pause_task() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 创建任务
        let task_id = TaskService::create_task(
            &conn,
            "启动测试任务",
            None,
            TaskCategory::Dev,
            TaskPriority::Medium,
        )
        .unwrap();

        // 启动任务
        TaskService::start_task(&conn, task_id).unwrap();

        // 验证任务状态为 active
        let tasks = TaskService::get_all_tasks(&conn).unwrap();
        assert_eq!(tasks[0].status, TaskStatus::Active);
        assert!(tasks[0].started_at.is_some());

        // 暂停任务
        TaskService::pause_task(&conn, task_id, None).unwrap();

        // 验证任务状态为 todo
        let tasks = TaskService::get_all_tasks(&conn).unwrap();
        assert_eq!(tasks[0].status, TaskStatus::Todo);
    }

    #[test]
    fn test_complete_task() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 创建并完成任务
        let task_id = TaskService::create_task(
            &conn,
            "完成测试任务",
            None,
            TaskCategory::Dev,
            TaskPriority::Low,
        )
        .unwrap();

        TaskService::complete_task(&conn, task_id).unwrap();

        // 验证任务已完成
        let completed_tasks = TaskService::get_completed_tasks(&conn, 7).unwrap();
        assert_eq!(completed_tasks.len(), 1);
        assert_eq!(completed_tasks[0].status, TaskStatus::Done);
        assert!(completed_tasks[0].completed_at.is_some());
    }

    #[test]
    fn test_update_task() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 创建任务
        let task_id = TaskService::create_task(
            &conn,
            "原始标题",
            None,
            TaskCategory::Dev,
            TaskPriority::Low,
        )
        .unwrap();

        // 更新任务
        TaskService::update_task(
            &conn,
            task_id,
            Some("更新后的标题"),
            Some("新增的描述"),
            Some(TaskCategory::Study),
            Some(TaskPriority::High),
            Some("feature/test-branch"),
            Some("测试备注"),
            None, // quadrant
        )
        .unwrap();

        // 验证更新
        let tasks = TaskService::get_all_tasks(&conn).unwrap();
        assert_eq!(tasks[0].title, "更新后的标题");
        assert_eq!(tasks[0].description, Some("新增的描述".to_string()));
        assert_eq!(tasks[0].category, TaskCategory::Study);
        assert_eq!(tasks[0].priority, TaskPriority::High);
        assert_eq!(tasks[0].git_branch, Some("feature/test-branch".to_string()));
        assert_eq!(tasks[0].notes, Some("测试备注".to_string()));
    }

    #[test]
    fn test_delete_task() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 创建任务
        let task_id = TaskService::create_task(
            &conn,
            "待删除的任务",
            None,
            TaskCategory::Dev,
            TaskPriority::Medium,
        )
        .unwrap();

        // 验证任务已创建
        let tasks_before = TaskService::get_all_tasks(&conn).unwrap();
        assert_eq!(tasks_before.len(), 1);

        // 删除任务
        TaskService::delete_task(&conn, task_id).unwrap();

        // 验证任务已删除
        let tasks_after = TaskService::get_all_tasks(&conn).unwrap();
        assert_eq!(tasks_after.len(), 0);
    }

    #[test]
    fn test_get_stale_tasks() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 创建一个任务
        TaskService::create_task(
            &conn,
            "新任务",
            None,
            TaskCategory::Dev,
            TaskPriority::Medium,
        )
        .unwrap();

        // 获取超过7天未处理的任务（应该为空，因为刚创建）
        let stale_tasks = TaskService::get_stale_tasks(&conn, 7).unwrap();
        assert_eq!(stale_tasks.len(), 0);

        // 手动插入一个旧任务
        conn.execute(
            "INSERT INTO tasks (title, status, created_at)
             VALUES ('旧任务', 'todo', datetime('now', '-10 days'))",
            [],
        )
        .unwrap();

        // 现在应该能获取到旧任务
        let stale_tasks = TaskService::get_stale_tasks(&conn, 7).unwrap();
        assert_eq!(stale_tasks.len(), 1);
        assert_eq!(stale_tasks[0].title, "旧任务");
    }

    #[test]
    fn test_start_task_pauses_other_active_tasks() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 创建两个任务
        let task_id_1 =
            TaskService::create_task(&conn, "任务1", None, TaskCategory::Dev, TaskPriority::High)
                .unwrap();

        let task_id_2 = TaskService::create_task(
            &conn,
            "任务2",
            None,
            TaskCategory::Dev,
            TaskPriority::Medium,
        )
        .unwrap();

        // 启动任务1
        TaskService::start_task(&conn, task_id_1).unwrap();

        let tasks = TaskService::get_all_tasks(&conn).unwrap();
        let task1 = tasks.iter().find(|t| t.id == Some(task_id_1)).unwrap();
        assert_eq!(task1.status, TaskStatus::Active);

        // 启动任务2（应该自动暂停任务1）
        TaskService::start_task(&conn, task_id_2).unwrap();

        let tasks = TaskService::get_all_tasks(&conn).unwrap();
        let task1 = tasks.iter().find(|t| t.id == Some(task_id_1)).unwrap();
        let task2 = tasks.iter().find(|t| t.id == Some(task_id_2)).unwrap();

        assert_eq!(task1.status, TaskStatus::Todo, "Task 1 should be paused");
        assert_eq!(task2.status, TaskStatus::Active, "Task 2 should be active");
    }
}

mod sql_service_tests {
    use super::*;
    use dev_assistant_lib::services::sql_service::SqlService;

    #[test]
    fn test_detect_sql_type() {
        assert_eq!(
            SqlService::detect_sql_type("SELECT * FROM users"),
            Some("SELECT".to_string())
        );
        assert_eq!(
            SqlService::detect_sql_type("  select * from users"),
            Some("SELECT".to_string())
        );
        assert_eq!(
            SqlService::detect_sql_type("INSERT INTO users VALUES (1, 'test')"),
            Some("INSERT".to_string())
        );
        assert_eq!(
            SqlService::detect_sql_type("UPDATE users SET name='test'"),
            Some("UPDATE".to_string())
        );
        assert_eq!(
            SqlService::detect_sql_type("DELETE FROM users"),
            Some("DELETE".to_string())
        );
        assert_eq!(
            SqlService::detect_sql_type("CREATE TABLE users (id INT)"),
            Some("CREATE".to_string())
        );
        assert_eq!(
            SqlService::detect_sql_type("ALTER TABLE users ADD COLUMN email"),
            Some("ALTER".to_string())
        );
        assert_eq!(
            SqlService::detect_sql_type("DROP TABLE users"),
            Some("DROP".to_string())
        );
        assert_eq!(SqlService::detect_sql_type("INVALID SQL"), None);
    }

    #[test]
    fn test_is_valid_sql() {
        assert!(SqlService::is_valid_sql("SELECT * FROM users"));
        assert!(SqlService::is_valid_sql("  SELECT * FROM users"));
        assert!(SqlService::is_valid_sql("INSERT INTO users VALUES (1)"));
        assert!(SqlService::is_valid_sql("UPDATE users SET x=1"));
        assert!(SqlService::is_valid_sql("DELETE FROM users"));
        assert!(!SqlService::is_valid_sql("INVALID STATEMENT"));
        assert!(!SqlService::is_valid_sql("Just some text"));
    }

    #[test]
    fn test_save_sql() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 保存SQL记录
        let sql_id =
            SqlService::save_sql(&conn, "SELECT * FROM users WHERE id = 1", "clipboard").unwrap();

        assert!(sql_id > 0);

        // 验证记录已保存
        let records = SqlService::get_recent_sqls(&conn, 10).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].sql_text, "SELECT * FROM users WHERE id = 1");
        assert_eq!(records[0].sql_type, Some("SELECT".to_string()));
        assert_eq!(records[0].execution_source, Some("clipboard".to_string()));
    }

    #[test]
    fn test_get_recent_sqls() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 保存多条SQL记录
        SqlService::save_sql(&conn, "SELECT * FROM table1", "clipboard").unwrap();
        SqlService::save_sql(&conn, "INSERT INTO table2 VALUES (1)", "clipboard").unwrap();
        SqlService::save_sql(&conn, "UPDATE table3 SET x=1", "clipboard").unwrap();

        // 获取所有记录
        let all_records = SqlService::get_recent_sqls(&conn, 10).unwrap();
        assert_eq!(all_records.len(), 3);

        // 获取最近的2条记录
        let records = SqlService::get_recent_sqls(&conn, 2).unwrap();
        assert_eq!(records.len(), 2);

        // 验证所有SQL都保存了
        let sql_texts: Vec<String> = all_records.iter().map(|r| r.sql_text.clone()).collect();
        assert!(sql_texts.contains(&"SELECT * FROM table1".to_string()));
        assert!(sql_texts.contains(&"INSERT INTO table2 VALUES (1)".to_string()));
        assert!(sql_texts.contains(&"UPDATE table3 SET x=1".to_string()));
    }

    #[test]
    fn test_toggle_favorite() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 保存SQL记录
        let sql_id = SqlService::save_sql(&conn, "SELECT * FROM favorites", "clipboard").unwrap();

        // 初始状态不是收藏
        let records = SqlService::get_favorite_sqls(&conn).unwrap();
        assert_eq!(records.len(), 0);

        // 标记为收藏
        SqlService::toggle_favorite(&conn, sql_id).unwrap();

        // 验证已收藏
        let records = SqlService::get_favorite_sqls(&conn).unwrap();
        assert_eq!(records.len(), 1);
        assert!(records[0].is_favorite);

        // 再次切换（取消收藏）
        SqlService::toggle_favorite(&conn, sql_id).unwrap();

        // 验证已取消收藏
        let records = SqlService::get_favorite_sqls(&conn).unwrap();
        assert_eq!(records.len(), 0);
    }

    #[test]
    fn test_delete_sql() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 保存SQL记录
        let sql_id = SqlService::save_sql(&conn, "SELECT * FROM to_delete", "clipboard").unwrap();

        // 验证记录存在
        let records_before = SqlService::get_recent_sqls(&conn, 10).unwrap();
        assert_eq!(records_before.len(), 1);

        // 删除记录
        SqlService::delete_sql(&conn, sql_id).unwrap();

        // 验证记录已删除
        let records_after = SqlService::get_recent_sqls(&conn, 10).unwrap();
        assert_eq!(records_after.len(), 0);
    }

    #[test]
    fn test_get_favorite_sqls() {
        let conn = Connection::open_in_memory().unwrap();
        dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();

        // 保存多条SQL，部分标记为收藏
        let sql_id_1 = SqlService::save_sql(&conn, "SELECT 1", "clipboard").unwrap();
        let _sql_id_2 = SqlService::save_sql(&conn, "SELECT 2", "clipboard").unwrap();
        let sql_id_3 = SqlService::save_sql(&conn, "SELECT 3", "clipboard").unwrap();

        SqlService::toggle_favorite(&conn, sql_id_1).unwrap();
        SqlService::toggle_favorite(&conn, sql_id_3).unwrap();

        // 获取收藏的SQL
        let favorites = SqlService::get_favorite_sqls(&conn).unwrap();
        assert_eq!(favorites.len(), 2);

        // 验证是收藏的记录
        let favorite_texts: Vec<String> = favorites.iter().map(|r| r.sql_text.clone()).collect();
        assert!(favorite_texts.contains(&"SELECT 1".to_string()));
        assert!(favorite_texts.contains(&"SELECT 3".to_string()));
        assert!(!favorite_texts.contains(&"SELECT 2".to_string()));
    }
}
