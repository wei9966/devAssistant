// TODO Prediction Service
// 基于活动总结（Activity Summary）智能推断和生成待办任务的服务
// 优化说明：使用已汇总的活动总结数据而非原始截图，大幅减少 token 消耗

use anyhow::{anyhow, Result};
use chrono::{Duration as ChronoDuration, Local};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ai_service::{AiService, ChatMessage};
use super::prompt_db_service::PromptDbService;
use super::task_service::TaskService;
use crate::models::task::{TaskCategory, TaskPriority};

/// 活动总结数据（用于 TODO 预测）
#[derive(Debug, Clone)]
struct ActivitySummaryData {
    pub id: i64,
    pub start_time: String,
    pub end_time: String,
    pub summary_text: String,
    pub activity_type: String,
    pub created_at: String,
}

/// AI 预测的任务结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictedTask {
    pub id: Option<i64>,           // 数据库 ID（仅从数据库读取时有值）
    pub description: String,       // 任务描述
    pub reason: String,            // 生成原因
    pub priority: String,          // 优先级 (urgent/high/medium/low)
    pub due_date: Option<String>,  // 截止日期
    pub status: Option<String>,    // 状态: pending/accepted/ignored
    pub task_id: Option<i64>,      // 关联的任务 ID（接受后）
    pub created_at: Option<String>,// 创建时间
}

/// 预测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionResult {
    pub tasks: Vec<PredictedTask>,
    pub analyzed_contexts: i32,
    pub generated_at: String,
}

/// TODO 预测服务
pub struct TodoPredictionService {
    db_path: String,
}

impl TodoPredictionService {
    /// 创建新的服务实例
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }

    /// 获取数据库连接
    fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
            .map_err(|e| anyhow!("打开数据库失败: {}", e))
    }

    /// 获取指定时间范围内的活动总结记录
    ///
    /// 根据配置的预测间隔时间，查询该时间段内的所有活动总结
    /// 例如：如果间隔是 120 分钟，则查询最近 120 分钟内的活动总结
    fn get_activity_summaries_in_range(&self, conn: &Connection, interval_minutes: i64) -> Result<Vec<ActivitySummaryData>> {
        let since = Local::now() - ChronoDuration::minutes(interval_minutes);
        let since_str = since.format("%Y-%m-%d %H:%M:%S").to_string();

        // 不使用 LIMIT，而是根据时间范围查询所有记录
        let mut stmt = conn.prepare(
            "SELECT id, start_time, end_time, summary_text, activity_type, created_at
             FROM activity_summaries
             WHERE start_time >= ?
             ORDER BY start_time ASC",
        )?;

        let summaries = stmt
            .query_map(params![since_str], |row| {
                Ok(ActivitySummaryData {
                    id: row.get(0)?,
                    start_time: row.get(1)?,
                    end_time: row.get(2)?,
                    summary_text: row.get(3)?,
                    activity_type: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(summaries)
    }

    /// 获取历史任务列表（用于去重），返回格式化的字符串
    fn get_historical_todos(&self, conn: &Connection, limit: i32) -> Result<String> {
        let mut stmt = conn.prepare(
            "SELECT title, description, created_at, status
             FROM tasks
             ORDER BY created_at DESC
             LIMIT ?",
        )?;

        let tasks: Vec<String> = stmt
            .query_map(params![limit], |row| {
                let title: String = row.get(0)?;
                let description: Option<String> = row.get(1)?;
                let created_at: Option<String> = row.get(2)?;
                let status: String = row.get(3)?;

                let desc = description.unwrap_or_default();
                let time = created_at.unwrap_or_default();

                Ok(format!(
                    "[{}] {} - {} (状态: {})",
                    time, title, desc, status
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if tasks.is_empty() {
            Ok("暂无历史任务".to_string())
        } else {
            Ok(tasks.join("\n"))
        }
    }

    /// 将活动总结格式化为提示词所需的字符串
    /// 活动总结已经是汇总后的数据，token 消耗较小且信息完整
    fn format_activity_summaries(&self, summaries: &[ActivitySummaryData]) -> String {
        if summaries.is_empty() {
            return "暂无活动记录".to_string();
        }

        summaries
            .iter()
            .map(|s| {
                format!(
                    "[{} ~ {}] ({}) {}",
                    s.start_time, s.end_time, s.activity_type, s.summary_text
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// 核心方法：预测待办任务
    ///
    /// 优化：基于活动总结（Activity Summary）而非原始截图数据进行预测
    /// 这样可以避免 token 过大的问题，同时保留完整的活动信息
    ///
    /// 参数：
    /// - ai_service: AI 服务实例
    /// - interval_minutes: 预测间隔时间（分钟），查询该时间段内的活动总结
    ///   如果为 None，默认使用 120 分钟（2 小时）
    pub async fn predict_todos(
        &self,
        ai_service: &AiService,
        interval_minutes: Option<i64>,
    ) -> Result<PredictionResult> {
        let conn = self.get_connection()?;

        // 1. 根据间隔时间获取活动总结（默认 120 分钟）
        let interval = interval_minutes.unwrap_or(120);
        let summaries = self.get_activity_summaries_in_range(&conn, interval)?;
        let analyzed_count = summaries.len() as i32;

        log::info!(
            "[TODO预测] 查询最近 {} 分钟的活动总结，获取到 {} 条记录",
            interval,
            analyzed_count
        );

        if summaries.is_empty() {
            log::info!("[TODO预测] 时间范围内没有活动总结数据，跳过预测");
            return Ok(PredictionResult {
                tasks: Vec::new(),
                analyzed_contexts: 0,
                generated_at: Local::now().to_rfc3339(),
            });
        }

        // 2. 获取历史任务（最近 20 条，用于去重）
        let historical_todos = self.get_historical_todos(&conn, 20)?;

        // 3. 格式化活动总结数据（已汇总的数据，token 消耗小）
        let context_data = self.format_activity_summaries(&summaries);
        let context_chars = context_data.chars().count();
        log::info!(
            "[TODO预测] 活动总结数据长度: {} 字符",
            context_chars
        );

        // 4. 准备提示词变量
        let mut vars = HashMap::new();
        vars.insert("current_time".to_string(), Local::now().to_rfc3339());
        vars.insert("historical_todos".to_string(), historical_todos);
        vars.insert("context_data".to_string(), context_data);

        // 5. 渲染提示词（从数据库加载 todo_extraction）
        let rendered = PromptDbService::render_prompt_cached("todo_extraction", &vars)
            .map_err(|e| anyhow!("获取提示词失败: {}", e))?;

        // 6. 调用 AI 服务生成预测任务
        let messages = if let Some(system) = rendered.system {
            vec![
                ChatMessage::system(system),
                ChatMessage::user(rendered.user),
            ]
        } else {
            vec![ChatMessage::user(rendered.user)]
        };

        let response = ai_service
            .chat_with_log(&conn, "todo_prediction", "predict", messages)
            .await?;

        log::info!(
            "[TODO预测] AI 响应长度: {} 字符",
            response.chars().count()
        );

        // 7. 解析 JSON 响应
        let predicted_tasks = self.parse_prediction_response(&response)?;

        log::info!(
            "[TODO预测] 解析得到 {} 个预测任务",
            predicted_tasks.len()
        );

        // 8. 返回预测结果
        Ok(PredictionResult {
            tasks: predicted_tasks,
            analyzed_contexts: analyzed_count,
            generated_at: Local::now().to_rfc3339(),
        })
    }

    /// 解析 AI 响应的 JSON 数组
    fn parse_prediction_response(&self, response: &str) -> Result<Vec<PredictedTask>> {
        // 检查响应是否为空
        if response.trim().is_empty() {
            log::warn!("[TODO预测] AI 响应为空");
            return Ok(Vec::new());
        }

        // 尝试提取 JSON 数组部分
        let json_str = if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                &response[start..=end]
            } else {
                log::warn!("[TODO预测] 找到 '[' 但未找到 ']'");
                response
            }
        } else {
            // 如果没有找到数组，可能是空响应或者 AI 返回了非 JSON 格式
            log::warn!(
                "[TODO预测] 未找到 JSON 数组，响应前 200 字符: {}",
                response.chars().take(200).collect::<String>()
            );
            return Ok(Vec::new());
        };

        // 尝试解析 JSON
        let tasks: Vec<PredictedTask> = serde_json::from_str(json_str).map_err(|e| {
            log::error!(
                "[TODO预测] JSON 解析失败: {}. JSON 内容前 500 字符: {}",
                e,
                json_str.chars().take(500).collect::<String>()
            );
            anyhow!("解析 AI 响应失败: {}", e)
        })?;

        Ok(tasks)
    }

    /// 将预测任务转换为实际任务并保存到数据库（可选，用户确认后才创建）
    pub async fn create_tasks_from_predictions(
        &self,
        predictions: &[PredictedTask],
    ) -> Result<Vec<i64>> {
        let conn = self.get_connection()?;
        let mut created_ids = Vec::new();

        for predicted in predictions {
            // 解析优先级
            let priority = match predicted.priority.to_lowercase().as_str() {
                "urgent" => TaskPriority::High,  // urgent 映射为 High
                "high" => TaskPriority::High,
                "medium" => TaskPriority::Medium,
                "low" => TaskPriority::Low,
                _ => TaskPriority::Medium,
            };

            // 构建任务描述（包含生成原因）
            let description = if !predicted.reason.is_empty() {
                Some(format!("{}\n\n生成原因：{}", predicted.description, predicted.reason))
            } else {
                Some(predicted.description.clone())
            };

            // 创建任务
            let task_id = TaskService::create_task(
                &conn,
                &predicted.description,
                description.as_deref(),
                TaskCategory::Other, // 默认分类，可以后续通过 AI 分类
                priority,
            )?;

            // 如果有截止日期，更新任务
            if let Some(due_date) = &predicted.due_date {
                conn.execute(
                    "UPDATE tasks SET due_date = ? WHERE id = ?",
                    params![due_date, task_id],
                )?;
            }

            created_ids.push(task_id);
        }

        Ok(created_ids)
    }

    /// 批量创建任务（便捷方法）
    pub async fn predict_and_create_tasks(
        &self,
        ai_service: &AiService,
        interval_minutes: Option<i64>,
    ) -> Result<(PredictionResult, Vec<i64>)> {
        let result = self.predict_todos(ai_service, interval_minutes).await?;
        let task_ids = self.create_tasks_from_predictions(&result.tasks).await?;
        Ok((result, task_ids))
    }

    /// 保存预测任务到 predicted_tasks 表（不直接创建为任务）
    pub fn save_predictions(&self, predictions: &[PredictedTask]) -> Result<Vec<i64>> {
        let conn = self.get_connection()?;
        let mut ids = Vec::new();

        for pred in predictions {
            conn.execute(
                "INSERT INTO predicted_tasks (description, reason, priority, due_date, status)
                 VALUES (?, ?, ?, ?, 'pending')",
                params![
                    pred.description,
                    pred.reason,
                    pred.priority,
                    pred.due_date,
                ],
            )?;
            ids.push(conn.last_insert_rowid());
        }

        Ok(ids)
    }

    /// 预测并保存到 predicted_tasks 表（推荐使用，不直接创建任务）
    ///
    /// 参数：
    /// - ai_service: AI 服务实例
    /// - interval_minutes: 预测间隔时间（分钟），查询该时间段内的活动总结
    ///   如果为 None，默认使用 120 分钟（2 小时）
    pub async fn predict_and_save(
        &self,
        ai_service: &AiService,
        interval_minutes: Option<i64>,
    ) -> Result<PredictionResult> {
        let result = self.predict_todos(ai_service, interval_minutes).await?;

        if !result.tasks.is_empty() {
            self.save_predictions(&result.tasks)?;
        }

        Ok(result)
    }

    /// 获取待处理的预测任务列表
    pub fn get_pending_predictions(&self) -> Result<Vec<PredictedTask>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, description, reason, priority, due_date, status, task_id, created_at
             FROM predicted_tasks
             WHERE status = 'pending'
             ORDER BY
                CASE priority
                    WHEN 'urgent' THEN 1
                    WHEN 'high' THEN 2
                    WHEN 'medium' THEN 3
                    WHEN 'low' THEN 4
                    ELSE 5
                END,
                created_at DESC",
        )?;

        let tasks = stmt
            .query_map([], |row| {
                Ok(PredictedTask {
                    id: Some(row.get(0)?),
                    description: row.get(1)?,
                    reason: row.get(2)?,
                    priority: row.get(3)?,
                    due_date: row.get(4)?,
                    status: row.get(5)?,
                    task_id: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// 获取所有预测任务列表（包含已处理的）
    pub fn get_all_predictions(&self, limit: i32) -> Result<Vec<PredictedTask>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, description, reason, priority, due_date, status, task_id, created_at
             FROM predicted_tasks
             ORDER BY created_at DESC
             LIMIT ?",
        )?;

        let tasks = stmt
            .query_map(params![limit], |row| {
                Ok(PredictedTask {
                    id: Some(row.get(0)?),
                    description: row.get(1)?,
                    reason: row.get(2)?,
                    priority: row.get(3)?,
                    due_date: row.get(4)?,
                    status: row.get(5)?,
                    task_id: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// 接受预测任务（创建为实际任务）
    pub fn accept_prediction(&self, prediction_id: i64) -> Result<i64> {
        let conn = self.get_connection()?;

        // 获取预测任务
        let pred: PredictedTask = conn.query_row(
            "SELECT id, description, reason, priority, due_date, status, task_id, created_at
             FROM predicted_tasks WHERE id = ?",
            params![prediction_id],
            |row| {
                Ok(PredictedTask {
                    id: Some(row.get(0)?),
                    description: row.get(1)?,
                    reason: row.get(2)?,
                    priority: row.get(3)?,
                    due_date: row.get(4)?,
                    status: row.get(5)?,
                    task_id: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        )?;

        // 解析优先级
        let priority = match pred.priority.to_lowercase().as_str() {
            "urgent" => TaskPriority::High,
            "high" => TaskPriority::High,
            "medium" => TaskPriority::Medium,
            "low" => TaskPriority::Low,
            _ => TaskPriority::Medium,
        };

        // 构建任务描述
        let description = if !pred.reason.is_empty() {
            Some(format!("{}\n\n生成原因：{}", pred.description, pred.reason))
        } else {
            Some(pred.description.clone())
        };

        // 创建任务
        let task_id = TaskService::create_task(
            &conn,
            &pred.description,
            description.as_deref(),
            TaskCategory::Other,
            priority,
        )?;

        // 如果有截止日期，更新任务
        if let Some(due_date) = &pred.due_date {
            conn.execute(
                "UPDATE tasks SET due_date = ? WHERE id = ?",
                params![due_date, task_id],
            )?;
        }

        // 更新预测任务状态
        conn.execute(
            "UPDATE predicted_tasks SET status = 'accepted', task_id = ?, processed_at = datetime('now', 'localtime')
             WHERE id = ?",
            params![task_id, prediction_id],
        )?;

        Ok(task_id)
    }

    /// 忽略预测任务
    pub fn ignore_prediction(&self, prediction_id: i64) -> Result<()> {
        let conn = self.get_connection()?;

        conn.execute(
            "UPDATE predicted_tasks SET status = 'ignored', processed_at = datetime('now', 'localtime')
             WHERE id = ?",
            params![prediction_id],
        )?;

        Ok(())
    }

    /// 批量接受预测任务
    pub fn accept_predictions(&self, prediction_ids: &[i64]) -> Result<Vec<i64>> {
        let mut task_ids = Vec::new();
        for &id in prediction_ids {
            let task_id = self.accept_prediction(id)?;
            task_ids.push(task_id);
        }
        Ok(task_ids)
    }

    /// 批量忽略预测任务
    pub fn ignore_predictions(&self, prediction_ids: &[i64]) -> Result<()> {
        let conn = self.get_connection()?;

        for &id in prediction_ids {
            conn.execute(
                "UPDATE predicted_tasks SET status = 'ignored', processed_at = datetime('now', 'localtime')
                 WHERE id = ?",
                params![id],
            )?;
        }

        Ok(())
    }

    /// 获取待处理的预测任务数量
    pub fn get_pending_count(&self) -> Result<i32> {
        let conn = self.get_connection()?;

        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM predicted_tasks WHERE status = 'pending'",
            [],
            |row| row.get(0),
        )?;

        Ok(count)
    }

    /// 清理过期的预测任务（超过 7 天的已处理任务）
    pub fn cleanup_old_predictions(&self, days: i32) -> Result<i32> {
        let conn = self.get_connection()?;

        let deleted = conn.execute(
            "DELETE FROM predicted_tasks
             WHERE status != 'pending'
             AND created_at < datetime('now', 'localtime', ? || ' days')",
            params![format!("-{}", days)],
        )?;

        Ok(deleted as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_prediction_response() {
        let service = TodoPredictionService::new(":memory:".to_string());

        // 测试有效的 JSON 响应（AI 返回的格式不包含 id, status 等字段）
        let response = r#"[
            {
                "description": "完成功能开发",
                "reason": "基于截图分析",
                "priority": "high",
                "dueDate": "2024-01-15"
            }
        ]"#;

        let result = service.parse_prediction_response(response);
        assert!(result.is_ok());
        let tasks = result.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].description, "完成功能开发");
        assert_eq!(tasks[0].priority, "high");
    }

    #[test]
    fn test_parse_empty_response() {
        let service = TodoPredictionService::new(":memory:".to_string());

        // 测试空数组
        let response = "[]";
        let result = service.parse_prediction_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);

        // 测试没有数组的响应
        let response = "暂无任务";
        let result = service.parse_prediction_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_format_activity_summaries() {
        let service = TodoPredictionService::new(":memory:".to_string());

        // 测试空活动总结
        let summaries: Vec<ActivitySummaryData> = Vec::new();
        let result = service.format_activity_summaries(&summaries);
        assert_eq!(result, "暂无活动记录");

        // 测试有活动总结
        let summaries = vec![ActivitySummaryData {
            id: 1,
            start_time: "2024-01-15 10:00:00".to_string(),
            end_time: "2024-01-15 10:15:00".to_string(),
            summary_text: "在VSCode中编写Rust代码，完成了TODO预测服务的重构".to_string(),
            activity_type: "coding".to_string(),
            created_at: "2024-01-15 10:15:00".to_string(),
        }];

        let result = service.format_activity_summaries(&summaries);
        assert!(result.contains("VSCode"));
        assert!(result.contains("coding"));
        assert!(result.contains("2024-01-15 10:00:00"));
    }
}
