use crate::db::connection::DbConnection;
use crate::services::ai_chat_service::{
    AiChatService, ChatResponse, DashboardStats, OperationContext,
    ParsedFunctionCall, MultiStepResult, safe_truncate,
    TaskContext, PomodoroContext,
};
use crate::services::ai_service::AiService;
use crate::services::context_memory_service::ContextMemoryService;
use crate::services::conversation_context_service::{
    ConversationContextService, EntityType, ConversationTopic
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

// 全局对话上下文（跨请求保持）
static CONVERSATION_CONTEXT: Lazy<Mutex<ConversationContextService>> =
    Lazy::new(|| Mutex::new(ConversationContextService::new()));

/// AI 对话消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// AI 对话会话结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

/// AI 思考状态事件
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingStatus {
    pub status: String,
    pub detail: Option<String>,
    pub step: u32,
    pub total_steps: Option<u32>,
}

/// AI 处理过程日志
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessLog {
    pub id: i64,
    pub request_id: String,
    pub session_id: Option<String>,
    pub step_number: i32,
    pub step_type: String,
    pub user_input: Option<String>,
    pub ai_request: Option<String>,
    pub ai_response: Option<String>,
    pub function_name: Option<String>,
    pub function_args: Option<String>,
    pub function_result: Option<String>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

/// 保存处理过程日志
fn save_process_log(
    conn: &Connection,
    request_id: &str,
    session_id: Option<&str>,
    step_number: i32,
    step_type: &str,
    user_input: Option<&str>,
    ai_request: Option<&str>,
    ai_response: Option<&str>,
    function_name: Option<&str>,
    function_args: Option<&str>,
    function_result: Option<&str>,
    error_message: Option<&str>,
    duration_ms: Option<i64>,
) {
    let result = conn.execute(
        "INSERT INTO ai_chat_process_logs
         (request_id, session_id, step_number, step_type, user_input, ai_request, ai_response,
          function_name, function_args, function_result, error_message, duration_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            request_id,
            session_id,
            step_number,
            step_type,
            user_input,
            ai_request,
            ai_response,
            function_name,
            function_args,
            function_result,
            error_message,
            duration_ms,
        ],
    );

    if let Err(e) = result {
        log::warn!("[AI Chat] 保存处理日志失败: {}", e);
    }
}

/// 发送思考状态事件
fn emit_thinking_status(app: &AppHandle, status: &str, detail: Option<&str>, step: u32, total: Option<u32>) {
    let _ = app.emit("ai-thinking-status", ThinkingStatus {
        status: status.to_string(),
        detail: detail.map(|s| s.to_string()),
        step,
        total_steps: total,
    });
}

/// 判断是否应该跳过 AI 润色，直接返回结果
///
/// 跳过润色的场景：
/// 1. 错误响应（response_type 为 "text" 且无数据）
/// 2. 空查询结果（没有找到数据）
/// 3. 简单的操作确认（创建/删除/更新/完成任务的确认消息）
/// 4. 结果条数少于3条的简单查询
/// 5. 日报/周报（已经由 AI 生成，在前面已处理）
fn should_skip_polishing(response: &ChatResponse) -> bool {
    // 1. 错误响应：response_type 是 "text" 且 data 为 None
    if response.response_type == "text" && response.data.is_none() {
        return true;
    }

    // 2. 内容包含错误标记
    if response.content.contains("❌") || response.content.contains("抱歉") {
        return true;
    }

    // 3. 空查询结果
    if response.response_type == "query_result" && response.content.contains("没有找到") {
        return true;
    }

    // 4. 简单操作确认（已经有友好的确认信息）
    // 这些操作类型通常已经返回了友好的文本，不需要再润色
    let simple_operations = [
        "task_created",      // 创建任务
        "task_updated",      // 更新任务
        "task_completed",    // 完成任务
        "pomodoro_started",  // 启动番茄钟
    ];

    if simple_operations.contains(&response.response_type.as_str()) {
        return true;
    }

    // 5. 简单查询（结果少于3条）
    if let Some(data) = &response.data {
        // 检查任务列表
        if response.response_type == "tasks" {
            if let Some(total) = data.get("total").and_then(|v| v.as_i64()) {
                if total < 3 {
                    return true;
                }
            }
        }

        // 检查查询结果
        if response.response_type == "query_result" {
            if let Some(rows) = data.get("rows").and_then(|v| v.as_array()) {
                if rows.len() < 3 {
                    return true;
                }
            }
        }
    }

    // 其他情况需要润色
    false
}

/// 获取跳过润色的原因（用于日志）
fn get_skip_reason(response: &ChatResponse) -> String {
    if response.response_type == "text" && response.data.is_none() {
        return "错误响应或简单回复".to_string();
    }
    if response.content.contains("❌") || response.content.contains("抱歉") {
        return "包含错误标记".to_string();
    }
    if response.response_type == "query_result" && response.content.contains("没有找到") {
        return "空查询结果".to_string();
    }

    let simple_operations = ["task_created", "task_updated", "task_completed", "pomodoro_started"];
    if simple_operations.contains(&response.response_type.as_str()) {
        return format!("简单操作确认 ({})", response.response_type);
    }

    if let Some(data) = &response.data {
        if response.response_type == "tasks" {
            if let Some(total) = data.get("total").and_then(|v| v.as_i64()) {
                if total < 3 {
                    return format!("简单查询（{}条结果）", total);
                }
            }
        }
        if response.response_type == "query_result" {
            if let Some(rows) = data.get("rows").and_then(|v| v.as_array()) {
                if rows.len() < 3 {
                    return format!("简单查询（{}条结果）", rows.len());
                }
            }
        }
    }

    "未知原因".to_string()
}

/// 为简单操作生成模板化响应，无需 AI 润色
///
/// 对于简单的操作确认，使用预定义的友好模板，避免消耗 AI Token
fn generate_simple_response(response: &ChatResponse) -> Option<String> {
    // 获取数据
    let data = response.data.as_ref()?;

    match response.response_type.as_str() {
        // 任务创建
        "task_created" => {
            let title = data.get("title")?.as_str()?;
            let task_id = data.get("task_id")?.as_i64()?;
            Some(format!("好的，已经帮你创建了任务「{}」（ID: {}）。需要我帮你启动番茄钟吗？", title, task_id))
        }

        // 任务更新
        "task_updated" => {
            let title = data.get("title")?.as_str()?;
            Some(format!("好的，已经更新了任务「{}」。", title))
        }

        // 任务完成
        "task_completed" => {
            let title = data.get("title")?.as_str()?;
            Some(format!("太棒了！任务「{}」已标记为完成。继续加油！", title))
        }

        // 启动番茄钟
        "pomodoro_started" => {
            let duration = data.get("duration")?.as_i64()?;
            let task_title = data.get("task_title").and_then(|v| v.as_str());

            if let Some(title) = task_title {
                Some(format!("好的，已经为任务「{}」启动了 {} 分钟的番茄钟。专注工作吧！", title, duration))
            } else {
                Some(format!("好的，已经启动了 {} 分钟的番茄钟。专注工作吧！", duration))
            }
        }

        // 简单任务查询（1-2条）
        "tasks" => {
            let total = data.get("total")?.as_i64()?;
            if total == 1 {
                let tasks = data.get("tasks")?.as_array()?;
                let task = tasks.get(0)?;
                let title = task.get("title")?.as_str()?;
                let status = task.get("status")?.as_str()?;
                let status_text = match status {
                    "todo" => "待办",
                    "active" => "进行中",
                    "done" => "已完成",
                    _ => status,
                };
                Some(format!("找到了一个任务：「{}」，状态是{}。", title, status_text))
            } else if total == 2 {
                Some(format!("找到了{}个任务。", total))
            } else {
                None
            }
        }

        // 简单查询结果（1-2条）
        "query_result" => {
            let rows = data.get("rows")?.as_array()?;
            if rows.len() == 1 {
                Some("查询到了1条结果。".to_string())
            } else if rows.len() == 2 {
                Some("查询到了2条结果。".to_string())
            } else {
                None
            }
        }

        _ => None,
    }
}

/// AI 对话主命令 - 处理用户消息并返回 AI 响应
#[tauri::command]
pub async fn ai_assistant_chat(
    app: AppHandle,
    db: State<'_, DbConnection>,
    messages: Vec<ChatMessage>,
    #[allow(non_snake_case)]
    sessionId: Option<String>,
    // 操作上下文，用于代词解析（如"它"、"这个任务"）
    context: Option<OperationContext>,
) -> Result<ChatResponse, String> {
    use crate::services::ai_service::ChatMessage as AiChatMessage;

    // 生成唯一请求 ID 用于追踪整个处理过程
    let request_id = Uuid::new_v4().to_string();
    let mut step_number = 0i32;

    // 发送初始状态
    emit_thinking_status(&app, "正在理解你的问题...", None, 1, Some(4));

    // 获取最后一条用户消息（用于日志）
    let user_message = messages
        .iter()
        .rev()
        .find(|msg| msg.role == "user")
        .ok_or_else(|| "没有找到用户消息".to_string())?
        .content
        .clone();

    // 解析用户消息中的代词（失败时使用原始消息）
    let resolved_message = if let Ok(mut ctx) = CONVERSATION_CONTEXT.lock() {
        let _ = ctx.process_message(&user_message); // 忽略处理错误

        // 尝试解析代词
        let mut resolved = user_message.clone();
        let pronouns = ["它", "这个", "那个", "那个任务", "这个任务", "刚才的", "上一个"];
        for pronoun in &pronouns {
            if user_message.contains(pronoun) {
                if let Some(entity) = ctx.resolve_reference(pronoun) {
                    // 如果是任务，将代词替换为任务信息
                    if entity.entity_type == EntityType::Task {
                        if let Some(id) = entity.id {
                            let replacement = format!("任务ID{}({})", id, entity.name);
                            resolved = resolved.replace(pronoun, &replacement);
                            log::info!("[AI Chat] 🔗 代词解析: \"{}\" -> \"{}\"", pronoun, replacement);
                        }
                    }
                    // 如果是番茄钟
                    else if entity.entity_type == EntityType::Pomodoro {
                        if let Some(id) = entity.id {
                            let replacement = format!("番茄钟ID{}", id);
                            resolved = resolved.replace(pronoun, &replacement);
                            log::info!("[AI Chat] 🔗 代词解析: \"{}\" -> \"{}\"", pronoun, replacement);
                        }
                    }
                }
            }
        }
        resolved
    } else {
        log::warn!("[AI Chat] ⚠️ 无法获取对话上下文锁进行代词解析，使用原始消息");
        user_message.clone()
    };

    // 第一步：同步获取配置和系统提示词（在锁内完成）
    let (config, provider_str, model_str, mut system_prompt) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let cfg = AiService::load_config(&conn).map_err(|e| e.to_string())?;
        let provider = cfg.provider.to_string();
        let model = cfg.get_model().to_string();
        let prompt = AiChatService::build_system_prompt(&conn, context.as_ref()).map_err(|e| e.to_string())?;

        // 记录初始请求
        step_number += 1;
        save_process_log(
            &conn,
            &request_id,
            sessionId.as_deref(),
            step_number,
            "init",
            Some(&user_message),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        (cfg, provider, model, prompt)
    }; // 锁在这里释放

    // 获取对话上下文提示并追加到系统提示词（失败不影响主流程）
    if let Ok(ctx) = CONVERSATION_CONTEXT.lock() {
        let context_hint = ctx.generate_context_hint();
        if !context_hint.is_empty() && !context_hint.contains("没有活跃的对话上下文") {
            system_prompt.push_str(&format!("\n\n【对话上下文】\n{}", context_hint));
            log::info!("[AI Chat] 📋 对话上下文已注入到系统提示词");
        }
    } else {
        log::warn!("[AI Chat] ⚠️ 无法获取对话上下文锁，跳过上下文提示注入");
    }

    if !config.enabled {
        return Err("AI 功能未启用".to_string());
    }

    let ai_service = AiService::new(config);

    // 第二步：构建完整的消息历史（包含对话上下文）
    let mut ai_messages = vec![AiChatMessage::system(system_prompt.clone())];

    // 添加对话历史（最多保留最近 10 条消息以控制 token）
    let history_messages: Vec<_> = messages.iter().rev().take(10).collect();
    for msg in history_messages.into_iter().rev() {
        match msg.role.as_str() {
            "user" => ai_messages.push(AiChatMessage::user(msg.content.clone())),
            "assistant" => ai_messages.push(AiChatMessage::assistant(msg.content.clone())),
            "system" => {} // 忽略前端传来的 system 消息
            _ => {}
        }
    }

    // 第三步：异步调用 AI API（不持有锁）
    log::info!("[AI Chat] 📝 用户消息: {}", user_message);
    if resolved_message != user_message {
        log::info!("[AI Chat] 🔗 解析后消息: {}", resolved_message);
    }
    log::info!("[AI Chat] 🤖 调用 AI API (provider: {}, model: {})", provider_str, model_str);

    // 发送思考状态
    emit_thinking_status(&app, "正在思考...", Some(&format!("使用 {} 模型", model_str)), 2, Some(4));

    let start = std::time::Instant::now();
    let ai_response = ai_service.chat(ai_messages.clone())
        .await
        .map_err(|e| e.to_string())?;

    log::info!("[AI Chat] 💬 AI 原始响应 ({}ms): {}",
        start.elapsed().as_millis(),
        safe_truncate(&ai_response, 200));

    // 记录初次 AI 调用结果
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        step_number += 1;
        save_process_log(
            &conn,
            &request_id,
            sessionId.as_deref(),
            step_number,
            "ai_call",
            None,
            Some(&format!("系统提示词: {}字, 消息数: {}", system_prompt.len(), ai_messages.len())),
            Some(&ai_response),
            None,
            None,
            None,
            None,
            Some(start.elapsed().as_millis() as i64),
        );
    }

    // 第四步：解析响应并执行 Function Call（如果需要）
    // 支持单步和多步操作
    let result = if let Ok(parsed_call) = AiChatService::parse_function_call_extended(&ai_response) {
        match parsed_call {
            // 多步操作：按顺序执行所有步骤
            ParsedFunctionCall::Multi(multi_step) => {
                log::info!("[AI Chat] 🔗 执行多步操作，共 {} 步", multi_step.steps.len());
                emit_thinking_status(&app, &format!("正在执行 {} 个连续操作...", multi_step.steps.len()), None, 3, Some(4));

                // 执行多步操作
                let multi_result = {
                    let conn = db.0.lock().map_err(|e| e.to_string())?;
                    AiChatService::execute_multi_step_operations(&conn, &multi_step)
                        .map_err(|e| e.to_string())?
                };

                log::info!("[AI Chat] 📊 多步操作完成: {}/{} 成功", multi_result.success_count, multi_result.total_count);

                // 发送完成状态
                emit_thinking_status(&app, "多步操作完成", Some(&format!("{}/{} 成功", multi_result.success_count, multi_result.total_count)), 4, Some(4));

                // 保存多步操作日志
                {
                    let conn = db.0.lock().map_err(|e| e.to_string())?;
                    step_number += 1;
                    let result_json = serde_json::to_string(&serde_json::json!({
                        "total_steps": multi_result.total_count,
                        "success_count": multi_result.success_count,
                        "all_success": multi_result.all_success,
                    })).unwrap_or_default();

                    save_process_log(
                        &conn,
                        &request_id,
                        sessionId.as_deref(),
                        step_number,
                        "multi_step_exec",
                        None,
                        None,
                        None,
                        Some("multi_step"),
                        Some(&serde_json::to_string(&multi_step.steps).unwrap_or_default()),
                        Some(&result_json),
                        if !multi_result.all_success { Some("部分步骤失败") } else { None },
                        None,
                    );
                }

                // 从最后一个成功步骤提取上下文，用于后续对话引用
                let multi_step_context = multi_result.steps.iter().rev()
                    .find(|step| step.success)
                    .and_then(|step| {
                        step.data.as_ref().map(|data| {
                            let mut ctx = OperationContext::default();
                            // 提取任务上下文
                            if let (Some(task_id), Some(task_title)) = (
                                data.get("task_id").and_then(|v| v.as_i64()),
                                data.get("task_title").and_then(|v| v.as_str())
                            ) {
                                ctx.last_task = Some(TaskContext {
                                    id: task_id,
                                    title: task_title.to_string(),
                                });
                            }
                            // 提取番茄钟上下文
                            if let Some(pomodoro_id) = data.get("pomodoro_id").and_then(|v| v.as_i64()) {
                                ctx.last_pomodoro = Some(PomodoroContext {
                                    id: pomodoro_id,
                                    task_id: data.get("task_id").and_then(|v| v.as_i64()),
                                    task_title: data.get("task_title").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                });
                            }
                            ctx
                        })
                    });

                // 保存多步操作的上下文记忆
                // 遍历所有成功的步骤，提取并保存记忆
                {
                    let conn = db.0.lock().map_err(|e| e.to_string())?;
                    for step_result in &multi_result.steps {
                        if step_result.success {
                            // 构建临时 ChatResponse 用于提取上下文
                            let step_response = ChatResponse {
                                content: step_result.content.clone(),
                                response_type: step_result.function.clone(),
                                data: step_result.data.clone(),
                                context: None,
                            };

                            let contexts = AiChatService::extract_context_from_response(
                                &user_message,
                                Some(&step_result.function),
                                &step_response,
                            );

                            for (context_type, key, value, importance) in &contexts {
                                if let Err(e) = ContextMemoryService::save_context(
                                    &conn,
                                    context_type,
                                    key,
                                    value,
                                    *importance,
                                ) {
                                    log::warn!("[AI Chat] 多步操作保存上下文记忆失败: {} - {}", key, e);
                                } else {
                                    log::info!("[AI Chat] 💾 多步操作保存上下文记忆: {} (type={}, importance={})", key, context_type, importance);
                                }
                            }
                        }
                    }
                }

                // 记录多步操作结果到对话上下文（失败不影响响应）
                if let Ok(mut ctx) = CONVERSATION_CONTEXT.lock() {
                    for step_result in &multi_result.steps {
                        if step_result.success {
                            match step_result.function.as_str() {
                                "create_task" => {
                                    if let Some(data) = &step_result.data {
                                        if let Some(task) = data.get("task") {
                                            let task_id = task.get("id").and_then(|v| v.as_i64());
                                            let task_title = task.get("title").and_then(|v| v.as_str()).unwrap_or("未命名任务");
                                            ctx.add_operation_result(EntityType::Task, task_id, task_title.to_string(), None);
                                        } else if let Some(task_id) = data.get("task_id").and_then(|v| v.as_i64()) {
                                            let task_title = data.get("title").and_then(|v| v.as_str()).unwrap_or("未命名任务");
                                            ctx.add_operation_result(EntityType::Task, Some(task_id), task_title.to_string(), None);
                                        }
                                    }
                                    ctx.context_mut().update_topic(ConversationTopic::TaskManagement);
                                }
                                "start_pomodoro" => {
                                    if let Some(data) = &step_result.data {
                                        if let Some(session) = data.get("session") {
                                            let session_id = session.get("id").and_then(|v| v.as_i64());
                                            ctx.add_operation_result(EntityType::Pomodoro, session_id, "番茄钟".to_string(), None);
                                        } else if let Some(pomodoro_id) = data.get("pomodoro_id").and_then(|v| v.as_i64()) {
                                            ctx.add_operation_result(EntityType::Pomodoro, Some(pomodoro_id), "番茄钟".to_string(), None);
                                        }
                                    }
                                    ctx.context_mut().update_topic(ConversationTopic::PomodoroFocus);
                                }
                                _ => {}
                            }
                        }
                    }
                    log::info!("[AI Chat] 📝 多步操作结果已记录到对话上下文");
                } else {
                    log::warn!("[AI Chat] ⚠️ 无法获取对话上下文锁，跳过多步操作上下文记录");
                }

                // 构建响应
                let summary = multi_result.summary.clone();
                ChatResponse {
                    content: summary,
                    response_type: "multi_step_result".to_string(),
                    data: Some(serde_json::to_value(&multi_result).unwrap_or_default()),
                    context: multi_step_context,
                }
            }

            // 单步操作：使用智能体模式，支持自动纠错和重试
            ParsedFunctionCall::Single(mut current_function_call) => {
        const MAX_RETRIES: u32 = 10;
        let mut retry_count = 0u32;
        let mut current_ai_response = ai_response.clone();
        let mut retry_messages = ai_messages.clone();

        // 智能体循环：执行 -> 检查结果 -> 如需重试则让 AI 纠正 -> 再执行
        let function_result = loop {
            // 记录当前 Function Call
            log::info!("[AI Chat] 🔧 Function Call (尝试 {}/{}): ", retry_count + 1, MAX_RETRIES);
            log::info!("[AI Chat]   - 函数名: {}", current_function_call.name);
            log::info!("[AI Chat]   - 参数: {}", current_function_call.arguments);

            // 发送执行状态
            let function_desc = match current_function_call.name.as_str() {
                "query_data" => "正在查询数据...",
                "create_task" => "正在创建任务...",
                "update_task" => "正在更新任务...",
                "delete_task" => "正在删除任务...",
                "complete_task" => "正在完成任务...",
                "get_tasks" => "正在获取任务列表...",
                "start_pomodoro" => "正在启动番茄钟...",
                "generate_daily_report" => "正在生成日报...",
                "generate_weekly_report" => "正在生成周报...",
                "analyze_efficiency" => "正在分析效率...",
                _ => "正在执行操作...",
            };
            emit_thinking_status(&app, function_desc, None, 3, Some(4));

            // 执行 Function Call
            let func_start = std::time::Instant::now();
            let exec_result = {
                let conn = db.0.lock().map_err(|e| e.to_string())?;
                AiChatService::execute_function_sync(&conn, &current_function_call)
                    .map_err(|e| e.to_string())?
            };
            let func_duration = func_start.elapsed().as_millis() as i64;

            // 记录执行结果
            log::info!("[AI Chat] 📊 Function 执行结果:");
            log::info!("[AI Chat]   - 响应类型: {}", exec_result.response_type);
            log::info!("[AI Chat]   - 内容预览: {}", safe_truncate(&exec_result.content, 100));

            // 保存 Function Call 执行日志
            {
                let conn = db.0.lock().map_err(|e| e.to_string())?;
                step_number += 1;
                let result_json = serde_json::to_string(&serde_json::json!({
                    "response_type": &exec_result.response_type,
                    "content": safe_truncate(&exec_result.content, 500),
                    "has_data": exec_result.data.is_some(),
                })).unwrap_or_default();

                save_process_log(
                    &conn,
                    &request_id,
                    sessionId.as_deref(),
                    step_number,
                    "function_exec",
                    None,
                    None,
                    None,
                    Some(&current_function_call.name),
                    Some(&current_function_call.arguments),
                    Some(&result_json),
                    if exec_result.response_type == "retry_sql" { Some("需要重试") } else { None },
                    Some(func_duration),
                );
            }

            // 检查是否需要重试（response_type 为 "retry_sql"）
            if exec_result.response_type == "retry_sql" {
                retry_count += 1;

                if retry_count >= MAX_RETRIES {
                    log::warn!("[AI Chat] ⚠️ 达到最大重试次数 {}，返回友好错误", MAX_RETRIES);
                    break ChatResponse {
                        content: "抱歉，我尝试了几次都没能正确理解你的问题。能换个方式描述一下吗？比如告诉我你想查询什么具体信息。".to_string(),
                        response_type: "text".to_string(),
                        data: None,
                        context: None,
                    };
                }

                // 提取错误信息，构建纠错提示
                let error_data = exec_result.data.as_ref();
                let error_msg = error_data
                    .and_then(|d| d["error_message"].as_str())
                    .unwrap_or("未知错误");
                let failed_sql = error_data
                    .and_then(|d| d["failed_sql"].as_str())
                    .unwrap_or("");
                let description = error_data
                    .and_then(|d| d["description"].as_str())
                    .unwrap_or("数据查询");

                log::info!("[AI Chat] 🔄 SQL 执行失败，让 AI 自动纠正 (重试 {}/{})", retry_count, MAX_RETRIES);
                log::info!("[AI Chat]   - 错误信息: {}", error_msg);
                log::info!("[AI Chat]   - 失败 SQL: {}", failed_sql);

                // 发送重试状态
                emit_thinking_status(
                    &app,
                    &format!("正在自动纠正... ({}/{})", retry_count, MAX_RETRIES),
                    Some("检测到问题，正在重新生成"),
                    3,
                    Some(4)
                );

                // 构建纠错提示
                let correction_prompt = format!(
                    r#"你之前生成的 SQL 执行失败了，请分析错误并重新生成正确的 SQL。

【错误信息】
{}

【失败的 SQL】
{}

【原始查询意图】
{}

【修正要求】
1. 分析错误原因
2. 必须使用 SELECT 语句（query_data 只支持 SELECT 查询！）
3. 参考数据库表结构，使用正确的表名和列名
4. 重新生成正确的 query_data Function Call
5. 只返回纯 JSON，不要任何解释文字

【数据库表结构提醒】
- tasks 表：id, title, description, status(todo/active/done), priority(1/2/3), created_at, completed_at, due_date
- pomodoro_sessions 表：id, task_id, duration_minutes, status, started_at, completed_at
- tags 表：id, name, color

【SQL 语法提醒】
- 必须以 SELECT 开头
- 日期函数：date('now','localtime'), datetime('now','localtime')
- 字符串匹配：LIKE '%关键词%'
- 今天的任务：WHERE date(created_at) = date('now','localtime')

【正确示例】
{{"function":"query_data","arguments":{{"sql":"SELECT * FROM tasks WHERE status IN ('todo','active')","description":"查询待办任务"}}}}"#,
                    error_msg,
                    failed_sql,
                    description
                );

                // 添加纠错对话到消息历史
                retry_messages.push(AiChatMessage::assistant(current_ai_response.clone()));
                retry_messages.push(AiChatMessage::user(correction_prompt.clone()));

                // 让 AI 重新生成
                let retry_start = std::time::Instant::now();
                let new_response = ai_service.chat(retry_messages.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                let retry_duration = retry_start.elapsed().as_millis() as i64;

                log::info!("[AI Chat] 🤖 AI 纠正后响应: {}", safe_truncate(&new_response, 100));

                // 保存重试 AI 调用日志
                {
                    let conn = db.0.lock().map_err(|e| e.to_string())?;
                    step_number += 1;
                    save_process_log(
                        &conn,
                        &request_id,
                        sessionId.as_deref(),
                        step_number,
                        &format!("retry_{}", retry_count),
                        None,
                        Some(&safe_truncate(&correction_prompt, 500)),
                        Some(&new_response),
                        None,
                        None,
                        None,
                        Some(&format!("错误: {}, 失败SQL: {}", error_msg, safe_truncate(failed_sql, 100))),
                        Some(retry_duration),
                    );
                }

                // 解析新的 Function Call
                match AiChatService::parse_function_call(&new_response) {
                    Ok(new_call) => {
                        current_function_call = new_call;
                        current_ai_response = new_response;
                        // 继续循环，执行新的 Function Call
                        continue;
                    }
                    Err(_) => {
                        // AI 没有返回有效的 Function Call，可能是放弃了
                        log::warn!("[AI Chat] ⚠️ AI 未能生成有效的纠正方案");
                        break ChatResponse {
                            content: new_response,
                            response_type: "text".to_string(),
                            data: None,
                            context: None,
                        };
                    }
                }
            } else {
                // 执行成功，保存上下文记忆
                // 这是关键的集成点：让 AI 真正能够学习和记忆
                if exec_result.response_type != "error" && exec_result.response_type != "text" {
                    // 提取上下文信息
                    let contexts = AiChatService::extract_context_from_response(
                        &user_message,
                        Some(&current_function_call.name),
                        &exec_result,
                    );

                    // 保存到数据库
                    if !contexts.is_empty() {
                        let conn = db.0.lock().map_err(|e| e.to_string())?;
                        for (context_type, key, value, importance) in &contexts {
                            if let Err(e) = ContextMemoryService::save_context(
                                &conn,
                                context_type,
                                key,
                                value,
                                *importance,
                            ) {
                                log::warn!("[AI Chat] 保存上下文记忆失败: {} - {}", key, e);
                            } else {
                                log::info!("[AI Chat] 💾 保存上下文记忆: {} (type={}, importance={})", key, context_type, importance);
                            }
                        }
                        log::info!("[AI Chat] ✅ 已保存 {} 条上下文记忆", contexts.len());
                    }
                }

                // 记录操作结果到对话上下文（失败不影响响应）
                if let Ok(mut ctx) = CONVERSATION_CONTEXT.lock() {
                    match current_function_call.name.as_str() {
                        "create_task" => {
                            if let Some(task_data) = exec_result.data.as_ref() {
                                if let Some(task) = task_data.get("task") {
                                    let task_id = task.get("id").and_then(|v| v.as_i64());
                                    let task_title = task.get("title").and_then(|v| v.as_str()).unwrap_or("未命名任务");
                                    ctx.add_operation_result(EntityType::Task, task_id, task_title.to_string(), None);
                                    log::info!("[AI Chat] 📝 记录任务到对话上下文: {} (ID: {:?})", task_title, task_id);
                                }
                            } else if let Some(task_id) = exec_result.data.as_ref().and_then(|d| d.get("task_id")).and_then(|v| v.as_i64()) {
                                let task_title = exec_result.data.as_ref().and_then(|d| d.get("title")).and_then(|v| v.as_str()).unwrap_or("未命名任务");
                                ctx.add_operation_result(EntityType::Task, Some(task_id), task_title.to_string(), None);
                                log::info!("[AI Chat] 📝 记录任务到对话上下文: {} (ID: {})", task_title, task_id);
                            }
                            ctx.context_mut().update_topic(ConversationTopic::TaskManagement);
                        }
                        "start_pomodoro" => {
                            if let Some(data) = exec_result.data.as_ref() {
                                if let Some(session) = data.get("session") {
                                    let session_id = session.get("id").and_then(|v| v.as_i64());
                                    ctx.add_operation_result(EntityType::Pomodoro, session_id, "番茄钟".to_string(), None);
                                    log::info!("[AI Chat] 🍅 记录番茄钟到对话上下文 (ID: {:?})", session_id);
                                } else if let Some(pomodoro_id) = data.get("pomodoro_id").and_then(|v| v.as_i64()) {
                                    ctx.add_operation_result(EntityType::Pomodoro, Some(pomodoro_id), "番茄钟".to_string(), None);
                                    log::info!("[AI Chat] 🍅 记录番茄钟到对话上下文 (ID: {})", pomodoro_id);
                                }
                            }
                            ctx.context_mut().update_topic(ConversationTopic::PomodoroFocus);
                        }
                        "get_tasks" | "update_task" | "complete_task" | "delete_task" => {
                            // 更新任务信息到上下文
                            if let Some(data) = exec_result.data.as_ref() {
                                if let Some(task) = data.get("task") {
                                    let task_id = task.get("id").and_then(|v| v.as_i64());
                                    let task_title = task.get("title").and_then(|v| v.as_str()).unwrap_or("未命名任务");
                                    ctx.add_operation_result(EntityType::Task, task_id, task_title.to_string(), None);
                                }
                            }
                            ctx.context_mut().update_topic(ConversationTopic::TaskManagement);
                        }
                        "query_data" | "search_sql" => {
                            ctx.context_mut().update_topic(ConversationTopic::SqlSearch);
                        }
                        "generate_daily_report" | "generate_weekly_report" => {
                            ctx.context_mut().update_topic(ConversationTopic::ReportGeneration);
                        }
                        _ => {}
                    }
                } else {
                    log::warn!("[AI Chat] ⚠️ 无法获取对话上下文锁，跳过上下文更新");
                }

                // 跳出循环
                break exec_result;
            }
        };

        // 特殊处理：如果需要 AI 生成（如日报生成）
        if function_result.response_type == "need_ai_generation" {
            log::info!("[AI Chat] 🤖 检测到需要 AI 生成内容");
            emit_thinking_status(&app, "正在生成智能日报...", None, 4, Some(4));

            // function_result.content 包含了 AI 提示词
            let generation_prompt = function_result.content.clone();

            // 直接使用提示词调用 AI（不带对话历史，只用提示词生成）
            let generated_content = ai_service.chat(vec![
                AiChatMessage::user(generation_prompt)
            ])
                .await
                .map_err(|e| e.to_string())?;

            log::info!("[AI Chat] ✅ AI 生成完成: {}", safe_truncate(&generated_content, 150));

            // 保存到工作日志
            if let Some(data) = &function_result.data {
                if let Some(date_str) = data["date"].as_str() {
                    use crate::services::work_log_service::WorkLogService;

                    let conn = db.0.lock().map_err(|e| e.to_string())?;
                    if let Err(e) = WorkLogService::save_work_log(&conn, date_str, "daily", &generated_content, true) {
                        log::warn!("[AI Chat] 保存日报到工作日志失败: {}", e);
                    } else {
                        log::info!("[AI Chat] ✅ AI 生成的日报已保存到工作日志，日期: {}", date_str);
                    }
                }
            }

            ChatResponse {
                content: generated_content,
                response_type: "daily_report".to_string(),
                data: function_result.data,
                context: function_result.context,
            }
        } else {
            // 判断是否需要跳过润色：错误结果、简单操作或少量数据直接返回
            let should_skip = should_skip_polishing(&function_result);

            if should_skip {
                log::info!("[AI Chat] ⏭️ 跳过润色：{}", get_skip_reason(&function_result));
                emit_thinking_status(&app, "完成", None, 4, Some(4));

                // 尝试生成简单模板响应（如果适用）
                if let Some(simple_response) = generate_simple_response(&function_result) {
                    log::info!("[AI Chat] 📝 使用模板响应");
                    ChatResponse {
                        content: simple_response,
                        response_type: function_result.response_type.clone(),
                        data: function_result.data.clone(),
                        context: function_result.context.clone(),
                    }
                } else {
                    function_result
                }
            } else {
            // 发送润色状态
            emit_thinking_status(&app, "正在整理回复...", None, 4, Some(4));

            // 将 Function Call 结果交给 AI 进行自然语言润色
            let data_summary = if let Some(ref data) = function_result.data {
                serde_json::to_string_pretty(data).unwrap_or_else(|_| "无数据".to_string())
            } else {
                "无数据".to_string()
            };

            let polishing_prompt = format!(
                r#"【查询结果数据】
{}

【原始数据 JSON】
{}

【你的任务】
请基于上述查询结果，用自然友好的语言回答用户的问题。

【严格要求】
1. 必须且只能使用上述查询结果中的真实数据
2. 绝对禁止编造任何不在查询结果中的信息
3. 不要提及 SQL、数据库、查询等技术词汇
4. 如果查询结果为空或"没有找到"，直接告诉用户没有这条记录
5. 用口语化的方式回答，像朋友聊天一样"#,
                function_result.content,
                data_summary
            );

            log::info!("[AI Chat] 🎨 润色提示词: {}", safe_truncate(&polishing_prompt, 150));

            // 添加 function 结果到消息历史，让 AI 生成自然回复
            ai_messages.push(AiChatMessage::assistant(ai_response.clone()));
            ai_messages.push(AiChatMessage::user(polishing_prompt));

            let polished_response = ai_service.chat(ai_messages)
                .await
                .map_err(|e| e.to_string())?;

            log::info!("[AI Chat] ✅ 润色后回复: {}", safe_truncate(&polished_response, 150));

            ChatResponse {
                content: polished_response,
                response_type: function_result.response_type,
                data: function_result.data,
                context: function_result.context,  // 保留操作上下文
            }
            }
        }
            } // 结束 ParsedFunctionCall::Single 分支
        } // 结束 match parsed_call
    } else {
        // 普通文本回复
        emit_thinking_status(&app, "完成", None, 4, Some(4));
        ChatResponse {
            content: ai_response,
            response_type: "text".to_string(),
            data: None,
            context: None,
        }
    };

    let duration_ms = start.elapsed().as_millis() as i64;

    // 第五步：记录日志（失败不影响响应）
    if let Ok(conn) = db.0.lock() {
        let _ = AiService::save_log(
            &conn,
            "ai_chat",
            "assistant_chat",
            &provider_str,
            Some(&model_str),
            &user_message,
            Some(&result.content),
            None,
            Some(duration_ms),
            "success",
            None,
        );
    } else {
        log::warn!("[AI Chat] ⚠️ 无法获取数据库锁来记录日志，跳过");
    }

    Ok(result)
}

/// 获取仪表盘统计数据
#[tauri::command]
pub async fn ai_get_dashboard_stats(db: State<'_, DbConnection>) -> Result<DashboardStats, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    AiChatService::get_dashboard_stats(&conn).map_err(|e| e.to_string())
}

/// 保存会话到数据库
#[tauri::command]
pub async fn ai_save_chat_session(
    db: State<'_, DbConnection>,
    session: ChatSession,
) -> Result<(), String> {
    log::info!("[AI Chat] 📁 保存会话: id={}, title={}", session.id, session.title);

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 使用 INSERT ... ON CONFLICT ... DO UPDATE 而不是 INSERT OR REPLACE
    // 因为 INSERT OR REPLACE 会先删除再插入，触发 ON DELETE CASCADE 删除所有消息！
    conn.execute(
        "INSERT INTO ai_chat_sessions (id, title, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET
           title = excluded.title,
           updated_at = excluded.updated_at",
        rusqlite::params![
            &session.id,
            &session.title,
            &session.created_at,
            &session.updated_at
        ],
    )
    .map_err(|e| format!("保存会话失败: {}", e))?;

    log::info!("[AI Chat] ✅ 会话保存成功: {}", session.id);
    Ok(())
}

/// 获取会话列表
#[tauri::command]
pub async fn ai_get_chat_sessions(
    db: State<'_, DbConnection>,
) -> Result<Vec<ChatSession>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, created_at, updated_at FROM ai_chat_sessions
             ORDER BY updated_at DESC LIMIT 50",
        )
        .map_err(|e| format!("准备查询失败: {}", e))?;

    let sessions = stmt
        .query_map([], |row| {
            Ok(ChatSession {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| format!("查询会话失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("收集会话数据失败: {}", e))?;

    Ok(sessions)
}

/// 删除会话
#[tauri::command]
pub async fn ai_delete_chat_session(
    db: State<'_, DbConnection>,
    #[allow(non_snake_case)]
    sessionId: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 删除会话
    conn.execute(
        "DELETE FROM ai_chat_sessions WHERE id = ?1",
        rusqlite::params![&sessionId],
    )
    .map_err(|e| format!("删除会话失败: {}", e))?;

    // 同时删除该会话的消息
    conn.execute(
        "DELETE FROM ai_chat_messages WHERE session_id = ?1",
        rusqlite::params![&sessionId],
    )
    .map_err(|e| format!("删除会话消息失败: {}", e))?;

    Ok(())
}

/// 获取会话的消息列表
#[tauri::command]
pub async fn ai_get_session_messages(
    db: State<'_, DbConnection>,
    #[allow(non_snake_case)]
    sessionId: String,
) -> Result<Vec<ChatMessage>, String> {
    log::info!("[AI Chat] ========== 加载会话消息 ==========");
    log::info!("[AI Chat] 📥 sessionId: {}", sessionId);

    let conn = db.0.lock().map_err(|e| {
        log::error!("[AI Chat] ❌ 获取数据库连接失败: {}", e);
        e.to_string()
    })?;

    // 先检查会话是否存在
    let session_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM ai_chat_sessions WHERE id = ?1",
            [&sessionId],
            |row| row.get(0),
        )
        .unwrap_or(false);
    log::info!("[AI Chat] 📥 会话存在: {}", session_exists);

    // 统计该会话的消息数量
    let msg_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ai_chat_messages WHERE session_id = ?1",
            [&sessionId],
            |row| row.get(0),
        )
        .unwrap_or(0);
    log::info!("[AI Chat] 📥 数据库中消息数量: {}", msg_count);

    let mut stmt = conn
        .prepare(
            "SELECT role, content FROM ai_chat_messages
             WHERE session_id = ?1
             ORDER BY created_at ASC",
        )
        .map_err(|e| {
            log::error!("[AI Chat] ❌ 准备查询失败: {}", e);
            format!("准备查询失败: {}", e)
        })?;

    let messages = stmt
        .query_map([&sessionId], |row| {
            Ok(ChatMessage {
                role: row.get(0)?,
                content: row.get(1)?,
            })
        })
        .map_err(|e| {
            log::error!("[AI Chat] ❌ 查询消息失败: {}", e);
            format!("查询消息失败: {}", e)
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            log::error!("[AI Chat] ❌ 收集消息数据失败: {}", e);
            format!("收集消息数据失败: {}", e)
        })?;

    log::info!("[AI Chat] ✅ 成功加载 {} 条消息", messages.len());
    for (i, msg) in messages.iter().enumerate() {
        log::info!("[AI Chat]   {}. [{}] {}", i + 1, msg.role, safe_truncate(&msg.content, 50));
    }
    log::info!("[AI Chat] ========== 加载完成 ==========");

    Ok(messages)
}

/// 保存单条消息到会话
#[tauri::command]
pub async fn ai_save_chat_message(
    db: State<'_, DbConnection>,
    #[allow(non_snake_case)]
    sessionId: String,
    role: String,
    content: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now_str = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO ai_chat_messages (session_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![&sessionId, &role, &content, &now_str],
    ).map_err(|e| format!("保存消息失败: {}", e))?;

    log::info!("[AI Chat] 💾 消息已保存: session={}, role={}", sessionId, role);
    Ok(())
}

/// 清空会话的所有消息
#[tauri::command]
pub async fn ai_clear_session_messages(
    db: State<'_, DbConnection>,
    #[allow(non_snake_case)]
    sessionId: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM ai_chat_messages WHERE session_id = ?1",
        rusqlite::params![&sessionId],
    )
    .map_err(|e| format!("清空会话消息失败: {}", e))?;

    Ok(())
}

/// 获取 AI 处理过程日志（用于调试和追踪）
#[tauri::command]
pub async fn ai_get_process_logs(
    db: State<'_, DbConnection>,
    #[allow(non_snake_case)]
    requestId: Option<String>,
    #[allow(non_snake_case)]
    sessionId: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<ProcessLog>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let limit = limit.unwrap_or(100);

    // 辅助函数：从行映射到 ProcessLog
    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ProcessLog> {
        Ok(ProcessLog {
            id: row.get(0)?,
            request_id: row.get(1)?,
            session_id: row.get(2)?,
            step_number: row.get(3)?,
            step_type: row.get(4)?,
            user_input: row.get(5)?,
            ai_request: row.get(6)?,
            ai_response: row.get(7)?,
            function_name: row.get(8)?,
            function_args: row.get(9)?,
            function_result: row.get(10)?,
            error_message: row.get(11)?,
            duration_ms: row.get(12)?,
            created_at: row.get(13)?,
        })
    }

    if let Some(req_id) = requestId {
        // 按请求 ID 查询
        let mut stmt = conn
            .prepare(
                "SELECT id, request_id, session_id, step_number, step_type,
                        user_input, ai_request, ai_response, function_name,
                        function_args, function_result, error_message, duration_ms, created_at
                 FROM ai_chat_process_logs
                 WHERE request_id = ?1
                 ORDER BY step_number ASC",
            )
            .map_err(|e| format!("准备查询失败: {}", e))?;

        let logs: Vec<ProcessLog> = stmt
            .query_map([&req_id], map_row)
            .map_err(|e| format!("查询失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("收集数据失败: {}", e))?;

        Ok(logs)
    } else if let Some(sess_id) = sessionId {
        // 按会话 ID 查询
        let mut stmt = conn
            .prepare(
                "SELECT id, request_id, session_id, step_number, step_type,
                        user_input, ai_request, ai_response, function_name,
                        function_args, function_result, error_message, duration_ms, created_at
                 FROM ai_chat_process_logs
                 WHERE session_id = ?1
                 ORDER BY created_at DESC, step_number ASC
                 LIMIT ?2",
            )
            .map_err(|e| format!("准备查询失败: {}", e))?;

        let logs: Vec<ProcessLog> = stmt
            .query_map(rusqlite::params![&sess_id, limit], map_row)
            .map_err(|e| format!("查询失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("收集数据失败: {}", e))?;

        Ok(logs)
    } else {
        // 查询最近的日志
        let mut stmt = conn
            .prepare(
                "SELECT id, request_id, session_id, step_number, step_type,
                        user_input, ai_request, ai_response, function_name,
                        function_args, function_result, error_message, duration_ms, created_at
                 FROM ai_chat_process_logs
                 ORDER BY created_at DESC, step_number ASC
                 LIMIT ?1",
            )
            .map_err(|e| format!("准备查询失败: {}", e))?;

        let logs: Vec<ProcessLog> = stmt
            .query_map([limit], map_row)
            .map_err(|e| format!("查询失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("收集数据失败: {}", e))?;

        Ok(logs)
    }
}

/// 清空 AI 处理过程日志
#[tauri::command]
pub async fn ai_clear_process_logs(
    db: State<'_, DbConnection>,
    #[allow(non_snake_case)]
    sessionId: Option<String>,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let deleted = if let Some(sess_id) = sessionId {
        conn.execute(
            "DELETE FROM ai_chat_process_logs WHERE session_id = ?1",
            rusqlite::params![&sess_id],
        )
        .map_err(|e| format!("清空日志失败: {}", e))?
    } else {
        conn.execute("DELETE FROM ai_chat_process_logs", [])
            .map_err(|e| format!("清空日志失败: {}", e))?
    };

    Ok(deleted as i64)
}

/// 重置对话上下文
#[tauri::command]
pub async fn ai_reset_conversation_context() -> Result<String, String> {
    let mut ctx = CONVERSATION_CONTEXT.lock().map_err(|e| e.to_string())?;
    ctx.reset();
    log::info!("[AI Chat] 🔄 对话上下文已重置");
    Ok("对话上下文已重置".to_string())
}

/// 获取对话上下文统计信息
#[tauri::command]
pub async fn ai_get_conversation_stats() -> Result<serde_json::Value, String> {
    let ctx = CONVERSATION_CONTEXT.lock().map_err(|e| e.to_string())?;
    let stats = ctx.get_stats();
    serde_json::to_value(stats).map_err(|e| e.to_string())
}
