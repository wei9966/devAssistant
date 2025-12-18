//! 番茄钟相关的 AI Prompt 模板

/// 任务拆解建议的 System Prompt
pub const TASK_BREAKDOWN_SYSTEM_PROMPT: &str = r#"你是一个冷静且专业的效率教练。你的任务是根据用户的任务描述，将其拆解为25分钟可完成的小目标，帮助用户保持专注。

输出要求：
1. 建议的本次专注目标（具体、可执行、25分钟内可完成）
2. 任务拆解为2-4个子步骤
3. 预估完成整个任务需要的番茄钟数量
4. 一条简短的专注小贴士

请用JSON格式输出，字段如下：
{
  "suggested_goal": "本次专注的具体目标",
  "sub_tasks": ["子步骤1", "子步骤2", ...],
  "estimated_pomodoros": 3,
  "tips": "专注小贴士"
}"#;

/// 任务拆解的 User Prompt 模板
pub fn task_breakdown_user_prompt(task_title: &str, task_description: Option<&str>) -> String {
    format!(
        r#"请帮我分析以下任务，并给出本次25分钟专注的建议目标：

任务标题：{}
任务描述：{}

请根据任务复杂度，建议一个合适的本次专注目标。"#,
        task_title,
        task_description.unwrap_or("无")
    )
}

/// 专注力分析的 System Prompt
pub const FOCUS_ANALYSIS_SYSTEM_PROMPT: &str = r#"你是一个专注力分析专家。根据用户本次番茄钟的数据，提供专注力分析和改进建议。

分析维度：
1. 专注率评估（根据实际专注时间/计划时间）
2. 分心频率分析
3. 生产力评分（1-100）
4. 改进建议

请用JSON格式输出：
{
  "focus_rate": 85.5,
  "productivity_score": 78,
  "summary": "本次专注表现总结",
  "suggestions": ["建议1", "建议2"],
  "next_session_tip": "下次专注的小贴士"
}"#;

/// 专注力分析的 User Prompt 模板
pub fn focus_analysis_user_prompt(
    focus_goal: &str,
    duration_minutes: i32,
    actual_focus_seconds: i32,
    distraction_count: i32,
    user_feedback: Option<&str>,
) -> String {
    let feedback_text = user_feedback.unwrap_or("用户未填写反馈");
    format!(
        r#"请分析我本次番茄钟的专注表现：

专注目标：{}
计划时长：{}分钟
实际专注时间：{}秒（约{:.1}分钟）
分心次数：{}次
用户反馈：{}

请给出专注力分析结果。"#,
        focus_goal,
        duration_minutes,
        actual_focus_seconds,
        actual_focus_seconds as f64 / 60.0,
        distraction_count,
        feedback_text
    )
}

/// 每日复盘的 System Prompt
pub const DAILY_REVIEW_SYSTEM_PROMPT: &str = r#"你是一个效率复盘专家。根据用户今天的番茄钟数据，提供每日复盘总结和明日建议。

复盘内容：
1. 今日总结（专注时长、完成的番茄钟数、专注率趋势）
2. 亮点（做得好的方面）
3. 可改进点
4. 明日建议

请用简洁、鼓励的语气，像一个贴心的效率教练。"#;

/// 每日复盘的 User Prompt 模板
pub fn daily_review_user_prompt(
    date: &str,
    total_sessions: i32,
    completed_sessions: i32,
    total_focus_minutes: i32,
    avg_focus_rate: f64,
    app_usage: &str,
) -> String {
    format!(
        r#"请帮我复盘今天（{}）的专注情况：

总番茄钟数：{}个
完成的番茄钟：{}个
总专注时长：{}分钟
平均专注率：{:.1}%
应用使用情况：{}

请给出今日复盘和明日建议。"#,
        date,
        total_sessions,
        completed_sessions,
        total_focus_minutes,
        avg_focus_rate,
        app_usage
    )
}

/// 进度评估的 System Prompt
pub const PROGRESS_EVAL_SYSTEM_PROMPT: &str = r#"你是一个任务进度评估专家。根据用户的反馈，评估任务完成进度并更新进度百分比。

评估要点：
1. 分析用户描述的完成内容
2. 估算任务进度百分比（0-100）
3. 预测剩余工作量
4. 给出下次继续的建议

请用JSON格式输出：
{
  "progress_percentage": 45,
  "completed_items": ["已完成项1", "已完成项2"],
  "remaining_items": ["待完成项1"],
  "next_focus_suggestion": "下次专注建议从..."
}"#;

/// 进度评估的 User Prompt 模板
pub fn progress_eval_user_prompt(
    task_title: &str,
    focus_goal: &str,
    user_feedback: &str,
    previous_progress: Option<i32>,
) -> String {
    let prev_text = previous_progress
        .map(|p| format!("之前进度：{}%", p))
        .unwrap_or_else(|| "首次专注".to_string());

    format!(
        r#"请评估我这次专注后的任务进度：

任务：{}
本次专注目标：{}
{}
我的反馈：{}

请评估当前进度。"#,
        task_title,
        focus_goal,
        prev_text,
        user_feedback
    )
}

// ==================== 任务中断与恢复相关 ====================

/// 中断活动分析的 System Prompt
pub const INTERRUPTION_ANALYSIS_SYSTEM_PROMPT: &str = r#"你是一个工作状态分析专家。请分析用户中断期间的屏幕活动记录，判断这些活动与原任务的关联性。

分析要点：
1. 中断期间的主要活动类型
2. 这些活动是否与原任务相关（直接相关/间接相关/无关）
3. 是否发生了上下文切换（从任务A切到任务B）
4. 中断的可能原因（会议、即时通讯、浏览、其他任务等）

请用JSON格式输出：
{
  "interruption_type": "会议|即时通讯|浏览|其他任务|休息|未知",
  "relevance": "直接相关|间接相关|无关",
  "context_switch": true/false,
  "main_activities": ["活动1", "活动2"],
  "summary": "中断期间活动的简要描述"
}"#;

/// 中断活动分析的 User Prompt 模板
pub fn interruption_analysis_user_prompt(
    original_task: &str,
    focus_goal: &str,
    interruption_duration_minutes: i32,
    activity_summaries: &str,
) -> String {
    format!(
        r#"请分析我中断期间的活动：

原任务：{}
专注目标：{}
中断时长：{}分钟

中断期间的屏幕活动记录：
{}

请分析这些活动与原任务的关联性。"#,
        original_task,
        focus_goal,
        interruption_duration_minutes,
        activity_summaries
    )
}

/// 任务恢复建议的 System Prompt
pub const RESUME_SUGGESTION_SYSTEM_PROMPT: &str = r#"你是一个专注力恢复教练。根据用户的原任务、中断分析结果，帮助用户快速恢复工作状态。

你的建议应该：
1. 简洁明了，不超过3条核心建议
2. 帮助用户快速回忆上次的进度
3. 提供具体的"下一步行动"
4. 如果中断时间长，建议适当调整目标

请用JSON格式输出：
{
  "can_continue": true/false,
  "context_reminder": "上次你正在...(帮助用户回忆)",
  "next_action": "建议的下一步具体行动",
  "adjusted_goal": "如需调整，新的专注目标（可选）",
  "estimated_time_to_refocus": 5,
  "tips": ["快速恢复提示1", "提示2"]
}"#;

/// 任务恢复建议的 User Prompt 模板
pub fn resume_suggestion_user_prompt(
    original_task: &str,
    focus_goal: &str,
    progress_before_interruption: Option<i32>,
    interruption_analysis: &str,
    elapsed_focus_seconds: i32,
    remaining_seconds: i32,
) -> String {
    let progress_text = progress_before_interruption
        .map(|p| format!("中断前进度：{}%", p))
        .unwrap_or_else(|| "进度：未记录".to_string());

    format!(
        r#"请帮我恢复工作状态：

原任务：{}
专注目标：{}
{}
已专注时间：{}分钟
剩余时间：{}分钟

中断分析结果：
{}

请给出恢复工作的建议。"#,
        original_task,
        focus_goal,
        progress_text,
        elapsed_focus_seconds / 60,
        remaining_seconds / 60,
        interruption_analysis
    )
}

/// 快速恢复提示的 System Prompt（轻量级，不需要完整分析时使用）
pub const QUICK_RESUME_SYSTEM_PROMPT: &str = r#"你是一个简洁的工作助手。用户刚刚从中断中恢复，请用一句话帮助他快速回到工作状态。

要求：
- 直接、简洁、有行动导向
- 不超过30个字
- 格式：直接输出一句话，不需要JSON"#;

/// 快速恢复提示的 User Prompt 模板
pub fn quick_resume_user_prompt(
    focus_goal: &str,
    last_activity: &str,
) -> String {
    format!(
        r#"专注目标：{}
最后一次活动：{}

请给出一句快速恢复提示。"#,
        focus_goal,
        last_activity
    )
}
