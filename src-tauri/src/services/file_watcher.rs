//! 文件变化监控模块
//!
//! 使用 Windows API `ReadDirectoryChangesW` 监控文件系统变化。
//! 支持监控多个驱动器，每个驱动器使用独立的监控线程。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

#[cfg(target_os = "windows")]
use std::ffi::OsString;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStringExt;
#[cfg(target_os = "windows")]
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use std::thread;

#[cfg(target_os = "windows")]
use winapi::shared::minwindef::{DWORD, FALSE, LPVOID, TRUE};
#[cfg(target_os = "windows")]
use winapi::shared::winerror::ERROR_OPERATION_ABORTED;
#[cfg(target_os = "windows")]
use winapi::um::errhandlingapi::GetLastError;
#[cfg(target_os = "windows")]
use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
#[cfg(target_os = "windows")]
use winapi::um::winbase::ReadDirectoryChangesW;
#[cfg(target_os = "windows")]
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
#[cfg(target_os = "windows")]
use winapi::um::ioapiset::CancelIo;
#[cfg(target_os = "windows")]
use winapi::um::winbase::FILE_FLAG_BACKUP_SEMANTICS;
#[cfg(target_os = "windows")]
use winapi::um::winnt::{
    FILE_LIST_DIRECTORY, FILE_NOTIFY_CHANGE_CREATION, FILE_NOTIFY_CHANGE_DIR_NAME,
    FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_CHANGE_SIZE,
    FILE_NOTIFY_INFORMATION, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, HANDLE,
};

use serde::Serialize;

/// 文件变化事件类型
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FileChangeEvent {
    /// 文件创建
    Created { path: String },
    /// 文件删除
    Deleted { path: String },
    /// 文件修改
    Modified { path: String },
    /// 文件重命名
    Renamed { old_path: String, new_path: String },
}

/// Windows 文件操作常量
#[cfg(target_os = "windows")]
const FILE_ACTION_ADDED: DWORD = 1;
#[cfg(target_os = "windows")]
const FILE_ACTION_REMOVED: DWORD = 2;
#[cfg(target_os = "windows")]
const FILE_ACTION_MODIFIED: DWORD = 3;
#[cfg(target_os = "windows")]
const FILE_ACTION_RENAMED_OLD_NAME: DWORD = 4;
#[cfg(target_os = "windows")]
const FILE_ACTION_RENAMED_NEW_NAME: DWORD = 5;

/// 缓冲区大小 - 64KB
const BUFFER_SIZE: usize = 64 * 1024;

/// 包装 HANDLE 使其可以安全地在线程间传递
/// HANDLE 本身是可以在线程间传递的，只要正确管理生命周期
#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
struct SendableHandle(HANDLE);

#[cfg(target_os = "windows")]
unsafe impl Send for SendableHandle {}
#[cfg(target_os = "windows")]
unsafe impl Sync for SendableHandle {}

#[cfg(target_os = "windows")]
impl SendableHandle {
    fn new(handle: HANDLE) -> Self {
        Self(handle)
    }

    fn get(&self) -> HANDLE {
        self.0
    }
}

/// 文件变化监控器
///
/// 使用 Windows ReadDirectoryChangesW API 监控指定驱动器的文件系统变化。
/// 每个驱动器使用独立的监控线程。
pub struct FileWatcher {
    /// 监控的驱动器列表
    drives: Vec<char>,
    /// 是否正在运行
    running: Arc<AtomicBool>,
    /// 监控线程句柄
    #[cfg(target_os = "windows")]
    handles: Arc<std::sync::Mutex<Vec<SendableHandle>>>,
}

impl FileWatcher {
    /// 创建新的文件监控器
    ///
    /// # 参数
    /// - `drives`: 要监控的驱动器字母列表，如 `vec!['C', 'D']`
    ///
    /// # 示例
    /// ```ignore
    /// let watcher = FileWatcher::new(vec!['C', 'D']);
    /// ```
    pub fn new(drives: Vec<char>) -> Self {
        Self {
            drives,
            running: Arc::new(AtomicBool::new(false)),
            #[cfg(target_os = "windows")]
            handles: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// 开始监控文件系统变化
    ///
    /// 为每个驱动器创建独立的监控线程，监控文件的创建、删除、修改和重命名事件。
    ///
    /// # 参数
    /// - `sender`: 用于发送文件变化事件的通道发送端
    ///
    /// # 返回
    /// - `Ok(())`: 监控成功启动
    /// - `Err(String)`: 启动失败的错误信息
    ///
    /// # 示例
    /// ```ignore
    /// use std::sync::mpsc::channel;
    ///
    /// let watcher = FileWatcher::new(vec!['C']);
    /// let (sender, receiver) = channel();
    /// watcher.start(sender)?;
    ///
    /// // 在另一个线程中接收事件
    /// for event in receiver {
    ///     println!("{:?}", event);
    /// }
    /// ```
    #[cfg(target_os = "windows")]
    pub fn start(&self, sender: Sender<FileChangeEvent>) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Err("File watcher is already running".to_string());
        }

        self.running.store(true, Ordering::SeqCst);

        // 清空之前的句柄
        {
            let mut handles = self.handles.lock().map_err(|e| e.to_string())?;
            handles.clear();
        }

        for drive in &self.drives {
            let drive_path = format!("{}:\\", drive);
            let sender_clone = sender.clone();
            let running_clone = Arc::clone(&self.running);
            let handles_clone = Arc::clone(&self.handles);
            let drive_char = *drive;

            thread::spawn(move || {
                if let Err(e) =
                    Self::watch_directory(&drive_path, sender_clone, running_clone, handles_clone)
                {
                    log::error!("Error watching drive {}: {}", drive_char, e);
                }
            });
        }

        log::info!(
            "File watcher started for drives: {:?}",
            self.drives
                .iter()
                .map(|d| format!("{}:", d))
                .collect::<Vec<_>>()
        );
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn start(&self, _sender: Sender<FileChangeEvent>) -> Result<(), String> {
        Err("File watcher is only supported on Windows".to_string())
    }

    /// 停止所有监控线程
    ///
    /// 设置运行标志为 false，取消所有 I/O 操作，等待线程退出。
    #[cfg(target_os = "windows")]
    pub fn stop(&self) {
        log::info!("Stopping file watcher...");
        self.running.store(false, Ordering::SeqCst);

        // 取消所有 I/O 操作
        if let Ok(handles) = self.handles.lock() {
            for &handle in handles.iter() {
                let h = handle.get();
                if !h.is_null() && h != INVALID_HANDLE_VALUE {
                    unsafe {
                        CancelIo(h);
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// 检查监控器是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// 监控指定目录的文件变化
    ///
    /// 使用 ReadDirectoryChangesW API 监控目录变化。
    #[cfg(target_os = "windows")]
    fn watch_directory(
        path: &str,
        sender: Sender<FileChangeEvent>,
        running: Arc<AtomicBool>,
        handles: Arc<std::sync::Mutex<Vec<SendableHandle>>>,
    ) -> Result<(), String> {
        use std::os::windows::ffi::OsStrExt;

        // 将路径转换为宽字符
        let wide_path: Vec<u16> = std::ffi::OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // 打开目录句柄
        let handle = unsafe {
            CreateFileW(
                wide_path.as_ptr(),
                FILE_LIST_DIRECTORY,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                std::ptr::null_mut(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS,
                std::ptr::null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return Err(format!(
                "Failed to open directory {}: error code {}",
                path,
                unsafe { GetLastError() }
            ));
        }

        // 保存句柄以便后续取消
        {
            let mut handle_list = handles.lock().map_err(|e| e.to_string())?;
            handle_list.push(SendableHandle::new(handle));
        }

        log::info!("Started watching directory: {}", path);

        // 分配缓冲区
        let mut buffer: Vec<u8> = vec![0; BUFFER_SIZE];
        let mut bytes_returned: DWORD = 0;

        // 用于处理重命名事件的临时存储
        let mut pending_rename_old_path: Option<String> = None;

        // 监控循环
        while running.load(Ordering::SeqCst) {
            let success = unsafe {
                ReadDirectoryChangesW(
                    handle,
                    buffer.as_mut_ptr() as LPVOID,
                    BUFFER_SIZE as DWORD,
                    TRUE, // 监控子目录
                    FILE_NOTIFY_CHANGE_FILE_NAME
                        | FILE_NOTIFY_CHANGE_DIR_NAME
                        | FILE_NOTIFY_CHANGE_SIZE
                        | FILE_NOTIFY_CHANGE_LAST_WRITE
                        | FILE_NOTIFY_CHANGE_CREATION,
                    &mut bytes_returned,
                    std::ptr::null_mut(),
                    None,
                )
            };

            if success == FALSE {
                let error = unsafe { GetLastError() };
                if error == ERROR_OPERATION_ABORTED {
                    // 操作被取消，正常退出
                    log::info!("Directory watch cancelled for: {}", path);
                    break;
                }
                log::error!(
                    "ReadDirectoryChangesW failed for {}: error code {}",
                    path,
                    error
                );
                continue;
            }

            if bytes_returned == 0 {
                continue;
            }

            // 解析通知信息
            let mut offset: usize = 0;
            loop {
                let info =
                    unsafe { &*(buffer.as_ptr().add(offset) as *const FILE_NOTIFY_INFORMATION) };

                // 获取文件名
                // FILE_NOTIFY_INFORMATION 结构: NextEntryOffset(4) + Action(4) + FileNameLength(4) + FileName(变长)
                // FileName 字段偏移量为 12 字节
                let file_name_length = info.FileNameLength as usize / 2; // 长度是字节，需要转换为字符数
                let file_name_ptr = info.FileName.as_ptr();
                let file_name_slice =
                    unsafe { std::slice::from_raw_parts(file_name_ptr, file_name_length) };
                let file_name = OsString::from_wide(file_name_slice)
                    .to_string_lossy()
                    .to_string();

                // 构建完整路径
                let full_path = PathBuf::from(path).join(&file_name);
                let full_path_str = full_path.to_string_lossy().to_string();

                // 根据操作类型创建事件
                let event = match info.Action {
                    FILE_ACTION_ADDED => Some(FileChangeEvent::Created {
                        path: full_path_str,
                    }),
                    FILE_ACTION_REMOVED => Some(FileChangeEvent::Deleted {
                        path: full_path_str,
                    }),
                    FILE_ACTION_MODIFIED => Some(FileChangeEvent::Modified {
                        path: full_path_str,
                    }),
                    FILE_ACTION_RENAMED_OLD_NAME => {
                        // 保存旧路径，等待新路径
                        pending_rename_old_path = Some(full_path_str);
                        None
                    }
                    FILE_ACTION_RENAMED_NEW_NAME => {
                        // 配对旧路径创建重命名事件
                        if let Some(old_path) = pending_rename_old_path.take() {
                            Some(FileChangeEvent::Renamed {
                                old_path,
                                new_path: full_path_str,
                            })
                        } else {
                            // 如果没有旧路径，当作创建事件处理
                            Some(FileChangeEvent::Created {
                                path: full_path_str,
                            })
                        }
                    }
                    _ => None,
                };

                // 发送事件
                if let Some(evt) = event {
                    if sender.send(evt).is_err() {
                        log::warn!("Failed to send file change event, receiver dropped");
                        break;
                    }
                }

                // 检查是否有更多条目
                if info.NextEntryOffset == 0 {
                    break;
                }
                offset += info.NextEntryOffset as usize;
            }
        }

        // 关闭句柄
        unsafe {
            CloseHandle(handle);
        }

        // 从句柄列表中移除
        if let Ok(mut handle_list) = handles.lock() {
            handle_list.retain(|&h| h.get() != handle);
        }

        log::info!("Stopped watching directory: {}", path);
        Ok(())
    }

    /// 获取所有可用的驱动器列表
    ///
    /// # 返回
    /// 返回系统中所有可用的驱动器字母列表
    #[cfg(target_os = "windows")]
    pub fn get_available_drives() -> Vec<char> {
        use winapi::um::fileapi::GetLogicalDrives;

        let drive_mask = unsafe { GetLogicalDrives() };
        let mut drives = Vec::new();

        for i in 0..26 {
            if (drive_mask >> i) & 1 != 0 {
                let drive_letter = (b'A' + i as u8) as char;
                drives.push(drive_letter);
            }
        }

        drives
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get_available_drives() -> Vec<char> {
        Vec::new()
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        if self.is_running() {
            self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_watcher_creation() {
        let watcher = FileWatcher::new(vec!['C', 'D']);
        assert!(!watcher.is_running());
        assert_eq!(watcher.drives.len(), 2);
    }

    #[test]
    fn test_file_watcher_stop_when_not_running() {
        let watcher = FileWatcher::new(vec!['C']);
        watcher.stop();
        assert!(!watcher.is_running());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_get_available_drives() {
        let drives = FileWatcher::get_available_drives();
        // 至少应该有一个驱动器（通常是 C:）
        assert!(!drives.is_empty());
        // 验证返回的是有效的驱动器字母
        for drive in &drives {
            assert!(drive.is_ascii_uppercase());
        }
    }

    #[test]
    fn test_file_change_event_serialization() {
        let event = FileChangeEvent::Created {
            path: "C:\\test\\file.txt".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("created"));
        assert!(json.contains("C:\\\\test\\\\file.txt"));
    }

    #[test]
    fn test_renamed_event_serialization() {
        let event = FileChangeEvent::Renamed {
            old_path: "C:\\test\\old.txt".to_string(),
            new_path: "C:\\test\\new.txt".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("renamed"));
        assert!(json.contains("old.txt"));
        assert!(json.contains("new.txt"));
    }
}
