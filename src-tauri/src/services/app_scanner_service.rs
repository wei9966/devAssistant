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

        // 如果没有指定路径，扫描整个系统
        let mut app_map: HashMap<String, AppItem> = HashMap::new();

        // 1. 扫描开始菜单
        let start_menu_apps = Self::scan_start_menu().await?;
        for app in start_menu_apps {
            app_map.insert(app.id.clone(), app);
        }

        // 2. 扫描常见安装目录
        let program_dirs_apps = Self::scan_program_directories().await?;
        for app in program_dirs_apps {
            if !app_map.contains_key(&app.id) {
                app_map.insert(app.id.clone(), app);
            }
        }

        // 3. 扫描注册表（Windows）
        #[cfg(target_os = "windows")]
        {
            let registry_apps = Self::scan_windows_registry().await?;
            for app in registry_apps {
                if !app_map.contains_key(&app.id) {
                    app_map.insert(app.id.clone(), app);
                }
            }
        }

        // 转换为Vec
        apps.extend(app_map.into_values());

        // 按名称排序
        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(apps)
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
                let paths_array: Vec<String> = batch
                    .iter()
                    .map(|p| format!("'{}'", p.display().to_string().replace("'", "''")))
                    .collect();

                let ps_script = format!(
                    r#"
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
                    .arg("-NoProfile")
                    .arg("-NonInteractive")
                    .arg("-Command")
                    .arg(&ps_script)
                    .output();

                if let Ok(output) = output {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
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

    /// 非Windows平台的批量解析（占位）
    #[cfg(not(target_os = "windows"))]
    fn batch_parse_shortcuts(_lnk_paths: &[PathBuf]) -> Vec<AppItem> {
        Vec::new()
    }

    /// 解析Windows快捷方式(.lnk)
    #[cfg(target_os = "windows")]
    fn parse_shortcut(lnk_path: &Path) -> Result<AppItem, String> {
        use std::process::Command;

        // 使用PowerShell解析快捷方式
        let output = Command::new("powershell")
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-Command")
            .arg(format!(
                "$sh = New-Object -ComObject WScript.Shell; \
                 $lnk = $sh.CreateShortcut('{}'); \
                 Write-Output $lnk.TargetPath",
                lnk_path.display()
            ))
            .output()
            .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

        if !output.status.success() {
            return Err("Failed to parse shortcut".to_string());
        }

        let target_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

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
        use winreg::RegKey;

        // 获取应用名称
        let display_name: String = app_key
            .get_value("DisplayName")
            .map_err(|_| "No DisplayName".to_string())?;

        // 获取安装位置或可执行文件路径
        let install_location: Result<String, _> = app_key.get_value("InstallLocation");
        let display_icon: Result<String, _> = app_key.get_value("DisplayIcon");

        // 尝试找到可执行文件路径
        let exe_path = if let Ok(icon) = display_icon {
            // DisplayIcon通常包含exe路径
            icon.split(',').next().unwrap_or("").trim().to_string()
        } else if let Ok(location) = install_location {
            location
        } else {
            return Err("No executable path found".to_string());
        };

        // 验证路径是否存在
        if !Path::new(&exe_path).exists() {
            return Err("Executable not found".to_string());
        }

        let app_id = Self::generate_app_id(&display_name, &exe_path);
        let mut app = AppItem::new(app_id, display_name, exe_path);
        app.category = Some(Self::auto_categorize(&app.name, &app.path));

        Ok(app)
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
        assert_eq!(category, "development");

        let category = AppScannerService::auto_categorize("IntelliJ IDEA", "C:\\idea.exe");
        assert_eq!(category, "development");
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
}
