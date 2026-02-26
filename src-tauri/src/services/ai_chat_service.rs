use anyhow::{anyhow, Result};
use chrono::Timelike;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;
use regex::Regex;
use once_cell::sync::Lazy;
use crate::services::ai_service::{AiService, ChatMessage};
use crate::services::task_service::TaskService;
use crate::services::pomodoro_service::PomodoroService;
use crate::services::reminder_service::ReminderService;
use crate::services::user_behavior_service::UserBehaviorService;
use crate::models::task::{TaskCategory, TaskPriority};

// ============================================================================
// 隐含偏好识别的正则表达式模式（使用 Lazy 静态初始化）
// ============================================================================

/// 时间偏好识别模式
static TIME_PREFERENCE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // 匹配：我一般/习惯/通常/喜欢 + 早上/上午/中午/下午/晚上/深夜
        Regex::new(r"我(一般|习惯|通常|喜欢).{0,15}(早上|上午|中午|下午|晚上|深夜)").unwrap(),
        // 匹配：早上/上午/中午/下午/晚上/深夜 + 效率高/习惯/喜欢
        Regex::new(r"(早上|上午|中午|下午|晚上|深夜).{0,10}(效率高|效率最高|习惯|喜欢|适合)").unwrap(),
    ]
});

/// 工作模式识别模式
static WORK_METHOD_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // 匹配：番茄钟/专注 + 数字 + 分钟/小时
        Regex::new(r"(番茄钟|专注|工作).{0,5}(\d+).{0,3}(分钟|小时|min)").unwrap(),
        // 匹配：我习惯/喜欢 + 番茄钟/分块/集中
        Regex::new(r"我(习惯|喜欢|用|使用).{0,10}(番茄钟|分块|集中|专注)").unwrap(),
        // 匹配：一个番茄钟 + 数字
        Regex::new(r"一个?番茄钟.{0,5}(\d+)").unwrap(),
    ]
});

/// 优先级表达识别模式
static PRIORITY_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // 匹配：很重要/非常重要/特别重要/必须
        Regex::new(r"(很|非常|特别|超级)重要|必须|紧急|急").unwrap(),
        // 匹配：不着急/不急/慢慢来/有空再
        Regex::new(r"不着急|不急|慢慢来|有空再|随时|随便").unwrap(),
        // 匹配：今天必须/今天一定要/必须今天
        Regex::new(r"(今天|今日).{0,5}(必须|一定要|得)|必须.{0,5}(今天|今日)").unwrap(),
    ]
});

/// 项目关联识别模式
static PROJECT_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // 匹配：这是XXX项目的
        Regex::new(r"这是(.{1,20})项目").unwrap(),
        // 匹配：XXX项目相关
        Regex::new(r"(.{1,20})项目.{0,5}(相关|有关|的)").unwrap(),
        // 匹配：和上次那个任务/和之前的
        Regex::new(r"和(上次|之前|刚才).{0,10}(任务|项目)").unwrap(),
    ]
});

/// 安全截断字符串，确保不会切到 UTF-8 字符中间
pub fn safe_truncate(s: &str, max_chars: usize) -> String {
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

/// 操作上下文 - 记录最近操作的实体，用于代词解析
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationContext {
    /// 最近操作的任务
    pub last_task: Option<TaskContext>,
    /// 最近操作的番茄钟
    pub last_pomodoro: Option<PomodoroContext>,
    /// 最近查询涉及的任务列表（用于"第一个"、"最后一个"等引用）
    pub recent_tasks: Option<Vec<TaskContext>>,
}

/// 任务上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskContext {
    pub id: i64,
    pub title: String,
}

/// 番茄钟上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroContext {
    pub id: i64,
    pub task_id: Option<i64>,
    pub task_title: Option<String>,
}

/// AI 聊天响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub content: String,
    pub response_type: String,
    pub data: Option<serde_json::Value>,
    /// 操作上下文，前端需保存并在下次请求时传回
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<OperationContext>,
}

/// Function Call 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// 单步操作 - 多步操作中的每一步
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOperation {
    /// 要调用的函数名
    pub function: String,
    /// 函数参数，可能包含 $previous.xxx 变量引用
    pub arguments: serde_json::Value,
}

/// 多步 Function Call 结构 - 支持一句话触发多个操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiStepFunctionCall {
    /// 操作步骤列表
    pub steps: Vec<StepOperation>,
}

/// 单步执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// 步骤索引（从0开始）
    pub step_index: usize,
    /// 函数名
    pub function: String,
    /// 是否成功
    pub success: bool,
    /// 执行结果数据（用于变量引用）
    pub data: Option<serde_json::Value>,
    /// 响应内容
    pub content: String,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

/// 多步执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiStepResult {
    /// 所有步骤的执行结果
    pub steps: Vec<StepResult>,
    /// 是否全部成功
    pub all_success: bool,
    /// 成功执行的步骤数
    pub success_count: usize,
    /// 总步骤数
    pub total_count: usize,
    /// 汇总内容（用于展示给用户）
    pub summary: String,
}

/// 解析后的 Function Call 类型
#[derive(Debug, Clone)]
pub enum ParsedFunctionCall {
    /// 单步操作
    Single(FunctionCall),
    /// 多步操作
    Multi(MultiStepFunctionCall),
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

    /// 构建系统提示词（包含用户数据上下文和操作上下文）
    pub fn build_system_prompt(conn: &Connection, op_context: Option<&OperationContext>) -> Result<String> {
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

        // 构建记忆上下文提示
        let memory_context_hint = Self::build_memory_context_hint(conn);

        // 构建操作上下文提示
        let context_hint = Self::build_context_hint(op_context);

        // 构建智能提醒提示（任务截止日期、高优先级任务等）
        let reminder_hint = Self::build_reminder_hint(conn);

        // 构建用户行为偏好提示
        let user_preference_hint = Self::build_user_preference_hint(conn);

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

  ★★★ 特别重要 - 任务列表查询 ★★★
  - "今天有什么任务" / "我有哪些任务" / "待办任务" / "任务列表"
    → 查询所有未完成的任务：SELECT id, title, priority, status, due_date FROM tasks WHERE status IN ('todo', 'active') ORDER BY priority ASC, created_at DESC
    → 不要限制创建日期或截止日期！用户想看的是所有待办任务，不是某一天创建或截止的
  - "高优先级任务" → SELECT * FROM tasks WHERE status IN ('todo', 'active') AND priority=1
  - "所有任务" → SELECT * FROM tasks ORDER BY status, priority

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

【多步操作 - 一句话触发多个连续操作】
当用户需要连续执行多个操作时，使用 steps 数组格式：

格式：
{{"steps": [
  {{"function": "操作1", "arguments": {{...}}}},
  {{"function": "操作2", "arguments": {{...}}}}
]}}

变量引用 - 后续步骤可以引用前面步骤的结果：
- $previous.task_id - 引用上一步创建/操作的任务ID
- $previous.pomodoro_id - 引用上一步创建的番茄钟ID
- $step[0].task_id - 引用第1步的结果

示例1 - 创建任务并开始番茄钟：
用户："创建一个高优先级任务'完成报告'，然后开始25分钟番茄钟"
{{"steps": [
  {{"function": "create_task", "arguments": {{"title": "完成报告", "priority": 1}}}},
  {{"function": "start_pomodoro", "arguments": {{"task_id": "$previous.task_id", "duration": 25}}}}
]}}

示例2 - 完成多个任务：
用户："把'整理文档'和'回复邮件'都标记为完成"
{{"steps": [
  {{"function": "complete_task", "arguments": {{"title": "整理文档"}}}},
  {{"function": "complete_task", "arguments": {{"title": "回复邮件"}}}}
]}}

判断规则：
- 如果用户请求包含"然后"、"接着"、"并且"、"同时"等连接词，考虑使用多步操作
- 如果用户明确要求多个操作，使用多步格式
- 简单的单一操作仍然使用普通格式

{}{}{}{}
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
            current_date,          // 用于 generate_daily_report 说明中的日期占位符
            memory_context_hint,   // 长期记忆上下文
            context_hint,          // 操作上下文提示
            reminder_hint,         // 智能提醒
            user_preference_hint   // 用户行为偏好
        );

        Ok(system_prompt)
    }

    /// 构建记忆上下文提示（从数据库查询长期记忆）
    fn build_memory_context_hint(conn: &Connection) -> String {
        // 查询高重要性的上下文记忆（importance >= 3）
        let query = "
            SELECT context_type, key, value, importance, last_used_at
            FROM ai_context_memory
            WHERE importance >= 3
            ORDER BY importance DESC, last_used_at DESC
            LIMIT 10
        ";

        let mut stmt = match conn.prepare(query) {
            Ok(s) => s,
            Err(_) => return String::new(),
        };

        let memories: Vec<(String, String, String, i32)> = match stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,  // context_type
                    row.get::<_, String>(1)?,  // key
                    row.get::<_, String>(2)?,  // value
                    row.get::<_, i32>(3)?,     // importance
                ))
            }) {
            Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
            Err(_) => return String::new(),
        };

        if memories.is_empty() {
            return String::new();
        }

        // 按类型分组记忆
        let mut user_preferences = Vec::new();
        let mut project_info = Vec::new();
        let mut recent_facts = Vec::new();

        for (context_type, key, value, _importance) in memories {
            let entry = format!("{}：{}", key, value);
            match context_type.as_str() {
                "user_preference" => user_preferences.push(entry),
                "project_info" => project_info.push(entry),
                "fact" | "important_fact" => recent_facts.push(entry),
                _ => {}
            }
        }

        let mut hints = Vec::new();

        if !user_preferences.is_empty() {
            hints.push("【用户偏好】".to_string());
            for pref in user_preferences.iter().take(3) {
                hints.push(format!("- {}", pref));
            }
        }

        if !project_info.is_empty() {
            hints.push("【项目信息】".to_string());
            for info in project_info.iter().take(3) {
                hints.push(format!("- {}", info));
            }
        }

        if !recent_facts.is_empty() {
            hints.push("【近期重要事项】".to_string());
            for fact in recent_facts.iter().take(4) {
                hints.push(format!("- {}", fact));
            }
        }

        if hints.is_empty() {
            String::new()
        } else {
            format!("\n【长期记忆 - AI 记住的重要信息】\n{}\n", hints.join("\n"))
        }
    }

    /// 构建操作上下文提示（用于代词解析）
    fn build_context_hint(op_context: Option<&OperationContext>) -> String {
        let ctx = match op_context {
            Some(c) => c,
            None => return String::new(),
        };

        let mut hints = Vec::new();

        // 最近操作的任务
        if let Some(task) = &ctx.last_task {
            hints.push(format!(
                "- 刚刚操作的任务：ID={}, 标题=\"{}\"",
                task.id, task.title
            ));
            hints.push(format!(
                "  → 用户说\"它\"、\"这个任务\"、\"刚才那个\"时，指的是这个任务 (ID={})",
                task.id
            ));
        }

        // 最近操作的番茄钟
        if let Some(pomodoro) = &ctx.last_pomodoro {
            let task_info = match (&pomodoro.task_id, &pomodoro.task_title) {
                (Some(id), Some(title)) => format!("关联任务: ID={}, \"{}\"", id, title),
                _ => "无关联任务".to_string(),
            };
            hints.push(format!(
                "- 刚刚操作的番茄钟：ID={}, {}",
                pomodoro.id, task_info
            ));
        }

        // 最近查询的任务列表
        if let Some(tasks) = &ctx.recent_tasks {
            if !tasks.is_empty() {
                hints.push("- 最近查询涉及的任务：".to_string());
                for (i, task) in tasks.iter().take(5).enumerate() {
                    hints.push(format!("  {}. ID={}, \"{}\"", i + 1, task.id, task.title));
                }
                hints.push("  → 用户说\"第一个\"、\"第二个\"时，参考上面的序号".to_string());
            }
        }

        if hints.is_empty() {
            String::new()
        } else {
            format!("\n【对话上下文 - 代词解析参考】\n{}\n", hints.join("\n"))
        }
    }

    /// 构建智能提醒提示（任务截止日期、高优先级任务等）
    ///
    /// 从 ReminderService 获取待办提醒，格式化为系统提示词
    fn build_reminder_hint(conn: &Connection) -> String {
        match ReminderService::get_pending_reminders(conn) {
            Ok(reminders) => {
                if reminders.is_empty() {
                    String::new()
                } else {
                    ReminderService::format_reminders_for_prompt(&reminders)
                }
            }
            Err(e) => {
                log::warn!("[AI Chat] 获取智能提醒失败: {}", e);
                String::new()
            }
        }
    }

    /// 构建用户行为偏好提示
    ///
    /// 从 UserBehaviorService 推断用户偏好和表达习惯，格式化为系统提示词
    fn build_user_preference_hint(conn: &Connection) -> String {
        let mut parts = Vec::new();

        // 1. 获取用户行为偏好
        match UserBehaviorService::infer_preferences(conn) {
            Ok(prefs) => {
                // 检查是否有有效的偏好数据
                let has_data = prefs.preferred_pomodoro_duration.is_some()
                    || prefs.preferred_priority.is_some()
                    || !prefs.active_hours.is_empty()
                    || prefs.avg_task_completion_days.is_some();

                if has_data {
                    let prefs_text = UserBehaviorService::format_preferences_for_prompt(&prefs);
                    parts.push(format!("【用户行为偏好】\n{}", prefs_text));
                }
            }
            Err(e) => {
                log::warn!("[AI Chat] 获取用户行为偏好失败: {}", e);
            }
        }

        // 2. 获取用户表达习惯
        match UserBehaviorService::get_expression_preferences(conn) {
            Ok(expr_prefs) => {
                let expr_text = UserBehaviorService::format_expression_preferences_for_prompt(&expr_prefs);
                if !expr_text.contains("暂无") {
                    parts.push(expr_text);
                }
            }
            Err(e) => {
                log::warn!("[AI Chat] 获取用户表达习惯失败: {}", e);
            }
        }

        // 3. 组合提示词
        if !parts.is_empty() {
            parts.push("\n提示：在理解用户意图和给出建议时，请参考上述用户习惯。例如：\n- 识别用户的优先级表达方式（如「很急」表示高优先级）\n- 使用用户习惯的命名风格创建任务\n- 推荐用户常用的标签\n- 参考用户偏好的时间设置".to_string());
            format!("\n{}\n", parts.join("\n\n"))
        } else {
            String::new()
        }
    }

    /// 构建日报生成的 AI 提示词模板
    ///
    /// 此函数生成一个专业的提示词，引导 AI 基于真实数据分析生成有洞察力的日报。
    ///
    /// # 参数
    /// * `date` - 日报日期 (格式: YYYY-MM-DD)
    /// * `completed_tasks` - 今日完成的任务列表 (标题, 优先级)
    /// * `focus_minutes` - 今日专注时长（分钟）
    /// * `pomodoro_count` - 今日番茄钟数量
    /// * `yesterday_task_count` - 昨日完成任务数量
    /// * `yesterday_focus_minutes` - 昨日专注时长（分钟）
    ///
    /// # 返回
    /// 返回格式化的 AI 提示词字符串
    fn build_daily_report_prompt(
        date: &str,
        completed_tasks: &[(String, i32)],
        focus_minutes: i32,
        pomodoro_count: i32,
        yesterday_task_count: i32,
        yesterday_focus_minutes: i32,
    ) -> String {
        // 格式化任务列表
        let task_list = if completed_tasks.is_empty() {
            "无完成任务".to_string()
        } else {
            completed_tasks
                .iter()
                .enumerate()
                .map(|(i, (title, priority))| {
                    let priority_label = match *priority {
                        1 => "高优先级",
                        2 => "中优先级",
                        _ => "低优先级",
                    };
                    format!("{}. {} ({})", i + 1, title, priority_label)
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        // 计算时长格式
        let focus_hours = focus_minutes / 60;
        let focus_mins = focus_minutes % 60;
        let yesterday_focus_hours = yesterday_focus_minutes / 60;
        let yesterday_focus_mins = yesterday_focus_minutes % 60;

        // 计算变化趋势
        let task_change = (completed_tasks.len() as i32) - yesterday_task_count;
        let focus_change = focus_minutes - yesterday_focus_minutes;

        format!(
            r#"你是 DevAssistant 的 AI 助手小D，现在需要为用户生成一份专业且有温度的工作日报。

【核心要求】
1. 必须基于真实数据进行分析，严禁编造任何信息
2. 如果数据为空或不足，如实说明，不要假设或虚构
3. 语气要友好亲切，像工作伙伴一样给予鼓励和建议
4. 分析要具体，避免空泛的套话

【今日真实数据 - {}】
完成任务: {} 个
专注时长: {} 小时 {} 分钟
番茄钟数: {} 个

【任务详情】
{}

【昨日数据对比】
完成任务: {} 个 (今日变化: {})
专注时长: {} 小时 {} 分钟 (今日变化: {} 分钟)

【生成要求】
请生成一份包含以下三个部分的日报，使用 Markdown 格式：

## 📊 今日工作总结
- 基于完成的任务列表，总结今天的工作重点和成果
- 如果有高优先级任务完成，重点提及
- 如果没有完成任务，简单说明今天可能是休息或规划日

## 📈 效率分析
- 对比昨日数据，分析今天的工作效率变化
- 如果效率提升，给予肯定和鼓励
- 如果效率下降，以关心的语气提醒，但不要责备
- 结合番茄钟数量和专注时长，分析时间管理情况
- 如果数据异常（如专注时长很少但任务很多），可以善意提醒记录准确性

## 💡 明日建议
- 基于今天的数据和趋势，给出1-2条具体的改进建议
- 建议要实用、可执行，不要泛泛而谈
- 如果今天表现很好，建议可以是保持或优化方向
- 如果今天表现一般，给出温和的改进方向

【写作风格】
- 用"你"来称呼用户，拉近距离
- 使用适当的emoji让内容更生动（但不要过度）
- 数据分析要准确，但表达要人性化
- 避免机械的模板式语言，让每句话都有价值

【重要提醒】
- 不要编造任何不存在的任务或数据
- 不要假设用户的工作内容或心理状态
- 如果数据不足以得出结论，坦诚说明即可
- 保持鼓励和支持的基调，但要真诚不虚假

请现在开始生成日报内容。"#,
            date,
            completed_tasks.len(),
            focus_hours,
            focus_mins,
            pomodoro_count,
            task_list,
            yesterday_task_count,
            if task_change > 0 {
                format!("+{}", task_change)
            } else if task_change < 0 {
                task_change.to_string()
            } else {
                "持平".to_string()
            },
            yesterday_focus_hours,
            yesterday_focus_mins,
            focus_change
        )
    }

    /// 构建周报生成的 AI 提示词
    ///
    /// 包含本周数据和上周对比，让 AI 生成有洞察力的周报
    fn build_weekly_report_prompt(
        completed_count: i32,
        focus_minutes: i32,
        pomodoro_count: i32,
        pending_count: i32,
        completed_tasks: &[(String, i32)],
        last_week_completed: i32,
        last_week_focus_minutes: i32,
        last_week_pomodoro: i32,
    ) -> String {
        // 格式化任务列表
        let task_list = if completed_tasks.is_empty() {
            "无完成任务".to_string()
        } else {
            completed_tasks
                .iter()
                .enumerate()
                .map(|(i, (title, priority))| {
                    let priority_label = match *priority {
                        1 => "高优先级",
                        2 => "中优先级",
                        _ => "低优先级",
                    };
                    format!("{}. {} ({})", i + 1, title, priority_label)
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        // 计算时长格式
        let focus_hours = focus_minutes / 60;
        let focus_mins = focus_minutes % 60;
        let last_week_focus_hours = last_week_focus_minutes / 60;
        let last_week_focus_mins = last_week_focus_minutes % 60;

        // 计算变化趋势
        let task_change = completed_count - last_week_completed;
        let focus_change = focus_minutes - last_week_focus_minutes;
        let pomodoro_change = pomodoro_count - last_week_pomodoro;

        // 计算日均数据
        let avg_daily_tasks = completed_count as f32 / 7.0;
        let avg_daily_focus = focus_minutes / 7;

        format!(
            r#"你是 DevAssistant 的 AI 助手小D，现在需要为用户生成一份专业且有温度的工作周报。

【核心要求】
1. 必须基于真实数据进行分析，严禁编造任何信息
2. 如果数据为空或不足，如实说明，不要假设或虚构
3. 语气要友好亲切，像工作伙伴一样给予鼓励和建议
4. 分析要具体，避免空泛的套话

【本周真实数据】
完成任务: {} 个
专注时长: {} 小时 {} 分钟
番茄钟数: {} 个
待办积压: {} 个

日均完成: {:.1} 个任务
日均专注: {} 分钟

【主要完成任务】
{}

【上周数据对比】
完成任务: {} 个 (本周变化: {})
专注时长: {} 小时 {} 分钟 (本周变化: {} 分钟)
番茄钟数: {} 个 (本周变化: {})

【生成要求】
请生成一份包含以下四个部分的周报，使用 Markdown 格式：

## 📊 本周工作总结
- 概述本周的整体工作情况和主要成果
- 如果有高优先级任务完成，重点提及
- 总结工作的重心和方向

## 📈 效率分析
- 对比上周数据，分析本周的工作效率变化
- 分析日均完成量和专注时长的健康程度
- 如果效率提升，给予肯定和鼓励
- 如果效率下降，以关心的语气分析可能原因
- 对比番茄钟使用情况，评估时间管理水平

## 🎯 待办管理
- 如果待办积压较多（>10），提醒用户关注
- 给出任务清理或优先级调整的建议
- 如果待办量健康，肯定用户的任务管理

## 💡 下周建议
- 基于本周的数据和趋势，给出2-3条具体的建议
- 建议要实用、可执行，涵盖效率提升、任务管理等方面
- 如果本周表现很好，建议如何保持或更上一层楼
- 如果本周表现一般，给出温和的改进方向

【写作风格】
- 用"你"来称呼用户，拉近距离
- 使用适当的emoji让内容更生动（但不要过度）
- 数据分析要准确，但表达要人性化
- 避免机械的模板式语言，让每句话都有价值

【重要提醒】
- 不要编造任何不存在的任务或数据
- 不要假设用户的工作内容或心理状态
- 如果数据不足以得出结论，坦诚说明即可
- 保持鼓励和支持的基调，但要真诚不虚假

请现在开始生成周报内容。"#,
            completed_count,
            focus_hours,
            focus_mins,
            pomodoro_count,
            pending_count,
            avg_daily_tasks,
            avg_daily_focus,
            task_list,
            last_week_completed,
            if task_change > 0 {
                format!("+{}", task_change)
            } else if task_change < 0 {
                task_change.to_string()
            } else {
                "持平".to_string()
            },
            last_week_focus_hours,
            last_week_focus_mins,
            focus_change,
            last_week_pomodoro,
            if pomodoro_change > 0 {
                format!("+{}", pomodoro_change)
            } else if pomodoro_change < 0 {
                pomodoro_change.to_string()
            } else {
                "持平".to_string()
            }
        )
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

    /// 使用 AI 生成报表内容
    ///
    /// # Arguments
    /// * `ai_service` - AI 服务实例
    /// * `report_type` - 报表类型 ("daily" 或 "weekly")
    /// * `prompt` - 构建好的提示词
    ///
    /// # Returns
    /// AI 生成的报表内容字符串
    ///
    /// # Example
    /// ```rust
    /// let report = AiChatService::generate_report_with_ai(
    ///     &ai_service,
    ///     "daily",
    ///     "请根据以下数据生成日报..."
    /// ).await?;
    /// ```
    pub async fn generate_report_with_ai(
        ai_service: &AiService,
        report_type: &str,
        prompt: &str,
    ) -> Result<String> {
        // 构建 system 消息，指导 AI 生成专业的报表
        let system_message = match report_type {
            "daily" => {
                "你是一个专业的工作日报生成助手。请根据用户提供的数据，生成一份结构清晰、内容详实的工作日报。\
                 日报应包含：\
                 1. 当日完成的任务列表（按优先级排序）\
                 2. 专注时长和番茄钟统计\
                 3. 工作总结和亮点\
                 4. 遇到的问题（如果有）\
                 5. 明日计划建议\
                 \
                 请使用 Markdown 格式，语言简洁专业，突出关键信息。"
            }
            "weekly" => {
                "你是一个专业的周报生成助手。请根据用户提供的数据，生成一份全面、结构化的周报。\
                 周报应包含：\
                 1. 本周工作概览（完成任务数、专注时长等关键指标）\
                 2. 重点任务回顾（按优先级列出主要完成的任务）\
                 3. 工作亮点和成果\
                 4. 时间管理分析（专注效率、番茄钟使用情况）\
                 5. 问题与改进建议\
                 6. 下周工作计划\
                 \
                 请使用 Markdown 格式，内容要有深度，体现工作价值。"
            }
            _ => {
                "你是一个专业的工作报表生成助手。请根据用户提供的数据，生成一份结构清晰、内容详实的工作报表。\
                 请使用 Markdown 格式，确保内容专业、准确。"
            }
        };

        // 构建消息列表
        let messages = vec![
            ChatMessage::system(system_message.to_string()),
            ChatMessage::user(prompt.to_string()),
        ];

        // 调用 AI API 生成报表
        let response = ai_service.chat(messages).await
            .map_err(|e| anyhow!("AI 生成{}报表失败: {}",
                match report_type {
                    "daily" => "日",
                    "weekly" => "周",
                    _ => ""
                },
                e
            ))?;

        // 返回生成的报表内容（去除首尾空白）
        Ok(response.trim().to_string())
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

    /// 解析 Function Call（支持单步和多步）
    ///
    /// 多步格式示例:
    /// ```json
    /// {
    ///   "steps": [
    ///     {"function": "create_task", "arguments": {"title": "完成报告", "priority": 1}},
    ///     {"function": "start_pomodoro", "arguments": {"task_id": "$previous.task_id", "duration": 25}}
    ///   ]
    /// }
    /// ```
    pub fn parse_function_call_extended(response: &str) -> Result<ParsedFunctionCall> {
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

        // 检查是否是多步操作格式
        if let Some(steps) = parsed.get("steps").and_then(|s| s.as_array()) {
            log::info!("[AI Chat] 🔗 检测到多步操作，共 {} 步", steps.len());

            let mut step_operations = Vec::new();
            for (i, step) in steps.iter().enumerate() {
                let function = step.get("function")
                    .and_then(|f| f.as_str())
                    .ok_or_else(|| anyhow!("步骤 {} 缺少 function 字段", i + 1))?
                    .to_string();

                let arguments = step.get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                step_operations.push(StepOperation {
                    function,
                    arguments,
                });
            }

            Ok(ParsedFunctionCall::Multi(MultiStepFunctionCall {
                steps: step_operations,
            }))
        } else {
            // 单步操作格式
            let function_name = parsed["function"]
                .as_str()
                .ok_or_else(|| anyhow!("缺少 function 字段"))?
                .to_string();

            let arguments = serde_json::to_string(&parsed["arguments"])?;

            Ok(ParsedFunctionCall::Single(FunctionCall {
                name: function_name,
                arguments,
            }))
        }
    }

    /// 解析变量引用并替换为实际值
    ///
    /// 支持的变量格式:
    /// - $previous.task_id - 上一步结果中的 task_id
    /// - $previous.data.xxx - 上一步结果 data 中的字段
    /// - $step[0].task_id - 指定步骤的结果
    fn resolve_step_variables(
        arguments: &serde_json::Value,
        previous_results: &[StepResult],
    ) -> serde_json::Value {
        match arguments {
            serde_json::Value::String(s) => {
                // 检查是否是变量引用
                if s.starts_with("$previous.") {
                    let field_path = &s[10..]; // 去掉 "$previous."
                    if let Some(last_result) = previous_results.last() {
                        if let Some(data) = &last_result.data {
                            return Self::get_nested_value(data, field_path);
                        }
                    }
                    log::warn!("[AI Chat] ⚠️ 无法解析变量: {}", s);
                    serde_json::Value::Null
                } else if s.starts_with("$step[") {
                    // 解析 $step[0].field 格式
                    if let Some(bracket_end) = s.find(']') {
                        if let Ok(step_index) = s[6..bracket_end].parse::<usize>() {
                            let field_path = if s.len() > bracket_end + 2 {
                                &s[bracket_end + 2..] // 去掉 "]."
                            } else {
                                ""
                            };
                            if let Some(step_result) = previous_results.get(step_index) {
                                if let Some(data) = &step_result.data {
                                    return Self::get_nested_value(data, field_path);
                                }
                            }
                        }
                    }
                    log::warn!("[AI Chat] ⚠️ 无法解析变量: {}", s);
                    serde_json::Value::Null
                } else {
                    serde_json::Value::String(s.clone())
                }
            }
            serde_json::Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (key, value) in obj {
                    new_obj.insert(key.clone(), Self::resolve_step_variables(value, previous_results));
                }
                serde_json::Value::Object(new_obj)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(
                    arr.iter()
                        .map(|v| Self::resolve_step_variables(v, previous_results))
                        .collect()
                )
            }
            other => other.clone(),
        }
    }

    /// 从嵌套的 JSON 值中获取指定路径的值
    fn get_nested_value(data: &serde_json::Value, path: &str) -> serde_json::Value {
        if path.is_empty() {
            return data.clone();
        }

        let parts: Vec<&str> = path.split('.').collect();
        let mut current = data;

        for part in parts {
            match current {
                serde_json::Value::Object(obj) => {
                    if let Some(value) = obj.get(part) {
                        current = value;
                    } else {
                        return serde_json::Value::Null;
                    }
                }
                serde_json::Value::Array(arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        if let Some(value) = arr.get(index) {
                            current = value;
                        } else {
                            return serde_json::Value::Null;
                        }
                    } else {
                        return serde_json::Value::Null;
                    }
                }
                _ => return serde_json::Value::Null,
            }
        }

        current.clone()
    }

    /// 执行多步操作
    ///
    /// 按顺序执行每个步骤，收集结果，处理变量替换
    /// 如果某步失败，后续步骤继续执行但会标记失败
    /// 注意：多步操作中的上下文通过变量引用（$previous.xxx）传递，不使用 OperationContext
    pub fn execute_multi_step_operations(
        conn: &Connection,
        multi_step: &MultiStepFunctionCall,
    ) -> Result<MultiStepResult> {
        let mut results: Vec<StepResult> = Vec::new();
        let total_count = multi_step.steps.len();

        log::info!("[AI Chat] 🚀 开始执行多步操作，共 {} 步", total_count);

        for (index, step) in multi_step.steps.iter().enumerate() {
            log::info!("[AI Chat] 📍 执行步骤 {}/{}: {}", index + 1, total_count, step.function);

            // 解析变量引用
            let resolved_arguments = Self::resolve_step_variables(&step.arguments, &results);
            let arguments_str = serde_json::to_string(&resolved_arguments)?;

            log::debug!("[AI Chat] 📋 解析后的参数: {}", arguments_str);

            // 构建 FunctionCall
            let function_call = FunctionCall {
                name: step.function.clone(),
                arguments: arguments_str,
            };

            // 执行单步操作（多步操作中的上下文通过变量引用传递，不需要 OperationContext）
            match Self::execute_function_sync(conn, &function_call) {
                Ok(response) => {
                    log::info!("[AI Chat] ✅ 步骤 {} 执行成功", index + 1);

                    // 从响应中提取可引用的数据
                    let step_data = response.data.clone().or_else(|| {
                        // 尝试从 context 中提取数据
                        response.context.as_ref().map(|ctx| {
                            let mut data = serde_json::Map::new();
                            if let Some(task) = &ctx.last_task {
                                data.insert("task_id".to_string(), json!(task.id));
                                data.insert("task_title".to_string(), json!(task.title));
                            }
                            if let Some(pomodoro) = &ctx.last_pomodoro {
                                data.insert("pomodoro_id".to_string(), json!(pomodoro.id));
                            }
                            serde_json::Value::Object(data)
                        })
                    });

                    results.push(StepResult {
                        step_index: index,
                        function: step.function.clone(),
                        success: true,
                        data: step_data,
                        content: response.content,
                        error: None,
                    });
                }
                Err(e) => {
                    log::error!("[AI Chat] ❌ 步骤 {} 执行失败: {}", index + 1, e);

                    results.push(StepResult {
                        step_index: index,
                        function: step.function.clone(),
                        success: false,
                        data: None,
                        content: String::new(),
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        // 统计结果
        let success_count = results.iter().filter(|r| r.success).count();
        let all_success = success_count == total_count;

        // 生成汇总内容
        let summary = Self::generate_multi_step_summary(&results);

        log::info!("[AI Chat] 📊 多步操作完成: {}/{} 成功", success_count, total_count);

        Ok(MultiStepResult {
            steps: results,
            all_success,
            success_count,
            total_count,
            summary,
        })
    }

    /// 生成多步操作的汇总内容
    fn generate_multi_step_summary(results: &[StepResult]) -> String {
        let mut summary = String::new();

        let success_count = results.iter().filter(|r| r.success).count();
        let total = results.len();

        if success_count == total {
            summary.push_str(&format!("✅ 已完成全部 {} 个操作：\n\n", total));
        } else {
            summary.push_str(&format!("⚠️ 完成了 {}/{} 个操作：\n\n", success_count, total));
        }

        for (i, result) in results.iter().enumerate() {
            let status = if result.success { "✓" } else { "✗" };
            let function_display = Self::get_function_display_name(&result.function);

            summary.push_str(&format!("{}. {} {}", i + 1, status, function_display));

            if result.success {
                if !result.content.is_empty() {
                    // 只取内容的第一行或前50个字符
                    let brief = result.content.lines().next()
                        .map(|l| if l.len() > 50 { format!("{}...", &l[..50]) } else { l.to_string() })
                        .unwrap_or_default();
                    if !brief.is_empty() {
                        summary.push_str(&format!(" - {}", brief));
                    }
                }
            } else if let Some(err) = &result.error {
                summary.push_str(&format!(" - 失败: {}", err));
            }

            summary.push('\n');
        }

        summary
    }

    /// 获取函数的友好显示名称
    fn get_function_display_name(function: &str) -> &str {
        match function {
            // 任务操作
            "create_task" => "创建任务",
            "update_task" => "更新任务",
            "delete_task" => "删除任务",
            "complete_task" => "完成任务",
            "get_tasks" => "查询任务列表",
            "get_task_detail" => "查询任务详情",
            "search_tasks" => "搜索任务",
            // 番茄钟操作
            "start_pomodoro" => "开始番茄钟",
            "stop_pomodoro" => "停止番茄钟",
            "pause_pomodoro" => "暂停番茄钟",
            "resume_pomodoro" => "继续番茄钟",
            // 数据查询
            "query_data" => "查询数据",
            "search_sql" => "搜索SQL历史",
            "get_time_stats" => "获取时间统计",
            // 报表生成
            "generate_daily_report" => "生成日报",
            "generate_weekly_report" => "生成周报",
            "analyze_efficiency" => "分析效率",
            _ => function,
        }
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

                // 获取昨日数据用于对比分析
                let yesterday = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                    .ok()
                    .and_then(|d| d.pred_opt())
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| {
                        // 如果解析失败，手动计算昨天
                        chrono::Local::now()
                            .date_naive()
                            .pred_opt()
                            .unwrap()
                            .format("%Y-%m-%d")
                            .to_string()
                    });

                let yesterday_start = format!("{} 00:00:00", yesterday);
                let yesterday_end = format!("{} 23:59:59", yesterday);

                // 查询昨日完成的任务数量
                let yesterday_task_count: i32 = conn.query_row(
                    "SELECT COUNT(*) FROM tasks
                     WHERE status = 'done'
                       AND completed_at >= ?1
                       AND completed_at <= ?2",
                    [&yesterday_start, &yesterday_end],
                    |row| row.get(0),
                ).unwrap_or(0);

                // 查询昨日专注时长
                let yesterday_focus_minutes: i32 = conn.query_row(
                    "SELECT COALESCE(SUM(focus_time), 0) FROM pomodoro_sessions
                     WHERE created_at >= ?1 AND created_at <= ?2",
                    [&yesterday_start, &yesterday_end],
                    |row| row.get(0),
                ).unwrap_or(0);

                log::info!("[AI Chat] 📊 昨日数据 ({}): 完成任务 {} 个, 专注时长 {} 分钟",
                    yesterday, yesterday_task_count, yesterday_focus_minutes);

                // 使用 AI 提示词模板生成智能日报
                let ai_prompt = Self::build_daily_report_prompt(
                    date,
                    &completed_tasks,
                    focus_minutes,
                    pomodoro_count,
                    yesterday_task_count,
                    yesterday_focus_minutes,
                );

                log::info!("[AI Chat] 🤖 准备调用 AI 生成智能日报");

                // 返回一个特殊的响应类型，让主流程调用 AI 生成日报
                // 将提示词和数据打包在响应中
                Ok(ChatResponse {
                    content: ai_prompt,
                    response_type: "need_ai_generation".to_string(),
                    data: Some(json!({
                        "date": date,
                        "completed_tasks": completed_tasks.iter().map(|(title, priority)| json!({
                            "title": title,
                            "priority": priority
                        })).collect::<Vec<_>>(),
                        "task_count": completed_tasks.len(),
                        "focus_minutes": focus_minutes,
                        "pomodoro_count": pomodoro_count,
                        "yesterday_task_count": yesterday_task_count,
                        "yesterday_focus_minutes": yesterday_focus_minutes,
                        "generation_type": "daily_report"
                    })),
                    context: None,
                })
            }
            "generate_weekly_report" => {
                log::info!("[AI Chat] 📊 生成周报");

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

                log::info!("[AI Chat] 📊 本周数据: 完成任务 {} 个, 专注时长 {} 分钟, 番茄钟 {} 个, 待办 {} 个",
                    completed_count, focus_minutes, pomodoro_count, pending_count);

                // 获取上周数据用于对比分析
                let last_week_completed: i32 = conn.query_row(
                    "SELECT COUNT(*) FROM tasks WHERE status = 'done'
                     AND date(completed_at) >= date('now', 'localtime', 'weekday 0', '-13 days')
                     AND date(completed_at) < date('now', 'localtime', 'weekday 0', '-6 days')",
                    [],
                    |row| row.get(0),
                ).unwrap_or(0);

                let last_week_focus_minutes: i32 = conn.query_row(
                    "SELECT COALESCE(SUM(focus_time), 0) FROM pomodoro_sessions
                     WHERE date(created_at) >= date('now', 'localtime', 'weekday 0', '-13 days')
                     AND date(created_at) < date('now', 'localtime', 'weekday 0', '-6 days')",
                    [],
                    |row| row.get(0),
                ).unwrap_or(0);

                let last_week_pomodoro: i32 = conn.query_row(
                    "SELECT COUNT(*) FROM pomodoro_sessions
                     WHERE date(created_at) >= date('now', 'localtime', 'weekday 0', '-13 days')
                     AND date(created_at) < date('now', 'localtime', 'weekday 0', '-6 days')",
                    [],
                    |row| row.get(0),
                ).unwrap_or(0);

                log::info!("[AI Chat] 📊 上周数据: 完成任务 {} 个, 专注时长 {} 分钟, 番茄钟 {} 个",
                    last_week_completed, last_week_focus_minutes, last_week_pomodoro);

                // 使用 AI 提示词模板生成智能周报
                let ai_prompt = Self::build_weekly_report_prompt(
                    completed_count,
                    focus_minutes,
                    pomodoro_count,
                    pending_count,
                    &completed_tasks,
                    last_week_completed,
                    last_week_focus_minutes,
                    last_week_pomodoro,
                );

                log::info!("[AI Chat] 🤖 准备调用 AI 生成智能周报");

                // 返回一个特殊的响应类型，让主流程调用 AI 生成周报
                Ok(ChatResponse {
                    content: ai_prompt,
                    response_type: "need_ai_generation".to_string(),
                    data: Some(json!({
                        "completed_tasks": completed_count,
                        "focus_minutes": focus_minutes,
                        "pomodoro_count": pomodoro_count,
                        "pending_count": pending_count,
                        "task_list": completed_tasks.iter().map(|(title, priority)| json!({
                            "title": title,
                            "priority": priority
                        })).collect::<Vec<_>>(),
                        "last_week_completed": last_week_completed,
                        "last_week_focus_minutes": last_week_focus_minutes,
                        "last_week_pomodoro": last_week_pomodoro,
                        "generation_type": "weekly_report"
                    })),
                    context: None,
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
                context: None,
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
                        context: None,
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
                    context: None,
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
                    context: None,
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

        // 如果没有查询到数据，根据查询类型给出更友好的提示
        if row_count == 0 {
            log::info!("[AI Chat] ℹ️ 查询无结果: {}", description);
            let desc_lower = description.to_lowercase();
            let empty_message = if desc_lower.contains("任务") || desc_lower.contains("task") {
                if desc_lower.contains("待办") || desc_lower.contains("todo") || desc_lower.contains("今天") {
                    "🎉 太棒了！当前没有待办任务，你可以好好休息一下，或者创建一个新任务开始新的工作！".to_string()
                } else if desc_lower.contains("完成") || desc_lower.contains("done") {
                    "暂时没有已完成的任务记录。完成一些任务后再来查看吧！".to_string()
                } else {
                    "当前没有符合条件的任务。你可以创建一个新任务开始工作！".to_string()
                }
            } else if desc_lower.contains("番茄") || desc_lower.contains("pomodoro") || desc_lower.contains("专注") {
                "暂时没有番茄钟记录。开启一个番茄钟，专注工作吧！🍅".to_string()
            } else if desc_lower.contains("sql") {
                "没有找到相关的 SQL 记录。".to_string()
            } else {
                "抱歉，没有找到相关的数据。你可以换个关键词或条件试试。".to_string()
            };
            return Ok(ChatResponse {
                content: empty_message,
                response_type: "text".to_string(),
                data: None,
                context: None,
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
            context: None,
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
            context: None,
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
                    context: None,
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
                    context: None,
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
                context: None,
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
            context: None,
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
                    context: None,
                })
            }
            None => {
                Ok(ChatResponse {
                    content: format!("❌ 没有找到 ID 为 {} 的任务", task_id),
                    response_type: "text".to_string(),
                    data: None,
                    context: None,
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
                "SELECT id, title FROM tasks WHERE title LIKE ?1 AND status NOT IN ('done', 'cancelled') ORDER BY created_at DESC LIMIT 1",
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
                    "UPDATE tasks SET status = 'done', completed_at = ?1 WHERE id = ?2",
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

                // 构建操作上下文
                let context = OperationContext {
                    last_task: Some(TaskContext {
                        id,
                        title: title.clone(),
                    }),
                    last_pomodoro: None,
                    recent_tasks: None,
                };

                Ok(ChatResponse {
                    content,
                    response_type: "text".to_string(),
                    data: Some(json!({
                        "task_id": id,
                        "title": title,
                        "completed_at": now,
                        "today_done_count": today_done,
                    })),
                    context: Some(context),
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
                    context: None,
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
            context: None,
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
            context: None,
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

        // 构建操作上下文，记录刚创建的任务
        let context = OperationContext {
            last_task: Some(TaskContext {
                id: task_id,
                title: title.to_string(),
            }),
            last_pomodoro: None,
            recent_tasks: None,
        };

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
            context: Some(context),
        })
    }

    /// 处理 update_task
    /// 支持两种调用方式：
    /// 1. 通过 task_id 直接更新
    /// 2. 通过 title 模糊匹配查找任务后更新（用户友好）
    fn handle_update_task(conn: &Connection, args: &serde_json::Value) -> Result<ChatResponse> {
        // 获取参数
        let task_id_arg = args["task_id"].as_i64();
        let title_arg = args["title"].as_str();
        let new_status = args["status"].as_str();
        let new_description = args["description"].as_str();
        let new_priority = args["priority"].as_i64().map(|p| TaskPriority::from_i32(p as i32));

        // 确定任务 ID
        let task_id = if let Some(id) = task_id_arg {
            // 直接使用提供的 task_id
            id
        } else if let Some(title_keyword) = title_arg {
            // 通过标题关键词查找任务
            let keyword = title_keyword.trim();
            log::info!("[AI Chat] 🔍 通过标题查找任务: \"{}\"", keyword);

            // 先尝试精确匹配，再尝试模糊匹配
            let found_id: Option<i64> = conn.query_row(
                "SELECT id FROM tasks WHERE title = ?1 AND status NOT IN ('done', 'cancelled') LIMIT 1",
                [keyword],
                |row| row.get(0),
            ).ok();

            let task_id = if let Some(id) = found_id {
                log::info!("[AI Chat] ✅ 精确匹配找到任务 ID: {}", id);
                id
            } else {
                // 尝试模糊匹配（LIKE）
                let like_pattern = format!("%{}%", keyword);
                let fuzzy_id: Option<i64> = conn.query_row(
                    "SELECT id FROM tasks WHERE title LIKE ?1 AND status NOT IN ('done', 'cancelled') ORDER BY created_at DESC LIMIT 1",
                    [&like_pattern],
                    |row| row.get(0),
                ).ok();

                if let Some(id) = fuzzy_id {
                    log::info!("[AI Chat] ✅ 模糊匹配找到任务 ID: {}", id);
                    id
                } else {
                    // 没找到任务，返回友好提示
                    return Ok(ChatResponse {
                        content: format!("抱歉，没有找到标题包含 \"{}\" 的待办任务。\n\n请确认：\n1. 任务名称是否正确？\n2. 任务是否已经完成？\n\n你可以说 \"查看我的任务\" 来查看当前所有任务。", keyword),
                        response_type: "error".to_string(),
                        data: Some(json!({
                            "error": "task_not_found",
                            "search_keyword": keyword
                        })),
                        context: None,
                    });
                }
            };

            task_id
        } else {
            // 既没有 task_id 也没有 title
            return Ok(ChatResponse {
                content: "请告诉我要更新哪个任务。你可以：\n1. 提供任务 ID\n2. 提供任务标题（或部分标题）\n\n例如：\"把'写周报'设为进行中\"".to_string(),
                response_type: "error".to_string(),
                data: Some(json!({
                    "error": "missing_task_identifier"
                })),
                context: None,
            });
        };

        // 获取原任务信息
        let old_task: (String, String, i32, String) = conn.query_row(
            "SELECT title, COALESCE(description, ''), priority, status FROM tasks WHERE id = ?1",
            [task_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;

        let old_status = old_task.3.clone();
        let mut has_changes = false;
        let mut content = format!("## ✏️ 任务更新成功！\n\n");
        content.push_str(&format!("📋 **任务**：{}\n", old_task.0));
        content.push_str(&format!("🆔 **ID**：#{}\n\n", task_id));
        content.push_str("### 更新内容：\n\n");

        // 处理状态更新
        if let Some(status_str) = new_status {
            let status_changed = status_str != old_status;
            if status_changed {
                match status_str {
                    "active" | "in_progress" | "进行中" => {
                        TaskService::start_task(conn, task_id)?;
                        let old_status_text = match old_status.as_str() {
                            "todo" => "⏳ 待办",
                            "active" | "in_progress" => "🔥 进行中",
                            "done" => "✅ 已完成",
                            "deferred" => "⏸️ 已推迟",
                            _ => "未知",
                        };
                        content.push_str(&format!("📊 **状态**：{} → 🔥 进行中\n", old_status_text));
                        has_changes = true;
                    }
                    "done" | "completed" | "已完成" => {
                        TaskService::complete_task(conn, task_id)?;
                        content.push_str(&format!("📊 **状态**：{} → ✅ 已完成\n",
                            match old_status.as_str() {
                                "todo" => "⏳ 待办",
                                "active" => "🔥 进行中",
                                _ => &old_status,
                            }
                        ));
                        has_changes = true;
                    }
                    "deferred" | "postponed" | "推迟" => {
                        TaskService::defer_task(conn, task_id)?;
                        content.push_str(&format!("📊 **状态**：{} → ⏸️ 已推迟\n",
                            match old_status.as_str() {
                                "todo" => "⏳ 待办",
                                "active" => "🔥 进行中",
                                _ => &old_status,
                            }
                        ));
                        has_changes = true;
                    }
                    "todo" => {
                        // 重置为待办状态
                        conn.execute(
                            "UPDATE tasks SET status = 'todo' WHERE id = ?1",
                            [task_id],
                        )?;
                        content.push_str(&format!("📊 **状态**：{} → ⏳ 待办\n",
                            match old_status.as_str() {
                                "active" => "🔥 进行中",
                                "done" => "✅ 已完成",
                                "deferred" => "⏸️ 已推迟",
                                _ => &old_status,
                            }
                        ));
                        has_changes = true;
                    }
                    _ => {
                        log::warn!("[AI Chat] 未知的状态值: {}", status_str);
                    }
                }
            }
        }

        // 更新其他字段（标题、描述、优先级）
        // 注意：如果用 title 查找任务，不更新标题本身
        let update_title = if task_id_arg.is_some() { title_arg } else { None };

        if update_title.is_some() || new_description.is_some() || new_priority.is_some() {
            TaskService::update_task(
                conn,
                task_id,
                update_title,
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

            if let Some(title) = update_title {
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
        }

        if !has_changes {
            content.push_str("(没有检测到实际更改)\n");
        }

        content.push_str("\n💡 继续加油！");

        // 获取最终的任务标题
        let final_title = update_title.unwrap_or(&old_task.0);

        // 构建操作上下文
        let context = OperationContext {
            last_task: Some(TaskContext {
                id: task_id,
                title: final_title.to_string(),
            }),
            last_pomodoro: None,
            recent_tasks: None,
        };

        Ok(ChatResponse {
            content,
            response_type: "task_updated".to_string(),
            data: Some(json!({
                "task_id": task_id,
                "task_title": final_title,
                "updated_fields": {
                    "title": update_title,
                    "status": new_status,
                    "description": new_description,
                    "priority": new_priority.map(|p| p as i32)
                }
            })),
            context: Some(context),
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

        // 构建操作上下文
        let context = OperationContext {
            last_task: Some(TaskContext {
                id: task_id,
                title: task_title.clone(),
            }),
            last_pomodoro: Some(PomodoroContext {
                id: session_id,
                task_id: Some(task_id),
                task_title: Some(task_title.clone()),
            }),
            recent_tasks: None,
        };

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
            context: Some(context),
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
            context: None,
        })
    }

    // ==================== 关键信息提取功能 ====================

    /// 从对话结果中提取可记忆的上下文信息
    ///
    /// # 参数
    /// - `user_message`: 用户的原始消息
    /// - `function_name`: 执行的函数名（如果有）
    /// - `result`: AI 的响应结果
    ///
    /// # 返回
    /// 返回需要保存的上下文列表: (context_type, key, value, importance)
    /// - context_type: "entity"（实体）、"preference"（偏好）、"fact"（事实）
    /// - key: 上下文的键名
    /// - value: JSON 格式的详细信息
    /// - importance: 重要性评分 (1-5)
    pub fn extract_context_from_response(
        user_message: &str,
        function_name: Option<&str>,
        result: &ChatResponse,
    ) -> Vec<(String, String, serde_json::Value, i32)> {
        let mut contexts = Vec::new();

        // 1. 从用户消息中提取实体
        let entities = Self::extract_entities_from_message(user_message);
        for (key, value_str, importance) in entities {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&value_str) {
                contexts.push(("entity".to_string(), key, value, importance));
            }
        }

        // 2. 从执行结果中提取偏好（Function Call 参数）
        if let Some(func_name) = function_name {
            // 从 result.data 中提取参数信息
            if let Some(ref data) = result.data {
                if let Some((pref_key, pref_value, importance)) =
                    Self::extract_user_preferences(func_name, data) {
                    contexts.push(("preference".to_string(), pref_key, pref_value, importance));
                }
            }
        }

        // 3. 从用户消息中提取隐含偏好（新增）
        let implicit_preferences = Self::extract_implicit_preferences(user_message);
        for (pref_type, key, value, importance) in implicit_preferences {
            contexts.push((pref_type, key, value, importance));
        }

        // 4. 从用户消息中提取事实（如截止日期、计划）
        let facts = Self::extract_facts_from_message(user_message);
        for (key, value, importance) in facts {
            contexts.push(("fact".to_string(), key, value, importance));
        }

        contexts
    }

    /// 从用户消息中提取实体（项目名、任务名等）
    ///
    /// # 返回
    /// (key, value_json_string, importance)
    fn extract_entities_from_message(message: &str) -> Vec<(String, String, i32)> {
        let mut entities = Vec::new();
        let msg_lower = message.to_lowercase();

        // 检测任务创建意图
        if msg_lower.contains("创建") || msg_lower.contains("新建") ||
           msg_lower.contains("添加") || msg_lower.contains("建个") {

            // 尝试提取任务标题（引号内容或"创建/新建"后的内容）
            if let Some(task_title) = Self::extract_quoted_text(message)
                .or_else(|| Self::extract_after_keywords(message, &["创建", "新建", "添加", "建个"])) {

                let entity_key = format!("task_{}", Self::generate_entity_id(&task_title));
                let entity_value = json!({
                    "type": "task",
                    "title": task_title,
                    "source": "user_message",
                    "original_message": message,
                });

                entities.push((
                    entity_key,
                    entity_value.to_string(),
                    3, // 重要性: 中等
                ));
            }
        }

        // 检测项目名提及
        if let Some(project_name) = Self::extract_project_name(message) {
            let entity_key = format!("project_{}", Self::generate_entity_id(&project_name));
            let entity_value = json!({
                "type": "project",
                "name": project_name,
                "mentioned_at": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            });

            entities.push((
                entity_key,
                entity_value.to_string(),
                4, // 重要性: 较高
            ));
        }

        entities
    }

    /// 从用户的操作模式中提取偏好（如常用的番茄钟时长）
    ///
    /// # 返回
    /// Some((key, value, importance))
    fn extract_user_preferences(
        function_name: &str,
        args: &serde_json::Value,
    ) -> Option<(String, serde_json::Value, i32)> {
        match function_name {
            "start_pomodoro" => {
                // 提取番茄钟时长偏好
                if let Some(duration) = args.get("duration")
                    .or_else(|| args.get("duration_minutes"))
                    .and_then(|v| v.as_i64()) {

                    let pref_key = "pomodoro_duration_preference".to_string();
                    let pref_value = json!({
                        "duration_minutes": duration,
                        "last_used": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    });

                    return Some((pref_key, pref_value, 3));
                }
            }

            "create_task" => {
                // 提取任务优先级偏好
                if let Some(priority) = args.get("priority").and_then(|v| v.as_i64()) {
                    let pref_key = "task_priority_preference".to_string();
                    let pref_value = json!({
                        "priority": priority,
                        "last_used": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    });

                    return Some((pref_key, pref_value, 2));
                }
            }

            _ => {}
        }

        None
    }

    /// 从用户消息中提取隐含的习惯和偏好
    ///
    /// 这个方法能够识别用户在对话中表达的隐含习惯，例如：
    /// - 时间偏好："我一般下午效率高" → preference, "high_efficiency_time", "下午", 7
    /// - 工作模式："我习惯用番茄钟" → preference, "work_method", "番茄钟", 6
    /// - 优先级表达："这个很重要" → 当前任务重要性提示
    /// - 项目关联："这是XXX项目的" → entity, "task_project", "XXX项目", 8
    ///
    /// # 参数
    /// * `user_message` - 用户的原始消息
    ///
    /// # 返回
    /// Vec<(context_type, key, value, importance)>
    /// - context_type: "preference"（偏好）或 "entity"（实体）
    /// - key: 上下文键名
    /// - value: JSON 值
    /// - importance: 重要性评分 (1-10)
    pub fn extract_implicit_preferences(
        user_message: &str,
    ) -> Vec<(String, String, serde_json::Value, i32)> {
        let mut preferences = Vec::new();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // 1. 检测时间偏好
        for pattern in TIME_PREFERENCE_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(user_message) {
                // 提取时间段（早上/下午等）
                let time_period = if let Some(m) = caps.get(2) {
                    m.as_str()
                } else if let Some(m) = caps.get(1) {
                    m.as_str()
                } else {
                    continue;
                };

                // 判断是否是明确表达的偏好（"我习惯"、"我喜欢"）
                let importance = if user_message.contains("我习惯") || user_message.contains("我一般") {
                    8  // 明确表达的习惯，重要性很高
                } else if user_message.contains("我喜欢") {
                    7  // 明确表达的偏好
                } else {
                    6  // 隐含的偏好
                };

                let key = "time_preference".to_string();
                let value = json!({
                    "time_period": time_period,
                    "pattern": "efficiency_or_habit",
                    "original_message": user_message,
                    "extracted_at": now,
                });

                preferences.push(("preference".to_string(), key, value, importance));

                log::info!("[Context Memory] 提取到时间偏好: {} (重要性: {})", time_period, importance);
                break; // 只提取第一个匹配的时间偏好
            }
        }

        // 2. 检测工作模式偏好
        for pattern in WORK_METHOD_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(user_message) {
                // 提取工作方法和时长
                let mut method = String::new();
                let mut duration: Option<i32> = None;

                // 尝试提取番茄钟时长
                for i in 1..=caps.len() {
                    if let Some(m) = caps.get(i) {
                        let text = m.as_str();

                        // 检查是否是数字（时长）
                        if let Ok(num) = text.parse::<i32>() {
                            duration = Some(num);
                        } else if text.contains("番茄钟") || text.contains("专注") ||
                                  text.contains("分块") || text.contains("集中") {
                            method = text.to_string();
                        }
                    }
                }

                // 如果没有明确方法，默认为番茄钟
                if method.is_empty() && duration.is_some() {
                    method = "番茄钟".to_string();
                } else if method.is_empty() {
                    method = "专注工作".to_string();
                }

                let importance = if user_message.contains("我习惯") || user_message.contains("我用") {
                    7  // 明确表达的工作习惯
                } else {
                    6  // 隐含的工作偏好
                };

                let key = if duration.is_some() {
                    "pomodoro_duration_preference".to_string()
                } else {
                    "work_method_preference".to_string()
                };

                let mut value_map = serde_json::Map::new();
                value_map.insert("method".to_string(), json!(method));
                if let Some(dur) = duration {
                    value_map.insert("duration_minutes".to_string(), json!(dur));
                }
                value_map.insert("original_message".to_string(), json!(user_message));
                value_map.insert("extracted_at".to_string(), json!(now));

                let value = serde_json::Value::Object(value_map);
                preferences.push(("preference".to_string(), key, value, importance));

                log::info!("[Context Memory] 提取到工作模式偏好: {} (时长: {:?}, 重要性: {})",
                          method, duration, importance);
                break; // 只提取第一个匹配的工作模式
            }
        }

        // 3. 检测优先级表达
        for pattern in PRIORITY_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(user_message) {
                let matched_text = caps.get(0).map(|m| m.as_str()).unwrap_or("");

                let (priority_level, importance) = if matched_text.contains("重要") ||
                                                      matched_text.contains("必须") ||
                                                      matched_text.contains("紧急") {
                    ("high", 8)  // 高优先级表达
                } else if matched_text.contains("不着急") ||
                          matched_text.contains("不急") ||
                          matched_text.contains("慢慢来") {
                    ("low", 5)  // 低优先级表达
                } else if matched_text.contains("今天") {
                    ("urgent", 9)  // 紧急任务
                } else {
                    ("normal", 4)
                };

                let key = "current_task_priority".to_string();
                let value = json!({
                    "priority_level": priority_level,
                    "priority_expression": matched_text,
                    "original_message": user_message,
                    "extracted_at": now,
                });

                preferences.push(("preference".to_string(), key, value, importance));

                log::info!("[Context Memory] 提取到优先级表达: {} (重要性: {})",
                          priority_level, importance);
                break; // 只提取第一个优先级表达
            }
        }

        // 4. 检测项目关联
        for pattern in PROJECT_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(user_message) {
                let mut project_name = String::new();

                // 提取项目名称
                for i in 1..=caps.len() {
                    if let Some(m) = caps.get(i) {
                        let text = m.as_str().trim();
                        // 排除一些非项目名的词
                        if !text.is_empty() &&
                           text != "上次" && text != "之前" && text != "刚才" &&
                           text != "相关" && text != "有关" && text != "的" &&
                           text != "任务" && text != "项目" {
                            project_name = text.to_string();
                            break;
                        }
                    }
                }

                if !project_name.is_empty() {
                    let importance = if user_message.contains("这是") {
                        8  // 明确的项目关联
                    } else if user_message.contains("和上次") || user_message.contains("和之前") {
                        7  // 关联之前的任务
                    } else {
                        6  // 隐含的项目关联
                    };

                    let key = format!("task_project_{}", Self::generate_entity_id(&project_name));
                    let value = json!({
                        "project_name": project_name,
                        "association_type": if user_message.contains("和上次") { "previous_task" } else { "current_task" },
                        "original_message": user_message,
                        "extracted_at": now,
                    });

                    preferences.push(("entity".to_string(), key, value, importance));

                    log::info!("[Context Memory] 提取到项目关联: {} (重要性: {})",
                              project_name, importance);
                    break; // 只提取第一个项目关联
                }
            }
        }

        preferences
    }

    /// 从用户消息中提取事实（截止日期、计划等）
    ///
    /// # 返回
    /// Vec<(key, value, importance)>
    fn extract_facts_from_message(message: &str) -> Vec<(String, serde_json::Value, i32)> {
        let mut facts = Vec::new();
        let msg_lower = message.to_lowercase();

        // 检测截止日期
        let deadline_keywords = [
            ("今天", 0),
            ("明天", 1),
            ("后天", 2),
            ("周一", 1),
            ("周二", 2),
            ("周三", 3),
            ("周四", 4),
            ("周五", 5),
            ("周六", 6),
            ("周日", 7),
            ("下周", 7),
        ];

        for (keyword, days_offset) in deadline_keywords {
            if msg_lower.contains(keyword) {
                // 提取截止日期相关的任务描述
                if let Some(task_desc) = Self::extract_task_near_keyword(message, keyword) {
                    let deadline_date = chrono::Local::now() + chrono::Duration::days(days_offset);

                    let fact_key = format!("deadline_{}", Self::generate_entity_id(&task_desc));
                    let fact_value = json!({
                        "type": "deadline",
                        "task_description": task_desc,
                        "deadline": deadline_date.format("%Y-%m-%d").to_string(),
                        "keyword": keyword,
                        "original_message": message,
                        "created_at": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    });

                    facts.push((fact_key, fact_value, 5)); // 截止日期很重要
                    break; // 只提取第一个匹配的截止日期
                }
            }
        }

        // 检测计划/安排
        if msg_lower.contains("计划") || msg_lower.contains("打算") ||
           msg_lower.contains("准备") || msg_lower.contains("要做") {

            if let Some(plan_desc) = Self::extract_after_keywords(message, &["计划", "打算", "准备", "要做"]) {
                let fact_key = format!("plan_{}", Self::generate_entity_id(&plan_desc));
                let fact_value = json!({
                    "type": "plan",
                    "description": plan_desc,
                    "mentioned_at": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    "original_message": message,
                });

                facts.push((fact_key, fact_value, 3));
            }
        }

        facts
    }

    // ==================== 辅助函数 ====================

    /// 提取引号内的文本
    fn extract_quoted_text(text: &str) -> Option<String> {
        // 支持中英文引号
        let quote_pairs = [
            ('"', '"'),
            ('「', '」'),
            ('『', '』'),
            ('【', '】'),
            ('\'', '\''),
        ];

        for (open, close) in quote_pairs {
            if let Some(start) = text.find(open) {
                if let Some(end) = text[start + 1..].find(close) {
                    let content = &text[start + 1..start + 1 + end];
                    if !content.trim().is_empty() {
                        return Some(content.trim().to_string());
                    }
                }
            }
        }

        None
    }

    /// 提取关键词后的文本内容
    fn extract_after_keywords(text: &str, keywords: &[&str]) -> Option<String> {
        for keyword in keywords {
            if let Some(pos) = text.find(keyword) {
                let after = &text[pos + keyword.len()..];
                let trimmed = after.trim();

                // 提取到下一个标点符号或结尾
                let content = if let Some(punct_pos) = trimmed
                    .find(|c: char| matches!(c, '，' | '。' | '！' | '？' | ',' | '.' | '!' | '?')) {
                    &trimmed[..punct_pos]
                } else {
                    trimmed
                };

                let content = content.trim();
                if !content.is_empty() && content.len() <= 100 {
                    return Some(content.to_string());
                }
            }
        }

        None
    }

    /// 提取项目名称（简单规则：识别常见项目名模式）
    fn extract_project_name(text: &str) -> Option<String> {
        let msg_lower = text.to_lowercase();

        // 检测项目关键词
        if msg_lower.contains("项目") {
            if let Some(project_name) = Self::extract_after_keywords(text, &["项目"]) {
                // 排除一些非项目名的常见词
                let excluded = ["这个", "那个", "我的", "我们"];
                if !excluded.iter().any(|&ex| project_name.starts_with(ex)) {
                    return Some(project_name);
                }
            }
        }

        None
    }

    /// 提取关键词附近的任务描述
    fn extract_task_near_keyword(text: &str, keyword: &str) -> Option<String> {
        if let Some(pos) = text.find(keyword) {
            // 尝试提取关键词前后的内容
            let before = if pos > 0 { &text[..pos] } else { "" };
            let after = &text[pos + keyword.len()..];

            // 优先提取关键词前的内容（通常是"XX要交报告"中的"报告"）
            if !before.trim().is_empty() {
                let before_trimmed = before.trim();
                // 从最后一个标点符号开始提取
                if let Some(last_punct) = before_trimmed.rfind(|c: char|
                    matches!(c, '，' | '。' | '！' | '？' | ',' | '.' | '!' | '?')) {
                    let task = &before_trimmed[last_punct + 1..];
                    if !task.trim().is_empty() {
                        return Some(task.trim().to_string());
                    }
                } else {
                    return Some(before_trimmed.to_string());
                }
            }

            // 如果前面没有内容，尝试提取后面的
            if !after.trim().is_empty() {
                return Self::extract_after_keywords(text, &[keyword]);
            }
        }

        None
    }

    /// 生成实体 ID（基于内容的简单哈希）
    fn generate_entity_id(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = hasher.finish();

        // 返回哈希的前8位（16进制）
        format!("{:x}", hash & 0xFFFFFFFF)
    }

    /// 对话历史摘要
    /// 当消息数量超过阈值时，将旧消息压缩为摘要
    ///
    /// # 参数
    /// * `messages` - 原始消息列表
    /// * `max_messages` - 保留的最大消息数（包括摘要）
    ///
    /// # 返回
    /// 压缩后的消息列表
    ///
    /// # 策略
    /// - 如果消息数 <= max_messages，直接返回原消息
    /// - 如果消息数 > max_messages：
    ///   - 保留最近 max_messages/2 条消息（原样）
    ///   - 将更早的消息压缩为一条摘要消息
    pub fn summarize_conversation_history(
        messages: &[ChatMessage],
        max_messages: usize,
    ) -> Vec<ChatMessage> {
        // 如果消息数量在限制内，直接返回
        if messages.len() <= max_messages {
            return messages.to_vec();
        }

        log::info!(
            "[AI Chat] 对话历史过长 ({} 条)，开始压缩，保留最近 {} 条",
            messages.len(),
            max_messages
        );

        // 计算保留的最近消息数量（至少保留一半）
        let keep_recent = (max_messages / 2).max(1);
        let compress_count = messages.len() - keep_recent;

        // 分割消息：需要压缩的旧消息 和 保留的新消息
        let (old_messages, recent_messages) = messages.split_at(compress_count);

        // 从旧消息中提取关键信息
        let key_points = Self::extract_conversation_key_points(old_messages);

        // 创建摘要消息
        let summary = if key_points.is_empty() {
            "[对话摘要] 之前进行了一些日常交流".to_string()
        } else {
            format!("[对话摘要] 之前讨论了：\n{}", key_points.join("\n"))
        };

        // 构建新的消息列表：摘要 + 最近消息
        let mut result = vec![ChatMessage::system(summary)];
        result.extend_from_slice(recent_messages);

        log::info!(
            "[AI Chat] 压缩完成：{} 条旧消息 -> 1 条摘要，保留 {} 条最近消息",
            compress_count,
            keep_recent
        );

        result
    }

    /// 从消息列表中提取关键信息点
    ///
    /// # 参数
    /// * `messages` - 消息列表
    ///
    /// # 返回
    /// 关键信息点列表
    fn extract_conversation_key_points(messages: &[ChatMessage]) -> Vec<String> {
        let mut key_points = Vec::new();

        for message in messages {
            if let Some(point) = Self::extract_key_points(message) {
                key_points.push(point);
            }
        }

        // 去重并限制数量（最多10个关键点）
        key_points.sort();
        key_points.dedup();
        key_points.truncate(10);

        key_points
    }

    /// 从单条消息中提取关键信息
    ///
    /// # 参数
    /// * `message` - 单条消息
    ///
    /// # 返回
    /// 提取的关键信息，如果不重要则返回 None
    fn extract_key_points(message: &ChatMessage) -> Option<String> {
        // 只处理用户和助手的消息，忽略系统消息
        if message.role != "user" && message.role != "assistant" {
            return None;
        }

        let content = message.content.trim();

        // 忽略过短的消息
        if content.chars().count() < 5 {
            return None;
        }

        // 忽略纯问候语
        if Self::is_greeting(content) {
            return None;
        }

        // 提取任务相关操作
        if let Some(point) = Self::extract_task_operation(content) {
            return Some(point);
        }

        // 提取番茄钟操作
        if let Some(point) = Self::extract_pomodoro_operation(content) {
            return Some(point);
        }

        // 提取查询操作
        if let Some(point) = Self::extract_query_operation(content) {
            return Some(point);
        }

        // 提取用户偏好或意图
        if let Some(point) = Self::extract_user_preference_from_message(content) {
            return Some(point);
        }

        None
    }

    /// 判断消息是否为纯问候语
    fn is_greeting(content: &str) -> bool {
        let greetings = [
            "你好", "您好", "hi", "hello", "嗨", "早", "晚上好",
            "好的", "谢谢", "感谢", "ok", "嗯", "哦", "是的", "对",
            "收到", "知道了", "明白", "了解"
        ];

        let lower = content.to_lowercase();
        greetings.iter().any(|g| lower == *g || lower == format!("{}！", g) || lower == format!("{}。", g))
    }

    /// 提取任务相关操作
    fn extract_task_operation(content: &str) -> Option<String> {
        let patterns = [
            ("创建", "任务"),
            ("新建", "任务"),
            ("添加", "任务"),
            ("完成", "任务"),
            ("删除", "任务"),
            ("修改", "任务"),
            ("更新", "任务"),
        ];

        for (action, obj) in patterns {
            if content.contains(action) && content.contains(obj) {
                // 尝试提取任务名称
                let summary = safe_truncate(content, 30);
                return Some(format!("- {}了{}: {}", action, obj, summary));
            }
        }

        None
    }

    /// 提取番茄钟操作
    fn extract_pomodoro_operation(content: &str) -> Option<String> {
        let patterns = [
            "开始番茄钟",
            "启动番茄钟",
            "暂停番茄钟",
            "继续番茄钟",
            "完成番茄钟",
            "结束番茄钟",
        ];

        for pattern in patterns {
            if content.contains(pattern) {
                let summary = safe_truncate(content, 30);
                return Some(format!("- {}", summary));
            }
        }

        None
    }

    /// 提取查询操作
    fn extract_query_operation(content: &str) -> Option<String> {
        let query_keywords = [
            "查询", "查看", "显示", "列出", "统计", "分析",
            "有哪些", "多少", "什么时候", "怎么样"
        ];

        for keyword in query_keywords {
            if content.contains(keyword) {
                let summary = safe_truncate(content, 30);
                return Some(format!("- 查询了: {}", summary));
            }
        }

        None
    }

    /// 提取用户偏好或意图
    fn extract_user_preference_from_message(content: &str) -> Option<String> {
        let preference_keywords = [
            "喜欢", "希望", "想要", "需要", "偏好", "习惯",
            "每天", "每周", "通常", "一般"
        ];

        for keyword in preference_keywords {
            if content.contains(keyword) {
                let summary = safe_truncate(content, 30);
                return Some(format!("- 用户偏好: {}", summary));
            }
        }

        None
    }

    /// 判断消息是否重要（需要保留）
    ///
    /// # 参数
    /// * `message` - 消息对象
    ///
    /// # 返回
    /// true 表示重要需要保留，false 表示可以压缩
    #[allow(dead_code)]
    fn is_important_message(message: &ChatMessage) -> bool {
        let content = message.content.trim();

        // 系统消息通常重要
        if message.role == "system" {
            return true;
        }

        // 包含操作指令的消息
        let important_keywords = [
            "创建", "删除", "修改", "更新", "完成",
            "番茄钟", "提醒", "统计", "分析", "报告"
        ];

        important_keywords.iter().any(|k| content.contains(k))
    }
}

// ============================================================================
// 模板化响应函数
// ============================================================================

/// 将任务列表格式化为友好的文本响应
///
/// # 参数
/// * `tasks` - 任务数据 JSON 数组
/// * `query_type` - 查询类型（如 "待办"、"今日完成"、"本周"）
pub fn format_task_list_response(tasks: &serde_json::Value, query_type: &str) -> String {
    let tasks_array = match tasks.as_array() {
        Some(arr) => arr,
        None => return format!("❌ 任务数据格式错误"),
    };

    if tasks_array.is_empty() {
        return match query_type {
            "待办" => "✨ 太棒了！你暂时没有待办任务。".to_string(),
            "今日完成" => "📝 今天还没有完成任何任务，加油！".to_string(),
            "本周" => "📅 本周还没有任务记录。".to_string(),
            _ => format!("暂无{}任务", query_type),
        };
    }

    let count = tasks_array.len();
    let mut response = format!("你有 {} 个{}任务：\n\n", count, query_type);

    for (index, task) in tasks_array.iter().enumerate() {
        let title = task["title"].as_str().unwrap_or("未命名任务");
        let priority = task["priority"].as_i64().unwrap_or(2);

        let priority_emoji = match priority {
            1 => "🔴",  // 高优先级
            2 => "🟡",  // 中优先级
            3 => "🟢",  // 低优先级
            _ => "⚪",
        };

        let priority_text = match priority {
            1 => "高优先级",
            2 => "中优先级",
            3 => "低优先级",
            _ => "普通",
        };

        response.push_str(&format!(
            "{}. {} {}（{}）\n",
            index + 1,
            priority_emoji,
            title,
            priority_text
        ));

        // 如果任务有描述，显示简短描述
        if let Some(description) = task["description"].as_str() {
            if !description.is_empty() {
                let short_desc = safe_truncate(description, 30);
                response.push_str(&format!("   💬 {}\n", short_desc));
            }
        }

        // 如果是待办任务，显示截止日期
        if query_type == "待办" {
            if let Some(due_date) = task["due_date"].as_str() {
                response.push_str(&format!("   📅 截止：{}\n", due_date));
            }
        }

        response.push('\n');
    }

    // 如果任务较多，添加总结提示
    if count > 5 {
        response.push_str(&format!(
            "💡 提示：任务较多，建议优先处理高优先级任务。\n"
        ));
    }

    response
}

/// 生成操作确认的模板响应
///
/// # 参数
/// * `operation` - 操作类型（create/update/delete/complete）
/// * `target` - 操作对象（如任务标题）
/// * `success` - 是否成功
pub fn format_operation_confirmation(operation: &str, target: &str, success: bool) -> String {
    if !success {
        return match operation {
            "create" => format!("❌ 创建任务失败：请检查输入信息"),
            "update" => format!("❌ 更新任务「{}」失败：找不到该任务", target),
            "delete" => format!("❌ 删除任务「{}」失败：找不到该任务", target),
            "complete" => format!("❌ 无法完成任务「{}」：找不到该任务", target),
            "pause_pomodoro" => format!("❌ 暂停番茄钟失败"),
            "resume_pomodoro" => format!("❌ 恢复番茄钟失败"),
            "complete_pomodoro" => format!("❌ 完成番茄钟失败"),
            _ => format!("❌ 操作失败"),
        };
    }

    match operation {
        "create" => format!("✅ 已创建任务「{}」", target),
        "update" => format!("✅ 已更新任务「{}」", target),
        "delete" => format!("✅ 已删除任务「{}」", target),
        "complete" => format!("✅ 已将「{}」标记为完成", target),
        "pause_pomodoro" => format!("⏸️ 番茄钟已暂停"),
        "resume_pomodoro" => format!("▶️ 番茄钟已恢复"),
        "complete_pomodoro" => format!("✅ 番茄钟已完成！休息一下吧~"),
        _ => format!("✅ 操作成功"),
    }
}

/// 格式化统计数据为友好文本
///
/// # 参数
/// * `stats` - 统计数据 JSON
/// * `period` - 时间段（"今日"、"本周"、"本月"）
pub fn format_stats_response(stats: &serde_json::Value, period: &str) -> String {
    let total = stats["total"].as_i64().unwrap_or(0);
    let completed = stats["completed"].as_i64().unwrap_or(0);
    let pending = stats["pending"].as_i64().unwrap_or(0);
    let focus_time = stats["focus_time"].as_i64().unwrap_or(0);
    let pomodoro_count = stats["pomodoro_count"].as_i64().unwrap_or(0);

    let mut response = format!("## 📊 {}统计\n\n", period);

    // 任务完成情况
    response.push_str("### 任务完成情况\n");
    response.push_str(&format!("- ✅ 已完成：{} 个\n", completed));
    response.push_str(&format!("- 📝 待办中：{} 个\n", pending));
    response.push_str(&format!("- 📊 总计：{} 个\n\n", total));

    // 完成率
    if total > 0 {
        let completion_rate = (completed as f64 / total as f64 * 100.0) as i32;
        let rate_emoji = if completion_rate >= 80 {
            "🌟"
        } else if completion_rate >= 60 {
            "👍"
        } else {
            "💪"
        };
        response.push_str(&format!(
            "{} 完成率：{}%\n\n",
            rate_emoji, completion_rate
        ));
    }

    // 专注时长
    if focus_time > 0 || pomodoro_count > 0 {
        response.push_str("### 专注情况\n");

        if pomodoro_count > 0 {
            response.push_str(&format!("- 🍅 番茄钟：{} 个\n", pomodoro_count));
        }

        if focus_time > 0 {
            let hours = focus_time / 60;
            let minutes = focus_time % 60;
            if hours > 0 {
                response.push_str(&format!("- ⏱️ 专注时长：{} 小时 {} 分钟\n", hours, minutes));
            } else {
                response.push_str(&format!("- ⏱️ 专注时长：{} 分钟\n", minutes));
            }
        }
        response.push('\n');
    }

    // 添加鼓励语
    if completed > 0 {
        response.push_str(match period {
            "今日" => "💪 今天很充实，继续保持！\n",
            "本周" => "🌟 本周表现不错，再接再厉！\n",
            "本月" => "🎉 本月成果丰硕，值得庆祝！\n",
            _ => "👏 做得很棒！\n",
        });
    } else {
        response.push_str(match period {
            "今日" => "📝 新的一天，从第一个任务开始吧！\n",
            "本周" => "🚀 本周刚开始，加油！\n",
            "本月" => "✨ 新的月份，新的开始！\n",
            _ => "加油！\n",
        });
    }

    response
}

/// 格式化番茄钟操作响应
///
/// # 参数
/// * `operation` - 操作类型（start/pause/resume/complete）
/// * `duration` - 时长（分钟）
/// * `task_title` - 关联的任务标题
pub fn format_pomodoro_response(
    operation: &str,
    duration: Option<i32>,
    task_title: Option<&str>,
) -> String {
    match operation {
        "start" => {
            let duration_text = duration.map(|d| format!("{} 分钟", d)).unwrap_or_else(|| "25 分钟".to_string());

            if let Some(title) = task_title {
                format!(
                    "🍅 番茄钟已开始！专注 {}，任务：{}\n\n💡 保持专注，避免分心。加油！",
                    duration_text, title
                )
            } else {
                format!(
                    "🍅 番茄钟已开始！专注 {}\n\n💡 保持专注，避免分心。加油！",
                    duration_text
                )
            }
        }
        "pause" => {
            "⏸️ 番茄钟已暂停\n\n💡 需要休息的话随时恢复即可。".to_string()
        }
        "resume" => {
            "▶️ 番茄钟已恢复，继续专注！\n\n💪 保持专注，你可以的！".to_string()
        }
        "complete" => {
            "✅ 番茄钟已完成！\n\n🎉 做得很棒！休息 5 分钟，然后继续下一个番茄钟吧~".to_string()
        }
        "cancel" => {
            "🔴 番茄钟已取消\n\n没关系，调整好状态后再开始新的番茄钟。".to_string()
        }
        _ => {
            "🍅 番茄钟操作完成".to_string()
        }
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

    // 模板化响应测试
    #[test]
    fn test_format_task_list_response_empty() {
        let tasks = serde_json::json!([]);
        let response = format_task_list_response(&tasks, "待办");
        assert!(response.contains("暂时没有待办任务"));
    }

    #[test]
    fn test_format_task_list_response_with_tasks() {
        let tasks = serde_json::json!([
            {
                "title": "完成报告",
                "priority": 1,
                "description": "需要在周五前完成"
            },
            {
                "title": "整理文档",
                "priority": 2,
                "description": ""
            }
        ]);
        let response = format_task_list_response(&tasks, "待办");
        assert!(response.contains("2 个"));
        assert!(response.contains("完成报告"));
        assert!(response.contains("高优先级"));
    }

    #[test]
    fn test_format_operation_confirmation_success() {
        let response = format_operation_confirmation("create", "完成报告", true);
        assert!(response.contains("✅"));
        assert!(response.contains("完成报告"));
    }

    #[test]
    fn test_format_operation_confirmation_failure() {
        let response = format_operation_confirmation("delete", "不存在的任务", false);
        assert!(response.contains("❌"));
        assert!(response.contains("找不到"));
    }

    #[test]
    fn test_format_stats_response() {
        let stats = serde_json::json!({
            "total": 10,
            "completed": 7,
            "pending": 3,
            "focus_time": 125,
            "pomodoro_count": 5
        });
        let response = format_stats_response(&stats, "今日");
        assert!(response.contains("已完成：7"));
        assert!(response.contains("待办中：3"));
        assert!(response.contains("番茄钟：5"));
        assert!(response.contains("2 小时 5 分钟"));
    }

    #[test]
    fn test_format_pomodoro_response_start() {
        let response = format_pomodoro_response("start", Some(25), Some("完成报告"));
        assert!(response.contains("🍅"));
        assert!(response.contains("25 分钟"));
        assert!(response.contains("完成报告"));
    }

    #[test]
    fn test_format_pomodoro_response_complete() {
        let response = format_pomodoro_response("complete", None, None);
        assert!(response.contains("✅"));
        assert!(response.contains("休息"));
    }

    // 对话历史摘要测试
    #[test]
    fn test_summarize_conversation_history_within_limit() {
        // 消息数量在限制内，应该原样返回
        let messages = vec![
            ChatMessage::user("你好".to_string()),
            ChatMessage::assistant("你好！有什么可以帮你的吗？".to_string()),
            ChatMessage::user("今天有什么任务？".to_string()),
        ];

        let result = AiChatService::summarize_conversation_history(&messages, 10);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].content, "你好");
    }

    #[test]
    fn test_summarize_conversation_history_exceeds_limit() {
        // 消息数量超过限制，应该压缩
        let mut messages = vec![];
        for i in 0..20 {
            messages.push(ChatMessage::user(format!("消息 {}", i)));
            messages.push(ChatMessage::assistant(format!("回复 {}", i)));
        }

        let result = AiChatService::summarize_conversation_history(&messages, 10);

        // 应该保留最多10条消息（包括摘要）
        assert!(result.len() <= 11); // 1条摘要 + 5条最近消息

        // 第一条应该是摘要
        assert!(result[0].role == "system");
        assert!(result[0].content.contains("[对话摘要]"));
    }

    #[test]
    fn test_extract_task_operation() {
        let content = "帮我创建一个任务：完成项目文档";
        let result = AiChatService::extract_task_operation(content);
        assert!(result.is_some());
        assert!(result.unwrap().contains("创建了任务"));
    }

    #[test]
    fn test_extract_pomodoro_operation() {
        let content = "开始番茄钟，专注工作";
        let result = AiChatService::extract_pomodoro_operation(content);
        assert!(result.is_some());
        assert!(result.unwrap().contains("开始番茄钟"));
    }

    #[test]
    fn test_is_greeting() {
        assert!(AiChatService::is_greeting("你好"));
        assert!(AiChatService::is_greeting("谢谢"));
        assert!(AiChatService::is_greeting("好的"));
        assert!(!AiChatService::is_greeting("创建任务"));
        assert!(!AiChatService::is_greeting("查询今天的待办事项"));
    }

    #[test]
    fn test_extract_query_operation() {
        let content = "查询一下今天的任务列表";
        let result = AiChatService::extract_query_operation(content);
        assert!(result.is_some());
        assert!(result.unwrap().contains("查询了"));
    }

    #[test]
    fn test_extract_user_preference() {
        let content = "我希望每天早上9点开始工作";
        let result = AiChatService::extract_user_preference_from_message(content);
        assert!(result.is_some());
        assert!(result.unwrap().contains("用户偏好"));
    }

    #[test]
    fn test_is_important_message() {
        let important = ChatMessage::user("创建一个新任务".to_string());
        assert!(AiChatService::is_important_message(&important));

        let greeting = ChatMessage::user("你好".to_string());
        assert!(!AiChatService::is_important_message(&greeting));

        let system = ChatMessage::system("系统消息".to_string());
        assert!(AiChatService::is_important_message(&system));
    }

    #[test]
    fn test_extract_conversation_key_points() {
        let messages = vec![
            ChatMessage::user("你好".to_string()),
            ChatMessage::assistant("你好！".to_string()),
            ChatMessage::user("帮我创建一个任务：写周报".to_string()),
            ChatMessage::assistant("好的，已创建".to_string()),
            ChatMessage::user("开始番茄钟".to_string()),
            ChatMessage::user("查询今天的任务".to_string()),
        ];

        let key_points = AiChatService::extract_conversation_key_points(&messages);

        // 应该提取出关键操作，忽略问候语
        assert!(!key_points.is_empty());
        assert!(key_points.iter().any(|p| p.contains("创建了任务") || p.contains("查询了")));
    }
}
