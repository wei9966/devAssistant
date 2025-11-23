use std::process::Command;
use anyhow::{Result, Context};

pub struct GitService;

impl GitService {
    /// 获取当前 Git 分支
    pub fn get_current_branch() -> Result<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .context("执行 git 命令失败")?;

        if !output.status.success() {
            anyhow::bail!("不是 Git 仓库或 git 命令不可用");
        }

        let branch = String::from_utf8(output.stdout)
            .context("解析 git 输出失败")?
            .trim()
            .to_string();

        Ok(branch)
    }

    /// 获取所有未合并的分支
    pub fn get_unmerged_branches(base_branch: &str) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["branch", "--no-merged", base_branch])
            .output()
            .context("获取未合并分支失败")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let branches: Vec<String> = String::from_utf8(output.stdout)
            .context("解析分支列表失败")?
            .lines()
            .map(|line| line.trim().trim_start_matches('*').trim().to_string())
            .filter(|branch| !branch.is_empty())
            .collect();

        Ok(branches)
    }

    /// 获取指定分支的最后一次提交信息
    pub fn get_last_commit(branch: &str) -> Result<CommitInfo> {
        let output = Command::new("git")
            .args(["log", branch, "-1", "--format=%H|%s|%an|%at"])
            .output()
            .context("获取提交信息失败")?;

        if !output.status.success() {
            anyhow::bail!("获取分支 {} 的提交信息失败", branch);
        }

        let commit_str = String::from_utf8(output.stdout)
            .context("解析提交信息失败")?;

        let parts: Vec<&str> = commit_str.trim().split('|').collect();

        if parts.len() < 4 {
            anyhow::bail!("提交信息格式不正确");
        }

        Ok(CommitInfo {
            hash: parts[0].to_string(),
            message: parts[1].to_string(),
            author: parts[2].to_string(),
            timestamp: parts[3].parse().unwrap_or(0),
        })
    }

    /// 获取最近 N 天的提交记录
    pub fn get_recent_commits(days: usize) -> Result<Vec<CommitInfo>> {
        let since = format!("{} days ago", days);

        let output = Command::new("git")
            .args(["log", "--since", &since, "--format=%H|%s|%an|%at"])
            .output()
            .context("获取提交记录失败")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let commits: Vec<CommitInfo> = String::from_utf8(output.stdout)
            .context("解析提交记录失败")?
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 4 {
                    Some(CommitInfo {
                        hash: parts[0].to_string(),
                        message: parts[1].to_string(),
                        author: parts[2].to_string(),
                        timestamp: parts[3].parse().unwrap_or(0),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(commits)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: i64,
}
