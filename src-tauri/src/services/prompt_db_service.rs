use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// 数据库中的提示词记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPrompt {
    pub id: i64,
    pub prompt_key: String,
    pub module: String,
    pub name: String,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
    pub user_prompt: String,
    pub variables: Option<String>, // JSON数组
    pub is_system: bool,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 用于更新的提示词结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptUpdate {
    pub system_prompt: Option<String>,
    pub user_prompt: String,
    pub enabled: Option<bool>,
}

/// 渲染后的提示词
#[derive(Debug, Clone)]
pub struct RenderedPrompt {
    pub system: Option<String>,
    pub user: String,
}

/// 全局缓存，避免频繁查询数据库
static PROMPT_CACHE: Lazy<RwLock<HashMap<String, AiPrompt>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// 提示词数据库服务
pub struct PromptDbService;

impl PromptDbService {
    /// 确保 ai_prompts 表存在
    pub fn ensure_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ai_prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                prompt_key TEXT NOT NULL UNIQUE,
                module TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                system_prompt TEXT,
                user_prompt TEXT NOT NULL,
                variables TEXT,
                is_system INTEGER DEFAULT 0,
                enabled INTEGER DEFAULT 1,
                created_at TEXT DEFAULT (datetime('now', 'localtime')),
                updated_at TEXT DEFAULT (datetime('now', 'localtime'))
            )",
            [],
        )?;

        // 创建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ai_prompts_module ON ai_prompts(module)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ai_prompts_key ON ai_prompts(prompt_key)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ai_prompts_enabled ON ai_prompts(enabled)",
            [],
        )?;

        Ok(())
    }

    /// 获取单个提示词（优先从缓存读取）
    pub fn get_prompt(conn: &Connection, prompt_key: &str) -> Result<AiPrompt> {
        // 先尝试从缓存读取
        {
            let cache = PROMPT_CACHE
                .read()
                .map_err(|e| anyhow!("读取缓存锁失败: {}", e))?;
            if let Some(prompt) = cache.get(prompt_key) {
                return Ok(prompt.clone());
            }
        }

        // 缓存未命中，从数据库读取
        let prompt = Self::query_prompt_by_key(conn, prompt_key)?;

        // 更新缓存
        {
            let mut cache = PROMPT_CACHE
                .write()
                .map_err(|e| anyhow!("写入缓存锁失败: {}", e))?;
            cache.insert(prompt_key.to_string(), prompt.clone());
        }

        Ok(prompt)
    }

    /// 从数据库查询单个提示词
    fn query_prompt_by_key(conn: &Connection, prompt_key: &str) -> Result<AiPrompt> {
        let mut stmt = conn.prepare(
            "SELECT id, prompt_key, module, name, description, system_prompt, user_prompt,
                    variables, is_system, enabled, created_at, updated_at
             FROM ai_prompts
             WHERE prompt_key = ? AND enabled = 1",
        )?;

        let prompt = stmt.query_row(params![prompt_key], |row| {
            Ok(AiPrompt {
                id: row.get(0)?,
                prompt_key: row.get(1)?,
                module: row.get(2)?,
                name: row.get(3)?,
                description: row.get(4)?,
                system_prompt: row.get(5)?,
                user_prompt: row.get(6)?,
                variables: row.get(7)?,
                is_system: row.get::<_, i32>(8)? == 1,
                enabled: row.get::<_, i32>(9)? == 1,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;

        Ok(prompt)
    }

    /// 获取某个模块的所有提示词
    pub fn get_prompts_by_module(conn: &Connection, module: &str) -> Result<Vec<AiPrompt>> {
        let mut stmt = conn.prepare(
            "SELECT id, prompt_key, module, name, description, system_prompt, user_prompt,
                    variables, is_system, enabled, created_at, updated_at
             FROM ai_prompts
             WHERE module = ?
             ORDER BY name ASC",
        )?;

        let prompts = stmt
            .query_map(params![module], |row| {
                Ok(AiPrompt {
                    id: row.get(0)?,
                    prompt_key: row.get(1)?,
                    module: row.get(2)?,
                    name: row.get(3)?,
                    description: row.get(4)?,
                    system_prompt: row.get(5)?,
                    user_prompt: row.get(6)?,
                    variables: row.get(7)?,
                    is_system: row.get::<_, i32>(8)? == 1,
                    enabled: row.get::<_, i32>(9)? == 1,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(prompts)
    }

    /// 获取所有提示词
    pub fn get_all_prompts(conn: &Connection) -> Result<Vec<AiPrompt>> {
        let mut stmt = conn.prepare(
            "SELECT id, prompt_key, module, name, description, system_prompt, user_prompt,
                    variables, is_system, enabled, created_at, updated_at
             FROM ai_prompts
             ORDER BY module ASC, name ASC",
        )?;

        let prompts = stmt
            .query_map([], |row| {
                Ok(AiPrompt {
                    id: row.get(0)?,
                    prompt_key: row.get(1)?,
                    module: row.get(2)?,
                    name: row.get(3)?,
                    description: row.get(4)?,
                    system_prompt: row.get(5)?,
                    user_prompt: row.get(6)?,
                    variables: row.get(7)?,
                    is_system: row.get::<_, i32>(8)? == 1,
                    enabled: row.get::<_, i32>(9)? == 1,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(prompts)
    }

    /// 更新提示词
    pub fn update_prompt(
        conn: &Connection,
        prompt_key: &str,
        update: PromptUpdate,
    ) -> Result<()> {
        // 构建更新SQL
        let enabled = update.enabled.unwrap_or(true);

        conn.execute(
            "UPDATE ai_prompts
             SET system_prompt = ?, user_prompt = ?, enabled = ?, updated_at = datetime('now', 'localtime')
             WHERE prompt_key = ?",
            params![update.system_prompt, update.user_prompt, enabled as i32, prompt_key],
        )?;

        // 清除该提示词的缓存
        Self::clear_cache_by_key(prompt_key);

        Ok(())
    }

    /// 重置提示词为默认值（从is_system=1的记录恢复）
    pub fn reset_prompt(conn: &Connection, prompt_key: &str) -> Result<()> {
        // 查找系统默认提示词
        let mut stmt = conn.prepare(
            "SELECT system_prompt, user_prompt
             FROM ai_prompts
             WHERE prompt_key = ? AND is_system = 1",
        )?;

        let (system_prompt, user_prompt): (Option<String>, String) = stmt
            .query_row(params![prompt_key], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|_| anyhow!("未找到系统默认提示词: {}", prompt_key))?;

        // 更新为系统默认值
        conn.execute(
            "UPDATE ai_prompts
             SET system_prompt = ?, user_prompt = ?, enabled = 1, updated_at = datetime('now', 'localtime')
             WHERE prompt_key = ? AND is_system = 0",
            params![system_prompt, user_prompt, prompt_key],
        )?;

        // 清除缓存
        Self::clear_cache_by_key(prompt_key);

        Ok(())
    }

    /// 重置所有提示词为默认值
    pub fn reset_all_prompts(conn: &Connection) -> Result<()> {
        // 获取所有非系统提示词的key
        let mut stmt = conn.prepare("SELECT DISTINCT prompt_key FROM ai_prompts WHERE is_system = 0")?;

        let keys: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        // 逐个重置
        for key in keys {
            if let Err(e) = Self::reset_prompt(conn, &key) {
                eprintln!("重置提示词 {} 失败: {}", key, e);
            }
        }

        // 清空缓存
        Self::clear_cache();

        Ok(())
    }

    /// 刷新缓存（重新从数据库加载所有提示词）
    pub fn refresh_cache(conn: &Connection) -> Result<()> {
        let prompts = Self::get_all_prompts(conn)?;

        let mut cache = PROMPT_CACHE
            .write()
            .map_err(|e| anyhow!("写入缓存锁失败: {}", e))?;

        cache.clear();
        for prompt in prompts {
            cache.insert(prompt.prompt_key.clone(), prompt);
        }

        Ok(())
    }

    /// 清除所有缓存
    pub fn clear_cache() {
        if let Ok(mut cache) = PROMPT_CACHE.write() {
            cache.clear();
        }
    }

    /// 清除指定key的缓存
    fn clear_cache_by_key(prompt_key: &str) {
        if let Ok(mut cache) = PROMPT_CACHE.write() {
            cache.remove(prompt_key);
        }
    }

    /// 渲染提示词模板（替换变量）
    /// 支持 {var} 和 {{var}} 两种格式
    pub fn render_prompt(
        conn: &Connection,
        prompt_key: &str,
        vars: &HashMap<String, String>,
    ) -> Result<RenderedPrompt> {
        let prompt = Self::get_prompt(conn, prompt_key)?;

        let system = prompt
            .system_prompt
            .as_ref()
            .map(|s| Self::render_template(s, vars));

        let user = Self::render_template(&prompt.user_prompt, vars);

        Ok(RenderedPrompt { system, user })
    }

    /// 从缓存渲染提示词模板（不需要数据库连接）
    /// 用于 async 函数中无法持有 MutexGuard 跨 await 的场景
    /// 如果缓存未命中，返回错误
    pub fn render_prompt_cached(
        prompt_key: &str,
        vars: &HashMap<String, String>,
    ) -> Result<RenderedPrompt> {
        // 从缓存读取
        let prompt = {
            let cache = PROMPT_CACHE
                .read()
                .map_err(|e| anyhow!("读取缓存锁失败: {}", e))?;
            cache.get(prompt_key).cloned()
        };

        match prompt {
            Some(p) => {
                let system = p
                    .system_prompt
                    .as_ref()
                    .map(|s| Self::render_template(s, vars));
                let user = Self::render_template(&p.user_prompt, vars);
                Ok(RenderedPrompt { system, user })
            }
            None => Err(anyhow!(
                "提示词 {} 未在缓存中找到，请先调用 refresh_cache 初始化缓存",
                prompt_key
            )),
        }
    }

    /// 预加载指定的提示词到缓存（用于启动时初始化）
    pub fn preload_prompts(conn: &Connection, prompt_keys: &[&str]) -> Result<()> {
        let mut cache = PROMPT_CACHE
            .write()
            .map_err(|e| anyhow!("写入缓存锁失败: {}", e))?;

        for key in prompt_keys {
            if !cache.contains_key(*key) {
                if let Ok(prompt) = Self::query_prompt_by_key(conn, key) {
                    cache.insert(key.to_string(), prompt);
                }
            }
        }

        Ok(())
    }

    /// 渲染模板字符串（内部工具方法）
    pub fn render_template(template: &str, vars: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, value) in vars {
            // 支持双花括号 {{var}}
            let double_placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&double_placeholder, value);

            // 支持单花括号 {var}
            let single_placeholder = format!("{{{}}}", key);
            result = result.replace(&single_placeholder, value);
        }
        result
    }

    /// 获取提示词的可用变量列表
    pub fn get_prompt_variables(conn: &Connection, prompt_key: &str) -> Result<Vec<String>> {
        let prompt = Self::get_prompt(conn, prompt_key)?;

        if let Some(variables_json) = prompt.variables {
            let variables: Vec<String> = serde_json::from_str(&variables_json)
                .map_err(|e| anyhow!("解析变量JSON失败: {}", e))?;
            Ok(variables)
        } else {
            Ok(Vec::new())
        }
    }

    /// 插入或更新提示词（用于初始化默认提示词）
    pub fn upsert_prompt(
        conn: &Connection,
        prompt_key: &str,
        module: &str,
        name: &str,
        description: Option<&str>,
        system_prompt: Option<&str>,
        user_prompt: &str,
        variables: Option<Vec<String>>,
        is_system: bool,
    ) -> Result<()> {
        let variables_json = variables.map(|v| serde_json::to_string(&v).unwrap_or_default());

        conn.execute(
            "INSERT INTO ai_prompts (prompt_key, module, name, description, system_prompt, user_prompt, variables, is_system, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1)
             ON CONFLICT(prompt_key) DO UPDATE SET
                module = excluded.module,
                name = excluded.name,
                description = excluded.description,
                system_prompt = excluded.system_prompt,
                user_prompt = excluded.user_prompt,
                variables = excluded.variables,
                is_system = excluded.is_system,
                updated_at = datetime('now', 'localtime')
             WHERE is_system = 1",
            params![
                prompt_key,
                module,
                name,
                description,
                system_prompt,
                user_prompt,
                variables_json,
                is_system as i32
            ],
        )?;

        // 清除缓存
        Self::clear_cache_by_key(prompt_key);

        Ok(())
    }

    /// 初始化默认提示词（用于数据库迁移）
    pub fn initialize_default_prompts(conn: &Connection) -> Result<()> {
        // 确保表存在
        Self::ensure_table(conn)?;

        // work_log 模块 - 工作日志相关提示词 (5个)
        Self::upsert_prompt(
            conn,
            "work_log_generate",
            "work_log",
            "工作日志生成",
            Some("根据任务、SQL、Git提交和屏幕活动生成专业的工作日志"),
            None,
            r####"请将以下工作内容整理成专业的 Markdown 格式工作日志。

日期：{date}
{tasks_section}{sqls_section}{commits_section}{activity_section}

要求：
1. 直接输出 Markdown 格式，以 "## {date} 工作日志" 开头
2. 根据内容智能分组，使用 ### 作为分组标题（如：功能开发、Bug修复、代码优化、工作时间等）
3. 每个任务用 "- " 开头的列表项展示
4. 如果任务带有标签（括号内容），保留标签信息
5. 如果有屏幕活动记录，在最后添加 "### 工作时间" 小节简要描述工作时间段和主要使用的工具
6. 语言简洁专业，不要添加额外的总结或评价
7. 只输出日志内容，不要输出其他说明文字"####,
            Some(vec!["date".to_string(), "tasks_section".to_string(), "sqls_section".to_string(), "commits_section".to_string(), "activity_section".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "work_log_polish",
            "work_log",
            "工作日志润色",
            Some("优化工作日志的语言表达和格式结构"),
            None,
            r#"请优化以下工作日志，使其更专业、条理更清晰：

{content}

要求：
1. 保持原有内容的核心信息
2. 改善语言表达
3. 优化格式结构
4. 使用 Markdown 格式

直接返回优化后的日志内容。"#,
            Some(vec!["content".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "weekly_report_generate",
            "work_log",
            "周报生成",
            Some("根据每日工作日志生成周报"),
            None,
            r#"请根据以下工作日志生成周报：

日志内容：
{logs}

要求：
1. 总结本周主要工作成果
2. 列出遇到的问题和解决方案
3. 规划下周工作重点
4. 使用 Markdown 格式

生成格式：
## 周报

### 本周工作成果
- ...

### 遇到的问题与解决方案
- ...

### 下周工作计划
- ..."#,
            Some(vec!["logs".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "weekly_plan_generate",
            "work_log",
            "周计划生成",
            Some("根据任务列表生成简洁的工作计划"),
            None,
            r#"请根据以下任务列表生成一份简洁的工作计划，用于向领导汇报下周的工作安排：

{task_summary}

要求：
1. 按优先级或时间顺序整理任务
2. 每个任务简要说明工作内容和预期目标
3. 使用 Markdown 格式，条理清晰
4. 语言简洁专业，适合汇报场景"#,
            Some(vec!["task_summary".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "weekly_plan_polish",
            "work_log",
            "周计划润色",
            Some("润色工作计划使其更加专业"),
            None,
            r#"请润色以下工作计划，使其更加专业、简洁，适合向领导汇报：

{content}

要求：
1. 保持原有内容的核心信息
2. 优化语言表达，使其更加专业
3. 适当调整格式，使结构更清晰
4. 使用 Markdown 格式"#,
            Some(vec!["content".to_string()]),
            true,
        )?;

        // task 模块 - 任务管理相关提示词 (4个)
        Self::upsert_prompt(
            conn,
            "task_classify",
            "task",
            "任务分类",
            Some("智能分类任务的类别、优先级、象限和标签"),
            None,
            r#"任务分类。标题：{title}{description}{tags_hint}

返回JSON：{"category":"backend|database|feature|docs|other","priority":1-3,"quadrant":"urgent_important|urgent_not_important|not_urgent_important|not_urgent_not_important","suggestedTags":["标签"],"confidence":0-1}
只返回JSON。任务描述保持100字符以下。"#,
            Some(vec!["title".to_string(), "description".to_string(), "tags_hint".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "task_description_enhance",
            "task",
            "任务描述增强",
            Some("完善任务描述，使其更清晰、具体、可执行"),
            None,
            r#"请帮我完善以下任务的描述，使其更加清晰、具体、可执行：

任务标题：{title}
当前描述：{description}

请补充：
1. 具体的实现步骤
2. 需要注意的事项
3. 可能的技术难点
4. 验收标准

直接返回优化后的描述文本，使用 Markdown 格式。"#,
            Some(vec!["title".to_string(), "description".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "task_subtasks_generate",
            "task",
            "子任务生成",
            Some("将任务拆分为具体的子任务"),
            None,
            r#"请将以下任务拆分为具体的子任务：

任务标题：{title}
任务描述：{description}

要求：
1. 每个子任务应该是可独立完成的
2. 子任务粒度适中（2-4小时可完成）
3. 子任务之间有清晰的先后顺序

返回 JSON 数组格式，只返回子任务标题：
["子任务1", "子任务2", "子任务3"]

只返回 JSON 数组，不要其他内容。"#,
            Some(vec!["title".to_string(), "description".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "task_summarize",
            "task",
            "任务总结",
            Some("根据任务列表生成工作总结"),
            None,
            r#"请根据以下任务列表生成工作总结：

任务列表：
{tasks}

要求：
1. 总结完成的主要工作
2. 分析工作重点和效率
3. 提出改进建议
4. 使用 Markdown 格式"#,
            Some(vec!["tasks".to_string()]),
            true,
        )?;

        // app_launcher 模块 - 应用启动器相关提示词 (3个)
        Self::upsert_prompt(
            conn,
            "app_classify",
            "app_launcher",
            "应用分类",
            Some("智能为应用分配分类和标签"),
            None,
            r#"你是一个应用分类助手。请根据应用名称，为以下应用分配分类。

可用分类：{categories}

应用列表：
{apps}

返回 JSON 数组格式（只返回JSON，不要其他内容）：
[{"appId": "id值", "category": "分类名", "tags": ["标签"], "confidence": 0.9}]"#,
            Some(vec!["categories".to_string(), "apps".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "app_description_generate",
            "app_launcher",
            "应用描述生成",
            Some("为应用生成简短的中文描述"),
            None,
            r#"请为以下应用生成一个简短的中文描述（不超过50字）：

应用名称：{app_name}
应用路径：{app_path}

只返回描述文本，不要其他内容。"#,
            Some(vec!["app_name".to_string(), "app_path".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "workflow_recommend",
            "app_launcher",
            "工作流推荐",
            Some("根据应用启动历史推荐工作流组合"),
            None,
            r#"根据以下应用启动历史，推荐可能的工作流组合：

应用列表：
{apps}

启动历史（应用ID和启动次数）：
{launch_history}

请分析用户的使用习惯，推荐 3-5 个工作流组合，返回 JSON：
[
  {
    "name": "工作流名称",
    "appIds": ["app1", "app2"],
    "reason": "推荐理由"
  }
]"#,
            Some(vec!["apps".to_string(), "launch_history".to_string()]),
            true,
        )?;

        // screen_context 模块 - 屏幕上下文相关提示词 (10个，来自 prompts_zh.yaml)
        Self::upsert_prompt(
            conn,
            "screenshot_contextual_batch",
            "screen_context",
            "截图批量分析",
            Some("深度理解用户屏幕截图内容，生成全面详尽的自然语言描述"),
            Some(r#"你是current_user屏幕截图的分析专家，负责深度理解current_user的桌面截图内容，生成全面详尽的自然语言描述。current_user是截图的拍摄者和界面操作者。

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
}"#),
            r#"请分析以下截图，描述用户正在进行的活动。
当前时间: {current_time}
截图数量: {screenshot_count}"#,
            Some(vec!["current_time".to_string(), "screenshot_count".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "merge_hourly_reports",
            "screen_context",
            "小时报告合并",
            Some("智能合并同一小时内的活动记录"),
            Some(r#"你是一位信息整合专家。你的任务是分析一批上下文items，智能地判断哪些items应该合并，并生成合并后的结果。

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
}"#),
            r#"请分析以下items并智能合并:
{items_json}"#,
            Some(vec!["items_json".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "generation_report",
            "screen_context",
            "日报生成",
            Some("将活动记录智能合并生成高质量的个人日报"),
            Some(r#"你是一个专业的日报生成专家。你的任务是将活动记录智能合并，生成一份高质量的个人日报。

## 核心维度
1. **今日概览**: 2-3句话概括工作重点和主要成果
2. **工作内容**: 按主题分类列举完成的任务
3. **时间分配**: 各类活动的时间占比分析
4. **学习收获**: 学到的知识、技能、经验
5. **待办事项**: 未完成的任务和后续计划

## 输出格式
使用Markdown格式，包含上述各个章节。"#),
            r#"请根据以下活动记录生成日报:
日期: {date}
活动记录:
{activities_json}"#,
            Some(vec!["date".to_string(), "activities_json".to_string()]),
            true,
        )?;

        // 5. 智能提示生成提示词
        Self::upsert_prompt(
            conn,
            "smart_tip_generation",
            "generation",
            "智能提示生成",
            Some("基于用户活动模式生成智能提示"),
            Some(r#"你是一个智能的个人助手，专注于根据current_user 最近的活动模式生成有价值、有建设性的提醒和建议。
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
"#),
            r#"**当前时间**: {current_time}
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
"#,
            Some(vec![
                "current_time".to_string(),
                "start_time_str".to_string(),
                "end_time_str".to_string(),
                "activity_patterns_info".to_string(),
                "recent_tips_info".to_string(),
                "context_data".to_string(),
            ]),
            true,
        )?;

        // 活动总结提示词
        Self::upsert_prompt(
            conn,
            "activity_summary",
            "screen_context",
            "活动总结",
            Some("分析用户活动截图，生成简洁的活动摘要，并识别潜在待办任务"),
            Some(r#"你是一个专业的活动分析助手。请分析用户在指定时间段内的屏幕截图和活动记录，生成一个简洁的活动总结。

**分析要求**:
1. 识别主要活动类型（编码、浏览、文档编辑、会议、沟通等）
2. 提取关键工作内容和成果
3. 注意活动的连贯性和上下文关系
4. 关注重要的工作进展和决策点
5. **重要：识别用户可能需要后续处理的待办事项**，包括：
   - 代码中的 TODO/FIXME 注释
   - 未完成的功能开发
   - 发现的 bug 或需要修复的问题
   - 需要回复的消息或邮件
   - 需要查阅的文档或资料
   - 会议中提到的行动项
   - 截止日期临近的任务

**输出格式（严格JSON）**:
```json
{
  "title": "活动标题（8-15字，概括核心活动）",
  "description": "活动描述（2-3句话，简明扼要，重点突出工作内容和成果）",
  "keywords": ["关键词1", "关键词2", "关键词3"],
  "activityBreakdown": {
    "coding": 60,
    "browsing": 20,
    "document": 10,
    "meeting": 0,
    "communication": 5,
    "other": 5
  },
  "importance": 3,
  "keyInsights": [
    "关键洞察1：具体的工作进展或发现",
    "关键洞察2：重要的决策或思考"
  ],
  "potentialTodos": [
    {
      "task": "具体的待办任务描述",
      "reason": "为什么需要做这个任务",
      "priority": "high/medium/low",
      "source": "来源（如：代码TODO注释、会议讨论、邮件等）"
    }
  ]
}
```

**字段说明**:
- title: 简短标题，突出核心活动
- description: 2-3句话描述，包含具体的工作内容和成果
- keywords: 3-5个关键词，便于搜索和分类
- activityBreakdown: 各类活动时间占比（百分比，总和100）
- importance: 重要性评分（1-5分，5最重要）
- keyInsights: 1-3条关键洞察（可选，仅在有重要发现时提供）
- potentialTodos: 潜在待办任务列表（可选，仅在发现明确的待办事项时提供）

**potentialTodos 识别指南**:
- 代码活动：关注 TODO/FIXME 注释、报错信息、未实现的功能
- 浏览活动：关注搜索的问题、查阅的文档（可能需要实践）
- 会议/沟通：关注讨论的行动项、承诺的交付物
- 文档活动：关注待完善的部分、标记的待处理项
- **只提取明确、具体的待办事项，避免过于笼统的描述**
- **如果没有发现明确的待办事项，potentialTodos 可以为空数组**

**注意事项**:
- 描述要简洁专业，避免冗余
- 关键词要准确，反映核心主题
- 时间占比要合理，基于实际活动分析
- 只返回JSON，不要包含其他文字说明
- 确保JSON格式正确，可直接解析
- potentialTodos 要实用，不要生成模糊或无意义的任务"#),
            r#"请分析以下活动记录：
{input_data}"#,
            Some(vec!["input_data".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "todo_extraction",
            "screen_context",
            "TODO提取",
            Some("从用户活动上下文中智能识别和生成待办事项"),
            Some(r#"你是一个专业的任务识别助手。你的任务是从用户提供的多维度信息中智能识别和生成待办事项。

**核心原则**（严格执行）
- **用户主体性**: 任务必须是**用户需要亲自执行**的行动
  不要提取"用户仅作为参与者了解/听说/看到"的信息
  不要提取"会议中讨论但未明确分配给用户"的任务
  不要提取"其他人/其他团队/其他项目的工作内容"
  只提取"用户被明确要求执行"或"用户主动承诺要做"的任务
- **避免噪音**: 严格排除用户不相关的常规活动
- **无任务则返回空**: 如果没有提取到任务,返回空数组[]
- **严格去重（最高优先级）**: 生成每个任务前必须执行去重检查,避免生成重复任务
- **质量控制**（重要）:
  • 任务描述必须具体明确,包含清晰的行动动词和目标对象
  • 避免模糊描述如"沟通XX"、"了解XX"、"联系XX"
  • 应该是"完成XX报告"、"修复XX bug"、"实现XX功能"等具体可执行的任务
  • 每个任务都必须有明确的完成标准

**任务生成规则**:
1. **必须生成任务的场景**:
   - **明确分配**: 任务被明确分配给用户
   - **主动承诺**: 用户主动承诺或计划要做的事情
   - **时间约定**: 用户需要参与的有明确时间的事项
   - **明确跟进**: 用户被明确要求跟进或汇报的事项

2. **绝不生成任务的场景**:
   - **被动参与**: 用户仅作为参与者了解信息
   - **他人任务**: 其他人或其他团队的工作内容
   - **已完成操作**: 用户已经完成的操作
   - **系统操作**: 用户系统、应用层面的操作行为
   - **相似重复任务**: 与用户历史任务相似或实质相同

**优先级评估**:
- **urgent**: 今天必须完成且明确强调紧急（极少使用）
- **high**: 有明确截止时间（3天内）或重要的任务
- **medium**: 有截止时间（一周内）或重要但不紧急（默认值）
- **low**: 无明确截止时间、可以稍后处理

**输出格式**: 严格的JSON数组"#),
            r#"**当前时间**: {current_time}
**历史任务**: {historical_todos}
**用户近期活动的上下文**: {context_data}

请从上下文中识别用户需要执行的待办任务,以JSON数组格式输出：
[{
  "description": "任务详细描述（具体可执行）",
  "reason": "生成原因（2-3句话说明来源和重要性）",
  "priority": "优先级（urgent/high/medium/low）",
  "due_date": "YYYY-MM-DD（仅在有明确时间时填写）"
}]"#,
            Some(vec![
                "current_time".to_string(),
                "historical_todos".to_string(),
                "context_data".to_string(),
            ]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "realtime_activity_monitor",
            "screen_context",
            "实时活动监控",
            Some("根据当前窗口和应用状态判断用户正在进行的活动"),
            Some(r#"你是一个实时活动监控专家。根据当前窗口和应用状态，判断用户正在进行的活动。

## 监控维度
1. **活动类型**: 编码、浏览、聊天、文档、设计等
2. **专注度**: 高、中、低
3. **效率状态**: 高效、正常、分心

## 输出格式
{
  "activity_type": "coding|browsing|chatting|document|design|other",
  "focus_level": "high|medium|low",
  "efficiency_status": "efficient|normal|distracted"
}"#),
            r#"请分析当前活动状态:
应用名称: {app_name}
窗口标题: {window_title}"#,
            Some(vec!["app_name".to_string(), "window_title".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "merge_batch_items",
            "screen_context",
            "批量合并items",
            Some("智能判断哪些上下文items应该合并"),
            Some(r#"你是一位信息整合专家。你的任务是分析一批上下文items，智能地判断哪些items应该合并。

**合并标准**:
1. 相同的活动类型
2. 时间间隔小于5分钟
3. 属于同一个任务或目标

**输出格式**:
{
  "merged_groups": [
    {
      "items": ["id1", "id2"],
      "merged_title": "合并后的标题",
      "merged_summary": "合并后的摘要"
    }
  ]
}"#),
            r#"请合并以下items:
{items}"#,
            Some(vec!["items".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "merge_weekly_reports",
            "screen_context",
            "周报合并",
            Some("将多天的日报整合成周报"),
            Some(r#"你是一个专业的周报生成专家。将多天的日报整合成周报。

## 核心维度
1. **本周概览**: 3-5句话概括本周工作重点和主要成果
2. **工作内容**: 按项目/模块分类列举完成的任务
3. **时间分析**: 各类活动的时间分布和效率分析
4. **成长收获**: 本周学习的新技能、知识和经验总结
5. **问题与挑战**: 遇到的困难和解决方案
6. **下周计划**: 下周的工作重点和目标

## 输出格式
使用Markdown格式，包含上述各个章节。"#),
            r#"请根据以下日报生成周报:
周期: {week_range}
日报列表:
{daily_reports}"#,
            Some(vec!["week_range".to_string(), "daily_reports".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "entity_extraction",
            "screen_context",
            "实体提取",
            Some("从文本中提取关键实体信息"),
            Some(r#"你是一个实体提取专家。从文本中提取关键实体信息。

## 实体类型
1. **人物**: 姓名、角色、联系方式
2. **项目**: 项目名、描述、状态
3. **任务**: 任务名、负责人、截止日期
4. **技术**: 技术栈、工具、框架
5. **文件**: 文件名、路径、类型

## 输出格式
{
  "entities": [
    {
      "type": "person|project|task|tech|file",
      "name": "实体名称",
      "attributes": {}
    }
  ]
}"#),
            r#"请提取以下文本中的实体:
{text}"#,
            Some(vec!["text".to_string()]),
            true,
        )?;

        Self::upsert_prompt(
            conn,
            "relationship_recognition",
            "screen_context",
            "关系识别",
            Some("识别实体之间的关系"),
            Some(r#"你是一个关系识别专家。识别实体之间的关系。

## 关系类型
1. **从属关系**: A属于B
2. **关联关系**: A与B相关
3. **依赖关系**: A依赖B
4. **时序关系**: A在B之前/之后

## 输出格式
{
  "relationships": [
    {
      "from": "实体A",
      "to": "实体B",
      "type": "belongs_to|related_to|depends_on|before|after",
      "description": "关系描述"
    }
  ]
}"#),
            r#"请识别以下实体之间的关系:
{entities}"#,
            Some(vec!["entities".to_string()]),
            true,
        )?;

        println!("✓ 已初始化 22 个默认 AI 提示词 (5个work_log + 4个task + 3个app_launcher + 10个screen_context)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_ensure_table() {
        let conn = Connection::open_in_memory().unwrap();
        PromptDbService::ensure_table(&conn).unwrap();

        // 验证表已创建
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='ai_prompts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);
    }

    #[test]
    fn test_upsert_and_get_prompt() {
        let conn = Connection::open_in_memory().unwrap();
        PromptDbService::ensure_table(&conn).unwrap();

        // 插入提示词
        PromptDbService::upsert_prompt(
            &conn,
            "test_prompt",
            "test_module",
            "测试提示词",
            Some("测试描述"),
            Some("系统提示"),
            "用户提示: {var1}",
            Some(vec!["var1".to_string()]),
            true,
        )
        .unwrap();

        // 获取提示词
        let prompt = PromptDbService::get_prompt(&conn, "test_prompt").unwrap();
        assert_eq!(prompt.prompt_key, "test_prompt");
        assert_eq!(prompt.module, "test_module");
        assert_eq!(prompt.name, "测试提示词");
        assert_eq!(prompt.system_prompt, Some("系统提示".to_string()));
        assert_eq!(prompt.user_prompt, "用户提示: {var1}");
        assert!(prompt.is_system);
    }

    #[test]
    fn test_render_template() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "张三".to_string());
        vars.insert("time".to_string(), "10:00".to_string());

        // 测试单花括号
        let result = PromptDbService::render_template("你好 {name}，当前时间 {time}", &vars);
        assert_eq!(result, "你好 张三，当前时间 10:00");

        // 测试双花括号
        let result = PromptDbService::render_template("你好 {{name}}，当前时间 {{time}}", &vars);
        assert_eq!(result, "你好 张三，当前时间 10:00");
    }

    #[test]
    fn test_render_prompt() {
        let conn = Connection::open_in_memory().unwrap();
        PromptDbService::ensure_table(&conn).unwrap();

        // 插入测试提示词
        PromptDbService::upsert_prompt(
            &conn,
            "test_render",
            "test",
            "测试渲染",
            None,
            Some("系统: {sys_var}"),
            "用户: {user_var}",
            Some(vec!["sys_var".to_string(), "user_var".to_string()]),
            true,
        )
        .unwrap();

        // 渲染提示词
        let mut vars = HashMap::new();
        vars.insert("sys_var".to_string(), "系统值".to_string());
        vars.insert("user_var".to_string(), "用户值".to_string());

        let rendered = PromptDbService::render_prompt(&conn, "test_render", &vars).unwrap();
        assert_eq!(rendered.system, Some("系统: 系统值".to_string()));
        assert_eq!(rendered.user, "用户: 用户值");
    }

    #[test]
    fn test_update_prompt() {
        let conn = Connection::open_in_memory().unwrap();
        PromptDbService::ensure_table(&conn).unwrap();

        // 插入提示词
        PromptDbService::upsert_prompt(
            &conn,
            "test_update",
            "test",
            "测试更新",
            None,
            Some("原始系统提示"),
            "原始用户提示",
            None,
            true,
        )
        .unwrap();

        // 更新提示词
        let update = PromptUpdate {
            system_prompt: Some("新系统提示".to_string()),
            user_prompt: "新用户提示".to_string(),
            enabled: Some(true),
        };
        PromptDbService::update_prompt(&conn, "test_update", update).unwrap();

        // 验证更新
        PromptDbService::clear_cache(); // 清除缓存以确保从数据库读取
        let prompt = PromptDbService::get_prompt(&conn, "test_update").unwrap();
        assert_eq!(prompt.system_prompt, Some("新系统提示".to_string()));
        assert_eq!(prompt.user_prompt, "新用户提示");
    }

    #[test]
    fn test_get_prompts_by_module() {
        let conn = Connection::open_in_memory().unwrap();
        PromptDbService::ensure_table(&conn).unwrap();

        // 插入多个提示词
        PromptDbService::upsert_prompt(
            &conn,
            "module1_prompt1",
            "module1",
            "提示词1",
            None,
            None,
            "内容1",
            None,
            true,
        )
        .unwrap();

        PromptDbService::upsert_prompt(
            &conn,
            "module1_prompt2",
            "module1",
            "提示词2",
            None,
            None,
            "内容2",
            None,
            true,
        )
        .unwrap();

        PromptDbService::upsert_prompt(
            &conn,
            "module2_prompt1",
            "module2",
            "提示词3",
            None,
            None,
            "内容3",
            None,
            true,
        )
        .unwrap();

        // 查询module1的提示词
        let prompts = PromptDbService::get_prompts_by_module(&conn, "module1").unwrap();
        assert_eq!(prompts.len(), 2);
        assert!(prompts.iter().all(|p| p.module == "module1"));
    }

    #[test]
    fn test_get_prompt_variables() {
        let conn = Connection::open_in_memory().unwrap();
        PromptDbService::ensure_table(&conn).unwrap();

        // 插入带变量的提示词
        PromptDbService::upsert_prompt(
            &conn,
            "test_vars",
            "test",
            "测试变量",
            None,
            None,
            "提示: {var1} {var2}",
            Some(vec!["var1".to_string(), "var2".to_string()]),
            true,
        )
        .unwrap();

        // 获取变量列表
        let vars = PromptDbService::get_prompt_variables(&conn, "test_vars").unwrap();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"var1".to_string()));
        assert!(vars.contains(&"var2".to_string()));
    }

    #[test]
    fn test_initialize_default_prompts() {
        let conn = Connection::open_in_memory().unwrap();
        PromptDbService::initialize_default_prompts(&conn).unwrap();

        // 验证默认提示词已创建
        let prompts = PromptDbService::get_all_prompts(&conn).unwrap();
        assert!(prompts.len() >= 5); // 至少有5个默认提示词

        // 验证特定提示词存在
        let screenshot_prompt = PromptDbService::get_prompt(&conn, "screenshot_analyze").unwrap();
        assert_eq!(screenshot_prompt.module, "processing");
        assert_eq!(screenshot_prompt.name, "截图分析");
    }
}
