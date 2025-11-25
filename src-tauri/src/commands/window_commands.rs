use tauri::{AppHandle, Emitter, Manager};

/// 显示窗口并聚焦
#[tauri::command]
pub fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("找不到主窗口".to_string())
    }
}

/// 隐藏窗口
#[tauri::command]
pub fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("找不到主窗口".to_string())
    }
}

/// 切换窗口显示状态
#[tauri::command]
pub fn toggle_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().map_err(|e| e.to_string())? {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
        Ok(())
    } else {
        Err("找不到主窗口".to_string())
    }
}

/// 显示窗口并导航到指定路由
#[tauri::command]
pub fn show_window_with_route(app: AppHandle, route: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;

        // 触发前端事件,让前端导航到指定路由
        window
            .emit("navigate-to", route)
            .map_err(|e| e.to_string())?;

        Ok(())
    } else {
        Err("找不到主窗口".to_string())
    }
}

/// 设置窗口置顶状态
#[tauri::command]
pub fn set_always_on_top(app: AppHandle, always_on_top: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_always_on_top(always_on_top)
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("找不到主窗口".to_string())
    }
}

/// 获取窗口置顶状态
#[tauri::command]
pub fn get_always_on_top(app: AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("main") {
        window.is_always_on_top().map_err(|e| e.to_string())
    } else {
        Err("找不到主窗口".to_string())
    }
}

/// 切换窗口置顶状态
#[tauri::command]
pub fn toggle_always_on_top(app: AppHandle) -> Result<bool, String> {
    let current = get_always_on_top(app.clone())?;
    set_always_on_top(app, !current)?;
    Ok(!current)
}
