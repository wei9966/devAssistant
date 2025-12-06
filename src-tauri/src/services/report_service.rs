// Report Service
// 日报生成服务 - 按小时分块处理，合并生成完整日报

use super::ai_service::{AiService, ChatMessage};
use super::context_store_service::ContextStoreService;
use super::prompt_db_service::PromptDbService;
use crate::models::screen_context::ScreenContext;
use anyhow::{anyhow, Result};
use chrono::{NaiveDateTime, Timelike};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 日报模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyReport {
    pub id: Option<i64>,
    pub date: String,
    pub summary_text: String,
    pub highlights: String, // JSON
    pub insights: String,   // JSON
    pub total_screenshots: i32,
    pub activity_breakdown: String, // JSON
    pub created_at: Option<String>,
}

/// 小时活动块
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyBlock {
    pub hour: i32,
    pub contexts: Vec<ScreenContext>,
    pub summary: Option<String>,
}

/// 日报亮点
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportHighlight {
    pub title: String,
    pub description: String,
    pub time_range: String,
}

/// 日报洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportInsight {
    pub category: String, // "productivity", "focus", "patterns", "suggestions"
    pub content: String,
}

/// 活动分类统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityBreakdown {
    pub activity_type: String,
    pub count: i32,
    pub percentage: f32,
    pub duration_minutes: Option<i32>,
}

pub struct ReportService;

/// 日报生成输入数据（在async之前收集）
#[derive(Debug, Clone)]
pub struct ReportInputData {
    pub date: String,
    pub contexts: Vec<ScreenContext>,
    pub hourly_blocks: Vec<HourlyBlock>,
    pub stats_text: String,
}

impl ReportService {
    /// 收集日报生成所需数据（同步，在async之前调用）
    pub fn collect_report_data(conn: &Connection, date: &str) -> Result<ReportInputData> {
        // 获取指定日期的所有上下文
        let contexts = ContextStoreService::list_by_date(conn, date)?;

        if contexts.is_empty() {
            return Err(anyhow!("指定日期没有活动记录"));
        }

        // 按小时分块
        let hourly_blocks = Self::group_by_hour(&contexts)?;

        // 获取统计信息
        let stats = ContextStoreService::get_day_stats(conn, date)?;
        let stats_text = format!(
            "总活动数: {}\n应用使用: {:?}\n活动类型: {:?}",
            stats.total_count, stats.app_distribution, stats.activity_distribution
        );

        Ok(ReportInputData {
            date: date.to_string(),
            contexts,
            hourly_blocks,
            stats_text,
        })
    }

    /// 生成日报（按小时分块处理）- 异步部分
    pub async fn generate_report_async(
        input: ReportInputData,
        ai_service: &AiService,
    ) -> Result<DailyReport> {
        // 1. 为每个小时块生成摘要
        let mut block_summaries = Vec::new();
        for block in &input.hourly_blocks {
            let summary = Self::summarize_hour_block(ai_service, block).await?;
            block_summaries.push((block.hour, summary));
        }

        // 2. 合并所有小时摘要，生成完整日报
        let (summary_text, highlights, insights) =
            Self::generate_full_report_async(ai_service, &block_summaries, &input.stats_text, &input.date).await?;

        // 3. 计算活动统计
        let activity_breakdown = Self::calculate_activity_breakdown(&input.contexts)?;

        // 4. 构建日报对象
        let report = DailyReport {
            id: None,
            date: input.date,
            summary_text,
            highlights: serde_json::to_string(&highlights)?,
            insights: serde_json::to_string(&insights)?,
            total_screenshots: input.contexts.len() as i32,
            activity_breakdown: serde_json::to_string(&activity_breakdown)?,
            created_at: None,
        };

        Ok(report)
    }

    /// 保存并返回完整日报（同步，在async之后调用）
    pub fn save_and_get_report(conn: &Connection, report: &DailyReport) -> Result<DailyReport> {
        Self::save_report(conn, report)?;
        Self::get_report(conn, &report.date)?
            .ok_or_else(|| anyhow!("保存后无法读取日报"))
    }

    /// 按小时分组上下文
    fn group_by_hour(contexts: &[ScreenContext]) -> Result<Vec<HourlyBlock>> {
        let mut blocks: HashMap<i32, Vec<ScreenContext>> = HashMap::new();

        for context in contexts {
            // 解析时间戳，提取小时
            let dt = NaiveDateTime::parse_from_str(&context.captured_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| anyhow!("解析时间失败: {}", e))?;
            let hour = dt.hour() as i32;

            blocks.entry(hour).or_insert_with(Vec::new).push(context.clone());
        }

        // 转换为有序列表
        let mut hourly_blocks: Vec<_> = blocks
            .into_iter()
            .map(|(hour, contexts)| HourlyBlock {
                hour,
                contexts,
                summary: None,
            })
            .collect();

        hourly_blocks.sort_by_key(|b| b.hour);
        Ok(hourly_blocks)
    }

    /// 为单个小时块生成摘要
    async fn summarize_hour_block(
        ai_service: &AiService,
        block: &HourlyBlock,
    ) -> Result<String> {
        // 精简上下文信息，只保留关键字段
        let simplified_contexts: Vec<_> = block
            .contexts
            .iter()
            .map(|ctx| {
                serde_json::json!({
                    "time": ctx.captured_at,
                    "app": ctx.app_name,
                    "activity": ctx.activity_type,
                    "description": ctx.description,
                    "key_content": ctx.key_content,
                })
            })
            .collect();

        let activities_json = serde_json::to_string_pretty(&simplified_contexts)?;

        // 构建变量映射
        let mut vars = HashMap::new();
        vars.insert("hour".to_string(), block.hour.to_string());
        vars.insert("count".to_string(), block.contexts.len().to_string());
        vars.insert("activities_json".to_string(), activities_json.clone());

        // 尝试使用数据库提示词
        let messages = match PromptDbService::render_prompt_cached("hour_summary", &vars) {
            Ok(rendered) => {
                let mut msgs = Vec::new();
                if let Some(system) = rendered.system {
                    msgs.push(ChatMessage::system(system));
                }
                msgs.push(ChatMessage::user(rendered.user));
                msgs
            }
            Err(e) => {
                // 如果获取配置失败，使用硬编码的提示词作为后备
                log::warn!("使用数据库提示词失败，回退到默认提示词: {}", e);
                let prompt = format!(
                    r#"请总结以下{}点的工作活动（约{}条记录）：

活动记录：
{}

要求：
1. 用2-3句话简要概括该时段的主要活动
2. 突出关键工作内容和使用的工具
3. 语言简洁专业
4. 只返回摘要文本，不要添加标题或额外说明"#,
                    block.hour,
                    block.contexts.len(),
                    activities_json
                );
                vec![ChatMessage::user(prompt)]
            }
        };

        let summary = ai_service.chat(messages).await?;
        Ok(summary.trim().to_string())
    }

    /// 合并小时摘要，生成完整日报（异步，不使用Connection）
    async fn generate_full_report_async(
        ai_service: &AiService,
        block_summaries: &[(i32, String)],
        stats_text: &str,
        date: &str,
    ) -> Result<(String, Vec<ReportHighlight>, Vec<ReportInsight>)> {
        // 尝试使用数据库提示词生成日报
        match Self::generate_report_with_db_prompt(ai_service, block_summaries, stats_text, date).await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::warn!("使用数据库提示词生成日报失败，回退到默认方式: {}", e);
                Self::generate_report_fallback(ai_service, block_summaries, stats_text, date).await
            }
        }
    }

    /// 使用数据库提示词生成日报
    async fn generate_report_with_db_prompt(
        ai_service: &AiService,
        block_summaries: &[(i32, String)],
        _stats_text: &str,
        date: &str,
    ) -> Result<(String, Vec<ReportHighlight>, Vec<ReportInsight>)> {
        // 构建小时摘要文本
        let hourly_summaries = block_summaries
            .iter()
            .map(|(hour, summary)| format!("**{}:00 - {}:59**\n{}", hour, hour, summary))
            .collect::<Vec<_>>()
            .join("\n\n");

        // 构建变量映射
        let mut vars = HashMap::new();
        vars.insert("date".to_string(), date.to_string());
        vars.insert("activities_json".to_string(), hourly_summaries);

        // 使用数据库提示词 generation_report
        let rendered = PromptDbService::render_prompt_cached("generation_report", &vars)
            .map_err(|e| anyhow!("无法获取日报提示词配置: {}", e))?;

        // 构建消息
        let mut messages = Vec::new();
        if let Some(system) = rendered.system {
            messages.push(ChatMessage::system(system));
        }
        messages.push(ChatMessage::user(rendered.user));

        let response = ai_service.chat(messages).await?;

        // 解析 Markdown 格式的日报
        Self::parse_markdown_report(&response, date)
    }

    /// 回退方案：使用原有的JSON格式生成日报
    async fn generate_report_fallback(
        ai_service: &AiService,
        block_summaries: &[(i32, String)],
        stats_text: &str,
        date: &str,
    ) -> Result<(String, Vec<ReportHighlight>, Vec<ReportInsight>)> {
        // 构建小时摘要文本
        let summaries_text = block_summaries
            .iter()
            .map(|(hour, summary)| format!("{}:00 - {}:59\n{}", hour, hour, summary))
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            r#"请根据以下小时摘要和统计信息，生成一份完整的工作日报。

日期：{}

小时摘要：
{}

统计信息：
{}

请以JSON格式返回，包含以下字段：
{{
  "summary": "当日工作总结（200-300字，包含：工作时段、主要工作内容、关键成果、总体评价）",
  "highlights": [
    {{"title": "亮点标题", "description": "亮点描述", "timeRange": "时间段"}}
  ],
  "insights": [
    {{"category": "productivity|focus|patterns|suggestions", "content": "洞察内容"}}
  ]
}}

要求：
1. summary 需要结构化、专业化，突出关键信息
2. highlights 提取 2-4 个关键亮点（重要工作成果或高效时段）
3. insights 提供 3-5 条洞察，涵盖效率、专注度、工作模式、改进建议等
4. 只返回 JSON，不要其他内容"#,
            date, summaries_text, stats_text
        );

        let messages = vec![ChatMessage::user(prompt)];
        let response = ai_service.chat(messages).await?;

        // 解析 JSON 响应
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                &response
            }
        } else {
            &response
        };

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| anyhow!("解析 AI 响应失败: {}", e))?;

        let summary = parsed["summary"]
            .as_str()
            .unwrap_or("无法生成摘要")
            .to_string();

        let highlights: Vec<ReportHighlight> = parsed["highlights"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(ReportHighlight {
                            title: item["title"].as_str()?.to_string(),
                            description: item["description"].as_str()?.to_string(),
                            time_range: item["timeRange"].as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let insights: Vec<ReportInsight> = parsed["insights"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(ReportInsight {
                            category: item["category"].as_str()?.to_string(),
                            content: item["content"].as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok((summary, highlights, insights))
    }

    /// 解析 Markdown 格式的日报
    fn parse_markdown_report(
        markdown: &str,
        _date: &str,
    ) -> Result<(String, Vec<ReportHighlight>, Vec<ReportInsight>)> {
        // 提取 "今日概览" 或 "概览" 部分作为摘要
        let summary = Self::extract_section(markdown, &["## 📊 今日概览", "## 📊 概览", "## 今日概览", "## 概览"])
            .unwrap_or_else(|| {
                // 如果没有找到概览部分，取前300字符作为摘要
                markdown.chars().take(300).collect::<String>()
            });

        // 从 Markdown 中提取亮点
        let highlights = Self::extract_highlights_from_markdown(markdown);

        // 从 Markdown 中提取洞察
        let insights = Self::extract_insights_from_markdown(markdown);

        // 如果解析失败，返回整个 Markdown 作为摘要
        if highlights.is_empty() && insights.is_empty() {
            log::warn!("无法从Markdown中提取结构化信息，返回原始内容");
            return Ok((markdown.to_string(), vec![], vec![]));
        }

        Ok((summary, highlights, insights))
    }

    /// 从 Markdown 中提取指定章节的内容
    fn extract_section(markdown: &str, headers: &[&str]) -> Option<String> {
        for header in headers {
            if let Some(start) = markdown.find(header) {
                let content_start = start + header.len();
                let remaining = &markdown[content_start..];

                // 查找下一个二级标题
                if let Some(end) = remaining.find("\n##") {
                    return Some(remaining[..end].trim().to_string());
                } else {
                    return Some(remaining.trim().to_string());
                }
            }
        }
        None
    }

    /// 从 Markdown 中提取亮点
    fn extract_highlights_from_markdown(markdown: &str) -> Vec<ReportHighlight> {
        let mut highlights = Vec::new();

        // 查找 "完成事项" 或 "✅ 完成事项" 章节
        if let Some(section) = Self::extract_section(markdown, &["## ✅ 完成事项", "## 完成事项"]) {
            // 按行分割，查找列表项
            for line in section.lines() {
                let line = line.trim();
                if line.starts_with("- ") || line.starts_with("* ") {
                    let content = line.trim_start_matches("- ").trim_start_matches("* ").trim();
                    if !content.is_empty() {
                        highlights.push(ReportHighlight {
                            title: "完成事项".to_string(),
                            description: content.to_string(),
                            time_range: "全天".to_string(),
                        });
                    }
                }
            }
        }

        // 限制亮点数量
        highlights.truncate(5);
        highlights
    }

    /// 从 Markdown 中提取洞察
    fn extract_insights_from_markdown(markdown: &str) -> Vec<ReportInsight> {
        let mut insights = Vec::new();

        // 从 "自我评估" 章节提取洞察
        if let Some(section) = Self::extract_section(markdown, &["## 🔍 自我评估", "## 自我评估"]) {
            // 查找 "做得好的地方"
            if let Some(good_part) = section.find("做得好") {
                let remaining = &section[good_part..];
                if let Some(content) = Self::extract_subsection_content(remaining) {
                    insights.push(ReportInsight {
                        category: "productivity".to_string(),
                        content: format!("优势：{}", content),
                    });
                }
            }

            // 查找 "需要改进的地方"
            if let Some(improve_part) = section.find("需要改进") {
                let remaining = &section[improve_part..];
                if let Some(content) = Self::extract_subsection_content(remaining) {
                    insights.push(ReportInsight {
                        category: "suggestions".to_string(),
                        content: format!("改进建议：{}", content),
                    });
                }
            }
        }

        // 从 "学习与成长" 章节提取洞察
        if let Some(section) = Self::extract_section(markdown, &["## 📚 学习与成长", "## 学习与成长"]) {
            let content = section.lines()
                .filter(|line| {
                    let l = line.trim();
                    l.starts_with("- ") || l.starts_with("* ")
                })
                .take(3)
                .map(|line| line.trim_start_matches("- ").trim_start_matches("* ").trim())
                .collect::<Vec<_>>()
                .join("；");

            if !content.is_empty() {
                insights.push(ReportInsight {
                    category: "focus".to_string(),
                    content: format!("学习收获：{}", content),
                });
            }
        }

        insights
    }

    /// 提取子章节内容（直到遇到下一个三级标题或章节结束）
    fn extract_subsection_content(text: &str) -> Option<String> {
        // 跳过标题行
        let lines: Vec<&str> = text.lines().skip(1).collect();
        let mut content_lines = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            // 遇到新的子标题则停止
            if trimmed.starts_with("###") || trimmed.starts_with("## ") {
                break;
            }
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                content_lines.push(trimmed.trim_start_matches("- ").trim_start_matches("* "));
            }
        }

        if content_lines.is_empty() {
            None
        } else {
            Some(content_lines.join("；"))
        }
    }

    /// 计算活动分类统计
    fn calculate_activity_breakdown(contexts: &[ScreenContext]) -> Result<Vec<ActivityBreakdown>> {
        let mut activity_counts: HashMap<String, i32> = HashMap::new();
        let total = contexts.len() as i32;

        for context in contexts {
            *activity_counts.entry(context.activity_type.clone()).or_insert(0) += 1;
        }

        let breakdown: Vec<ActivityBreakdown> = activity_counts
            .into_iter()
            .map(|(activity_type, count)| ActivityBreakdown {
                activity_type,
                count,
                percentage: (count as f32 / total as f32) * 100.0,
                duration_minutes: None, // 可以后续根据时间戳计算
            })
            .collect();

        Ok(breakdown)
    }

    /// 保存日报到数据库
    pub fn save_report(conn: &Connection, report: &DailyReport) -> Result<i64> {
        // 检查是否已存在该日期的日报
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM daily_reports WHERE date = ?",
            params![report.date],
            |row| row.get(0),
        )?;

        if exists > 0 {
            // 更新现有日报
            conn.execute(
                "UPDATE daily_reports
                 SET summary_text = ?, highlights = ?, insights = ?,
                     total_screenshots = ?, activity_breakdown = ?
                 WHERE date = ?",
                params![
                    report.summary_text,
                    report.highlights,
                    report.insights,
                    report.total_screenshots,
                    report.activity_breakdown,
                    report.date,
                ],
            )?;

            // 获取已存在记录的 ID
            let id: i64 = conn.query_row(
                "SELECT id FROM daily_reports WHERE date = ?",
                params![report.date],
                |row| row.get(0),
            )?;
            Ok(id)
        } else {
            // 插入新日报
            conn.execute(
                "INSERT INTO daily_reports (
                    date, summary_text, highlights, insights,
                    total_screenshots, activity_breakdown
                ) VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    report.date,
                    report.summary_text,
                    report.highlights,
                    report.insights,
                    report.total_screenshots,
                    report.activity_breakdown,
                ],
            )?;

            Ok(conn.last_insert_rowid())
        }
    }

    /// 获取指定日期的日报
    pub fn get_report(conn: &Connection, date: &str) -> Result<Option<DailyReport>> {
        let mut stmt = conn.prepare(
            "SELECT id, date, summary_text, highlights, insights,
                    total_screenshots, activity_breakdown, created_at
             FROM daily_reports
             WHERE date = ?",
        )?;

        let mut rows = stmt.query(params![date])?;

        if let Some(row) = rows.next()? {
            Ok(Some(DailyReport {
                id: Some(row.get(0)?),
                date: row.get(1)?,
                summary_text: row.get(2)?,
                highlights: row.get(3)?,
                insights: row.get(4)?,
                total_screenshots: row.get(5)?,
                activity_breakdown: row.get(6)?,
                created_at: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 获取日报列表（按日期倒序）
    pub fn list_reports(
        conn: &Connection,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<DailyReport>> {
        let limit = limit.unwrap_or(30);
        let offset = offset.unwrap_or(0);

        let mut stmt = conn.prepare(
            "SELECT id, date, summary_text, highlights, insights,
                    total_screenshots, activity_breakdown, created_at
             FROM daily_reports
             ORDER BY date DESC
             LIMIT ? OFFSET ?",
        )?;

        let reports = stmt
            .query_map(params![limit, offset], |row| {
                Ok(DailyReport {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    summary_text: row.get(2)?,
                    highlights: row.get(3)?,
                    insights: row.get(4)?,
                    total_screenshots: row.get(5)?,
                    activity_breakdown: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(reports)
    }

    /// 删除日报
    pub fn delete_report(conn: &Connection, date: &str) -> Result<()> {
        conn.execute("DELETE FROM daily_reports WHERE date = ?", params![date])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_save_and_get_report() {
        let conn = setup_test_db();

        let report = DailyReport {
            id: None,
            date: "2024-12-04".to_string(),
            summary_text: "今日主要进行代码开发工作".to_string(),
            highlights: r#"[{"title":"完成功能开发","description":"实现日报生成功能","timeRange":"10:00-12:00"}]"#.to_string(),
            insights: r#"[{"category":"productivity","content":"上午效率较高"}]"#.to_string(),
            total_screenshots: 50,
            activity_breakdown: r#"[{"activityType":"coding","count":30,"percentage":60.0}]"#.to_string(),
            created_at: None,
        };

        let id = ReportService::save_report(&conn, &report).unwrap();
        assert!(id > 0);

        let retrieved = ReportService::get_report(&conn, "2024-12-04")
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.summary_text, "今日主要进行代码开发工作");
        assert_eq!(retrieved.total_screenshots, 50);
    }

    #[test]
    fn test_list_reports() {
        let conn = setup_test_db();

        // 插入多条日报
        for i in 1..=5 {
            let report = DailyReport {
                id: None,
                date: format!("2024-12-{:02}", i),
                summary_text: format!("日报 {}", i),
                highlights: "[]".to_string(),
                insights: "[]".to_string(),
                total_screenshots: i * 10,
                activity_breakdown: "[]".to_string(),
                created_at: None,
            };
            ReportService::save_report(&conn, &report).unwrap();
        }

        let reports = ReportService::list_reports(&conn, Some(3), None).unwrap();
        assert_eq!(reports.len(), 3);
        assert_eq!(reports[0].date, "2024-12-05"); // 最新的排在前面
    }

    #[test]
    fn test_update_report() {
        let conn = setup_test_db();

        // 首次保存
        let report1 = DailyReport {
            id: None,
            date: "2024-12-04".to_string(),
            summary_text: "初始版本".to_string(),
            highlights: "[]".to_string(),
            insights: "[]".to_string(),
            total_screenshots: 10,
            activity_breakdown: "[]".to_string(),
            created_at: None,
        };
        ReportService::save_report(&conn, &report1).unwrap();

        // 更新
        let report2 = DailyReport {
            id: None,
            date: "2024-12-04".to_string(),
            summary_text: "更新版本".to_string(),
            highlights: "[]".to_string(),
            insights: "[]".to_string(),
            total_screenshots: 20,
            activity_breakdown: "[]".to_string(),
            created_at: None,
        };
        ReportService::save_report(&conn, &report2).unwrap();

        // 验证更新
        let retrieved = ReportService::get_report(&conn, "2024-12-04")
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.summary_text, "更新版本");
        assert_eq!(retrieved.total_screenshots, 20);
    }

    #[test]
    fn test_calculate_activity_breakdown() {
        let contexts = vec![
            ScreenContext {
                id: Some(1),
                captured_at: "2024-12-04 10:00:00".to_string(),
                app_name: Some("VS Code".to_string()),
                window_title: None,
                activity_type: "coding".to_string(),
                description: "编码".to_string(),
                key_content: None,
                screenshot_hash: None,
                screenshot_path: None,
                processing_time_ms: None,
            },
            ScreenContext {
                id: Some(2),
                captured_at: "2024-12-04 10:10:00".to_string(),
                app_name: Some("VS Code".to_string()),
                window_title: None,
                activity_type: "coding".to_string(),
                description: "编码".to_string(),
                key_content: None,
                screenshot_hash: None,
                screenshot_path: None,
                processing_time_ms: None,
            },
            ScreenContext {
                id: Some(3),
                captured_at: "2024-12-04 10:20:00".to_string(),
                app_name: Some("Chrome".to_string()),
                window_title: None,
                activity_type: "browsing".to_string(),
                description: "浏览".to_string(),
                key_content: None,
                screenshot_hash: None,
                screenshot_path: None,
                processing_time_ms: None,
            },
        ];

        let breakdown = ReportService::calculate_activity_breakdown(&contexts).unwrap();
        assert_eq!(breakdown.len(), 2);

        let coding = breakdown.iter().find(|b| b.activity_type == "coding").unwrap();
        assert_eq!(coding.count, 2);
        assert!((coding.percentage - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_group_by_hour() {
        let contexts = vec![
            ScreenContext {
                id: Some(1),
                captured_at: "2024-12-04 10:00:00".to_string(),
                app_name: Some("VS Code".to_string()),
                window_title: None,
                activity_type: "coding".to_string(),
                description: "编码".to_string(),
                key_content: None,
                screenshot_hash: None,
                screenshot_path: None,
                processing_time_ms: None,
            },
            ScreenContext {
                id: Some(2),
                captured_at: "2024-12-04 10:30:00".to_string(),
                app_name: Some("VS Code".to_string()),
                window_title: None,
                activity_type: "coding".to_string(),
                description: "编码".to_string(),
                key_content: None,
                screenshot_hash: None,
                screenshot_path: None,
                processing_time_ms: None,
            },
            ScreenContext {
                id: Some(3),
                captured_at: "2024-12-04 14:00:00".to_string(),
                app_name: Some("Chrome".to_string()),
                window_title: None,
                activity_type: "browsing".to_string(),
                description: "浏览".to_string(),
                key_content: None,
                screenshot_hash: None,
                screenshot_path: None,
                processing_time_ms: None,
            },
        ];

        let blocks = ReportService::group_by_hour(&contexts).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].hour, 10);
        assert_eq!(blocks[0].contexts.len(), 2);
        assert_eq!(blocks[1].hour, 14);
        assert_eq!(blocks[1].contexts.len(), 1);
    }
}
