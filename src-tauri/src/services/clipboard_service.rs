use clipboard::{ClipboardContext, ClipboardProvider};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::services::sql_service::SqlService;

pub struct ClipboardService {
    last_content: Arc<Mutex<String>>,
}

impl ClipboardService {
    pub fn new() -> Self {
        Self {
            last_content: Arc::new(Mutex::new(String::new())),
        }
    }

    /// 启动剪贴板监听（后台线程）
    pub fn start_monitoring(&self, db_path: String) {
        let last_content = Arc::clone(&self.last_content);

        thread::spawn(move || {
            println!("剪贴板监控线程已启动");
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();

            loop {
                thread::sleep(Duration::from_secs(2));

                if let Ok(content) = ctx.get_contents() {
                    let mut last = last_content.lock().unwrap();

                    // 检查是否是新内容
                    if content != *last && !content.is_empty() {
                        // 安全地截取前50个字符（处理UTF-8字符边界）
                        let preview = content.chars().take(50).collect::<String>();
                        println!("检测到新的剪贴板内容 (前50字符): {}", preview);
                        *last = content.clone();

                        // 检查是否是 SQL 语句
                        if SqlService::is_valid_sql(&content) {
                            let preview = content.chars().take(50).collect::<String>();
                            println!("✓ 确认为 SQL 语句: {}", preview);

                            // 保存到数据库
                            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                                match SqlService::save_sql(&conn, &content, "clipboard") {
                                    Ok(id) => println!("✓ SQL已保存到数据库, ID: {}", id),
                                    Err(e) => println!("✗ 保存SQL失败: {}", e),
                                }
                            } else {
                                println!("✗ 无法打开数据库连接");
                            }
                        } else {
                            println!("✗ 不是有效的SQL语句");
                        }
                    }
                }
            }
        });
    }
}
