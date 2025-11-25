use tauri::State;
use crate::db::connection::DbConnection;
use crate::models::{
    AppItem, Category, Workflow, LaunchHistory,
    AppSearchParams, LaunchResult, WorkflowLaunchResult
};
use crate::services::{AppScannerService, AppLauncherService};

// ==================== 应用扫描 ====================

/// 扫描已安装的应用
#[tauri::command]
pub async fn scan_installed_apps(path: Option<String>) -> Result<Vec<AppItem>, String> {
    AppScannerService::scan_installed_apps(path).await
}

/// 手动添加应用
#[tauri::command]
pub async fn add_manual_app(path: String, name: Option<String>) -> Result<AppItem, String> {
    AppScannerService::add_manual_app(path, name).await
}

// ==================== 应用管理 ====================

/// 获取所有应用
#[tauri::command]
pub async fn get_all_apps(db: State<'_, DbConnection>) -> Result<Vec<AppItem>, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.get_all_apps().await
}

/// 根据ID获取应用
#[tauri::command]
pub async fn get_app_by_id(db: State<'_, DbConnection>, app_id: String) -> Result<AppItem, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.get_app_by_id(&app_id).await
}

/// 搜索应用
#[tauri::command]
pub async fn search_apps(
    db: State<'_, DbConnection>,
    params: AppSearchParams,
) -> Result<Vec<AppItem>, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.search_apps(params).await
}

/// 添加应用
#[tauri::command]
pub async fn add_app(
    db: State<'_, DbConnection>,
    app: AppItem,
) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.add_app(app).await
}

/// 更新应用
#[tauri::command]
pub async fn update_app(
    db: State<'_, DbConnection>,
    app: AppItem,
) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.update_app(app).await
}

/// 删除应用
#[tauri::command]
pub async fn delete_app(
    db: State<'_, DbConnection>,
    app_id: String,
) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.delete_app(&app_id).await
}

// ==================== 应用启动 ====================

/// 启动应用
#[tauri::command]
pub async fn launch_app(
    db: State<'_, DbConnection>,
    app_id: String,
) -> Result<LaunchResult, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.launch_app(&app_id).await
}

/// 批量启动应用
#[tauri::command]
pub async fn launch_apps(
    db: State<'_, DbConnection>,
    app_ids: Vec<String>,
    delay_ms: Option<i32>,
) -> Result<Vec<LaunchResult>, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);

    let mut results = Vec::new();

    for app_id in app_ids {
        let result = service.launch_app(&app_id).await.unwrap_or_else(|e| {
            LaunchResult::failure(app_id.clone(), app_id, e)
        });
        results.push(result);

        // 延迟
        if let Some(delay) = delay_ms {
            if delay > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay as u64)).await;
            }
        }
    }

    Ok(results)
}

/// 启动工作流
#[tauri::command]
pub async fn launch_workflow(
    db: State<'_, DbConnection>,
    workflow_id: String,
) -> Result<WorkflowLaunchResult, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.launch_workflow(&workflow_id).await
}

// ==================== 分类管理 ====================

/// 获取所有分类
#[tauri::command]
pub async fn get_categories(db: State<'_, DbConnection>) -> Result<Vec<Category>, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.get_all_categories().await
}

/// 添加分类
#[tauri::command]
pub async fn add_category(
    db: State<'_, DbConnection>,
    category: Category,
) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.add_category(category).await
}

/// 更新分类
#[tauri::command]
pub async fn update_category(
    db: State<'_, DbConnection>,
    category: Category,
) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.update_category(category).await
}

/// 保存分类（自动判断是添加还是更新）
#[tauri::command]
pub async fn save_category(
    db: State<'_, DbConnection>,
    category: Category,
) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);

    // 尝试获取现有分类，如果存在则更新，否则添加
    match service.get_category_by_id(&category.id).await {
        Ok(_) => service.update_category(category).await,
        Err(_) => service.add_category(category).await,
    }
}

/// 删除分类
#[tauri::command]
pub async fn delete_category(
    db: State<'_, DbConnection>,
    id: String,
) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.delete_category(&id).await
}

// ==================== 工作流管理 ====================

/// 获取所有工作流
#[tauri::command]
pub async fn get_workflows(db: State<'_, DbConnection>) -> Result<Vec<Workflow>, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.get_all_workflows().await
}

/// 根据ID获取工作流
#[tauri::command]
pub async fn get_workflow_by_id(
    db: State<'_, DbConnection>,
    workflow_id: String,
) -> Result<Workflow, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.get_workflow_by_id(&workflow_id).await
}

/// 添加工作流
#[tauri::command]
pub async fn add_workflow(
    db: State<'_, DbConnection>,
    workflow: Workflow,
) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.add_workflow(workflow).await
}

/// 更新工作流
#[tauri::command]
pub async fn update_workflow(
    db: State<'_, DbConnection>,
    workflow: Workflow,
) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.update_workflow(workflow).await
}

/// 删除工作流
#[tauri::command]
pub async fn delete_workflow(
    db: State<'_, DbConnection>,
    workflow_id: String,
) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.delete_workflow(&workflow_id).await
}

// ==================== 启动历史 ====================

/// 获取启动历史
#[tauri::command]
pub async fn get_launch_history(
    db: State<'_, DbConnection>,
    limit: Option<usize>,
) -> Result<Vec<LaunchHistory>, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.get_launch_history(limit.unwrap_or(50)).await
}

/// 清除启动历史
#[tauri::command]
pub async fn clear_launch_history(
    db: State<'_, DbConnection>,
) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.clear_launch_history().await
}

// ==================== 辅助功能 ====================

/// 切换应用置顶状态
#[tauri::command]
pub async fn toggle_app_pin(
    db: State<'_, DbConnection>,
    app_id: String,
) -> Result<bool, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);

    let mut app = service.get_app_by_id(&app_id).await?;
    app.is_pinned = !app.is_pinned;
    service.update_app(app.clone()).await?;

    Ok(app.is_pinned)
}

/// 切换应用隐藏状态
#[tauri::command]
pub async fn toggle_app_hidden(
    db: State<'_, DbConnection>,
    app_id: String,
) -> Result<bool, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);

    let mut app = service.get_app_by_id(&app_id).await?;
    app.is_hidden = !app.is_hidden;
    service.update_app(app.clone()).await?;

    Ok(app.is_hidden)
}

/// 验证路径是否有效
#[tauri::command]
pub fn validate_path(path: String) -> Result<bool, String> {
    use std::path::Path;
    Ok(Path::new(&path).exists())
}

/// 导出配置（JSON格式）
#[tauri::command]
pub async fn export_config(db: State<'_, DbConnection>) -> Result<String, String> {
    use serde_json::json;

    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);

    let apps = service.get_all_apps().await?;
    let categories = service.get_all_categories().await?;
    let workflows = service.get_all_workflows().await?;

    let config = json!({
        "version": "1.0",
        "exported_at": chrono::Utc::now().timestamp(),
        "apps": apps,
        "categories": categories,
        "workflows": workflows,
    });

    serde_json::to_string_pretty(&config).map_err(|e| e.to_string())
}

/// 导入配置（JSON格式）
#[tauri::command]
pub async fn import_config(
    db: State<'_, DbConnection>,
    config_json: String,
) -> Result<(), String> {
    use serde_json::Value;

    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);

    let config: Value = serde_json::from_str(&config_json).map_err(|e| e.to_string())?;

    // 导入分类
    if let Some(categories) = config["categories"].as_array() {
        for category_val in categories {
            if let Ok(category) = serde_json::from_value::<Category>(category_val.clone()) {
                let _ = service.add_category(category).await;
            }
        }
    }

    // 导入应用
    if let Some(apps) = config["apps"].as_array() {
        for app_val in apps {
            if let Ok(app) = serde_json::from_value::<AppItem>(app_val.clone()) {
                let _ = service.add_app(app).await;
            }
        }
    }

    // 导入工作流
    if let Some(workflows) = config["workflows"].as_array() {
        for workflow_val in workflows {
            if let Ok(workflow) = serde_json::from_value::<Workflow>(workflow_val.clone()) {
                let _ = service.add_workflow(workflow).await;
            }
        }
    }

    Ok(())
}

// ==================== 图标管理 ====================

/// 刷新所有应用的图标
#[tauri::command]
pub async fn refresh_all_icons(db: State<'_, DbConnection>) -> Result<u32, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.refresh_all_icons().await
}

/// 更新应用的自定义图标
#[tauri::command]
pub async fn update_app_icon(
    db: State<'_, DbConnection>,
    app_id: String,
    icon_data: Option<String>,
) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.update_app_icon(&app_id, icon_data).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path() {
        // 测试当前目录
        let result = validate_path(".".to_string());
        assert!(result.is_ok());
        assert!(result.unwrap());

        // 测试不存在的路径
        let result = validate_path("C:\\this\\path\\should\\not\\exist\\12345".to_string());
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
