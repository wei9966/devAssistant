use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct DbConnection(pub Mutex<Connection>);

/// 初始化数据库连接
pub fn init_database() -> Result<Connection> {
    let db_path = get_db_path();

    // 确保目录存在
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let conn = Connection::open(db_path)?;

    // 执行迁移
    crate::db::migrations::run_migrations(&conn)?;

    Ok(conn)
}

/// 获取数据库文件路径
fn get_db_path() -> PathBuf {
    let app_dir = dirs::data_local_dir()
        .expect("无法获取应用数据目录")
        .join("dev-assistant");

    app_dir.join("dev_assistant.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_db_path() {
        let path = get_db_path();
        assert!(path.to_str().unwrap().contains("dev-assistant"));
        assert!(path.to_str().unwrap().ends_with("dev_assistant.db"));
    }

    #[test]
    fn test_init_database() {
        // 测试数据库初始化
        let result = init_database();
        assert!(result.is_ok());

        // 验证数据库文件已创建
        let db_path = get_db_path();
        assert!(db_path.exists());
    }
}
