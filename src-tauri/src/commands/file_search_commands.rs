use crate::services::file_search_service::{FileSearchResult, FileSearchService};

/// 搜索文件
///
/// # Arguments
/// * `keyword` - 搜索关键词
/// * `search_paths` - 搜索路径列表（可选，默认搜索所有驱动器）
/// * `max_results` - 最大结果数量（可选，默认 100）
///
/// # Returns
/// 返回匹配的文件列表
///
/// # Example
/// ```javascript
/// // 搜索所有驱动器
/// const results = await invoke('search_files', { keyword: 'document' });
///
/// // 在指定路径中搜索
/// const results = await invoke('search_files', {
///   keyword: 'readme',
///   searchPaths: ['C:\\Users', 'D:\\Projects'],
///   maxResults: 50
/// });
/// ```
#[tauri::command]
pub async fn search_files(
    keyword: String,
    search_paths: Option<Vec<String>>,
    max_results: Option<usize>,
) -> Result<Vec<FileSearchResult>, String> {
    // 空关键词检查
    if keyword.trim().is_empty() {
        return Err("搜索关键词不能为空".to_string());
    }

    // 在异步上下文中执行阻塞操作
    tokio::task::spawn_blocking(move || {
        FileSearchService::search_files(&keyword, search_paths, max_results)
    })
    .await
    .map_err(|e| format!("搜索任务执行失败: {}", e))?
}

/// 使用系统默认程序打开文件
///
/// # Arguments
/// * `path` - 文件路径
///
/// # Returns
/// 成功返回 Ok(())，失败返回错误信息
///
/// # Example
/// ```javascript
/// await invoke('open_file', { path: 'C:\\Users\\Documents\\file.txt' });
/// ```
#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    FileSearchService::open_file(&path)
}

/// 在文件管理器中显示文件
///
/// # Arguments
/// * `path` - 文件路径
///
/// # Returns
/// 成功返回 Ok(())，失败返回错误信息
///
/// # Example
/// ```javascript
/// await invoke('open_file_in_folder', { path: 'C:\\Users\\Documents\\file.txt' });
/// ```
#[tauri::command]
pub fn open_file_in_folder(path: String) -> Result<(), String> {
    FileSearchService::open_in_folder(&path)
}

/// 获取所有驱动器
///
/// # Returns
/// 返回所有可用驱动器的路径列表
///
/// # Example
/// ```javascript
/// const drives = await invoke('get_all_drives');
/// console.log('可用驱动器:', drives);
/// // Windows 输出示例: ['C:\\', 'D:\\', 'E:\\']
/// ```
#[tauri::command]
pub fn get_all_drives() -> Vec<String> {
    FileSearchService::get_all_drives()
}
