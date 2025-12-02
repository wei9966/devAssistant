use crate::models::clipboard::{ClipboardConfig, ClipboardContentType, ClipboardHistory, CreateClipboardRecord};
use crate::services::sql_service::SqlService;
use arboard::Clipboard;
use chrono::Local;
use image::ImageFormat;
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 剪切板历史服务
pub struct ClipboardHistoryService {
    last_text_content: Arc<Mutex<String>>,
    last_image_hash: Arc<Mutex<u64>>,
    config: Arc<Mutex<ClipboardConfig>>,
}

impl ClipboardHistoryService {
    pub fn new() -> Self {
        Self {
            last_text_content: Arc::new(Mutex::new(String::new())),
            last_image_hash: Arc::new(Mutex::new(0)),
            config: Arc::new(Mutex::new(ClipboardConfig::default())),
        }
    }

    /// 加载配置
    pub fn load_config(&self, conn: &Connection) {
        if let Ok(config) = Self::get_config_from_db(conn) {
            *self.config.lock().unwrap() = config;
        }
    }

    /// 从数据库获取配置
    fn get_config_from_db(conn: &Connection) -> Result<ClipboardConfig, String> {
        let config_json: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'clipboard_config'",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(json) = config_json {
            serde_json::from_str(&json).map_err(|e| e.to_string())
        } else {
            Ok(ClipboardConfig::default())
        }
    }

    /// 保存配置到数据库
    pub fn save_config_to_db(conn: &Connection, config: &ClipboardConfig) -> Result<(), String> {
        let config_json = serde_json::to_string(config).map_err(|e| e.to_string())?;
        let now = Local::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?, ?, ?)",
            rusqlite::params!["clipboard_config", config_json, now],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 启动剪切板监控（后台线程）
    pub fn start_monitoring(&self, db_path: String) {
        let last_text = Arc::clone(&self.last_text_content);
        let last_image = Arc::clone(&self.last_image_hash);
        let config = Arc::clone(&self.config);

        thread::spawn(move || {
            println!("剪切板历史监控线程已启动");

            // 创建 arboard 剪切板实例
            let mut clipboard = match Clipboard::new() {
                Ok(cb) => cb,
                Err(e) => {
                    eprintln!("无法创建剪切板实例: {}", e);
                    return;
                }
            };

            loop {
                thread::sleep(Duration::from_millis(500));

                let current_config = config.lock().unwrap().clone();
                if !current_config.enabled {
                    continue;
                }

                // 检查文本内容
                if let Ok(text) = clipboard.get_text() {
                    let mut last = last_text.lock().unwrap();
                    if text != *last && !text.trim().is_empty() {
                        *last = text.clone();

                        // 保存到数据库
                        if let Ok(conn) = Connection::open(&db_path) {
                            let preview = text.chars().take(100).collect::<String>();
                            let record = CreateClipboardRecord {
                                content_type: ClipboardContentType::Text.to_string(),
                                content: text.clone(),
                                preview: Some(preview.clone()),
                                image_path: None,
                                source_app: None,
                            };

                            match Self::save_record(&conn, &record) {
                                Ok(id) => {
                                    println!("✓ 剪切板文本已保存, ID: {}, 预览: {}", id,
                                        preview.chars().take(30).collect::<String>());

                                    // 检查是否是 SQL 语句，如果是也保存到 SQL 历史
                                    if SqlService::is_valid_sql(&text) {
                                        if let Ok(sql_id) = SqlService::save_sql(&conn, &text, "clipboard") {
                                            println!("  ↳ 同时保存为 SQL 历史, ID: {}", sql_id);
                                        }
                                    }

                                    // 清理超出数量的旧记录
                                    let _ = Self::cleanup_old_records(&conn, current_config.max_history);
                                }
                                Err(e) => println!("✗ 保存剪切板内容失败: {}", e),
                            }
                        }
                    }
                }

                // 检查图片内容
                if let Ok(img) = clipboard.get_image() {
                    // 计算图片哈希（简单方式：使用图片尺寸和部分像素）
                    let hash = Self::calculate_image_hash(&img);
                    let mut last = last_image.lock().unwrap();

                    if hash != *last && hash != 0 {
                        *last = hash;

                        // 保存图片到文件
                        if let Ok(conn) = Connection::open(&db_path) {
                            let save_dir = &current_config.image_save_dir;

                            // 确保目录存在
                            if let Err(e) = fs::create_dir_all(save_dir) {
                                eprintln!("创建图片目录失败: {}", e);
                                continue;
                            }

                            // 生成文件名
                            let timestamp = Local::now().format("%Y%m%d_%H%M%S_%3f");
                            let filename = format!("clip_{}.png", timestamp);
                            let file_path = Path::new(save_dir).join(&filename);
                            let file_path_str = file_path.to_string_lossy().to_string();

                            // 保存图片
                            match Self::save_image_to_file(&img, &file_path) {
                                Ok(_) => {
                                    println!("✓ 剪切板图片已保存: {}", file_path_str);

                                    // 保存记录到数据库
                                    let record = CreateClipboardRecord {
                                        content_type: ClipboardContentType::Image.to_string(),
                                        content: file_path_str.clone(),
                                        preview: Some(format!("[图片] {}", filename)),
                                        image_path: Some(file_path_str.clone()),
                                        source_app: None,
                                    };

                                    if let Ok(id) = Self::save_record(&conn, &record) {
                                        println!("  ↳ 图片记录已保存, ID: {}", id);
                                    }

                                    // 如果配置了自动复制路径，将路径写回剪切板
                                    if current_config.auto_copy_image_path {
                                        if let Ok(mut cb) = Clipboard::new() {
                                            if cb.set_text(&file_path_str).is_ok() {
                                                // 更新 last_text 以避免重复记录
                                                *last_text.lock().unwrap() = file_path_str.clone();
                                                println!("  ↳ 图片路径已复制到剪切板: {}", file_path_str);
                                            }
                                        }
                                    }

                                    // 清理超出数量的旧记录
                                    let _ = Self::cleanup_old_records(&conn, current_config.max_history);
                                }
                                Err(e) => eprintln!("保存图片失败: {}", e),
                            }
                        }
                    }
                }
            }
        });
    }

    /// 计算图片哈希（简单实现）
    fn calculate_image_hash(img: &arboard::ImageData) -> u64 {
        let mut hash: u64 = 0;
        hash = hash.wrapping_add(img.width as u64);
        hash = hash.wrapping_mul(31).wrapping_add(img.height as u64);

        // 使用部分像素数据计算哈希
        let step = (img.bytes.len() / 100).max(1);
        for (i, &byte) in img.bytes.iter().step_by(step).take(100).enumerate() {
            hash = hash.wrapping_mul(31).wrapping_add((byte as u64).wrapping_mul(i as u64 + 1));
        }

        hash
    }

    /// 保存图片到文件
    fn save_image_to_file(img: &arboard::ImageData, path: &Path) -> Result<(), String> {
        // arboard 的图片数据是 RGBA 格式
        let img_buffer = image::RgbaImage::from_raw(
            img.width as u32,
            img.height as u32,
            img.bytes.to_vec(),
        )
        .ok_or("无法创建图片缓冲区")?;

        img_buffer
            .save_with_format(path, ImageFormat::Png)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 保存记录到数据库
    fn save_record(conn: &Connection, record: &CreateClipboardRecord) -> Result<i64, String> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // 检查是否已存在相同内容（避免重复）
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM clipboard_history WHERE content = ? AND content_type = ? LIMIT 1",
                rusqlite::params![&record.content, &record.content_type],
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = existing {
            // 更新已存在记录的时间
            conn.execute(
                "UPDATE clipboard_history SET created_at = ? WHERE id = ?",
                rusqlite::params![&now, id],
            )
            .map_err(|e| e.to_string())?;
            return Ok(id);
        }

        // 插入新记录
        conn.execute(
            "INSERT INTO clipboard_history (content_type, content, preview, image_path, source_app, is_pinned, created_at)
             VALUES (?, ?, ?, ?, ?, 0, ?)",
            rusqlite::params![
                &record.content_type,
                &record.content,
                &record.preview,
                &record.image_path,
                &record.source_app,
                &now,
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    /// 清理超出数量的旧记录（归档而非删除）
    fn cleanup_old_records(conn: &Connection, max_count: i32) -> Result<(), String> {
        // 获取需要归档的记录
        let config = Self::get_config_from_db(conn).unwrap_or_default();

        // 确保归档目录存在
        if let Err(e) = fs::create_dir_all(&config.image_archive_dir) {
            eprintln!("创建归档目录失败: {}", e);
        }

        // 获取要归档的记录（超出 max_count 的非置顶记录）
        let mut stmt = conn.prepare(
            "SELECT id, image_path FROM clipboard_history
             WHERE is_pinned = 0
             ORDER BY created_at DESC
             LIMIT -1 OFFSET ?"
        ).map_err(|e| e.to_string())?;

        let records: Vec<(i64, Option<String>)> = stmt
            .query_map(rusqlite::params![max_count], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        // 归档图片并删除记录
        for (id, image_path) in records {
            if let Some(path) = image_path {
                if !path.is_empty() {
                    // 移动图片到归档目录
                    let src_path = Path::new(&path);
                    if src_path.exists() {
                        if let Some(filename) = src_path.file_name() {
                            let dest_path = Path::new(&config.image_archive_dir).join(filename);
                            let _ = fs::rename(&path, &dest_path);
                        }
                    }
                }
            }

            // 删除数据库记录
            let _ = conn.execute(
                "DELETE FROM clipboard_history WHERE id = ?",
                rusqlite::params![id],
            );
        }

        Ok(())
    }

    /// 获取历史记录
    pub fn get_history(
        conn: &Connection,
        limit: i32,
        offset: i32,
        content_type: Option<&str>,
        keyword: Option<&str>,
        pinned_only: bool,
    ) -> Result<Vec<ClipboardHistory>, String> {
        let mut sql = String::from(
            "SELECT id, content_type, content, preview, image_path, source_app, is_pinned, created_at
             FROM clipboard_history WHERE 1=1"
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if pinned_only {
            sql.push_str(" AND is_pinned = 1");
        }

        if let Some(ct) = content_type {
            sql.push_str(" AND content_type = ?");
            params.push(Box::new(ct.to_string()));
        }

        if let Some(kw) = keyword {
            sql.push_str(" AND (content LIKE ? OR preview LIKE ?)");
            let pattern = format!("%{}%", kw);
            params.push(Box::new(pattern.clone()));
            params.push(Box::new(pattern));
        }

        sql.push_str(" ORDER BY is_pinned DESC, created_at DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit));
        params.push(Box::new(offset));

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(ClipboardHistory {
                    id: row.get(0)?,
                    content_type: row.get(1)?,
                    content: row.get(2)?,
                    preview: row.get(3)?,
                    image_path: row.get(4)?,
                    source_app: row.get(5)?,
                    is_pinned: row.get::<_, i32>(6)? == 1,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut history = Vec::new();
        for row in rows {
            if let Ok(item) = row {
                history.push(item);
            }
        }

        Ok(history)
    }

    /// 从历史复制内容
    pub fn copy_from_history(conn: &Connection, id: i64) -> Result<String, String> {
        let record: ClipboardHistory = conn
            .query_row(
                "SELECT id, content_type, content, preview, image_path, source_app, is_pinned, created_at
                 FROM clipboard_history WHERE id = ?",
                rusqlite::params![id],
                |row| {
                    Ok(ClipboardHistory {
                        id: row.get(0)?,
                        content_type: row.get(1)?,
                        content: row.get(2)?,
                        preview: row.get(3)?,
                        image_path: row.get(4)?,
                        source_app: row.get(5)?,
                        is_pinned: row.get::<_, i32>(6)? == 1,
                        created_at: row.get(7)?,
                    })
                },
            )
            .map_err(|e| format!("记录不存在: {}", e))?;

        let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;

        match record.content_type.as_str() {
            "image" => {
                // 如果是图片，复制图片路径
                if let Some(path) = &record.image_path {
                    clipboard.set_text(path).map_err(|e| e.to_string())?;
                    Ok(path.clone())
                } else {
                    clipboard.set_text(&record.content).map_err(|e| e.to_string())?;
                    Ok(record.content)
                }
            }
            _ => {
                clipboard.set_text(&record.content).map_err(|e| e.to_string())?;
                Ok(record.content)
            }
        }
    }

    /// 删除记录
    pub fn delete_record(conn: &Connection, id: i64) -> Result<(), String> {
        // 如果是图片，也删除对应的文件
        let image_path: Option<String> = conn
            .query_row(
                "SELECT image_path FROM clipboard_history WHERE id = ?",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .ok();

        if let Some(Some(path)) = image_path.map(Some) {
            if !path.is_empty() {
                let _ = fs::remove_file(&path);
            }
        }

        conn.execute(
            "DELETE FROM clipboard_history WHERE id = ?",
            rusqlite::params![id],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 清空历史
    pub fn clear_history(conn: &Connection, keep_pinned: bool) -> Result<i32, String> {
        // 获取要删除的图片路径
        let sql = if keep_pinned {
            "SELECT image_path FROM clipboard_history WHERE is_pinned = 0 AND image_path IS NOT NULL"
        } else {
            "SELECT image_path FROM clipboard_history WHERE image_path IS NOT NULL"
        };

        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let paths: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        // 删除图片文件
        for path in paths {
            let _ = fs::remove_file(&path);
        }

        // 删除数据库记录
        let delete_sql = if keep_pinned {
            "DELETE FROM clipboard_history WHERE is_pinned = 0"
        } else {
            "DELETE FROM clipboard_history"
        };

        let deleted = conn.execute(delete_sql, []).map_err(|e| e.to_string())?;

        Ok(deleted as i32)
    }

    /// 切换置顶状态
    pub fn toggle_pin(conn: &Connection, id: i64) -> Result<bool, String> {
        let current: i32 = conn
            .query_row(
                "SELECT is_pinned FROM clipboard_history WHERE id = ?",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let new_value = if current == 1 { 0 } else { 1 };

        conn.execute(
            "UPDATE clipboard_history SET is_pinned = ? WHERE id = ?",
            rusqlite::params![new_value, id],
        )
        .map_err(|e| e.to_string())?;

        Ok(new_value == 1)
    }

    /// 搜索历史
    pub fn search_history(conn: &Connection, keyword: &str, limit: i32) -> Result<Vec<ClipboardHistory>, String> {
        Self::get_history(conn, limit, 0, None, Some(keyword), false)
    }

    /// 获取配置
    pub fn get_config(conn: &Connection) -> Result<ClipboardConfig, String> {
        Self::get_config_from_db(conn)
    }

    /// 更新配置
    pub fn update_config(conn: &Connection, config: ClipboardConfig) -> Result<(), String> {
        Self::save_config_to_db(conn, &config)
    }
}
