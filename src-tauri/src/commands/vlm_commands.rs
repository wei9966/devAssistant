use crate::db::connection::DbConnection;
use crate::services::vlm_service::{VlmConfig, VlmConfigResponse, VlmService};
use std::sync::Mutex;
use tauri::State;

/// VLM 状态管理
pub struct VlmState(pub Mutex<Option<VlmService>>);

impl VlmState {
    pub fn new() -> Self {
        VlmState(Mutex::new(None))
    }
}

/// 获取数据库路径
fn get_db_path() -> String {
    let db_path = dirs::data_local_dir()
        .expect("无法获取应用数据目录")
        .join("dev-assistant")
        .join("dev_assistant.db");
    db_path.to_string_lossy().to_string()
}

/// 保存 VLM 配置
#[tauri::command]
pub async fn vlm_save_config(
    provider: String,
    api_key: String,
    base_url: Option<String>,
    model: Option<String>,
    enabled: bool,
    max_image_size: Option<u32>,
    image_quality: Option<u32>,
    timeout: Option<u32>,
    max_tokens: Option<u32>,
    vlm_state: State<'_, VlmState>,
) -> Result<(), String> {
    let db_path = get_db_path();

    let config = VlmConfig {
        provider,
        api_key,
        base_url,
        model,
        enabled,
        max_image_size,
        image_quality,
        timeout,
        max_tokens,
    };

    let service = VlmService::new(db_path.clone());
    service.save_config(config).map_err(|e| e.to_string())?;

    // 更新状态
    let mut state = vlm_state.0.lock().unwrap();
    *state = Some(VlmService::new(db_path));

    Ok(())
}

/// 获取 VLM 配置
#[tauri::command]
pub async fn vlm_get_config() -> Result<Option<VlmConfigResponse>, String> {
    let db_path = get_db_path();
    let service = VlmService::new(db_path);
    service.get_config().map_err(|e| e.to_string())
}

/// 测试 VLM 连接
#[tauri::command]
pub async fn vlm_test_connection() -> Result<bool, String> {
    let db_path = get_db_path();
    let service = VlmService::new(db_path);
    service.test_connection().await.map_err(|e| e.to_string())
}

/// 分析图片
#[tauri::command]
pub async fn vlm_analyze_image(
    image_base64: String,
    prompt: String,
) -> Result<String, String> {
    let db_path = get_db_path();
    let service = VlmService::new(db_path);
    service
        .analyze_image(&image_base64, &prompt)
        .await
        .map_err(|e| e.to_string())
}

/// 检查 VLM 是否启用
#[tauri::command]
pub async fn vlm_is_enabled() -> Result<bool, String> {
    let db_path = get_db_path();
    let service = VlmService::new(db_path);
    service.is_enabled().map_err(|e| e.to_string())
}
