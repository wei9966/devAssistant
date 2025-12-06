use crate::db::connection::DbConnection;
use crate::services::prompt_db_service::{AiPrompt, PromptDbService, PromptUpdate, RenderedPrompt};
use std::collections::HashMap;
use tauri::State;

/// 获取所有提示词
#[tauri::command]
pub async fn get_all_prompts(db: State<'_, DbConnection>) -> Result<Vec<AiPrompt>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PromptDbService::get_all_prompts(&conn).map_err(|e| e.to_string())
}

/// 获取单个提示词
#[tauri::command]
pub async fn get_prompt(db: State<'_, DbConnection>, prompt_key: String) -> Result<AiPrompt, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PromptDbService::get_prompt(&conn, &prompt_key).map_err(|e| e.to_string())
}

/// 获取某个模块的所有提示词
#[tauri::command]
pub async fn get_prompts_by_module(db: State<'_, DbConnection>, module: String) -> Result<Vec<AiPrompt>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PromptDbService::get_prompts_by_module(&conn, &module).map_err(|e| e.to_string())
}

/// 更新提示词
#[tauri::command]
pub async fn update_prompt(
    db: State<'_, DbConnection>,
    prompt_key: String,
    system_prompt: Option<String>,
    user_prompt: String,
    enabled: Option<bool>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let update = PromptUpdate {
        system_prompt,
        user_prompt,
        enabled,
    };

    PromptDbService::update_prompt(&conn, &prompt_key, update).map_err(|e| e.to_string())
}

/// 重置单个提示词为默认值
#[tauri::command]
pub async fn reset_prompt(db: State<'_, DbConnection>, prompt_key: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PromptDbService::reset_prompt(&conn, &prompt_key).map_err(|e| e.to_string())
}

/// 重置所有提示词为默认值
#[tauri::command]
pub async fn reset_all_prompts(db: State<'_, DbConnection>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PromptDbService::reset_all_prompts(&conn).map_err(|e| e.to_string())
}

/// 渲染后的提示词响应（用于序列化）
#[derive(Debug, Clone, serde::Serialize)]
pub struct RenderedPromptResponse {
    pub system: Option<String>,
    pub user: String,
}

impl From<RenderedPrompt> for RenderedPromptResponse {
    fn from(rendered: RenderedPrompt) -> Self {
        RenderedPromptResponse {
            system: rendered.system,
            user: rendered.user,
        }
    }
}

/// 渲染提示词（预览功能）
#[tauri::command]
pub async fn render_prompt_preview(
    db: State<'_, DbConnection>,
    prompt_key: String,
    variables: HashMap<String, String>,
) -> Result<RenderedPromptResponse, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let rendered = PromptDbService::render_prompt(&conn, &prompt_key, &variables)
        .map_err(|e| e.to_string())?;
    Ok(rendered.into())
}

/// 获取提示词的可用变量
#[tauri::command]
pub async fn get_prompt_variables(db: State<'_, DbConnection>, prompt_key: String) -> Result<Vec<String>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PromptDbService::get_prompt_variables(&conn, &prompt_key).map_err(|e| e.to_string())
}

/// 刷新提示词缓存
#[tauri::command]
pub async fn refresh_prompt_cache(db: State<'_, DbConnection>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    PromptDbService::refresh_cache(&conn).map_err(|e| e.to_string())
}
