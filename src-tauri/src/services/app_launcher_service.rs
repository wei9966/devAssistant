use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use crate::models::{
    AppItem, Category, Workflow, LaunchHistory,
    AppSearchParams, LaunchResult, WorkflowLaunchResult
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
            .prepare("SELECT id, name, path, icon, category, tags, launch_count, last_launched_at, is_pinned, is_hidden, launch_args, created_at, updated_at FROM apps ORDER BY name")
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
            .prepare("SELECT id, name, path, icon, category, tags, launch_count, last_launched_at, is_pinned, is_hidden, launch_args, created_at, updated_at FROM apps WHERE id = ?")
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
                })
            })
            .map_err(|e| format!("App not found: {}", e))?;

        Ok(app)
    }

    /// 搜索应用
    pub async fn search_apps(&self, params: AppSearchParams) -> Result<Vec<AppItem>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        let mut sql = String::from(
            "SELECT id, name, path, icon, category, tags, launch_count, last_launched_at, is_pinned, is_hidden, launch_args, created_at, updated_at FROM apps WHERE 1=1"
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

        let params_slice: Vec<&dyn rusqlite::ToSql> = conditions.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

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
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(apps)
    }

    /// 添加应用
    pub async fn add_app(&self, app: AppItem) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        db.execute(
            "INSERT INTO apps (id, name, path, icon, category, tags, launch_count, last_launched_at, is_pinned, is_hidden, launch_args, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                app.id,
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
                app.created_at,
                app.updated_at,
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 更新应用
    pub async fn update_app(&self, app: AppItem) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;

        let now = chrono::Utc::now().timestamp();

        db.execute(
            "UPDATE apps SET name = ?1, path = ?2, icon = ?3, category = ?4, tags = ?5,
             launch_count = ?6, last_launched_at = ?7, is_pinned = ?8, is_hidden = ?9,
             launch_args = ?10, updated_at = ?11 WHERE id = ?12",
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
        use std::process::Command;

        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("cmd");
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            cmd.arg("/C");
            cmd.arg("start");
            cmd.arg("");
            cmd.arg(&app.path);

            // 添加启动参数
            if let Some(args) = &app.launch_args {
                cmd.arg(args);
            }

            match cmd.spawn() {
                Ok(_) => LaunchResult::success(app.id.clone(), app.name.clone()),
                Err(e) => LaunchResult::failure(app.id.clone(), app.name.clone(), e.to_string()),
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
        let now = chrono::Utc::now().timestamp();

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
        let now = chrono::Utc::now().timestamp();

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
        let now = chrono::Utc::now().timestamp();

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
            let launch_result = self.launch_app(app_id).await.unwrap_or_else(|e| {
                LaunchResult::failure(app_id.clone(), app_id.clone(), e)
            });

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

        let app1 = AppItem::new("app1".to_string(), "VSCode".to_string(), "C:\\vscode.exe".to_string());
        let app2 = AppItem::new("app2".to_string(), "Chrome".to_string(), "C:\\chrome.exe".to_string());

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
}
