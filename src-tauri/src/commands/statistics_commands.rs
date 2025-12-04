// Statistics Commands
// 数据统计相关的 Tauri 命令

use crate::db::connection::DbConnection;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

/// 热力图数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapData {
    pub date: String,
    pub count: u32,
}

/// 每日趋势数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyTrendData {
    pub date: String,
    pub total_count: u32,
    pub coding_count: u32,
    pub browsing_count: u32,
    pub document_count: u32,
    pub other_count: u32,
}

/// 时段分布数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyDistribution {
    pub hour: u8,
    pub count: u32,
}

/// 应用使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUsageStats {
    pub app_name: String,
    pub count: u32,
    pub percentage: f32,
}

/// 活动类型统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityTypeStats {
    pub activity_type: String,
    pub count: u32,
    pub percentage: f32,
}

/// 获取热力图数据
///
/// 返回指定日期范围内每天的活动数量
///
/// # 参数
/// * `start_date` - 开始日期 (YYYY-MM-DD)，可选，默认为3个月前
/// * `end_date` - 结束日期 (YYYY-MM-DD)，可选，默认为今天
///
/// # 返回
/// 返回每天的活动数量列表
#[tauri::command]
pub async fn statistics_get_heatmap(
    start_date: Option<String>,
    end_date: Option<String>,
    db: State<'_, DbConnection>,
) -> Result<Vec<HeatmapData>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 默认查询最近3个月
    let start = start_date.unwrap_or_else(|| {
        let now = chrono::Local::now();
        (now - chrono::Duration::days(90)).format("%Y-%m-%d").to_string()
    });

    let end = end_date.unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    });

    let mut stmt = conn.prepare(
        "SELECT date(captured_at) as date, COUNT(*) as count
         FROM screen_contexts
         WHERE date(captured_at) BETWEEN ? AND ?
         GROUP BY date(captured_at)
         ORDER BY date ASC"
    ).map_err(|e| e.to_string())?;

    let heatmap_data = stmt.query_map(params![start, end], |row| {
        Ok(HeatmapData {
            date: row.get(0)?,
            count: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(heatmap_data)
}

/// 获取每日趋势数据
///
/// 返回最近N天的统计数据，按活动类型分组
///
/// # 参数
/// * `days` - 查询天数，默认7天
///
/// # 返回
/// 返回每天按活动类型分组的统计数据
#[tauri::command]
pub async fn statistics_get_daily_trend(
    days: Option<u32>,
    db: State<'_, DbConnection>,
) -> Result<Vec<DailyTrendData>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let days_count = days.unwrap_or(7);

    // 查询每天的总数和各类型数量
    let mut stmt = conn.prepare(
        "SELECT
            date(captured_at) as date,
            COUNT(*) as total_count,
            SUM(CASE WHEN activity_type = 'coding' THEN 1 ELSE 0 END) as coding_count,
            SUM(CASE WHEN activity_type = 'browsing' THEN 1 ELSE 0 END) as browsing_count,
            SUM(CASE WHEN activity_type = 'document' THEN 1 ELSE 0 END) as document_count,
            SUM(CASE WHEN activity_type NOT IN ('coding', 'browsing', 'document') THEN 1 ELSE 0 END) as other_count
         FROM screen_contexts
         WHERE date(captured_at) >= date('now', '-' || ? || ' days')
         GROUP BY date(captured_at)
         ORDER BY date DESC"
    ).map_err(|e| e.to_string())?;

    let trend_data = stmt.query_map(params![days_count], |row| {
        Ok(DailyTrendData {
            date: row.get(0)?,
            total_count: row.get(1)?,
            coding_count: row.get(2)?,
            browsing_count: row.get(3)?,
            document_count: row.get(4)?,
            other_count: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(trend_data)
}

/// 获取时段分布数据
///
/// 返回指定日期24小时的活动分布
///
/// # 参数
/// * `date` - 日期 (YYYY-MM-DD)，可选，默认为今天
///
/// # 返回
/// 返回24小时的活动分布数据
#[tauri::command]
pub async fn statistics_get_hourly_distribution(
    date: Option<String>,
    db: State<'_, DbConnection>,
) -> Result<Vec<HourlyDistribution>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let target_date = date.unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    });

    let mut stmt = conn.prepare(
        "SELECT
            CAST(strftime('%H', captured_at) AS INTEGER) as hour,
            COUNT(*) as count
         FROM screen_contexts
         WHERE date(captured_at) = ?
         GROUP BY hour
         ORDER BY hour ASC"
    ).map_err(|e| e.to_string())?;

    let hourly_data = stmt.query_map(params![target_date], |row| {
        Ok(HourlyDistribution {
            hour: row.get(0)?,
            count: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(hourly_data)
}

/// 获取应用使用统计
///
/// 返回指定日期范围内的应用使用占比
///
/// # 参数
/// * `start_date` - 开始日期 (YYYY-MM-DD)，可选，默认为7天前
/// * `end_date` - 结束日期 (YYYY-MM-DD)，可选，默认为今天
/// * `limit` - 返回前N个应用，默认10
///
/// # 返回
/// 返回应用使用统计列表，按使用次数降序排列
#[tauri::command]
pub async fn statistics_get_app_usage(
    start_date: Option<String>,
    end_date: Option<String>,
    limit: Option<u32>,
    db: State<'_, DbConnection>,
) -> Result<Vec<AppUsageStats>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let start = start_date.unwrap_or_else(|| {
        let now = chrono::Local::now();
        (now - chrono::Duration::days(7)).format("%Y-%m-%d").to_string()
    });

    let end = end_date.unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    });

    let limit_count = limit.unwrap_or(10);

    // 先获取总数
    let total_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM screen_contexts
         WHERE date(captured_at) BETWEEN ? AND ? AND app_name IS NOT NULL",
        params![start, end],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    if total_count == 0 {
        return Ok(vec![]);
    }

    // 查询应用统计
    let mut stmt = conn.prepare(
        "SELECT app_name, COUNT(*) as count
         FROM screen_contexts
         WHERE date(captured_at) BETWEEN ? AND ? AND app_name IS NOT NULL
         GROUP BY app_name
         ORDER BY count DESC
         LIMIT ?"
    ).map_err(|e| e.to_string())?;

    let app_stats = stmt.query_map(params![start, end, limit_count], |row| {
        let count: u32 = row.get(1)?;
        let percentage = (count as f32 / total_count as f32) * 100.0;
        Ok(AppUsageStats {
            app_name: row.get(0)?,
            count,
            percentage,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(app_stats)
}

/// 获取活动类型统计
///
/// 返回指定日期范围内的活动类型占比
///
/// # 参数
/// * `start_date` - 开始日期 (YYYY-MM-DD)，可选，默认为7天前
/// * `end_date` - 结束日期 (YYYY-MM-DD)，可选，默认为今天
///
/// # 返回
/// 返回活动类型统计列表，按使用次数降序排列
#[tauri::command]
pub async fn statistics_get_activity_types(
    start_date: Option<String>,
    end_date: Option<String>,
    db: State<'_, DbConnection>,
) -> Result<Vec<ActivityTypeStats>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let start = start_date.unwrap_or_else(|| {
        let now = chrono::Local::now();
        (now - chrono::Duration::days(7)).format("%Y-%m-%d").to_string()
    });

    let end = end_date.unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    });

    // 先获取总数
    let total_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM screen_contexts
         WHERE date(captured_at) BETWEEN ? AND ?",
        params![start, end],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    if total_count == 0 {
        return Ok(vec![]);
    }

    // 查询活动类型统计
    let mut stmt = conn.prepare(
        "SELECT activity_type, COUNT(*) as count
         FROM screen_contexts
         WHERE date(captured_at) BETWEEN ? AND ?
         GROUP BY activity_type
         ORDER BY count DESC"
    ).map_err(|e| e.to_string())?;

    let activity_stats = stmt.query_map(params![start, end], |row| {
        let count: u32 = row.get(1)?;
        let percentage = (count as f32 / total_count as f32) * 100.0;
        Ok(ActivityTypeStats {
            activity_type: row.get(0)?,
            count,
            percentage,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(activity_stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::models::screen_context::ScreenContext;
    use crate::services::context_store_service::ContextStoreService;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 插入测试数据
        for i in 0..10 {
            let context = ScreenContext {
                id: None,
                captured_at: format!("2024-12-{:02} {:02}:00:00", 4 - (i / 3), 10 + i),
                app_name: Some(if i % 3 == 0 { "VS Code" } else if i % 3 == 1 { "Chrome" } else { "Terminal" }.to_string()),
                window_title: None,
                activity_type: if i % 2 == 0 { "coding" } else { "browsing" }.to_string(),
                description: format!("测试活动 {}", i),
                key_content: None,
                screenshot_hash: None,
                screenshot_path: None,
                processing_time_ms: None,
            };
            ContextStoreService::save_context(&conn, &context).unwrap();
        }

        conn
    }

    #[test]
    fn test_heatmap_query() {
        let conn = setup_test_db();

        let mut stmt = conn.prepare(
            "SELECT date(captured_at) as date, COUNT(*) as count
             FROM screen_contexts
             WHERE date(captured_at) BETWEEN '2024-12-01' AND '2024-12-31'
             GROUP BY date(captured_at)
             ORDER BY date ASC"
        ).unwrap();

        let results: Vec<_> = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

        assert!(!results.is_empty());
    }

    #[test]
    fn test_daily_trend_query() {
        let conn = setup_test_db();

        let mut stmt = conn.prepare(
            "SELECT
                date(captured_at) as date,
                COUNT(*) as total_count,
                SUM(CASE WHEN activity_type = 'coding' THEN 1 ELSE 0 END) as coding_count
             FROM screen_contexts
             WHERE date(captured_at) >= date('now', '-7 days')
             GROUP BY date(captured_at)
             ORDER BY date DESC"
        ).unwrap();

        let results: Vec<_> = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

        assert!(!results.is_empty());
    }

    #[test]
    fn test_app_usage_query() {
        let conn = setup_test_db();

        let mut stmt = conn.prepare(
            "SELECT app_name, COUNT(*) as count
             FROM screen_contexts
             WHERE app_name IS NOT NULL
             GROUP BY app_name
             ORDER BY count DESC
             LIMIT 10"
        ).unwrap();

        let results: Vec<_> = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].0, "VS Code"); // VS Code should be the most used
    }
}
