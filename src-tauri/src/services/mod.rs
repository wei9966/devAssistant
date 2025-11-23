// Services module
// 业务逻辑层

pub mod task_service;
pub mod sql_service;
pub mod clipboard_service;
pub mod work_log_service;

// 以下模块暂时注释，因为依赖其他额外的crate
// pub mod git_service;
// pub mod ai_service;

// 重新导出常用类型
pub use task_service::TaskService;
pub use sql_service::{SqlService, SqlRecord};
pub use clipboard_service::ClipboardService;
pub use work_log_service::WorkLogService;
