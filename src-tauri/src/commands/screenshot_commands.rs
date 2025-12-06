// Screenshot Commands
// 截图回顾相关的 Tauri 命令

use crate::db::connection::DbConnection;
use crate::models::screen_context::ScreenContext;
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
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

    // 如果需要生成缩略图，先检查缓存
    if generate_thumbnail.unwrap_or(false) {
        // 计算缓存文件路径
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("dev-assistant")
            .join("thumbnails");

        // 创建缓存目录
        let _ = std::fs::create_dir_all(&cache_dir);

        // 使用文件路径哈希作为缓存文件名
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
                        // 读取缓存的缩略图
                        if let Ok(cached_data) = std::fs::read(&cache_path) {
                            return Ok(general_purpose::STANDARD.encode(&cached_data));
                        }
                    }
                }
            }
        }

        // 读取原文件
        let image_data = std::fs::read(path)
            .map_err(|e| format!("读取截图文件失败: {}", e))?;

        // 生成缩略图
        match generate_thumbnail_data(&image_data) {
            Ok((thumbnail_base64, thumbnail_bytes)) => {
                // 写入缓存（异步，不阻塞返回）
                let cache_path_clone = cache_path.clone();
                std::thread::spawn(move || {
                    let _ = std::fs::write(cache_path_clone, thumbnail_bytes);
                });
                return Ok(thumbnail_base64);
            }
            Err(e) => {
                eprintln!("生成缩略图失败，返回原图: {}", e);
                // 如果生成缩略图失败，返回原图
                return Ok(general_purpose::STANDARD.encode(&image_data));
            }
        }
    }

    // 读取文件
    let image_data = std::fs::read(path)
        .map_err(|e| format!("读取截图文件失败: {}", e))?;

    // 直接返回base64编码的原图
    Ok(general_purpose::STANDARD.encode(&image_data))
}

/// 生成缩略图数据，返回 (base64编码, 原始字节)
fn generate_thumbnail_data(image_data: &[u8]) -> Result<(String, Vec<u8>)> {
    use image::GenericImageView;

    // 解码图片
    let img = image::load_from_memory(image_data)
        .context("无法解码图片")?;

    // 计算缩略图尺寸（最大宽度或高度为300px，更小更快）
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

    // 生成缩略图（使用 Triangle 算法，比 Lanczos3 快很多）
    let thumbnail = img.resize(
        thumb_width,
        thumb_height,
        image::imageops::FilterType::Triangle,
    );

    // 编码为JPEG格式（更小的文件大小，质量75%足够缩略图）
    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);

    // 使用 JpegEncoder 设置质量
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 75);
    thumbnail.write_with_encoder(encoder)
        .context("无法编码缩略图")?;

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

    // 1. 先查询该日期已有的小时总结（来自 activity_summaries 表）
    let mut hourly_summaries: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();
    {
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
        log::info!("找到 {} 个已有的小时总结", hourly_summaries.len());
    }

    // 2. 查询截图数据
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

    // 3. 处理每个小时的数据，结合已有的小时总结
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
