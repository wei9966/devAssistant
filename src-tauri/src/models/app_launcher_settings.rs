use serde::{Deserialize, Serialize};

/// 应用启动器设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLauncherSettings {
    /// 允许扫描的文件后缀列表
    pub allowed_extensions: Vec<String>,
}

impl Default for AppLauncherSettings {
    fn default() -> Self {
        Self {
            // 默认只允许 .exe 和 .lnk
            allowed_extensions: vec!["exe".to_string(), "lnk".to_string()],
        }
    }
}

impl AppLauncherSettings {
    /// 创建新的设置
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加允许的文件后缀
    pub fn add_extension(&mut self, extension: String) {
        let ext = extension.trim_start_matches('.').to_lowercase();
        if !self.allowed_extensions.contains(&ext) {
            self.allowed_extensions.push(ext);
        }
    }

    /// 移除允许的文件后缀
    pub fn remove_extension(&mut self, extension: &str) {
        let ext = extension.trim_start_matches('.').to_lowercase();
        self.allowed_extensions.retain(|e| e != &ext);
    }

    /// 检查文件后缀是否被允许
    pub fn is_extension_allowed(&self, extension: &str) -> bool {
        let ext = extension.trim_start_matches('.').to_lowercase();
        self.allowed_extensions.contains(&ext)
    }

    /// 重置为默认设置
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// 转换为JSON字符串
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    /// 从JSON字符串解析
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppLauncherSettings::default();
        assert_eq!(settings.allowed_extensions.len(), 2);
        assert!(settings.allowed_extensions.contains(&"exe".to_string()));
        assert!(settings.allowed_extensions.contains(&"lnk".to_string()));
    }

    #[test]
    fn test_add_extension() {
        let mut settings = AppLauncherSettings::new();
        settings.add_extension("rdp".to_string());
        assert!(settings.allowed_extensions.contains(&"rdp".to_string()));

        // 测试去除点前缀
        settings.add_extension(".url".to_string());
        assert!(settings.allowed_extensions.contains(&"url".to_string()));

        // 测试大小写不敏感
        settings.add_extension("PDF".to_string());
        assert!(settings.allowed_extensions.contains(&"pdf".to_string()));
    }

    #[test]
    fn test_remove_extension() {
        let mut settings = AppLauncherSettings::new();
        settings.add_extension("rdp".to_string());
        assert!(settings.is_extension_allowed("rdp"));

        settings.remove_extension("rdp");
        assert!(!settings.is_extension_allowed("rdp"));
    }

    #[test]
    fn test_is_extension_allowed() {
        let settings = AppLauncherSettings::default();
        assert!(settings.is_extension_allowed("exe"));
        assert!(settings.is_extension_allowed(".exe"));
        assert!(settings.is_extension_allowed("EXE"));
        assert!(!settings.is_extension_allowed("pdf"));
    }

    #[test]
    fn test_serialization() {
        let settings = AppLauncherSettings::default();
        let json = settings.to_json().unwrap();
        let parsed = AppLauncherSettings::from_json(&json).unwrap();
        assert_eq!(settings.allowed_extensions, parsed.allowed_extensions);
    }

    #[test]
    fn test_reset() {
        let mut settings = AppLauncherSettings::new();
        settings.add_extension("rdp".to_string());
        settings.add_extension("url".to_string());
        assert_eq!(settings.allowed_extensions.len(), 4);

        settings.reset();
        assert_eq!(settings.allowed_extensions.len(), 2);
        assert!(settings.is_extension_allowed("exe"));
        assert!(settings.is_extension_allowed("lnk"));
    }
}
