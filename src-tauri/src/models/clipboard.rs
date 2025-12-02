use serde::{Deserialize, Serialize};

/// 剪切板内容类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardContentType {
    Text,
    Image,
    File,
}

impl ToString for ClipboardContentType {
    fn to_string(&self) -> String {
        match self {
            ClipboardContentType::Text => "text".to_string(),
            ClipboardContentType::Image => "image".to_string(),
            ClipboardContentType::File => "file".to_string(),
        }
    }
}

impl From<&str> for ClipboardContentType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "image" => ClipboardContentType::Image,
            "file" => ClipboardContentType::File,
            _ => ClipboardContentType::Text,
        }
    }
}

/// 剪切板历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardHistory {
    pub id: i64,
    pub content_type: String,
    pub content: String,
    pub preview: Option<String>,
    pub image_path: Option<String>,
    pub source_app: Option<String>,
    pub is_pinned: bool,
    pub created_at: String,
}

/// 创建剪切板记录请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClipboardRecord {
    pub content_type: String,
    pub content: String,
    pub preview: Option<String>,
    pub image_path: Option<String>,
    pub source_app: Option<String>,
}

/// 剪切板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipboardConfig {
    /// 图片保存目录（使用中的图片）
    pub image_save_dir: String,
    /// 图片归档目录
    pub image_archive_dir: String,
    /// 最大历史记录数
    pub max_history: i32,
    /// 是否自动将图片路径复制到剪切板
    pub auto_copy_image_path: bool,
    /// 是否启用剪切板监控
    pub enabled: bool,
    /// 剪切板历史快捷键
    pub shortcut: String,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        // 默认保存到用户图片目录下的 ClipboardImages 文件夹
        let base_dir = dirs::picture_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default());

        let default_dir = base_dir.join("ClipboardImages").to_string_lossy().to_string();
        let archive_dir = base_dir.join("ClipboardArchive").to_string_lossy().to_string();

        Self {
            image_save_dir: default_dir,
            image_archive_dir: archive_dir,
            max_history: 100,
            auto_copy_image_path: true,
            enabled: true,
            shortcut: "Ctrl+Shift+C".to_string(),
        }
    }
}

/// 剪切板历史查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardQueryParams {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub content_type: Option<String>,
    pub keyword: Option<String>,
    pub pinned_only: Option<bool>,
}

impl Default for ClipboardQueryParams {
    fn default() -> Self {
        Self {
            limit: Some(50),
            offset: Some(0),
            content_type: None,
            keyword: None,
            pinned_only: None,
        }
    }
}
