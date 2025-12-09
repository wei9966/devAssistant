use crate::db::connection::DbConnection;
use crate::models::{
    AppItem, AppLauncherSettings, AppSearchParams, Category, LaunchHistory, LaunchResult, Workflow,
    WorkflowLaunchResult, SyncResult,
};
use crate::services::{AppLauncherService, AppScannerService};
use tauri::State;
use std::collections::HashMap;

// ==================== 应用扫描 ====================

/// 扫描已安装的应用
#[tauri::command]
pub async fn scan_installed_apps(
    db: State<'_, DbConnection>,
    path: Option<String>,
) -> Result<Vec<AppItem>, String> {
    // 从数据库读取用户配置的扩展名设置
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT allowed_extensions FROM app_launcher_settings WHERE id = 1",
            [],
            |row| {
                let extensions_json: String = row.get(0)?;
                let allowed_extensions: Vec<String> =
                    serde_json::from_str(&extensions_json).unwrap_or_else(|_| vec!["exe".to_string(), "lnk".to_string()]);
                Ok(AppLauncherSettings { allowed_extensions })
            },
        ).unwrap_or_else(|_| AppLauncherSettings::default())
    };

    AppScannerService::scan_installed_apps_with_settings(path, settings).await
}

/// 手动添加应用
#[tauri::command]
pub async fn add_manual_app(path: String, name: Option<String>) -> Result<AppItem, String> {
    AppScannerService::add_manual_app(path, name, None).await
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

/// 添加应用（返回保存后的完整应用数据，包含自动提取的图标）
#[tauri::command]
pub async fn add_app(db: State<'_, DbConnection>, app: AppItem) -> Result<AppItem, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.add_app(app).await
}

/// 更新应用
#[tauri::command]
pub async fn update_app(db: State<'_, DbConnection>, app: AppItem) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.update_app(app).await
}

/// 删除应用
#[tauri::command]
pub async fn delete_app(db: State<'_, DbConnection>, app_id: String) -> Result<(), String> {
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
        let result = service
            .launch_app(&app_id)
            .await
            .unwrap_or_else(|e| LaunchResult::failure(app_id.clone(), app_id, e));
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
pub async fn add_category(db: State<'_, DbConnection>, category: Category) -> Result<(), String> {
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
pub async fn save_category(db: State<'_, DbConnection>, category: Category) -> Result<(), String> {
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
pub async fn delete_category(db: State<'_, DbConnection>, id: String) -> Result<(), String> {
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
pub async fn add_workflow(db: State<'_, DbConnection>, workflow: Workflow) -> Result<(), String> {
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

/// 保存工作流（自动判断是添加还是更新）
#[tauri::command]
pub async fn save_workflow(db: State<'_, DbConnection>, workflow: Workflow) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);

    // 尝试获取现有工作流，如果存在则更新，否则添加
    match service.get_workflow_by_id(&workflow.id).await {
        Ok(_) => service.update_workflow(workflow).await,
        Err(_) => service.add_workflow(workflow).await,
    }
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
pub async fn clear_launch_history(db: State<'_, DbConnection>) -> Result<(), String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn);
    service.clear_launch_history().await
}

// ==================== 辅助功能 ====================

/// 切换应用置顶状态
#[tauri::command]
pub async fn toggle_app_pin(db: State<'_, DbConnection>, app_id: String) -> Result<bool, String> {
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

/// 在文件管理器中显示文件
#[tauri::command]
pub fn show_in_folder(path: String) -> Result<(), String> {
    use std::path::Path;
    use std::process::Command;
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;

    // 处理路径：去除引号，处理特殊路径格式
    let clean_path = path.trim().trim_matches('"').to_string();

    // 检查是否是 shell: 协议路径（如 shell:AppsFolder\xxx）
    if clean_path.starts_with("shell:") {
        // 对于 shell: 路径，直接用 explorer 打开
        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .arg(&clean_path)
                .spawn()
                .map_err(|e| format!("无法打开: {}", e))?;
        }
        return Ok(());
    }

    let file_path = Path::new(&clean_path);

    // 如果路径不存在，尝试打开父目录
    if !file_path.exists() {
        // 尝试获取父目录
        if let Some(parent) = file_path.parent() {
            if parent.exists() {
                #[cfg(target_os = "windows")]
                {
                    Command::new("explorer")
                        .arg(parent)
                        .spawn()
                        .map_err(|e| format!("无法打开文件管理器: {}", e))?;
                }
                return Ok(());
            }
        }
        return Err(format!("路径不存在: {}", clean_path));
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 explorer 命令并选中文件
        // 注意：路径需要用引号包裹以处理空格
        let arg = if file_path.is_dir() {
            // 如果是目录，直接打开该目录
            clean_path.clone()
        } else {
            // 如果是文件，打开所在目录并选中该文件
            format!("/select,\"{}\"", clean_path)
        };

        Command::new("explorer")
            .raw_arg(&arg)
            .spawn()
            .map_err(|e| format!("无法打开文件管理器: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-R")
            .arg(&clean_path)
            .spawn()
            .map_err(|e| format!("无法打开访达: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 尝试使用 xdg-open 打开父目录
        let parent = file_path.parent().unwrap_or(file_path);
        Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| format!("无法打开文件管理器: {}", e))?;
    }

    Ok(())
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
        "exported_at": chrono::Local::now().timestamp(),
        "apps": apps,
        "categories": categories,
        "workflows": workflows,
    });

    serde_json::to_string_pretty(&config).map_err(|e| e.to_string())
}

/// 导入配置（JSON格式）
#[tauri::command]
pub async fn import_config(db: State<'_, DbConnection>, config_json: String) -> Result<(), String> {
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

// ==================== 应用启动器设置 ====================

/// 获取应用启动器设置
#[tauri::command]
pub async fn get_launcher_settings(db: State<'_, DbConnection>) -> Result<AppLauncherSettings, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 查询设置
    let result = conn.query_row(
        "SELECT allowed_extensions FROM app_launcher_settings WHERE id = 1",
        [],
        |row| {
            let extensions_json: String = row.get(0)?;
            let allowed_extensions: Vec<String> =
                serde_json::from_str(&extensions_json).unwrap_or_else(|_| vec!["exe".to_string(), "lnk".to_string()]);
            Ok(AppLauncherSettings { allowed_extensions })
        },
    );

    match result {
        Ok(settings) => Ok(settings),
        Err(_) => {
            // 如果查询失败,返回默认设置
            Ok(AppLauncherSettings::default())
        }
    }
}

/// 更新应用启动器设置
#[tauri::command]
pub async fn update_launcher_settings(
    db: State<'_, DbConnection>,
    settings: AppLauncherSettings,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().timestamp();

    // 只序列化数组部分，保持与数据库初始化格式一致
    let extensions_json = serde_json::to_string(&settings.allowed_extensions)
        .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE app_launcher_settings SET allowed_extensions = ?1, updated_at = ?2 WHERE id = 1",
        rusqlite::params![extensions_json, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ==================== 完整扫描与同步 ====================

/// 完整扫描所有来源的应用
/// 扫描开始菜单、注册表、shell:AppsFolder等所有来源，返回合并去重后的完整应用列表
#[tauri::command]
pub async fn full_scan_apps() -> Result<Vec<AppItem>, String> {
    // 调用 AppScannerService 扫描所有来源
    // 这会扫描：开始菜单、常见安装目录、注册表、shell:AppsFolder
    AppScannerService::scan_installed_apps(None).await
}

/// 同步扫描结果到数据库
/// 将扫描到的应用列表与数据库中的应用进行对比，执行增删改操作
#[tauri::command]
pub async fn sync_apps_to_db(
    db: State<'_, DbConnection>,
    apps: Vec<AppItem>,
) -> Result<SyncResult, String> {
    let conn = db.0.clone();
    let service = AppLauncherService::new(conn.clone());

    let mut result = SyncResult::new();

    // 获取数据库中现有的所有应用
    let existing_apps = service.get_all_apps().await?;

    // 创建现有应用的映射表（使用路径作为唯一标识）
    let mut existing_map: HashMap<String, AppItem> = existing_apps
        .into_iter()
        .map(|app| (app.path.clone(), app))
        .collect();

    // 创建扫描应用的映射表
    let scanned_map: HashMap<String, AppItem> = apps
        .iter()
        .map(|app| (app.path.clone(), app.clone()))
        .collect();

    // 遍历扫描到的应用，判断是新增还是更新
    for scanned_app in apps {
        if let Some(existing_app) = existing_map.remove(&scanned_app.path) {
            // 应用已存在，检查是否需要更新
            // 保留用户自定义的数据（如分类、标签、置顶、隐藏状态等）
            let mut updated_app = scanned_app.clone();
            updated_app.id = existing_app.id.clone();
            updated_app.category = existing_app.category.clone();
            updated_app.tags = existing_app.tags.clone();
            updated_app.is_pinned = existing_app.is_pinned;
            updated_app.is_hidden = existing_app.is_hidden;
            updated_app.launch_count = existing_app.launch_count;
            updated_app.last_launched_at = existing_app.last_launched_at;
            updated_app.launch_args = existing_app.launch_args.clone();
            updated_app.created_at = existing_app.created_at;

            // 如果用户设置了自定义图标，保留它
            if existing_app.icon.is_some() && !existing_app.icon.as_ref().unwrap().is_empty() {
                updated_app.icon = existing_app.icon.clone();
            }

            // 检查应用名称或路径是否发生变化
            if updated_app.name != existing_app.name || updated_app.path != existing_app.path {
                service.update_app(updated_app).await?;
                result.updated += 1;
            } else {
                result.unchanged += 1;
            }
        } else {
            // 新应用，添加到数据库
            service.add_app(scanned_app).await?;
            result.added += 1;
        }
    }

    // 处理数据库中剩余的应用（这些应用在扫描结果中不存在）
    // 选项1: 删除这些应用
    // 选项2: 标记为不可用（推荐，因为可能是用户手动添加的）
    for (path, existing_app) in existing_map {
        // 如果是用户手动添加的特殊类型（如文件夹、URL），不删除
        if !scanned_map.contains_key(&path) {
            // 检查文件是否仍然存在
            if !std::path::Path::new(&path).exists() {
                // 文件不存在，标记为隐藏而不是删除
                let mut hidden_app = existing_app.clone();
                hidden_app.is_hidden = true;
                service.update_app(hidden_app).await?;
                result.removed += 1;
            } else {
                // 文件存在但未被扫描到，保持不变
                result.unchanged += 1;
            }
        }
    }

    Ok(result)
}

/// 一键扫描并同步应用
/// 组合命令：先执行完整扫描，然后同步到数据库
#[tauri::command]
pub async fn scan_and_sync_apps(db: State<'_, DbConnection>) -> Result<SyncResult, String> {
    // 执行完整扫描
    let apps = full_scan_apps().await?;

    // 同步到数据库
    sync_apps_to_db(db, apps).await
}

// ==================== 应用监控（预留接口）====================
// 注意：AppMonitorService 需要在 services 模块中实现
// 这里提供命令接口定义，具体实现可以后续补充

/// 启动应用监控服务
/// 监控系统应用的安装和卸载，自动同步到数据库
#[tauri::command]
pub async fn start_app_monitor(
    _app_handle: tauri::AppHandle,
    _db: State<'_, DbConnection>,
) -> Result<(), String> {
    // TODO: 实现 AppMonitorService
    // let monitor = app_handle.state::<AppMonitorState>();
    // monitor.start().await
    Err("应用监控服务尚未实现".to_string())
}

/// 停止应用监控服务
#[tauri::command]
pub async fn stop_app_monitor() -> Result<(), String> {
    // TODO: 实现 AppMonitorService
    // let monitor = app_handle.state::<AppMonitorState>();
    // monitor.stop().await
    Err("应用监控服务尚未实现".to_string())
}

/// 获取应用监控状态
#[tauri::command]
pub async fn get_monitor_status() -> Result<bool, String> {
    // TODO: 实现 AppMonitorService
    // let monitor = app_handle.state::<AppMonitorState>();
    // Ok(monitor.is_running().await)
    Ok(false)
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
