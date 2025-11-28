// Services module
// 业务逻辑层

pub mod ai_service;
pub mod app_launcher_service;
pub mod app_scanner_service;
pub mod clipboard_service;
pub mod settings_service;
pub mod sql_ai_service;
pub mod sql_service;
pub mod tag_service;
pub mod task_service;
pub mod weekly_plan_service;
pub mod work_log_service;

// 以下模块暂时注释，因为依赖其他额外的crate
// pub mod git_service;

// 重新导出常用类型
pub use ai_service::{AiConfig, AiService, ChatMessage};
pub use app_launcher_service::AppLauncherService;
pub use app_scanner_service::AppScannerService;
pub use clipboard_service::ClipboardService;
pub use settings_service::{AppSettings, SettingsService};
pub use sql_ai_service::{AiProvider, SqlAiConfig, SqlAiService, SqlClassifyResult};
pub use sql_service::{SqlCategory, SqlRecord, SqlService};
pub use tag_service::TagService;
pub use task_service::TaskService;
pub use weekly_plan_service::WeeklyPlanService;
pub use work_log_service::WorkLogService;
