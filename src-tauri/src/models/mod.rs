pub mod app_launcher;
pub mod app_launcher_settings;
pub mod clipboard;
pub mod file_index;
pub mod pomodoro;
pub mod screen_context;
pub mod sql_record;
pub mod task;
pub mod task_category;
pub mod tool;
pub mod weekly_plan;
pub mod work_context;
pub mod work_log;

// Re-export commonly used types
pub use app_launcher::{
    AppItem, AppSearchParams, Category, ItemType, LaunchHistory, LaunchResult, SyncResult, Workflow,
    WorkflowLaunchResult,
};
pub use app_launcher_settings::AppLauncherSettings;
pub use clipboard::{ClipboardConfig, ClipboardContentType, ClipboardHistory, ClipboardQueryParams, CreateClipboardRecord};
pub use pomodoro::{
    AiFocusAnalysis, AiFocusSuggestion, CompletePomodoroRequest, CreatePomodoroRequest, FocusApp,
    PomodoroDailyStats, PomodoroPhase, PomodoroSession, PomodoroStatus,
};
pub use screen_context::{
    ActiveTimeRange, ActivityDistribution, AppUsage, DailyActiveHours, DailySummary, DayContextSummary,
    DayStats, KeyActivity, ScreenContext, WeekContextSummary, WeeklyActivitySummary, WeeklyAppRanking,
    WorkPatterns,
};
pub use sql_record::SqlRecord;
pub use task::{
    FileContext, Tag, Task, TaskCategory, TaskMilestone, TaskPriority, TaskQuadrant, TaskStatus, WorkContext,
};
pub use task_category::TaskCategoryDefinition;
pub use tool::{PinnedTool, ToolCategory, ToolItem, ToolUsageRecord};
pub use weekly_plan::{WeeklyPlan, WeeklyPlanStatus};
pub use work_log::WorkLog;
pub use file_index::{
    FileIndex, FileIndexQueryParams, IndexStats, DriveStats, IndexTaskStatus, IndexProgress,
    CREATE_FILE_INDEX_TABLE_SQL, CREATE_FILE_INDEX_INDEXES_SQL, CREATE_INDEX_METADATA_TABLE_SQL,
};
