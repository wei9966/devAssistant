// Services module
// 业务逻辑层

pub mod activity_summary_service;
pub mod ai_service;
pub mod ai_chat_service;
pub mod ai_functions_service;
pub mod app_launcher_service;
pub mod app_monitor_service;
pub mod app_scanner_service;
pub mod clipboard_history_service;
pub mod context_manager_service;
pub mod context_store_service;
pub mod milestone_service;
pub mod notification_service;
pub mod pomodoro_service;
pub mod prompt_service;
pub mod prompt_manager_service;
pub mod prompt_db_service;
pub mod report_service;
pub mod scheduler_service;
pub mod screen_capture_service;
pub mod screenshot_batch_processor_service;
pub mod settings_service;
pub mod sql_ai_service;
pub mod sql_service;
pub mod sql_template_service;
pub mod tag_service;
pub mod task_service;
pub mod tips_service;
pub mod todo_prediction_service;
pub mod user_activity_service;
pub mod vlm_service;
pub mod weekly_plan_service;
pub mod work_log_service;
pub mod tool_service;

// 以下模块暂时注释，因为依赖其他额外的crate或有编码问题
// pub mod git_service;

// 重新导出常用类型
pub use activity_summary_service::{ActivitySummary, ActivitySummaryService, ActivityType};
pub use ai_service::{AiConfig, AiService, ChatMessage};
pub use ai_chat_service::{AiChatService, ChatResponse, DashboardStats, FunctionCall};
pub use ai_functions_service::AiFunctionsService;
pub use app_launcher_service::AppLauncherService;
pub use app_monitor_service::{AppMonitorService, NewAppsDetectedEvent};
pub use app_scanner_service::AppScannerService;
pub use clipboard_history_service::ClipboardHistoryService;
pub use context_store_service::ContextStoreService;
pub use milestone_service::MilestoneService;
pub use notification_service::{Notification, NotificationService, NotificationSettings, NotificationType};
pub use pomodoro_service::{PomodoroService, PomodoroAiService};
pub use prompt_service::{PromptConfig, PromptService, PromptSettings};
pub use prompt_manager_service::{
    PromptManager, PromptsConfig, ProcessingPrompts, GenerationPrompts,
    MergingPrompts, EntityPrompts, ExtractionPrompts
};
pub use prompt_db_service::{AiPrompt, PromptDbService, PromptUpdate, RenderedPrompt};
pub use report_service::{DailyReport, ReportInputData, ReportService};
pub use scheduler_service::{SchedulerConfig, SchedulerService};
pub use screenshot_batch_processor_service::{
    BatchAnalysisResult, BatchItem, ProcessorState, ScreenshotBatchProcessor,
};
pub use settings_service::{AppSettings, SettingsService};
pub use sql_ai_service::{AiProvider, SqlAiConfig, SqlAiService, SqlClassifyResult};
pub use sql_service::{SqlCategory, SqlRecord, SqlService};
pub use sql_template_service::{ConsolidateResult, SqlTemplate, SqlTemplateService};
pub use tag_service::TagService;
pub use task_service::TaskService;
pub use tips_service::{ActivityPattern, Tip, TipCategory, TipPriority, TipsService};
pub use todo_prediction_service::{PredictedTask, PredictionResult, TodoPredictionService};
pub use user_activity_service::UserActivityService;
pub use vlm_service::{ScreenshotAnalysisResponse, VlmConfig, VlmConfigResponse, VlmService};
pub use weekly_plan_service::WeeklyPlanService;
pub use work_log_service::WorkLogService;
