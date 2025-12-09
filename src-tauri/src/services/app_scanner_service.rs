use crate::models::{AppItem, AppLauncherSettings, ItemType};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
use base64::{engine::general_purpose, Engine as _};

/// 默认扫描深度限制
const DEFAULT_MAX_DEPTH: usize = 5;
/// 批量解析快捷方式的大小
const SHORTCUT_BATCH_SIZE: usize = 50;

/// 应用扫描服务
pub struct AppScannerService;

impl AppScannerService {
    /// 扫描系统已安装的应用（使用默认设置）
    /// 如果指定了path参数，则只扫描该目录；否则扫描整个系统
    pub async fn scan_installed_apps(path: Option<String>) -> Result<Vec<AppItem>, String> {
        Self::scan_installed_apps_with_settings(path, AppLauncherSettings::default()).await
    }

    /// 扫描系统已安装的应用（使用指定设置）
    /// 如果指定了path参数，则只扫描该目录；否则扫描整个系统
    pub async fn scan_installed_apps_with_settings(
        path: Option<String>,
        settings: AppLauncherSettings,
    ) -> Result<Vec<AppItem>, String> {
        let mut apps = Vec::new();

        // 如果指定了路径，只扫描该目录
        if let Some(dir_path) = path {
            let path_buf = PathBuf::from(&dir_path);
            if !path_buf.exists() {
                return Err(format!("指定的路径不存在: {}", dir_path));
            }
            if !path_buf.is_dir() {
                return Err(format!("指定的路径不是目录: {}", dir_path));
            }

            // 使用用户配置的设置扫描指定目录
            apps = Self::scan_directory_with_settings(&path_buf, &settings)?;

            // 按名称排序
            apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            return Ok(apps);
        }

        // 如果没有指定路径，扫描整个系统并智能合并
        let mut scan_results = Vec::new();

        // 1. 扫描开始菜单
        let start_menu_apps = Self::scan_start_menu().await?;
        scan_results.push(start_menu_apps);

        // 2. 扫描常见安装目录 - 已禁用，会扫描到卸载程序等无用文件
        // let program_dirs_apps = Self::scan_program_directories().await?;
        // scan_results.push(program_dirs_apps);

        // 3. 扫描注册表（Windows）- 已禁用，容易产生重复应用
        // #[cfg(target_os = "windows")]
        // {
        //     let registry_apps = Self::scan_windows_registry().await?;
        //     scan_results.push(registry_apps);
        // }

        // 4. 扫描 shell:AppsFolder（Windows）- 已禁用，UWP应用体验不佳
        // #[cfg(target_os = "windows")]
        // {
        //     let shell_apps = Self::scan_shell_apps_folder().await?;
        //     scan_results.push(shell_apps);
        // }

        // 智能合并所有扫描结果
        apps = Self::merge_scan_results(scan_results);

        // 按名称排序
        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(apps)
    }

    /// 智能合并多个数据源的扫描结果
    ///
    /// 合并策略：
    /// 1. 基于路径去重：相同路径的应用只保留一个
    /// 2. 基于AUMID去重：UWP应用用aumid去重
    /// 3. 基于名称+发布者去重：相同名称和发布者的应用合并
    ///
    /// 信息合并优先级（当同一应用来自多个源时）：
    /// - 优先使用 shell_apps 的信息（最准确）
    /// - 其次使用 registry 的信息（有版本、发布者等元数据）
    /// - 最后使用 start_menu 的信息
    fn merge_scan_results(results: Vec<Vec<AppItem>>) -> Vec<AppItem> {
        // 使用HashMap进行去重和合并
        // Key为去重标识符，Value为应用信息
        let mut merged_map: HashMap<String, AppItem> = HashMap::new();

        // 路径规范化缓存，用于处理相同路径的不同表示方式
        let mut path_to_key: HashMap<String, String> = HashMap::new();

        // 规范化名称到key的映射，用于匹配相似名称的应用
        let mut name_to_key: HashMap<String, String> = HashMap::new();

        // 按优先级顺序处理：start_menu -> program_dirs -> registry -> shell_apps
        // 注意：我们按照数据质量从低到高的顺序处理，这样高质量数据会覆盖低质量数据
        for apps in results {
            for app in apps {
                // 生成去重键
                let dedup_key = Self::generate_dedup_key(&app);

                // 检查是否已存在（通过dedup_key）
                if let Some(existing_app) = merged_map.get_mut(&dedup_key) {
                    // 合并信息：保留最丰富的信息
                    Self::merge_app_info(existing_app, app);
                    continue;
                }

                // 检查是否有相同路径的应用（处理路径大小写、斜杠等差异）
                let normalized_path = Self::normalize_path(&app.path);
                if let Some(existing_key) = path_to_key.get(&normalized_path) {
                    if let Some(existing_app) = merged_map.get_mut(existing_key) {
                        Self::merge_app_info(existing_app, app);
                        continue;
                    }
                }

                // 如果有目标路径，也检查目标路径
                if let Some(target) = &app.install_location {
                    let normalized_target = Self::normalize_path(target);
                    if let Some(existing_key) = path_to_key.get(&normalized_target) {
                        if let Some(existing_app) = merged_map.get_mut(existing_key) {
                            Self::merge_app_info(existing_app, app);
                            continue;
                        }
                    }
                }

                // 检查是否有相似名称的应用（如 "DataGrip" 和 "DataGrip 2022.1.5"）
                let normalized_name = Self::normalize_app_name(&app.name);
                if let Some(existing_key) = name_to_key.get(&normalized_name) {
                    if let Some(existing_app) = merged_map.get_mut(existing_key) {
                        // 额外验证：检查是否真的是同一个应用
                        // 通过比较路径的目录部分或发布者
                        let should_merge = Self::should_merge_by_similarity(existing_app, &app);
                        if should_merge {
                            Self::merge_app_info(existing_app, app);
                            continue;
                        }
                    }
                }

                // 新应用，添加到各个map
                path_to_key.insert(normalized_path.clone(), dedup_key.clone());
                if let Some(target) = &app.install_location {
                    path_to_key.insert(Self::normalize_path(target), dedup_key.clone());
                }
                name_to_key.insert(normalized_name, dedup_key.clone());
                merged_map.insert(dedup_key, app);
            }
        }

        // 转换为Vec并返回
        merged_map.into_values().collect()
    }

    /// 判断两个应用是否应该基于相似性合并
    fn should_merge_by_similarity(existing: &AppItem, new_app: &AppItem) -> bool {
        // 如果发布者相同，可以合并
        if let (Some(pub1), Some(pub2)) = (&existing.publisher, &new_app.publisher) {
            if pub1.to_lowercase() == pub2.to_lowercase() {
                return true;
            }
        }

        // 如果路径在同一目录下，可以合并
        let path1 = Path::new(&existing.path);
        let path2 = Path::new(&new_app.path);

        // 对于快捷方式，使用目标路径
        let target1 = existing.install_location.as_ref()
            .map(|p| Path::new(p).parent())
            .flatten()
            .or_else(|| path1.parent());
        let target2 = new_app.install_location.as_ref()
            .map(|p| Path::new(p).parent())
            .flatten()
            .or_else(|| path2.parent());

        if let (Some(dir1), Some(dir2)) = (target1, target2) {
            if dir1 == dir2 {
                return true;
            }
            // 检查是否在同一个应用目录下（如 C:\Program Files\JetBrains\DataGrip）
            if let (Some(parent1), Some(parent2)) = (dir1.parent(), dir2.parent()) {
                if parent1 == parent2 {
                    return true;
                }
            }
        }

        // 检查核心名称是否相同（去除版本号后）
        let core1 = Self::extract_core_name(&existing.name);
        let core2 = Self::extract_core_name(&new_app.name);
        if !core1.is_empty() && core1 == core2 {
            return true;
        }

        false
    }

    /// 生成去重键
    ///
    /// 优先级：
    /// 1. UWP应用：使用AUMID
    /// 2. 快捷方式：使用install_location（目标路径）
    /// 3. 普通应用：使用路径
    /// 4. 如果有发布者和名称：使用"名称+发布者"组合
    fn generate_dedup_key(app: &AppItem) -> String {
        // UWP应用：使用AUMID作为唯一标识
        if let Some(aumid) = &app.aumid {
            return format!("aumid:{}", aumid);
        }

        // 快捷方式：使用目标路径（install_location）进行去重
        // 这样可以将指向同一exe的快捷方式和exe本身合并
        if app.item_type == ItemType::Shortcut {
            if let Some(target) = &app.install_location {
                if !target.is_empty() {
                    let normalized_target = Self::normalize_path(target);
                    return format!("path:{}", normalized_target);
                }
            }
        }

        // 普通应用：使用路径
        if !app.path.is_empty() && app.item_type != ItemType::UwpApp {
            let normalized_path = Self::normalize_path(&app.path);
            return format!("path:{}", normalized_path);
        }

        // 如果有发布者和名称，使用组合键
        if let Some(publisher) = &app.publisher {
            if !publisher.is_empty() && !app.name.is_empty() {
                return format!("name_pub:{}:{}", app.name.to_lowercase(), publisher.to_lowercase());
            }
        }

        // 最后使用应用ID
        format!("id:{}", app.id)
    }

    /// 规范化路径（处理大小写、斜杠等差异）
    fn normalize_path(path: &str) -> String {
        // 转小写
        let mut normalized = path.to_lowercase();

        // 统一使用反斜杠（Windows）
        normalized = normalized.replace('/', "\\");

        // 移除引号
        normalized = normalized.replace('"', "");

        // 移除尾部空格
        normalized = normalized.trim().to_string();

        normalized
    }

    /// 规范化应用名称（用于匹配相似名称）
    /// 例如：将 "DataGrip 2022.1.5" 转换为 "datagrip"
    fn normalize_app_name(name: &str) -> String {
        let mut normalized = name.to_lowercase();

        // 移除版本号模式（如 1.0.0, 2022.1.5, v1.2.3 等）
        let version_patterns = [
            r"\s+v?\d+(\.\d+)*\s*$",           // 结尾的版本号
            r"\s+\d{4}\.\d+(\.\d+)*\s*$",       // 年份格式版本号 (2022.1.5)
            r"\s+\(\d+(\.\d+)*\)\s*$",          // 括号内的版本号
            r"\s+-\s*\d+(\.\d+)*\s*$",          // 连字符后的版本号
        ];

        for pattern in &version_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                normalized = re.replace(&normalized, "").to_string();
            }
        }

        // 移除常见后缀
        let suffixes = [" - shortcut", " shortcut", " 快捷方式", "(x64)", "(x86)", "(64-bit)", "(32-bit)"];
        for suffix in &suffixes {
            normalized = normalized.trim_end_matches(suffix).to_string();
        }

        // 移除多余空格并 trim
        normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
        normalized.trim().to_string()
    }

    /// 提取应用的核心名称（用于相似度匹配）
    /// 例如：从 "DJ音乐盒" 和 "DJBox" 中都能提取出相似的标识
    fn extract_core_name(name: &str) -> String {
        let normalized = Self::normalize_app_name(name);

        // 只保留字母和数字，用于基本匹配
        normalized.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase()
    }

    /// 合并两个应用的信息
    ///
    /// 将source的信息合并到target中，遵循以下原则：
    /// 1. 优先保留shell_apps的信息（最准确）
    /// 2. 其次保留registry的信息（有元数据）
    /// 3. 如果target缺失某些字段而source有，则补充
    /// 4. 合并tags
    fn merge_app_info(target: &mut AppItem, source: AppItem) {
        // 判断来源优先级
        let target_priority = Self::get_source_priority(target.app_source.as_deref());
        let source_priority = Self::get_source_priority(source.app_source.as_deref());

        // 处理图标（需要提前处理以避免所有权问题）
        if source.icon.is_some() {
            if target.icon.is_none() {
                // target没有图标，使用source的
                target.icon = source.icon.clone();
            } else if source_priority > target_priority {
                // source优先级更高，使用source的
                target.icon = source.icon.clone();
            }
        }

        // 如果source的优先级更高，替换主要信息
        if source_priority > target_priority {
            // 更新app_source
            target.app_source = source.app_source.clone();

            // 更新item_type（如果source更精确）
            if source.item_type == ItemType::UwpApp {
                target.item_type = source.item_type.clone();
            }
        }

        // 合并元数据信息（优先保留已有的，补充缺失的）
        if target.publisher.is_none() && source.publisher.is_some() {
            target.publisher = source.publisher;
        }

        if target.version.is_none() && source.version.is_some() {
            target.version = source.version;
        }

        if target.description.is_none() && source.description.is_some() {
            target.description = source.description;
        }

        if target.install_location.is_none() && source.install_location.is_some() {
            target.install_location = source.install_location;
        }

        if target.aumid.is_none() && source.aumid.is_some() {
            target.aumid = source.aumid;
        }

        // 合并tags
        if let Some(source_tags) = source.tags {
            if let Some(target_tags) = &mut target.tags {
                // 合并tags，去重
                for tag in source_tags {
                    if !target_tags.contains(&tag) {
                        target_tags.push(tag);
                    }
                }
            } else {
                // target没有tags，直接使用source的
                target.tags = Some(source_tags);
            }
        }

        // 更新updated_at
        target.updated_at = chrono::Local::now().timestamp();
    }

    /// 获取数据源优先级
    ///
    /// shell_apps: 3 (最高优先级，信息最准确)
    /// registry: 2 (有版本、发布者等元数据)
    /// start_menu: 1 (基础信息)
    /// 其他: 0
    fn get_source_priority(source: Option<&str>) -> i32 {
        match source {
            Some("shell_apps") => 3,
            Some("registry") => 2,
            Some("start_menu") => 1,
            _ => 0,
        }
    }

    /// 扫描指定目录下的所有可执行文件
    /// settings参数可选,如果不传则使用默认设置(只扫描exe和lnk)
    fn scan_directory_for_executables(dir: &Path) -> Result<Vec<AppItem>, String> {
        Self::scan_directory_with_settings(dir, &AppLauncherSettings::default())
    }

    /// 使用指定设置扫描目录 - 优化版：并行扫描 + 深度限制
    fn scan_directory_with_settings(
        dir: &Path,
        settings: &AppLauncherSettings,
    ) -> Result<Vec<AppItem>, String> {
        if !dir.exists() {
            return Ok(Vec::new());
        }

        // 使用迭代 + 并行的方式扫描，避免深层递归
        Self::scan_directory_parallel(dir, settings, DEFAULT_MAX_DEPTH)
    }

    /// 并行扫描目录 - 迭代式广度优先遍历
    fn scan_directory_parallel(
        root: &Path,
        settings: &AppLauncherSettings,
        max_depth: usize,
    ) -> Result<Vec<AppItem>, String> {
        let apps = Arc::new(Mutex::new(Vec::new()));
        let allowed_extensions: Vec<String> = settings.allowed_extensions.clone();

        let mut dirs_to_scan = vec![(root.to_path_buf(), 0usize)];

        while !dirs_to_scan.is_empty() {
            let current_dirs: Vec<_> = dirs_to_scan.drain(..).collect();

            // 并行处理当前层级的目录
            let results: Vec<(Vec<AppItem>, Vec<(PathBuf, usize)>)> = current_dirs
                .par_iter()
                .filter_map(|(dir, depth)| {
                    if *depth > max_depth {
                        return None;
                    }

                    let mut found_apps = Vec::new();
                    let mut subdirs = Vec::new();

                    if let Ok(entries) = fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();

                            if path.is_dir() {
                                subdirs.push((path, depth + 1));
                            } else if path.is_file() {
                                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                                    let ext_lower = ext.to_lowercase();
                                    if allowed_extensions.iter().any(|e| e.to_lowercase() == ext_lower) {
                                        if let Ok(app) = Self::create_app_from_file(&path, ext) {
                                            found_apps.push(app);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Some((found_apps, subdirs))
                })
                .collect();

            // 收集结果
            for (found_apps, subdirs) in results {
                apps.lock().unwrap().extend(found_apps);
                dirs_to_scan.extend(subdirs);
            }
        }

        Ok(Arc::try_unwrap(apps).unwrap().into_inner().unwrap())
    }

    /// 从文件创建AppItem,自动判断文件类型
    fn create_app_from_file(path: &Path, extension: &str) -> Result<AppItem, String> {
        let ext_lower = extension.to_lowercase();

        match ext_lower.as_str() {
            "exe" => {
                // 过滤掉明显的非应用程序
                if !Self::is_likely_app_executable(path) {
                    return Err("Not a likely application executable".to_string());
                }

                let app_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let target_path = path.to_string_lossy().to_string();
                let app_id = Self::generate_app_id(&app_name, &target_path);

                let mut app =
                    AppItem::new_with_type(app_id, app_name, target_path.clone(), ItemType::Application);
                app.category = Some(Self::auto_categorize(&app.name, &app.path));
                app.icon = Self::extract_icon_base64(path);

                Ok(app)
            }
            "lnk" => {
                // 解析快捷方式
                Self::parse_shortcut(path)
            }
            "rdp" => {
                // 远程桌面连接文件
                let app_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Remote Desktop")
                    .to_string();

                let target_path = path.to_string_lossy().to_string();
                let app_id = Self::generate_app_id(&app_name, &target_path);

                let mut app =
                    AppItem::new_with_type(app_id, app_name, target_path, ItemType::RemoteDesktop);
                app.category = Some("remote".to_string());

                Ok(app)
            }
            "url" => {
                // URL链接文件
                let app_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Web Link")
                    .to_string();

                let target_path = path.to_string_lossy().to_string();
                let app_id = Self::generate_app_id(&app_name, &target_path);

                let mut app =
                    AppItem::new_with_type(app_id, app_name, target_path, ItemType::UrlLink);
                app.category = Some("web".to_string());

                Ok(app)
            }
            _ => {
                // 其他文件类型,作为普通文件处理
                let app_name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown File")
                    .to_string();

                let target_path = path.to_string_lossy().to_string();
                let app_id = Self::generate_app_id(&app_name, &target_path);

                let mut app = AppItem::new_with_type(app_id, app_name, target_path, ItemType::File);
                app.category = Some("file".to_string());

                Ok(app)
            }
        }
    }

    /// 扫描开始菜单
    async fn scan_start_menu() -> Result<Vec<AppItem>, String> {
        let mut apps = Vec::new();

        #[cfg(target_os = "windows")]
        {
            // 获取开始菜单路径
            let start_menu_paths = vec![
                Self::get_common_start_menu_path(),
                Self::get_user_start_menu_path(),
            ];

            for start_menu_path in start_menu_paths {
                if let Some(path) = start_menu_path {
                    if path.exists() {
                        let found_apps = Self::scan_directory_for_shortcuts(&path)?;
                        apps.extend(found_apps);
                    }
                }
            }
        }

        Ok(apps)
    }

    /// 获取公共开始菜单路径
    #[cfg(target_os = "windows")]
    fn get_common_start_menu_path() -> Option<PathBuf> {
        dirs::data_dir().map(|p| {
            p.parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("ProgramData")
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
        })
    }

    /// 获取用户开始菜单路径
    #[cfg(target_os = "windows")]
    fn get_user_start_menu_path() -> Option<PathBuf> {
        dirs::data_dir().map(|p| {
            p.join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
        })
    }

    /// 扫描目录查找快捷方式 - 优化版：使用并行扫描 + 批量解析
    fn scan_directory_for_shortcuts(dir: &Path) -> Result<Vec<AppItem>, String> {
        if !dir.exists() {
            return Ok(Vec::new());
        }

        // 第一步：并行收集所有快捷方式路径（使用迭代而非递归）
        let lnk_paths = Self::collect_shortcut_paths_parallel(dir, DEFAULT_MAX_DEPTH);

        // 第二步：批量解析快捷方式
        let apps = Self::batch_parse_shortcuts(&lnk_paths);

        Ok(apps)
    }

    /// 并行收集所有快捷方式路径
    fn collect_shortcut_paths_parallel(root: &Path, max_depth: usize) -> Vec<PathBuf> {
        let paths = Arc::new(Mutex::new(Vec::new()));
        let mut dirs_to_scan = vec![(root.to_path_buf(), 0usize)];

        while !dirs_to_scan.is_empty() {
            // 获取当前层级的所有目录
            let current_dirs: Vec<_> = dirs_to_scan.drain(..).collect();

            // 并行处理当前层级的目录
            let results: Vec<(Vec<PathBuf>, Vec<(PathBuf, usize)>)> = current_dirs
                .par_iter()
                .filter_map(|(dir, depth)| {
                    if *depth > max_depth {
                        return None;
                    }

                    let mut lnk_files = Vec::new();
                    let mut subdirs = Vec::new();

                    if let Ok(entries) = fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                subdirs.push((path, depth + 1));
                            } else if path.extension().and_then(|s| s.to_str()) == Some("lnk") {
                                lnk_files.push(path);
                            }
                        }
                    }

                    Some((lnk_files, subdirs))
                })
                .collect();

            // 收集结果
            for (lnk_files, subdirs) in results {
                paths.lock().unwrap().extend(lnk_files);
                dirs_to_scan.extend(subdirs);
            }
        }

        Arc::try_unwrap(paths).unwrap().into_inner().unwrap()
    }

    /// 批量解析快捷方式 - 一次 PowerShell 调用解析多个文件
    #[cfg(target_os = "windows")]
    fn batch_parse_shortcuts(lnk_paths: &[PathBuf]) -> Vec<AppItem> {
        use std::process::Command;

        if lnk_paths.is_empty() {
            return Vec::new();
        }

        let apps = Arc::new(Mutex::new(Vec::new()));

        // 分批处理
        lnk_paths
            .par_chunks(SHORTCUT_BATCH_SIZE)
            .for_each(|batch| {
                // 构建批量解析的 PowerShell 脚本
                // 添加 [Console]::OutputEncoding 强制使用 UTF-8 输出
                let paths_array: Vec<String> = batch
                    .iter()
                    .map(|p| format!("'{}'", p.display().to_string().replace("'", "''")))
                    .collect();

                let ps_script = format!(
                    r#"
                    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
                    [Console]::InputEncoding = [System.Text.Encoding]::UTF8
                    $OutputEncoding = [System.Text.Encoding]::UTF8
                    $sh = New-Object -ComObject WScript.Shell
                    $paths = @({})
                    foreach ($path in $paths) {{
                        try {{
                            $lnk = $sh.CreateShortcut($path)
                            Write-Output "$path|$($lnk.TargetPath)"
                        }} catch {{
                            Write-Output "$path|ERROR"
                        }}
                    }}
                    "#,
                    paths_array.join(",")
                );

                let output = Command::new("powershell")
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .env("PYTHONIOENCODING", "utf-8")  // 以防万一
                    .arg("-NoProfile")
                    .arg("-NonInteractive")
                    .arg("-OutputFormat")
                    .arg("Text")
                    .arg("-Command")
                    .arg(&ps_script)
                    .output();

                if let Ok(output) = output {
                    if output.status.success() {
                        // 尝试 UTF-8 解码，失败则使用本地编码
                        let stdout = Self::decode_powershell_output(&output.stdout);
                        let mut batch_apps = Vec::new();

                        for line in stdout.lines() {
                            if let Some((lnk_path_str, target_path)) = line.split_once('|') {
                                if target_path == "ERROR" || target_path.is_empty() {
                                    continue;
                                }
                                if !Path::new(target_path).exists() {
                                    continue;
                                }

                                let lnk_path = Path::new(lnk_path_str);
                                let app_name = lnk_path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("Unknown")
                                    .to_string();

                                let lnk_path_string = lnk_path_str.to_string();
                                let app_id = Self::generate_app_id(&app_name, &lnk_path_string);

                                let mut app = AppItem::new_with_type(
                                    app_id,
                                    app_name,
                                    lnk_path_string,
                                    ItemType::Shortcut,
                                );
                                app.category = Some(Self::auto_categorize(&app.name, target_path));
                                app.app_source = Some("start_menu".to_string());
                                // 存储目标路径，用于去重
                                app.install_location = Some(target_path.to_string());
                                // 图标提取延迟到需要时再执行，提升扫描速度
                                // app.icon = Self::extract_icon_base64(Path::new(target_path));

                                batch_apps.push(app);
                            }
                        }

                        apps.lock().unwrap().extend(batch_apps);
                    }
                }
            });

        Arc::try_unwrap(apps).unwrap().into_inner().unwrap()
    }

    /// 解码 PowerShell 输出，处理 Windows 编码问题
    /// Windows PowerShell 默认使用系统代码页(中文系统为GBK/CP936)输出
    #[cfg(target_os = "windows")]
    fn decode_powershell_output(bytes: &[u8]) -> String {
        use encoding_rs::GBK;

        if bytes.is_empty() {
            return String::new();
        }

        // 检查 UTF-8 BOM
        let bytes_to_decode = if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
            &bytes[3..]
        } else {
            bytes
        };

        // 尝试 UTF-8 解码
        if let Ok(s) = String::from_utf8(bytes_to_decode.to_vec()) {
            // 验证是否包含有效的中文字符或者全是ASCII
            // 如果 UTF-8 解码成功但包含替换字符，说明可能不是真正的 UTF-8
            if !s.contains('\u{FFFD}') {
                // 额外检查：如果包含高字节但不包含有效的中文，可能是误判
                let has_high_bytes = bytes_to_decode.iter().any(|&b| b > 127);
                let has_valid_chinese = s.chars().any(|c| c >= '\u{4E00}' && c <= '\u{9FFF}');

                // 如果有高字节但没有有效中文，尝试 GBK
                if has_high_bytes && !has_valid_chinese {
                    let (decoded, _, had_errors) = GBK.decode(bytes_to_decode);
                    if !had_errors {
                        return decoded.into_owned();
                    }
                }
                return s;
            }
        }

        // 尝试 GBK (Windows 中文系统默认编码)
        let (decoded, _, had_errors) = GBK.decode(bytes_to_decode);
        if !had_errors {
            return decoded.into_owned();
        }

        // 最后使用 lossy 转换
        String::from_utf8_lossy(bytes_to_decode).into_owned()
    }

    /// 非Windows平台的批量解析（占位）
    #[cfg(not(target_os = "windows"))]
    fn batch_parse_shortcuts(_lnk_paths: &[PathBuf]) -> Vec<AppItem> {
        Vec::new()
    }

    /// 解析Windows快捷方式(.lnk)
    #[cfg(target_os = "windows")]
    fn parse_shortcut(lnk_path: &Path) -> Result<AppItem, String> {
        use std::process::Command;

        // 使用PowerShell解析快捷方式，设置UTF-8输出编码
        let output = Command::new("powershell")
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-Command")
            .arg(format!(
                "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
                 $sh = New-Object -ComObject WScript.Shell; \
                 $lnk = $sh.CreateShortcut('{}'); \
                 Write-Output $lnk.TargetPath",
                lnk_path.display()
            ))
            .output()
            .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

        if !output.status.success() {
            return Err("Failed to parse shortcut".to_string());
        }

        let target_path = Self::decode_powershell_output(&output.stdout).trim().to_string();

        if target_path.is_empty() || !Path::new(&target_path).exists() {
            return Err("Invalid target path".to_string());
        }

        // 提取应用名称（使用快捷方式的文件名）
        let app_name = lnk_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // 使用快捷方式文件本身的路径，而不是目标路径
        // 这样可以避免多个指向同一程序的快捷方式被误判为重复
        let lnk_path_str = lnk_path.to_string_lossy().to_string();

        // 生成唯一ID（使用快捷方式路径）
        let app_id = Self::generate_app_id(&app_name, &lnk_path_str);

        let mut app = AppItem::new_with_type(app_id, app_name, lnk_path_str, ItemType::Shortcut);

        // 设置应用来源
        app.app_source = Some("start_menu".to_string());

        // 存储目标路径，用于去重
        app.install_location = Some(target_path.clone());

        // 尝试自动分类（基于目标程序路径进行分类）
        app.category = Some(Self::auto_categorize(&app.name, &target_path));

        // 提取图标（从目标程序提取）
        app.icon = Self::extract_icon_base64(Path::new(&target_path));

        Ok(app)
    }

    /// 非Windows平台的快捷方式解析（占位）
    #[cfg(not(target_os = "windows"))]
    fn parse_shortcut(_lnk_path: &Path) -> Result<AppItem, String> {
        Err("Shortcut parsing is only supported on Windows".to_string())
    }

    /// 扫描常见程序安装目录
    async fn scan_program_directories() -> Result<Vec<AppItem>, String> {
        let mut apps = Vec::new();

        #[cfg(target_os = "windows")]
        {
            let program_dirs = vec![
                PathBuf::from("C:\\Program Files"),
                PathBuf::from("C:\\Program Files (x86)"),
            ];

            for dir in program_dirs {
                if dir.exists() {
                    let found_apps = Self::scan_program_dir_shallow(&dir).await?;
                    apps.extend(found_apps);
                }
            }
        }

        Ok(apps)
    }

    /// 浅扫描程序目录（只扫描一级子目录）- 优化版：并行扫描
    async fn scan_program_dir_shallow(dir: &Path) -> Result<Vec<AppItem>, String> {
        // 首先收集所有一级子目录
        let subdirs: Vec<PathBuf> = match fs::read_dir(dir) {
            Ok(entries) => entries
                .flatten()
                .filter_map(|e| {
                    let path = e.path();
                    if path.is_dir() { Some(path) } else { None }
                })
                .collect(),
            Err(_) => return Ok(Vec::new()),
        };

        // 并行扫描所有子目录
        let apps: Vec<AppItem> = subdirs
            .par_iter()
            .flat_map(|subdir| {
                let mut found_apps = Vec::new();
                if let Ok(sub_entries) = fs::read_dir(subdir) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_file()
                            && sub_path.extension().and_then(|s| s.to_str()) == Some("exe")
                        {
                            if Self::is_likely_app_executable(&sub_path) {
                                if let Ok(app) = Self::create_app_from_exe_sync(&sub_path) {
                                    found_apps.push(app);
                                }
                            }
                        }
                    }
                }
                found_apps
            })
            .collect();

        Ok(apps)
    }

    /// 同步版本的从exe文件创建AppItem（用于并行处理）
    fn create_app_from_exe_sync(exe_path: &Path) -> Result<AppItem, String> {
        let app_name = exe_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let target_path = exe_path.to_string_lossy().to_string();
        let app_id = Self::generate_app_id(&app_name, &target_path);

        let mut app = AppItem::new(app_id, app_name, target_path);
        app.category = Some(Self::auto_categorize(&app.name, &app.path));

        Ok(app)
    }

    /// 判断exe文件是否可能是应用程序
    fn is_likely_app_executable(path: &Path) -> bool {
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let filename_lower = filename.to_lowercase();

        // 排除明显的非应用程序
        let excluded_keywords = [
            "unins",
            "uninst",
            "uninstall",
            "setup",
            "install",
            "update",
            "updater",
            "crash",
            "report",
            "helper",
            "service",
        ];

        !excluded_keywords
            .iter()
            .any(|keyword| filename_lower.contains(keyword))
    }

    /// 从exe文件创建AppItem
    async fn create_app_from_exe(exe_path: &Path) -> Result<AppItem, String> {
        let app_name = exe_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let target_path = exe_path.to_string_lossy().to_string();
        let app_id = Self::generate_app_id(&app_name, &target_path);

        let mut app = AppItem::new(app_id, app_name, target_path);
        app.category = Some(Self::auto_categorize(&app.name, &app.path));

        Ok(app)
    }

    /// 扫描Windows注册表
    #[cfg(target_os = "windows")]
    async fn scan_windows_registry() -> Result<Vec<AppItem>, String> {
        use winreg::enums::*;
        use winreg::RegKey;

        let mut apps = Vec::new();

        // 扫描注册表中的卸载信息
        let registry_paths = vec![
            (
                HKEY_LOCAL_MACHINE,
                "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            ),
            (
                HKEY_LOCAL_MACHINE,
                "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            ),
            (
                HKEY_CURRENT_USER,
                "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            ),
        ];

        for (hkey, subkey) in registry_paths {
            if let Ok(uninstall_key) = RegKey::predef(hkey).open_subkey(subkey) {
                for name in uninstall_key.enum_keys().flatten() {
                    if let Ok(app_key) = uninstall_key.open_subkey(&name) {
                        if let Ok(app) = Self::parse_registry_app(&app_key).await {
                            apps.push(app);
                        }
                    }
                }
            }
        }

        Ok(apps)
    }

    /// 非Windows平台的注册表扫描（占位）
    #[cfg(not(target_os = "windows"))]
    async fn scan_windows_registry() -> Result<Vec<AppItem>, String> {
        Ok(Vec::new())
    }

    /// 解析注册表中的应用信息
    #[cfg(target_os = "windows")]
    async fn parse_registry_app(app_key: &winreg::RegKey) -> Result<AppItem, String> {
        // 获取应用名称
        let display_name: String = app_key
            .get_value("DisplayName")
            .map_err(|_| "No DisplayName".to_string())?;

        // 过滤掉没有DisplayName的条目
        if display_name.trim().is_empty() {
            return Err("Empty DisplayName".to_string());
        }

        // 过滤掉系统更新和驱动程序
        let name_lower = display_name.to_lowercase();
        if name_lower.contains("update")
            || name_lower.contains("kb") && name_lower.contains("microsoft")
            || name_lower.contains("security update")
            || name_lower.contains("hotfix")
            || name_lower.contains("driver")
            || name_lower.starts_with("update for")
        {
            return Err("System update or driver".to_string());
        }

        // 获取UninstallString - 用于判断应用是否有效
        let uninstall_string: Result<String, _> = app_key.get_value("UninstallString");
        if uninstall_string.is_err() {
            // 没有卸载字符串的条目通常不是真正的应用程序
            return Err("No UninstallString".to_string());
        }

        // 获取其他注册表字段
        let install_location: Result<String, _> = app_key.get_value("InstallLocation");
        let display_icon: Result<String, _> = app_key.get_value("DisplayIcon");
        let publisher: Result<String, _> = app_key.get_value("Publisher");
        let display_version: Result<String, _> = app_key.get_value("DisplayVersion");

        // 尝试找到可执行文件路径
        let (exe_path, icon_path) = if let Ok(icon) = &display_icon {
            // DisplayIcon 可能是 "path,iconIndex" 格式，需要正确解析
            let parts: Vec<&str> = icon.split(',').collect();
            let icon_file = parts[0].trim().trim_matches('"');

            // 如果图标路径存在且是exe文件，使用它作为主路径
            if icon_file.ends_with(".exe") && Path::new(icon_file).exists() {
                (icon_file.to_string(), Some(icon_file.to_string()))
            } else if let Ok(location) = &install_location {
                // 尝试从安装位置找到主程序
                let main_exe = Self::find_main_executable(location);
                (main_exe.unwrap_or_else(|| location.clone()), Some(icon_file.to_string()))
            } else {
                // 如果图标路径存在，使用它
                if Path::new(icon_file).exists() {
                    (icon_file.to_string(), Some(icon_file.to_string()))
                } else {
                    return Err("No valid executable path found".to_string());
                }
            }
        } else if let Ok(location) = &install_location {
            // 尝试从安装位置找到主程序
            let main_exe = Self::find_main_executable(location);
            (main_exe.ok_or("No executable in install location")?.clone(), None)
        } else {
            return Err("No executable path found".to_string());
        };

        // 验证路径是否存在
        if !Path::new(&exe_path).exists() {
            return Err("Executable not found".to_string());
        }

        let app_id = Self::generate_app_id(&display_name, &exe_path);
        let mut app = AppItem::new(app_id, display_name, exe_path.clone());

        // 设置应用来源
        app.app_source = Some("registry".to_string());

        // 设置发布者
        if let Ok(pub_name) = publisher {
            if !pub_name.trim().is_empty() {
                app.publisher = Some(pub_name);
            }
        }

        // 设置版本
        if let Ok(ver) = display_version {
            if !ver.trim().is_empty() {
                app.version = Some(ver);
            }
        }

        // 设置安装位置
        if let Ok(location) = install_location {
            if !location.trim().is_empty() && Path::new(&location).exists() {
                app.install_location = Some(location);
            }
        }

        // 自动分类
        app.category = Some(Self::auto_categorize(&app.name, &app.path));

        // 提取图标 - 优先使用DisplayIcon指定的路径
        if let Some(icon_src) = icon_path {
            app.icon = Self::extract_icon_base64(Path::new(&icon_src));
        } else {
            app.icon = Self::extract_icon_base64(Path::new(&exe_path));
        }

        Ok(app)
    }

    /// 从安装目录中查找主可执行文件
    #[cfg(target_os = "windows")]
    fn find_main_executable(install_location: &str) -> Option<String> {
        let install_path = Path::new(install_location);
        if !install_path.exists() || !install_path.is_dir() {
            return None;
        }

        // 首先尝试查找与目录名相同的exe
        if let Some(dir_name) = install_path.file_name().and_then(|s| s.to_str()) {
            let potential_exe = install_path.join(format!("{}.exe", dir_name));
            if potential_exe.exists() {
                return Some(potential_exe.to_string_lossy().to_string());
            }
        }

        // 遍历安装目录，查找可能的主程序
        if let Ok(entries) = fs::read_dir(install_path) {
            let mut exe_files: Vec<PathBuf> = entries
                .flatten()
                .filter_map(|e| {
                    let path = e.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("exe") {
                        // 排除明显的非主程序exe
                        if Self::is_likely_app_executable(&path) {
                            Some(path)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            // 如果只有一个exe，直接返回
            if exe_files.len() == 1 {
                return Some(exe_files[0].to_string_lossy().to_string());
            }

            // 如果有多个exe，尝试找最可能的主程序（文件名不含特殊关键字的）
            exe_files.sort_by_key(|p| {
                let name = p.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                // 优先选择不含特殊关键字的exe
                if name.contains("launcher") || name.contains("main") || name.contains("app") {
                    0
                } else {
                    1
                }
            });

            if !exe_files.is_empty() {
                return Some(exe_files[0].to_string_lossy().to_string());
            }
        }

        None
    }

    /// 扫描 shell:AppsFolder 获取所有应用（包括UWP应用）
    #[cfg(target_os = "windows")]
    async fn scan_shell_apps_folder() -> Result<Vec<AppItem>, String> {
        use std::process::Command;

        // 使用 PowerShell 枚举 shell:AppsFolder
        let ps_script = r#"
            $ErrorActionPreference = 'SilentlyContinue'

            try {
                $shell = New-Object -ComObject Shell.Application
                $folder = $shell.NameSpace('shell:AppsFolder')

                if ($folder) {
                    foreach ($item in $folder.Items()) {
                        $name = $item.Name
                        $path = $item.Path

                        # 过滤掉无效项
                        if ($name -and $path) {
                            # 输出格式: Name|Path
                            Write-Output "$name|$path"
                        }
                    }
                }
            } catch {
                # 静默处理错误
            }
        "#;

        let output = Command::new("powershell")
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(ps_script)
            .output()
            .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

        if !output.status.success() {
            return Err("Failed to scan shell:AppsFolder".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut apps = Vec::new();

        for line in stdout.lines() {
            if let Some((name, path)) = line.split_once('|') {
                let name = name.trim();
                let path = path.trim();

                if name.is_empty() || path.is_empty() {
                    continue;
                }

                // 生成唯一ID
                let app_id = Self::generate_app_id(name, path);

                // 判断是否为UWP应用（通常AUMID包含'!'字符）
                let is_uwp = path.contains('!');
                let item_type = if is_uwp {
                    ItemType::UwpApp
                } else {
                    ItemType::Application
                };

                let mut app = AppItem::new_with_type(
                    app_id,
                    name.to_string(),
                    path.to_string(),
                    item_type,
                );

                // 设置应用来源
                app.app_source = Some("shell_apps".to_string());

                // 如果是UWP应用，保存AUMID
                if is_uwp {
                    app.aumid = Some(path.to_string());
                }

                // 自动分类
                app.category = Some(Self::auto_categorize(name, path));

                // 注意：shell:AppsFolder 中的应用图标提取比较复杂，暂时跳过
                // 后续可以通过其他方式获取UWP应用的图标

                apps.push(app);
            }
        }

        Ok(apps)
    }

    /// 非Windows平台的shell:AppsFolder扫描（占位）
    #[cfg(not(target_os = "windows"))]
    async fn scan_shell_apps_folder() -> Result<Vec<AppItem>, String> {
        Ok(Vec::new())
    }

    /// 生成应用唯一ID
    fn generate_app_id(name: &str, path: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{}{}", name, path).hash(&mut hasher);
        format!("app_{:x}", hasher.finish())
    }

    /// 自动分类应用
    /// 返回的分类ID需要与前端DEFAULT_CATEGORIES中的id匹配
    fn auto_categorize(name: &str, path: &str) -> String {
        let name_lower = name.to_lowercase();
        let path_lower = path.to_lowercase();
        let combined = format!("{} {}", name_lower, path_lower);

        // 开发工具 - 返回 "dev" 与前端分类ID匹配
        if combined.contains("visual studio")
            || combined.contains("vscode")
            || combined.contains("intellij")
            || combined.contains("pycharm")
            || combined.contains("webstorm")
            || combined.contains("android studio")
            || combined.contains("eclipse")
            || combined.contains("git")
            || combined.contains("docker")
            || combined.contains("postman")
            || combined.contains("sublime")
            || combined.contains("atom")
            || combined.contains("node")
            || combined.contains("python")
            || combined.contains("java")
            || combined.contains("cursor")
            || combined.contains("notepad++")
        {
            return "dev".to_string();
        }

        // 浏览器 - 返回 "browser"
        if combined.contains("chrome")
            || combined.contains("firefox")
            || combined.contains("edge")
            || combined.contains("safari")
            || combined.contains("opera")
            || combined.contains("brave")
        {
            return "browser".to_string();
        }

        // 办公软件 - 返回 "office"
        if combined.contains("word")
            || combined.contains("excel")
            || combined.contains("powerpoint")
            || combined.contains("outlook")
            || combined.contains("onenote")
            || combined.contains("teams")
            || combined.contains("wps")
            || combined.contains("foxit")
            || combined.contains("adobe acrobat")
            || combined.contains("pdf")
            || combined.contains("typora")
            || combined.contains("notion")
        {
            return "office".to_string();
        }

        // 设计工具 - 返回 "design"
        if combined.contains("photoshop")
            || combined.contains("illustrator")
            || combined.contains("figma")
            || combined.contains("sketch")
            || combined.contains("blender")
            || combined.contains("gimp")
            || combined.contains("inkscape")
            || combined.contains("xd")
            || combined.contains("axure")
        {
            return "design".to_string();
        }

        // 影音娱乐 - 返回 "media"
        if combined.contains("vlc")
            || combined.contains("potplayer")
            || combined.contains("spotify")
            || combined.contains("itunes")
            || combined.contains("music")
            || combined.contains("video")
            || combined.contains("player")
            || combined.contains("网易云")
            || combined.contains("qq音乐")
            || combined.contains("bilibili")
        {
            return "media".to_string();
        }

        // 游戏 - 返回 "game"
        if combined.contains("steam")
            || combined.contains("epic")
            || combined.contains("origin")
            || combined.contains("uplay")
            || combined.contains("game")
            || combined.contains("wegame")
        {
            return "game".to_string();
        }

        // 通讯工具 - 归入 "other"（前端没有单独的通讯分类）
        // 如果用户需要可以手动调整

        // 默认分类 - 返回 "other"
        "other".to_string()
    }

    /// 手动添加应用（用户指定路径）
    /// 支持添加文件、文件夹等各种类型
    pub async fn add_manual_app(
        path: String,
        name: Option<String>,
        settings: Option<AppLauncherSettings>,
    ) -> Result<AppItem, String> {
        let app_path = Path::new(&path);

        // 验证路径是否存在
        if !app_path.exists() {
            return Err("Path does not exist".to_string());
        }

        let settings = settings.unwrap_or_default();

        // 处理文件夹
        if app_path.is_dir() {
            let app_name = name.unwrap_or_else(|| {
                app_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Folder")
                    .to_string()
            });

            let app_id = Self::generate_app_id(&app_name, &path);
            let mut app = AppItem::new_with_type(app_id, app_name, path, ItemType::Folder);
            app.category = Some("folder".to_string());

            return Ok(app);
        }

        // 处理文件
        let extension = app_path.extension().and_then(|s| s.to_str());

        if let Some(ext) = extension {
            // 检查扩展名是否被允许
            if !settings.is_extension_allowed(ext) {
                return Err(format!(
                    "File type '.{}' is not in the allowed extensions list",
                    ext
                ));
            }

            // 使用create_app_from_file创建AppItem
            let mut app = Self::create_app_from_file(app_path, ext)?;

            // 如果用户指定了名称,则使用用户指定的名称
            if let Some(custom_name) = name {
                app.name = custom_name;
            }

            return Ok(app);
        }

        Err("File has no extension".to_string())
    }

    /// 提取文件图标并转换为base64
    #[cfg(target_os = "windows")]
    pub fn extract_icon_base64(file_path: &Path) -> Option<String> {
        use std::process::Command;

        // 使用PowerShell提取图标
        // 我们将使用一个简单的方法：调用PowerShell脚本提取图标到临时文件
        let temp_icon = std::env::temp_dir().join(format!("icon_{}.png", std::process::id()));

        // 添加错误处理和 SetErrorMode 来抑制系统错误对话框（如"损坏的映像"等）
        // 在 PowerShell 中调用 kernel32.dll 的 SetErrorMode 函数
        let ps_script = format!(
            r#"
            $ErrorActionPreference = 'SilentlyContinue'

            # 设置错误模式，禁用系统错误对话框
            $signature = @'
            [DllImport("kernel32.dll")]
            public static extern uint SetErrorMode(uint uMode);
'@
            try {{
                $kernel32 = Add-Type -MemberDefinition $signature -Name 'Kernel32' -Namespace 'Win32' -PassThru -ErrorAction SilentlyContinue
                # SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX = 0x8003
                $null = $kernel32::SetErrorMode(0x8003)
            }} catch {{}}

            try {{
                Add-Type -AssemblyName System.Drawing -ErrorAction SilentlyContinue
                $path = '{}'
                $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($path)
                if ($icon) {{
                    $bitmap = $icon.ToBitmap()
                    $bitmap.Save('{}', [System.Drawing.Imaging.ImageFormat]::Png)
                    $bitmap.Dispose()
                    $icon.Dispose()
                }}
            }} catch {{
                # 静默忽略图标提取错误
            }}
            "#,
            file_path.display().to_string().replace("'", "''"),
            temp_icon.display().to_string().replace("'", "''")
        );

        let output = Command::new("powershell")
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(&ps_script)
            .output();

        if output.is_err() {
            return None;
        }

        // 读取临时图标文件并转换为base64
        if temp_icon.exists() {
            if let Ok(icon_data) = fs::read(&temp_icon) {
                // 清理临时文件
                let _ = fs::remove_file(&temp_icon);

                // 转换为base64
                let base64_icon = general_purpose::STANDARD.encode(&icon_data);
                return Some(format!("data:image/png;base64,{}", base64_icon));
            }
        }

        None
    }

    /// 非Windows平台的图标提取（占位）
    #[cfg(not(target_os = "windows"))]
    pub fn extract_icon_base64(_file_path: &Path) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_app_id() {
        let id1 = AppScannerService::generate_app_id("VSCode", "C:\\vscode.exe");
        let id2 = AppScannerService::generate_app_id("VSCode", "C:\\vscode.exe");
        let id3 = AppScannerService::generate_app_id("Chrome", "C:\\chrome.exe");

        // 相同输入应产生相同ID
        assert_eq!(id1, id2);
        // 不同输入应产生不同ID
        assert_ne!(id1, id3);
        // ID应该以app_开头
        assert!(id1.starts_with("app_"));
    }

    #[test]
    fn test_auto_categorize_development() {
        let category = AppScannerService::auto_categorize(
            "Visual Studio Code",
            "C:\\Program Files\\VSCode\\Code.exe",
        );
        assert_eq!(category, "dev");

        let category = AppScannerService::auto_categorize("IntelliJ IDEA", "C:\\idea.exe");
        assert_eq!(category, "dev");
    }

    #[test]
    fn test_auto_categorize_browser() {
        let category = AppScannerService::auto_categorize("Google Chrome", "C:\\chrome.exe");
        assert_eq!(category, "browser");

        let category = AppScannerService::auto_categorize("Firefox", "C:\\firefox.exe");
        assert_eq!(category, "browser");
    }

    #[test]
    fn test_auto_categorize_office() {
        let category =
            AppScannerService::auto_categorize("Microsoft Word", "C:\\Office\\WINWORD.EXE");
        assert_eq!(category, "office");

        let category = AppScannerService::auto_categorize("WPS Office", "C:\\wps.exe");
        assert_eq!(category, "office");
    }

    #[test]
    fn test_auto_categorize_other() {
        let category = AppScannerService::auto_categorize("Unknown App", "C:\\app.exe");
        assert_eq!(category, "other");
    }

    #[test]
    fn test_is_likely_app_executable() {
        assert!(AppScannerService::is_likely_app_executable(Path::new(
            "C:\\app.exe"
        )));
        assert!(AppScannerService::is_likely_app_executable(Path::new(
            "C:\\MyApp.exe"
        )));

        assert!(!AppScannerService::is_likely_app_executable(Path::new(
            "C:\\uninstall.exe"
        )));
        assert!(!AppScannerService::is_likely_app_executable(Path::new(
            "C:\\setup.exe"
        )));
        assert!(!AppScannerService::is_likely_app_executable(Path::new(
            "C:\\updater.exe"
        )));
    }

    #[tokio::test]
    async fn test_add_manual_app_invalid_path() {
        let result =
            AppScannerService::add_manual_app("C:\\nonexistent\\app.exe".to_string(), None, None).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_path() {
        let path1 = AppScannerService::normalize_path("C:\\Program Files\\App\\app.exe");
        let path2 = AppScannerService::normalize_path("C:/Program Files/App/app.exe");
        let path3 = AppScannerService::normalize_path("\"C:\\Program Files\\App\\app.exe\"");

        assert_eq!(path1, path2);
        assert_eq!(path1, path3);
    }

    #[test]
    fn test_generate_dedup_key_with_aumid() {
        let mut app = AppItem::new(
            "test".to_string(),
            "Test App".to_string(),
            "test.exe".to_string(),
        );
        app.aumid = Some("Microsoft.WindowsCalculator_8wekyb3d8bbwe!App".to_string());

        let key = AppScannerService::generate_dedup_key(&app);
        assert!(key.starts_with("aumid:"));
    }

    #[test]
    fn test_generate_dedup_key_with_path() {
        let app = AppItem::new(
            "test".to_string(),
            "Test App".to_string(),
            "C:\\Program Files\\Test\\test.exe".to_string(),
        );

        let key = AppScannerService::generate_dedup_key(&app);
        assert!(key.starts_with("path:"));
    }

    #[test]
    fn test_generate_dedup_key_with_name_and_publisher() {
        let mut app = AppItem::new(
            "test".to_string(),
            "Test App".to_string(),
            "".to_string(), // 空路径
        );
        app.publisher = Some("Test Publisher".to_string());

        let key = AppScannerService::generate_dedup_key(&app);
        assert!(key.starts_with("name_pub:"));
    }

    #[test]
    fn test_get_source_priority() {
        assert_eq!(AppScannerService::get_source_priority(Some("shell_apps")), 3);
        assert_eq!(AppScannerService::get_source_priority(Some("registry")), 2);
        assert_eq!(AppScannerService::get_source_priority(Some("start_menu")), 1);
        assert_eq!(AppScannerService::get_source_priority(None), 0);
        assert_eq!(AppScannerService::get_source_priority(Some("unknown")), 0);
    }

    #[test]
    fn test_merge_app_info_with_higher_priority() {
        let mut target = AppItem::new(
            "test".to_string(),
            "Test App".to_string(),
            "C:\\test.exe".to_string(),
        );
        target.app_source = Some("start_menu".to_string());

        let mut source = AppItem::new(
            "test".to_string(),
            "Test App".to_string(),
            "C:\\test.exe".to_string(),
        );
        source.app_source = Some("registry".to_string());
        source.version = Some("1.0.0".to_string());
        source.publisher = Some("Test Publisher".to_string());

        AppScannerService::merge_app_info(&mut target, source);

        assert_eq!(target.app_source, Some("registry".to_string()));
        assert_eq!(target.version, Some("1.0.0".to_string()));
        assert_eq!(target.publisher, Some("Test Publisher".to_string()));
    }

    #[test]
    fn test_merge_app_info_with_tags() {
        let mut target = AppItem::new(
            "test".to_string(),
            "Test App".to_string(),
            "C:\\test.exe".to_string(),
        );
        target.tags = Some(vec!["dev".to_string(), "tool".to_string()]);

        let mut source = AppItem::new(
            "test".to_string(),
            "Test App".to_string(),
            "C:\\test.exe".to_string(),
        );
        source.tags = Some(vec!["tool".to_string(), "editor".to_string()]);

        AppScannerService::merge_app_info(&mut target, source);

        let merged_tags = target.tags.unwrap();
        assert_eq!(merged_tags.len(), 3); // dev, tool, editor
        assert!(merged_tags.contains(&"dev".to_string()));
        assert!(merged_tags.contains(&"tool".to_string()));
        assert!(merged_tags.contains(&"editor".to_string()));
    }

    #[test]
    fn test_merge_scan_results_dedup_by_path() {
        let mut app1 = AppItem::new(
            "test1".to_string(),
            "VSCode".to_string(),
            "C:\\Program Files\\VSCode\\Code.exe".to_string(),
        );
        app1.app_source = Some("start_menu".to_string());

        let mut app2 = AppItem::new(
            "test2".to_string(),
            "Visual Studio Code".to_string(),
            "C:\\Program Files\\VSCode\\Code.exe".to_string(), // 相同路径
        );
        app2.app_source = Some("registry".to_string());
        app2.version = Some("1.85.0".to_string());

        let results = vec![vec![app1], vec![app2]];
        let merged = AppScannerService::merge_scan_results(results);

        // 应该合并为一个应用
        assert_eq!(merged.len(), 1);
        // 应该保留更高优先级的信息
        assert_eq!(merged[0].app_source, Some("registry".to_string()));
        assert_eq!(merged[0].version, Some("1.85.0".to_string()));
    }

    #[test]
    fn test_merge_scan_results_dedup_by_aumid() {
        let mut app1 = AppItem::new(
            "test1".to_string(),
            "Calculator".to_string(),
            "shell:AppsFolder\\Microsoft.WindowsCalculator_8wekyb3d8bbwe!App".to_string(),
        );
        app1.item_type = ItemType::UwpApp;
        app1.aumid = Some("Microsoft.WindowsCalculator_8wekyb3d8bbwe!App".to_string());
        app1.app_source = Some("start_menu".to_string());

        let mut app2 = AppItem::new(
            "test2".to_string(),
            "Windows Calculator".to_string(),
            "shell:AppsFolder\\Microsoft.WindowsCalculator_8wekyb3d8bbwe!App".to_string(),
        );
        app2.item_type = ItemType::UwpApp;
        app2.aumid = Some("Microsoft.WindowsCalculator_8wekyb3d8bbwe!App".to_string());
        app2.app_source = Some("shell_apps".to_string());
        app2.publisher = Some("Microsoft Corporation".to_string());

        let results = vec![vec![app1], vec![app2]];
        let merged = AppScannerService::merge_scan_results(results);

        // 应该合并为一个应用
        assert_eq!(merged.len(), 1);
        // 应该保留shell_apps的信息（最高优先级）
        assert_eq!(merged[0].app_source, Some("shell_apps".to_string()));
        assert_eq!(merged[0].publisher, Some("Microsoft Corporation".to_string()));
    }

    #[test]
    fn test_merge_scan_results_no_duplicates() {
        let app1 = AppItem::new(
            "test1".to_string(),
            "VSCode".to_string(),
            "C:\\Program Files\\VSCode\\Code.exe".to_string(),
        );

        let app2 = AppItem::new(
            "test2".to_string(),
            "Chrome".to_string(),
            "C:\\Program Files\\Google\\Chrome\\chrome.exe".to_string(),
        );

        let app3 = AppItem::new(
            "test3".to_string(),
            "Firefox".to_string(),
            "C:\\Program Files\\Mozilla Firefox\\firefox.exe".to_string(),
        );

        let results = vec![vec![app1], vec![app2], vec![app3]];
        let merged = AppScannerService::merge_scan_results(results);

        // 没有重复，应该保留所有3个应用
        assert_eq!(merged.len(), 3);
    }

    #[test]
    fn test_merge_scan_results_dedup_shortcut_with_exe() {
        // 模拟：同一个应用，一个来自开始菜单（.lnk），一个来自注册表（.exe）
        let mut app1 = AppItem::new_with_type(
            "test1".to_string(),
            "Coodesker".to_string(),
            "C:\\Users\\Test\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Coodesker.lnk".to_string(),
            ItemType::Shortcut,
        );
        app1.app_source = Some("start_menu".to_string());
        // 快捷方式的目标路径
        app1.install_location = Some("C:\\Program Files\\Coodesker\\Coodesker.exe".to_string());

        let mut app2 = AppItem::new(
            "test2".to_string(),
            "Coodesker".to_string(),
            "C:\\Program Files\\Coodesker\\Coodesker.exe".to_string(),
        );
        app2.app_source = Some("registry".to_string());
        app2.version = Some("1.0.0".to_string());
        app2.publisher = Some("Coodesker Team".to_string());

        let results = vec![vec![app1], vec![app2]];
        let merged = AppScannerService::merge_scan_results(results);

        // 应该合并为一个应用
        assert_eq!(merged.len(), 1);
        // 应该保留registry的版本和发布者信息
        assert!(merged[0].version.is_some());
        assert!(merged[0].publisher.is_some());
    }

    #[test]
    fn test_merge_scan_results_dedup_similar_names() {
        // 模拟：同一个应用，名称不同（带版本号 vs 不带版本号）
        let mut app1 = AppItem::new(
            "test1".to_string(),
            "DataGrip".to_string(),
            "C:\\Program Files\\JetBrains\\DataGrip 2022.1\\bin\\datagrip64.exe".to_string(),
        );
        app1.app_source = Some("start_menu".to_string());

        let mut app2 = AppItem::new(
            "test2".to_string(),
            "DataGrip 2022.1.5".to_string(),
            "C:\\Program Files\\JetBrains\\DataGrip 2022.1\\bin\\datagrip64.exe".to_string(),
        );
        app2.app_source = Some("registry".to_string());
        app2.publisher = Some("JetBrains s.r.o.".to_string());
        app2.version = Some("2022.1.5".to_string());

        let results = vec![vec![app1], vec![app2]];
        let merged = AppScannerService::merge_scan_results(results);

        // 应该合并为一个应用（路径相同）
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn test_normalize_app_name() {
        // 测试名称规范化
        assert_eq!(AppScannerService::normalize_app_name("DataGrip 2022.1.5"), "datagrip");
        assert_eq!(AppScannerService::normalize_app_name("VSCode 1.85.0"), "vscode");
        assert_eq!(AppScannerService::normalize_app_name("Chrome"), "chrome");
        assert_eq!(AppScannerService::normalize_app_name("Cursor 0.45.14"), "cursor");
    }

    #[test]
    fn test_extract_core_name() {
        // 测试核心名称提取
        assert_eq!(AppScannerService::extract_core_name("DataGrip 2022.1.5"), "datagrip");
        assert_eq!(AppScannerService::extract_core_name("DataGrip"), "datagrip");
    }
}
