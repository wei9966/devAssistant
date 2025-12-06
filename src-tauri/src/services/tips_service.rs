// Tips Service
// 智能提示生成服务 - 分析活动模式并生成个性化建议

use anyhow::{anyhow, Result};
use chrono::{Local, Duration as ChronoDuration};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ai_service::{AiService, ChatMessage};
use crate::services::prompt_db_service::PromptDbService;

/// 提示类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TipCategory {
    Health,       // 健康提醒
    Productivity, // 生产力建议
    Focus,        // 专注力提醒
}

impl TipCategory {
    pub fn as_str(&self) -> &str {
        match self {
            TipCategory::Health => "health",
            TipCategory::Productivity => "productivity",
            TipCategory::Focus => "focus",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "health" => Some(TipCategory::Health),
            "productivity" => Some(TipCategory::Productivity),
            "focus" => Some(TipCategory::Focus),
            _ => None,
        }
    }
}

/// 提示优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TipPriority {
    Low,
    Medium,
    High,
}

impl TipPriority {
    pub fn as_str(&self) -> &str {
        match self {
            TipPriority::Low => "low",
            TipPriority::Medium => "medium",
            TipPriority::High => "high",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "low" => Some(TipPriority::Low),
            "medium" => Some(TipPriority::Medium),
            "high" => Some(TipPriority::High),
            _ => None,
        }
    }
}

/// 智能提示
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tip {
    pub id: Option<i64>,
    pub content: String,
    pub category: String,
    pub priority: String,
    pub is_read: bool,
    pub created_at: String,
}

/// 活动模式分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityPattern {
    pub continuous_work_minutes: i64,
    pub dominant_activity_type: Option<String>,
    pub dominant_app: Option<String>,
    pub activity_diversity: f64,
    pub total_activities: usize,
    /// 活动类型分布 (activity_type -> count)
    pub activity_distribution: HashMap<String, usize>,
    /// 最近使用的应用列表 (最多5个)
    pub recent_apps: Vec<String>,
    /// 专注领域 (基于关键词分析)
    pub focus_areas: Vec<String>,
}

/// Tips 服务
pub struct TipsService;

impl TipsService {
    /// 分析最近的活动模式
    pub fn analyze_recent_pattern(conn: &Connection, hours: i64) -> Result<ActivityPattern> {
        let since = Local::now() - ChronoDuration::hours(hours);
        let since_str = since.format("%Y-%m-%d %H:%M:%S").to_string();

        // 获取最近的活动记录（包括关键词）
        let mut stmt = conn.prepare(
            "SELECT captured_at, app_name, activity_type, keywords
             FROM screen_contexts
             WHERE captured_at >= ?
             ORDER BY captured_at ASC",
        )?;

        let contexts: Vec<(String, Option<String>, String, Option<String>)> = stmt
            .query_map(params![since_str], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if contexts.is_empty() {
            return Ok(ActivityPattern {
                continuous_work_minutes: 0,
                dominant_activity_type: None,
                dominant_app: None,
                activity_diversity: 0.0,
                total_activities: 0,
                activity_distribution: HashMap::new(),
                recent_apps: Vec::new(),
                focus_areas: Vec::new(),
            });
        }

        // 计算连续工作时长（从第一条记录到现在）
        let first_time = chrono::NaiveDateTime::parse_from_str(&contexts[0].0, "%Y-%m-%d %H:%M:%S")
            .map_err(|e| anyhow!("解析时间失败: {}", e))?;
        let now = Local::now().naive_local();
        let continuous_minutes = (now - first_time).num_minutes();

        // 统计活动类型分布
        let mut activity_counts: HashMap<String, usize> = HashMap::new();
        for (_, _, activity_type, _) in &contexts {
            *activity_counts.entry(activity_type.clone()).or_insert(0) += 1;
        }

        // 找出主要活动类型
        let dominant_activity_type = activity_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(activity, _)| activity.clone());

        // 统计应用分布
        let mut app_counts: HashMap<String, usize> = HashMap::new();
        for (_, app_name, _, _) in &contexts {
            if let Some(app) = app_name {
                *app_counts.entry(app.clone()).or_insert(0) += 1;
            }
        }

        // 找出主要应用
        let dominant_app = app_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(app, _)| app.clone());

        // 获取最近使用的应用列表（按时间倒序，去重，最多5个）
        let mut recent_apps_list = Vec::new();
        let mut seen_apps = std::collections::HashSet::new();
        for (_, app_name, _, _) in contexts.iter().rev() {
            if let Some(app) = app_name {
                if !seen_apps.contains(app) {
                    recent_apps_list.push(app.clone());
                    seen_apps.insert(app.clone());
                    if recent_apps_list.len() >= 5 {
                        break;
                    }
                }
            }
        }

        // 分析关键词，提取专注领域
        let mut keyword_counts: HashMap<String, usize> = HashMap::new();
        for (_, _, _, keywords) in &contexts {
            if let Some(kw_str) = keywords {
                // 假设keywords是JSON数组或逗号分隔的字符串
                if let Ok(kw_array) = serde_json::from_str::<Vec<String>>(kw_str) {
                    for kw in kw_array {
                        *keyword_counts.entry(kw).or_insert(0) += 1;
                    }
                }
            }
        }

        // 提取出现频率最高的关键词作为专注领域（最多5个）
        let mut focus_areas: Vec<(String, usize)> = keyword_counts.into_iter().collect();
        focus_areas.sort_by(|a, b| b.1.cmp(&a.1));
        let focus_areas: Vec<String> = focus_areas
            .into_iter()
            .take(5)
            .map(|(kw, _)| kw)
            .collect();

        // 计算活动多样性（使用香农熵）
        let total = contexts.len() as f64;
        let diversity: f64 = activity_counts
            .values()
            .map(|&count| {
                let p = count as f64 / total;
                -p * p.log2()
            })
            .sum();

        Ok(ActivityPattern {
            continuous_work_minutes: continuous_minutes,
            dominant_activity_type,
            dominant_app,
            activity_diversity: diversity,
            total_activities: contexts.len(),
            activity_distribution: activity_counts,
            recent_apps: recent_apps_list,
            focus_areas,
        })
    }

    /// 生成智能提示（使用AI）
    pub async fn generate_tip_with_ai(
        conn: &Connection,
        ai_service: &AiService,
        pattern: &ActivityPattern,
    ) -> Result<(String, TipCategory, TipPriority)> {
        // 尝试使用新的PromptManager
        match Self::generate_tip_with_prompt_manager(conn, ai_service, pattern).await {
            Ok(result) => Ok(result),
            Err(e) => {
                // 如果PromptManager加载失败，回退到默认提示生成逻辑
                log::warn!("PromptManager加载失败，使用默认提示生成: {}", e);
                Self::generate_tip_fallback(conn, ai_service, pattern).await
            }
        }
    }

    /// 使用数据库提示词生成提示
    async fn generate_tip_with_prompt_manager(
        conn: &Connection,
        ai_service: &AiService,
        pattern: &ActivityPattern,
    ) -> Result<(String, TipCategory, TipPriority)> {
        // 准备变量
        let now = Local::now();
        let mut vars = HashMap::new();
        vars.insert("current_time".to_string(), now.to_rfc3339());
        vars.insert("current_date".to_string(), now.format("%Y-%m-%d").to_string());
        vars.insert("current_timestamp".to_string(), now.format("%Y-%m-%d %H:%M:%S").to_string());

        // 计算分析时间范围
        let start_time = now - ChronoDuration::minutes(pattern.continuous_work_minutes);
        vars.insert("start_time_str".to_string(), start_time.format("%Y-%m-%d %H:%M:%S").to_string());
        vars.insert("end_time_str".to_string(), now.format("%Y-%m-%d %H:%M:%S").to_string());

        // 序列化活动模式信息
        let activity_patterns_info = serde_json::to_string_pretty(pattern)
            .unwrap_or_else(|_| format!("{:?}", pattern));
        vars.insert("activity_patterns_info".to_string(), activity_patterns_info);

        // 获取最近提示历史
        let recent_tips = Self::list_tips(conn, None, Some(5))?;
        let recent_tips_info = if recent_tips.is_empty() {
            "暂无历史提醒".to_string()
        } else {
            recent_tips
                .iter()
                .map(|tip| format!("- [{}] {}", tip.created_at, tip.content))
                .collect::<Vec<_>>()
                .join("\n")
        };
        vars.insert("recent_tips_info".to_string(), recent_tips_info);

        // 获取上下文数据（最近的活动摘要）
        let context_data = Self::get_recent_context_summary(conn, 10)?;
        vars.insert("context_data".to_string(), context_data);

        // 优先从数据库获取提示词
        let messages = match PromptDbService::render_prompt_cached("smart_tip_generation", &vars) {
            Ok(rendered) => {
                log::info!("[智能提示] 使用数据库提示词");
                if let Some(system) = rendered.system {
                    vec![
                        ChatMessage::system(system),
                        ChatMessage::user(rendered.user),
                    ]
                } else {
                    vec![ChatMessage::user(rendered.user)]
                }
            }
            Err(e) => {
                log::warn!("[智能提示] 加载数据库提示词失败，使用简单提示词: {}", e);
                // 使用简单的硬编码 fallback 提示词
                let fallback_system = "你是一个智能的个人助手，负责根据用户活动模式生成有价值的提醒和建议。";
                let fallback_user = format!(
                    "当前时间: {}\n活动模式: {}\n\n请根据用户活动生成一条简洁实用的建议（不超过100字）。如果没有有价值的建议，返回\"暂无重要提醒\"。",
                    vars.get("current_timestamp").unwrap_or(&"未知".to_string()),
                    vars.get("activity_patterns_info").unwrap_or(&"无数据".to_string())
                );
                vec![
                    ChatMessage::system(fallback_system.to_string()),
                    ChatMessage::user(fallback_user),
                ]
            }
        };
        let content = ai_service.chat_with_log(conn, "tips", "generate", messages).await?;

        // 解析响应
        let tip_content = Self::parse_tip_response(&content)?;

        // 根据模式判断类别和优先级
        let (category, priority) = Self::classify_tip(pattern);

        Ok((tip_content, category, priority))
    }

    /// 回退的默认提示生成逻辑
    async fn generate_tip_fallback(
        conn: &Connection,
        ai_service: &AiService,
        pattern: &ActivityPattern,
    ) -> Result<(String, TipCategory, TipPriority)> {
        // 构建提示词
        let prompt = format!(
            "你是一个专业的工作健康和生产力顾问。根据以下用户的工作模式，生成一条简洁、实用的建议（不超过100字）：\n\n\
            - 连续工作时长: {} 分钟\n\
            - 主要活动类型: {}\n\
            - 主要使用应用: {}\n\
            - 活动多样性: {:.2}\n\
            - 总活动次数: {}\n\n\
            请直接返回建议内容，不要包含任何额外的解释或格式。建议应该是积极的、可操作的。",
            pattern.continuous_work_minutes,
            pattern.dominant_activity_type.as_deref().unwrap_or("未知"),
            pattern.dominant_app.as_deref().unwrap_or("未知"),
            pattern.activity_diversity,
            pattern.total_activities
        );

        let messages = vec![ChatMessage::user(prompt)];
        let content = ai_service.chat_with_log(conn, "tips", "generate", messages).await?;

        // 根据模式判断类别和优先级
        let (category, priority) = Self::classify_tip(pattern);

        Ok((content, category, priority))
    }

    /// 解析Markdown格式的Tips响应
    fn parse_tip_response(response: &str) -> Result<String> {
        let trimmed = response.trim();

        // 检查是否为"暂无重要提醒"
        if trimmed.contains("暂无重要提醒") || trimmed.is_empty() {
            return Err(anyhow!("AI未生成有价值的提醒"));
        }

        // 返回完整的Markdown内容
        Ok(trimmed.to_string())
    }

    /// 获取最近活动的上下文摘要
    fn get_recent_context_summary(conn: &Connection, limit: i32) -> Result<String> {
        let mut stmt = conn.prepare(
            "SELECT title, summary, activity_type, captured_at
             FROM screen_contexts
             ORDER BY captured_at DESC
             LIMIT ?",
        )?;

        let contexts: Vec<String> = stmt
            .query_map(params![limit], |row| {
                let title: String = row.get(0)?;
                let summary: String = row.get(1)?;
                let activity_type: String = row.get(2)?;
                let captured_at: String = row.get(3)?;
                Ok(format!(
                    "[{}] {} - {} ({})",
                    captured_at, activity_type, title, summary
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if contexts.is_empty() {
            Ok("暂无上下文数据".to_string())
        } else {
            Ok(contexts.join("\n"))
        }
    }

    /// 根据活动模式分类提示
    fn classify_tip(pattern: &ActivityPattern) -> (TipCategory, TipPriority) {
        // 超过2小时连续工作，健康提醒，高优先级
        if pattern.continuous_work_minutes >= 120 {
            return (TipCategory::Health, TipPriority::High);
        }

        // 超过1小时连续工作，健康提醒，中优先级
        if pattern.continuous_work_minutes >= 60 {
            return (TipCategory::Health, TipPriority::Medium);
        }

        // 活动多样性低（小于1.0），说明过于专注单一活动
        if pattern.activity_diversity < 1.0 && pattern.total_activities >= 5 {
            return (TipCategory::Focus, TipPriority::Medium);
        }

        // 默认生产力建议，低优先级
        (TipCategory::Productivity, TipPriority::Low)
    }

    /// 生成默认提示（不使用AI）
    pub fn generate_default_tip(pattern: &ActivityPattern) -> (String, TipCategory, TipPriority) {
        // 超过2小时连续工作
        if pattern.continuous_work_minutes >= 120 {
            let apps_info = if !pattern.recent_apps.is_empty() {
                format!("（主要使用：{}）", pattern.recent_apps.join("、"))
            } else {
                String::new()
            };
            return (
                format!(
                    "您已连续工作 {} 分钟{}，建议稍作休息。起身活动一下，远眺窗外放松眼睛，喝杯水补充水分。",
                    pattern.continuous_work_minutes,
                    apps_info
                ),
                TipCategory::Health,
                TipPriority::High,
            );
        }

        // 超过1小时连续工作
        if pattern.continuous_work_minutes >= 60 {
            return (
                format!(
                    "您已持续工作 {} 分钟，记得适时休息哦！",
                    pattern.continuous_work_minutes
                ),
                TipCategory::Health,
                TipPriority::Medium,
            );
        }

        // 活动类型过于单一
        if pattern.activity_diversity < 1.0 && pattern.total_activities >= 5 {
            let activity = pattern.dominant_activity_type.as_deref().unwrap_or("当前活动");
            let focus_info = if !pattern.focus_areas.is_empty() {
                format!("（专注于：{}）", pattern.focus_areas.join("、"))
            } else {
                String::new()
            };
            return (
                format!(
                    "您最近主要在进行 {}{}，建议切换到其他任务，保持工作多样性可以提高效率。",
                    activity,
                    focus_info
                ),
                TipCategory::Focus,
                TipPriority::Medium,
            );
        }

        // 默认鼓励（提供专注领域信息）
        let message = if !pattern.focus_areas.is_empty() {
            format!(
                "保持良好的工作节奏！当前专注于：{}。记得劳逸结合。",
                pattern.focus_areas.join("、")
            )
        } else {
            "保持良好的工作节奏！记得劳逸结合。".to_string()
        };

        (
            message,
            TipCategory::Productivity,
            TipPriority::Low,
        )
    }

    /// 保存提示到数据库
    pub fn save_tip(
        conn: &Connection,
        content: &str,
        category: &TipCategory,
        priority: &TipPriority,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO tips (content, category, priority) VALUES (?1, ?2, ?3)",
            params![content, category.as_str(), priority.as_str()],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 获取提示列表
    pub fn list_tips(
        conn: &Connection,
        is_read: Option<bool>,
        limit: Option<i32>,
    ) -> Result<Vec<Tip>> {
        let mut sql = String::from(
            "SELECT id, content, category, priority, is_read, created_at FROM tips WHERE 1=1"
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(read) = is_read {
            sql.push_str(" AND is_read = ?");
            params_vec.push(Box::new(if read { 1 } else { 0 }));
        }

        sql.push_str(" ORDER BY created_at DESC");

        if let Some(l) = limit {
            sql.push_str(&format!(" LIMIT {}", l));
        }

        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let tips = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(Tip {
                    id: Some(row.get(0)?),
                    content: row.get(1)?,
                    category: row.get(2)?,
                    priority: row.get(3)?,
                    is_read: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tips)
    }

    /// 标记提示已读
    pub fn mark_read(conn: &Connection, id: i64) -> Result<()> {
        conn.execute(
            "UPDATE tips SET is_read = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// 获取未读提示数量
    pub fn get_unread_count(conn: &Connection) -> Result<i32> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM tips WHERE is_read = 0",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// 检查是否应该生成新提示
    pub fn should_generate_tip(conn: &Connection, interval_minutes: i64) -> Result<bool> {
        // 检查最近一次提示的时间
        let last_tip_time: Option<String> = conn
            .query_row(
                "SELECT created_at FROM tips ORDER BY created_at DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(last_time_str) = last_tip_time {
            let last_time = chrono::NaiveDateTime::parse_from_str(&last_time_str, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| anyhow!("解析时间失败: {}", e))?;
            let now = Local::now().naive_local();
            let elapsed_minutes = (now - last_time).num_minutes();

            // 如果距离上次提示不足间隔时间，不生成
            if elapsed_minutes < interval_minutes {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// 清理旧提示
    pub fn cleanup_old_tips(conn: &Connection, days: i64) -> Result<i64> {
        let cutoff = (Local::now() - ChronoDuration::days(days))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let deleted = conn.execute(
            "DELETE FROM tips WHERE created_at < ?1 AND is_read = 1",
            params![cutoff],
        )?;

        Ok(deleted as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tip_category() {
        assert_eq!(TipCategory::Health.as_str(), "health");
        assert_eq!(TipCategory::Productivity.as_str(), "productivity");
        assert_eq!(TipCategory::Focus.as_str(), "focus");

        assert!(matches!(
            TipCategory::from_str("health"),
            Some(TipCategory::Health)
        ));
    }

    #[test]
    fn test_tip_priority() {
        assert_eq!(TipPriority::Low.as_str(), "low");
        assert_eq!(TipPriority::Medium.as_str(), "medium");
        assert_eq!(TipPriority::High.as_str(), "high");
    }

    #[test]
    fn test_classify_tip() {
        // 测试长时间工作
        let mut activity_distribution = HashMap::new();
        activity_distribution.insert("coding".to_string(), 10);

        let pattern = ActivityPattern {
            continuous_work_minutes: 130,
            dominant_activity_type: Some("coding".to_string()),
            dominant_app: Some("VSCode".to_string()),
            activity_diversity: 1.5,
            total_activities: 10,
            activity_distribution: activity_distribution.clone(),
            recent_apps: vec!["VSCode".to_string()],
            focus_areas: vec!["Rust".to_string(), "Backend".to_string()],
        };
        let (category, priority) = TipsService::classify_tip(&pattern);
        assert_eq!(category, TipCategory::Health);
        assert_eq!(priority, TipPriority::High);

        // 测试活动单一
        let pattern = ActivityPattern {
            continuous_work_minutes: 30,
            dominant_activity_type: Some("coding".to_string()),
            dominant_app: Some("VSCode".to_string()),
            activity_diversity: 0.8,
            total_activities: 10,
            activity_distribution,
            recent_apps: vec!["VSCode".to_string()],
            focus_areas: vec!["Rust".to_string()],
        };
        let (category, priority) = TipsService::classify_tip(&pattern);
        assert_eq!(category, TipCategory::Focus);
        assert_eq!(priority, TipPriority::Medium);
    }
}
