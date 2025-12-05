// Notification Commands
// 通知命令 - 通知中心的所有 Tauri 命令

use crate::db::connection::DbConnection;
use crate::prompts::report_prompts::{DAILY_REPORT_PROMPT, WEEKLY_REPORT_PROMPT};
use crate::services::activity_summary_service::ActivitySummaryService;
use crate::services::ai_service::{AiService, ChatMessage};
use crate::services::context_store_service::ContextStoreService;
use crate::services::notification_service::{Notification, NotificationService, NotificationSettings};
use crate::services::work_log_service::WorkLogService;
use chrono::{Local, Duration as ChronoDuration};
use tauri::State;

/// 获取通知列表
#[tauri::command]
pub fn notification_list(
    db: State<DbConnection>,
    notification_type: Option<String>,
    is_read: Option<bool>,
    limit: Option<i32>,
) -> Result<Vec<Notification>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::list(
        &conn,
        notification_type.as_deref(),
        is_read,
        limit,
    )
    .map_err(|e| e.to_string())
}

/// 获取未读通知数量
#[tauri::command]
pub fn notification_get_unread_count(db: State<DbConnection>) -> Result<i32, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::get_unread_count(&conn).map_err(|e| e.to_string())
}

/// 标记通知已读
#[tauri::command]
pub fn notification_mark_read(db: State<DbConnection>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::mark_read(&conn, id).map_err(|e| e.to_string())
}

/// 标记所有通知已读
#[tauri::command]
pub fn notification_mark_all_read(db: State<DbConnection>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::mark_all_read(&conn).map_err(|e| e.to_string())
}

/// 删除通知
#[tauri::command]
pub fn notification_delete(db: State<DbConnection>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::delete(&conn, id).map_err(|e| e.to_string())
}

/// 清空所有通知
#[tauri::command]
pub fn notification_clear_all(db: State<DbConnection>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::clear_all(&conn).map_err(|e| e.to_string())
}

/// 获取通知设置
#[tauri::command]
pub fn notification_get_settings(db: State<DbConnection>) -> Result<NotificationSettings, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::get_settings(&conn).map_err(|e| e.to_string())
}

/// 更新通知设置
#[tauri::command]
pub fn notification_update_settings(
    db: State<DbConnection>,
    settings: NotificationSettings,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::update_settings(&conn, &settings).map_err(|e| e.to_string())
}

/// 生成日报（手动触发）
/// 基于活动摘要（截图回顾）生成日报，与工作日志（任务面板）区分开
#[tauri::command]
pub async fn notification_generate_daily_report(
    db: State<'_, DbConnection>,
) -> Result<i64, String> {
    // 获取昨天的日期
    let yesterday = (Local::now() - ChronoDuration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    // 收集数据（在作用域内获取锁并释放）
    let (ai_config, activity_summaries, stats) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;

        let ai_config = AiService::load_config(&conn)
            .map_err(|e| format!("加载AI配置失败: {}", e))?;

        // 获取数据库路径用于创建 ActivitySummaryService
        let db_path = conn.path().map(|p| p.to_string()).unwrap_or_default();

        // 从活动摘要表获取当天的所有活动摘要（每15分钟/每小时的汇总）
        let activity_service = ActivitySummaryService::new(db_path);
        let activity_summaries = activity_service
            .get_summaries_by_date(&yesterday)
            .unwrap_or_default();

        // 获取统计数据（仍从screen_contexts获取，用于补充统计信息）
        let stats = ContextStoreService::get_day_stats(&conn, &yesterday).ok();

        (ai_config, activity_summaries, stats)
    };

    // 构建基于活动摘要的输入数据
    let input_data = serde_json::json!({
        "date": yesterday,
        "source": "screenshot_review",  // 标记数据来源是截图回顾
        "activitySummaries": activity_summaries.iter().map(|s| {
            serde_json::json!({
                "timeRange": format!("{} - {}", s.start_time, s.end_time),
                "summaryText": s.summary_text,
                "activityType": s.activity_type,
                "screenshotCount": s.screenshot_count,
                "mainApps": s.main_apps
            })
        }).collect::<Vec<_>>(),
        "summaryCount": activity_summaries.len(),
        "statistics": stats.as_ref().map(|s| serde_json::json!({
            "totalCount": s.total_count,
            "appDistribution": s.app_distribution,
            "activityDistribution": s.activity_distribution,
            "timeRange": s.time_range
        }))
    });

    // 尝试使用AI生成日报
    let summary = if ai_config.is_valid() {
        let ai_service = AiService::new(ai_config);

        let messages = vec![
            ChatMessage::system(DAILY_REPORT_PROMPT.to_string()),
            ChatMessage::user(input_data.to_string()),
        ];

        match ai_service.chat(messages).await {
            Ok(response) => {
                // 尝试解析并格式化AI返回的JSON
                format_daily_report_response(&response)
            }
            Err(e) => {
                log::warn!("AI生成日报失败，使用回退方案: {}", e);
                generate_simple_daily_report_from_summaries(&yesterday, &activity_summaries, &stats)
            }
        }
    } else {
        log::info!("AI未配置，使用简单日报模板");
        generate_simple_daily_report_from_summaries(&yesterday, &activity_summaries, &stats)
    };

    // 创建通知
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::generate_daily_report_notification(&conn, &yesterday, &summary)
        .map_err(|e| e.to_string())
}

// 格式化AI返回的日报响应
fn format_daily_report_response(response: &str) -> String {
    // 尝试解析JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
        let mut content = String::new();

        // 添加总结
        if let Some(summary) = json.get("summary").and_then(|v| v.as_str()) {
            content.push_str(&format!("## 📊 工作总结\n{}\n\n", summary));
        }

        // 添加亮点
        if let Some(highlights) = json.get("highlights").and_then(|v| v.as_array()) {
            content.push_str("## ✨ 工作亮点\n");
            for h in highlights {
                if let (Some(time), Some(title)) = (
                    h.get("time").and_then(|v| v.as_str()),
                    h.get("title").and_then(|v| v.as_str())
                ) {
                    content.push_str(&format!("- **{}**: {}\n", time, title));
                    if let Some(c) = h.get("content").and_then(|v| v.as_str()) {
                        content.push_str(&format!("  {}\n", c));
                    }
                }
            }
            content.push('\n');
        }

        // 添加洞察
        if let Some(insights) = json.get("insights").and_then(|v| v.as_array()) {
            content.push_str("## 💡 工作洞察\n");
            for i in insights {
                if let Some(c) = i.get("content").and_then(|v| v.as_str()) {
                    content.push_str(&format!("- {}\n", c));
                }
            }
            content.push('\n');
        }

        // 添加建议
        if let Some(recs) = json.get("recommendations").and_then(|v| v.as_array()) {
            content.push_str("## 📝 改进建议\n");
            for r in recs {
                if let Some(s) = r.as_str() {
                    content.push_str(&format!("- {}\n", s));
                }
            }
        }

        // 如果成功提取了内容，返回格式化后的内容
        if !content.is_empty() {
            return content;
        }
    }

    // 如果JSON解析失败或没有预期字段，尝试清理响应
    // 检查是否是被```json```包裹的JSON
    let cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // 再次尝试解析清理后的内容
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(cleaned) {
        if let Some(summary) = json.get("summary").and_then(|v| v.as_str()) {
            return format!("## 📊 工作总结\n{}", summary);
        }
        // 如果有任何文本字段，提取它
        if let Some(s) = json.as_str() {
            return s.to_string();
        }
    }

    // 最终回退：如果响应看起来像JSON，不要直接显示
    if response.trim().starts_with('{') || response.trim().starts_with('[') {
        "AI生成的日报内容解析失败，请重试".to_string()
    } else {
        // 如果不是JSON，可能是普通文本，直接返回
        response.to_string()
    }
}

// 简单日报生成（基于活动摘要的回退方案）
fn generate_simple_daily_report_from_summaries(
    date: &str,
    summaries: &[crate::services::activity_summary_service::ActivitySummary],
    stats: &Option<crate::models::screen_context::DayStats>,
) -> String {
    if summaries.is_empty() {
        return format!("# {} 截图回顾日报\n\n当天无活动记录", date);
    }

    let mut content = format!("# {} 截图回顾日报\n\n", date);
    content.push_str(&format!("## 活动时段汇总\n\n共 {} 个活动时段\n\n", summaries.len()));

    // 按时间顺序展示活动摘要
    for summary in summaries {
        let time_range = format!("{} - {}",
            summary.start_time.split(' ').last().unwrap_or(&summary.start_time),
            summary.end_time.split(' ').last().unwrap_or(&summary.end_time)
        );

        content.push_str(&format!("### {}\n", time_range));
        content.push_str(&format!("- **活动类型**: {}\n", summary.activity_type));

        if let Some(ref apps) = summary.main_apps {
            if !apps.is_empty() {
                content.push_str(&format!("- **主要应用**: {}\n", apps));
            }
        }

        if let Some(count) = summary.screenshot_count {
            content.push_str(&format!("- **截图数量**: {}\n", count));
        }

        content.push_str(&format!("- **摘要**: {}\n\n", summary.summary_text));
    }

    // 添加统计信息
    if let Some(s) = stats {
        content.push_str("## 统计数据\n\n");
        content.push_str(&format!("- 总截图数: {}\n", s.total_count));
        if let Some(ref time_range) = s.time_range {
            content.push_str(&format!("- 活动时间范围: {} 至 {}\n", time_range.0, time_range.1));
        }
    }

    content
}

/// 生成周报（手动触发）
#[tauri::command]
pub async fn notification_generate_weekly_report(
    db: State<'_, DbConnection>,
) -> Result<i64, String> {
    let now = Local::now();
    let week_start = now - ChronoDuration::days(7);
    let week_str = format!(
        "{} 至 {}",
        week_start.format("%Y-%m-%d"),
        now.format("%Y-%m-%d")
    );

    // 收集一周的数据
    let (ai_config, daily_data) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;

        let ai_config = AiService::load_config(&conn)
            .map_err(|e| format!("加载AI配置失败: {}", e))?;

        let mut daily_data = Vec::new();
        for i in 0..7 {
            let date = (week_start + ChronoDuration::days(i))
                .format("%Y-%m-%d")
                .to_string();

            // 获取工作日志
            let log = WorkLogService::get_work_log(&conn, &date).ok().flatten();

            // 获取活动统计
            let stats = ContextStoreService::get_day_stats(&conn, &date).ok();

            // 获取活动数量
            let contexts = ContextStoreService::list_by_date(&conn, &date).unwrap_or_default();

            daily_data.push(serde_json::json!({
                "date": date,
                "summary": log.as_ref().map(|l| l.content.clone()).unwrap_or_default(),
                "activityCount": contexts.len(),
                "statistics": stats
            }));
        }

        (ai_config, daily_data)
    };

    // 构建输入数据
    let input_data = serde_json::json!({
        "weekRange": week_str,
        "dailyReports": daily_data
    });

    // 尝试使用AI生成周报
    let summary = if ai_config.is_valid() {
        let ai_service = AiService::new(ai_config);

        let messages = vec![
            ChatMessage::system(WEEKLY_REPORT_PROMPT.to_string()),
            ChatMessage::user(input_data.to_string()),
        ];

        match ai_service.chat(messages).await {
            Ok(response) => {
                // 尝试解析并格式化AI返回的JSON
                format_weekly_report_response(&response)
            }
            Err(e) => {
                log::warn!("AI生成周报失败，使用回退方案: {}", e);
                generate_simple_weekly_report(&week_str, &daily_data)
            }
        }
    } else {
        log::info!("AI未配置，使用简单周报模板");
        generate_simple_weekly_report(&week_str, &daily_data)
    };

    // 创建通知
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    NotificationService::generate_weekly_report_notification(&conn, &week_str, &summary)
        .map_err(|e| e.to_string())
}

// 格式化AI返回的周报响应
fn format_weekly_report_response(response: &str) -> String {
    // 尝试解析JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
        let mut content = String::new();

        // 添加周总结
        if let Some(week_summary) = json.get("weekSummary").and_then(|v| v.as_str()) {
            content.push_str(&format!("## 📊 周度总结\n{}\n\n", week_summary));
        }

        // 添加关键成果
        if let Some(achievements) = json.get("keyAchievements").and_then(|v| v.as_array()) {
            content.push_str("## 🏆 关键成果\n");
            for a in achievements {
                if let Some(category) = a.get("category").and_then(|v| v.as_str()) {
                    content.push_str(&format!("\n### {}\n", category));
                    if let Some(items) = a.get("items").and_then(|v| v.as_array()) {
                        for item in items {
                            if let Some(s) = item.as_str() {
                                content.push_str(&format!("- {}\n", s));
                            }
                        }
                    }
                }
            }
            content.push('\n');
        }

        // 添加周洞察
        if let Some(insights) = json.get("weeklyInsights").and_then(|v| v.as_array()) {
            content.push_str("## 💡 本周洞察\n");
            for i in insights {
                if let Some(s) = i.as_str() {
                    content.push_str(&format!("- {}\n", s));
                }
            }
            content.push('\n');
        }

        // 添加下周计划
        if let Some(plans) = json.get("nextWeekPlan").and_then(|v| v.as_array()) {
            content.push_str("## 📝 下周计划\n");
            for p in plans {
                if let Some(s) = p.as_str() {
                    content.push_str(&format!("- {}\n", s));
                }
            }
        }

        // 如果成功提取了内容，返回格式化后的内容
        if !content.is_empty() {
            return content;
        }
    }

    // 如果JSON解析失败，尝试清理响应（可能被```json```包裹）
    let cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // 再次尝试解析清理后的内容
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(cleaned) {
        if let Some(summary) = json.get("weekSummary").and_then(|v| v.as_str()) {
            return format!("## 📊 周度总结\n{}", summary);
        }
    }

    // 最终回退：如果响应看起来像JSON，不要直接显示
    if response.trim().starts_with('{') || response.trim().starts_with('[') {
        "AI生成的周报内容解析失败，请重试".to_string()
    } else {
        // 如果不是JSON，可能是普通文本，直接返回
        response.to_string()
    }
}

// 简单周报生成（回退方案）
fn generate_simple_weekly_report(
    week_str: &str,
    daily_data: &[serde_json::Value],
) -> String {
    let mut summary = format!("# {} 工作周报\n\n", week_str);

    for day in daily_data {
        if let Some(date) = day.get("date").and_then(|v| v.as_str()) {
            let day_summary = day.get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or("无记录");
            let count = day.get("activityCount")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            if count > 0 || !day_summary.is_empty() {
                summary.push_str(&format!("## {}\n", date));
                if count > 0 {
                    summary.push_str(&format!("活动次数: {}\n", count));
                }
                if !day_summary.is_empty() && day_summary != "无记录" {
                    summary.push_str(&format!("{}\n", day_summary));
                }
                summary.push('\n');
            }
        }
    }

    summary
}

/// 生成休息提醒（手动触发）
#[tauri::command]
pub fn notification_generate_tip(
    db: State<DbConnection>,
    work_minutes: Option<i32>,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let minutes = work_minutes.unwrap_or(60);
    NotificationService::generate_tip(&conn, minutes).map_err(|e| e.to_string())
}

/// 检查并生成定时通知（由后台任务调用）
#[tauri::command]
pub async fn notification_check_and_generate(db: State<'_, DbConnection>) -> Result<(), String> {
    let should_daily = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        NotificationService::should_generate_daily_report(&conn).map_err(|e| e.to_string())?
    };

    if should_daily {
        notification_generate_daily_report(db.clone()).await?;
        return Ok(());
    }

    let should_weekly = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        NotificationService::should_generate_weekly_report(&conn).map_err(|e| e.to_string())?
    };

    if should_weekly {
        notification_generate_weekly_report(db.clone()).await?;
        return Ok(());
    }

    Ok(())
}
