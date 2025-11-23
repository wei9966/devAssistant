use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

/// AI服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: Some("https://api.anthropic.com".to_string()),
            model: "claude-3-5-sonnet-20241022".to_string(),
        }
    }
}

/// 工作日志数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLogData {
    pub date: String,
    pub git_commits: Vec<String>,
    pub sql_queries: Vec<String>,
    pub tasks_completed: Vec<String>,
    pub tasks_active: Vec<String>,
}

/// AI服务
pub struct AiService {
    config: AiConfig,
}

impl AiService {
    /// 创建新的AI服务实例
    pub fn new(config: AiConfig) -> Self {
        Self { config }
    }

    /// 生成工作日志（调用Claude API）
    ///
    /// 此方法为预留接口，实际实现需要：
    /// 1. 集成 reqwest 或其他 HTTP 客户端
    /// 2. 构建 Claude API 请求
    /// 3. 解析响应并返回生成的日志
    pub async fn generate_work_log(&self, data: WorkLogData) -> Result<String> {
        // TODO: 实现实际的API调用
        // 目前返回一个模拟的日志
        let log = self.format_work_log(&data);
        Ok(log)
    }

    /// 格式化工作日志（简单版本，不调用AI）
    fn format_work_log(&self, data: &WorkLogData) -> String {
        let mut log = format!("# 工作日志 - {}\n\n", data.date);

        if !data.git_commits.is_empty() {
            log.push_str("## Git 提交记录\n");
            for commit in &data.git_commits {
                log.push_str(&format!("- {}\n", commit));
            }
            log.push('\n');
        }

        if !data.tasks_completed.is_empty() {
            log.push_str("## 已完成任务\n");
            for task in &data.tasks_completed {
                log.push_str(&format!("- [x] {}\n", task));
            }
            log.push('\n');
        }

        if !data.tasks_active.is_empty() {
            log.push_str("## 进行中任务\n");
            for task in &data.tasks_active {
                log.push_str(&format!("- [ ] {}\n", task));
            }
            log.push('\n');
        }

        if !data.sql_queries.is_empty() {
            log.push_str("## SQL 执行记录\n");
            log.push_str(&format!("今日共执行 {} 条 SQL 语句\n", data.sql_queries.len()));
            log.push('\n');
        }

        log
    }

    /// 验证API密钥是否有效
    ///
    /// 预留接口，用于测试API连接
    pub async fn validate_api_key(&self) -> Result<bool> {
        // TODO: 实现实际的API密钥验证
        if self.config.api_key.is_empty() {
            return Ok(false);
        }
        Ok(true)
    }

    /// 更新配置
    pub fn update_config(&mut self, config: AiConfig) {
        self.config = config;
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &AiConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_work_log() {
        let config = AiConfig::default();
        let service = AiService::new(config);

        let data = WorkLogData {
            date: "2025-01-23".to_string(),
            git_commits: vec![
                "feat: 添加任务看板功能".to_string(),
                "fix: 修复SQL历史记录问题".to_string(),
            ],
            sql_queries: vec!["SELECT * FROM tasks".to_string()],
            tasks_completed: vec!["实现任务CRUD".to_string()],
            tasks_active: vec!["集成AI服务".to_string()],
        };

        let log = service.format_work_log(&data);
        assert!(log.contains("工作日志"));
        assert!(log.contains("Git 提交记录"));
        assert!(log.contains("已完成任务"));
    }
}
