use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 文件索引记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileIndexRecord {
    /// 文件ID（数据库自增）
    pub id: Option<i64>,
    /// 文件名
    pub name: String,
    /// 文件完整路径
    pub path: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 修改时间（格式化字符串）
    pub modified_time: String,
    /// 是否是目录
    pub is_dir: bool,
    /// 文件类型（扩展名）
    pub file_type: String,
    /// 所属驱动器（如 C:, D: 等）
    pub drive: String,
    /// 索引时间
    pub indexed_at: Option<String>,
}

/// 索引统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStats {
    /// 总文件数
    pub total_files: u64,
    /// 总目录数
    pub total_dirs: u64,
    /// 已索引的驱动器列表
    pub indexed_drives: Vec<String>,
    /// 最后更新时间
    pub last_updated: Option<String>,
}

/// 文件索引服务
///
/// 提供基于 SQLite 的文件索引管理功能，支持快速搜索文件
#[derive(Clone)]
pub struct FileIndexService {
    db_path: PathBuf,
}

impl FileIndexService {
    /// 创建服务实例
    ///
    /// # Arguments
    /// * `db_path` - SQLite 数据库文件路径
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// 获取数据库连接
    fn get_connection(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path)
            .map_err(|e| format!("无法打开数据库: {}", e))
    }

    /// 初始化数据库（创建表）
    pub fn init_db(&self) -> Result<(), String> {
        let conn = self.get_connection()?;

        // 创建文件索引表（添加 name_lower 列用于快速搜索）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_index (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                name_lower TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                size INTEGER NOT NULL DEFAULT 0,
                modified_time TEXT,
                is_dir INTEGER NOT NULL DEFAULT 0,
                file_type TEXT,
                drive TEXT NOT NULL,
                indexed_at TEXT DEFAULT (datetime('now', 'localtime'))
            )",
            [],
        )
        .map_err(|e| format!("创建文件索引表失败: {}", e))?;

        // 创建 FTS5 全文搜索虚拟表（用于快速模糊搜索）
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS file_index_fts USING fts5(
                name,
                path,
                content='file_index',
                content_rowid='id',
                tokenize='unicode61 remove_diacritics 1'
            )",
            [],
        )
        .map_err(|e| format!("创建 FTS5 索引失败: {}", e))?;

        // 创建触发器以保持 FTS 索引同步
        conn.execute_batch(
            "CREATE TRIGGER IF NOT EXISTS file_index_ai AFTER INSERT ON file_index BEGIN
                INSERT INTO file_index_fts(rowid, name, path) VALUES (new.id, new.name, new.path);
            END;
            CREATE TRIGGER IF NOT EXISTS file_index_ad AFTER DELETE ON file_index BEGIN
                INSERT INTO file_index_fts(file_index_fts, rowid, name, path) VALUES('delete', old.id, old.name, old.path);
            END;
            CREATE TRIGGER IF NOT EXISTS file_index_au AFTER UPDATE ON file_index BEGIN
                INSERT INTO file_index_fts(file_index_fts, rowid, name, path) VALUES('delete', old.id, old.name, old.path);
                INSERT INTO file_index_fts(rowid, name, path) VALUES (new.id, new.name, new.path);
            END;"
        )
        .map_err(|e| format!("创建 FTS 触发器失败: {}", e))?;

        // 创建索引以加速搜索
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_index_name_lower ON file_index(name_lower)",
            [],
        )
        .map_err(|e| format!("创建名称索引失败: {}", e))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_index_drive ON file_index(drive)",
            [],
        )
        .map_err(|e| format!("创建驱动器索引失败: {}", e))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_index_modified ON file_index(modified_time DESC)",
            [],
        )
        .map_err(|e| format!("创建修改时间索引失败: {}", e))?;

        // 创建索引元信息表（记录索引状态）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_index_meta (
                drive TEXT PRIMARY KEY,
                last_indexed_at TEXT,
                file_count INTEGER DEFAULT 0,
                dir_count INTEGER DEFAULT 0
            )",
            [],
        )
        .map_err(|e| format!("创建索引元信息表失败: {}", e))?;

        Ok(())
    }

    /// 批量插入文件记录（用于首次索引）
    ///
    /// 使用事务提高批量插入性能
    ///
    /// # Arguments
    /// * `files` - 要插入的文件记录列表
    ///
    /// # Returns
    /// 成功插入的记录数量
    pub fn batch_insert(&self, files: Vec<FileIndexRecord>) -> Result<usize, String> {
        let mut conn = self.get_connection()?;

        let tx = conn.transaction()
            .map_err(|e| format!("开始事务失败: {}", e))?;

        let mut inserted_count = 0;

        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO file_index (name, name_lower, path, size, modified_time, is_dir, file_type, drive, indexed_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now', 'localtime'))"
            ).map_err(|e| format!("准备插入语句失败: {}", e))?;

            for file in &files {
                let name_lower = file.name.to_lowercase();
                match stmt.execute(params![
                    file.name,
                    name_lower,
                    file.path,
                    file.size as i64,
                    file.modified_time,
                    file.is_dir as i32,
                    file.file_type,
                    file.drive
                ]) {
                    Ok(_) => inserted_count += 1,
                    Err(e) => {
                        log::warn!("插入文件记录失败 {}: {}", file.path, e);
                    }
                }
            }
        }

        tx.commit()
            .map_err(|e| format!("提交事务失败: {}", e))?;

        Ok(inserted_count)
    }

    /// 启用高性能写入模式（用于批量索引前调用）
    pub fn enable_fast_write_mode(&self) -> Result<(), String> {
        let conn = self.get_connection()?;
        conn.execute_batch(
            "PRAGMA synchronous = OFF;
             PRAGMA journal_mode = MEMORY;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -64000;"
        ).map_err(|e| format!("设置高性能模式失败: {}", e))?;
        Ok(())
    }

    /// 恢复正常写入模式（批量索引后调用）
    pub fn disable_fast_write_mode(&self) -> Result<(), String> {
        let conn = self.get_connection()?;
        conn.execute_batch(
            "PRAGMA synchronous = NORMAL;
             PRAGMA journal_mode = WAL;"
        ).map_err(|e| format!("恢复正常模式失败: {}", e))?;
        Ok(())
    }

    /// 高性能批量插入（使用多值 INSERT 语法）
    ///
    /// 这个方法在同一个数据库连接上：
    /// 1. 设置高性能 PRAGMA
    /// 2. 使用多值 INSERT 语法批量插入（一条 SQL 插入多行）
    ///
    /// # Arguments
    /// * `files` - 要插入的文件记录列表
    ///
    /// # Returns
    /// 成功插入的记录数量
    pub fn batch_insert_optimized(&self, files: Vec<FileIndexRecord>) -> Result<usize, String> {
        if files.is_empty() {
            return Ok(0);
        }

        let mut conn = self.get_connection()?;

        // 在同一连接上设置高性能 PRAGMA
        conn.execute_batch(
            "PRAGMA synchronous = OFF;
             PRAGMA journal_mode = MEMORY;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -64000;
             PRAGMA locking_mode = EXCLUSIVE;"
        ).map_err(|e| format!("设置高性能模式失败: {}", e))?;

        let tx = conn.transaction()
            .map_err(|e| format!("开始事务失败: {}", e))?;

        let mut inserted_count = 0;

        // 使用多值 INSERT 语法，每次插入 500 行
        // SQLite 对 INSERT 语句有参数数量限制（默认 999），每行 8 个参数，500 * 8 = 4000 > 999
        // 所以我们使用 100 行每批，100 * 8 = 800 < 999
        const MULTI_INSERT_BATCH: usize = 100;

        for chunk in files.chunks(MULTI_INSERT_BATCH) {
            // 构建多值 INSERT 语句
            let placeholders: Vec<String> = chunk.iter().map(|_| {
                "(?, ?, ?, ?, ?, ?, ?, ?, datetime('now', 'localtime'))".to_string()
            }).collect();

            let sql = format!(
                "INSERT OR REPLACE INTO file_index (name, name_lower, path, size, modified_time, is_dir, file_type, drive, indexed_at) VALUES {}",
                placeholders.join(", ")
            );

            // 收集所有参数
            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::with_capacity(chunk.len() * 8);
            for file in chunk {
                let name_lower = file.name.to_lowercase();
                params_vec.push(Box::new(file.name.clone()));
                params_vec.push(Box::new(name_lower));
                params_vec.push(Box::new(file.path.clone()));
                params_vec.push(Box::new(file.size as i64));
                params_vec.push(Box::new(file.modified_time.clone()));
                params_vec.push(Box::new(file.is_dir as i32));
                params_vec.push(Box::new(file.file_type.clone()));
                params_vec.push(Box::new(file.drive.clone()));
            }

            // 转换为引用切片
            let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

            match tx.execute(&sql, params_refs.as_slice()) {
                Ok(count) => inserted_count += count,
                Err(e) => {
                    log::warn!("批量插入失败: {}", e);
                    // 回退到单条插入
                    let mut stmt = tx.prepare_cached(
                        "INSERT OR REPLACE INTO file_index (name, name_lower, path, size, modified_time, is_dir, file_type, drive, indexed_at)
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now', 'localtime'))"
                    ).map_err(|e| format!("准备插入语句失败: {}", e))?;

                    for file in chunk {
                        let name_lower = file.name.to_lowercase();
                        if stmt.execute(params![
                            file.name,
                            name_lower,
                            file.path,
                            file.size as i64,
                            file.modified_time,
                            file.is_dir as i32,
                            file.file_type,
                            file.drive
                        ]).is_ok() {
                            inserted_count += 1;
                        }
                    }
                }
            }
        }

        tx.commit()
            .map_err(|e| format!("提交事务失败: {}", e))?;

        Ok(inserted_count)
    }

    /// 重建 FTS 索引（在清空后重建索引时调用）
    pub fn rebuild_fts_index(&self) -> Result<(), String> {
        let conn = self.get_connection()?;
        conn.execute("INSERT INTO file_index_fts(file_index_fts) VALUES('rebuild')", [])
            .map_err(|e| format!("重建 FTS 索引失败: {}", e))?;
        Ok(())
    }

    /// 禁用 FTS 触发器（批量插入前调用，避免每条记录都触发 FTS 更新）
    pub fn disable_fts_triggers(&self) -> Result<(), String> {
        let conn = self.get_connection()?;
        conn.execute_batch(
            "DROP TRIGGER IF EXISTS file_index_ai;
             DROP TRIGGER IF EXISTS file_index_ad;
             DROP TRIGGER IF EXISTS file_index_au;"
        ).map_err(|e| format!("禁用 FTS 触发器失败: {}", e))?;
        log::info!("已禁用 FTS 触发器以加速批量插入");
        Ok(())
    }

    /// 重新启用 FTS 触发器（批量插入后调用）
    pub fn enable_fts_triggers(&self) -> Result<(), String> {
        let conn = self.get_connection()?;
        conn.execute_batch(
            "CREATE TRIGGER IF NOT EXISTS file_index_ai AFTER INSERT ON file_index BEGIN
                INSERT INTO file_index_fts(rowid, name, path) VALUES (new.id, new.name, new.path);
            END;
            CREATE TRIGGER IF NOT EXISTS file_index_ad AFTER DELETE ON file_index BEGIN
                INSERT INTO file_index_fts(file_index_fts, rowid, name, path) VALUES('delete', old.id, old.name, old.path);
            END;
            CREATE TRIGGER IF NOT EXISTS file_index_au AFTER UPDATE ON file_index BEGIN
                INSERT INTO file_index_fts(file_index_fts, rowid, name, path) VALUES('delete', old.id, old.name, old.path);
                INSERT INTO file_index_fts(rowid, name, path) VALUES (new.id, new.name, new.path);
            END;"
        ).map_err(|e| format!("启用 FTS 触发器失败: {}", e))?;
        log::info!("已重新启用 FTS 触发器");
        Ok(())
    }

    /// 从主表重建完整的 FTS 索引（批量插入后调用）
    ///
    /// 对于外部内容 FTS5 表，使用 'rebuild' 命令重建索引
    pub fn rebuild_fts_from_main_table(&self) -> Result<(), String> {
        let conn = self.get_connection()?;

        log::info!("开始重建 FTS 索引...");

        // 获取总记录数用于日志
        let total_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM file_index",
            [],
            |row| row.get(0)
        ).map_err(|e| format!("获取记录数失败: {}", e))?;

        log::info!("FTS 索引需要处理 {} 条记录", total_count);

        if total_count == 0 {
            log::info!("没有记录需要索引");
            return Ok(());
        }

        // 对于外部内容 FTS5 表，使用 'rebuild' 命令重建整个索引
        // 这是 SQLite FTS5 文档推荐的方式
        conn.execute(
            "INSERT INTO file_index_fts(file_index_fts) VALUES('rebuild')",
            []
        ).map_err(|e| format!("重建 FTS 索引失败: {}", e))?;

        log::info!("FTS 索引重建完成，共 {} 条记录", total_count);
        Ok(())
    }

    /// 搜索文件（使用 FTS5 全文搜索，毫秒级响应）
    ///
    /// # Arguments
    /// * `keyword` - 搜索关键词
    /// * `max_results` - 最大结果数量
    ///
    /// # Returns
    /// 匹配的文件记录列表，按相关性排序
    pub fn search(&self, keyword: &str, max_results: usize) -> Result<Vec<FileIndexRecord>, String> {
        let conn = self.get_connection()?;

        // 为 FTS5 构建搜索词（支持前缀匹配）
        let fts_keyword = format!("\"{}\"*", keyword.replace("\"", "\"\""));

        // 使用 FTS5 搜索，JOIN 回主表获取完整信息
        let mut stmt = conn.prepare(
            "SELECT f.id, f.name, f.path, f.size, f.modified_time, f.is_dir, f.file_type, f.drive, f.indexed_at
             FROM file_index f
             INNER JOIN file_index_fts fts ON f.id = fts.rowid
             WHERE file_index_fts MATCH ?
             ORDER BY rank
             LIMIT ?"
        ).map_err(|e| format!("准备 FTS 搜索语句失败: {}", e))?;

        let records = stmt.query_map(params![fts_keyword, max_results as i64], |row| {
            Ok(FileIndexRecord {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                path: row.get(2)?,
                size: row.get::<_, i64>(3)? as u64,
                modified_time: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                is_dir: row.get::<_, i32>(5)? != 0,
                file_type: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                drive: row.get(7)?,
                indexed_at: row.get(8)?,
            })
        });

        match records {
            Ok(iter) => iter.collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("读取搜索结果失败: {}", e)),
            Err(_) => {
                // FTS 搜索失败，回退到传统 LIKE 搜索（使用 name_lower 列）
                self.search_fallback(keyword, max_results)
            }
        }
    }

    /// 回退搜索方法（当 FTS 不可用时使用）
    fn search_fallback(&self, keyword: &str, max_results: usize) -> Result<Vec<FileIndexRecord>, String> {
        let conn = self.get_connection()?;
        let search_pattern = format!("%{}%", keyword.to_lowercase());

        let mut stmt = conn.prepare(
            "SELECT id, name, path, size, modified_time, is_dir, file_type, drive, indexed_at
             FROM file_index
             WHERE name_lower LIKE ?
             ORDER BY modified_time DESC
             LIMIT ?"
        ).map_err(|e| format!("准备搜索语句失败: {}", e))?;

        let records = stmt.query_map(params![search_pattern, max_results as i64], |row| {
            Ok(FileIndexRecord {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                path: row.get(2)?,
                size: row.get::<_, i64>(3)? as u64,
                modified_time: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                is_dir: row.get::<_, i32>(5)? != 0,
                file_type: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                drive: row.get(7)?,
                indexed_at: row.get(8)?,
            })
        }).map_err(|e| format!("执行搜索失败: {}", e))?;

        records
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("读取搜索结果失败: {}", e))
    }

    /// 按驱动器搜索（使用 FTS5）
    ///
    /// # Arguments
    /// * `keyword` - 搜索关键词
    /// * `drive` - 驱动器标识（如 "C:", "D:" 等）
    /// * `max_results` - 最大结果数量
    ///
    /// # Returns
    /// 匹配的文件记录列表
    pub fn search_in_drive(&self, keyword: &str, drive: &str, max_results: usize) -> Result<Vec<FileIndexRecord>, String> {
        let conn = self.get_connection()?;

        // 为 FTS5 构建搜索词
        let fts_keyword = format!("\"{}\"*", keyword.replace("\"", "\"\""));

        // 使用 FTS5 搜索并过滤驱动器
        let mut stmt = conn.prepare(
            "SELECT f.id, f.name, f.path, f.size, f.modified_time, f.is_dir, f.file_type, f.drive, f.indexed_at
             FROM file_index f
             INNER JOIN file_index_fts fts ON f.id = fts.rowid
             WHERE file_index_fts MATCH ? AND f.drive = ?
             ORDER BY rank
             LIMIT ?"
        ).map_err(|e| format!("准备 FTS 搜索语句失败: {}", e))?;

        let records = stmt.query_map(params![fts_keyword, drive, max_results as i64], |row| {
            Ok(FileIndexRecord {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                path: row.get(2)?,
                size: row.get::<_, i64>(3)? as u64,
                modified_time: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                is_dir: row.get::<_, i32>(5)? != 0,
                file_type: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                drive: row.get(7)?,
                indexed_at: row.get(8)?,
            })
        });

        match records {
            Ok(iter) => iter.collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("读取搜索结果失败: {}", e)),
            Err(_) => {
                // 回退到传统搜索
                self.search_in_drive_fallback(keyword, drive, max_results)
            }
        }
    }

    /// 回退的驱动器搜索方法
    fn search_in_drive_fallback(&self, keyword: &str, drive: &str, max_results: usize) -> Result<Vec<FileIndexRecord>, String> {
        let conn = self.get_connection()?;
        let search_pattern = format!("%{}%", keyword.to_lowercase());

        let mut stmt = conn.prepare(
            "SELECT id, name, path, size, modified_time, is_dir, file_type, drive, indexed_at
             FROM file_index
             WHERE name_lower LIKE ? AND drive = ?
             ORDER BY modified_time DESC
             LIMIT ?"
        ).map_err(|e| format!("准备搜索语句失败: {}", e))?;

        let records = stmt.query_map(params![search_pattern, drive, max_results as i64], |row| {
            Ok(FileIndexRecord {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                path: row.get(2)?,
                size: row.get::<_, i64>(3)? as u64,
                modified_time: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                is_dir: row.get::<_, i32>(5)? != 0,
                file_type: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                drive: row.get(7)?,
                indexed_at: row.get(8)?,
            })
        }).map_err(|e| format!("执行搜索失败: {}", e))?;

        records
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("读取搜索结果失败: {}", e))
    }

    /// 添加单个文件
    ///
    /// # Arguments
    /// * `file` - 要添加的文件记录
    pub fn add_file(&self, file: FileIndexRecord) -> Result<(), String> {
        let conn = self.get_connection()?;
        let name_lower = file.name.to_lowercase();

        conn.execute(
            "INSERT OR REPLACE INTO file_index (name, name_lower, path, size, modified_time, is_dir, file_type, drive, indexed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now', 'localtime'))",
            params![
                file.name,
                name_lower,
                file.path,
                file.size as i64,
                file.modified_time,
                file.is_dir as i32,
                file.file_type,
                file.drive
            ],
        ).map_err(|e| format!("添加文件记录失败: {}", e))?;

        Ok(())
    }

    /// 删除文件（按路径）
    ///
    /// # Arguments
    /// * `path` - 要删除的文件路径
    pub fn remove_file(&self, path: &str) -> Result<(), String> {
        let conn = self.get_connection()?;

        conn.execute(
            "DELETE FROM file_index WHERE path = ?",
            params![path],
        ).map_err(|e| format!("删除文件记录失败: {}", e))?;

        Ok(())
    }

    /// 更新文件
    ///
    /// # Arguments
    /// * `file` - 更新后的文件记录
    pub fn update_file(&self, file: FileIndexRecord) -> Result<(), String> {
        let conn = self.get_connection()?;
        let name_lower = file.name.to_lowercase();

        conn.execute(
            "UPDATE file_index
             SET name = ?, name_lower = ?, size = ?, modified_time = ?, is_dir = ?, file_type = ?, indexed_at = datetime('now', 'localtime')
             WHERE path = ?",
            params![
                file.name,
                name_lower,
                file.size as i64,
                file.modified_time,
                file.is_dir as i32,
                file.file_type,
                file.path
            ],
        ).map_err(|e| format!("更新文件记录失败: {}", e))?;

        Ok(())
    }

    /// 获取索引统计信息
    ///
    /// # Returns
    /// 索引统计信息，包括总文件数、目录数、已索引驱动器列表等
    pub fn get_stats(&self) -> Result<IndexStats, String> {
        let conn = self.get_connection()?;

        // 获取总文件数
        let total_files: u64 = conn.query_row(
            "SELECT COUNT(*) FROM file_index WHERE is_dir = 0",
            [],
            |row| row.get::<_, i64>(0),
        ).map_err(|e| format!("查询文件数失败: {}", e))? as u64;

        // 获取总目录数
        let total_dirs: u64 = conn.query_row(
            "SELECT COUNT(*) FROM file_index WHERE is_dir = 1",
            [],
            |row| row.get::<_, i64>(0),
        ).map_err(|e| format!("查询目录数失败: {}", e))? as u64;

        // 获取已索引的驱动器列表
        let mut stmt = conn.prepare(
            "SELECT DISTINCT drive FROM file_index ORDER BY drive"
        ).map_err(|e| format!("准备查询驱动器语句失败: {}", e))?;

        let indexed_drives: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("查询驱动器失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        // 获取最后更新时间
        let last_updated: Option<String> = conn.query_row(
            "SELECT MAX(indexed_at) FROM file_index",
            [],
            |row| row.get(0),
        ).unwrap_or(None);

        Ok(IndexStats {
            total_files,
            total_dirs,
            indexed_drives,
            last_updated,
        })
    }

    /// 清空索引
    pub fn clear_index(&self) -> Result<(), String> {
        let conn = self.get_connection()?;

        // 清空主表（触发器会自动同步 FTS 表）
        conn.execute("DELETE FROM file_index", [])
            .map_err(|e| format!("清空索引失败: {}", e))?;

        conn.execute("DELETE FROM file_index_meta", [])
            .map_err(|e| format!("清空索引元信息失败: {}", e))?;

        // 重建 FTS 索引以释放空间
        let _ = conn.execute("INSERT INTO file_index_fts(file_index_fts) VALUES('rebuild')", []);

        Ok(())
    }

    /// 按驱动器清空
    ///
    /// # Arguments
    /// * `drive` - 要清空的驱动器标识（如 "C:", "D:" 等）
    pub fn clear_drive(&self, drive: &str) -> Result<(), String> {
        let conn = self.get_connection()?;

        conn.execute(
            "DELETE FROM file_index WHERE drive = ?",
            params![drive],
        ).map_err(|e| format!("清空驱动器索引失败: {}", e))?;

        conn.execute(
            "DELETE FROM file_index_meta WHERE drive = ?",
            params![drive],
        ).map_err(|e| format!("清空驱动器元信息失败: {}", e))?;

        Ok(())
    }

    /// 更新驱动器索引元信息
    ///
    /// # Arguments
    /// * `drive` - 驱动器标识
    /// * `file_count` - 文件数量
    /// * `dir_count` - 目录数量
    pub fn update_drive_meta(&self, drive: &str, file_count: u64, dir_count: u64) -> Result<(), String> {
        let conn = self.get_connection()?;

        conn.execute(
            "INSERT OR REPLACE INTO file_index_meta (drive, last_indexed_at, file_count, dir_count)
             VALUES (?, datetime('now', 'localtime'), ?, ?)",
            params![drive, file_count as i64, dir_count as i64],
        ).map_err(|e| format!("更新驱动器元信息失败: {}", e))?;

        Ok(())
    }

    /// 获取指定驱动器的索引信息
    ///
    /// # Arguments
    /// * `drive` - 驱动器标识
    ///
    /// # Returns
    /// 返回 (最后索引时间, 文件数, 目录数)
    pub fn get_drive_meta(&self, drive: &str) -> Result<Option<(String, u64, u64)>, String> {
        let conn = self.get_connection()?;

        let result = conn.query_row(
            "SELECT last_indexed_at, file_count, dir_count FROM file_index_meta WHERE drive = ?",
            params![drive],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)? as u64,
                    row.get::<_, i64>(2)? as u64,
                ))
            },
        );

        match result {
            Ok(data) => Ok(Some(data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("查询驱动器元信息失败: {}", e)),
        }
    }

    /// 检查路径是否已索引
    ///
    /// # Arguments
    /// * `path` - 文件路径
    ///
    /// # Returns
    /// 如果已索引返回 true
    pub fn is_path_indexed(&self, path: &str) -> Result<bool, String> {
        let conn = self.get_connection()?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM file_index WHERE path = ?",
            params![path],
            |row| row.get(0),
        ).map_err(|e| format!("检查路径索引失败: {}", e))?;

        Ok(count > 0)
    }

    /// 批量删除文件记录
    ///
    /// # Arguments
    /// * `paths` - 要删除的文件路径列表
    ///
    /// # Returns
    /// 成功删除的记录数量
    pub fn batch_remove(&self, paths: Vec<String>) -> Result<usize, String> {
        let mut conn = self.get_connection()?;

        let tx = conn.transaction()
            .map_err(|e| format!("开始事务失败: {}", e))?;

        let mut removed_count = 0;

        {
            let mut stmt = tx.prepare("DELETE FROM file_index WHERE path = ?")
                .map_err(|e| format!("准备删除语句失败: {}", e))?;

            for path in &paths {
                match stmt.execute(params![path]) {
                    Ok(count) => removed_count += count,
                    Err(e) => {
                        log::warn!("删除文件记录失败 {}: {}", path, e);
                    }
                }
            }
        }

        tx.commit()
            .map_err(|e| format!("提交事务失败: {}", e))?;

        Ok(removed_count)
    }

    /// 按文件类型搜索
    ///
    /// # Arguments
    /// * `file_type` - 文件类型（扩展名）
    /// * `max_results` - 最大结果数量
    ///
    /// # Returns
    /// 匹配的文件记录列表
    pub fn search_by_type(&self, file_type: &str, max_results: usize) -> Result<Vec<FileIndexRecord>, String> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, path, size, modified_time, is_dir, file_type, drive, indexed_at
             FROM file_index
             WHERE LOWER(file_type) = LOWER(?)
             ORDER BY modified_time DESC
             LIMIT ?"
        ).map_err(|e| format!("准备搜索语句失败: {}", e))?;

        let records = stmt.query_map(params![file_type, max_results as i64], |row| {
            Ok(FileIndexRecord {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                path: row.get(2)?,
                size: row.get::<_, i64>(3)? as u64,
                modified_time: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                is_dir: row.get::<_, i32>(5)? != 0,
                file_type: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                drive: row.get(7)?,
                indexed_at: row.get(8)?,
            })
        }).map_err(|e| format!("执行搜索失败: {}", e))?;

        records
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("读取搜索结果失败: {}", e))
    }

    /// 获取最近索引的文件
    ///
    /// # Arguments
    /// * `limit` - 返回数量限制
    ///
    /// # Returns
    /// 最近索引的文件记录列表
    pub fn get_recent_indexed(&self, limit: usize) -> Result<Vec<FileIndexRecord>, String> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, path, size, modified_time, is_dir, file_type, drive, indexed_at
             FROM file_index
             ORDER BY indexed_at DESC
             LIMIT ?"
        ).map_err(|e| format!("准备查询语句失败: {}", e))?;

        let records = stmt.query_map(params![limit as i64], |row| {
            Ok(FileIndexRecord {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                path: row.get(2)?,
                size: row.get::<_, i64>(3)? as u64,
                modified_time: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                is_dir: row.get::<_, i32>(5)? != 0,
                file_type: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                drive: row.get(7)?,
                indexed_at: row.get(8)?,
            })
        }).map_err(|e| format!("执行查询失败: {}", e))?;

        records
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("读取查询结果失败: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_service() -> (FileIndexService, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_file_index.db");
        let service = FileIndexService::new(db_path);
        service.init_db().unwrap();
        (service, dir)
    }

    #[test]
    fn test_init_db() {
        let (service, _dir) = create_test_service();
        // 初始化应该成功
        assert!(service.init_db().is_ok());
    }

    #[test]
    fn test_add_and_search_file() {
        let (service, _dir) = create_test_service();

        let file = FileIndexRecord {
            id: None,
            name: "test_document.txt".to_string(),
            path: "C:\\Users\\test\\test_document.txt".to_string(),
            size: 1024,
            modified_time: "2024-01-01 12:00:00".to_string(),
            is_dir: false,
            file_type: "txt".to_string(),
            drive: "C:".to_string(),
            indexed_at: None,
        };

        service.add_file(file).unwrap();

        let results = service.search("document", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test_document.txt");
    }

    #[test]
    fn test_batch_insert() {
        let (service, _dir) = create_test_service();

        let files = vec![
            FileIndexRecord {
                id: None,
                name: "file1.txt".to_string(),
                path: "C:\\file1.txt".to_string(),
                size: 100,
                modified_time: "2024-01-01 12:00:00".to_string(),
                is_dir: false,
                file_type: "txt".to_string(),
                drive: "C:".to_string(),
                indexed_at: None,
            },
            FileIndexRecord {
                id: None,
                name: "file2.doc".to_string(),
                path: "D:\\file2.doc".to_string(),
                size: 200,
                modified_time: "2024-01-02 12:00:00".to_string(),
                is_dir: false,
                file_type: "doc".to_string(),
                drive: "D:".to_string(),
                indexed_at: None,
            },
        ];

        let count = service.batch_insert(files).unwrap();
        assert_eq!(count, 2);

        let stats = service.get_stats().unwrap();
        assert_eq!(stats.total_files, 2);
    }

    #[test]
    fn test_search_in_drive() {
        let (service, _dir) = create_test_service();

        let files = vec![
            FileIndexRecord {
                id: None,
                name: "test.txt".to_string(),
                path: "C:\\test.txt".to_string(),
                size: 100,
                modified_time: "2024-01-01 12:00:00".to_string(),
                is_dir: false,
                file_type: "txt".to_string(),
                drive: "C:".to_string(),
                indexed_at: None,
            },
            FileIndexRecord {
                id: None,
                name: "test.txt".to_string(),
                path: "D:\\test.txt".to_string(),
                size: 200,
                modified_time: "2024-01-02 12:00:00".to_string(),
                is_dir: false,
                file_type: "txt".to_string(),
                drive: "D:".to_string(),
                indexed_at: None,
            },
        ];

        service.batch_insert(files).unwrap();

        let results = service.search_in_drive("test", "C:", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].drive, "C:");
    }

    #[test]
    fn test_remove_file() {
        let (service, _dir) = create_test_service();

        let file = FileIndexRecord {
            id: None,
            name: "to_delete.txt".to_string(),
            path: "C:\\to_delete.txt".to_string(),
            size: 100,
            modified_time: "2024-01-01 12:00:00".to_string(),
            is_dir: false,
            file_type: "txt".to_string(),
            drive: "C:".to_string(),
            indexed_at: None,
        };

        service.add_file(file).unwrap();

        let results = service.search("to_delete", 10).unwrap();
        assert_eq!(results.len(), 1);

        service.remove_file("C:\\to_delete.txt").unwrap();

        let results = service.search("to_delete", 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_clear_drive() {
        let (service, _dir) = create_test_service();

        let files = vec![
            FileIndexRecord {
                id: None,
                name: "c_file.txt".to_string(),
                path: "C:\\c_file.txt".to_string(),
                size: 100,
                modified_time: "2024-01-01 12:00:00".to_string(),
                is_dir: false,
                file_type: "txt".to_string(),
                drive: "C:".to_string(),
                indexed_at: None,
            },
            FileIndexRecord {
                id: None,
                name: "d_file.txt".to_string(),
                path: "D:\\d_file.txt".to_string(),
                size: 200,
                modified_time: "2024-01-02 12:00:00".to_string(),
                is_dir: false,
                file_type: "txt".to_string(),
                drive: "D:".to_string(),
                indexed_at: None,
            },
        ];

        service.batch_insert(files).unwrap();
        service.clear_drive("C:").unwrap();

        let stats = service.get_stats().unwrap();
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.indexed_drives, vec!["D:".to_string()]);
    }

    #[test]
    fn test_get_stats() {
        let (service, _dir) = create_test_service();

        let files = vec![
            FileIndexRecord {
                id: None,
                name: "file.txt".to_string(),
                path: "C:\\file.txt".to_string(),
                size: 100,
                modified_time: "2024-01-01 12:00:00".to_string(),
                is_dir: false,
                file_type: "txt".to_string(),
                drive: "C:".to_string(),
                indexed_at: None,
            },
            FileIndexRecord {
                id: None,
                name: "folder".to_string(),
                path: "C:\\folder".to_string(),
                size: 0,
                modified_time: "2024-01-01 12:00:00".to_string(),
                is_dir: true,
                file_type: "".to_string(),
                drive: "C:".to_string(),
                indexed_at: None,
            },
        ];

        service.batch_insert(files).unwrap();

        let stats = service.get_stats().unwrap();
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.total_dirs, 1);
        assert_eq!(stats.indexed_drives, vec!["C:".to_string()]);
    }
}
