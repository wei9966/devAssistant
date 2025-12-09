use crate::models::{
    AppItem, AppSearchParams, Category, ItemType, LaunchHistory, LaunchResult, SyncResult, Workflow, WorkflowLaunchResult,
};
use crate::services::app_scanner_service::AppScannerService;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::path::Path;

/// 应用启动器服务
pub struct AppLauncherService {
    db: Arc<Mutex<Connection>>,
}

impl AppLauncherService {
    /// 创建新的服务实例
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    // ==================== 应用管理 ====================

    /// 获取所有应用
    pub async fn get_all_apps(&self) -> Result<Vec<AppItem>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db
            .prepare("SELECT id, name, path, icon, category, tags, launch_count, last_launched_at, is_pinned, is_hidden, launch_args, created_at, updated_at, app_source FROM apps ORDER BY name")
            .map_err(|e| e.to_string())?;

        let apps = stmt
            .query_map([], |row| {
                Ok(AppItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    icon: row.get(3)?,
                    category: row.get(4)?,
                    tags: AppItem::tags_from_json(row.get::<_, Option<String>>(5)?.as_deref()),
                    launch_count: row.get(6)?,
                    last_launched_at: row.get(7)?,
                    is_pinned: row.get::<_, i32>(8)? == 1,
                    is_hidden: row.get::<_, i32>(9)? == 1,
                    launch_args: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    app_source: row.get(13)?,
                    item_type: ItemType::default(),
                    aumid: None,
                    publisher: None,
                    version: None,
                    description: None,
                    install_location: None,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(apps)
    }

    /// 根据ID获取应用
    pub async fn get_app_by_id(&self, id: &str) -> Result<AppItem, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db
            .prepare("SELECT id, name, path, icon, category, tags, launch_count, last_launched_at, is_pinned, is_hidden, launch_args, created_at, updated_at, app_source FROM apps WHERE id = ?")
            .map_err(|e| e.to_string())?;

        let app = stmt
            .query_row([id], |row| {
                Ok(AppItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    icon: row.get(3)?,
                    category: row.get(4)?,
                    tags: AppItem::tags_from_json(row.get::<_, Option<String>>(5)?.as_deref()),
                    launch_count: row.get(6)?,
                    last_launched_at: row.get(7)?,
                    is_pinned: row.get::<_, i32>(8)? == 1,
                    is_hidden: row.get::<_, i32>(9)? == 1,
                    launch_args: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    app_source: row.get(13)?,
                    item_type: ItemType::default(),
                    aumid: None,
                    publisher: None,
                    version: None,
                    description: None,
                    install_location: None,
                })
            })
            .map_err(|e| format!("App not found: {}", e))?;

        Ok(app)
    }

    /// 搜索应用
    pub async fn search_apps(&self, params: AppSearchParams) -> Result<Vec<AppItem>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        let mut sql = String::from(
            "SELECT id, name, path, icon, category, tags, launch_count, last_launched_at, is_pinned, is_hidden, launch_args, created_at, updated_at, app_source FROM apps WHERE 1=1"
        );
        let mut conditions = Vec::new();

        // 关键词搜索
        if let Some(keyword) = &params.keyword {
            if !keyword.is_empty() {
                sql.push_str(" AND (name LIKE ?1 OR path LIKE ?1 OR tags LIKE ?1)");
                conditions.push(format!("%{}%", keyword));
            }
        }

        // 分类筛选
        if let Some(category) = &params.category {
            sql.push_str(&format!(" AND category = ?{}", conditions.len() + 1));
            conditions.push(category.clone());
        }

        // 置顶筛选
        if let Some(is_pinned) = params.is_pinned {
            sql.push_str(&format!(" AND is_pinned = ?{}", conditions.len() + 1));
            conditions.push(if is_pinned { "1" } else { "0" }.to_string());
        }

        // 隐藏筛选
        if let Some(is_hidden) = params.is_hidden {
            sql.push_str(&format!(" AND is_hidden = ?{}", conditions.len() + 1));
            conditions.push(if is_hidden { "1" } else { "0" }.to_string());
        }

        // 排序：置顶 > 启动次数 > 最近启动时间 > 名称
        sql.push_str(" ORDER BY is_pinned DESC, launch_count DESC, last_launched_at DESC, name");

        // 限制数量
        if let Some(limit) = params.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;

        let params_slice: Vec<&dyn rusqlite::ToSql> = conditions
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let apps = stmt
            .query_map(params_slice.as_slice(), |row| {
                Ok(AppItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    icon: row.get(3)?,
                    category: row.get(4)?,
                    tags: AppItem::tags_from_json(row.get::<_, Option<String>>(5)?.as_deref()),
                    launch_count: row.get(6)?,
                    last_launched_at: row.get(7)?,
                    is_pinned: row.get::<_, i32>(8)? == 1,
                    is_hidden: row.get::<_, i32>(9)? == 1,
                    launch_args: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    app_source: row.get(13)?,
                    item_type: ItemType::default(),
                    aumid: None,
                    publisher: None,
                    version: None,
                    description: None,
                    install_location: None,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(apps)
    }

    /// 添加应用（返回保存后的完整应用数据，包含自动提取的图标）
    pub async fn add_app(&self, app: AppItem) -> Result<AppItem, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        // 检查图标是否为空（None 或空字符串都视为没有图标）
        let has_icon = app.icon.as_ref().map_or(false, |s| !s.trim().is_empty());

        // 如果应用没有图标，尝试自动提取
        let icon = if !has_icon && !app.path.is_empty() {
            AppScannerService::extract_icon_base64(Path::new(&app.path))
        } else {
            app.icon.clone()
        };

        db.execute(
            "INSERT INTO apps (id, name, path, icon, category, tags, launch_count, last_launched_at, is_pinned, is_hidden, launch_args, created_at, updated_at, app_source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            rusqlite::params![
                app.id,
                app.name,
                app.path,
                icon,
                app.category,
                app.tags_to_json(),
                app.launch_count,
                app.last_launched_at,
                if app.is_pinned { 1 } else { 0 },
                if app.is_hidden { 1 } else { 0 },
                app.launch_args,
                app.created_at,
                app.updated_at,
                app.app_source,
            ],
        )
        .map_err(|e| e.to_string())?;

        // 返回包含图标的完整应用数据
        let mut saved_app = app;
        saved_app.icon = icon;
        Ok(saved_app)
    }

    /// 更新应用
    pub async fn update_app(&self, app: AppItem) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        let now = chrono::Local::now().timestamp();

        db.execute(
            "UPDATE apps SET name = ?1, path = ?2, icon = ?3, category = ?4, tags = ?5,
             launch_count = ?6, last_launched_at = ?7, is_pinned = ?8, is_hidden = ?9,
             launch_args = ?10, updated_at = ?11, app_source = ?12 WHERE id = ?13",
            rusqlite::params![
                app.name,
                app.path,
                app.icon,
                app.category,
                app.tags_to_json(),
                app.launch_count,
                app.last_launched_at,
                if app.is_pinned { 1 } else { 0 },
                if app.is_hidden { 1 } else { 0 },
                app.launch_args,
                now,
                app.app_source,
                app.id,
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 删除应用
    pub async fn delete_app(&self, id: &str) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        // 删除应用的启动历史
        db.execute("DELETE FROM launch_history WHERE app_id = ?", [id])
            .map_err(|e| e.to_string())?;

        // 删除应用
        db.execute("DELETE FROM apps WHERE id = ?", [id])
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 同步扫描的应用到数据库
    ///
    /// 此函数执行增量同步，比较扫描结果和数据库中的现有应用，并执行以下操作：
    /// - 新应用：添加到数据库
    /// - 已存在应用：更新元数据，但保留用户自定义数据
    /// - 已删除应用：标记为隐藏（除非是手动添加的应用）
    ///
    /// # 参数
    /// * `scanned_apps` - 扫描到的应用列表
    ///
    /// # 返回
    /// 返回同步结果，包含添加、更新、删除和未变化的应用数量
    pub async fn sync_scanned_apps(&self, scanned_apps: Vec<AppItem>) -> Result<SyncResult, String> {
        use std::collections::{HashMap, HashSet};

        let mut result = SyncResult::new();

        // 获取数据库中现有的所有应用
        let existing_apps = self.get_all_apps().await?;

        // 创建扫描应用的映射（按路径索引，因为路径是唯一标识符）
        let mut scanned_map: HashMap<String, AppItem> = scanned_apps
            .into_iter()
            .map(|app| (app.path.to_lowercase(), app))
            .collect();

        // 创建数据库应用的映射（按路径索引）
        let mut existing_map: HashMap<String, AppItem> = existing_apps
            .into_iter()
            .map(|app| (app.path.to_lowercase(), app))
            .collect();

        // 记录已处理的路径
        let mut processed_paths: HashSet<String> = HashSet::new();

        // 处理扫描到的应用
        for (path, scanned_app) in scanned_map.iter() {
            processed_paths.insert(path.clone());

            if let Some(existing_app) = existing_map.get(path) {
                // 应用已存在：检查是否需要更新
                let needs_update =
                    existing_app.name != scanned_app.name ||
                    existing_app.version != scanned_app.version ||
                    existing_app.publisher != scanned_app.publisher ||
                    existing_app.install_location != scanned_app.install_location ||
                    existing_app.description != scanned_app.description ||
                    existing_app.aumid != scanned_app.aumid ||
                    existing_app.app_source != scanned_app.app_source;

                if needs_update {
                    // 更新应用：只更新元数据，保留用户自定义数据
                    let mut updated_app = existing_app.clone();

                    // 更新元数据字段
                    updated_app.name = scanned_app.name.clone();
                    updated_app.path = scanned_app.path.clone();
                    updated_app.version = scanned_app.version.clone();
                    updated_app.publisher = scanned_app.publisher.clone();
                    updated_app.install_location = scanned_app.install_location.clone();
                    updated_app.description = scanned_app.description.clone();
                    updated_app.aumid = scanned_app.aumid.clone();
                    updated_app.app_source = scanned_app.app_source.clone();
                    updated_app.item_type = scanned_app.item_type.clone();

                    // 如果扫描到的应用有图标，且现有应用没有图标，则更新图标
                    if scanned_app.icon.is_some() && existing_app.icon.is_none() {
                        updated_app.icon = scanned_app.icon.clone();
                    }

                    // 保留用户自定义数据：
                    // - category（用户可能已手动分类）
                    // - tags（用户标签）
                    // - is_pinned（置顶状态）
                    // - launch_count 和 last_launched_at（使用统计）
                    // - launch_args（自定义启动参数）
                    // - is_hidden（隐藏状态）

                    self.update_app(updated_app).await?;
                    result.updated += 1;
                } else {
                    // 应用未变化
                    result.unchanged += 1;
                }
            } else {
                // 新应用：添加到数据库
                self.add_app(scanned_app.clone()).await?;
                result.added += 1;
            }
        }

        // 处理数据库中但扫描不到的应用
        for (path, existing_app) in existing_map.iter() {
            if processed_paths.contains(path) {
                continue; // 已在上面处理过
            }

            // 检查是否是手动添加的应用
            let is_manual = existing_app.app_source.as_deref() == Some("manual");

            if !is_manual && !existing_app.is_hidden {
                // 非手动添加的应用，且当前未隐藏，则标记为隐藏
                let mut hidden_app = existing_app.clone();
                hidden_app.is_hidden = true;
                self.update_app(hidden_app).await?;
                result.removed += 1;
            } else if is_manual {
                // 手动添加的应用：保留，不做任何处理
                result.unchanged += 1;
            } else {
                // 已经是隐藏状态，不重复计数
                result.unchanged += 1;
            }
        }

        Ok(result)
    }

    // ==================== 应用启动 ====================

    /// 启动应用
    pub async fn launch_app(&self, id: &str) -> Result<LaunchResult, String> {
        // 获取应用信息
        let app = self.get_app_by_id(id).await?;

        // 执行启动
        let result = Self::execute_launch(&app).await;

        // 如果启动成功，更新统计信息
        if result.success {
            let _ = self.record_launch(&app.id).await;
            let _ = self.update_launch_stats(&app.id).await;
        }

        Ok(result)
    }

    /// 执行应用启动
    async fn execute_launch(app: &AppItem) -> LaunchResult {
        // 获取应用程序所在的目录作为工作目录
        let working_dir = Path::new(&app.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string());

        #[cfg(target_os = "windows")]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use std::mem;
            use std::ptr;
            use winapi::um::shellapi::{ShellExecuteExW, SHELLEXECUTEINFOW, SEE_MASK_FLAG_NO_UI, SEE_MASK_NOCLOSEPROCESS};
            use winapi::um::winuser::SW_SHOWNORMAL;

            // 将字符串转换为宽字符(UTF-16)
            fn to_wide(s: &str) -> Vec<u16> {
                OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
            }

            // 根据不同的item_type使用不同的启动方式
            let (operation, file, parameters, directory) = match app.item_type {
                ItemType::Folder => {
                    // 文件夹,使用explorer打开
                    ("explore", app.path.as_str(), None, None)
                }
                _ => {
                    // 应用程序、快捷方式、URL链接、远程桌面、普通文件
                    // 都使用 "open" 操作
                    ("open", app.path.as_str(), app.launch_args.as_deref(), working_dir.as_deref())
                }
            };

            let operation_wide = to_wide(operation);
            let file_wide = to_wide(file);
            let parameters_wide = parameters.map(|p| to_wide(p));
            let directory_wide = directory.map(|d| to_wide(d));

            // 使用 ShellExecuteExW 并设置 SEE_MASK_FLAG_NO_UI 来禁止系统错误弹窗
            let mut sei: SHELLEXECUTEINFOW = unsafe { mem::zeroed() };
            sei.cbSize = mem::size_of::<SHELLEXECUTEINFOW>() as u32;
            // SEE_MASK_FLAG_NO_UI: 禁止显示错误对话框（如 DLL 加载失败等系统错误）
            sei.fMask = SEE_MASK_FLAG_NO_UI | SEE_MASK_NOCLOSEPROCESS;
            sei.hwnd = ptr::null_mut();
            sei.lpVerb = operation_wide.as_ptr();
            sei.lpFile = file_wide.as_ptr();
            sei.lpParameters = parameters_wide.as_ref().map_or(ptr::null(), |p| p.as_ptr());
            sei.lpDirectory = directory_wide.as_ref().map_or(ptr::null(), |d| d.as_ptr());
            sei.nShow = SW_SHOWNORMAL;

            let success = unsafe { ShellExecuteExW(&mut sei) };

            if success != 0 {
                // 关闭进程句柄（如果有的话）
                if !sei.hProcess.is_null() {
                    unsafe {
                        winapi::um::handleapi::CloseHandle(sei.hProcess);
                    }
                }
                LaunchResult::success(app.id.clone(), app.name.clone())
            } else {
                // 获取最后一个错误码
                let error_code = unsafe { winapi::um::errhandlingapi::GetLastError() };
                let error_msg = match error_code {
                    0 => "操作成功",
                    2 => "文件未找到",
                    3 => "路径未找到",
                    5 => "访问被拒绝",
                    8 => "内存不足",
                    11 => "可执行文件格式无效",
                    26 => "共享冲突",
                    27 => "文件关联不完整",
                    28 => "DDE操作超时",
                    29 => "DDE操作失败",
                    30 => "DDE操作繁忙",
                    31 => "没有关联的应用程序",
                    32 => "DLL未找到",
                    1155 => "没有关联的应用程序打开此文件类型",
                    _ => "未知错误",
                };
                LaunchResult::failure(
                    app.id.clone(),
                    app.name.clone(),
                    format!("{} (错误码: {})", error_msg, error_code),
                )
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            LaunchResult::failure(
                app.id.clone(),
                app.name.clone(),
                "Launch is only supported on Windows".to_string(),
            )
        }
    }

    /// 记录启动历史
    async fn record_launch(&self, app_id: &str) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let now = chrono::Local::now().timestamp();

        db.execute(
            "INSERT INTO launch_history (app_id, launched_at) VALUES (?1, ?2)",
            rusqlite::params![app_id, now],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 更新启动统计
    async fn update_launch_stats(&self, app_id: &str) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let now = chrono::Local::now().timestamp();

        db.execute(
            "UPDATE apps SET launch_count = launch_count + 1, last_launched_at = ?1 WHERE id = ?2",
            rusqlite::params![now, app_id],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    // ==================== 工作流管理 ====================

    /// 获取所有工作流
    pub async fn get_all_workflows(&self) -> Result<Vec<Workflow>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db
            .prepare("SELECT id, name, app_ids, launch_delay, created_at, updated_at FROM workflows ORDER BY name")
            .map_err(|e| e.to_string())?;

        let workflows = stmt
            .query_map([], |row| {
                let app_ids_json: String = row.get(2)?;
                Ok(Workflow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    app_ids: Workflow::app_ids_from_json(&app_ids_json),
                    launch_delay: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(workflows)
    }

    /// 根据ID获取工作流
    pub async fn get_workflow_by_id(&self, id: &str) -> Result<Workflow, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db
            .prepare("SELECT id, name, app_ids, launch_delay, created_at, updated_at FROM workflows WHERE id = ?")
            .map_err(|e| e.to_string())?;

        let workflow = stmt
            .query_row([id], |row| {
                let app_ids_json: String = row.get(2)?;
                Ok(Workflow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    app_ids: Workflow::app_ids_from_json(&app_ids_json),
                    launch_delay: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Workflow not found: {}", e))?;

        Ok(workflow)
    }

    /// 添加工作流
    pub async fn add_workflow(&self, workflow: Workflow) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        db.execute(
            "INSERT INTO workflows (id, name, app_ids, launch_delay, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                workflow.id,
                workflow.name,
                workflow.app_ids_to_json(),
                workflow.launch_delay,
                workflow.created_at,
                workflow.updated_at,
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 更新工作流
    pub async fn update_workflow(&self, workflow: Workflow) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let now = chrono::Local::now().timestamp();

        db.execute(
            "UPDATE workflows SET name = ?1, app_ids = ?2, launch_delay = ?3, updated_at = ?4 WHERE id = ?5",
            rusqlite::params![
                workflow.name,
                workflow.app_ids_to_json(),
                workflow.launch_delay,
                now,
                workflow.id,
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 删除工作流
    pub async fn delete_workflow(&self, id: &str) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        db.execute("DELETE FROM workflows WHERE id = ?", [id])
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 启动工作流
    pub async fn launch_workflow(&self, id: &str) -> Result<WorkflowLaunchResult, String> {
        let workflow = self.get_workflow_by_id(id).await?;
        let mut result = WorkflowLaunchResult::new(workflow.id.clone(), workflow.name.clone());

        let delay_ms = workflow.launch_delay.unwrap_or(0);

        for app_id in &workflow.app_ids {
            // 启动应用
            let launch_result = self
                .launch_app(app_id)
                .await
                .unwrap_or_else(|e| LaunchResult::failure(app_id.clone(), app_id.clone(), e));

            result.add_result(launch_result);

            // 延迟
            if delay_ms > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms as u64)).await;
            }
        }

        Ok(result)
    }

    // ==================== 分类管理 ====================

    /// 获取所有分类
    pub async fn get_all_categories(&self) -> Result<Vec<Category>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db
            .prepare("SELECT id, name, color, icon, sort_order, created_at FROM categories ORDER BY sort_order, name")
            .map_err(|e| e.to_string())?;

        let categories = stmt
            .query_map([], |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    icon: row.get(3)?,
                    sort_order: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(categories)
    }

    /// 根据ID获取分类
    pub async fn get_category_by_id(&self, id: &str) -> Result<Category, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        let category = db
            .query_row(
                "SELECT id, name, color, icon, sort_order, created_at FROM categories WHERE id = ?",
                [id],
                |row| {
                    Ok(Category {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        color: row.get(2)?,
                        icon: row.get(3)?,
                        sort_order: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                },
            )
            .map_err(|e| e.to_string())?;

        Ok(category)
    }

    /// 添加分类
    pub async fn add_category(&self, category: Category) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        db.execute(
            "INSERT INTO categories (id, name, color, icon, sort_order, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                category.id,
                category.name,
                category.color,
                category.icon,
                category.sort_order,
                category.created_at,
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 更新分类
    pub async fn update_category(&self, category: Category) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        db.execute(
            "UPDATE categories SET name = ?1, color = ?2, icon = ?3, sort_order = ?4 WHERE id = ?5",
            rusqlite::params![
                category.name,
                category.color,
                category.icon,
                category.sort_order,
                category.id,
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 删除分类
    pub async fn delete_category(&self, id: &str) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        db.execute("DELETE FROM categories WHERE id = ?", [id])
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    // ==================== 启动历史 ====================

    /// 获取启动历史
    pub async fn get_launch_history(&self, limit: usize) -> Result<Vec<LaunchHistory>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db
            .prepare("SELECT id, app_id, launched_at FROM launch_history ORDER BY launched_at DESC LIMIT ?")
            .map_err(|e| e.to_string())?;

        let history = stmt
            .query_map([limit], |row| {
                Ok(LaunchHistory {
                    id: row.get(0)?,
                    app_id: row.get(1)?,
                    launched_at: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(history)
    }

    /// 清空启动历史
    pub async fn clear_launch_history(&self) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        db.execute("DELETE FROM launch_history", [])
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    // ==================== 图标管理 ====================

    /// 刷新所有应用的图标（只处理没有图标的应用）
    pub async fn refresh_all_icons(&self) -> Result<u32, String> {
        use crate::services::AppScannerService;
        use std::path::Path;

        // 获取所有应用
        let apps = self.get_all_apps().await?;
        let mut updated_count = 0u32;

        for app in apps {
            // 跳过已有图标的应用（只要有图标就跳过，不管是提取的还是自定义的）
            if let Some(icon) = &app.icon {
                if !icon.is_empty() {
                    continue;
                }
            }

            // 只为没有图标的应用提取图标
            if let Some(new_icon) = AppScannerService::extract_icon_base64(Path::new(&app.path)) {
                // 更新数据库
                let db = self.db.lock().map_err(|e| e.to_string())?;
                db.execute(
                    "UPDATE apps SET icon = ?, updated_at = ? WHERE id = ?",
                    rusqlite::params![new_icon, chrono::Local::now().timestamp(), app.id],
                )
                .map_err(|e| e.to_string())?;

                updated_count += 1;
            }
        }

        Ok(updated_count)
    }

    /// 更新应用的自定义图标
    pub async fn update_app_icon(
        &self,
        app_id: &str,
        icon_data: Option<String>,
    ) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        db.execute(
            "UPDATE apps SET icon = ?, updated_at = ? WHERE id = ?",
            rusqlite::params![icon_data, chrono::Local::now().timestamp(), app_id],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    async fn setup_test_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[tokio::test]
    async fn test_add_and_get_app() {
        let db = setup_test_db().await;
        let service = AppLauncherService::new(db);

        let app = AppItem::new(
            "test_app".to_string(),
            "Test App".to_string(),
            "C:\\test.exe".to_string(),
        );

        // 添加应用
        service.add_app(app.clone()).await.unwrap();

        // 获取应用
        let retrieved = service.get_app_by_id("test_app").await.unwrap();
        assert_eq!(retrieved.name, "Test App");
    }

    #[tokio::test]
    async fn test_search_apps() {
        let db = setup_test_db().await;
        let service = AppLauncherService::new(db);

        let app1 = AppItem::new(
            "app1".to_string(),
            "VSCode".to_string(),
            "C:\\vscode.exe".to_string(),
        );
        let app2 = AppItem::new(
            "app2".to_string(),
            "Chrome".to_string(),
            "C:\\chrome.exe".to_string(),
        );

        service.add_app(app1).await.unwrap();
        service.add_app(app2).await.unwrap();

        let params = AppSearchParams {
            keyword: Some("Code".to_string()),
            ..Default::default()
        };

        let results = service.search_apps(params).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "VSCode");
    }

    #[tokio::test]
    async fn test_workflow_management() {
        let db = setup_test_db().await;
        let service = AppLauncherService::new(db);

        let workflow = Workflow::new(
            "wf1".to_string(),
            "Test Workflow".to_string(),
            vec!["app1".to_string(), "app2".to_string()],
        );

        // 添加工作流
        service.add_workflow(workflow.clone()).await.unwrap();

        // 获取工作流
        let retrieved = service.get_workflow_by_id("wf1").await.unwrap();
        assert_eq!(retrieved.name, "Test Workflow");
        assert_eq!(retrieved.app_ids.len(), 2);
    }

    #[tokio::test]
    async fn test_category_management() {
        let db = setup_test_db().await;
        let service = AppLauncherService::new(db);

        let category = Category::new("dev".to_string(), "开发工具".to_string());

        // 添加分类
        service.add_category(category).await.unwrap();

        // 获取所有分类
        let categories = service.get_all_categories().await.unwrap();
        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0].name, "开发工具");
    }

    #[tokio::test]
    async fn test_sync_scanned_apps_add_new() {
        let db = setup_test_db().await;
        let service = AppLauncherService::new(db);

        // 创建扫描到的应用
        let mut scanned_app = AppItem::new(
            "new_app".to_string(),
            "New App".to_string(),
            "C:\\new_app.exe".to_string(),
        );
        scanned_app.app_source = Some("registry".to_string());
        scanned_app.version = Some("1.0.0".to_string());

        // 同步
        let result = service
            .sync_scanned_apps(vec![scanned_app])
            .await
            .unwrap();

        // 验证结果
        assert_eq!(result.added, 1);
        assert_eq!(result.updated, 0);
        assert_eq!(result.removed, 0);
        assert_eq!(result.unchanged, 0);

        // 验证应用已添加
        let app = service.get_app_by_id("new_app").await.unwrap();
        assert_eq!(app.name, "New App");
        assert_eq!(app.version, Some("1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_sync_scanned_apps_update_existing() {
        let db = setup_test_db().await;
        let service = AppLauncherService::new(db);

        // 添加一个现有应用
        let mut existing_app = AppItem::new(
            "app1".to_string(),
            "Old Name".to_string(),
            "C:\\app.exe".to_string(),
        );
        existing_app.app_source = Some("registry".to_string());
        existing_app.category = Some("dev".to_string());
        existing_app.is_pinned = true;
        existing_app.launch_count = 10;
        service.add_app(existing_app).await.unwrap();

        // 创建更新的应用信息
        let mut updated_app = AppItem::new(
            "app1".to_string(),
            "New Name".to_string(),
            "C:\\app.exe".to_string(),
        );
        updated_app.app_source = Some("registry".to_string());
        updated_app.version = Some("2.0.0".to_string());
        updated_app.publisher = Some("New Publisher".to_string());

        // 同步
        let result = service
            .sync_scanned_apps(vec![updated_app])
            .await
            .unwrap();

        // 验证结果
        assert_eq!(result.added, 0);
        assert_eq!(result.updated, 1);
        assert_eq!(result.removed, 0);
        assert_eq!(result.unchanged, 0);

        // 验证应用已更新，但用户数据保留
        let app = service.get_app_by_id("app1").await.unwrap();
        assert_eq!(app.name, "New Name"); // 元数据已更新
        assert_eq!(app.version, Some("2.0.0".to_string())); // 元数据已更新
        assert_eq!(app.publisher, Some("New Publisher".to_string())); // 元数据已更新
        assert_eq!(app.category, Some("dev".to_string())); // 用户数据保留
        assert_eq!(app.is_pinned, true); // 用户数据保留
        assert_eq!(app.launch_count, 10); // 用户数据保留
    }

    #[tokio::test]
    async fn test_sync_scanned_apps_hide_deleted() {
        let db = setup_test_db().await;
        let service = AppLauncherService::new(db);

        // 添加两个应用：一个注册表应用，一个手动添加的应用
        let mut registry_app = AppItem::new(
            "reg_app".to_string(),
            "Registry App".to_string(),
            "C:\\reg_app.exe".to_string(),
        );
        registry_app.app_source = Some("registry".to_string());
        service.add_app(registry_app).await.unwrap();

        let mut manual_app = AppItem::new(
            "manual_app".to_string(),
            "Manual App".to_string(),
            "C:\\manual_app.exe".to_string(),
        );
        manual_app.app_source = Some("manual".to_string());
        service.add_app(manual_app).await.unwrap();

        // 同步时不包含这两个应用（模拟应用被删除）
        let result = service.sync_scanned_apps(vec![]).await.unwrap();

        // 验证结果：注册表应用被标记为隐藏，手动应用保留
        assert_eq!(result.added, 0);
        assert_eq!(result.updated, 0);
        assert_eq!(result.removed, 1); // 注册表应用被隐藏
        assert_eq!(result.unchanged, 1); // 手动应用保留

        // 验证注册表应用被隐藏
        let reg_app = service.get_app_by_id("reg_app").await.unwrap();
        assert_eq!(reg_app.is_hidden, true);

        // 验证手动应用未被隐藏
        let man_app = service.get_app_by_id("manual_app").await.unwrap();
        assert_eq!(man_app.is_hidden, false);
    }

    #[tokio::test]
    async fn test_sync_scanned_apps_no_changes() {
        let db = setup_test_db().await;
        let service = AppLauncherService::new(db);

        // 添加一个应用
        let mut app = AppItem::new(
            "app1".to_string(),
            "Test App".to_string(),
            "C:\\test.exe".to_string(),
        );
        app.app_source = Some("registry".to_string());
        app.version = Some("1.0.0".to_string());
        service.add_app(app.clone()).await.unwrap();

        // 使用相同的应用信息同步
        let result = service.sync_scanned_apps(vec![app]).await.unwrap();

        // 验证结果：没有变化
        assert_eq!(result.added, 0);
        assert_eq!(result.updated, 0);
        assert_eq!(result.removed, 0);
        assert_eq!(result.unchanged, 1);
    }
}
