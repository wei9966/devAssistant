use rusqlite::Connection;

fn main() {
    println!("=== 测试数据库迁移 ===\n");
    
    // 创建内存数据库
    let conn = Connection::open_in_memory().unwrap();
    
    // 运行迁移
    println!("1. 运行数据库迁移...");
    dev_assistant_lib::db::migrations::run_migrations(&conn).unwrap();
    println!("   ✓ 迁移成功\n");
    
    // 检查 tags 表
    println!("2. 检查 tags 表...");
    let tags_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tags'",
        [],
        |row| row.get(0),
    ).unwrap();
    println!("   tags 表存在: {}", tags_exists == 1);
    
    // 检查 task_tags 表
    println!("3. 检查 task_tags 表...");
    let task_tags_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='task_tags'",
        [],
        |row| row.get(0),
    ).unwrap();
    println!("   task_tags 表存在: {}", task_tags_exists == 1);
    
    // 检查 tasks 表的 quadrant 字段
    println!("4. 检查 tasks 表的 quadrant 字段...");
    let quadrant_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('tasks') WHERE name='quadrant'",
        [],
        |row| row.get(0),
    ).unwrap();
    println!("   quadrant 字段存在: {}", quadrant_exists == 1);
    
    // 测试插入标签
    println!("\n5. 测试插入标签...");
    conn.execute(
        "INSERT INTO tags (name, color) VALUES (?, ?)",
        ["测试标签", "#FF5733"],
    ).unwrap();
    let tag_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tags",
        [],
        |row| row.get(0),
    ).unwrap();
    println!("   ✓ 插入成功，标签数量: {}", tag_count);
    
    // 测试插入任务（带quadrant）
    println!("6. 测试插入任务...");
    conn.execute(
        "INSERT INTO tasks (title, description, category, priority, status, quadrant) 
         VALUES (?, ?, ?, ?, ?, ?)",
        ["测试任务", "这是一个测试", "dev", "1", "todo", "urgent_important"],
    ).unwrap();
    
    let quadrant: String = conn.query_row(
        "SELECT quadrant FROM tasks WHERE title = ?",
        ["测试任务"],
        |row| row.get(0),
    ).unwrap();
    println!("   ✓ 任务插入成功，四象限值: {}", quadrant);
    
    println!("\n=== 所有测试通过！ ===");
}
