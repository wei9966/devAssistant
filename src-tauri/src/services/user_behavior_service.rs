//! 用户行为学习服务
//!
//! 本模块提供用户行为分析和个性化偏好学习功能，包括：
//!
//! # 功能模块
//!
//! ## 1. 行为统计 (BehaviorStatistics)
//! - 番茄钟时长分布
//! - 任务优先级分布
//! - 每小时活动统计
//! - 总任务完成数
//!
//! ## 2. 用户偏好推断 (UserPreferences)
//! - 首选番茄钟时长
//! - 首选任务优先级
//! - 活跃时间段
//! - 平均任务完成天数
//!
//! ## 3. 表达偏好学习 (ExpressionPreferences) ✨ 新增
//! - 任务命名模式识别（前缀习惯、命名风格）
//! - 常用标签统计
//! - 优先级表达映射学习（如"很急" → 高优先级）
//! - 常用时间表达
//! - 常用项目名称
//!
//! # 使用示例
//!
//! ```rust,no_run
//! use user_behavior_service::UserBehaviorService;
//!
//! // 获取用户偏好
//! let prefs = UserBehaviorService::infer_preferences(&conn)?;
//!
//! // 获取表达偏好
//! let expr_prefs = UserBehaviorService::get_expression_preferences(&conn)?;
//!
//! // 构建完整的系统提示词（整合所有偏好）
//! let system_prompt = UserBehaviorService::build_system_prompt(&conn)?;
//! ```

use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 用户偏好数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// 首选番茄钟时长（分钟）
    pub preferred_pomodoro_duration: Option<i32>,
    /// 首选任务优先级
    pub preferred_priority: Option<i32>,
    /// 活跃时间段（0-23小时）
    pub active_hours: Vec<i32>,
    /// 平均任务完成天数
    pub avg_task_completion_days: Option<f32>,
}

/// 用户行为统计数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorStatistics {
    /// 番茄钟时长分布 (时长, 次数)
    pub pomodoro_duration_distribution: Vec<(i32, i32)>,
    /// 任务优先级分布 (优先级, 次数)
    pub priority_distribution: Vec<(i32, i32)>,
    /// 每小时活动次数 (小时, 活动次数)
    pub hourly_activity: Vec<(i32, i32)>,
    /// 总番茄钟数
    pub total_pomodoros: i32,
    /// 总完成任务数
    pub total_tasks_completed: i32,
}

/// 表达偏好数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionPreferences {
    /// 常用任务命名模式（如：前缀习惯、动词使用）
    pub task_naming_patterns: Vec<String>,
    /// 常用标签
    pub frequent_tags: Vec<String>,
    /// 优先级表达映射（用户说法 -> 标准优先级）
    pub priority_expressions: HashMap<String, String>,
    /// 常用时间表达
    pub time_expressions: Vec<String>,
    /// 常用项目名称
    pub frequent_projects: Vec<String>,
}

/// 用户行为学习服务
pub struct UserBehaviorService;

impl UserBehaviorService {
    /// 获取用户行为统计
    pub fn get_behavior_statistics(conn: &Connection) -> Result<BehaviorStatistics> {
        // 1. 统计番茄钟时长分布
        let pomodoro_duration_distribution = Self::get_pomodoro_duration_distribution(conn)?;

        // 2. 统计任务优先级分布
        let priority_distribution = Self::get_priority_distribution(conn)?;

        // 3. 统计每小时活动次数
        let hourly_activity = Self::get_hourly_activity(conn)?;

        // 4. 总番茄钟数（已完成的）
        let total_pomodoros: i32 = conn.query_row(
            "SELECT COUNT(*) FROM pomodoro_sessions WHERE status = 'completed'",
            [],
            |row| row.get(0),
        )?;

        // 5. 总完成任务数
        let total_tasks_completed: i32 = conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE status = 'done'",
            [],
            |row| row.get(0),
        )?;

        Ok(BehaviorStatistics {
            pomodoro_duration_distribution,
            priority_distribution,
            hourly_activity,
            total_pomodoros,
            total_tasks_completed,
        })
    }

    /// 推断用户偏好
    pub fn infer_preferences(conn: &Connection) -> Result<UserPreferences> {
        // 1. 推断首选番茄钟时长（使用频率最高的时长）
        let preferred_pomodoro_duration = Self::get_most_frequent_pomodoro_duration(conn)?;

        // 2. 推断首选任务优先级（使用频率最高的优先级）
        let preferred_priority = Self::get_most_frequent_priority(conn)?;

        // 3. 推断活跃时间段（活动次数超过平均值的小时）
        let active_hours = Self::get_active_hours(conn)?;

        // 4. 计算平均任务完成天数
        let avg_task_completion_days = Self::get_avg_task_completion_days(conn)?;

        Ok(UserPreferences {
            preferred_pomodoro_duration,
            preferred_priority,
            active_hours,
            avg_task_completion_days,
        })
    }

    /// 获取推荐的番茄钟时长
    pub fn get_recommended_pomodoro_duration(conn: &Connection) -> Result<i32> {
        // 优先使用用户最常用的时长，如果没有数据则返回默认值 25 分钟
        match Self::get_most_frequent_pomodoro_duration(conn)? {
            Some(duration) => Ok(duration),
            None => Ok(25), // 默认 25 分钟
        }
    }

    /// 获取推荐的任务优先级
    pub fn get_recommended_priority(conn: &Connection) -> Result<i32> {
        // 优先使用用户最常用的优先级，如果没有数据则返回默认值 2（中等优先级）
        match Self::get_most_frequent_priority(conn)? {
            Some(priority) => Ok(priority),
            None => Ok(2), // 默认优先级 2（中等）
        }
    }

    /// 格式化偏好为文本（用于注入系统提示词）
    pub fn format_preferences_for_prompt(prefs: &UserPreferences) -> String {
        let mut parts = Vec::new();

        // 番茄钟时长偏好
        if let Some(duration) = prefs.preferred_pomodoro_duration {
            parts.push(format!("用户偏好的番茄钟时长：{} 分钟", duration));
        }

        // 任务优先级偏好
        if let Some(priority) = prefs.preferred_priority {
            let priority_name = match priority {
                1 => "高优先级",
                2 => "中等优先级",
                3 => "低优先级",
                _ => "未知优先级",
            };
            parts.push(format!("用户偏好的任务优先级：{}", priority_name));
        }

        // 活跃时间段
        if !prefs.active_hours.is_empty() {
            let hours_str = prefs
                .active_hours
                .iter()
                .map(|h| format!("{}:00", h))
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("用户活跃时间段：{}", hours_str));
        }

        // 平均任务完成天数
        if let Some(avg_days) = prefs.avg_task_completion_days {
            parts.push(format!("用户平均任务完成时间：{:.1} 天", avg_days));
        }

        if parts.is_empty() {
            "暂无足够的用户行为数据来推断偏好。".to_string()
        } else {
            parts.join("；")
        }
    }

    /// 从历史任务中学习命名模式
    pub fn learn_task_naming_patterns(conn: &Connection) -> Result<Vec<String>> {
        let mut patterns = Vec::new();

        // 获取所有任务标题
        let mut stmt = conn.prepare(
            "SELECT title FROM tasks ORDER BY created_at DESC LIMIT 100"
        )?;

        let titles: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        if titles.is_empty() {
            return Ok(patterns);
        }

        // 分析常用前缀（动词）
        let common_prefixes = vec![
            "完成", "修复", "优化", "添加", "实现", "开发", "测试", "调试",
            "更新", "重构", "删除", "改进", "修改", "创建", "设计", "学习",
            "fix", "add", "update", "implement", "refactor", "optimize", "test"
        ];

        let mut prefix_counts: HashMap<String, usize> = HashMap::new();
        for title in &titles {
            for prefix in &common_prefixes {
                if title.starts_with(prefix) {
                    *prefix_counts.entry(prefix.to_string()).or_insert(0) += 1;
                }
            }
        }

        // 提取使用频率超过5%的前缀
        let total = titles.len();
        for (prefix, count) in prefix_counts {
            if count as f32 / total as f32 >= 0.05 {
                patterns.push(format!("常用前缀：{}", prefix));
            }
        }

        // 判断命名风格（中文/英文/混合）
        let chinese_count = titles.iter().filter(|t| {
            t.chars().any(|c| c >= '\u{4E00}' && c <= '\u{9FA5}')
        }).count();

        let english_count = titles.iter().filter(|t| {
            t.chars().any(|c| c.is_ascii_alphabetic())
        }).count();

        if chinese_count as f32 / total as f32 >= 0.7 {
            patterns.push("命名风格：主要使用中文".to_string());
        } else if english_count as f32 / total as f32 >= 0.7 {
            patterns.push("命名风格：主要使用英文".to_string());
        } else if chinese_count > 0 && english_count > 0 {
            patterns.push("命名风格：中英文混合".to_string());
        }

        Ok(patterns)
    }

    /// 统计常用标签
    pub fn learn_frequent_tags(conn: &Connection) -> Result<Vec<String>> {
        // 从 task_tags 和 tags 表中统计标签使用频率
        let mut stmt = conn.prepare(
            "SELECT t.name, COUNT(tt.id) as count
             FROM tags t
             LEFT JOIN task_tags tt ON t.id = tt.tag_id
             GROUP BY t.id, t.name
             HAVING count > 0
             ORDER BY count DESC
             LIMIT 10"
        )?;

        let tags: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(tags)
    }

    /// 学习优先级表达方式
    /// 分析用户消息和实际设置的优先级的对应关系
    pub fn learn_priority_expressions(conn: &Connection) -> Result<HashMap<String, String>> {
        let mut expressions = HashMap::new();

        // 从 ai_context_memory 中查找优先级相关的记忆
        let mut stmt = conn.prepare(
            "SELECT key, value FROM ai_context_memory
             WHERE context_type = 'priority_mapping'
             ORDER BY importance DESC, last_used_at DESC
             LIMIT 20"
        )?;

        let mappings: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        for (key, value) in mappings {
            expressions.insert(key, value);
        }

        // 如果没有学习数据，提供一些默认的常见映射
        if expressions.is_empty() {
            expressions.insert("很重要".to_string(), "高优先级".to_string());
            expressions.insert("紧急".to_string(), "高优先级".to_string());
            expressions.insert("很急".to_string(), "高优先级".to_string());
            expressions.insert("重要".to_string(), "高优先级".to_string());
            expressions.insert("一般".to_string(), "中等优先级".to_string());
            expressions.insert("正常".to_string(), "中等优先级".to_string());
            expressions.insert("普通".to_string(), "中等优先级".to_string());
            expressions.insert("不着急".to_string(), "低优先级".to_string());
            expressions.insert("有空再做".to_string(), "低优先级".to_string());
            expressions.insert("慢慢来".to_string(), "低优先级".to_string());
        }

        Ok(expressions)
    }

    /// 学习常用时间表达
    pub fn learn_time_expressions(conn: &Connection) -> Result<Vec<String>> {
        let mut expressions = Vec::new();

        // 从 ai_context_memory 中查找时间相关的记忆
        let mut stmt = conn.prepare(
            "SELECT DISTINCT value FROM ai_context_memory
             WHERE context_type = 'time_expression'
             ORDER BY importance DESC, last_used_at DESC
             LIMIT 10"
        )?;

        let time_exprs: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        expressions.extend(time_exprs);

        Ok(expressions)
    }

    /// 学习常用项目名称
    pub fn learn_frequent_projects(conn: &Connection) -> Result<Vec<String>> {
        let mut projects = Vec::new();

        // 从任务描述和上下文中提取项目名称
        let mut stmt = conn.prepare(
            "SELECT DISTINCT value FROM ai_context_memory
             WHERE context_type = 'project_name'
             ORDER BY importance DESC, last_used_at DESC
             LIMIT 10"
        )?;

        let project_names: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        projects.extend(project_names);

        Ok(projects)
    }

    /// 获取完整的表达偏好
    pub fn get_expression_preferences(conn: &Connection) -> Result<ExpressionPreferences> {
        Ok(ExpressionPreferences {
            task_naming_patterns: Self::learn_task_naming_patterns(conn)?,
            frequent_tags: Self::learn_frequent_tags(conn)?,
            priority_expressions: Self::learn_priority_expressions(conn)?,
            time_expressions: Self::learn_time_expressions(conn)?,
            frequent_projects: Self::learn_frequent_projects(conn)?,
        })
    }

    /// 格式化表达偏好为提示词
    pub fn format_expression_preferences_for_prompt(prefs: &ExpressionPreferences) -> String {
        let mut parts = Vec::new();

        parts.push("【用户表达习惯】".to_string());

        // 任务命名模式
        if !prefs.task_naming_patterns.is_empty() {
            parts.push(format!(
                "- 任务命名习惯：{}",
                prefs.task_naming_patterns.join("、")
            ));
        }

        // 常用标签
        if !prefs.frequent_tags.is_empty() {
            parts.push(format!(
                "- 常用标签：{}",
                prefs.frequent_tags.join("、")
            ));
        }

        // 优先级表达
        if !prefs.priority_expressions.is_empty() {
            let expr_strs: Vec<String> = prefs
                .priority_expressions
                .iter()
                .take(5)  // 只显示前5个最常用的
                .map(|(k, v)| format!("「{}」→{}", k, v))
                .collect();
            parts.push(format!(
                "- 优先级表达习惯：{}",
                expr_strs.join("，")
            ));
        }

        // 时间表达
        if !prefs.time_expressions.is_empty() {
            parts.push(format!(
                "- 常用时间表达：{}",
                prefs.time_expressions.join("、")
            ));
        }

        // 项目名称
        if !prefs.frequent_projects.is_empty() {
            parts.push(format!(
                "- 常提及的项目：{}",
                prefs.frequent_projects.join("、")
            ));
        }

        if parts.len() == 1 {
            // 只有标题，没有实际内容
            "暂无足够的用户表达习惯数据。".to_string()
        } else {
            parts.join("\n")
        }
    }

    /// 构建系统提示词（整合用户偏好和表达习惯）
    pub fn build_system_prompt(conn: &Connection) -> Result<String> {
        let mut prompt_parts = Vec::new();

        // 1. 基础系统提示
        prompt_parts.push("你是一个智能任务管理助手，帮助用户高效管理任务和时间。".to_string());

        // 2. 用户偏好
        match Self::infer_preferences(conn) {
            Ok(prefs) => {
                let prefs_text = Self::format_preferences_for_prompt(&prefs);
                if !prefs_text.contains("暂无") {
                    prompt_parts.push("\n【用户偏好】".to_string());
                    prompt_parts.push(prefs_text);
                }
            }
            Err(_) => {}
        }

        // 3. 表达习惯
        match Self::get_expression_preferences(conn) {
            Ok(expr_prefs) => {
                let expr_text = Self::format_expression_preferences_for_prompt(&expr_prefs);
                if !expr_text.contains("暂无") {
                    prompt_parts.push("\n".to_string());
                    prompt_parts.push(expr_text);
                }
            }
            Err(_) => {}
        }

        // 4. 使用指南
        prompt_parts.push("\n【注意事项】".to_string());
        prompt_parts.push("- 根据用户的表达习惯理解其意图".to_string());
        prompt_parts.push("- 使用用户熟悉的词汇和命名风格".to_string());
        prompt_parts.push("- 优先推荐用户常用的标签和分类".to_string());
        prompt_parts.push("- 参考用户的活跃时间段安排任务".to_string());

        Ok(prompt_parts.join("\n"))
    }

    // ==================== 私有辅助方法 ====================

    /// 获取番茄钟时长分布
    fn get_pomodoro_duration_distribution(conn: &Connection) -> Result<Vec<(i32, i32)>> {
        let mut stmt = conn.prepare(
            "SELECT duration_minutes, COUNT(*) as count
             FROM pomodoro_sessions
             WHERE status = 'completed' AND duration_minutes IS NOT NULL
             GROUP BY duration_minutes
             ORDER BY count DESC",
        )?;

        let distributions: Vec<(i32, i32)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(distributions)
    }

    /// 获取任务优先级分布
    fn get_priority_distribution(conn: &Connection) -> Result<Vec<(i32, i32)>> {
        let mut stmt = conn.prepare(
            "SELECT priority, COUNT(*) as count
             FROM tasks
             WHERE priority IS NOT NULL
             GROUP BY priority
             ORDER BY count DESC",
        )?;

        let distributions: Vec<(i32, i32)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(distributions)
    }

    /// 获取每小时活动次数（基于番茄钟开始时间和任务创建时间）
    fn get_hourly_activity(conn: &Connection) -> Result<Vec<(i32, i32)>> {
        // 统计番茄钟开始时间的小时分布
        let mut pomodoro_hours: HashMap<i32, i32> = HashMap::new();

        let mut stmt = conn.prepare(
            "SELECT started_at FROM pomodoro_sessions WHERE started_at IS NOT NULL",
        )?;

        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        for row in rows.filter_map(|r| r.ok()) {
            if let Some(hour) = Self::extract_hour_from_datetime(&row) {
                *pomodoro_hours.entry(hour).or_insert(0) += 1;
            }
        }

        // 统计任务创建时间的小时分布
        let mut task_hours: HashMap<i32, i32> = HashMap::new();

        let mut stmt = conn.prepare("SELECT created_at FROM tasks WHERE created_at IS NOT NULL")?;

        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        for row in rows.filter_map(|r| r.ok()) {
            if let Some(hour) = Self::extract_hour_from_datetime(&row) {
                *task_hours.entry(hour).or_insert(0) += 1;
            }
        }

        // 合并两个统计结果
        let mut hourly_activity: HashMap<i32, i32> = HashMap::new();

        for (hour, count) in pomodoro_hours {
            *hourly_activity.entry(hour).or_insert(0) += count;
        }

        for (hour, count) in task_hours {
            *hourly_activity.entry(hour).or_insert(0) += count;
        }

        // 转换为排序后的 Vec
        let mut result: Vec<(i32, i32)> = hourly_activity.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(result)
    }

    /// 获取最常用的番茄钟时长
    fn get_most_frequent_pomodoro_duration(conn: &Connection) -> Result<Option<i32>> {
        let result: Option<i32> = conn
            .query_row(
                "SELECT duration_minutes
                 FROM pomodoro_sessions
                 WHERE status = 'completed' AND duration_minutes IS NOT NULL
                 GROUP BY duration_minutes
                 ORDER BY COUNT(*) DESC
                 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        Ok(result)
    }

    /// 获取最常用的任务优先级
    fn get_most_frequent_priority(conn: &Connection) -> Result<Option<i32>> {
        let result: Option<i32> = conn
            .query_row(
                "SELECT priority
                 FROM tasks
                 WHERE priority IS NOT NULL
                 GROUP BY priority
                 ORDER BY COUNT(*) DESC
                 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        Ok(result)
    }

    /// 获取活跃时间段（活动次数超过平均值的小时）
    fn get_active_hours(conn: &Connection) -> Result<Vec<i32>> {
        let hourly_activity = Self::get_hourly_activity(conn)?;

        if hourly_activity.is_empty() {
            return Ok(Vec::new());
        }

        // 计算平均活动次数
        let total_count: i32 = hourly_activity.iter().map(|(_, count)| count).sum();
        let avg_count = total_count as f32 / hourly_activity.len() as f32;

        // 筛选出活动次数超过平均值的小时
        let active_hours: Vec<i32> = hourly_activity
            .into_iter()
            .filter(|(_, count)| *count as f32 >= avg_count)
            .map(|(hour, _)| hour)
            .collect();

        Ok(active_hours)
    }

    /// 计算平均任务完成天数
    fn get_avg_task_completion_days(conn: &Connection) -> Result<Option<f32>> {
        // 只统计已完成的任务
        let mut stmt = conn.prepare(
            "SELECT created_at, completed_at
             FROM tasks
             WHERE status = 'done'
             AND created_at IS NOT NULL
             AND completed_at IS NOT NULL",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut total_days = 0.0;
        let mut count = 0;

        for row in rows.filter_map(|r| r.ok()) {
            let (created_at, completed_at) = row;

            if let (Some(created), Some(completed)) =
                (Self::parse_datetime(&created_at), Self::parse_datetime(&completed_at))
            {
                let duration = completed.signed_duration_since(created);
                let days = duration.num_seconds() as f32 / 86400.0; // 转换为天数
                total_days += days;
                count += 1;
            }
        }

        if count > 0 {
            Ok(Some(total_days / count as f32))
        } else {
            Ok(None)
        }
    }

    /// 从日期时间字符串中提取小时（支持多种格式）
    fn extract_hour_from_datetime(datetime: &str) -> Option<i32> {
        use chrono::{NaiveDateTime, Timelike};

        // 尝试多种日期时间格式
        let formats = [
            "%Y-%m-%d %H:%M:%S",        // 2024-01-01 10:30:00
            "%Y-%m-%dT%H:%M:%S",        // 2024-01-01T10:30:00
            "%Y-%m-%d %H:%M:%S%.f",     // 2024-01-01 10:30:00.123
            "%Y-%m-%dT%H:%M:%S%.f",     // 2024-01-01T10:30:00.123
            "%Y-%m-%dT%H:%M:%S%z",      // 2024-01-01T10:30:00+08:00
            "%Y-%m-%dT%H:%M:%S%.f%z",   // 2024-01-01T10:30:00.123+08:00
        ];

        for format in &formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(datetime, format) {
                return Some(dt.hour() as i32);
            }
        }

        None
    }

    /// 解析日期时间字符串为 NaiveDateTime
    fn parse_datetime(datetime: &str) -> Option<chrono::NaiveDateTime> {
        use chrono::NaiveDateTime;

        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S%.f",
        ];

        for format in &formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(datetime, format) {
                return Some(dt);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // 创建测试表
        conn.execute(
            "CREATE TABLE pomodoro_sessions (
                id INTEGER PRIMARY KEY,
                duration_minutes INTEGER,
                status TEXT,
                started_at TEXT
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE tasks (
                id INTEGER PRIMARY KEY,
                priority INTEGER,
                status TEXT,
                created_at TEXT,
                completed_at TEXT
            )",
            [],
        )
        .unwrap();

        conn
    }

    #[test]
    fn test_get_behavior_statistics() {
        let conn = setup_test_db();

        // 插入测试数据
        conn.execute(
            "INSERT INTO pomodoro_sessions (duration_minutes, status, started_at) VALUES (25, 'completed', '2024-01-01 10:00:00')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO pomodoro_sessions (duration_minutes, status, started_at) VALUES (25, 'completed', '2024-01-01 14:00:00')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO pomodoro_sessions (duration_minutes, status, started_at) VALUES (50, 'completed', '2024-01-01 16:00:00')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tasks (priority, status, created_at, completed_at) VALUES (1, 'done', '2024-01-01 09:00:00', '2024-01-02 10:00:00')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tasks (priority, status, created_at, completed_at) VALUES (2, 'done', '2024-01-01 11:00:00', '2024-01-03 12:00:00')",
            [],
        )
        .unwrap();

        let stats = UserBehaviorService::get_behavior_statistics(&conn).unwrap();

        assert_eq!(stats.total_pomodoros, 3);
        assert_eq!(stats.total_tasks_completed, 2);
        assert!(!stats.pomodoro_duration_distribution.is_empty());
        assert!(!stats.priority_distribution.is_empty());
    }

    #[test]
    fn test_infer_preferences() {
        let conn = setup_test_db();

        // 插入测试数据
        conn.execute(
            "INSERT INTO pomodoro_sessions (duration_minutes, status, started_at) VALUES (25, 'completed', '2024-01-01 10:00:00')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO pomodoro_sessions (duration_minutes, status, started_at) VALUES (25, 'completed', '2024-01-01 14:00:00')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tasks (priority, status, created_at, completed_at) VALUES (1, 'done', '2024-01-01 09:00:00', '2024-01-02 10:00:00')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tasks (priority, status, created_at, completed_at) VALUES (1, 'done', '2024-01-01 11:00:00', '2024-01-03 12:00:00')",
            [],
        )
        .unwrap();

        let prefs = UserBehaviorService::infer_preferences(&conn).unwrap();

        assert_eq!(prefs.preferred_pomodoro_duration, Some(25));
        assert_eq!(prefs.preferred_priority, Some(1));
        assert!(prefs.avg_task_completion_days.is_some());
    }

    #[test]
    fn test_get_recommended_pomodoro_duration() {
        let conn = setup_test_db();

        // 没有数据时应返回默认值 25
        let duration = UserBehaviorService::get_recommended_pomodoro_duration(&conn).unwrap();
        assert_eq!(duration, 25);

        // 插入数据后应返回最常用的时长
        conn.execute(
            "INSERT INTO pomodoro_sessions (duration_minutes, status) VALUES (50, 'completed')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO pomodoro_sessions (duration_minutes, status) VALUES (50, 'completed')",
            [],
        )
        .unwrap();

        let duration = UserBehaviorService::get_recommended_pomodoro_duration(&conn).unwrap();
        assert_eq!(duration, 50);
    }

    #[test]
    fn test_format_preferences_for_prompt() {
        let prefs = UserPreferences {
            preferred_pomodoro_duration: Some(25),
            preferred_priority: Some(1),
            active_hours: vec![9, 10, 14, 15],
            avg_task_completion_days: Some(1.5),
        };

        let prompt = UserBehaviorService::format_preferences_for_prompt(&prefs);

        assert!(prompt.contains("25 分钟"));
        assert!(prompt.contains("高优先级"));
        assert!(prompt.contains("9:00"));
        assert!(prompt.contains("1.5 天"));
    }

    #[test]
    fn test_extract_hour_from_datetime() {
        assert_eq!(
            UserBehaviorService::extract_hour_from_datetime("2024-01-01 10:30:00"),
            Some(10)
        );
        assert_eq!(
            UserBehaviorService::extract_hour_from_datetime("2024-01-01T14:45:00"),
            Some(14)
        );
        assert_eq!(
            UserBehaviorService::extract_hour_from_datetime("invalid"),
            None
        );
    }

    #[test]
    fn test_learn_task_naming_patterns() {
        let conn = setup_test_db();

        // 插入一些测试任务
        let test_tasks = vec![
            "完成用户登录功能",
            "完成数据导出",
            "修复订单bug",
            "优化查询性能",
            "添加日志功能",
            "fix login issue",
            "update readme",
        ];

        for task in test_tasks {
            conn.execute(
                "INSERT INTO tasks (title, description, category, priority, status) VALUES (?, ?, ?, ?, ?)",
                [task, "测试", "dev", "1", "todo"],
            )
            .unwrap();
        }

        let patterns = UserBehaviorService::learn_task_naming_patterns(&conn).unwrap();

        // 应该识别出"完成"是常用前缀
        assert!(patterns.iter().any(|p| p.contains("完成")));
        // 应该识别出命名风格
        assert!(patterns.iter().any(|p| p.contains("命名风格")));
    }

    #[test]
    fn test_learn_frequent_tags() {
        let conn = setup_test_db();

        // 创建标签表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#6366f1',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS task_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(task_id, tag_id)
            )",
            [],
        )
        .unwrap();

        // 插入测试标签
        conn.execute("INSERT INTO tags (name) VALUES (?)", ["工作"])
            .unwrap();
        conn.execute("INSERT INTO tags (name) VALUES (?)", ["紧急"])
            .unwrap();

        // 插入测试任务
        conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status) VALUES (?, ?, ?, ?, ?)",
            ["测试任务1", "测试", "dev", "1", "todo"],
        )
        .unwrap();

        // 关联标签
        conn.execute("INSERT INTO task_tags (task_id, tag_id) VALUES (1, 1)", [])
            .unwrap();

        let tags = UserBehaviorService::learn_frequent_tags(&conn).unwrap();

        assert!(tags.contains(&"工作".to_string()));
    }

    #[test]
    fn test_learn_priority_expressions() {
        let conn = setup_test_db();

        // 创建 ai_context_memory 表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ai_context_memory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                context_type TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                importance INTEGER DEFAULT 1,
                last_used_at TEXT,
                created_at TEXT DEFAULT (datetime('now', 'localtime'))
            )",
            [],
        )
        .unwrap();

        // 插入优先级映射记忆
        conn.execute(
            "INSERT INTO ai_context_memory (context_type, key, value, importance) VALUES (?, ?, ?, ?)",
            ["priority_mapping", "超级重要", "高优先级", "3"],
        )
        .unwrap();

        let expressions = UserBehaviorService::learn_priority_expressions(&conn).unwrap();

        // 应该包含默认映射或学习到的映射
        assert!(!expressions.is_empty());
        assert!(expressions.contains_key("很重要") || expressions.contains_key("超级重要"));
    }

    #[test]
    fn test_get_expression_preferences() {
        let conn = setup_test_db();

        // 创建必要的表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#6366f1',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS task_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(task_id, tag_id)
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS ai_context_memory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                context_type TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                importance INTEGER DEFAULT 1,
                last_used_at TEXT,
                created_at TEXT DEFAULT (datetime('now', 'localtime'))
            )",
            [],
        )
        .unwrap();

        // 插入测试数据
        conn.execute(
            "INSERT INTO tasks (title, description, category, priority, status) VALUES (?, ?, ?, ?, ?)",
            ["完成测试任务", "测试", "dev", "1", "todo"],
        )
        .unwrap();

        let prefs = UserBehaviorService::get_expression_preferences(&conn).unwrap();

        // 验证返回的数据结构
        assert!(prefs.priority_expressions.len() > 0);
    }

    #[test]
    fn test_format_expression_preferences_for_prompt() {
        use std::collections::HashMap;

        let mut priority_exprs = HashMap::new();
        priority_exprs.insert("很重要".to_string(), "高优先级".to_string());
        priority_exprs.insert("一般".to_string(), "中等优先级".to_string());

        let prefs = ExpressionPreferences {
            task_naming_patterns: vec!["常用前缀：完成".to_string(), "命名风格：主要使用中文".to_string()],
            frequent_tags: vec!["工作".to_string(), "紧急".to_string()],
            priority_expressions: priority_exprs,
            time_expressions: vec!["明天".to_string(), "下周".to_string()],
            frequent_projects: vec!["项目A".to_string()],
        };

        let prompt = UserBehaviorService::format_expression_preferences_for_prompt(&prefs);

        assert!(prompt.contains("【用户表达习惯】"));
        assert!(prompt.contains("任务命名习惯"));
        assert!(prompt.contains("常用标签"));
        assert!(prompt.contains("优先级表达习惯"));
    }

    #[test]
    fn test_build_system_prompt() {
        let conn = setup_test_db();

        // 创建必要的表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#6366f1',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS task_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(task_id, tag_id)
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS ai_context_memory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                context_type TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                importance INTEGER DEFAULT 1,
                last_used_at TEXT,
                created_at TEXT DEFAULT (datetime('now', 'localtime'))
            )",
            [],
        )
        .unwrap();

        let prompt = UserBehaviorService::build_system_prompt(&conn).unwrap();

        // 验证提示词包含必要的部分
        assert!(prompt.contains("智能任务管理助手"));
        assert!(prompt.contains("【注意事项】"));
    }
}
