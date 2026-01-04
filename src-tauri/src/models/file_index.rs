use serde::{Deserialize, Serialize};

/// 文件索引记录
/// 用于存储文件系统索引信息，支持快速文件搜索
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileIndex {
    /// 主键ID
    pub id: Option<i64>,
    /// 文件名（不含路径）
    pub file_name: String,
    /// 完整文件路径
    pub file_path: String,
    /// 父目录路径
    pub parent_path: String,
    /// 文件大小（字节）
    pub size: i64,
    /// 修改时间（ISO 8601 格式）
    pub modified_time: String,
    /// 是否为目录
    pub is_dir: bool,
    /// 文件类型/扩展名（小写，不含点号）
    pub file_type: Option<String>,
    /// 驱动器标识（如 C:, D:）
    pub drive: String,
    /// NTFS 文件引用号（用于增量更新检测）
    pub file_ref_number: Option<i64>,
}

impl FileIndex {
    /// 创建新的文件索引记录
    pub fn new(
        file_name: String,
        file_path: String,
        parent_path: String,
        size: i64,
        modified_time: String,
        is_dir: bool,
        file_type: Option<String>,
        drive: String,
        file_ref_number: Option<i64>,
    ) -> Self {
        Self {
            id: None,
            file_name,
            file_path,
            parent_path,
            size,
            modified_time,
            is_dir,
            file_type,
            drive,
            file_ref_number,
        }
    }

    /// 从文件路径提取驱动器标识
    pub fn extract_drive(path: &str) -> String {
        if path.len() >= 2 && path.chars().nth(1) == Some(':') {
            path[..2].to_uppercase()
        } else {
            String::new()
        }
    }

    /// 从文件名提取扩展名
    pub fn extract_extension(file_name: &str) -> Option<String> {
        std::path::Path::new(file_name)
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase())
    }
}

/// 文件索引查询参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FileIndexQueryParams {
    /// 搜索关键词（文件名模糊匹配）
    pub keyword: Option<String>,
    /// 按驱动器过滤
    pub drive: Option<String>,
    /// 按文件类型过滤
    pub file_type: Option<String>,
    /// 仅显示目录
    pub dirs_only: Option<bool>,
    /// 仅显示文件
    pub files_only: Option<bool>,
    /// 父目录路径（用于浏览）
    pub parent_path: Option<String>,
    /// 最小文件大小（字节）
    pub min_size: Option<i64>,
    /// 最大文件大小（字节）
    pub max_size: Option<i64>,
    /// 限制返回数量
    pub limit: Option<i32>,
    /// 偏移量（分页）
    pub offset: Option<i32>,
}

/// 索引统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStats {
    /// 总文件数
    pub total_files: i64,
    /// 总目录数
    pub total_dirs: i64,
    /// 总大小（字节）
    pub total_size: i64,
    /// 各驱动器文件数
    pub files_by_drive: Vec<DriveStats>,
    /// 最后更新时间
    pub last_updated: Option<String>,
}

/// 驱动器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveStats {
    /// 驱动器标识
    pub drive: String,
    /// 文件数量
    pub file_count: i64,
    /// 目录数量
    pub dir_count: i64,
    /// 总大小（字节）
    pub total_size: i64,
}

/// 索引任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IndexTaskStatus {
    /// 空闲
    Idle,
    /// 正在索引
    Indexing,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 发生错误
    Error,
}

impl ToString for IndexTaskStatus {
    fn to_string(&self) -> String {
        match self {
            IndexTaskStatus::Idle => "idle".to_string(),
            IndexTaskStatus::Indexing => "indexing".to_string(),
            IndexTaskStatus::Paused => "paused".to_string(),
            IndexTaskStatus::Completed => "completed".to_string(),
            IndexTaskStatus::Error => "error".to_string(),
        }
    }
}

/// 索引任务进度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexProgress {
    /// 当前状态
    pub status: IndexTaskStatus,
    /// 当前正在处理的驱动器
    pub current_drive: Option<String>,
    /// 已处理文件数
    pub processed_count: i64,
    /// 总文件数（预估）
    pub total_count: Option<i64>,
    /// 进度百分比（0-100）
    pub progress_percent: Option<f32>,
    /// 当前处理的路径
    pub current_path: Option<String>,
    /// 错误信息
    pub error_message: Option<String>,
}

impl Default for IndexProgress {
    fn default() -> Self {
        Self {
            status: IndexTaskStatus::Idle,
            current_drive: None,
            processed_count: 0,
            total_count: None,
            progress_percent: None,
            current_path: None,
            error_message: None,
        }
    }
}

/// SQLite 建表 SQL 语句
pub const CREATE_FILE_INDEX_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS file_index (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL UNIQUE,
    parent_path TEXT NOT NULL,
    size INTEGER NOT NULL DEFAULT 0,
    modified_time TEXT NOT NULL,
    is_dir INTEGER NOT NULL DEFAULT 0,
    file_type TEXT,
    drive TEXT NOT NULL,
    file_ref_number INTEGER,
    created_at TEXT DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT DEFAULT (datetime('now', 'localtime'))
);
"#;

/// 创建索引的 SQL 语句
pub const CREATE_FILE_INDEX_INDEXES_SQL: &str = r#"
-- 文件名索引（用于搜索）
CREATE INDEX IF NOT EXISTS idx_file_index_file_name ON file_index(file_name);

-- 驱动器索引（用于按驱动器过滤）
CREATE INDEX IF NOT EXISTS idx_file_index_drive ON file_index(drive);

-- 目录标识索引（用于过滤文件/目录）
CREATE INDEX IF NOT EXISTS idx_file_index_is_dir ON file_index(is_dir);

-- 文件类型索引（用于按扩展名过滤）
CREATE INDEX IF NOT EXISTS idx_file_index_file_type ON file_index(file_type);

-- 父目录索引（用于目录浏览）
CREATE INDEX IF NOT EXISTS idx_file_index_parent_path ON file_index(parent_path);

-- 文件引用号索引（用于增量更新检测）
CREATE INDEX IF NOT EXISTS idx_file_index_file_ref_number ON file_index(file_ref_number);

-- 复合索引：驱动器 + 文件名（常用搜索场景）
CREATE INDEX IF NOT EXISTS idx_file_index_drive_name ON file_index(drive, file_name);
"#;

/// 索引元数据表 SQL（用于存储索引状态）
pub const CREATE_INDEX_METADATA_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS index_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    drive TEXT NOT NULL UNIQUE,
    last_index_time TEXT,
    file_count INTEGER DEFAULT 0,
    dir_count INTEGER DEFAULT 0,
    total_size INTEGER DEFAULT 0,
    status TEXT DEFAULT 'idle',
    error_message TEXT,
    created_at TEXT DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT DEFAULT (datetime('now', 'localtime'))
);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_drive() {
        assert_eq!(FileIndex::extract_drive("C:\\Users\\test"), "C:");
        assert_eq!(FileIndex::extract_drive("D:\\Documents"), "D:");
        assert_eq!(FileIndex::extract_drive("/home/user"), "");
    }

    #[test]
    fn test_extract_extension() {
        assert_eq!(FileIndex::extract_extension("test.txt"), Some("txt".to_string()));
        assert_eq!(FileIndex::extract_extension("document.PDF"), Some("pdf".to_string()));
        assert_eq!(FileIndex::extract_extension("archive.tar.gz"), Some("gz".to_string()));
        assert_eq!(FileIndex::extract_extension("noextension"), None);
    }

    #[test]
    fn test_new_file_index() {
        let index = FileIndex::new(
            "test.txt".to_string(),
            "C:\\Users\\test.txt".to_string(),
            "C:\\Users".to_string(),
            1024,
            "2024-01-01T00:00:00".to_string(),
            false,
            Some("txt".to_string()),
            "C:".to_string(),
            Some(12345),
        );

        assert!(index.id.is_none());
        assert_eq!(index.file_name, "test.txt");
        assert_eq!(index.size, 1024);
        assert!(!index.is_dir);
    }
}
