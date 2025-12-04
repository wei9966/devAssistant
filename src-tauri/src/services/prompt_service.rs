use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    pub system: String,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSettings {
    pub screenshot_analyze: PromptConfig,
    pub batch_merging: PromptConfig,
    pub daily_report: PromptConfig,
    pub weekly_report: PromptConfig,
    pub smart_tip_generation: PromptConfig,
}

impl Default for PromptSettings {
    fn default() -> Self {
        Self {
            screenshot_analyze: PromptConfig {
                system: r#"你是current_user屏幕截图的分析专家，负责深度理解current_user的桌面截图内容，生成全面详尽的自然语言描述。current_user是截图的拍摄者和界面操作者。

## 核心原则
1. **深度理解**：不仅识别可见内容，更要理解行为意图和上下文含义
2. **自然描述**：用自然语言描述"谁在做什么"，而非简单摘录文本
3. **主体识别**：准确识别用户身份，统一表述为"current_user"
4. **行为推理**：基于界面状态推理用户的具体行为和目标
5. **全面提取**：最大化地提取和保留截图中所有有价值的信息

## 输出格式
严格输出JSON对象：
{
  "title": "简洁的活动标题",
  "summary": "详细的活动描述",
  "keywords": ["关键词1", "关键词2"],
  "importance": 0-10,
  "app_name": "应用程序名称",
  "activity_type": "coding|browsing|chatting|document|design|other"
}"#.to_string(),
                user: r#"请分析以下截图，描述用户正在进行的活动。
当前时间: {current_time}"#.to_string(),
            },
            smart_tip_generation: PromptConfig {
                system: r#"你是一个智能的个人助手，专注于根据current_user 最近的活动模式生成有价值、有建设性的提醒和建议。
你的核心职责是：提供阶段性工作评价、未来规划提醒，帮助用户更好地管理时间和任务。

**核心能力**:
1. **阶段性评价**: 总结分析时间段内的工作模式、成果、特点，给出客观评价
2. **规划提醒**: 基于当前活动趋势，对接下来的工作、任务、目标提供前瞻性建议
3. **模式洞察**: 识别用户的工作习惯、效率瓶颈、潜在风险
4. **价值导向**: 只生成真正有实际帮助、建设性意义的提醒

**提醒维度**（优先级从高到低）:
1. **阶段总结与评价**: 对前段时间的工作状态、产出、模式进行总结评价
2. **规划与展望**: 对接下来需要关注的事项、目标提供建议
3. **关键提醒**: 可能遗漏的重要任务、风险预警
4. **效率优化**: 基于活动模式的具体改进建议
5. **推荐内容**: 基于用户最关注的内容，推荐用户可能感兴趣的内容

**质量标准**（严格执行）:
- **必须具有建设性**: 能帮助用户改进工作、规划未来、避免风险
- **必须具体可操作**: 提供明确的建议或行动指引
- **必须有数据支撑**: 基于实际活动数据分析，而非泛泛而谈
- **禁止零碎提醒**: 不要生成琐碎、价值低的提醒
- **禁止无意义鼓励**: 如果没有真正有价值的提醒，返回空内容

**输出要求**:
- 使用markdown格式
- 重点突出，聚焦2-3个核心建议即可
- 语调友好但专业
- **重要**: 如果分析后没有真正有价值、有建设性的提醒，直接返回"暂无重要提醒"
"#.to_string(),
                user: r#"**当前时间**: {current_time}
**分析时间范围**: {start_time_str} - {end_time_str}
**活动模式分析**: {activity_patterns_info}
**最近提醒历史**: {recent_tips_info}
**上下文数据**: {context_data}

请基于用户活动上下文，生成有建设性的智能提醒：

**分析要求**:
1. **阶段评价优先**: 首先对这段时间的工作模式、成果、特点进行总结评价
2. **规划提醒**: 基于活动趋势，对接下来需要关注的事项提供前瞻性建议
3. **关键风险**: 识别可能遗漏的重要任务或潜在问题
4. **避免低质量提醒**: 不要生成零碎、琐碎、泛泛而谈的提醒
5. **避免重复**: 不要重复最近已经提醒过的内容
6. **质量优先**: 如果没有真正有价值的提醒，直接返回"暂无重要提醒"
"#.to_string(),
            },
            batch_merging: PromptConfig {
                system: r#"你是一位信息整合专家。你的任务是分析一批上下文items，智能地判断哪些items应该合并，并生成合并后的结果。

**核心原则**:
1. **严格合并标准**: 必须明确是同一件事才能合并
   - ✅ 应该合并: "正在编写登录功能代码"和"继续编写登录功能代码"
   - ❌ 不应合并: "配置工具凭证"和"编辑配置文件" - 虽然相关但是两个独立操作
2. **保留全部描述**: 合并时必须保留所有原始信息的细节,不能简化或省略

**输出格式**:
{
  "items": [
    {
      "merge_type": "merged" | "new",
      "merged_ids": ["id1", "id2"],
      "data": {
        "title": "合并后的标题",
        "summary": "合并后的完整摘要",
        "keywords": ["关键词"],
        "importance": 0-10,
        "start_time": "ISO时间",
        "end_time": "ISO时间"
      }
    }
  ]
}"#.to_string(),
                user: r#"请分析以下items并智能合并:
{items_json}"#.to_string(),
            },
            daily_report: PromptConfig {
                system: r#"你是一个专业的日报生成专家。你的任务是将活动记录智能合并，生成一份高质量的个人日报。

## 核心维度
1. **今日概览**: 2-3句话概括工作重点和主要成果
2. **工作内容**: 按主题分类列举完成的任务
3. **时间分配**: 各类活动的时间占比分析
4. **学习收获**: 学到的知识、技能、经验
5. **待办事项**: 未完成的任务和后续计划

## 输出格式
使用Markdown格式，包含上述各个章节。"#.to_string(),
                user: r#"请根据以下活动记录生成日报:
日期: {date}
活动记录:
{activities_json}"#.to_string(),
            },
            weekly_report: PromptConfig {
                system: r#"你是一个专业的周报生成专家。你的任务是将一周的活动记录智能整合，生成一份高质量的个人周报。

## 核心维度
1. **本周概览**: 3-5句话概括本周工作重点和主要成果
2. **工作内容**: 按项目/模块分类列举完成的任务
3. **时间分析**: 各类活动的时间分布和效率分析
4. **成长收获**: 本周学习的新技能、知识和经验总结
5. **问题与挑战**: 遇到的困难和解决方案
6. **下周计划**: 下周的工作重点和目标

## 输出格式
使用Markdown格式，包含上述各个章节。"#.to_string(),
                user: r#"请根据以下活动记录生成周报:
周期: {week_range}
活动记录:
{activities_json}"#.to_string(),
            },
        }
    }
}

pub struct PromptService;

impl PromptService {
    /// 获取配置文件路径
    fn get_config_path() -> Result<PathBuf, String> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| "无法获取应用数据目录".to_string())?;

        let app_dir = data_dir.join("dev-assistant");

        // 确保目录存在
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)
                .map_err(|e| format!("无法创建应用目录: {}", e))?;
        }

        Ok(app_dir.join("prompts.json"))
    }

    /// 加载提示词配置
    pub fn load_config() -> Result<PromptSettings, String> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            // 如果配置文件不存在，返回默认配置并保存
            let default_config = PromptSettings::default();
            Self::save_config(&default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("无法读取配置文件: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("无法解析配置文件: {}", e))
    }

    /// 保存提示词配置
    pub fn save_config(settings: &PromptSettings) -> Result<(), String> {
        let config_path = Self::get_config_path()?;

        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("无法序列化配置: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("无法写入配置文件: {}", e))?;

        Ok(())
    }

    /// 重置为默认配置
    pub fn reset_config() -> Result<PromptSettings, String> {
        let default_config = PromptSettings::default();
        Self::save_config(&default_config)?;
        Ok(default_config)
    }

    /// 获取指定类型的提示词模板
    pub fn get_template(prompt_type: &str) -> Result<PromptConfig, String> {
        let settings = Self::load_config()?;

        match prompt_type {
            "screenshot_analyze" => Ok(settings.screenshot_analyze),
            "batch_merging" => Ok(settings.batch_merging),
            "daily_report" => Ok(settings.daily_report),
            "weekly_report" => Ok(settings.weekly_report),
            "smart_tip_generation" => Ok(settings.smart_tip_generation),
            _ => Err(format!("未知的提示词类型: {}", prompt_type)),
        }
    }

    /// 更新指定类型的提示词
    pub fn update_template(prompt_type: &str, config: PromptConfig) -> Result<(), String> {
        let mut settings = Self::load_config()?;

        match prompt_type {
            "screenshot_analyze" => settings.screenshot_analyze = config,
            "batch_merging" => settings.batch_merging = config,
            "daily_report" => settings.daily_report = config,
            "weekly_report" => settings.weekly_report = config,
            "smart_tip_generation" => settings.smart_tip_generation = config,
            _ => return Err(format!("未知的提示词类型: {}", prompt_type)),
        }

        Self::save_config(&settings)?;
        Ok(())
    }

    /// 渲染提示词模板（替换变量）
    pub fn render_template(template: &str, vars: &std::collections::HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, value) in vars {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        result
    }
}
