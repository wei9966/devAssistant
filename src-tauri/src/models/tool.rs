use serde::{Deserialize, Serialize};

/// 工具项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolItem {
    /// 工具唯一标识
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 工具图标（SVG 或 emoji）
    pub icon: String,
    /// 工具描述
    pub description: String,
    /// 工具组件标识（前端用于动态加载）
    pub component: String,
    /// 是否可固定到启动器
    pub is_pinnable: bool,
    /// 是否支持多实例
    pub multi_instance: bool,
    /// 工具分类
    pub category: ToolCategory,
    /// 快捷键（可选）
    pub shortcut: Option<String>,
}

/// 工具分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    /// 系统工具
    System,
    /// 开发工具
    Development,
    /// 网络工具
    Network,
    /// 文件工具
    File,
    /// 实用工具
    Utility,
    /// 其他
    Other,
}

/// 固定的工具记录（存储用户固定的工具）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinnedTool {
    /// 工具ID
    pub tool_id: String,
    /// 固定时间
    pub pinned_at: i64,
    /// 排序顺序
    pub sort_order: i32,
}

/// 工具使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolUsageRecord {
    /// 工具ID
    pub tool_id: String,
    /// 使用次数
    pub use_count: i32,
    /// 最后使用时间
    pub last_used_at: i64,
}
