use crate::services::{
    file_index_service::{FileIndexRecord, FileIndexService, IndexStats},
    file_watcher::{FileChangeEvent, FileWatcher},
    mft_reader::MftReader,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

#[cfg(windows)]
use windows::Win32::System::Threading::{
    GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_BELOW_NORMAL,
};

/// 索引状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStatus {
    /// 是否正在索引
    pub is_indexing: bool,
    /// 是否正在监控文件变化
    pub is_watching: bool,
    /// 是否有管理员权限
    pub has_admin_privilege: bool,
    /// 索引统计
    pub stats: Option<IndexStats>,
    /// 当前索引进度消息
    pub progress_message: Option<String>,
}

/// 索引进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexProgressEvent {
    /// 当前阶段
    pub stage: String,
    /// 进度百分比 (0-100)
    pub progress: u32,
    /// 消息
    pub message: String,
    /// 已处理数量
    pub processed: usize,
    /// 总数量
    pub total: usize,
}

/// 从路径中提取有效的驱动器字母
///
/// 验证路径格式是否为有效的 Windows 路径（如 `C:\...`）
/// 只接受 A-Z 的驱动器字母
///
/// # Arguments
/// * `path` - 文件路径字符串
///
/// # Returns
/// * `Some(String)` - 驱动器标识（如 "C:"）
/// * `None` - 如果路径不是有效的 Windows 驱动器路径
fn extract_drive_letter(path: &str) -> Option<String> {
    let chars: Vec<char> = path.chars().take(3).collect();

    // 检查路径格式：至少需要 3 个字符，格式为 "X:\" 或 "X:/"
    if chars.len() >= 2 {
        let first_char = chars[0];
        let second_char = chars[1];

        // 验证第一个字符是有效的驱动器字母 (A-Z 或 a-z)
        if first_char.is_ascii_alphabetic() && second_char == ':' {
            // 返回大写的驱动器字母
            return Some(format!("{}:", first_char.to_ascii_uppercase()));
        }
    }

    None
}

/// 文件索引状态管理
pub struct FileIndexState {
    /// 索引服务
    service: FileIndexService,
    /// 文件监控器
    watcher: Arc<Mutex<Option<FileWatcher>>>,
    /// 是否正在索引
    is_indexing: Arc<Mutex<bool>>,
    /// 进度消息
    progress_message: Arc<Mutex<Option<String>>>,
}

impl FileIndexState {
    /// 创建新的状态实例
    pub fn new() -> Self {
        let db_path = Self::get_index_db_path();
        let service = FileIndexService::new(db_path);

        // 初始化数据库
        if let Err(e) = service.init_db() {
            log::error!("初始化文件索引数据库失败: {}", e);
        }

        Self {
            service,
            watcher: Arc::new(Mutex::new(None)),
            is_indexing: Arc::new(Mutex::new(false)),
            progress_message: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取索引数据库路径
    fn get_index_db_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("dev-assistant")
            .join("file_index.db")
    }
}

impl Default for FileIndexState {
    fn default() -> Self {
        Self::new()
    }
}

/// 检查管理员权限
///
/// NTFS MFT 读取需要管理员权限
///
/// # Returns
/// 返回是否有管理员权限
#[tauri::command]
pub fn check_file_index_admin_privilege() -> bool {
    #[cfg(windows)]
    {
        MftReader::check_admin_privilege()
    }

    #[cfg(not(windows))]
    {
        // 非 Windows 系统不需要特殊权限
        true
    }
}

/// 获取索引状态
///
/// # Returns
/// 返回当前索引状态信息
#[tauri::command]
pub async fn get_file_index_status(
    state: State<'_, FileIndexState>,
) -> Result<IndexStatus, String> {
    let is_indexing = *state.is_indexing.lock().await;
    let watcher = state.watcher.lock().await;
    let is_watching = watcher.as_ref().map(|w| w.is_running()).unwrap_or(false);
    let progress_message = state.progress_message.lock().await.clone();

    let stats = state.service.get_stats().ok();

    Ok(IndexStatus {
        is_indexing,
        is_watching,
        has_admin_privilege: check_file_index_admin_privilege(),
        stats,
        progress_message,
    })
}

/// 开始建立文件索引
///
/// 使用 NTFS MFT 快速读取所有文件，建立搜索索引
///
/// # Arguments
/// * `drives` - 要索引的驱动器列表，如 ["C", "D"]，None 表示所有驱动器
///
/// # Returns
/// 返回是否成功开始索引
#[tauri::command]
pub async fn start_file_indexing(
    app: AppHandle,
    state: State<'_, FileIndexState>,
    drives: Option<Vec<String>>,
) -> Result<bool, String> {
    // 检查是否已在索引中
    {
        let mut is_indexing = state.is_indexing.lock().await;
        if *is_indexing {
            return Err("索引正在进行中".to_string());
        }
        *is_indexing = true;
    }

    // 检查管理员权限
    #[cfg(windows)]
    if !MftReader::check_admin_privilege() {
        let mut is_indexing = state.is_indexing.lock().await;
        *is_indexing = false;
        return Err("需要管理员权限才能读取 NTFS MFT，请以管理员身份运行程序".to_string());
    }

    let service = state.service.clone();
    let is_indexing_flag = state.is_indexing.clone();
    let progress_message = state.progress_message.clone();

    // 在后台线程执行索引
    tokio::task::spawn_blocking(move || {
        let runtime = tokio::runtime::Handle::current();

        // 确保数据库表存在（可能被删除或首次创建）
        if let Err(e) = service.init_db() {
            log::error!("初始化文件索引数据库失败: {}", e);
            runtime.block_on(async {
                let mut is_indexing = is_indexing_flag.lock().await;
                *is_indexing = false;
            });
            return Err(format!("初始化数据库失败: {}", e));
        }
        log::info!("数据库表已确认存在");

        // 设置线程为低优先级，减少对用户操作的影响
        #[cfg(windows)]
        {
            unsafe {
                let thread = GetCurrentThread();
                let _ = SetThreadPriority(thread, THREAD_PRIORITY_BELOW_NORMAL);
            }
            log::debug!("索引线程已设置为低优先级");
        }

        // 发送进度事件
        let emit_progress = |stage: &str, progress: u32, message: &str, processed: usize, total: usize| {
            let event = IndexProgressEvent {
                stage: stage.to_string(),
                progress,
                message: message.to_string(),
                processed,
                total,
            };
            let _ = app.emit("file-index:progress", event);
        };

        // 更新进度消息
        runtime.block_on(async {
            let mut msg = progress_message.lock().await;
            *msg = Some("正在读取文件系统...".to_string());
        });

        emit_progress("reading", 5, "正在检测可用驱动器...", 0, 0);

        // 读取 MFT - 使用多线程并行读取多个驱动器
        #[cfg(windows)]
        let files: Vec<crate::services::mft_reader::MftFileInfo> = {
            use rayon::prelude::*;
            use std::sync::atomic::{AtomicUsize, Ordering};

            // 获取要读取的驱动器列表
            let drive_letters: Vec<char> = if let Some(ref specified_drives) = drives {
                specified_drives.iter().filter_map(|d| d.chars().next()).collect()
            } else {
                // 检测所有可用的 NTFS 驱动器
                ('C'..='Z')
                    .filter(|c| {
                        let path = format!("{}:\\", c);
                        std::path::Path::new(&path).exists()
                    })
                    .collect()
            };

            let total_drives = drive_letters.len();
            if total_drives == 0 {
                runtime.block_on(async {
                    let mut is_indexing = is_indexing_flag.lock().await;
                    *is_indexing = false;
                });
                let _ = app.emit("file-index:error", "未找到可用驱动器".to_string());
                return Err("未找到可用驱动器".to_string());
            }

            log::info!("准备并行读取 {} 个驱动器: {:?}", total_drives, drive_letters);
            emit_progress("reading", 10, &format!("正在并行读取 {} 个驱动器...", total_drives), 0, total_drives);

            // 使用原子计数器跟踪进度
            let completed_drives = AtomicUsize::new(0);

            // 并行读取所有驱动器
            let all_results: Vec<Vec<crate::services::mft_reader::MftFileInfo>> = drive_letters
                .par_iter()
                .map(|drive_char| {
                    match MftReader::read_drive(*drive_char) {
                        Ok(files) => {
                            let count = completed_drives.fetch_add(1, Ordering::SeqCst) + 1;
                            log::info!("驱动器 {}: 读取到 {} 个文件 ({}/{})", drive_char, files.len(), count, total_drives);
                            files
                        }
                        Err(e) => {
                            completed_drives.fetch_add(1, Ordering::SeqCst);
                            log::warn!("读取驱动器 {} 失败: {}", drive_char, e);
                            Vec::new()
                        }
                    }
                })
                .collect();

            // 合并所有结果
            let total_files: usize = all_results.iter().map(|v| v.len()).sum();
            log::info!("所有驱动器读取完成，共 {} 个文件", total_files);

            all_results.into_iter().flatten().collect()
        };

        #[cfg(not(windows))]
        let files = Vec::new();

        if files.is_empty() {
            runtime.block_on(async {
                let mut is_indexing = is_indexing_flag.lock().await;
                *is_indexing = false;
                let mut msg = progress_message.lock().await;
                *msg = Some("未找到任何文件".to_string());
            });
            return Ok(false);
        }

        let total_files = files.len();
        emit_progress("building", 30, &format!("正在构建文件路径 ({} 个文件)...", total_files), 0, total_files);

        runtime.block_on(async {
            let mut msg = progress_message.lock().await;
            *msg = Some(format!("正在构建路径 ({} 个文件)...", total_files));
        });

        // 构建完整路径 - 使用 HashMap 保留文件信息，使用 Rayon 并行处理
        #[cfg(windows)]
        let files_with_paths: Vec<(crate::services::mft_reader::MftFileInfo, String)> = {
            use std::collections::HashMap;
            use std::sync::Arc;
            use rayon::prelude::*;

            // 构建引用号到文件信息的映射（使用 Arc 共享）
            let ref_map: Arc<HashMap<(char, u64), crate::services::mft_reader::MftFileInfo>> = Arc::new(
                files
                    .iter()
                    .map(|f| ((f.drive_letter, f.file_ref_number & 0x0000FFFFFFFFFFFF), f.clone()))
                    .collect()
            );

            // 使用 Rayon 并行构建路径
            files.par_iter().map(|file| {
                // 构建完整路径
                let mut path_parts = vec![file.file_name.clone()];
                let mut current_parent = file.parent_ref_number & 0x0000FFFFFFFFFFFF;
                let drive_letter = file.drive_letter;

                let mut depth = 0;
                const MAX_DEPTH: usize = 100;

                while depth < MAX_DEPTH {
                    if current_parent == 5 {
                        break;
                    }
                    if let Some(parent_file) = ref_map.get(&(drive_letter, current_parent)) {
                        path_parts.push(parent_file.file_name.clone());
                        current_parent = parent_file.parent_ref_number & 0x0000FFFFFFFFFFFF;
                    } else {
                        break;
                    }
                    depth += 1;
                }

                path_parts.reverse();
                let full_path = format!("{}:\\{}", drive_letter, path_parts.join("\\"));
                (file.clone(), full_path)
            }).collect()
        };

        #[cfg(not(windows))]
        let files_with_paths: Vec<(crate::services::mft_reader::MftFileInfo, String)> = Vec::new();

        emit_progress("indexing", 50, &format!("正在建立索引 ({} 个文件)...", files_with_paths.len()), 0, files_with_paths.len());

        runtime.block_on(async {
            let mut msg = progress_message.lock().await;
            *msg = Some(format!("正在建立索引 ({} 个文件)...", files_with_paths.len()));
        });

        // 禁用 FTS 触发器（关键优化：避免每条记录都触发 FTS 更新）
        if let Err(e) = service.disable_fts_triggers() {
            log::warn!("禁用 FTS 触发器失败: {}", e);
        }

        // 清空旧索引
        if let Err(e) = service.clear_index() {
            log::warn!("清空索引失败: {}", e);
        }

        // 批量插入索引 - 使用大批次和优化的插入方法
        let batch_size = 100000; // 使用更大的批次
        let total_files_count = files_with_paths.len();
        let mut processed = 0;

        // 使用 Rayon 并行准备数据
        use rayon::prelude::*;

        for chunk in files_with_paths.chunks(batch_size) {
            // 并行转换数据
            let records: Vec<FileIndexRecord> = chunk
                .par_iter()
                .filter_map(|(file_info, path)| {
                    let path_obj = std::path::Path::new(path);
                    let file_name = path_obj.file_name()?.to_string_lossy().to_string();
                    let drive = extract_drive_letter(path)?;
                    let file_type = if file_info.is_directory {
                        String::new()
                    } else {
                        path_obj.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
                    };

                    // 直接使用 MFT 解析得到的 size 和 modified_time，无需额外磁盘 I/O
                    Some(FileIndexRecord {
                        id: None,
                        name: file_name,
                        path: path.to_string(),
                        size: file_info.size,
                        modified_time: file_info.modified_time.clone(),
                        is_dir: file_info.is_directory,
                        file_type,
                        drive,
                        indexed_at: None,
                    })
                })
                .collect();

            // 使用优化的批量插入（在同一连接上设置 PRAGMA）
            if let Err(e) = service.batch_insert_optimized(records) {
                log::warn!("批量插入失败: {}", e);
            }

            processed += chunk.len();
            let progress = 50 + ((processed * 46) / total_files_count.max(1)) as u32;
            emit_progress(
                "indexing",
                progress,
                &format!("正在建立索引 ({}/{})...", processed, total_files_count),
                processed,
                total_files_count,
            );
        }

        // 从主表重建 FTS 索引（分批处理）
        emit_progress("indexing", 96, "正在构建搜索索引...", total_files_count, total_files_count);
        if let Err(e) = service.rebuild_fts_from_main_table() {
            log::error!("重建 FTS 索引失败: {}", e);
            let _ = app.emit("file-index:error", format!("构建搜索索引失败: {}", e));
        } else {
            log::info!("FTS 搜索索引构建成功");
        }

        // 重新启用 FTS 触发器（用于后续增量更新）
        if let Err(e) = service.enable_fts_triggers() {
            log::warn!("重新启用 FTS 触发器失败: {}", e);
        }

        // 完成
        runtime.block_on(async {
            let mut is_indexing = is_indexing_flag.lock().await;
            *is_indexing = false;
            let mut msg = progress_message.lock().await;
            *msg = None;
        });

        emit_progress("completed", 100, &format!("索引完成，共 {} 个文件", total_files_count), total_files_count, total_files_count);
        let _ = app.emit("file-index:completed", total_files_count);

        log::info!("文件索引完成，共 {} 个文件", total_files_count);
        Ok(true)
    })
    .await
    .map_err(|e| format!("索引任务执行失败: {}", e))?
}

/// 搜索已索引的文件
///
/// # Arguments
/// * `keyword` - 搜索关键词
/// * `drive` - 限定搜索的驱动器（可选）
/// * `max_results` - 最大结果数量（默认 100）
///
/// # Returns
/// 返回匹配的文件列表
#[tauri::command]
pub async fn search_indexed_files(
    state: State<'_, FileIndexState>,
    keyword: String,
    drive: Option<String>,
    max_results: Option<usize>,
) -> Result<Vec<FileIndexRecord>, String> {
    if keyword.trim().is_empty() {
        return Err("搜索关键词不能为空".to_string());
    }

    let max = max_results.unwrap_or(100);

    let service = state.service.clone();
    let keyword_clone = keyword.clone();
    let drive_clone = drive.clone();

    tokio::task::spawn_blocking(move || {
        // 确保数据库表存在
        if let Err(e) = service.init_db() {
            log::warn!("搜索前初始化数据库失败: {}", e);
        }

        if let Some(d) = drive_clone {
            service.search_in_drive(&keyword_clone, &d, max)
        } else {
            service.search(&keyword_clone, max)
        }
    })
    .await
    .map_err(|e| format!("搜索任务执行失败: {}", e))?
}

/// 获取索引统计信息
///
/// # Returns
/// 返回索引统计信息
#[tauri::command]
pub async fn get_file_index_stats(
    state: State<'_, FileIndexState>,
) -> Result<IndexStats, String> {
    let service = state.service.clone();

    tokio::task::spawn_blocking(move || {
        // 确保数据库表存在
        if let Err(e) = service.init_db() {
            log::warn!("获取统计前初始化数据库失败: {}", e);
        }
        service.get_stats()
    })
    .await
    .map_err(|e| format!("获取统计信息失败: {}", e))?
}

/// 清空文件索引
///
/// # Arguments
/// * `drive` - 要清空的驱动器（可选，None 表示清空全部）
///
/// # Returns
/// 返回是否成功清空
#[tauri::command]
pub async fn clear_file_index(
    state: State<'_, FileIndexState>,
    drive: Option<String>,
) -> Result<bool, String> {
    let service = state.service.clone();

    tokio::task::spawn_blocking(move || {
        if let Some(d) = drive {
            service.clear_drive(&d)?;
        } else {
            service.clear_index()?;
        }
        Ok(true)
    })
    .await
    .map_err(|e| format!("清空索引失败: {}", e))?
}

/// 开始监控文件变化
///
/// 使用 Windows ReadDirectoryChangesW API 监控文件系统变化
///
/// # Arguments
/// * `drives` - 要监控的驱动器列表，如 ["C", "D"]
///
/// # Returns
/// 返回是否成功开始监控
#[tauri::command]
pub async fn start_file_watching(
    app: AppHandle,
    state: State<'_, FileIndexState>,
    drives: Option<Vec<String>>,
) -> Result<bool, String> {
    #[cfg(windows)]
    {
        let mut watcher_guard = state.watcher.lock().await;

        // 如果已有监控在运行，先停止
        if let Some(ref mut w) = *watcher_guard {
            if w.is_running() {
                w.stop();
            }
        }

        // 确定要监控的驱动器
        let drive_chars: Vec<char> = if let Some(drive_list) = drives {
            // 从驱动器列表中提取有效的驱动器字母 (A-Z)
            // 过滤掉无效的驱动器（如中文字符、数字等）
            drive_list
                .iter()
                .filter_map(|d| {
                    let first_char = d.chars().next()?;
                    // 只接受有效的驱动器字母 (A-Z 或 a-z)
                    if first_char.is_ascii_alphabetic() {
                        Some(first_char.to_ascii_uppercase())
                    } else {
                        log::warn!("[文件监控] 忽略无效的驱动器: {}", d);
                        None
                    }
                })
                .collect()
        } else {
            // 默认监控所有可用驱动器
            ('C'..='Z')
                .filter(|c| {
                    let path = format!("{}:\\", c);
                    std::path::Path::new(&path).exists()
                })
                .collect()
        };

        if drive_chars.is_empty() {
            return Err("没有可用的驱动器".to_string());
        }

        let watcher = FileWatcher::new(drive_chars);
        let service = state.service.clone();

        // 创建 channel 用于接收文件变化事件
        let (sender, receiver) = std::sync::mpsc::channel::<FileChangeEvent>();

        // 启动监控
        watcher.start(sender)?;

        // 启动事件处理任务
        let app_clone = app.clone();
        std::thread::spawn(move || {
            for event in receiver {
                match &event {
                    FileChangeEvent::Created { path } => {
                        log::debug!("文件创建: {}", path);
                        // 添加到索引
                        if let Ok(metadata) = std::fs::metadata(path) {
                            let path_obj = std::path::Path::new(path);
                            if let Some(file_name) = path_obj.file_name() {
                                let record = FileIndexRecord {
                                    id: None,
                                    name: file_name.to_string_lossy().to_string(),
                                    path: path.to_string(),
                                    size: if metadata.is_dir() { 0 } else { metadata.len() },
                                    modified_time: metadata.modified().ok().map(|t| {
                                        let datetime: chrono::DateTime<chrono::Local> = t.into();
                                        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                                    }).unwrap_or_else(|| "未知".to_string()),
                                    is_dir: metadata.is_dir(),
                                    file_type: if metadata.is_dir() {
                                        String::new()
                                    } else {
                                        path_obj.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
                                    },
                                    drive: format!("{}:", path.chars().next().unwrap_or('C')),
                                    indexed_at: None,
                                };
                                if let Err(e) = service.add_file(record) {
                                    log::warn!("添加索引失败: {}", e);
                                }
                            }
                        }
                    }
                    FileChangeEvent::Deleted { path } => {
                        log::debug!("文件删除: {}", path);
                        if let Err(e) = service.remove_file(path) {
                            log::warn!("移除索引失败: {}", e);
                        }
                    }
                    FileChangeEvent::Modified { path } => {
                        log::debug!("文件修改: {}", path);
                        // 更新索引
                        if let Ok(metadata) = std::fs::metadata(path) {
                            let path_obj = std::path::Path::new(path);
                            if let Some(file_name) = path_obj.file_name() {
                                let record = FileIndexRecord {
                                    id: None,
                                    name: file_name.to_string_lossy().to_string(),
                                    path: path.to_string(),
                                    size: if metadata.is_dir() { 0 } else { metadata.len() },
                                    modified_time: metadata.modified().ok().map(|t| {
                                        let datetime: chrono::DateTime<chrono::Local> = t.into();
                                        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                                    }).unwrap_or_else(|| "未知".to_string()),
                                    is_dir: metadata.is_dir(),
                                    file_type: if metadata.is_dir() {
                                        String::new()
                                    } else {
                                        path_obj.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
                                    },
                                    drive: format!("{}:", path.chars().next().unwrap_or('C')),
                                    indexed_at: None,
                                };
                                if let Err(e) = service.update_file(record) {
                                    log::warn!("更新索引失败: {}", e);
                                }
                            }
                        }
                    }
                    FileChangeEvent::Renamed { old_path, new_path } => {
                        log::debug!("文件重命名: {} -> {}", old_path, new_path);
                        // 先删除旧路径，再添加新路径
                        let _ = service.remove_file(old_path);
                        if let Ok(metadata) = std::fs::metadata(new_path) {
                            let path_obj = std::path::Path::new(new_path);
                            if let Some(file_name) = path_obj.file_name() {
                                let record = FileIndexRecord {
                                    id: None,
                                    name: file_name.to_string_lossy().to_string(),
                                    path: new_path.clone(),
                                    size: if metadata.is_dir() { 0 } else { metadata.len() },
                                    modified_time: metadata.modified().ok().map(|t| {
                                        let datetime: chrono::DateTime<chrono::Local> = t.into();
                                        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                                    }).unwrap_or_else(|| "未知".to_string()),
                                    is_dir: metadata.is_dir(),
                                    file_type: if metadata.is_dir() {
                                        String::new()
                                    } else {
                                        path_obj.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
                                    },
                                    drive: format!("{}:", new_path.chars().next().unwrap_or('C')),
                                    indexed_at: None,
                                };
                                if let Err(e) = service.add_file(record) {
                                    log::warn!("添加索引失败: {}", e);
                                }
                            }
                        }
                    }
                }

                // 发送文件变化事件到前端
                let _ = app_clone.emit("file-index:change", &event);
            }
        });

        *watcher_guard = Some(watcher);

        Ok(true)
    }

    #[cfg(not(windows))]
    {
        Err("文件监控仅支持 Windows 系统".to_string())
    }
}

/// 停止监控文件变化
///
/// # Returns
/// 返回是否成功停止
#[tauri::command]
pub async fn stop_file_watching(
    state: State<'_, FileIndexState>,
) -> Result<bool, String> {
    let mut watcher_guard = state.watcher.lock().await;

    if let Some(ref mut watcher) = *watcher_guard {
        watcher.stop();
        *watcher_guard = None;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 获取所有可用驱动器
///
/// # Returns
/// 返回驱动器列表
#[tauri::command]
pub fn get_available_drives() -> Vec<String> {
    #[cfg(windows)]
    {
        ('A'..='Z')
            .filter(|c| {
                let path = format!("{}:\\", c);
                std::path::Path::new(&path).exists()
            })
            .map(|c| c.to_string())
            .collect()
    }

    #[cfg(not(windows))]
    {
        vec!["/".to_string()]
    }
}

/// 后台自动索引（内部函数）
///
/// 用于后台自动建立/更新文件索引，使用最低优先级执行，
/// 发送进度事件以便前端显示进度。
///
/// # Arguments
/// * `service` - 文件索引服务实例
/// * `drives` - 要索引的驱动器列表，None 表示所有驱动器
/// * `app` - 可选的 AppHandle，用于发送进度和完成事件
///
/// # Returns
/// 返回索引的文件数量，失败返回错误信息
pub fn start_background_indexing(
    service: FileIndexService,
    drives: Option<Vec<String>>,
    app: Option<AppHandle>,
) -> Result<usize, String> {
    // 设置线程为低优先级
    #[cfg(windows)]
    {
        unsafe {
            let thread = GetCurrentThread();
            let _ = SetThreadPriority(thread, THREAD_PRIORITY_BELOW_NORMAL);
        }
        log::debug!("后台索引线程已设置为低优先级");
    }

    // 发送进度事件的辅助闭包
    let emit_progress = |app_opt: &Option<AppHandle>, stage: &str, progress: u32, message: &str, processed: usize, total: usize| {
        if let Some(ref app_handle) = app_opt {
            let event = IndexProgressEvent {
                stage: stage.to_string(),
                progress,
                message: message.to_string(),
                processed,
                total,
            };
            let _ = app_handle.emit("file-index:progress", event);
        }
    };

    // 检查管理员权限
    #[cfg(windows)]
    if !MftReader::check_admin_privilege() {
        emit_progress(&app, "error", 0, "需要管理员权限", 0, 0);
        return Err("需要管理员权限才能读取 NTFS MFT".to_string());
    }

    // 确保数据库表存在（可能被删除或首次创建）
    if let Err(e) = service.init_db() {
        log::error!("后台索引: 初始化文件索引数据库失败: {}", e);
        emit_progress(&app, "error", 0, &format!("初始化数据库失败: {}", e), 0, 0);
        return Err(format!("初始化数据库失败: {}", e));
    }
    log::info!("后台索引: 数据库表已确认存在");

    emit_progress(&app, "reading", 5, "正在读取文件系统...", 0, 0);

    // 读取 MFT
    #[cfg(windows)]
    let files = if let Some(drive_letters) = drives {
        let mut all_files = Vec::new();
        let total_drives = drive_letters.len();
        for (i, drive) in drive_letters.iter().enumerate() {
            let drive_char = drive.chars().next().unwrap_or('C');
            let progress = 5 + ((i * 20) / total_drives.max(1)) as u32;
            emit_progress(&app, "reading", progress, &format!("正在读取驱动器 {}:...", drive_char), i, total_drives);

            match MftReader::read_drive(drive_char) {
                Ok(files) => {
                    log::info!("后台索引: 驱动器 {}: 读取到 {} 个文件", drive_char, files.len());
                    all_files.extend(files);
                }
                Err(e) => {
                    log::warn!("后台索引: 读取驱动器 {} 失败: {}", drive_char, e);
                }
            }
            // 驱动器之间休眠，减少资源占用
            std::thread::sleep(Duration::from_millis(100));
        }
        all_files
    } else {
        emit_progress(&app, "reading", 10, "正在读取所有驱动器...", 0, 0);
        match MftReader::read_all_drives() {
            Ok(files) => files,
            Err(e) => {
                emit_progress(&app, "error", 0, &format!("读取失败: {}", e), 0, 0);
                return Err(format!("后台索引: 读取失败: {}", e));
            }
        }
    };

    #[cfg(not(windows))]
    let files: Vec<crate::services::mft_reader::MftFileInfo> = Vec::new();

    if files.is_empty() {
        log::info!("后台索引: 未找到任何文件");
        emit_progress(&app, "completed", 100, "未找到任何文件", 0, 0);
        return Ok(0);
    }

    let total_files = files.len();
    log::info!("后台索引: 开始构建路径 ({} 个文件)", total_files);
    emit_progress(&app, "building", 25, &format!("正在构建路径 ({} 个文件)...", total_files), 0, total_files);

    // 构建完整路径 - 使用 Rayon 并行处理
    #[cfg(windows)]
    let files_with_paths: Vec<(crate::services::mft_reader::MftFileInfo, String)> = {
        use std::collections::HashMap;
        use std::sync::Arc;
        use rayon::prelude::*;

        // 构建引用号到文件信息的映射（使用 Arc 共享）
        let ref_map: Arc<HashMap<(char, u64), crate::services::mft_reader::MftFileInfo>> = Arc::new(
            files
                .iter()
                .map(|f| ((f.drive_letter, f.file_ref_number & 0x0000FFFFFFFFFFFF), f.clone()))
                .collect()
        );

        // 使用 Rayon 并行构建路径
        files.par_iter().map(|file| {
            let mut path_parts = vec![file.file_name.clone()];
            let mut current_parent = file.parent_ref_number & 0x0000FFFFFFFFFFFF;
            let drive_letter = file.drive_letter;

            let mut depth = 0;
            const MAX_DEPTH: usize = 100;

            while depth < MAX_DEPTH {
                if current_parent == 5 {
                    break;
                }
                if let Some(parent_file) = ref_map.get(&(drive_letter, current_parent)) {
                    path_parts.push(parent_file.file_name.clone());
                    current_parent = parent_file.parent_ref_number & 0x0000FFFFFFFFFFFF;
                } else {
                    break;
                }
                depth += 1;
            }

            path_parts.reverse();
            let full_path = format!("{}:\\{}", drive_letter, path_parts.join("\\"));
            (file.clone(), full_path)
        }).collect()
    };

    #[cfg(not(windows))]
    let files_with_paths: Vec<(crate::services::mft_reader::MftFileInfo, String)> = Vec::new();

    let total_files_count = files_with_paths.len();
    emit_progress(&app, "indexing", 35, &format!("正在建立索引 ({} 个文件)...", total_files_count), 0, total_files_count);

    // 禁用 FTS 触发器（关键优化：避免每条记录都触发 FTS 更新）
    if let Err(e) = service.disable_fts_triggers() {
        log::warn!("后台索引: 禁用 FTS 触发器失败: {}", e);
    }

    // 清空旧索引
    if let Err(e) = service.clear_index() {
        log::warn!("后台索引: 清空索引失败: {}", e);
    }

    // 使用大批次和优化的插入方法
    let batch_size = 100000; // 使用更大的批次
    let mut processed = 0;

    // 使用 Rayon 并行准备数据
    use rayon::prelude::*;

    for chunk in files_with_paths.chunks(batch_size) {
        // 并行转换数据
        let records: Vec<FileIndexRecord> = chunk
            .par_iter()
            .filter_map(|(file_info, path)| {
                let path_obj = std::path::Path::new(path);
                let file_name = path_obj.file_name()?.to_string_lossy().to_string();
                let drive = extract_drive_letter(path)?;
                let file_type = if file_info.is_directory {
                    String::new()
                } else {
                    path_obj.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
                };

                // 直接使用 MFT 解析得到的 size 和 modified_time
                Some(FileIndexRecord {
                    id: None,
                    name: file_name,
                    path: path.to_string(),
                    size: file_info.size,
                    modified_time: file_info.modified_time.clone(),
                    is_dir: file_info.is_directory,
                    file_type,
                    drive,
                    indexed_at: None,
                })
            })
            .collect();

        // 使用优化的批量插入（在同一连接上设置 PRAGMA）
        if let Err(e) = service.batch_insert_optimized(records) {
            log::warn!("后台索引: 批量插入失败: {}", e);
        }

        processed += chunk.len();
        let percent = 35 + ((processed * 60) / total_files_count.max(1)) as u32;
        emit_progress(
            &app,
            "indexing",
            percent.min(95),
            &format!("正在建立索引 ({}/{})...", processed, total_files_count),
            processed,
            total_files_count
        );
    }

    // 从主表重建 FTS 索引（分批处理）
    emit_progress(&app, "indexing", 96, "正在构建搜索索引...", total_files_count, total_files_count);
    if let Err(e) = service.rebuild_fts_from_main_table() {
        log::error!("后台索引: 重建 FTS 索引失败: {}", e);
        if let Some(ref app_handle) = app {
            let _ = app_handle.emit("file-index:error", format!("构建搜索索引失败: {}", e));
        }
    } else {
        log::info!("后台索引: FTS 搜索索引构建成功");
    }

    // 重新启用 FTS 触发器（用于后续增量更新）
    if let Err(e) = service.enable_fts_triggers() {
        log::warn!("后台索引: 重新启用 FTS 触发器失败: {}", e);
    }

    log::info!("后台索引完成，共 {} 个文件", total_files_count);

    // 发送完成事件
    emit_progress(&app, "completed", 100, &format!("索引完成，共 {} 个文件", total_files_count), total_files_count, total_files_count);

    if let Some(ref app_handle) = app {
        let _ = app_handle.emit("file-index:completed", total_files_count);
    }

    Ok(total_files_count)
}

/// 自动初始化文件索引
///
/// 此函数供程序启动时调用，用于自动检查和初始化文件索引。
/// 如果没有索引且有管理员权限，则后台自动建立索引；
/// 如果已有索引，则自动启动文件监控。
///
/// # Arguments
/// * `app` - Tauri AppHandle
/// * `state` - 文件索引状态
///
/// # Returns
/// 返回初始化结果消息
pub async fn auto_init_file_index(
    app: tauri::AppHandle,
    state: &FileIndexState,
) -> Result<String, String> {
    use crate::utils::crash_logger::log_runtime;

    log_runtime("[文件索引] 开始自动初始化检查...");

    // 1. 检查索引状态
    let stats = state.service.get_stats();
    let has_index = match &stats {
        Ok(s) => s.total_files > 0 || s.total_dirs > 0,
        Err(_) => false,
    };

    log_runtime(&format!(
        "[文件索引] 索引状态: has_index={}, stats={:?}",
        has_index, stats
    ));

    if has_index {
        // 已有索引，自动启动文件监控
        log_runtime("[文件索引] 检测到已有索引，准备启动文件监控...");

        #[cfg(windows)]
        {
            // 获取已索引的驱动器列表
            let indexed_drives = match &stats {
                Ok(s) => s.indexed_drives.clone(),
                Err(_) => Vec::new(),
            };

            if !indexed_drives.is_empty() {
                log_runtime(&format!(
                    "[文件索引] 将监控以下驱动器: {:?}",
                    indexed_drives
                ));

                // 异步启动文件监控
                let app_clone = app.clone();
                let watcher = state.watcher.clone();
                let service = state.service.clone();
                let drives = indexed_drives.clone();

                tokio::spawn(async move {
                    // 延迟启动，让程序有足够时间完成初始化
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                    // 执行监控启动逻辑
                    if let Err(e) = start_file_watching_internal(
                        app_clone,
                        watcher,
                        service,
                        Some(drives),
                    ).await {
                        log_runtime(&format!("[文件索引] 启动文件监控失败: {}", e));
                    } else {
                        log_runtime("[文件索引] 文件监控已自动启动");
                    }
                });

                return Ok("已有索引，文件监控已启动".to_string());
            } else {
                log_runtime("[文件索引] 索引中无驱动器记录，跳过文件监控");
                return Ok("已有索引但无驱动器记录".to_string());
            }
        }

        #[cfg(not(windows))]
        {
            log_runtime("[文件索引] 非 Windows 系统，跳过文件监控");
            return Ok("非 Windows 系统，跳过文件监控".to_string());
        }
    } else {
        // 没有索引，不再自动建立索引，改为手动触发
        // 用户可以在文件搜索工具中手动点击"建立索引"按钮
        log_runtime("[文件索引] 未检测到索引，跳过自动索引（需手动触发）");
        return Ok("无索引，请手动建立索引".to_string());
    }
}

/// 内部函数：启动文件监控（不通过 tauri::command）
#[cfg(windows)]
async fn start_file_watching_internal(
    app: tauri::AppHandle,
    watcher: Arc<Mutex<Option<FileWatcher>>>,
    service: FileIndexService,
    drives: Option<Vec<String>>,
) -> Result<(), String> {
    use crate::utils::crash_logger::log_runtime;

    let mut watcher_guard = watcher.lock().await;

    // 如果已有监控在运行，先停止
    if let Some(ref mut w) = *watcher_guard {
        if w.is_running() {
            log_runtime("[文件监控] 停止现有监控器");
            w.stop();
        }
    }

    // 确定要监控的驱动器
    let drive_chars: Vec<char> = if let Some(drive_list) = drives {
        // 从驱动器列表中提取有效的驱动器字母 (A-Z)
        // 过滤掉无效的驱动器（如中文字符、数字等）
        drive_list
            .iter()
            .filter_map(|d| {
                let first_char = d.chars().next()?;
                // 只接受有效的驱动器字母 (A-Z 或 a-z)
                if first_char.is_ascii_alphabetic() {
                    Some(first_char.to_ascii_uppercase())
                } else {
                    log::warn!("[文件监控] 忽略无效的驱动器: {}", d);
                    None
                }
            })
            .collect()
    } else {
        // 默认监控所有可用驱动器
        ('C'..='Z')
            .filter(|c| {
                let path = format!("{}:\\", c);
                std::path::Path::new(&path).exists()
            })
            .collect()
    };

    if drive_chars.is_empty() {
        return Err("没有可用的驱动器".to_string());
    }

    log_runtime(&format!("[文件监控] 将监控驱动器: {:?}", drive_chars));

    let new_watcher = FileWatcher::new(drive_chars);

    // 创建 channel 用于接收文件变化事件
    let (sender, receiver) = std::sync::mpsc::channel::<FileChangeEvent>();

    // 启动监控
    new_watcher.start(sender)?;

    // 启动事件处理任务
    let app_clone = app.clone();
    let service_clone = service.clone();
    std::thread::spawn(move || {
        for event in receiver {
            match &event {
                FileChangeEvent::Created { path } => {
                    log::debug!("文件创建: {}", path);
                    if let Ok(metadata) = std::fs::metadata(path) {
                        let path_obj = std::path::Path::new(path);
                        if let Some(file_name) = path_obj.file_name() {
                            let record = FileIndexRecord {
                                id: None,
                                name: file_name.to_string_lossy().to_string(),
                                path: path.to_string(),
                                size: if metadata.is_dir() { 0 } else { metadata.len() },
                                modified_time: metadata.modified().ok().map(|t| {
                                    let datetime: chrono::DateTime<chrono::Local> = t.into();
                                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                                }).unwrap_or_else(|| "未知".to_string()),
                                is_dir: metadata.is_dir(),
                                file_type: if metadata.is_dir() {
                                    String::new()
                                } else {
                                    path_obj.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
                                },
                                drive: format!("{}:", path.chars().next().unwrap_or('C')),
                                indexed_at: None,
                            };
                            if let Err(e) = service_clone.add_file(record) {
                                log::warn!("添加索引失败: {}", e);
                            }
                        }
                    }
                }
                FileChangeEvent::Deleted { path } => {
                    log::debug!("文件删除: {}", path);
                    if let Err(e) = service_clone.remove_file(path) {
                        log::warn!("移除索引失败: {}", e);
                    }
                }
                FileChangeEvent::Modified { path } => {
                    log::debug!("文件修改: {}", path);
                    if let Ok(metadata) = std::fs::metadata(path) {
                        let path_obj = std::path::Path::new(path);
                        if let Some(file_name) = path_obj.file_name() {
                            let record = FileIndexRecord {
                                id: None,
                                name: file_name.to_string_lossy().to_string(),
                                path: path.to_string(),
                                size: if metadata.is_dir() { 0 } else { metadata.len() },
                                modified_time: metadata.modified().ok().map(|t| {
                                    let datetime: chrono::DateTime<chrono::Local> = t.into();
                                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                                }).unwrap_or_else(|| "未知".to_string()),
                                is_dir: metadata.is_dir(),
                                file_type: if metadata.is_dir() {
                                    String::new()
                                } else {
                                    path_obj.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
                                },
                                drive: format!("{}:", path.chars().next().unwrap_or('C')),
                                indexed_at: None,
                            };
                            if let Err(e) = service_clone.update_file(record) {
                                log::warn!("更新索引失败: {}", e);
                            }
                        }
                    }
                }
                FileChangeEvent::Renamed { old_path, new_path } => {
                    log::debug!("文件重命名: {} -> {}", old_path, new_path);
                    let _ = service_clone.remove_file(old_path);
                    if let Ok(metadata) = std::fs::metadata(new_path) {
                        let path_obj = std::path::Path::new(new_path);
                        if let Some(file_name) = path_obj.file_name() {
                            let record = FileIndexRecord {
                                id: None,
                                name: file_name.to_string_lossy().to_string(),
                                path: new_path.clone(),
                                size: if metadata.is_dir() { 0 } else { metadata.len() },
                                modified_time: metadata.modified().ok().map(|t| {
                                    let datetime: chrono::DateTime<chrono::Local> = t.into();
                                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                                }).unwrap_or_else(|| "未知".to_string()),
                                is_dir: metadata.is_dir(),
                                file_type: if metadata.is_dir() {
                                    String::new()
                                } else {
                                    path_obj.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
                                },
                                drive: format!("{}:", new_path.chars().next().unwrap_or('C')),
                                indexed_at: None,
                            };
                            if let Err(e) = service_clone.add_file(record) {
                                log::warn!("添加索引失败: {}", e);
                            }
                        }
                    }
                }
            }

            // 发送文件变化事件到前端
            let _ = app_clone.emit("file-index:change", &event);
        }
    });

    *watcher_guard = Some(new_watcher);

    Ok(())
}
