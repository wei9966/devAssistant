use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use anyhow::{Context, Result};
use once_cell::sync::Lazy;

/// Prompt配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    pub system: String,
    pub user: String,
}

/// 完整的Prompt配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsConfig {
    pub processing: ProcessingPrompts,
    pub generation: GenerationPrompts,
    pub merging: MergingPrompts,
    pub entity_processing: EntityPrompts,
}

/// Processing模块Prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingPrompts {
    pub extraction: ExtractionPrompts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionPrompts {
    /// 单张截图分析提示词
    pub screenshot_single: PromptConfig,
    /// 批量截图分析提示词
    pub screenshot_contextual_batch: PromptConfig,
}

/// Generation模块Prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationPrompts {
    pub merge_hourly_reports: PromptConfig,
    pub generation_report: PromptConfig,
    pub smart_tip_generation: PromptConfig,
    pub todo_extraction: PromptConfig,
    pub realtime_activity_monitor: PromptConfig,
}

/// Merging模块Prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergingPrompts {
    #[serde(rename = "context_merging_multiple")]
    pub merge_batch_items: PromptConfig,
    #[serde(rename = "screenshot_batch_merging")]
    pub merge_weekly_reports: PromptConfig,
}

/// Entity Processing模块Prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityPrompts {
    pub entity_extraction: PromptConfig,
    #[serde(rename = "entity_meta_merging")]
    pub relationship_recognition: PromptConfig,
}

/// 全局单例缓存
static PROMPT_MANAGER: Lazy<Arc<RwLock<Option<PromptManager>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// PromptManager服务
#[derive(Debug, Clone)]
pub struct PromptManager {
    config: PromptsConfig,
    config_path: PathBuf,
}

impl PromptManager {
    /// 从文件加载配置
    pub fn load_from_file(path: &str) -> Result<Self> {
        let config_path = PathBuf::from(path);

        if !config_path.exists() {
            anyhow::bail!("配置文件不存在: {}", path);
        }

        let content = fs::read_to_string(&config_path)
            .context(format!("无法读取配置文件: {}", path))?;

        let config: PromptsConfig = serde_yaml::from_str(&content)
            .context(format!("无法解析YAML配置文件: {}", path))?;

        Ok(Self {
            config,
            config_path,
        })
    }

    /// 从默认路径加载
    pub fn load_default() -> Result<Self> {
        let mut possible_paths: Vec<PathBuf> = vec![
            PathBuf::from("config/prompts_zh.yaml"),
            PathBuf::from("./config/prompts_zh.yaml"),
            PathBuf::from("../config/prompts_zh.yaml"),
        ];

        // 基于可执行文件位置查找
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // 可执行文件同级目录下的 config
                possible_paths.push(exe_dir.join("config").join("prompts_zh.yaml"));
                // 上一级目录的 config（开发时）
                if let Some(parent_dir) = exe_dir.parent() {
                    possible_paths.push(parent_dir.join("config").join("prompts_zh.yaml"));
                }
            }
        }

        // 基于应用数据目录查找
        if let Some(data_dir) = dirs::data_local_dir() {
            possible_paths.push(data_dir.join("dev-assistant").join("config").join("prompts_zh.yaml"));
        }

        for path in &possible_paths {
            if path.exists() {
                return Self::load_from_file(path.to_str().unwrap_or(""));
            }
        }

        anyhow::bail!("找不到默认配置文件，请确保 config/prompts_zh.yaml 存在。已尝试路径: {:?}", possible_paths)
    }

    /// 获取全局单例
    pub fn get_instance() -> Result<Self> {
        let manager = PROMPT_MANAGER.read()
            .map_err(|e| anyhow::anyhow!("获取PromptManager锁失败: {}", e))?;

        if let Some(ref m) = *manager {
            return Ok(m.clone());
        }

        drop(manager);

        // 如果没有初始化，则加载默认配置
        let new_manager = Self::load_default()?;

        let mut manager = PROMPT_MANAGER.write()
            .map_err(|e| anyhow::anyhow!("获取PromptManager写锁失败: {}", e))?;

        *manager = Some(new_manager.clone());

        Ok(new_manager)
    }

    /// 初始化全局单例（使用自定义路径）
    pub fn initialize(path: &str) -> Result<()> {
        let new_manager = Self::load_from_file(path)?;

        let mut manager = PROMPT_MANAGER.write()
            .map_err(|e| anyhow::anyhow!("获取PromptManager写锁失败: {}", e))?;

        *manager = Some(new_manager);

        Ok(())
    }

    /// 热重载配置
    pub fn reload(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.config_path)
            .context(format!("无法读取配置文件: {:?}", self.config_path))?;

        let config: PromptsConfig = serde_yaml::from_str(&content)
            .context(format!("无法解析YAML配置文件: {:?}", self.config_path))?;

        self.config = config;

        // 更新全局缓存
        let mut manager = PROMPT_MANAGER.write()
            .map_err(|e| anyhow::anyhow!("获取PromptManager写锁失败: {}", e))?;

        if let Some(ref mut m) = *manager {
            m.config = self.config.clone();
        }

        Ok(())
    }

    /// 重载全局单例
    pub fn reload_global() -> Result<()> {
        let mut manager = PROMPT_MANAGER.write()
            .map_err(|e| anyhow::anyhow!("获取PromptManager写锁失败: {}", e))?;

        if let Some(ref mut m) = *manager {
            m.reload()?;
        } else {
            anyhow::bail!("PromptManager未初始化，请先调用 get_instance() 或 initialize()");
        }

        Ok(())
    }

    /// 获取单张截图分析Prompt
    pub fn get_screenshot_single_prompt(&self) -> &PromptConfig {
        &self.config.processing.extraction.screenshot_single
    }

    /// 获取批量截图分析Prompt（用于一次性分析多张截图）
    pub fn get_screenshot_prompt(&self) -> &PromptConfig {
        &self.config.processing.extraction.screenshot_contextual_batch
    }

    /// 获取小时报告合并Prompt
    pub fn get_hourly_merge_prompt(&self) -> &PromptConfig {
        &self.config.generation.merge_hourly_reports
    }

    /// 获取日报生成Prompt
    pub fn get_report_prompt(&self) -> &PromptConfig {
        &self.config.generation.generation_report
    }

    /// 获取Tips生成Prompt
    pub fn get_tips_prompt(&self) -> &PromptConfig {
        &self.config.generation.smart_tip_generation
    }

    /// 获取TODO提取Prompt
    pub fn get_todo_prompt(&self) -> &PromptConfig {
        &self.config.generation.todo_extraction
    }

    /// 获取实时活动监控Prompt
    pub fn get_activity_monitor_prompt(&self) -> &PromptConfig {
        &self.config.generation.realtime_activity_monitor
    }

    /// 获取批量合并Prompt
    pub fn get_batch_merge_prompt(&self) -> &PromptConfig {
        &self.config.merging.merge_batch_items
    }

    /// 获取周报合并Prompt
    pub fn get_weekly_merge_prompt(&self) -> &PromptConfig {
        &self.config.merging.merge_weekly_reports
    }

    /// 获取实体提取Prompt
    pub fn get_entity_extraction_prompt(&self) -> &PromptConfig {
        &self.config.entity_processing.entity_extraction
    }

    /// 获取关系识别Prompt
    pub fn get_relationship_prompt(&self) -> &PromptConfig {
        &self.config.entity_processing.relationship_recognition
    }

    /// 获取完整配置
    pub fn get_config(&self) -> &PromptsConfig {
        &self.config
    }

    /// 渲染Prompt模板（变量替换）
    ///
    /// # 参数
    /// * `template` - Prompt模板字符串，支持 {var_name} 和 {{var_name}} 两种占位符格式
    /// * `vars` - 变量映射表
    ///
    /// # 示例
    /// ```
    /// let mut vars = HashMap::new();
    /// vars.insert("current_time".to_string(), "2024-12-04 10:00:00".to_string());
    /// let rendered = PromptManager::render_prompt(
    ///     "当前时间: {current_time}",
    ///     &vars
    /// );
    /// assert_eq!(rendered, "当前时间: 2024-12-04 10:00:00");
    /// ```
    pub fn render_prompt(template: &str, vars: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        for (key, value) in vars {
            // 支持双花括号格式 {{var}}
            let double_placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&double_placeholder, value);

            // 支持单花括号格式 {var}（YAML 配置常用格式）
            let single_placeholder = format!("{{{}}}", key);
            result = result.replace(&single_placeholder, value);
        }

        result
    }

    /// 渲染PromptConfig（同时替换system和user）
    pub fn render_config(
        config: &PromptConfig,
        vars: &HashMap<String, String>
    ) -> PromptConfig {
        PromptConfig {
            system: Self::render_prompt(&config.system, vars),
            user: Self::render_prompt(&config.user, vars),
        }
    }

    /// 便捷方法：渲染截图分析Prompt
    pub fn render_screenshot_prompt(
        &self,
        vars: &HashMap<String, String>
    ) -> PromptConfig {
        Self::render_config(self.get_screenshot_prompt(), vars)
    }

    /// 便捷方法：渲染日报生成Prompt
    pub fn render_report_prompt(
        &self,
        vars: &HashMap<String, String>
    ) -> PromptConfig {
        Self::render_config(self.get_report_prompt(), vars)
    }

    /// 便捷方法：渲染Tips生成Prompt
    pub fn render_tips_prompt(
        &self,
        vars: &HashMap<String, String>
    ) -> PromptConfig {
        Self::render_config(self.get_tips_prompt(), vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_prompt_double_braces() {
        let template = "你好 {{name}}，当前时间是 {{time}}";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "用户".to_string());
        vars.insert("time".to_string(), "12:00".to_string());

        let result = PromptManager::render_prompt(template, &vars);
        assert_eq!(result, "你好 用户，当前时间是 12:00");
    }

    #[test]
    fn test_render_prompt_single_braces() {
        // 测试单花括号格式（YAML配置常用）
        let template = "当前日期: {current_date}，时区: {timezone}";
        let mut vars = HashMap::new();
        vars.insert("current_date".to_string(), "2024-12-04".to_string());
        vars.insert("timezone".to_string(), "+08:00".to_string());

        let result = PromptManager::render_prompt(template, &vars);
        assert_eq!(result, "当前日期: 2024-12-04，时区: +08:00");
    }

    #[test]
    fn test_render_prompt_missing_var() {
        let template = "你好 {name}";
        let vars = HashMap::new();

        let result = PromptManager::render_prompt(template, &vars);
        // 未找到的变量保持原样
        assert_eq!(result, "你好 {name}");
    }

    #[test]
    fn test_render_config() {
        let config = PromptConfig {
            system: "系统: {system_name}".to_string(),
            user: "用户: {user_name}".to_string(),
        };

        let mut vars = HashMap::new();
        vars.insert("system_name".to_string(), "助手".to_string());
        vars.insert("user_name".to_string(), "张三".to_string());

        let result = PromptManager::render_config(&config, &vars);
        assert_eq!(result.system, "系统: 助手");
        assert_eq!(result.user, "用户: 张三");
    }
}
