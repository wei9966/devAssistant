use serde::{Deserialize, Serialize};

/// 应用项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub launch_count: i32,
    pub last_launched_at: Option<i64>,
    pub is_pinned: bool,
    pub is_hidden: bool,
    pub launch_args: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl AppItem {
    /// 创建新的应用项
    pub fn new(id: String, name: String, path: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            name,
            path,
            icon: None,
            category: None,
            tags: None,
            launch_count: 0,
            last_launched_at: None,
            is_pinned: false,
            is_hidden: false,
            launch_args: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 转换tags为JSON字符串用于存储
    pub fn tags_to_json(&self) -> Option<String> {
        self.tags.as_ref().and_then(|tags| {
            serde_json::to_string(tags).ok()
        })
    }

    /// 从JSON字符串解析tags
    pub fn tags_from_json(json: Option<&str>) -> Option<Vec<String>> {
        json.and_then(|s| serde_json::from_str(s).ok())
    }
}

/// 应用分类
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub created_at: i64,
}

impl Category {
    /// 创建新的分类
    pub fn new(id: String, name: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            name,
            color: None,
            icon: None,
            sort_order: 0,
            created_at: now,
        }
    }

    /// 创建带颜色的分类
    pub fn with_color(id: String, name: String, color: String) -> Self {
        let mut category = Self::new(id, name);
        category.color = Some(color);
        category
    }
}

/// 工作流
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub app_ids: Vec<String>,
    pub launch_delay: Option<i32>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Workflow {
    /// 创建新的工作流
    pub fn new(id: String, name: String, app_ids: Vec<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            name,
            app_ids,
            launch_delay: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 转换app_ids为JSON字符串用于存储
    pub fn app_ids_to_json(&self) -> String {
        serde_json::to_string(&self.app_ids).unwrap_or_else(|_| "[]".to_string())
    }

    /// 从JSON字符串解析app_ids
    pub fn app_ids_from_json(json: &str) -> Vec<String> {
        serde_json::from_str(json).unwrap_or_default()
    }

    /// 设置启动延迟（毫秒）
    pub fn with_delay(mut self, delay_ms: i32) -> Self {
        self.launch_delay = Some(delay_ms);
        self
    }
}

/// 启动历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchHistory {
    pub id: Option<i64>,
    pub app_id: String,
    pub launched_at: i64,
}

impl LaunchHistory {
    /// 创建新的启动历史记录
    pub fn new(app_id: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: None,
            app_id,
            launched_at: now,
        }
    }

    /// 创建指定时间的启动历史记录
    pub fn with_timestamp(app_id: String, timestamp: i64) -> Self {
        Self {
            id: None,
            app_id,
            launched_at: timestamp,
        }
    }
}

/// 应用搜索参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSearchParams {
    pub keyword: Option<String>,
    pub category: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_hidden: Option<bool>,
    pub limit: Option<usize>,
}

impl Default for AppSearchParams {
    fn default() -> Self {
        Self {
            keyword: None,
            category: None,
            is_pinned: None,
            is_hidden: Some(false), // 默认不显示隐藏的应用
            limit: None,
        }
    }
}

/// 应用启动结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub success: bool,
    pub app_id: String,
    pub app_name: String,
    pub error: Option<String>,
}

impl LaunchResult {
    pub fn success(app_id: String, app_name: String) -> Self {
        Self {
            success: true,
            app_id,
            app_name,
            error: None,
        }
    }

    pub fn failure(app_id: String, app_name: String, error: String) -> Self {
        Self {
            success: false,
            app_id,
            app_name,
            error: Some(error),
        }
    }
}

/// 工作流启动结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowLaunchResult {
    pub workflow_id: String,
    pub workflow_name: String,
    pub total_apps: usize,
    pub successful_launches: usize,
    pub failed_launches: usize,
    pub results: Vec<LaunchResult>,
}

impl WorkflowLaunchResult {
    pub fn new(workflow_id: String, workflow_name: String) -> Self {
        Self {
            workflow_id,
            workflow_name,
            total_apps: 0,
            successful_launches: 0,
            failed_launches: 0,
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: LaunchResult) {
        self.total_apps += 1;
        if result.success {
            self.successful_launches += 1;
        } else {
            self.failed_launches += 1;
        }
        self.results.push(result);
    }

    pub fn is_all_successful(&self) -> bool {
        self.failed_launches == 0 && self.total_apps > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_item_creation() {
        let app = AppItem::new(
            "vscode".to_string(),
            "Visual Studio Code".to_string(),
            "C:\\Program Files\\VSCode\\Code.exe".to_string(),
        );

        assert_eq!(app.id, "vscode");
        assert_eq!(app.name, "Visual Studio Code");
        assert_eq!(app.launch_count, 0);
        assert!(!app.is_pinned);
        assert!(!app.is_hidden);
    }

    #[test]
    fn test_app_item_tags_serialization() {
        let mut app = AppItem::new("test".to_string(), "Test".to_string(), "test.exe".to_string());
        app.tags = Some(vec!["dev".to_string(), "editor".to_string()]);

        let json = app.tags_to_json();
        assert!(json.is_some());

        let tags = AppItem::tags_from_json(json.as_deref());
        assert_eq!(tags, app.tags);
    }

    #[test]
    fn test_category_creation() {
        let category = Category::new("dev".to_string(), "开发工具".to_string());
        assert_eq!(category.id, "dev");
        assert_eq!(category.name, "开发工具");
        assert_eq!(category.sort_order, 0);

        let category_with_color = Category::with_color(
            "design".to_string(),
            "设计工具".to_string(),
            "#f59e0b".to_string(),
        );
        assert_eq!(category_with_color.color, Some("#f59e0b".to_string()));
    }

    #[test]
    fn test_workflow_creation() {
        let app_ids = vec!["vscode".to_string(), "chrome".to_string()];
        let workflow = Workflow::new(
            "frontend-dev".to_string(),
            "前端开发".to_string(),
            app_ids.clone(),
        );

        assert_eq!(workflow.id, "frontend-dev");
        assert_eq!(workflow.app_ids, app_ids);
        assert!(workflow.launch_delay.is_none());

        let workflow_with_delay = workflow.with_delay(500);
        assert_eq!(workflow_with_delay.launch_delay, Some(500));
    }

    #[test]
    fn test_workflow_app_ids_serialization() {
        let app_ids = vec!["app1".to_string(), "app2".to_string(), "app3".to_string()];
        let workflow = Workflow::new("test".to_string(), "Test".to_string(), app_ids.clone());

        let json = workflow.app_ids_to_json();
        let parsed_ids = Workflow::app_ids_from_json(&json);

        assert_eq!(parsed_ids, app_ids);
    }

    #[test]
    fn test_launch_history_creation() {
        let history = LaunchHistory::new("vscode".to_string());
        assert_eq!(history.app_id, "vscode");
        assert!(history.id.is_none());

        let timestamp = 1732435200i64;
        let history_with_time = LaunchHistory::with_timestamp("chrome".to_string(), timestamp);
        assert_eq!(history_with_time.launched_at, timestamp);
    }

    #[test]
    fn test_app_search_params_default() {
        let params = AppSearchParams::default();
        assert!(params.keyword.is_none());
        assert!(params.category.is_none());
        assert_eq!(params.is_hidden, Some(false));
    }

    #[test]
    fn test_launch_result() {
        let success = LaunchResult::success("vscode".to_string(), "VSCode".to_string());
        assert!(success.success);
        assert!(success.error.is_none());

        let failure = LaunchResult::failure(
            "app1".to_string(),
            "App1".to_string(),
            "File not found".to_string(),
        );
        assert!(!failure.success);
        assert_eq!(failure.error, Some("File not found".to_string()));
    }

    #[test]
    fn test_workflow_launch_result() {
        let mut result = WorkflowLaunchResult::new(
            "wf1".to_string(),
            "Test Workflow".to_string(),
        );

        result.add_result(LaunchResult::success("app1".to_string(), "App1".to_string()));
        result.add_result(LaunchResult::success("app2".to_string(), "App2".to_string()));
        result.add_result(LaunchResult::failure(
            "app3".to_string(),
            "App3".to_string(),
            "Error".to_string(),
        ));

        assert_eq!(result.total_apps, 3);
        assert_eq!(result.successful_launches, 2);
        assert_eq!(result.failed_launches, 1);
        assert!(!result.is_all_successful());
    }
}
