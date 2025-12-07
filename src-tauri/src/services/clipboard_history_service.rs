use crate::models::clipboard::{ClipboardConfig, ClipboardContentType, ClipboardHistory, CreateClipboardRecord};
use crate::services::sql_service::SqlService;
use arboard::Clipboard;
use chrono::Local;
use image::ImageFormat;
use log::{error, warn, info, debug};
use rusqlite::Connection;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::panic;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// 全局标记：跳过下一次图片检测（避免复制图片时监控线程干扰）
static SKIP_NEXT_IMAGE: AtomicBool = AtomicBool::new(false);
// 记录最后一次手动复制的图片哈希
static LAST_MANUAL_COPY_HASH: AtomicU64 = AtomicU64::new(0);

/// 记录剪切板错误到日志文件
fn log_clipboard_error(error_msg: &str) {
    let log_dir = dirs::data_local_dir()
        .map(|p| p.join("dev-assistant").join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("./logs"));

    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("无法创建日志目录: {}", e);
        return;
    }

    let log_file = log_dir.join("clipboard_errors.log");
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_entry = format!("[{}] {}\n", timestamp, error_msg);

    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
    {
        Ok(mut file) => {
            let _ = file.write_all(log_entry.as_bytes());
        }
        Err(e) => {
            eprintln!("无法写入日志文件: {}", e);
        }
    }

    // 同时使用 log crate 记录
    error!("[Clipboard] {}", error_msg);
}

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
        // 首先尝试从 clipboard_config 键读取专用配置
        let config_json: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'clipboard_config'",
                [],
                |row| row.get(0),
            )
            .ok();

        let mut config = if let Some(json) = config_json {
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            ClipboardConfig::default()
        };

        // 从 app_settings 中读取 clipboard_interval 并同步到配置
        // 这样用户在设置界面修改的值会生效
        if let Ok(app_settings_json) = conn.query_row::<String, _, _>(
            "SELECT value FROM app_settings WHERE key = 'app_settings'",
            [],
            |row| row.get(0),
        ) {
            if let Ok(app_settings) = serde_json::from_str::<serde_json::Value>(&app_settings_json) {
                if let Some(interval) = app_settings.get("clipboard_interval").and_then(|v| v.as_u64()) {
                    // 将 app_settings 中的 clipboard_interval 同步到 poll_interval_secs
                    config.poll_interval_secs = interval.max(1); // 最小1秒
                }
            }
        }

        Ok(config)
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
    /// 优化：复用 Clipboard 实例，使用配置的轮询间隔
    pub fn start_monitoring(&self, db_path: String) {
        let last_text = Arc::clone(&self.last_text_content);
        let last_image = Arc::clone(&self.last_image_hash);
        let config = Arc::clone(&self.config);

        thread::spawn(move || {
            info!("剪切板历史监控线程已启动");
            println!("剪切板历史监控线程已启动");

            // 连续错误计数器
            let mut consecutive_errors = 0u32;
            const MAX_CONSECUTIVE_ERRORS: u32 = 10;

            // 复用 Clipboard 实例，避免每次轮询都重新创建
            let mut clipboard: Option<Clipboard> = None;
            // Clipboard 实例重建计数器（每隔一段时间重建以避免潜在的状态问题）
            let mut clipboard_usage_count = 0u32;
            const CLIPBOARD_REBUILD_INTERVAL: u32 = 100; // 每100次检查重建一次

            // 配置刷新计数器（每隔一段时间从数据库重新读取配置）
            let mut config_refresh_count = 0u32;
            const CONFIG_REFRESH_INTERVAL: u32 = 60; // 大约每60次轮询刷新一次配置

            loop {
                // 定期从数据库刷新配置
                config_refresh_count += 1;
                if config_refresh_count >= CONFIG_REFRESH_INTERVAL {
                    config_refresh_count = 0;
                    if let Ok(conn) = Connection::open(&db_path) {
                        if let Ok(new_config) = Self::get_config_from_db(&conn) {
                            if let Ok(mut cfg) = config.lock() {
                                *cfg = new_config;
                                debug!("剪切板配置已刷新，轮询间隔: {}秒", cfg.poll_interval_secs);
                            }
                        }
                    }
                }

                // 从配置获取轮询间隔，默认1秒
                let poll_interval = {
                    config.lock()
                        .map(|c| c.poll_interval_secs)
                        .unwrap_or(1)
                        .max(1) // 最小1秒，避免过于频繁
                };
                thread::sleep(Duration::from_secs(poll_interval));

                // 定期重建 Clipboard 实例
                clipboard_usage_count += 1;
                if clipboard_usage_count >= CLIPBOARD_REBUILD_INTERVAL || clipboard.is_none() {
                    clipboard = Clipboard::new().ok();
                    clipboard_usage_count = 0;
                    if clipboard.is_none() {
                        log_clipboard_error("无法创建剪切板实例，将在下次轮询时重试");
                        continue;
                    }
                }

                // 使用 catch_unwind 捕获 panic，防止线程崩溃导致整个程序崩溃
                let clipboard_ref = clipboard.as_mut();
                let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    if let Some(cb) = clipboard_ref {
                        Self::monitor_clipboard_once_optimized(
                            &db_path,
                            &last_text,
                            &last_image,
                            &config,
                            cb,
                        )
                    } else {
                        Err("剪切板实例不可用".to_string())
                    }
                }));

                match result {
                    Ok(Ok(())) => {
                        // 正常执行，重置错误计数
                        consecutive_errors = 0;
                    }
                    Ok(Err(e)) => {
                        // 普通错误（非 panic）
                        consecutive_errors += 1;
                        let error_msg = format!("剪切板监控错误 ({}/{}): {}",
                            consecutive_errors, MAX_CONSECUTIVE_ERRORS, e);
                        log_clipboard_error(&error_msg);

                        // 出错时重建 Clipboard 实例
                        clipboard = None;

                        if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                            warn!("剪切板监控连续错误过多，暂停 5 秒");
                            thread::sleep(Duration::from_secs(5));
                            consecutive_errors = 0;
                        }
                    }
                    Err(panic_info) => {
                        // 捕获到 panic
                        consecutive_errors += 1;
                        let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = panic_info.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "未知 panic".to_string()
                        };

                        let error_msg = format!("剪切板监控 PANIC ({}/{}): {}",
                            consecutive_errors, MAX_CONSECUTIVE_ERRORS, panic_msg);
                        log_clipboard_error(&error_msg);

                        // panic 后重建 Clipboard 实例
                        clipboard = None;

                        if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                            warn!("剪切板监控连续 panic 过多，暂停 10 秒后重试");
                            thread::sleep(Duration::from_secs(10));
                            consecutive_errors = 0;
                        }
                    }
                }
            }
        });
    }

    /// 优化版：执行一次剪切板监控检查（复用 Clipboard 实例）
    fn monitor_clipboard_once_optimized(
        db_path: &str,
        last_text: &Arc<Mutex<String>>,
        last_image: &Arc<Mutex<u64>>,
        config: &Arc<Mutex<ClipboardConfig>>,
        clipboard: &mut Clipboard,
    ) -> Result<(), String> {
        let current_config = config.lock().map_err(|e| format!("获取配置锁失败: {}", e))?.clone();
        if !current_config.enabled {
            return Ok(());
        }

        // 使用传入的复用 Clipboard 实例，不再每次创建新实例

        // 检查文本内容 - 使用 Result 处理错误
        match clipboard.get_text() {
            Ok(text) => {
                if !text.trim().is_empty() {
                    let mut last = last_text.lock().map_err(|e| format!("获取文本锁失败: {}", e))?;
                    if text != *last {
                        *last = text.clone();
                        drop(last); // 释放锁

                        // 保存到数据库
                        if let Ok(conn) = Connection::open(db_path) {
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
                                    debug!("✓ 剪切板文本已保存, ID: {}", id);
                                    println!("✓ 剪切板文本已保存, ID: {}, 预览: {}", id,
                                        preview.chars().take(30).collect::<String>());

                                    // 检查是否是 SQL 语句
                                    if SqlService::is_valid_sql(&text) {
                                        if let Ok(sql_id) = SqlService::save_sql(&conn, &text, "clipboard") {
                                            println!("  ↳ 同时保存为 SQL 历史, ID: {}", sql_id);
                                        }
                                    }

                                    let _ = Self::cleanup_old_records(&conn, current_config.max_history);
                                }
                                Err(e) => {
                                    log_clipboard_error(&format!("保存剪切板文本失败: {}", e));
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // 文本获取失败通常是因为剪切板中没有文本或格式不支持
                // 这不是严重错误，只记录 debug 级别
                debug!("获取剪切板文本失败（可能无文本内容）: {}", e);
            }
        }

        // 检查图片内容
        match clipboard.get_image() {
            Ok(img) => {
                // 检查是否需要跳过
                if SKIP_NEXT_IMAGE.swap(false, Ordering::SeqCst) {
                    debug!("跳过图片检测（手动复制触发）");
                    return Ok(());
                }

                // 验证图片数据有效性
                if img.width == 0 || img.height == 0 || img.bytes.is_empty() {
                    debug!("跳过无效图片数据: width={}, height={}, bytes={}",
                        img.width, img.height, img.bytes.len());
                    return Ok(());
                }

                // 验证图片尺寸是否合理（避免异常大的图片导致内存问题）
                const MAX_IMAGE_DIMENSION: usize = 16384; // 16K 分辨率
                const MAX_IMAGE_BYTES: usize = 100 * 1024 * 1024; // 100MB

                if img.width > MAX_IMAGE_DIMENSION || img.height > MAX_IMAGE_DIMENSION {
                    log_clipboard_error(&format!(
                        "跳过超大图片: {}x{} (最大 {}x{})",
                        img.width, img.height, MAX_IMAGE_DIMENSION, MAX_IMAGE_DIMENSION
                    ));
                    return Ok(());
                }

                if img.bytes.len() > MAX_IMAGE_BYTES {
                    log_clipboard_error(&format!(
                        "跳过超大图片数据: {} bytes (最大 {} bytes)",
                        img.bytes.len(), MAX_IMAGE_BYTES
                    ));
                    return Ok(());
                }

                let hash = Self::calculate_image_hash(&img);
                let mut last = last_image.lock().map_err(|e| format!("获取图片锁失败: {}", e))?;

                if hash != *last && hash != 0 {
                    *last = hash;
                    drop(last);

                    if let Ok(conn) = Connection::open(db_path) {
                        let save_dir = &current_config.image_save_dir;

                        if let Err(e) = fs::create_dir_all(save_dir) {
                            return Err(format!("创建图片目录失败: {}", e));
                        }

                        let timestamp = Local::now().format("%Y%m%d_%H%M%S_%3f");
                        let filename = format!("clip_{}.png", timestamp);
                        let file_path = Path::new(save_dir).join(&filename);
                        let file_path_str = file_path.to_string_lossy().to_string();

                        match Self::save_image_to_file(&img, &file_path) {
                            Ok(_) => {
                                println!("✓ 剪切板图片已保存: {}", file_path_str);

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

                                let _ = Self::cleanup_old_records(&conn, current_config.max_history);
                            }
                            Err(e) => {
                                log_clipboard_error(&format!("保存图片失败: {}", e));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // 图片获取失败通常是因为剪切板中没有图片
                // 这不是严重错误
                debug!("获取剪切板图片失败（可能无图片内容）: {}", e);
            }
        }

        Ok(())
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
        // 验证图片数据
        let expected_size = img.width * img.height * 4; // RGBA = 4 bytes per pixel
        if img.bytes.len() != expected_size {
            return Err(format!(
                "图片数据大小不匹配: 期望 {} bytes ({}x{}x4), 实际 {} bytes",
                expected_size, img.width, img.height, img.bytes.len()
            ));
        }

        // arboard 的图片数据是 RGBA 格式
        let img_buffer = image::RgbaImage::from_raw(
            img.width as u32,
            img.height as u32,
            img.bytes.to_vec(),
        )
        .ok_or_else(|| format!(
            "无法创建图片缓冲区: {}x{}, {} bytes",
            img.width, img.height, img.bytes.len()
        ))?;

        img_buffer
            .save_with_format(path, ImageFormat::Png)
            .map_err(|e| format!("保存 PNG 文件失败: {}", e))?;

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

        let mut clipboard = Clipboard::new().map_err(|e| format!("创建剪切板实例失败: {}", e))?;

        match record.content_type.as_str() {
            "image" => {
                // 如果是图片，复制实际图片数据
                let image_path = record.image_path.as_ref().unwrap_or(&record.content);
                let path_str = image_path.clone();

                debug!("尝试复制图片: {}", image_path);
                println!("尝试复制图片: {}", image_path);

                // 设置标记，让监控线程跳过这次
                SKIP_NEXT_IMAGE.store(true, Ordering::SeqCst);

                // 使用独立线程复制图片（Windows 剪切板 API 需要）
                // 使用 catch_unwind 防止 panic 导致程序崩溃
                let result = std::thread::spawn(move || {
                    panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        Self::copy_image_to_clipboard_impl(&path_str)
                    }))
                }).join();

                match result {
                    Ok(Ok(Ok(msg))) => {
                        println!("✓ {}", msg);
                        Ok(format!("[图片已复制] {}", image_path))
                    }
                    Ok(Ok(Err(e))) => {
                        SKIP_NEXT_IMAGE.store(false, Ordering::SeqCst);
                        let error_msg = format!("复制图片失败: {}", e);
                        log_clipboard_error(&error_msg);
                        println!("✗ {}", e);
                        Err(e)
                    }
                    Ok(Err(panic_info)) => {
                        SKIP_NEXT_IMAGE.store(false, Ordering::SeqCst);
                        let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = panic_info.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "未知错误".to_string()
                        };
                        let error_msg = format!("复制图片时发生 panic: {}", panic_msg);
                        log_clipboard_error(&error_msg);
                        Err(error_msg)
                    }
                    Err(_) => {
                        SKIP_NEXT_IMAGE.store(false, Ordering::SeqCst);
                        let error_msg = "复制图片线程执行失败".to_string();
                        log_clipboard_error(&error_msg);
                        Err(error_msg)
                    }
                }
            }
            _ => {
                clipboard.set_text(&record.content).map_err(|e| {
                    let error_msg = format!("复制文本到剪切板失败: {}", e);
                    log_clipboard_error(&error_msg);
                    error_msg
                })?;
                Ok(record.content)
            }
        }
    }

    /// 复制图片到剪切板的内部实现
    #[cfg(windows)]
    fn copy_image_to_clipboard_impl(image_path: &str) -> Result<String, String> {
        use clipboard_win::{formats, Clipboard, Setter};
        use image::GenericImageView;

        let path = Path::new(image_path);
        if !path.exists() {
            return Err(format!("图片文件不存在: {}", image_path));
        }

        // 直接读取 PNG 文件的原始字节
        let png_data = fs::read(path).map_err(|e| format!("无法读取图片文件: {}", e))?;

        println!("PNG 文件大小: {} bytes", png_data.len());

        // 打开剪切板
        let _clip = Clipboard::new_attempts(10)
            .map_err(|e| format!("无法打开剪切板: {}", e))?;

        // 注册 PNG 格式并写入
        // Windows 剪切板支持 "PNG" 格式
        let png_format = clipboard_win::register_format("PNG")
            .ok_or("无法注册 PNG 剪切板格式")?;

        clipboard_win::raw::set(png_format.get(), &png_data)
            .map_err(|e| format!("写入 PNG 到剪切板失败: {}", e))?;

        // 同时写入 DIB 格式以提高兼容性
        let img = image::open(path).map_err(|e| format!("无法打开图片: {}", e))?;
        let rgb = img.to_rgb8();
        let (width, height) = rgb.dimensions();

        // 创建 DIB - 使用 RGB 格式，自下而上
        let row_size = ((width * 3 + 3) / 4 * 4) as usize; // 4字节对齐
        let mut dib_data = Vec::new();

        // BITMAPINFOHEADER
        dib_data.extend_from_slice(&40u32.to_le_bytes());  // biSize
        dib_data.extend_from_slice(&(width as i32).to_le_bytes());  // biWidth
        dib_data.extend_from_slice(&(height as i32).to_le_bytes()); // biHeight (正值=自下而上)
        dib_data.extend_from_slice(&1u16.to_le_bytes());   // biPlanes
        dib_data.extend_from_slice(&24u16.to_le_bytes());  // biBitCount (24位RGB)
        dib_data.extend_from_slice(&0u32.to_le_bytes());   // biCompression
        dib_data.extend_from_slice(&((row_size * height as usize) as u32).to_le_bytes()); // biSizeImage
        dib_data.extend_from_slice(&0i32.to_le_bytes());   // biXPelsPerMeter
        dib_data.extend_from_slice(&0i32.to_le_bytes());   // biYPelsPerMeter
        dib_data.extend_from_slice(&0u32.to_le_bytes());   // biClrUsed
        dib_data.extend_from_slice(&0u32.to_le_bytes());   // biClrImportant

        // 像素数据 - 自下而上，BGR 格式
        for y in (0..height).rev() {
            for x in 0..width {
                let pixel = rgb.get_pixel(x, y);
                dib_data.push(pixel[2]); // B
                dib_data.push(pixel[1]); // G
                dib_data.push(pixel[0]); // R
            }
            // 行填充到 4 字节对齐
            let padding = row_size - (width as usize * 3);
            for _ in 0..padding {
                dib_data.push(0);
            }
        }

        formats::Bitmap.write_clipboard(&dib_data)
            .map_err(|e| format!("写入 DIB 到剪切板失败: {}", e))?;

        Ok("图片已成功复制到剪切板".to_string())
    }

    #[cfg(not(windows))]
    fn copy_image_to_clipboard_impl(image_path: &str) -> Result<String, String> {
        // 非 Windows 平台使用 arboard
        let path = Path::new(image_path);
        if !path.exists() {
            return Err(format!("图片文件不存在: {}", image_path));
        }

        let img = image::open(path).map_err(|e| format!("无法打开图片: {}", e))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let img_data = arboard::ImageData {
            width: width as usize,
            height: height as usize,
            bytes: std::borrow::Cow::Owned(rgba.into_raw()),
        };

        let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
        clipboard.set_image(img_data).map_err(|e| e.to_string())?;

        Ok("图片已成功复制到剪切板".to_string())
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
