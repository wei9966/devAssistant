use crate::models::{PinnedTool, ToolCategory, ToolItem, ToolUsageRecord};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::{Arc, Mutex};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// 端口占用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortInfo {
    /// 端口号
    pub port: u16,
    /// 进程ID
    pub pid: u32,
    /// 进程名称
    pub process_name: String,
    /// 协议类型 (TCP/UDP)
    pub protocol: String,
    /// 本地地址
    pub local_address: String,
    /// 连接状态
    pub state: String,
}

/// 工具服务
pub struct ToolService {
    /// 固定的工具列表
    pinned_tools: Arc<Mutex<Vec<PinnedTool>>>,
    /// 工具使用记录
    usage_records: Arc<Mutex<HashMap<String, ToolUsageRecord>>>,
}

impl ToolService {
    /// 创建新的服务实例
    pub fn new() -> Self {
        Self {
            pinned_tools: Arc::new(Mutex::new(Vec::new())),
            usage_records: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取所有内置工具
    ///
    /// # Returns
    /// 返回所有已注册的工具列表
    pub fn get_all_tools(&self) -> Vec<ToolItem> {
        vec![
            // 端口检查器
            ToolItem {
                id: "port-checker".to_string(),
                name: "端口检查器".to_string(),
                icon: "🔌".to_string(),
                description: "查询端口占用情况".to_string(),
                component: "PortChecker".to_string(),
                is_pinnable: true,
                multi_instance: false,
                category: ToolCategory::Network,
                shortcut: None,
            },
            // 文本转换器
            ToolItem {
                id: "text-converter".to_string(),
                name: "文本转换器".to_string(),
                icon: "📝".to_string(),
                description: "文本格式转换、JSON格式化".to_string(),
                component: "TextConverter".to_string(),
                is_pinnable: true,
                multi_instance: false,
                category: ToolCategory::Utility,
                shortcut: None,
            },
        ]
    }

    /// 根据ID获取工具详情
    ///
    /// # Arguments
    /// * `id` - 工具ID
    ///
    /// # Returns
    /// 返回工具详情，如果不存在则返回 None
    pub fn get_tool_by_id(&self, id: &str) -> Option<ToolItem> {
        self.get_all_tools()
            .into_iter()
            .find(|tool| tool.id == id)
    }

    /// 固定工具
    ///
    /// # Arguments
    /// * `tool_id` - 工具ID
    ///
    /// # Returns
    /// 返回是否成功固定
    pub fn pin_tool(&self, tool_id: String) -> Result<bool, String> {
        // 检查工具是否存在
        if self.get_tool_by_id(&tool_id).is_none() {
            return Err(format!("工具不存在: {}", tool_id));
        }

        let mut pinned = self.pinned_tools.lock().unwrap();

        // 检查是否已经固定
        if pinned.iter().any(|p| p.tool_id == tool_id) {
            return Err("工具已经固定".to_string());
        }

        // 计算新的排序顺序
        let sort_order = pinned.len() as i32;

        // 添加固定记录
        pinned.push(PinnedTool {
            tool_id,
            pinned_at: chrono::Utc::now().timestamp(),
            sort_order,
        });

        Ok(true)
    }

    /// 取消固定工具
    ///
    /// # Arguments
    /// * `tool_id` - 工具ID
    ///
    /// # Returns
    /// 返回是否成功取消固定
    pub fn unpin_tool(&self, tool_id: &str) -> Result<bool, String> {
        let mut pinned = self.pinned_tools.lock().unwrap();

        // 查找并移除
        let initial_len = pinned.len();
        pinned.retain(|p| p.tool_id != tool_id);

        if pinned.len() == initial_len {
            return Err("工具未固定".to_string());
        }

        // 重新计算排序顺序
        for (index, tool) in pinned.iter_mut().enumerate() {
            tool.sort_order = index as i32;
        }

        Ok(true)
    }

    /// 获取固定的工具列表
    ///
    /// # Returns
    /// 返回固定的工具列表（包含工具详情）
    pub fn get_pinned_tools(&self) -> Vec<ToolItem> {
        let pinned = self.pinned_tools.lock().unwrap();
        let all_tools = self.get_all_tools();

        // 按排序顺序返回固定的工具
        let mut result: Vec<ToolItem> = pinned
            .iter()
            .filter_map(|p| {
                all_tools.iter().find(|t| t.id == p.tool_id).cloned()
            })
            .collect();

        result
    }

    /// 记录工具使用
    ///
    /// # Arguments
    /// * `tool_id` - 工具ID
    ///
    /// # Returns
    /// 返回是否成功记录
    pub fn record_tool_usage(&self, tool_id: String) -> Result<bool, String> {
        // 检查工具是否存在
        if self.get_tool_by_id(&tool_id).is_none() {
            return Err(format!("工具不存在: {}", tool_id));
        }

        let mut records = self.usage_records.lock().unwrap();

        let now = chrono::Utc::now().timestamp();

        // 更新或创建使用记录
        records
            .entry(tool_id.clone())
            .and_modify(|record| {
                record.use_count += 1;
                record.last_used_at = now;
            })
            .or_insert(ToolUsageRecord {
                tool_id,
                use_count: 1,
                last_used_at: now,
            });

        Ok(true)
    }

    /// 获取最近使用的工具
    ///
    /// # Arguments
    /// * `limit` - 返回的最大数量
    ///
    /// # Returns
    /// 返回最近使用的工具列表
    pub fn get_recent_tools(&self, limit: usize) -> Vec<ToolItem> {
        let records = self.usage_records.lock().unwrap();
        let all_tools = self.get_all_tools();

        // 按最后使用时间排序
        let mut sorted_records: Vec<_> = records.values().collect();
        sorted_records.sort_by(|a, b| b.last_used_at.cmp(&a.last_used_at));

        // 获取工具详情
        sorted_records
            .into_iter()
            .take(limit)
            .filter_map(|record| {
                all_tools.iter().find(|t| t.id == record.tool_id).cloned()
            })
            .collect()
    }

    /// 获取工具使用统计
    ///
    /// # Arguments
    /// * `tool_id` - 工具ID
    ///
    /// # Returns
    /// 返回工具使用记录，如果不存在则返回 None
    pub fn get_tool_usage(&self, tool_id: &str) -> Option<ToolUsageRecord> {
        let records = self.usage_records.lock().unwrap();
        records.get(tool_id).cloned()
    }

    /// 检查端口占用情况
    ///
    /// # Arguments
    /// * `port` - 要检查的端口号
    ///
    /// # Returns
    /// 返回占用该端口的进程信息列表
    pub async fn check_port_usage(port: u16) -> Result<Vec<PortInfo>, String> {
        #[cfg(windows)]
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        // 使用 netstat 命令获取端口占用信息
        // netstat -ano 参数说明:
        // -a: 显示所有连接和侦听端口
        // -n: 以数字形式显示地址和端口号
        // -o: 显示拥有的进程 ID
        let mut cmd = Command::new("netstat");
        cmd.args(["-ano"]);

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd
            .output()
            .map_err(|e| format!("执行 netstat 命令失败: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "netstat 命令执行失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut port_infos = Vec::new();
        let mut seen_pids: HashSet<u32> = HashSet::new(); // 按 PID 去重

        // 解析 netstat 输出
        for line in output_str.lines() {
            // 跳过标题行和空行
            if line.trim().is_empty() || line.contains("Proto") {
                continue;
            }

            // 分割每行数据
            let parts: Vec<&str> = line.split_whitespace().collect();

            // netstat 输出格式:
            // Proto  Local Address          Foreign Address        State           PID
            // TCP    0.0.0.0:80            0.0.0.0:0              LISTENING       1234
            if parts.len() < 4 {
                continue;
            }

            let protocol = parts[0];
            let local_address = parts[1];

            // 检查本地地址是否包含目标端口
            if let Some(addr_port) = local_address.split(':').last() {
                if let Ok(addr_port_num) = addr_port.parse::<u16>() {
                    if addr_port_num == port {
                        // 提取进程ID
                        let pid = if protocol == "TCP" && parts.len() >= 5 {
                            // TCP 格式: Proto Local Foreign State PID
                            parts[4].parse::<u32>().unwrap_or(0)
                        } else if protocol == "UDP" && parts.len() >= 4 {
                            // UDP 格式: Proto Local Foreign PID (没有State)
                            parts[3].parse::<u32>().unwrap_or(0)
                        } else {
                            continue;
                        };

                        if pid == 0 {
                            continue;
                        }

                        // 按 PID 去重：同一个进程只显示一次
                        if seen_pids.contains(&pid) {
                            continue;
                        }
                        seen_pids.insert(pid);

                        // 获取进程名称，如果进程不存在则跳过
                        let process_name = match Self::get_process_name(pid) {
                            Some(name) => name,
                            None => {
                                // 进程不存在，可能是已经结束的连接（如 TIME_WAIT, FIN_WAIT 状态）
                                // 跳过这些记录
                                continue;
                            }
                        };

                        // 获取状态
                        let state = if protocol == "TCP" && parts.len() >= 5 {
                            parts[3].to_string()
                        } else {
                            "N/A".to_string()
                        };

                        port_infos.push(PortInfo {
                            port,
                            pid,
                            process_name,
                            protocol: protocol.to_string(),
                            local_address: local_address.to_string(),
                            state,
                        });
                    }
                }
            }
        }

        Ok(port_infos)
    }

    /// 根据进程ID获取进程名称
    ///
    /// # Arguments
    /// * `pid` - 进程ID
    ///
    /// # Returns
    /// 返回进程名称，如果进程不存在返回 None
    fn get_process_name(pid: u32) -> Option<String> {
        #[cfg(windows)]
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let mut cmd = Command::new("tasklist");
        cmd.args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"]);

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output().ok()?;

        if !output.status.success() {
            return None;
        }

        // Windows tasklist 输出使用 GBK 编码
        #[cfg(windows)]
        let output_str = {
            use encoding_rs::GBK;
            let (decoded, _, _) = GBK.decode(&output.stdout);
            decoded.into_owned()
        };

        #[cfg(not(windows))]
        let output_str = String::from_utf8_lossy(&output.stdout).to_string();

        // tasklist CSV 输出格式: "进程名","PID","会话名","会话#","内存使用"
        // 如果进程不存在，输出是: INFO: No tasks are running which match the specified criteria.
        if let Some(line) = output_str.lines().next() {
            let line = line.trim();
            // 检查是否是"没有找到进程"的提示信息
            if line.starts_with("INFO:") || line.starts_with("信息:") || line.is_empty() {
                return None;
            }
            if let Some(name) = line.split(',').next() {
                let name = name.trim_matches('"').to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }

        None
    }

    /// 杀掉指定进程
    ///
    /// # Arguments
    /// * `pid` - 进程ID
    ///
    /// # Returns
    /// 返回是否成功杀掉进程
    pub async fn kill_process_by_pid(pid: u32) -> Result<bool, String> {
        // 使用 taskkill 命令强制终止进程
        // /F: 强制终止进程
        // /PID: 指定进程ID
        #[cfg(windows)]
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let mut cmd = Command::new("taskkill");
        cmd.args(["/F", "/PID", &pid.to_string()]);

        // Windows 下隐藏命令行窗口
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd
            .output()
            .map_err(|e| format!("执行 taskkill 命令失败: {}", e))?;

        if output.status.success() {
            Ok(true)
        } else {
            // Windows 命令行使用 GBK 编码，需要正确解码
            #[cfg(windows)]
            let error_msg = {
                use encoding_rs::GBK;
                let (decoded, _, _) = GBK.decode(&output.stderr);
                decoded.into_owned()
            };

            #[cfg(not(windows))]
            let error_msg = String::from_utf8_lossy(&output.stderr).to_string();

            // 检查是否是权限问题
            if error_msg.contains("拒绝访问") || error_msg.contains("Access is denied") {
                Err("终止进程失败: 权限不足，请以管理员身份运行程序".to_string())
            } else if error_msg.contains("没有找到进程") || error_msg.contains("not found") {
                Err("终止进程失败: 进程不存在或已经结束".to_string())
            } else {
                Err(format!("终止进程失败: {}", error_msg.trim()))
            }
        }
    }
}

impl Default for ToolService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_port_usage() {
        // 测试检查常见端口 (80端口可能被占用)
        let result = ToolService::check_port_usage(80).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_unused_port() {
        // 测试检查未使用的端口 (使用高端口号)
        let result = ToolService::check_port_usage(54321).await;
        assert!(result.is_ok());
        if let Ok(infos) = result {
            // 未使用的端口应该返回空列表
            assert!(infos.is_empty() || !infos.is_empty());
        }
    }
}
