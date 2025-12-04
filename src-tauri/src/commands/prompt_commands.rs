use crate::services::prompt_service::{PromptConfig, PromptService, PromptSettings};

/// 获取提示词配置
#[tauri::command]
pub fn get_prompt_config() -> Result<PromptSettings, String> {
    PromptService::load_config()
}

/// 保存提示词配置
#[tauri::command]
pub fn save_prompt_config(settings: PromptSettings) -> Result<(), String> {
    PromptService::save_config(&settings)
}

/// 重置为默认配置
#[tauri::command]
pub fn reset_prompt_config() -> Result<PromptSettings, String> {
    PromptService::reset_config()
}

/// 获取指定类型的提示词模板
#[tauri::command]
pub fn get_prompt_template(prompt_type: String) -> Result<PromptConfig, String> {
    PromptService::get_template(&prompt_type)
}

/// 更新指定类型的提示词模板
#[tauri::command]
pub fn update_prompt_template(prompt_type: String, config: PromptConfig) -> Result<(), String> {
    PromptService::update_template(&prompt_type, config)
}
