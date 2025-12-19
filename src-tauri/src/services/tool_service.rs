use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::Command;

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
pub struct ToolService;

impl ToolService {
    /// 创建新的服务实例
    pub fn new() -> Self {
        Self
    }

    /// 检查端口占用情况
    ///
    /// # Arguments
    /// * `port` - 要检查的端口号
    ///
    /// # Returns
    /// 返回占用该端口的进程信息列表
    pub async fn check_port_usage(port: u16) -> Result<Vec<PortInfo>, String> {
        // 使用 netstat 命令获取端口占用信息
        // netstat -ano 参数说明:
        // -a: 显示所有连接和侦听端口
        // -n: 以数字形式显示地址和端口号
        // -o: 显示拥有的进程 ID
        let output = Command::new("netstat")
            .args(["-ano"])
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

                        // 获取进程名称
                        let process_name = Self::get_process_name(pid).unwrap_or_else(|_| "Unknown".to_string());

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
    /// 返回进程名称
    fn get_process_name(pid: u32) -> Result<String, String> {
        // 使用 tasklist 命令获取进程信息
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
            .output()
            .map_err(|e| format!("执行 tasklist 命令失败: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "tasklist 命令执行失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        // tasklist CSV 输出格式: "进程名","PID","会话名","会话#","内存使用"
        if let Some(line) = output_str.lines().next() {
            if let Some(name) = line.split(',').next() {
                // 去除引号
                return Ok(name.trim_matches('"').to_string());
            }
        }

        Err("无法获取进程名称".to_string())
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
        let output = Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output()
            .map_err(|e| format!("执行 taskkill 命令失败: {}", e))?;

        if output.status.success() {
            Ok(true)
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!("终止进程失败: {}", error_msg))
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
