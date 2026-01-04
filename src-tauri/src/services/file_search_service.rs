use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use walkdir::WalkDir;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// 需要跳过的目录名称（小写）
const SKIP_DIRS: &[&str] = &[
    // Windows 系统目录
    "windows", "$recycle.bin", "system volume information",
    "programdata", "recovery", "perflogs",
    // 开发相关
    "node_modules", ".git", ".svn", ".hg", "target", "dist", "build",
    "__pycache__", ".cache", ".npm", ".cargo", ".rustup",
    // 应用缓存
    "appdata", "application data", "local settings",
];

/// 文件搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchResult {
    /// 文件名
    pub name: String,
    /// 文件完整路径
    pub path: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 修改时间（格式化字符串）
    pub modified_time: String,
    /// 是否是目录
    pub is_dir: bool,
    /// 文件类型（扩展名）
    pub file_type: String,
}

/// 文件搜索服务
pub struct FileSearchService;

impl FileSearchService {
    /// 搜索文件
    ///
    /// # Arguments
    /// * `keyword` - 搜索关键词（支持模糊匹配）
    /// * `search_paths` - 搜索路径列表（可选，默认搜索所有驱动器）
    /// * `max_results` - 最大结果数量（可选，默认 100）
    ///
    /// # Returns
    /// 返回匹配的文件列表
    pub fn search_files(
        keyword: &str,
        search_paths: Option<Vec<String>>,
        max_results: Option<usize>,
    ) -> Result<Vec<FileSearchResult>, String> {
        let keyword_lower = keyword.to_lowercase();
        let max_results = max_results.unwrap_or(100);

        // 确定搜索路径
        let paths = match search_paths {
            Some(paths) if !paths.is_empty() => paths,
            _ => Self::get_search_paths(),
        };

        if paths.is_empty() {
            return Err("没有找到可用的搜索路径".to_string());
        }

        // 使用原子计数器跨线程共享结果数量
        let found_count = Arc::new(AtomicUsize::new(0));
        let should_stop = Arc::new(AtomicBool::new(false));

        // 使用 rayon 并行搜索所有路径
        let results: Vec<FileSearchResult> = paths
            .par_iter()
            .flat_map(|path| {
                Self::search_in_path(
                    path,
                    &keyword_lower,
                    max_results,
                    Arc::clone(&found_count),
                    Arc::clone(&should_stop),
                )
            })
            .collect();

        // 限制结果数量并按修改时间排序（最新的在前）
        let mut sorted_results = results;
        sorted_results.sort_by(|a, b| b.modified_time.cmp(&a.modified_time));
        sorted_results.truncate(max_results);

        Ok(sorted_results)
    }

    /// 获取优化的搜索路径（优先用户目录）
    fn get_search_paths() -> Vec<String> {
        let mut paths = Vec::new();

        #[cfg(windows)]
        {
            // 优先搜索用户目录
            if let Ok(user_profile) = std::env::var("USERPROFILE") {
                paths.push(user_profile);
            }

            // 然后搜索其他驱动器（排除 C 盘的系统部分）
            for letter in b'C'..=b'Z' {
                let drive = format!("{}:\\", letter as char);
                let path = Path::new(&drive);
                if path.exists() && letter != b'C' {
                    paths.push(drive);
                }
            }

            // C 盘只搜索 Users 和 Program Files
            if Path::new("C:\\Users").exists() && !paths.iter().any(|p| p.contains("Users")) {
                paths.push("C:\\Users".to_string());
            }
            if Path::new("C:\\Program Files").exists() {
                paths.push("C:\\Program Files".to_string());
            }
            if Path::new("C:\\Program Files (x86)").exists() {
                paths.push("C:\\Program Files (x86)".to_string());
            }
        }

        #[cfg(not(windows))]
        {
            if let Ok(home) = std::env::var("HOME") {
                paths.push(home);
            }
            paths.push("/".to_string());
        }

        paths
    }

    /// 检查是否应该跳过该目录
    fn should_skip_dir(dir_name: &str) -> bool {
        let lower = dir_name.to_lowercase();
        SKIP_DIRS.iter().any(|&skip| lower == skip)
    }

    /// 在指定路径中搜索文件
    fn search_in_path(
        base_path: &str,
        keyword: &str,
        max_results: usize,
        found_count: Arc<AtomicUsize>,
        should_stop: Arc<AtomicBool>,
    ) -> Vec<FileSearchResult> {
        let mut results = Vec::new();
        let path = Path::new(base_path);

        if !path.exists() {
            return results;
        }

        // 使用 WalkDir 遍历目录，设置最大深度为 15 层
        let walker = WalkDir::new(path)
            .max_depth(15)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // 跳过需要排除的目录
                if e.file_type().is_dir() {
                    if let Some(name) = e.file_name().to_str() {
                        if Self::should_skip_dir(name) {
                            return false;
                        }
                    }
                }
                true
            });

        for entry in walker.filter_map(|e| e.ok()) {
            // 检查是否应该停止搜索
            if should_stop.load(Ordering::Relaxed) {
                break;
            }

            // 检查全局结果数量
            let current_count = found_count.load(Ordering::Relaxed);
            if current_count >= max_results {
                should_stop.store(true, Ordering::Relaxed);
                break;
            }

            let file_path = entry.path();
            let file_name = match file_path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            // 检查文件名是否包含关键词（不区分大小写）
            if file_name.to_lowercase().contains(keyword) {
                if let Ok(metadata) = entry.metadata() {
                    let modified_time = Self::format_system_time(metadata.modified().ok());
                    let is_dir = metadata.is_dir();
                    let size = if is_dir { 0 } else { metadata.len() };
                    let file_type = if is_dir {
                        "文件夹".to_string()
                    } else {
                        Self::get_extension(&file_name)
                    };

                    results.push(FileSearchResult {
                        name: file_name,
                        path: file_path.to_string_lossy().to_string(),
                        size,
                        modified_time,
                        is_dir,
                        file_type,
                    });

                    // 更新全局计数
                    found_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        results
    }

    /// 获取所有驱动器（Windows）
    ///
    /// # Returns
    /// 返回所有可用驱动器的路径列表
    pub fn get_all_drives() -> Vec<String> {
        let mut drives = Vec::new();

        #[cfg(windows)]
        {
            // Windows: 检查 A-Z 驱动器
            for letter in b'A'..=b'Z' {
                let drive = format!("{}:\\", letter as char);
                let path = Path::new(&drive);
                if path.exists() {
                    drives.push(drive);
                }
            }
        }

        #[cfg(not(windows))]
        {
            // Unix: 使用根目录和常见挂载点
            drives.push("/".to_string());
            if Path::new("/home").exists() {
                drives.push("/home".to_string());
            }
        }

        drives
    }

    /// 使用系统默认程序打开文件
    ///
    /// # Arguments
    /// * `path` - 文件路径
    ///
    /// # Returns
    /// 成功返回 Ok(())，失败返回错误信息
    pub fn open_file(path: &str) -> Result<(), String> {
        let file_path = Path::new(path);

        if !file_path.exists() {
            return Err(format!("文件不存在: {}", path));
        }

        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            let mut cmd = Command::new("cmd");
            cmd.args(["/C", "start", "", path]);
            cmd.creation_flags(CREATE_NO_WINDOW);

            cmd.spawn()
                .map_err(|e| format!("打开文件失败: {}", e))?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(path)
                .spawn()
                .map_err(|e| format!("打开文件失败: {}", e))?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(path)
                .spawn()
                .map_err(|e| format!("打开文件失败: {}", e))?;
        }

        Ok(())
    }

    /// 在文件管理器中显示文件
    ///
    /// # Arguments
    /// * `path` - 文件路径
    ///
    /// # Returns
    /// 成功返回 Ok(())，失败返回错误信息
    pub fn open_in_folder(path: &str) -> Result<(), String> {
        let file_path = Path::new(path);

        if !file_path.exists() {
            return Err(format!("路径不存在: {}", path));
        }

        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            let mut cmd = Command::new("explorer");
            cmd.args(["/select,", path]);
            cmd.creation_flags(CREATE_NO_WINDOW);

            cmd.spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .args(["-R", path])
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }

        #[cfg(target_os = "linux")]
        {
            // 获取文件所在目录
            let parent = file_path.parent().unwrap_or(file_path);
            Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }

        Ok(())
    }

    /// 格式化系统时间
    fn format_system_time(time: Option<SystemTime>) -> String {
        match time {
            Some(t) => {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            }
            None => "未知".to_string(),
        }
    }

    /// 获取文件扩展名
    fn get_extension(file_name: &str) -> String {
        Path::new(file_name)
            .extension()
            .map(|ext| ext.to_string_lossy().to_string())
            .unwrap_or_else(|| "未知".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_drives() {
        let drives = FileSearchService::get_all_drives();
        // 至少应该有一个驱动器
        assert!(!drives.is_empty());
        println!("发现驱动器: {:?}", drives);
    }

    #[test]
    fn test_search_files() {
        // 搜索常见文件
        let result = FileSearchService::search_files(
            "desktop",
            Some(vec!["C:\\Users".to_string()]),
            Some(10),
        );
        assert!(result.is_ok());
        if let Ok(files) = result {
            println!("搜索结果数量: {}", files.len());
        }
    }
}
