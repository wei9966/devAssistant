use crate::models::ToolItem;
use crate::services::tool_service::{PortInfo, ToolService};
use tauri::{AppHandle, Emitter, Manager, State};
use std::sync::Mutex;

/// 检查端口占用情况
///
/// # Arguments
/// * `port` - 要检查的端口号
///
/// # Returns
/// 返回占用该端口的进程信息列表
///
/// # Example
/// ```javascript
/// const portInfos = await invoke('check_port_usage', { port: 8080 });
/// console.log('端口占用信息:', portInfos);
/// ```
#[tauri::command]
pub async fn check_port_usage(port: u16) -> Result<Vec<PortInfo>, String> {
    ToolService::check_port_usage(port).await
}

/// 杀掉指定进程
///
/// # Arguments
/// * `pid` - 进程ID
///
/// # Returns
/// 返回是否成功杀掉进程
///
/// # Example
/// ```javascript
/// const success = await invoke('kill_process_by_pid', { pid: 1234 });
/// if (success) {
///   console.log('进程已终止');
/// }
/// ```
#[tauri::command]
pub async fn kill_process_by_pid(pid: u32) -> Result<bool, String> {
    ToolService::kill_process_by_pid(pid).await
}

/// 打开工具窗口
///
/// # Arguments
/// * `tool_id` - 工具ID，用于确定打开哪个工具窗口
///
/// # Example
/// ```javascript
/// await invoke('open_tool_window', { toolId: 'port-checker' });
/// ```
#[tauri::command]
pub fn open_tool_window(app: AppHandle, tool_id: String) -> Result<(), String> {
    // 根据 tool_id 确定窗口标签
    let window_label = match tool_id.as_str() {
        "port-checker" => "tool-port-checker",
        _ => return Err(format!("未知的工具ID: {}", tool_id)),
    };

    // 获取并显示对应的窗口
    if let Some(window) = app.get_webview_window(window_label) {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err(format!("找不到工具窗口: {}", window_label))
    }
}

/// 获取所有工具列表
///
/// # Returns
/// 返回所有已注册的工具列表
///
/// # Example
/// ```javascript
/// const tools = await invoke('get_all_tools');
/// console.log('所有工具:', tools);
/// ```
#[tauri::command]
pub fn get_all_tools(tool_service: State<Mutex<ToolService>>) -> Result<Vec<ToolItem>, String> {
    let service = tool_service.lock().unwrap();
    Ok(service.get_all_tools())
}

/// 根据ID获取工具
///
/// # Arguments
/// * `tool_id` - 工具ID
///
/// # Returns
/// 返回工具详情，如果不存在则返回 None
///
/// # Example
/// ```javascript
/// const tool = await invoke('get_tool_by_id', { toolId: 'port-checker' });
/// if (tool) {
///   console.log('工具信息:', tool);
/// }
/// ```
#[tauri::command]
pub fn get_tool_by_id(
    tool_service: State<Mutex<ToolService>>,
    tool_id: String,
) -> Result<Option<ToolItem>, String> {
    let service = tool_service.lock().unwrap();
    Ok(service.get_tool_by_id(&tool_id))
}

/// 设置工具固定状态
///
/// # Arguments
/// * `tool_id` - 工具ID
/// * `pinned` - 是否固定
///
/// # Returns
/// 返回是否成功设置
///
/// # Example
/// ```javascript
/// await invoke('set_tool_pinned', { toolId: 'port-checker', pinned: true });
/// ```
#[tauri::command]
pub fn set_tool_pinned(
    tool_service: State<Mutex<ToolService>>,
    tool_id: String,
    pinned: bool,
) -> Result<bool, String> {
    let service = tool_service.lock().unwrap();
    if pinned {
        service.pin_tool(tool_id)
    } else {
        service.unpin_tool(&tool_id)
    }
}

/// 固定工具
///
/// # Arguments
/// * `tool_id` - 工具ID
///
/// # Returns
/// 返回是否成功固定
///
/// # Example
/// ```javascript
/// const success = await invoke('pin_tool', { toolId: 'port-checker' });
/// if (success) {
///   console.log('工具已固定');
/// }
/// ```
#[tauri::command]
pub fn pin_tool(
    tool_service: State<Mutex<ToolService>>,
    tool_id: String,
) -> Result<bool, String> {
    let service = tool_service.lock().unwrap();
    service.pin_tool(tool_id)
}

/// 取消固定工具
///
/// # Arguments
/// * `tool_id` - 工具ID
///
/// # Returns
/// 返回是否成功取消固定
///
/// # Example
/// ```javascript
/// const success = await invoke('unpin_tool', { toolId: 'port-checker' });
/// if (success) {
///   console.log('已取消固定');
/// }
/// ```
#[tauri::command]
pub fn unpin_tool(
    tool_service: State<Mutex<ToolService>>,
    tool_id: String,
) -> Result<bool, String> {
    let service = tool_service.lock().unwrap();
    service.unpin_tool(&tool_id)
}

/// 获取固定的工具列表
///
/// # Returns
/// 返回固定的工具列表
///
/// # Example
/// ```javascript
/// const pinnedTools = await invoke('get_pinned_tools');
/// console.log('固定的工具:', pinnedTools);
/// ```
#[tauri::command]
pub fn get_pinned_tools(
    tool_service: State<Mutex<ToolService>>,
) -> Result<Vec<ToolItem>, String> {
    let service = tool_service.lock().unwrap();
    Ok(service.get_pinned_tools())
}

/// 记录工具使用
///
/// # Arguments
/// * `tool_id` - 工具ID
///
/// # Returns
/// 返回是否成功记录
///
/// # Example
/// ```javascript
/// await invoke('record_tool_usage', { toolId: 'port-checker' });
/// ```
#[tauri::command]
pub fn record_tool_usage(
    tool_service: State<Mutex<ToolService>>,
    tool_id: String,
) -> Result<bool, String> {
    let service = tool_service.lock().unwrap();
    service.record_tool_usage(tool_id)
}

/// 获取最近使用的工具
///
/// # Arguments
/// * `limit` - 返回的最大数量
///
/// # Returns
/// 返回最近使用的工具列表
///
/// # Example
/// ```javascript
/// const recentTools = await invoke('get_recent_tools', { limit: 5 });
/// console.log('最近使用的工具:', recentTools);
/// ```
#[tauri::command]
pub fn get_recent_tools(
    tool_service: State<Mutex<ToolService>>,
    limit: usize,
) -> Result<Vec<ToolItem>, String> {
    let service = tool_service.lock().unwrap();
    Ok(service.get_recent_tools(limit))
}

/// 打开工具容器窗口
///
/// # Arguments
/// * `tool_id` - 工具ID
///
/// # Returns
/// 返回是否成功打开窗口
///
/// # Example
/// ```javascript
/// await invoke('open_tool_container', { toolId: 'port-checker' });
/// ```
#[tauri::command]
pub fn open_tool_container(app: AppHandle, tool_id: String) -> Result<(), String> {
    // 获取或创建工具容器窗口
    if let Some(window) = app.get_webview_window("tool-container") {
        // 更新窗口 URL 参数
        // 注意: Tauri 不支持直接导航，这里只是显示和聚焦窗口
        // URL 参数应该在前端通过 window.location 或路由处理
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;

        // 发送事件到前端，通知更新工具ID
        window.emit("tool-container:load-tool", tool_id).map_err(|e| e.to_string())?;

        Ok(())
    } else {
        Err("工具容器窗口不存在".to_string())
    }
}
