// PromptManager使用示例
// 本文件仅用于演示，不会被编译到最终程序中

use super::prompt_manager_service::PromptManager;
use std::collections::HashMap;

/// 示例1：获取单例并使用Prompt
pub fn example_get_screenshot_prompt() -> anyhow::Result<()> {
    // 获取全局单例
    let manager = PromptManager::get_instance()?;

    // 获取截图分析Prompt
    let prompt = manager.get_screenshot_prompt();

    println!("System Prompt: {}", prompt.system);
    println!("User Prompt: {}", prompt.user);

    Ok(())
}

/// 示例2：使用变量替换
pub fn example_render_prompt() -> anyhow::Result<()> {
    let manager = PromptManager::get_instance()?;

    // 准备变量
    let mut vars = HashMap::new();
    vars.insert("current_time".to_string(), "2024-12-04 10:00:00".to_string());
    vars.insert("screenshot_count".to_string(), "3".to_string());

    // 渲染截图分析Prompt
    let rendered = manager.render_screenshot_prompt(&vars);

    println!("Rendered System: {}", rendered.system);
    println!("Rendered User: {}", rendered.user);

    Ok(())
}

/// 示例3：获取日报生成Prompt
pub fn example_get_report_prompt() -> anyhow::Result<()> {
    let manager = PromptManager::get_instance()?;

    // 获取日报Prompt
    let prompt = manager.get_report_prompt();

    // 准备变量
    let mut vars = HashMap::new();
    vars.insert("date".to_string(), "2024-12-04".to_string());
    vars.insert("activities_json".to_string(), r#"[{"title": "编写代码", "duration": "2h"}]"#.to_string());

    // 渲染
    let rendered = manager.render_report_prompt(&vars);

    println!("日报生成Prompt:");
    println!("System: {}", rendered.system);
    println!("User: {}", rendered.user);

    Ok(())
}

/// 示例4：热重载配置
pub fn example_reload_config() -> anyhow::Result<()> {
    // 重新加载配置
    PromptManager::reload_global()?;

    println!("配置已重新加载");

    Ok(())
}

/// 示例5：自定义路径初始化
pub fn example_custom_path() -> anyhow::Result<()> {
    // 使用自定义路径初始化
    PromptManager::initialize("custom_config/prompts.yaml")?;

    println!("使用自定义路径初始化成功");

    Ok(())
}

/// 示例6：获取所有类型的Prompt
pub fn example_get_all_prompts() -> anyhow::Result<()> {
    let manager = PromptManager::get_instance()?;

    // 截图分析
    let screenshot = manager.get_screenshot_prompt();
    println!("截图分析Prompt: {}", screenshot.system.lines().next().unwrap_or(""));

    // 小时报告合并
    let hourly_merge = manager.get_hourly_merge_prompt();
    println!("小时报告合并Prompt: {}", hourly_merge.system.lines().next().unwrap_or(""));

    // 日报生成
    let report = manager.get_report_prompt();
    println!("日报生成Prompt: {}", report.system.lines().next().unwrap_or(""));

    // Tips生成
    let tips = manager.get_tips_prompt();
    println!("Tips生成Prompt: {}", tips.system.lines().next().unwrap_or(""));

    // TODO提取
    let todo = manager.get_todo_prompt();
    println!("TODO提取Prompt: {}", todo.system.lines().next().unwrap_or(""));

    // 实时活动监控
    let activity = manager.get_activity_monitor_prompt();
    println!("活动监控Prompt: {}", activity.system.lines().next().unwrap_or(""));

    // 批量合并
    let batch_merge = manager.get_batch_merge_prompt();
    println!("批量合并Prompt: {}", batch_merge.system.lines().next().unwrap_or(""));

    // 周报合并
    let weekly = manager.get_weekly_merge_prompt();
    println!("周报合并Prompt: {}", weekly.system.lines().next().unwrap_or(""));

    // 实体提取
    let entity = manager.get_entity_extraction_prompt();
    println!("实体提取Prompt: {}", entity.system.lines().next().unwrap_or(""));

    // 关系识别
    let relationship = manager.get_relationship_prompt();
    println!("关系识别Prompt: {}", relationship.system.lines().next().unwrap_or(""));

    Ok(())
}

/// 示例7：在AI服务中使用
pub async fn example_use_in_ai_service() -> anyhow::Result<()> {
    let manager = PromptManager::get_instance()?;

    // 准备数据
    let mut vars = HashMap::new();
    vars.insert("current_time".to_string(), chrono::Local::now().to_rfc3339());
    vars.insert("screenshot_count".to_string(), "5".to_string());

    // 渲染Prompt
    let prompt = manager.render_screenshot_prompt(&vars);

    // 构造AI请求（示例）
    // let messages = vec![
    //     ChatMessage {
    //         role: "system".to_string(),
    //         content: prompt.system,
    //     },
    //     ChatMessage {
    //         role: "user".to_string(),
    //         content: prompt.user,
    //     },
    // ];

    // 调用AI服务
    // let response = ai_service.chat(messages).await?;

    println!("AI请求已构造");

    Ok(())
}
