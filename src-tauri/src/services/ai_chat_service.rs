use anyhow::{anyhow, Result};
use chrono::Timelike;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::services::ai_service::{AiService, ChatMessage};
use crate::services::task_service::TaskService;
use crate::services::pomodoro_service::PomodoroService;
use crate::services::work_log_service::WorkLogService;
use crate::models::task::{TaskCategory, TaskPriority};

/// 安全截断字符串，确保不会切到 UTF-8 字符中间
fn safe_truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_chars).collect::<String>())
    }
}

/// AI 聊天响应类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    Text,
    FunctionCall,
    Error,
}

/// AI 聊天响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub content: String,
    pub response_type: String,
    pub data: Option<serde_json::Value>,
}

/// Function Call 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// 仪表盘统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub todo_count: i32,
    pub today_done_count: i32,
    pub today_focus_minutes: i32,
    pub weekly_done_count: i32,
}

/// AI 聊天服务
pub struct AiChatService;

impl AiChatService {
    /// 获取仪表盘统计数据用于上下文注入
    pub fn get_dashboard_stats(conn: &Connection) -> Result<DashboardStats> {
        // 获取待办任务数
        let todo_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE status IN ('todo', 'active')",
            [],
            |row| row.get(0),
        )?;

        // 获取今日完成任务数
        let today_done_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE status = 'done' AND date(completed_at) = date('now', 'localtime')",
            [],
            |row| row.get(0),
        )?;

        // 获取今日专注时长（分钟）
        let today_focus_minutes: i32 = conn.query_row(
            "SELECT COALESCE(SUM(focus_time), 0) FROM pomodoro_sessions WHERE date(created_at) = date('now', 'localtime')",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        // 获取本周完成任务数
        let weekly_done_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE status = 'done' AND date(completed_at) >= date('now', 'localtime', 'weekday 0', '-6 days')",
            [],
            |row| row.get(0),
        )?;

        Ok(DashboardStats {
            todo_count,
            today_done_count,
            today_focus_minutes,
            weekly_done_count,
        })
    }

    /// 构建系统提示词（包含用户数据上下文）
    pub fn build_system_prompt(conn: &Connection) -> Result<String> {
        let stats = Self::get_dashboard_stats(conn)?;

        // 获取当前时间段的问候语
        let hour = chrono::Local::now().hour();
        let greeting = if hour < 6 {
            "夜深了，还在工作吗？"
        } else if hour < 12 {
            "早上好！新的一天开始了"
        } else if hour < 14 {
            "中午好！别忘了休息一下"
        } else if hour < 18 {
            "下午好！继续加油"
        } else {
            "晚上好！辛苦了"
        };

        // 获取当前日期和时间
        let now = chrono::Local::now();
        let current_date = now.format("%Y-%m-%d").to_string();
        let current_datetime = now.format("%Y年%m月%d日 %H:%M").to_string();

        let system_prompt = format!(
            r#"你是 DevAssistant 的 AI 助手小D，一个友好、有亲和力的工作伙伴。你的说话风格应该像朋友一样自然亲切，而不是冷冰冰的机器人。

{} 现在是北京时间 {}。
【重要】今天的日期是：{}。当用户要求生成"今天的日报"或"日报"时，请使用这个日期。

当前用户的工作状态：
- 待办任务：{} 个
- 今日已完成：{} 个
- 今日专注时长：{} 分钟
- 本周已完成：{} 个

【你的性格特点】
- 热情友好，像朋友一样和用户聊天
- 关心用户的工作状态，适时给予鼓励
- 回答简洁有重点，但不失温度
- 会根据数据给出实用的建议
- 使用适当的表情符号让对话更生动

【数据库表结构 - 你可以查询这些数据】
你拥有查询数据库的能力。当用户询问任何与应用数据相关的问题时，你可以生成 SQL 来查询。

1. tasks（任务表）- 核心表
   - id: INTEGER 主键
   - title: TEXT 任务标题
   - description: TEXT 任务描述
   - category: TEXT 分类 (dev/work/life/other)
   - priority: INTEGER 优先级 (1=高, 2=中, 3=低)
   - status: TEXT 状态 (todo/active/done)
   - quadrant: TEXT 四象限 (urgent_important/urgent_not_important/not_urgent_important/not_urgent_not_important)
   - progress: INTEGER 进度百分比 (0-100)
   - created_at: TIMESTAMP 创建时间
   - started_at: TIMESTAMP 开始时间
   - completed_at: TIMESTAMP 完成时间
   - due_date: TEXT 截止日期
   - display_date: TEXT 日历显示日期
   - scheduled_start_time: TEXT 计划开始时间
   - notes: TEXT 备注

2. pomodoro_sessions（番茄钟会话表）
   - id: INTEGER 主键
   - task_id: INTEGER 关联任务ID
   - duration_minutes: INTEGER 专注时长（分钟）
   - status: TEXT 状态 (pending/running/completed/cancelled)
   - phase: TEXT 阶段 (prep/focus/break/done)
   - focus_goal: TEXT 专注目标
   - actual_focus_seconds: INTEGER 实际专注秒数
   - distraction_count: INTEGER 分心次数
   - focus_rate: REAL 专注率
   - started_at: TEXT 开始时间
   - completed_at: TEXT 完成时间
   - created_at: TEXT 创建时间

3. tags（标签表）
   - id: INTEGER 主键
   - name: TEXT 标签名（唯一）
   - color: TEXT 颜色

4. task_tags（任务标签关联表）
   - task_id: INTEGER
   - tag_id: INTEGER

5. sql_history（SQL历史记录表）
   - id: INTEGER 主键
   - sql_text: TEXT SQL内容
   - name: TEXT 名称
   - description: TEXT 描述
   - is_favorite: BOOLEAN 是否收藏
   - executed_at: TIMESTAMP 执行时间

6. clipboard_history（剪切板历史表）
   - id: INTEGER 主键
   - content_type: TEXT 内容类型
   - content: TEXT 内容
   - created_at: TEXT 创建时间

7. work_logs（工作日志表）
   - id: INTEGER 主键
   - date: DATE 日期
   - log_type: TEXT 日志类型
   - content: TEXT 内容
   - ai_generated: BOOLEAN 是否AI生成

8. notifications（通知表）
   - id: INTEGER 主键
   - notification_type: TEXT 通知类型
   - title: TEXT 标题
   - content: TEXT 内容
   - is_read: INTEGER 是否已读
   - created_at: TEXT 创建时间

【你可以调用的功能】
当用户的需求需要查询或操作数据时，返回 JSON 格式的 Function Call：

★★★ query_data - 自由查询数据（最强大的功能）★★★
   用法：{{"function":"query_data","arguments":{{"sql":"SELECT语句","description":"查询说明"}}}}
   说明：你可以根据上面的表结构，自由编写 SELECT 查询来回答用户的任何问题
   限制：只允许 SELECT 查询，最多返回 100 行
   示例：
   - 查询某个任务的创建时间：SELECT title, created_at FROM tasks WHERE title LIKE '%关键词%'
   - 查询今天完成的任务：SELECT * FROM tasks WHERE status='done' AND date(completed_at)=date('now','localtime')
   - 查询本周专注时长：SELECT SUM(actual_focus_seconds)/60 as minutes FROM pomodoro_sessions WHERE date(created_at) >= date('now','localtime','weekday 0','-6 days')
   - 查询高优先级待办：SELECT title, priority, created_at FROM tasks WHERE status='todo' AND priority=1

其他操作类功能：
1. create_task - 创建新任务
   用法：{{"function":"create_task","arguments":{{"title":"标题","description":"描述","priority":1-3,"tags":["标签1","标签2"]}}}}
   说明：tags 是可选的标签数组，用于给任务添加分类标签

2. update_task - 更新任务状态或内容
   用法：{{"function":"update_task","arguments":{{"task_id":ID,"status":"todo|active|done","title":"新标题","priority":1-3}}}}

3. delete_task - 删除任务
   用法：{{"function":"delete_task","arguments":{{"task_id":ID}}}}

4. complete_task - 完成任务
   用法：{{"function":"complete_task","arguments":{{"task_id":ID}}}} 或
   用法：{{"function":"complete_task","arguments":{{"title":"任务标题关键词"}}}}

5. start_pomodoro - 开始番茄钟
   用法：{{"function":"start_pomodoro","arguments":{{"task_id":ID,"duration":25}}}}

6. generate_daily_report - 生成日报
   用法：{{"function":"generate_daily_report","arguments":{{}}}}
   说明：date 参数可选，不传则默认使用今天的日期（{}）。用户说"生成日报"或"今天的日报"时，不要传 date 参数！

7. generate_weekly_report - 生成周报
   用法：{{"function":"generate_weekly_report","arguments":{{}}}}

8. analyze_efficiency - 分析工作效率
   用法：{{"function":"analyze_efficiency","arguments":{{"days":7}}}}

【智能判断规则 - 非常重要！】

★ 查询类问题（使用 query_data 生成 SELECT 语句）：
  - "xxx任务什么时候创建的" → 查询 created_at 字段
  - "xxx任务是哪天建的" → 查询 created_at 字段
  - "xxx任务的创建时间" → 查询 created_at 字段
  - "这个任务详情" → 查询任务所有字段
  - "我有多少任务" → SELECT COUNT(*)
  - "今天完成了什么" → 查询 status='done' 且 completed_at 是今天
  - "最近的番茄钟" → 查询 pomodoro_sessions

★ 操作类需求（使用专门的操作函数，不要用 query_data）：
  - "帮我创建一个任务：xxx" → 使用 create_task
  - "新建任务 xxx" → 使用 create_task
  - "添加一个任务" → 使用 create_task
  - "把任务标记为完成" → 使用 complete_task
  - "删除任务" → 使用 delete_task

★ 报表生成需求（必须使用专门的报表函数！）：
  - "生成日报" / "今天的日报" / "帮我生成今天的日报" → 使用 generate_daily_report（不传 date 参数）
  - "生成周报" / "本周报告" → 使用 generate_weekly_report
  - "分析效率" / "工作效率分析" → 使用 analyze_efficiency
  ⚠️ 绝对不要用 query_data 自己查询来生成日报！必须用 generate_daily_report！

★ 关键区分：
  - "什么时候创建的" = 查询创建时间（SELECT created_at）
  - "帮我创建" = 新建任务（create_task）
  - 问"创建时间"≠ 执行"创建操作"
  - "生成日报" ≠ "查询任务" → 必须用 generate_daily_report！
  - 日报 = 当天完成的任务，不是当天显示/到期的任务！

★ 普通对话 → 用自然语言回复

【重要规则】
- 调用功能时，只返回纯 JSON，不要添加任何解释文字
- 遇到查询需求，必须先调用 query_data 查询，不要直接回复猜测的内容
- query_data 只能使用 SELECT 语句，绝对不能用 INSERT/UPDATE/DELETE
- 查询时注意 SQLite 的日期函数：date(), datetime(), strftime()
- 当前时间获取用：datetime('now','localtime')，当前日期用：date('now','localtime')
- 绝对不要在回复中展示或提及 SQL 语句
- 如果用户问某个任务的信息，先用 SELECT 查询，不要用 INSERT 创建
- 绝对禁止编造任务名称、时间等数据！如果查不到就说查不到，不要瞎编
- 你不知道用户有什么任务，必须通过 query_data 查询才能知道"#,
            greeting,
            current_datetime,
            current_date,
            stats.todo_count,
            stats.today_done_count,
            stats.today_focus_minutes,
            stats.weekly_done_count,
            current_date  // 用于 generate_daily_report 说明中的日期占位符
        );

        Ok(system_prompt)
    }

    /// 纯异步调用 AI API（不需要 Connection，可以跨 await）
    pub async fn call_ai(
        ai_service: &AiService,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String> {
        // 构建消息列表
        let messages = vec![
            ChatMessage::system(system_prompt.to_string()),
            ChatMessage::user(user_message.to_string()),
        ];

        // 调用 AI API
        let response = ai_service.chat(messages).await?;
        Ok(response)
    }

    /// 解析 AI 返回的 Function Call（公开方法）
    pub fn parse_function_call(response: &str) -> Result<FunctionCall> {
        // 尝试从响应中提取 JSON
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            return Err(anyhow!("非 Function Call 响应"));
        };

        let parsed: serde_json::Value = serde_json::from_str(json_str)?;

        let function_name = parsed["function"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 function 字段"))?
            .to_string();

        let arguments = serde_json::to_string(&parsed["arguments"])?;

        Ok(FunctionCall {
            name: function_name,
            arguments,
        })
    }

    /// 同步执行 Function Call（不支持需要异步 AI 调用的函数）
    pub fn execute_function_sync(
        conn: &Connection,
        function_call: &FunctionCall,
    ) -> Result<ChatResponse> {
        let args: serde_json::Value = serde_json::from_str(&function_call.arguments)?;

        match function_call.name.as_str() {
            "query_data" => Self::handle_query_data(conn, &args),
            "get_tasks" => Self::handle_get_tasks(conn, &args),
            "get_task_detail" => Self::handle_get_task_detail(conn, &args),
            "search_tasks" => Self::handle_search_tasks(conn, &args),
            "search_sql" => Self::handle_search_sql(conn, &args),
            "get_time_stats" => Self::handle_get_time_stats(conn, &args),
            "create_task" => Self::handle_create_task(conn, &args),
            "update_task" => Self::handle_update_task(conn, &args),
            "delete_task" => Self::handle_delete_task(conn, &args),
            "complete_task" => Self::handle_complete_task(conn, &args),
            "start_pomodoro" => Self::handle_start_pomodoro(conn, &args),
            "analyze_efficiency" => Self::handle_analyze_efficiency(conn, &args),
            "generate_daily_report" => {
                let default_date = chrono::Local::now().format("%Y-%m-%d").to_string();
                let date = args["date"].as_str().unwrap_or(&default_date);

                // 调试日志：打印查询的日期
                log::info!("[AI Chat] 📅 生成日报，日期: {}", date);

                // 构建日期范围（当天 00:00:00 到 23:59:59）
                let date_start = format!("{} 00:00:00", date);
                let date_end = format!("{} 23:59:59", date);

                // 获取当天完成的任务列表
                // 使用时间范围比较，和 TaskService::get_completed_tasks 保持一致
                let completed_tasks: Vec<(String, i32)> = conn
                    .prepare(
                        "SELECT title, priority FROM tasks
                         WHERE status = 'done'
                           AND completed_at >= ?1
                           AND completed_at <= ?2
                         ORDER BY priority ASC, completed_at",
                    )?
                    .query_map([&date_start, &date_end], |row| Ok((row.get(0)?, row.get(1)?)))?
                    .collect::<std::result::Result<Vec<_>, _>>()?;

                log::info!("[AI Chat] 📊 找到 {} 个完成的任务 (范围: {} ~ {})",
                    completed_tasks.len(), date_start, date_end);

                // 如果还是没找到，尝试用 LIKE 模式匹配日期前缀
                let completed_tasks = if completed_tasks.is_empty() {
                    log::info!("[AI Chat] 🔍 尝试使用 LIKE 模式匹配...");
                    let like_pattern = format!("{}%", date);
                    let tasks: Vec<(String, i32)> = conn
                        .prepare(
                            "SELECT title, priority FROM tasks
                             WHERE status = 'done' AND completed_at LIKE ?1
                             ORDER BY priority ASC, completed_at",
                        )?
                        .query_map([&like_pattern], |row| Ok((row.get(0)?, row.get(1)?)))?
                        .collect::<std::result::Result<Vec<_>, _>>()?;
                    log::info!("[AI Chat] 📊 LIKE 模式找到 {} 个任务", tasks.len());
                    tasks
                } else {
                    completed_tasks
                };

                // 获取当天专注时长
                let focus_minutes: i32 = conn.query_row(
                    "SELECT COALESCE(SUM(focus_time), 0) FROM pomodoro_sessions
                     WHERE created_at >= ?1 AND created_at <= ?2",
                    [&date_start, &date_end],
                    |row| row.get(0),
                ).unwrap_or(0);

                // 获取番茄钟数量
                let pomodoro_count: i32 = conn.query_row(
                    "SELECT COUNT(*) FROM pomodoro_sessions
                     WHERE created_at >= ?1 AND created_at <= ?2",
                    [&date_start, &date_end],
                    |row| row.get(0),
                ).unwrap_or(0);

                log::info!("[AI Chat] 📊 专注时长: {} 分钟, 番茄钟: {} 个", focus_minutes, pomodoro_count);

                let mut content = format!("## 📅 {} 工作日报\n\n", date);

                // 数据概览
                let hours = focus_minutes / 60;
                let mins = focus_minutes % 60;
                content.push_str("### 📊 今日概览\n\n");
                content.push_str(&format!("| 指标 | 数据 |\n"));
                content.push_str(&format!("|------|------|\n"));
                content.push_str(&format!("| ✅ 完成任务 | {} 个 |\n", completed_tasks.len()));
                content.push_str(&format!("| ⏱️ 专注时长 | {} 小时 {} 分钟 |\n", hours, mins));
                content.push_str(&format!("| 🍅 番茄钟 | {} 个 |\n\n", pomodoro_count));

                // 完成的任务
                content.push_str("### ✅ 完成的任务\n\n");
                if completed_tasks.is_empty() {
                    content.push_str("今天暂无完成的任务记录。\n\n");
                } else {
                    for (i, (title, priority)) in completed_tasks.iter().enumerate() {
                        let priority_emoji = match *priority {
                            1 => "🔴",
                            2 => "🟡",
                            _ => "🟢",
                        };
                        content.push_str(&format!("{}. {} {}\n", i + 1, priority_emoji, title));
                    }
                    content.push_str("\n");
                }

                // 日报总结
                content.push_str("### 💬 日报总结\n\n");
                if completed_tasks.is_empty() && focus_minutes == 0 {
                    content.push_str("今天似乎是休息日，好好放松吧！\n");
                } else if completed_tasks.len() >= 5 {
                    content.push_str("🌟 今天效率很高，完成了大量任务！继续保持这个状态。\n");
                } else if completed_tasks.len() >= 2 {
                    content.push_str("👍 今天稳步推进，完成了既定目标。\n");
                } else {
                    content.push_str("💪 每一步进步都值得肯定，明天继续加油！\n");
                }

                let task_titles: Vec<String> = completed_tasks.iter().map(|(t, _)| t.clone()).collect();

                // 保存日报到工作日志表
                if let Err(e) = WorkLogService::save_work_log(conn, date, "daily", &content, true) {
                    log::warn!("[AI Chat] 保存日报到工作日志失败: {}", e);
                } else {
                    log::info!("[AI Chat] ✅ 日报已保存到工作日志，日期: {}", date);
                }

                Ok(ChatResponse {
                    content,
                    response_type: "daily_report".to_string(),
                    data: Some(json!({
                        "date": date,
                        "tasks": task_titles,
                        "task_count": completed_tasks.len(),
                        "focus_minutes": focus_minutes,
                        "pomodoro_count": pomodoro_count,
                        "saved": true
                    })),
                })
            }
            "generate_weekly_report" => {
                // 获取本周统计
                let completed_count: i32 = conn.query_row(
                    "SELECT COUNT(*) FROM tasks WHERE status = 'done'
                     AND date(completed_at) >= date('now', 'localtime', 'weekday 0', '-6 days')",
                    [],
                    |row| row.get(0),
                )?;

                let focus_minutes: i32 = conn.query_row(
                    "SELECT COALESCE(SUM(focus_time), 0) FROM pomodoro_sessions
                     WHERE date(created_at) >= date('now', 'localtime', 'weekday 0', '-6 days')",
                    [],
                    |row| row.get(0),
                ).unwrap_or(0);

                let pomodoro_count: i32 = conn.query_row(
                    "SELECT COUNT(*) FROM pomodoro_sessions
                     WHERE date(created_at) >= date('now', 'localtime', 'weekday 0', '-6 days')",
                    [],
                    |row| row.get(0),
                ).unwrap_or(0);

                // 获取本周完成的任务列表（按优先级排序，最多10个）
                let completed_tasks: Vec<(String, i32)> = conn
                    .prepare(
                        "SELECT title, priority FROM tasks
                         WHERE status = 'done'
                         AND date(completed_at) >= date('now', 'localtime', 'weekday 0', '-6 days')
                         ORDER BY priority ASC, completed_at DESC
                         LIMIT 10",
                    )?
                    .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                    .collect::<std::result::Result<Vec<_>, _>>()?;

                // 获取待办任务数
                let pending_count: i32 = conn.query_row(
                    "SELECT COUNT(*) FROM tasks WHERE status IN ('todo', 'active')",
                    [],
                    |row| row.get(0),
                ).unwrap_or(0);

                let hours = focus_minutes / 60;
                let mins = focus_minutes % 60;

                let mut content = String::from("## 📈 本周工作总结\n\n");

                // 周报概览
                content.push_str("### 📊 周度数据\n\n");
                content.push_str(&format!("| 指标 | 数据 |\n"));
                content.push_str(&format!("|------|------|\n"));
                content.push_str(&format!("| ✅ 完成任务 | {} 个 |\n", completed_count));
                content.push_str(&format!("| ⏱️ 专注时长 | {} 小时 {} 分钟 |\n", hours, mins));
                content.push_str(&format!("| 🍅 番茄钟 | {} 个 |\n", pomodoro_count));
                content.push_str(&format!("| 📋 待办积压 | {} 个 |\n\n", pending_count));

                // 主要完成事项
                content.push_str("### ✅ 主要完成事项\n\n");
                if completed_tasks.is_empty() {
                    content.push_str("本周暂无完成的任务记录。\n\n");
                } else {
                    for (i, (title, priority)) in completed_tasks.iter().enumerate() {
                        let priority_emoji = match *priority {
                            1 => "🔴",
                            2 => "🟡",
                            _ => "🟢",
                        };
                        content.push_str(&format!("{}. {} {}\n", i + 1, priority_emoji, title));
                    }
                    if completed_count as usize > completed_tasks.len() {
                        content.push_str(&format!("\n...还有 {} 个任务\n", completed_count as usize - completed_tasks.len()));
                    }
                    content.push_str("\n");
                }

                // 周报总结
                content.push_str("### 💬 周度评估\n\n");
                let avg_daily = completed_count as f32 / 7.0;
                let avg_focus = focus_minutes / 7;

                if completed_count >= 15 {
                    content.push_str("🌟 **出色的一周**！任务完成量超出预期，效率很高。\n\n");
                } else if completed_count >= 7 {
                    content.push_str("👍 **稳定的一周**，保持了良好的工作节奏。\n\n");
                } else if completed_count > 0 {
                    content.push_str("💪 本周有一些进展，下周可以尝试更有计划地安排任务。\n\n");
                } else {
                    content.push_str("📝 本周任务完成较少，建议下周制定更明确的目标。\n\n");
                }

                content.push_str(&format!("- 日均完成 {:.1} 个任务\n", avg_daily));
                content.push_str(&format!("- 日均专注 {} 分钟\n", avg_focus));

                if pending_count > 10 {
                    content.push_str(&format!("\n⚠️ 注意：当前积压 {} 个待办任务，建议适当清理或重新评估优先级。\n", pending_count));
                }

                Ok(ChatResponse {
                    content,
                    response_type: "weekly_report".to_string(),
                    data: Some(json!({
                        "completed_tasks": completed_count,
                        "focus_minutes": focus_minutes,
                        "pomodoro_count": pomodoro_count,
                        "pending_count": pending_count,
                        "avg_daily_tasks": format!("{:.1}", avg_daily),
                        "avg_daily_focus": avg_focus
                    })),
                })
            }
            _ => Err(anyhow!("未知的 Function: {}", function_call.name)),
        }
    }

    /// 定义可用的 AI Functions
    #[allow(dead_code)]
    fn get_function_definitions() -> Vec<serde_json::Value> {
        vec![
            json!({
                "name": "get_tasks",
                "description": "获取任务列表",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["todo", "active", "done", "all"],
                            "description": "任务状态筛选"
                        }
                    }
                }
            }),
            json!({
                "name": "search_sql",
                "description": "搜索 SQL 历史记录",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "keyword": {
                            "type": "string",
                            "description": "搜索关键词"
                        }
                    },
                    "required": ["keyword"]
                }
            }),
            json!({
                "name": "get_time_stats",
                "description": "获取时间统计数据",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "period": {
                            "type": "string",
                            "enum": ["today", "week", "month"],
                            "description": "统计周期"
                        }
                    },
                    "required": ["period"]
                }
            }),
            json!({
                "name": "create_task",
                "description": "创建新任务",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "任务标题"
                        },
                        "description": {
                            "type": "string",
                            "description": "任务描述"
                        },
                        "priority": {
                            "type": "integer",
                            "enum": [1, 2, 3],
                            "description": "优先级：1-高，2-中，3-低"
                        }
                    },
                    "required": ["title"]
                }
            }),
            json!({
                "name": "update_task",
                "description": "更新任务信息",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "integer",
                            "description": "任务 ID"
                        },
                        "title": {
                            "type": "string",
                            "description": "新的任务标题"
                        },
                        "description": {
                            "type": "string",
                            "description": "新的任务描述"
                        },
                        "priority": {
                            "type": "integer",
                            "enum": [1, 2, 3],
                            "description": "新的优先级"
                        }
                    },
                    "required": ["task_id"]
                }
            }),
            json!({
                "name": "generate_daily_report",
                "description": "生成日报",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "date": {
                            "type": "string",
                            "description": "日期（YYYY-MM-DD格式），默认今天"
                        }
                    }
                }
            }),
            json!({
                "name": "generate_weekly_report",
                "description": "生成周报",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "week_start": {
                            "type": "string",
                            "description": "周开始日期（YYYY-MM-DD格式）"
                        }
                    }
                }
            }),
            json!({
                "name": "start_pomodoro",
                "description": "开始番茄钟专注会话",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "integer",
                            "description": "关联的任务 ID"
                        },
                        "duration": {
                            "type": "integer",
                            "description": "专注时长（分钟），默认25分钟"
                        }
                    },
                    "required": ["task_id"]
                }
            }),
            json!({
                "name": "analyze_efficiency",
                "description": "分析工作效率",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "days": {
                            "type": "integer",
                            "description": "分析最近几天的数据，默认7天"
                        }
                    }
                }
            }),
        ]
    }


    /// 处理 query_data - 安全的只读 SQL 执行
    fn handle_query_data(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        let sql = args["sql"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 sql 参数"))?;
        let description = args["description"].as_str().unwrap_or("数据查询");

        // 安全检查：只允许 SELECT 语句
        let sql_upper = sql.trim().to_uppercase();
        if !sql_upper.starts_with("SELECT") {
            log::warn!("[AI Chat] ⚠️ SQL安全检查失败（可重试）：不是SELECT语句: {}", sql);
            // 返回可重试的错误，让 AI 重新生成正确的 SELECT 语句
            return Ok(ChatResponse {
                content: format!("SQL类型错误：query_data 只支持 SELECT 查询语句，但收到的是其他类型的语句"),
                response_type: "retry_sql".to_string(),
                data: Some(json!({
                    "error_type": "not_select",
                    "error_message": "query_data 只能执行 SELECT 查询，请生成 SELECT 语句",
                    "failed_sql": sql,
                    "description": description,
                })),
            });
        }

        // 安全检查：禁止危险关键字（即使在 SELECT 中也可能通过子查询注入）
        // 使用单词边界匹配，避免误判字段名（如 created_at 包含 CREATE）
        let dangerous_keywords = ["INSERT", "UPDATE", "DELETE", "DROP", "ALTER", "CREATE", "TRUNCATE", "EXEC", "EXECUTE"];
        for keyword in &dangerous_keywords {
            // 构建正则表达式，匹配独立的关键字（前后是单词边界）
            let pattern = format!(r"(?i)\b{}\b", keyword);
            if let Ok(re) = regex::Regex::new(&pattern) {
                if re.is_match(&sql_upper) {
                    log::warn!("[AI Chat] ⚠️ SQL安全检查失败（可重试）：包含危险关键字 '{}': {}", keyword, sql);
                    return Ok(ChatResponse {
                        content: format!("SQL包含不允许的关键字：{}", keyword),
                        response_type: "retry_sql".to_string(),
                        data: Some(json!({
                            "error_type": "dangerous_keyword",
                            "error_message": format!("SQL 不能包含 {} 关键字，请生成纯 SELECT 查询", keyword),
                            "failed_sql": sql,
                            "description": description,
                        })),
                    });
                }
            }
        }

        // 安全检查：白名单表
        let allowed_tables = [
            "tasks", "pomodoro_sessions", "tags", "task_tags",
            "sql_history", "clipboard_history", "work_logs", "notifications",
            "daily_reports", "weekly_plans", "task_milestones",
        ];

        // 添加 LIMIT 限制（如果没有的话）
        let sql_with_limit = if !sql_upper.contains("LIMIT") {
            format!("{} LIMIT 100", sql)
        } else {
            sql.to_string()
        };

        // 执行查询
        let mut stmt = match conn.prepare(&sql_with_limit) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("[AI Chat] ⚠️ SQL 准备失败（可重试）: {}, SQL: {}", e, sql);
                // 返回可重试的错误，包含详细信息供 AI 纠正
                return Ok(ChatResponse {
                    content: format!("SQL语法错误：{}", e),
                    response_type: "retry_sql".to_string(),
                    data: Some(json!({
                        "error_type": "syntax_error",
                        "error_message": e.to_string(),
                        "failed_sql": sql,
                        "description": description,
                    })),
                });
            }
        };

        // 获取列名
        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
        let column_count = column_names.len();

        // 执行查询并收集结果
        let mut rows: Vec<serde_json::Value> = Vec::new();
        let mut row_iter = match stmt.query([]) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("[AI Chat] ⚠️ SQL 执行失败（可重试）: {}, SQL: {}", e, sql);
                // 返回可重试的错误
                return Ok(ChatResponse {
                    content: format!("SQL执行错误：{}", e),
                    response_type: "retry_sql".to_string(),
                    data: Some(json!({
                        "error_type": "execution_error",
                        "error_message": e.to_string(),
                        "failed_sql": sql,
                        "description": description,
                    })),
                });
            }
        };

        while let Some(row) = row_iter.next()? {
            let mut row_obj = serde_json::Map::new();
            for i in 0..column_count {
                let value: serde_json::Value = match row.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                    rusqlite::types::ValueRef::Integer(v) => serde_json::Value::Number(v.into()),
                    rusqlite::types::ValueRef::Real(v) => {
                        serde_json::Number::from_f64(v)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null)
                    }
                    rusqlite::types::ValueRef::Text(v) => {
                        serde_json::Value::String(String::from_utf8_lossy(v).to_string())
                    }
                    rusqlite::types::ValueRef::Blob(v) => {
                        serde_json::Value::String(format!("[BLOB: {} bytes]", v.len()))
                    }
                };
                row_obj.insert(column_names[i].clone(), value);
            }
            rows.push(serde_json::Value::Object(row_obj));
        }

        let row_count = rows.len();

        // 如果没有查询到数据，直接返回友好的空结果消息（跳过后续润色）
        if row_count == 0 {
            log::info!("[AI Chat] ℹ️ 查询无结果: {}", description);
            return Ok(ChatResponse {
                content: format!("抱歉，没有找到相关的数据。你可以换个关键词或条件试试。"),
                response_type: "text".to_string(),
                data: None,
            });
        }

        // 生成人类可读的内容描述
        let mut content = format!("## 📊 {}\n\n", description);
        content.push_str(&format!("查询返回 **{}** 条记录\n\n", row_count));

        if row_count <= 10 {
            // 少量数据时，显示表格
            content.push_str("| ");
            content.push_str(&column_names.join(" | "));
            content.push_str(" |\n");
            content.push_str("|");
            for _ in &column_names {
                content.push_str("------|");
            }
            content.push_str("\n");

            for row in &rows {
                content.push_str("| ");
                let values: Vec<String> = column_names.iter().map(|col| {
                    match &row[col] {
                        serde_json::Value::Null => "-".to_string(),
                        serde_json::Value::String(s) => {
                            // 安全截断过长的字符串
                            safe_truncate(s, 27)
                        }
                        v => v.to_string(),
                    }
                }).collect();
                content.push_str(&values.join(" | "));
                content.push_str(" |\n");
            }
        } else {
            // 大量数据时，只显示摘要
            content.push_str("数据较多，仅显示前 5 条：\n\n");
            for (i, row) in rows.iter().take(5).enumerate() {
                content.push_str(&format!("**{}.**", i + 1));
                for (j, col) in column_names.iter().enumerate() {
                    if j < 3 {
                        // 只显示前3列
                        let val = match &row[col] {
                            serde_json::Value::Null => "-".to_string(),
                            serde_json::Value::String(s) => {
                                // 安全截断过长的字符串
                                safe_truncate(s, 17)
                            }
                            v => v.to_string(),
                        };
                        content.push_str(&format!(" {}: {}", col, val));
                        if j < 2 {
                            content.push_str(",");
                        }
                    }
                }
                content.push_str("\n");
            }
            if row_count > 5 {
                content.push_str(&format!("\n...还有 {} 条记录\n", row_count - 5));
            }
        }

        Ok(ChatResponse {
            content,
            response_type: "query_result".to_string(),
            data: Some(json!({
                "sql": sql,
                "description": description,
                "columns": column_names,
                "rows": rows,
                "row_count": row_count,
            })),
        })
    }

    /// 处理 get_tasks
    fn handle_get_tasks(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        let status = args["status"].as_str().unwrap_or("all");

        let tasks = match status {
            "done" => TaskService::get_completed_tasks(conn, 7)?,
            "all" => {
                let mut active_tasks = TaskService::get_all_tasks(conn)?;
                let completed_tasks = TaskService::get_completed_tasks(conn, 7)?;
                active_tasks.extend(completed_tasks);
                active_tasks
            }
            _ => TaskService::get_all_tasks(conn)?,
        };

        let task_count = tasks.len();

        // 构建更丰富的文本描述
        let content = if task_count == 0 {
            match status {
                "done" => "最近7天没有已完成的任务。".to_string(),
                "todo" | "active" => "当前没有待办任务，你可以创建一个新任务开始工作！".to_string(),
                _ => "当前没有任务记录。".to_string(),
            }
        } else {
            // 按优先级分组统计
            let high_priority = tasks.iter().filter(|t| t.priority == TaskPriority::High).count();
            let medium_priority = tasks.iter().filter(|t| t.priority == TaskPriority::Medium).count();
            let low_priority = tasks.iter().filter(|t| t.priority == TaskPriority::Low).count();

            // 获取前5个任务的标题
            let task_list: Vec<String> = tasks.iter()
                .take(5)
                .enumerate()
                .map(|(i, t)| {
                    let priority_emoji = match t.priority {
                        TaskPriority::High => "🔴",
                        TaskPriority::Medium => "🟡",
                        TaskPriority::Low => "🟢",
                    };
                    format!("{}. {} {}", i + 1, priority_emoji, t.title)
                })
                .collect();

            let mut description = format!(
                "找到 {} 个{}任务：\n\n",
                task_count,
                match status {
                    "done" => "已完成",
                    "todo" => "待办",
                    "active" => "进行中",
                    _ => "",
                }
            );

            // 添加优先级统计
            if high_priority > 0 || medium_priority > 0 {
                description.push_str(&format!(
                    "📊 优先级分布：高优先级 {} 个，中优先级 {} 个，低优先级 {} 个\n\n",
                    high_priority, medium_priority, low_priority
                ));
            }

            description.push_str("📋 任务列表：\n");
            description.push_str(&task_list.join("\n"));

            if task_count > 5 {
                description.push_str(&format!("\n...还有 {} 个任务", task_count - 5));
            }

            if high_priority > 0 {
                description.push_str("\n\n💡 建议：优先处理高优先级任务！");
            }

            description
        };

        // 构建前端期望的数据格式
        let tasks_for_frontend: Vec<serde_json::Value> = tasks.iter().map(|t| {
            json!({
                "id": t.id,
                "title": t.title.clone(),
                "status": format!("{:?}", t.status).to_lowercase(),
                "priority": t.priority.as_i32(),
            })
        }).collect();

        // 调试日志
        log::debug!("返回 {} 个任务到前端, tasks_for_frontend: {:?}", task_count, tasks_for_frontend);

        Ok(ChatResponse {
            content,
            response_type: "tasks".to_string(),
            data: Some(json!({
                "tasks": tasks_for_frontend,
                "total": task_count,
            })),
        })
    }

    /// 处理 get_task_detail - 获取单个任务的详细信息
    fn handle_get_task_detail(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        // 支持通过 task_id 或 title 查找
        let task_id = args["task_id"].as_i64();
        let title_keyword = args["title"].as_str();

        let task: Option<(i64, String, Option<String>, String, i32, String, Option<String>, Option<String>)> =
            if let Some(id) = task_id {
                // 通过 ID 精确查找
                conn.query_row(
                    "SELECT id, title, description, status, priority, created_at, updated_at, completed_at
                     FROM tasks WHERE id = ?1",
                    [id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,
                              row.get(5)?, row.get(6)?, row.get(7)?))
                ).ok()
            } else if let Some(keyword) = title_keyword {
                // 通过标题模糊搜索
                let search_pattern = format!("%{}%", keyword);
                conn.query_row(
                    "SELECT id, title, description, status, priority, created_at, updated_at, completed_at
                     FROM tasks WHERE title LIKE ?1 ORDER BY created_at DESC LIMIT 1",
                    [&search_pattern],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,
                              row.get(5)?, row.get(6)?, row.get(7)?))
                ).ok()
            } else {
                return Err(anyhow!("需要提供 task_id 或 title 参数"));
            };

        match task {
            Some((id, title, description, status, priority, created_at, updated_at, completed_at)) => {
                let priority_text = match priority {
                    1 => "高优先级 🔴",
                    2 => "中优先级 🟡",
                    _ => "低优先级 🟢",
                };

                let status_text = match status.as_str() {
                    "todo" => "待办",
                    "active" => "进行中",
                    "done" => "已完成",
                    _ => &status,
                };

                let mut content = format!("## 📋 任务详情\n\n");
                content.push_str(&format!("**标题**：{}\n", title));
                content.push_str(&format!("**状态**：{}\n", status_text));
                content.push_str(&format!("**优先级**：{}\n", priority_text));
                content.push_str(&format!("**创建时间**：{}\n", created_at));

                if let Some(updated) = &updated_at {
                    content.push_str(&format!("**更新时间**：{}\n", updated));
                }

                if let Some(completed) = &completed_at {
                    content.push_str(&format!("**完成时间**：{}\n", completed));
                }

                if let Some(desc) = &description {
                    if !desc.is_empty() {
                        content.push_str(&format!("\n**描述**：\n{}\n", desc));
                    }
                }

                Ok(ChatResponse {
                    content,
                    response_type: "task_detail".to_string(),
                    data: Some(json!({
                        "id": id,
                        "title": title,
                        "description": description,
                        "status": status,
                        "priority": priority,
                        "created_at": created_at,
                        "updated_at": updated_at,
                        "completed_at": completed_at,
                    })),
                })
            }
            None => {
                let search_hint = if let Some(keyword) = title_keyword {
                    format!("没有找到标题包含「{}」的任务", keyword)
                } else {
                    format!("没有找到 ID 为 {} 的任务", task_id.unwrap_or(0))
                };

                Ok(ChatResponse {
                    content: search_hint,
                    response_type: "text".to_string(),
                    data: None,
                })
            }
        }
    }

    /// 处理 search_tasks - 根据关键词搜索任务
    fn handle_search_tasks(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        let keyword = args["keyword"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 keyword 参数"))?;
        let status = args["status"].as_str().unwrap_or("all");

        let search_pattern = format!("%{}%", keyword);

        let query = match status {
            "todo" => "SELECT id, title, description, status, priority, created_at FROM tasks
                       WHERE (title LIKE ?1 OR description LIKE ?1) AND status = 'todo'
                       ORDER BY priority ASC, created_at DESC LIMIT 20",
            "active" => "SELECT id, title, description, status, priority, created_at FROM tasks
                         WHERE (title LIKE ?1 OR description LIKE ?1) AND status = 'active'
                         ORDER BY priority ASC, created_at DESC LIMIT 20",
            "done" => "SELECT id, title, description, status, priority, created_at FROM tasks
                       WHERE (title LIKE ?1 OR description LIKE ?1) AND status = 'done'
                       ORDER BY completed_at DESC LIMIT 20",
            _ => "SELECT id, title, description, status, priority, created_at FROM tasks
                  WHERE title LIKE ?1 OR description LIKE ?1
                  ORDER BY created_at DESC LIMIT 20",
        };

        let mut stmt = conn.prepare(query)?;
        let tasks: Vec<(i64, String, Option<String>, String, i32, String)> = stmt
            .query_map([&search_pattern], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let task_count = tasks.len();

        if task_count == 0 {
            return Ok(ChatResponse {
                content: format!("没有找到包含「{}」的任务", keyword),
                response_type: "text".to_string(),
                data: None,
            });
        }

        let mut content = format!("## 🔍 搜索结果\n\n");
        content.push_str(&format!("找到 {} 个包含「{}」的任务：\n\n", task_count, keyword));

        for (i, (id, title, _desc, status, priority, created_at)) in tasks.iter().enumerate() {
            let priority_emoji = match *priority {
                1 => "🔴",
                2 => "🟡",
                _ => "🟢",
            };
            let status_emoji = match status.as_str() {
                "done" => "✅",
                "active" => "🔄",
                _ => "⬜",
            };
            content.push_str(&format!("{}. {} {} {} (创建于: {})\n",
                i + 1, status_emoji, priority_emoji, title, created_at));
        }

        let tasks_for_frontend: Vec<serde_json::Value> = tasks.iter().map(|(id, title, desc, status, priority, created_at)| {
            json!({
                "id": id,
                "title": title.clone(),
                "description": desc.clone(),
                "status": status.clone(),
                "priority": *priority,
                "created_at": created_at.clone(),
            })
        }).collect();

        Ok(ChatResponse {
            content,
            response_type: "tasks".to_string(),
            data: Some(json!({
                "tasks": tasks_for_frontend,
                "total": task_count,
                "keyword": keyword,
            })),
        })
    }

    /// 处理 delete_task - 删除任务
    fn handle_delete_task(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        let task_id = args["task_id"]
            .as_i64()
            .ok_or_else(|| anyhow!("缺少 task_id 参数"))?;

        // 先获取任务信息
        let task_title: Option<String> = conn.query_row(
            "SELECT title FROM tasks WHERE id = ?1",
            [task_id],
            |row| row.get(0)
        ).ok();

        match task_title {
            Some(title) => {
                conn.execute("DELETE FROM tasks WHERE id = ?1", [task_id])?;

                Ok(ChatResponse {
                    content: format!("✅ 已删除任务「{}」(ID: {})", title, task_id),
                    response_type: "text".to_string(),
                    data: Some(json!({
                        "deleted_id": task_id,
                        "deleted_title": title,
                    })),
                })
            }
            None => {
                Ok(ChatResponse {
                    content: format!("❌ 没有找到 ID 为 {} 的任务", task_id),
                    response_type: "text".to_string(),
                    data: None,
                })
            }
        }
    }

    /// 处理 complete_task - 完成任务
    fn handle_complete_task(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        let task_id = args["task_id"].as_i64();
        let title_keyword = args["title"].as_str();

        // 查找任务
        let task: Option<(i64, String)> = if let Some(id) = task_id {
            conn.query_row(
                "SELECT id, title FROM tasks WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?))
            ).ok()
        } else if let Some(keyword) = title_keyword {
            let search_pattern = format!("%{}%", keyword);
            conn.query_row(
                "SELECT id, title FROM tasks WHERE title LIKE ?1 AND status != 'done' ORDER BY created_at DESC LIMIT 1",
                [&search_pattern],
                |row| Ok((row.get(0)?, row.get(1)?))
            ).ok()
        } else {
            return Err(anyhow!("需要提供 task_id 或 title 参数"));
        };

        match task {
            Some((id, title)) => {
                let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                conn.execute(
                    "UPDATE tasks SET status = 'done', completed_at = ?1, updated_at = ?1 WHERE id = ?2",
                    rusqlite::params![&now, id]
                )?;

                // 获取今日完成数量
                let today_done: i32 = conn.query_row(
                    "SELECT COUNT(*) FROM tasks WHERE status = 'done' AND date(completed_at) = date('now', 'localtime')",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0);

                let mut content = format!("## ✅ 任务已完成！\n\n");
                content.push_str(&format!("**任务**：{}\n", title));
                content.push_str(&format!("**完成时间**：{}\n\n", now));
                content.push_str(&format!("🎉 今天已完成 {} 个任务，继续加油！", today_done));

                Ok(ChatResponse {
                    content,
                    response_type: "text".to_string(),
                    data: Some(json!({
                        "task_id": id,
                        "title": title,
                        "completed_at": now,
                        "today_done_count": today_done,
                    })),
                })
            }
            None => {
                let hint = if let Some(keyword) = title_keyword {
                    format!("没有找到标题包含「{}」的未完成任务", keyword)
                } else {
                    format!("没有找到 ID 为 {} 的任务", task_id.unwrap_or(0))
                };

                Ok(ChatResponse {
                    content: format!("❌ {}", hint),
                    response_type: "text".to_string(),
                    data: None,
                })
            }
        }
    }

    /// 处理 search_sql
    fn handle_search_sql(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        let keyword = args["keyword"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 keyword 参数"))?;

        // 搜索 SQL 历史（简化实现，实际应调用 SqlService）
        let mut stmt = conn.prepare(
            "SELECT id, content, name, created_at FROM sql_history
             WHERE content LIKE ?1 OR name LIKE ?1
             ORDER BY created_at DESC LIMIT 10"
        )?;

        let search_pattern = format!("%{}%", keyword);
        let sqls: Vec<serde_json::Value> = stmt
            .query_map([&search_pattern], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "content": row.get::<_, String>(1)?,
                    "name": row.get::<_, Option<String>>(2)?,
                    "created_at": row.get::<_, String>(3)?,
                }))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let content = if sqls.is_empty() {
            format!("没有找到包含 '{}' 的 SQL 记录。\n\n💡 你可以尝试：\n- 使用更简短的关键词\n- 检查拼写是否正确", keyword)
        } else {
            let mut description = format!("🔍 找到 {} 条包含 '{}' 的 SQL 记录：\n\n", sqls.len(), keyword);

            for (i, sql) in sqls.iter().take(3).enumerate() {
                let name = sql["name"].as_str().unwrap_or("未命名");
                let content_preview = sql["content"].as_str().unwrap_or("")
                    .chars().take(80).collect::<String>();
                let created_at = sql["created_at"].as_str().unwrap_or("");

                description.push_str(&format!(
                    "{}. **{}**\n   ```sql\n   {}...\n   ```\n   📅 {}\n\n",
                    i + 1, name, content_preview, created_at
                ));
            }

            if sqls.len() > 3 {
                description.push_str(&format!("...还有 {} 条记录\n", sqls.len() - 3));
            }

            description.push_str("💡 点击 SQL 记录可以复制到剪贴板");
            description
        };

        Ok(ChatResponse {
            content,
            response_type: "sql".to_string(),
            data: Some(json!({
                "sql_list": sqls,
                "total": sqls.len(),
                // 提供第一条 SQL 用于简单展示
                "sql": sqls.first().map(|s| s["content"].as_str().unwrap_or("")).unwrap_or(""),
            })),
        })
    }

    /// 处理 get_time_stats
    fn handle_get_time_stats(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        let period = args["period"].as_str().unwrap_or("today");

        let (total_focus_time, completed_tasks, pomodoro_count): (i32, i32, i32) = match period {
            "today" => conn.query_row(
                "SELECT
                    COALESCE(SUM(ps.focus_time), 0) as total_focus,
                    (SELECT COUNT(*) FROM tasks WHERE status = 'done' AND date(completed_at) = date('now', 'localtime')) as completed,
                    COUNT(ps.id) as pomodoro_count
                 FROM pomodoro_sessions ps
                 WHERE date(ps.created_at) = date('now', 'localtime')",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?,
            "week" => conn.query_row(
                "SELECT
                    COALESCE(SUM(ps.focus_time), 0) as total_focus,
                    (SELECT COUNT(*) FROM tasks WHERE status = 'done'
                     AND date(completed_at) >= date('now', 'localtime', 'weekday 0', '-6 days')) as completed,
                    COUNT(ps.id) as pomodoro_count
                 FROM pomodoro_sessions ps
                 WHERE date(ps.created_at) >= date('now', 'localtime', 'weekday 0', '-6 days')",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?,
            "month" => conn.query_row(
                "SELECT
                    COALESCE(SUM(ps.focus_time), 0) as total_focus,
                    (SELECT COUNT(*) FROM tasks WHERE status = 'done'
                     AND date(completed_at) >= date('now', 'localtime', 'start of month')) as completed,
                    COUNT(ps.id) as pomodoro_count
                 FROM pomodoro_sessions ps
                 WHERE date(ps.created_at) >= date('now', 'localtime', 'start of month')",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?,
            _ => return Err(anyhow!("无效的 period 参数")),
        };

        let period_name = match period {
            "today" => "今日",
            "week" => "本周",
            "month" => "本月",
            _ => "",
        };

        let hours = total_focus_time / 60;
        let minutes = total_focus_time % 60;
        let time_str = if hours > 0 {
            format!("{} 小时 {} 分钟", hours, minutes)
        } else {
            format!("{} 分钟", minutes)
        };

        // 构建更丰富的内容
        let mut content = format!("## ⏰ {}时间统计\n\n", period_name);

        content.push_str(&format!("📊 **专注时长**：{}\n", time_str));
        content.push_str(&format!("✅ **完成任务**：{} 个\n", completed_tasks));
        content.push_str(&format!("🍅 **番茄钟**：{} 个\n\n", pomodoro_count));

        // 添加效率评价
        if total_focus_time > 0 {
            let avg_per_task = if completed_tasks > 0 {
                total_focus_time / completed_tasks
            } else {
                0
            };

            content.push_str("### 📈 效率分析\n\n");

            if completed_tasks > 0 {
                content.push_str(&format!("- 平均每个任务用时：{} 分钟\n", avg_per_task));
            }

            // 根据专注时长给出评价
            let evaluation = match period {
                "today" => {
                    if total_focus_time >= 180 {
                        "🌟 今天专注度很高，继续保持！"
                    } else if total_focus_time >= 60 {
                        "👍 还不错，可以尝试再多专注一会儿"
                    } else {
                        "💪 今天刚开始，加油！"
                    }
                },
                "week" => {
                    if total_focus_time >= 600 {
                        "🌟 本周工作效率很高！"
                    } else if total_focus_time >= 300 {
                        "👍 本周表现不错，继续努力"
                    } else {
                        "💪 可以尝试使用番茄钟提升专注度"
                    }
                },
                "month" => {
                    if total_focus_time >= 2400 {
                        "🌟 本月工作非常出色！"
                    } else if total_focus_time >= 1200 {
                        "👍 本月保持了良好的工作节奏"
                    } else {
                        "💪 建议制定更详细的工作计划"
                    }
                },
                _ => "",
            };

            content.push_str(&format!("\n{}", evaluation));
        } else {
            content.push_str("💡 **提示**：还没有专注记录，试试开始一个番茄钟吧！");
        }

        let stats_data = json!({
            "period": period,
            "total_focus_time_minutes": total_focus_time,
            "completed_tasks": completed_tasks,
            "pomodoro_count": pomodoro_count,
        });

        Ok(ChatResponse {
            content,
            response_type: "time_stats".to_string(),
            data: Some(stats_data),
        })
    }

    /// 处理 create_task
    fn handle_create_task(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        use crate::services::tag_service::TagService;

        let title = args["title"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 title 参数"))?;
        let description = args["description"].as_str();
        let priority = args["priority"].as_i64().unwrap_or(2) as i32;

        // 解析标签参数
        let tags: Option<Vec<String>> = args["tags"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        });

        let priority_enum = TaskPriority::from_i32(priority);
        let task_id = TaskService::create_task(
            conn,
            title,
            description,
            TaskCategory::Other,
            priority_enum.clone(),
        )?;

        // 处理标签
        let mut added_tags: Vec<String> = Vec::new();
        if let Some(tag_names) = &tags {
            for tag_name in tag_names {
                if tag_name.trim().is_empty() {
                    continue;
                }
                // 查找已存在的标签
                let existing_tags = TagService::get_all_tags(conn)?;
                let tag_id = if let Some(tag) = existing_tags.iter().find(|t| t.name.eq_ignore_ascii_case(tag_name)) {
                    tag.id.unwrap()
                } else {
                    // 创建新标签（使用默认颜色）
                    TagService::create_tag(conn, tag_name.trim(), "#6366f1")?
                };
                // 关联标签到任务
                if TagService::add_tag_to_task(conn, task_id, tag_id).is_ok() {
                    added_tags.push(tag_name.clone());
                }
            }
        }

        // 获取当前待办任务总数
        let todo_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE status IN ('todo', 'active')",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let priority_text = match priority_enum {
            TaskPriority::High => ("🔴 高优先级", "建议尽快处理这个任务！"),
            TaskPriority::Medium => ("🟡 中优先级", "可以安排在合适的时间处理"),
            TaskPriority::Low => ("🟢 低优先级", "有空的时候再处理即可"),
        };

        let mut content = format!("## ✅ 任务创建成功！\n\n");
        content.push_str(&format!("📋 **任务标题**：{}\n", title));

        if let Some(desc) = description {
            content.push_str(&format!("📝 **任务描述**：{}\n", desc));
        }

        content.push_str(&format!("🏷️ **优先级**：{}\n", priority_text.0));

        // 显示添加的标签
        if !added_tags.is_empty() {
            content.push_str(&format!("🏷️ **标签**：{}\n", added_tags.join("、")));
        }

        content.push_str(&format!("🆔 **任务 ID**：#{}\n\n", task_id));
        content.push_str(&format!("💡 {}\n\n", priority_text.1));
        content.push_str(&format!("📊 当前共有 {} 个待办任务", todo_count));

        Ok(ChatResponse {
            content,
            response_type: "task_created".to_string(),
            data: Some(json!({
                "task_id": task_id,
                "title": title,
                "priority": priority,
                "tags": added_tags,
                "todo_count": todo_count
            })),
        })
    }

    /// 处理 update_task
    fn handle_update_task(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        let task_id = args["task_id"]
            .as_i64()
            .ok_or_else(|| anyhow!("缺少 task_id 参数"))?;
        let new_title = args["title"].as_str();
        let new_description = args["description"].as_str();
        let new_priority = args["priority"].as_i64().map(|p| TaskPriority::from_i32(p as i32));

        // 先获取原任务信息
        let old_task: (String, String, i32) = conn.query_row(
            "SELECT title, COALESCE(description, ''), priority FROM tasks WHERE id = ?1",
            [task_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        TaskService::update_task(
            conn,
            task_id,
            new_title,
            new_description,
            None,
            new_priority.clone(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )?;

        let mut content = format!("## ✏️ 任务更新成功！\n\n");
        content.push_str(&format!("🆔 **任务 ID**：#{}\n\n", task_id));
        content.push_str("### 更新内容：\n\n");

        let mut has_changes = false;

        if let Some(title) = new_title {
            if title != old_task.0 {
                content.push_str(&format!("📋 **标题**：{} → {}\n", old_task.0, title));
                has_changes = true;
            }
        }

        if let Some(desc) = new_description {
            if desc != old_task.1 {
                let old_desc = if old_task.1.is_empty() { "(无)" } else { &old_task.1 };
                content.push_str(&format!("📝 **描述**：{} → {}\n", old_desc, desc));
                has_changes = true;
            }
        }

        if let Some(ref priority) = new_priority {
            let old_priority = TaskPriority::from_i32(old_task.2);
            if *priority != old_priority {
                let priority_text = |p: &TaskPriority| match p {
                    TaskPriority::High => "🔴 高",
                    TaskPriority::Medium => "🟡 中",
                    TaskPriority::Low => "🟢 低",
                };
                content.push_str(&format!(
                    "🏷️ **优先级**：{} → {}\n",
                    priority_text(&old_priority),
                    priority_text(priority)
                ));
                has_changes = true;
            }
        }

        if !has_changes {
            content.push_str("(没有检测到实际更改)\n");
        }

        content.push_str("\n💡 任务已保存，继续加油！");

        Ok(ChatResponse {
            content,
            response_type: "task_updated".to_string(),
            data: Some(json!({
                "task_id": task_id,
                "updated_fields": {
                    "title": new_title,
                    "description": new_description,
                    "priority": new_priority.map(|p| p as i32)
                }
            })),
        })
    }

    /// 处理 start_pomodoro
    fn handle_start_pomodoro(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        use crate::models::pomodoro::CreatePomodoroRequest;

        let task_id = args["task_id"]
            .as_i64()
            .ok_or_else(|| anyhow!("缺少 task_id 参数"))?;
        let duration = args["duration"].as_i64().unwrap_or(25) as i32;

        // 获取关联的任务信息
        let task_title: String = conn.query_row(
            "SELECT title FROM tasks WHERE id = ?1",
            [task_id],
            |row| row.get(0),
        ).unwrap_or_else(|_| "未知任务".to_string());

        let request = CreatePomodoroRequest {
            task_id: Some(task_id),
            duration_minutes: Some(duration),
            focus_goal: None,
            focus_apps: None,
            ai_suggestion: None,
        };

        let session = PomodoroService::create_session(conn, request)?;

        // 获取今日已完成的番茄钟数量
        let today_pomodoro_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM pomodoro_sessions WHERE date(created_at) = date('now', 'localtime')",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let session_id = session.id.unwrap_or(0);

        let mut content = format!("## 🍅 番茄钟已启动！\n\n");
        content.push_str(&format!("⏱️ **专注时长**：{} 分钟\n", duration));
        content.push_str(&format!("📋 **关联任务**：{}\n", task_title));
        content.push_str(&format!("🆔 **会话 ID**：#{}\n\n", session_id));

        content.push_str("### 💡 专注小贴士：\n\n");
        content.push_str("- 🔕 关闭手机通知，保持专注\n");
        content.push_str("- 🎯 一次只专注一件事\n");
        content.push_str("- ☕ 完成后别忘了休息 5 分钟\n\n");

        content.push_str(&format!("📊 今日已完成 {} 个番茄钟", today_pomodoro_count));

        if today_pomodoro_count >= 8 {
            content.push_str("\n\n🌟 今天已经很努力了，注意劳逸结合！");
        } else if today_pomodoro_count >= 4 {
            content.push_str("\n\n👍 状态不错，继续保持！");
        }

        Ok(ChatResponse {
            content,
            response_type: "pomodoro_started".to_string(),
            data: Some(json!({
                "session_id": session_id,
                "duration": duration,
                "task_id": task_id,
                "task_title": task_title,
                "today_count": today_pomodoro_count
            })),
        })
    }

    /// 处理 analyze_efficiency
    fn handle_analyze_efficiency(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        let days = args["days"].as_i64().unwrap_or(7);

        // 获取统计数据
        let (total_focus_time, completed_tasks, pomodoro_count): (i32, i32, i32) = conn.query_row(
            "SELECT
                COALESCE(SUM(ps.focus_time), 0) as total_focus,
                (SELECT COUNT(*) FROM tasks
                 WHERE status = 'done'
                 AND date(completed_at) >= date('now', 'localtime', ?1 || ' days')) as completed,
                COUNT(ps.id) as pomodoro_count
             FROM pomodoro_sessions ps
             WHERE date(ps.created_at) >= date('now', 'localtime', ?1 || ' days')",
            rusqlite::params![format!("-{}", days), days],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        // 获取待办任务数
        let pending_tasks: i32 = conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE status IN ('todo', 'active')",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        // 获取高优先级待办
        let high_priority_pending: i32 = conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE status IN ('todo', 'active') AND priority = 1",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let avg_daily_tasks = if days > 0 {
            completed_tasks as f64 / days as f64
        } else {
            0.0
        };

        let avg_daily_focus = if days > 0 {
            total_focus_time / days as i32
        } else {
            0
        };

        let hours = total_focus_time / 60;
        let minutes = total_focus_time % 60;
        let time_str = if hours > 0 {
            format!("{} 小时 {} 分钟", hours, minutes)
        } else {
            format!("{} 分钟", minutes)
        };

        let mut content = format!("## 📊 过去 {} 天效率分析\n\n", days);

        // 核心数据
        content.push_str("### 📈 核心指标\n\n");
        content.push_str(&format!("| 指标 | 数值 |\n"));
        content.push_str(&format!("|------|------|\n"));
        content.push_str(&format!("| ✅ 完成任务 | {} 个 |\n", completed_tasks));
        content.push_str(&format!("| ⏱️ 专注时长 | {} |\n", time_str));
        content.push_str(&format!("| 🍅 番茄钟 | {} 个 |\n", pomodoro_count));
        content.push_str(&format!("| 📊 日均完成 | {:.1} 个 |\n", avg_daily_tasks));
        content.push_str(&format!("| ⏰ 日均专注 | {} 分钟 |\n\n", avg_daily_focus));

        // 当前工作量
        content.push_str("### 📋 当前工作量\n\n");
        content.push_str(&format!("- 待办任务：{} 个\n", pending_tasks));
        if high_priority_pending > 0 {
            content.push_str(&format!("- 🔴 高优先级待办：{} 个\n", high_priority_pending));
        }
        content.push_str("\n");

        // 效率评估与建议
        content.push_str("### 💡 效率评估\n\n");

        // 根据数据给出评价
        if completed_tasks == 0 && total_focus_time == 0 {
            content.push_str("这段时间似乎没有什么工作记录。如果你正在休假，好好享受吧！如果是工作状态，可以试试用番茄钟来培养专注习惯。\n");
        } else {
            // 任务完成效率评价
            if avg_daily_tasks >= 3.0 {
                content.push_str("🌟 **任务完成**：非常棒！日均完成超过 3 个任务，效率很高。\n\n");
            } else if avg_daily_tasks >= 1.0 {
                content.push_str("👍 **任务完成**：保持稳定的工作节奏，每天都有产出。\n\n");
            } else {
                content.push_str("💪 **任务完成**：可以尝试将大任务拆分成小任务，更容易看到进展。\n\n");
            }

            // 专注时长评价
            if avg_daily_focus >= 120 {
                content.push_str("🌟 **专注时长**：日均专注超过 2 小时，深度工作做得很好！\n\n");
            } else if avg_daily_focus >= 60 {
                content.push_str("👍 **专注时长**：日均专注约 1 小时，还有提升空间。\n\n");
            } else if total_focus_time > 0 {
                content.push_str("💪 **专注时长**：建议多使用番茄钟，逐步增加专注时间。\n\n");
            }

            // 待办积压提醒
            if pending_tasks > 10 {
                content.push_str(&format!("⚠️ **待办积压**：目前有 {} 个待办任务，建议清理低优先级任务或委派他人。\n\n", pending_tasks));
            }

            if high_priority_pending >= 3 {
                content.push_str(&format!("🔥 **紧急提醒**：有 {} 个高优先级任务待处理，建议优先安排！\n", high_priority_pending));
            }
        }

        let efficiency_data = json!({
            "days": days,
            "total_focus_time_minutes": total_focus_time,
            "completed_tasks": completed_tasks,
            "pomodoro_count": pomodoro_count,
            "avg_daily_tasks": format!("{:.1}", avg_daily_tasks),
            "avg_daily_focus_minutes": avg_daily_focus,
            "pending_tasks": pending_tasks,
            "high_priority_pending": high_priority_pending,
        });

        Ok(ChatResponse {
            content,
            response_type: "efficiency_analysis".to_string(),
            data: Some(efficiency_data),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function_call() {
        let response = r#"{"function": "get_tasks", "arguments": {"status": "todo"}}"#;
        let result = AiChatService::parse_function_call(response);
        assert!(result.is_ok());
        let function_call = result.unwrap();
        assert_eq!(function_call.name, "get_tasks");
    }

    #[test]
    fn test_parse_function_call_with_text() {
        let response = r#"好的，我来帮你获取任务列表：{"function": "get_tasks", "arguments": {"status": "all"}}"#;
        let result = AiChatService::parse_function_call(response);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_non_function_call() {
        let response = "这是一个普通的回复";
        let result = AiChatService::parse_function_call(response);
        assert!(result.is_err());
    }
}
