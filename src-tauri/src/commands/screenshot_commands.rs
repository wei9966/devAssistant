// Screenshot Commands
// 截图回顾相关的 Tauri 命令

use crate::db::connection::DbConnection;
use crate::models::screen_context::ScreenContext;
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::Path;
use tauri::State;

/// 截图列表查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotListQuery {
    /// 开始日期 (YYYY-MM-DD)
    pub start_date: Option<String>,
    /// 结束日期 (YYYY-MM-DD)
    pub end_date: Option<String>,
    /// 应用名筛选
    pub app_name: Option<String>,
    /// 活动类型筛选 (coding, browsing, chatting, document, design, other)
    pub activity_type: Option<String>,
    /// 页码 (从1开始)
    pub page: Option<u32>,
    /// 每页数量
    pub page_size: Option<u32>,
}

/// 截图记录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotRecord {
    pub id: i64,
    pub captured_at: String,
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub activity_type: String,
    pub description: String,
    pub key_content: Option<String>,
    pub screenshot_path: Option<String>,
    pub processing_time_ms: Option<i64>,
}

/// 截图列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotListResponse {
    pub records: Vec<ScreenshotRecord>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

/// 截图详情响应（包含base64图片数据）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotDetail {
    pub id: i64,
    pub captured_at: String,
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub activity_type: String,
    pub description: String,
    pub key_content: Option<String>,
    pub screenshot_path: Option<String>,
    pub processing_time_ms: Option<i64>,
    /// Base64编码的图片数据
    pub image_data: Option<String>,
}

/// 截图资源（用于活动分组）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotResource {
    pub id: i64,
    pub path: Option<String>,
    pub captured_at: String,
    pub app_name: Option<String>,
    pub description: Option<String>,  // 单张截图的描述
}

/// 活动分组（时间段分组）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityGroup {
    pub id: String,              // 活动ID (使用开始时间戳)
    pub start_time: String,      // 时间段开始
    pub end_time: String,        // 时间段结束
    pub title: String,           // 时间段标题 (如 "10:00 - 11:00")
    pub description: String,     // 该时间段的活动总结（合并该段所有description）
    pub activity_type: String,   // 主要活动类型
    pub screenshots: Vec<ScreenshotResource>, // 该时间段的截图列表
}

/// 读取截图文件，返回base64编码的图片数据
#[tauri::command]
pub async fn screenshot_get_image(
    file_path: String,
    generate_thumbnail: Option<bool>,
) -> Result<String, String> {
    let path = Path::new(&file_path);

    // 检查文件是否存在
    if !path.exists() {
        return Err(format!("截图文件不存在: {}", file_path));
    }

    // 如果需要缩略图
    if generate_thumbnail.unwrap_or(false) {
        // 1. 首先尝试读取预生成的缩略图（在 thumbs 子目录中）
        if let Some(thumb_path) = get_pregenerated_thumb_path(&file_path) {
            if let Ok(thumb_data) = std::fs::read(&thumb_path) {
                return Ok(general_purpose::STANDARD.encode(&thumb_data));
            }
        }

        // 2. 检查缓存目录
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("dev-assistant")
            .join("thumbnails");

        let _ = std::fs::create_dir_all(&cache_dir);

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        let cache_name = format!("{:x}.jpg", hasher.finish());
        let cache_path = cache_dir.join(&cache_name);

        // 检查缓存是否存在且比原文件新
        if cache_path.exists() {
            if let (Ok(cache_meta), Ok(orig_meta)) = (cache_path.metadata(), path.metadata()) {
                if let (Ok(cache_time), Ok(orig_time)) = (cache_meta.modified(), orig_meta.modified()) {
                    if cache_time >= orig_time {
                        if let Ok(cached_data) = std::fs::read(&cache_path) {
                            return Ok(general_purpose::STANDARD.encode(&cached_data));
                        }
                    }
                }
            }
        }

        // 3. 需要生成缩略图 - 使用 tokio::task::spawn_blocking 避免阻塞异步运行时
        let path_clone = file_path.clone();
        let cache_path_clone = cache_path.clone();

        let result = tokio::task::spawn_blocking(move || {
            let image_data = std::fs::read(&path_clone)?;
            generate_thumbnail_data(&image_data)
        })
        .await
        .map_err(|e| format!("任务执行失败: {}", e))?;

        match result {
            Ok((thumbnail_base64, thumbnail_bytes)) => {
                // 异步写入缓存
                std::thread::spawn(move || {
                    let _ = std::fs::write(cache_path_clone, thumbnail_bytes);
                });
                return Ok(thumbnail_base64);
            }
            Err(e) => {
                eprintln!("生成缩略图失败，返回原图: {}", e);
                let image_data = std::fs::read(path)
                    .map_err(|e| format!("读取截图文件失败: {}", e))?;
                return Ok(general_purpose::STANDARD.encode(&image_data));
            }
        }
    }

    // 读取原图
    let image_data = std::fs::read(path)
        .map_err(|e| format!("读取截图文件失败: {}", e))?;

    Ok(general_purpose::STANDARD.encode(&image_data))
}

/// 根据原图路径获取预生成缩略图的路径
fn get_pregenerated_thumb_path(original_path: &str) -> Option<std::path::PathBuf> {
    let path = Path::new(original_path);
    let parent = path.parent()?;
    let file_name = path.file_name()?.to_str()?;

    // 原图格式: screenshot_20241207_153000_123456.png
    // 缩略图格式: thumb_20241207_153000_123456.jpg
    if file_name.starts_with("screenshot_") && file_name.ends_with(".png") {
        let thumb_name = file_name
            .replace("screenshot_", "thumb_")
            .replace(".png", ".jpg");
        let thumb_path = parent.join("thumbs").join(thumb_name);
        if thumb_path.exists() {
            return Some(thumb_path);
        }
    }

    None
}

/// 生成缩略图数据，返回 (base64编码, 原始字节)
/// 使用 Nearest 算法替代 Triangle，速度更快
fn generate_thumbnail_data(image_data: &[u8]) -> std::io::Result<(String, Vec<u8>)> {
    use image::GenericImageView;

    // 解码图片
    let img = image::load_from_memory(image_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    // 计算缩略图尺寸（最大宽度或高度为300px）
    let (width, height) = img.dimensions();
    let max_size = 300;

    let (thumb_width, thumb_height) = if width > height {
        if width > max_size {
            let ratio = max_size as f32 / width as f32;
            (max_size, (height as f32 * ratio) as u32)
        } else {
            (width, height)
        }
    } else {
        if height > max_size {
            let ratio = max_size as f32 / height as f32;
            ((width as f32 * ratio) as u32, max_size)
        } else {
            (width, height)
        }
    };

    // 生成缩略图（使用 Nearest 算法，最快）
    let thumbnail = img.resize(
        thumb_width,
        thumb_height,
        image::imageops::FilterType::Nearest,
    );

    // 编码为JPEG格式（质量70%，速度和大小的平衡）
    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);

    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 70);
    thumbnail.write_with_encoder(encoder)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // 返回 base64 编码和原始字节
    let base64 = general_purpose::STANDARD.encode(&buffer);
    Ok((base64, buffer))
}

/// 获取截图列表
#[tauri::command]
pub async fn screenshot_list(
    query: ScreenshotListQuery,
    db: State<'_, DbConnection>,
) -> Result<ScreenshotListResponse, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 构建SQL查询
    let mut sql = String::from(
        "SELECT id, captured_at, app_name, window_title, activity_type,
                description, key_content, screenshot_path, processing_time_ms
         FROM screen_contexts WHERE 1=1"
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    // 添加日期范围筛选（使用直接字符串比较避免date()函数的时区问题）
    if let Some(start_date) = &query.start_date {
        sql.push_str(" AND substr(captured_at, 1, 10) >= ?");
        params.push(Box::new(start_date.clone()));
    }

    if let Some(end_date) = &query.end_date {
        sql.push_str(" AND substr(captured_at, 1, 10) <= ?");
        params.push(Box::new(end_date.clone()));
    }

    // 添加应用名筛选
    if let Some(app_name) = &query.app_name {
        sql.push_str(" AND app_name = ?");
        params.push(Box::new(app_name.clone()));
    }

    // 添加活动类型筛选
    if let Some(activity_type) = &query.activity_type {
        sql.push_str(" AND activity_type = ?");
        params.push(Box::new(activity_type.clone()));
    }

    // 计算总数
    let count_sql = format!("SELECT COUNT(*) FROM ({}) AS total", sql);
    let total: u32 = {
        let mut stmt = conn.prepare(&count_sql).map_err(|e| e.to_string())?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        stmt.query_row(params_refs.as_slice(), |row| row.get(0))
            .map_err(|e| e.to_string())?
    };

    // 添加排序和分页
    sql.push_str(" ORDER BY captured_at DESC");

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1).min(100);
    let offset = (page - 1) * page_size;

    sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));

    // 执行查询
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let records = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(ScreenshotRecord {
                id: row.get(0)?,
                captured_at: row.get(1)?,
                app_name: row.get(2)?,
                window_title: row.get(3)?,
                activity_type: row.get(4)?,
                description: row.get(5)?,
                key_content: row.get(6)?,
                screenshot_path: row.get(7)?,
                processing_time_ms: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(ScreenshotListResponse {
        records,
        total,
        page,
        page_size,
    })
}

/// 获取单个截图详情（包含图片数据）
#[tauri::command]
pub async fn screenshot_get_detail(
    screenshot_id: i64,
    db: State<'_, DbConnection>,
) -> Result<ScreenshotDetail, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 查询截图记录
    let context: ScreenContext = conn
        .query_row(
            "SELECT id, captured_at, app_name, window_title, activity_type,
                    description, key_content, screenshot_hash, screenshot_path,
                    processing_time_ms
             FROM screen_contexts WHERE id = ?",
            [screenshot_id],
            |row| {
                Ok(ScreenContext {
                    id: Some(row.get(0)?),
                    captured_at: row.get(1)?,
                    app_name: row.get(2)?,
                    window_title: row.get(3)?,
                    activity_type: row.get(4)?,
                    description: row.get(5)?,
                    key_content: row.get(6)?,
                    screenshot_hash: row.get(7)?,
                    screenshot_path: row.get(8)?,
                    processing_time_ms: row.get(9)?,
                })
            },
        )
        .map_err(|e| format!("查询截图记录失败: {}", e))?;

    // 读取截图文件（如果存在）
    let image_data = if let Some(ref path) = context.screenshot_path {
        match std::fs::read(path) {
            Ok(data) => Some(general_purpose::STANDARD.encode(&data)),
            Err(e) => {
                eprintln!("读取截图文件失败: {} - {}", path, e);
                None
            }
        }
    } else {
        None
    };

    Ok(ScreenshotDetail {
        id: context.id.unwrap(),
        captured_at: context.captured_at,
        app_name: context.app_name,
        window_title: context.window_title,
        activity_type: context.activity_type,
        description: context.description,
        key_content: context.key_content,
        screenshot_path: context.screenshot_path,
        processing_time_ms: context.processing_time_ms,
        image_data,
    })
}

/// 获取按时间段分组的活动列表
#[tauri::command]
pub async fn screenshot_get_activities(
    date: String,  // YYYY-MM-DD 格式
    db: State<'_, DbConnection>,
) -> Result<Vec<ActivityGroup>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 查询指定日期的所有截图，按小时分组
    // 兼容两种时间格式：
    // - 旧格式 (RFC3339): 2024-12-04T21:30:00+08:00 -> substr 提取小时
    // - 新格式: 2024-12-04 21:30:00 -> strftime 提取小时
    // 注意：使用直接字符串比较避免 date() 函数的时区转换问题
    log::info!("查询活动列表，日期: {}", date);

    // 1. 读取调度器配置，判断是否启用小时总结功能
    let enable_hourly_summary = {
        let config_json: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'scheduler_config'",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(json) = config_json {
            // 尝试解析配置，如果失败则默认启用
            serde_json::from_str::<serde_json::Value>(&json)
                .ok()
                .and_then(|v| v.get("enable_hourly_summary").and_then(|v| v.as_bool()))
                .unwrap_or(true)
        } else {
            // 如果没有配置，默认启用小时总结
            true
        }
    };
    log::info!("小时总结功能开关: {}", if enable_hourly_summary { "已启用" } else { "已关闭" });

    // 2. 根据配置查询该日期已有的小时总结
    // 如果启用了小时总结：优先从 hourly_summaries 表获取，如果没有则从 activity_summaries 表获取（作为回退）
    // 如果未启用小时总结：直接使用 activity_summaries 表或截图描述拼接
    let mut hourly_summaries: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();
    {
        if enable_hourly_summary {
            // a) 优先从 hourly_summaries 表查询
            let hourly_sql = "
                SELECT
                    hour,
                    summary_text,
                    activity_type
                FROM hourly_summaries
                WHERE date = ?
            ";
            if let Ok(mut hourly_stmt) = conn.prepare(hourly_sql) {
                if let Ok(rows) = hourly_stmt.query_map([&date], |row| {
                    let hour: i64 = row.get(0)?;
                    let summary_text: String = row.get(1)?;
                    let activity_type: String = row.get(2)?;
                    Ok((hour, summary_text, activity_type))
                }) {
                    for row in rows.flatten() {
                        // 将小时转换为两位字符串格式 (如 "09", "14")
                        let hour_str = format!("{:02}", row.0);
                        hourly_summaries.insert(hour_str, (row.1, row.2));
                    }
                }
            }
            log::info!("从 hourly_summaries 表查询到 {} 个小时总结", hourly_summaries.len());
        }

        // b) 如果未启用小时总结或 hourly_summaries 没有数据，从 activity_summaries 查询作为回退
        if hourly_summaries.is_empty() {
            if enable_hourly_summary {
                log::info!("hourly_summaries 表无数据，从 activity_summaries 表查询作为回退");
            } else {
                log::info!("小时总结功能已关闭，从 activity_summaries 表查询");
            }

            let summary_sql = "
                SELECT
                    substr(start_time, 12, 2) as hour,
                    summary_text,
                    activity_type
                FROM activity_summaries
                WHERE substr(start_time, 1, 10) = ?
            ";
            if let Ok(mut summary_stmt) = conn.prepare(summary_sql) {
                if let Ok(rows) = summary_stmt.query_map([&date], |row| {
                    let hour: String = row.get(0)?;
                    let summary_text: String = row.get(1)?;
                    let activity_type: String = row.get(2)?;
                    Ok((hour, summary_text, activity_type))
                }) {
                    for row in rows.flatten() {
                        hourly_summaries.insert(row.0, (row.1, row.2));
                    }
                }
            }
        }
        log::info!("最终找到 {} 个已有的小时总结", hourly_summaries.len());
    }

    // 3. 查询截图数据
    let sql = "
        SELECT
            CASE
                WHEN captured_at LIKE '%T%' THEN substr(captured_at, 12, 2)
                ELSE substr(captured_at, 12, 2)
            END as hour,
            GROUP_CONCAT(id) as ids,
            GROUP_CONCAT(captured_at, '|||') as times,
            GROUP_CONCAT(screenshot_path, '|||') as paths,
            GROUP_CONCAT(COALESCE(app_name, ''), '|||') as apps,
            GROUP_CONCAT(COALESCE(description, ''), '|||') as descriptions,
            GROUP_CONCAT(COALESCE(activity_type, 'other'), '|||') as types,
            MIN(captured_at) as start_time,
            MAX(captured_at) as end_time
        FROM screen_contexts
        WHERE substr(captured_at, 1, 10) = ?
        GROUP BY hour
        ORDER BY end_time DESC
    ";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    // 收集原始数据
    let raw_activities: Vec<(String, String, String, String, String, String, String, String, String)> = stmt
        .query_map([&date], |row| {
            Ok((
                row.get::<_, String>(0)?,  // hour
                row.get::<_, String>(1)?,  // ids
                row.get::<_, String>(2)?,  // times
                row.get::<_, String>(3)?,  // paths
                row.get::<_, String>(4)?,  // apps
                row.get::<_, String>(5)?,  // descriptions
                row.get::<_, String>(6)?,  // types
                row.get::<_, String>(7)?,  // start_time
                row.get::<_, String>(8)?,  // end_time
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 4. 处理每个小时的数据，结合已有的小时总结
    let mut activities = Vec::new();
    for (hour, ids, times, paths, apps, descriptions, types, start_time, end_time) in raw_activities {
        // 解析ID列表
        let id_list: Vec<i64> = ids
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();

        // 解析时间列表
        let time_list: Vec<String> = times
            .split("|||")
            .map(|s| s.to_string())
            .collect();

        // 解析路径列表
        let path_list: Vec<Option<String>> = paths
            .split("|||")
            .map(|s| if s.is_empty() { None } else { Some(s.to_string()) })
            .collect();

        // 解析应用名列表
        let app_list: Vec<Option<String>> = apps
            .split("|||")
            .map(|s| if s.is_empty() { None } else { Some(s.to_string()) })
            .collect();

        // 解析描述列表（保持与其他字段同样的顺序）
        let desc_list: Vec<Option<String>> = descriptions
            .split("|||")
            .map(|s| if s.is_empty() { None } else { Some(s.to_string()) })
            .collect();

        // 解析活动类型列表
        let type_list: Vec<String> = types
            .split("|||")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        // 统计出现次数最多的活动类型（用作回退）
        let mut type_counts = std::collections::HashMap::new();
        for t in &type_list {
            *type_counts.entry(t.clone()).or_insert(0) += 1;
        }
        let fallback_activity_type = type_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(t, _)| t)
            .unwrap_or_else(|| "other".to_string());

        // 构建截图资源列表（每张截图有自己的描述）
        let screenshots: Vec<ScreenshotResource> = id_list
            .iter()
            .enumerate()
            .map(|(i, &id)| ScreenshotResource {
                id,
                path: path_list.get(i).and_then(|p| p.clone()),
                captured_at: time_list.get(i).cloned().unwrap_or_default(),
                app_name: app_list.get(i).and_then(|a| a.clone()),
                description: desc_list.get(i).and_then(|d| d.clone()),
            })
            .collect();

        // 优先使用已有的小时总结，否则拼接截图描述
        let (description, activity_type) = if let Some((summary_text, summary_type)) = hourly_summaries.get(&hour) {
            log::debug!("使用已有的小时总结: hour={}", hour);
            (summary_text.clone(), summary_type.clone())
        } else {
            // 生成活动组的简洁摘要（去重并截取前几个）
            let unique_descs: Vec<&str> = desc_list
                .iter()
                .filter_map(|d| d.as_deref())
                .filter(|s| !s.is_empty())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .take(3)  // 最多取3个不同的描述
                .collect();

            let desc = if unique_descs.is_empty() {
                "暂无描述".to_string()
            } else {
                // 每个描述截取前50个字符
                unique_descs
                    .iter()
                    .map(|d| {
                        let chars: String = d.chars().take(50).collect();
                        if d.chars().count() > 50 {
                            format!("{}...", chars)
                        } else {
                            chars
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("；")
            };
            (desc, fallback_activity_type)
        };

        // 生成标题：HH:00 - HH:59
        let title = format!("{}:00 - {}:59", hour, hour);

        // 使用开始时间戳作为ID
        let id = start_time.clone();

        activities.push(ActivityGroup {
            id,
            start_time,
            end_time,
            title,
            description,
            activity_type,
            screenshots,
        });
    }

    log::info!("查询到 {} 个活动分组", activities.len());
    for act in &activities {
        log::debug!("活动: {} ({} 张截图), 时间: {} - {}", act.title, act.screenshots.len(), act.start_time, act.end_time);
    }

    Ok(activities)
}

/// 调试命令：检查数据库中的截图时间格式
#[tauri::command]
pub async fn screenshot_debug_dates(
    date: Option<String>,  // 可选的日期参数，用于检查特定日期的数据
    db: State<'_, DbConnection>,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut results = Vec::new();

    // 1. 显示最近10条记录的原始格式
    results.push("=== 最近10条记录 ===".to_string());
    let sql1 = "
        SELECT id, captured_at, substr(captured_at, 1, 10) as date_part, substr(captured_at, 12, 2) as hour_part
        FROM screen_contexts
        ORDER BY id DESC
        LIMIT 10
    ";
    let mut stmt1 = conn.prepare(sql1).map_err(|e| e.to_string())?;
    let records: Vec<String> = stmt1
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let captured_at: String = row.get(1)?;
            let date_part: String = row.get(2)?;
            let hour_part: String = row.get(3)?;
            Ok(format!("ID:{}, time:'{}', date:'{}', hour:'{}'", id, captured_at, date_part, hour_part))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    results.extend(records);

    // 2. 按日期统计记录数
    results.push("\n=== 按日期统计 ===".to_string());
    let sql2 = "
        SELECT substr(captured_at, 1, 10) as date_part, COUNT(*) as cnt
        FROM screen_contexts
        GROUP BY date_part
        ORDER BY date_part DESC
        LIMIT 7
    ";
    let mut stmt2 = conn.prepare(sql2).map_err(|e| e.to_string())?;
    let stats: Vec<String> = stmt2
        .query_map([], |row| {
            let date_part: String = row.get(0)?;
            let cnt: i64 = row.get(1)?;
            Ok(format!("日期 '{}': {} 条记录", date_part, cnt))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    results.extend(stats);

    // 3. 如果提供了日期，检查该日期的数据
    if let Some(query_date) = date {
        results.push(format!("\n=== 查询日期 '{}' ===", query_date));
        let sql3 = "
            SELECT id, captured_at, substr(captured_at, 1, 10) as date_part
            FROM screen_contexts
            WHERE substr(captured_at, 1, 10) = ?
            ORDER BY captured_at DESC
            LIMIT 5
        ";
        let mut stmt3 = conn.prepare(sql3).map_err(|e| e.to_string())?;
        let filtered: Vec<String> = stmt3
            .query_map([&query_date], |row| {
                let id: i64 = row.get(0)?;
                let captured_at: String = row.get(1)?;
                let date_part: String = row.get(2)?;
                Ok(format!("ID:{}, time:'{}', date:'{}'", id, captured_at, date_part))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        if filtered.is_empty() {
            results.push("该日期无数据".to_string());
        } else {
            results.extend(filtered);
        }
    }

    Ok(results.join("\n"))
}

// === 活动窗口信息 ===

/// 活动窗口信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveWindowResponse {
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub process_name: Option<String>,
}

/// 获取当前活动窗口信息
#[tauri::command]
pub async fn get_active_window_info() -> Result<ActiveWindowResponse, String> {
    #[cfg(windows)]
    {
        use winapi::um::winuser::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};
        use winapi::um::processthreadsapi::OpenProcess;
        use winapi::um::psapi::GetModuleBaseNameW;
        use winapi::um::handleapi::CloseHandle;
        use winapi::um::winnt::PROCESS_QUERY_INFORMATION;

        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return Ok(ActiveWindowResponse {
                    app_name: None,
                    window_title: None,
                    process_name: None,
                });
            }

            // 获取窗口标题
            let mut title: [u16; 512] = [0; 512];
            let len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
            let window_title = if len > 0 {
                Some(String::from_utf16_lossy(&title[..len as usize]))
            } else {
                None
            };

            // 获取进程ID
            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut process_id);

            // 获取进程名
            let process_name = if process_id > 0 {
                let handle = OpenProcess(PROCESS_QUERY_INFORMATION | 0x0010, 0, process_id); // 0x0010 = PROCESS_VM_READ
                if !handle.is_null() {
                    let mut name: [u16; 260] = [0; 260];
                    let len = GetModuleBaseNameW(handle, std::ptr::null_mut(), name.as_mut_ptr(), name.len() as u32);
                    CloseHandle(handle);
                    if len > 0 {
                        Some(String::from_utf16_lossy(&name[..len as usize]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // 从进程名推断应用名
            let app_name = process_name.as_ref().map(|name| {
                let name_lower = name.to_lowercase();
                if name_lower.contains("code") {
                    "VS Code".to_string()
                } else if name_lower.contains("chrome") {
                    "Chrome".to_string()
                } else if name_lower.contains("firefox") {
                    "Firefox".to_string()
                } else if name_lower.contains("edge") || name_lower.contains("msedge") {
                    "Edge".to_string()
                } else if name_lower.contains("idea") || name_lower.contains("idea64") {
                    "IntelliJ IDEA".to_string()
                } else if name_lower.contains("webstorm") {
                    "WebStorm".to_string()
                } else if name_lower.contains("rider") {
                    "Rider".to_string()
                } else if name_lower.contains("figma") {
                    "Figma".to_string()
                } else if name_lower.contains("notion") {
                    "Notion".to_string()
                } else if name_lower.contains("postman") {
                    "Postman".to_string()
                } else if name_lower.contains("datagrip") {
                    "DataGrip".to_string()
                } else if name_lower.contains("navicat") {
                    "Navicat".to_string()
                } else if name_lower.contains("terminal") || name_lower.contains("cmd") || name_lower.contains("powershell") || name_lower.contains("windowsterminal") {
                    "Terminal".to_string()
                } else if name_lower.contains("sourcetree") {
                    "SourceTree".to_string()
                } else if name_lower.contains("wechat") || name_lower.contains("weixin") {
                    "微信".to_string()
                } else if name_lower.contains("qq") {
                    "QQ".to_string()
                } else if name_lower.contains("dingtalk") {
                    "钉钉".to_string()
                } else if name_lower.contains("slack") {
                    "Slack".to_string()
                } else if name_lower.contains("discord") {
                    "Discord".to_string()
                } else if name_lower.contains("steam") {
                    "Steam".to_string()
                } else if name_lower.contains("spotify") {
                    "Spotify".to_string()
                } else if name_lower.contains("bilibili") {
                    "哔哩哔哩".to_string()
                } else if name_lower.contains("youku") {
                    "优酷".to_string()
                } else if name_lower.contains("iqiyi") {
                    "爱奇艺".to_string()
                } else if name_lower.contains("tiktok") || name_lower.contains("douyin") {
                    "抖音".to_string()
                } else if name_lower.contains("explorer") {
                    "文件资源管理器".to_string()
                } else if name_lower.contains("游戏") || name_lower.contains("game") {
                    name.trim_end_matches(".exe").to_string()
                } else {
                    // 去掉.exe后缀，首字母大写
                    let clean_name = name.trim_end_matches(".exe").trim_end_matches(".EXE");
                    let mut chars: Vec<char> = clean_name.chars().collect();
                    if !chars.is_empty() {
                        chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
                    }
                    chars.iter().collect()
                }
            });

            Ok(ActiveWindowResponse {
                app_name,
                window_title,
                process_name,
            })
        }
    }

    #[cfg(not(windows))]
    {
        Ok(ActiveWindowResponse {
            app_name: Some("Unknown".to_string()),
            window_title: None,
            process_name: None,
        })
    }
}

/// 运行中的应用信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct RunningApp {
    pub name: String,
    pub process_name: String,
}

/// 获取当前运行的应用列表（用于白名单选择）
#[tauri::command]
pub async fn get_running_apps() -> Result<Vec<RunningApp>, String> {
    #[cfg(windows)]
    {
        use std::collections::{HashSet, HashMap};
        use winapi::um::processthreadsapi::OpenProcess;
        use winapi::um::psapi::{EnumProcesses, GetModuleBaseNameW};
        use winapi::um::handleapi::CloseHandle;
        use winapi::um::winnt::PROCESS_QUERY_INFORMATION;

        let mut apps: Vec<RunningApp> = Vec::new();
        let mut seen_processes: HashSet<String> = HashSet::new();
        let mut seen_app_names: HashMap<String, String> = HashMap::new(); // app_name_lower -> process_name

        unsafe {
            let mut process_ids: [u32; 2048] = [0; 2048];
            let mut bytes_returned: u32 = 0;

            if EnumProcesses(
                process_ids.as_mut_ptr(),
                (process_ids.len() * std::mem::size_of::<u32>()) as u32,
                &mut bytes_returned,
            ) == 0
            {
                return Err("Failed to enumerate processes".to_string());
            }

            let num_processes = bytes_returned as usize / std::mem::size_of::<u32>();

            for i in 0..num_processes {
                let pid = process_ids[i];
                if pid == 0 {
                    continue;
                }

                let handle = OpenProcess(PROCESS_QUERY_INFORMATION | 0x0010, 0, pid);
                if handle.is_null() {
                    continue;
                }

                let mut name: [u16; 260] = [0; 260];
                let len = GetModuleBaseNameW(handle, std::ptr::null_mut(), name.as_mut_ptr(), name.len() as u32);
                CloseHandle(handle);

                if len > 0 {
                    let process_name = String::from_utf16_lossy(&name[..len as usize]);
                    let process_lower = process_name.to_lowercase();

                    // 跳过已添加的进程
                    if seen_processes.contains(&process_lower) {
                        continue;
                    }

                    // 扩展的系统进程过滤列表
                    let skip_processes = [
                        // 核心系统进程
                        "system", "svchost", "csrss", "wininit", "services", "lsass",
                        "smss", "dwm", "conhost", "fontdrvhost", "sihost", "taskhostw",
                        "explorer", "searchhost", "runtimebroker", "applicationframehost",
                        "shellexperiencehost", "startmenuexperiencehost", "textinputhost",
                        "ctfmon", "dllhost", "audiodg", "spoolsv", "searchindexer",
                        "securityhealthservice", "sgrmbroker", "registry", "memory compression",
                        "system idle process", "ntoskrnl", "wudfhost", "wmiprvse",
                        "searchprotocolhost", "searchfilterhost", "gameinputsvc",
                        // 额外的系统和后台进程
                        "nvidia", "amd", "intel", "realtek", "logitech", "razer",
                        "msmpeng", "antimalware", "defender", "windows security",
                        "crashpad", "helper", "renderer", "gpu-process", "utility",
                        "broker", "host", "agent", "service", "daemon", "worker",
                        "update", "updater", "installer", "setup", "unins",
                        "systray", "tray", "widget", "sidebar", "gadget",
                        "backgroundtask", "background", "sync", "indexer",
                        "phone", "yourphone", "gamebar", "xbox", "gamemode",
                        "cortana", "widgets", "news", "weather", "clock",
                        "print", "fax", "bluetooth", "wifi", "network",
                        "tauri", "dev-assistant", // 排除自己
                    ];

                    let should_skip = skip_processes.iter().any(|&s| process_lower.contains(s));
                    if should_skip {
                        continue;
                    }

                    // 跳过短名称（通常是系统组件）
                    if process_lower.len() < 4 {
                        continue;
                    }

                    // 映射到应用名
                    let app_name = map_process_to_app_name(&process_name);
                    let app_name_lower = app_name.to_lowercase();

                    // 按应用名去重（同一应用可能有多个进程）
                    if seen_app_names.contains_key(&app_name_lower) {
                        seen_processes.insert(process_lower);
                        continue;
                    }

                    seen_processes.insert(process_lower.clone());
                    seen_app_names.insert(app_name_lower, process_lower);
                    apps.push(RunningApp {
                        name: app_name,
                        process_name: process_name,
                    });
                }
            }
        }

        // 按应用名排序
        apps.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(apps)
    }

    #[cfg(not(windows))]
    {
        Ok(vec![])
    }
}

/// 将进程名映射到友好的应用名
fn map_process_to_app_name(process_name: &str) -> String {
    let name_lower = process_name.to_lowercase();

    if name_lower.contains("code") && !name_lower.contains("unicode") {
        "VS Code".to_string()
    } else if name_lower.contains("chrome") {
        "Chrome".to_string()
    } else if name_lower.contains("firefox") {
        "Firefox".to_string()
    } else if name_lower.contains("edge") || name_lower.contains("msedge") {
        "Edge".to_string()
    } else if name_lower.contains("idea") || name_lower.contains("idea64") {
        "IntelliJ IDEA".to_string()
    } else if name_lower.contains("webstorm") {
        "WebStorm".to_string()
    } else if name_lower.contains("rider") {
        "Rider".to_string()
    } else if name_lower.contains("figma") {
        "Figma".to_string()
    } else if name_lower.contains("notion") {
        "Notion".to_string()
    } else if name_lower.contains("postman") {
        "Postman".to_string()
    } else if name_lower.contains("datagrip") {
        "DataGrip".to_string()
    } else if name_lower.contains("navicat") {
        "Navicat".to_string()
    } else if name_lower.contains("terminal") || name_lower.contains("windowsterminal") {
        "Terminal".to_string()
    } else if name_lower.contains("cmd") {
        "命令提示符".to_string()
    } else if name_lower.contains("powershell") {
        "PowerShell".to_string()
    } else if name_lower.contains("sourcetree") {
        "SourceTree".to_string()
    } else if name_lower.contains("wechat") || name_lower.contains("weixin") {
        "微信".to_string()
    } else if name_lower == "qq.exe" || name_lower.starts_with("qq") {
        "QQ".to_string()
    } else if name_lower.contains("dingtalk") {
        "钉钉".to_string()
    } else if name_lower.contains("slack") {
        "Slack".to_string()
    } else if name_lower.contains("discord") {
        "Discord".to_string()
    } else if name_lower.contains("steam") {
        "Steam".to_string()
    } else if name_lower.contains("spotify") {
        "Spotify".to_string()
    } else if name_lower.contains("typora") {
        "Typora".to_string()
    } else if name_lower.contains("obsidian") {
        "Obsidian".to_string()
    } else if name_lower.contains("notepad++") || name_lower.contains("notepad") {
        "Notepad".to_string()
    } else if name_lower.contains("word") || name_lower.contains("winword") {
        "Word".to_string()
    } else if name_lower.contains("excel") {
        "Excel".to_string()
    } else if name_lower.contains("powerpoint") || name_lower.contains("powerpnt") {
        "PowerPoint".to_string()
    } else if name_lower.contains("outlook") {
        "Outlook".to_string()
    } else if name_lower.contains("teams") {
        "Teams".to_string()
    } else if name_lower.contains("zoom") {
        "Zoom".to_string()
    } else if name_lower.contains("cursor") {
        "Cursor".to_string()
    } else if name_lower.contains("sublime") {
        "Sublime Text".to_string()
    } else if name_lower.contains("atom") {
        "Atom".to_string()
    } else if name_lower.contains("vim") || name_lower.contains("nvim") || name_lower.contains("gvim") {
        "Vim".to_string()
    } else {
        // 去掉.exe后缀，首字母大写
        let clean_name = process_name.trim_end_matches(".exe").trim_end_matches(".EXE");
        let mut chars: Vec<char> = clean_name.chars().collect();
        if !chars.is_empty() {
            chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
        }
        chars.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_thumbnail() {
        // 创建一个简单的测试图片
        use image::{ImageBuffer, Rgba};

        let img = ImageBuffer::from_fn(800, 600, |x, y| {
            Rgba([
                (x % 256) as u8,
                (y % 256) as u8,
                128,
                255,
            ])
        });

        // 编码为PNG
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();

        // 生成缩略图
        let result = generate_thumbnail_data(&buffer);
        assert!(result.is_ok());

        let (thumbnail_base64, thumbnail_bytes) = result.unwrap();
        assert!(!thumbnail_base64.is_empty());
        assert!(!thumbnail_bytes.is_empty());
    }

    #[test]
    fn test_screenshot_list_query_serialization() {
        let query = ScreenshotListQuery {
            start_date: Some("2024-12-01".to_string()),
            end_date: Some("2024-12-04".to_string()),
            app_name: Some("VS Code".to_string()),
            activity_type: Some("coding".to_string()),
            page: Some(1),
            page_size: Some(20),
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("startDate"));
        assert!(json.contains("2024-12-01"));
    }

    #[test]
    fn test_activity_group_serialization() {
        let screenshots = vec![
            ScreenshotResource {
                id: 1,
                path: Some("/path/to/screenshot1.png".to_string()),
                captured_at: "2024-12-04T10:15:30".to_string(),
                app_name: Some("VS Code".to_string()),
                description: Some("编写代码".to_string()),
            },
            ScreenshotResource {
                id: 2,
                path: Some("/path/to/screenshot2.png".to_string()),
                captured_at: "2024-12-04T10:25:30".to_string(),
                app_name: Some("Chrome".to_string()),
                description: Some("浏览文档".to_string()),
            },
        ];

        let activity = ActivityGroup {
            id: "2024-12-04T10:00:00".to_string(),
            start_time: "2024-12-04T10:15:30".to_string(),
            end_time: "2024-12-04T10:55:30".to_string(),
            title: "10:00 - 10:59".to_string(),
            description: "编写代码; 浏览文档".to_string(),
            activity_type: "coding".to_string(),
            screenshots,
        };

        let json = serde_json::to_string(&activity).unwrap();
        assert!(json.contains("startTime"));
        assert!(json.contains("endTime"));
        assert!(json.contains("activityType"));
        assert!(json.contains("screenshots"));
        assert!(json.contains("capturedAt"));
        assert!(json.contains("appName"));
    }
}
