use crate::services::tool_service::{PortInfo, ToolService};
use tauri::{AppHandle, Manager};

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
