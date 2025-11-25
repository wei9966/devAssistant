use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoStartStatus {
    pub enabled: bool,
    pub app_path: String,
}

/// 获取应用程序的可执行文件路径
#[cfg(target_os = "windows")]
fn get_app_path() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("无法获取应用程序路径: {}", e))
}

/// 获取开机自启动状态
#[tauri::command]
pub fn get_autostart_status() -> Result<AutoStartStatus, String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu
            .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .map_err(|e| format!("无法打开注册表: {}", e))?;

        let app_name = "DevAssistant";
        let app_path = get_app_path()?;
        let current_path = app_path.to_string_lossy().to_string();

        match run_key.get_value::<String, _>(app_name) {
            Ok(value) => {
                // 检查注册表中的路径是否与当前路径匹配
                let enabled = value.trim_matches('"') == current_path;
                Ok(AutoStartStatus {
                    enabled,
                    app_path: current_path,
                })
            }
            Err(_) => Ok(AutoStartStatus {
                enabled: false,
                app_path: current_path,
            }),
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS 实现 - 使用 LaunchAgents
        // TODO: 实现 macOS 自启动检测
        Err("macOS 开机自启动功能尚未实现".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        // Linux 实现 - 使用 ~/.config/autostart/
        // TODO: 实现 Linux 自启动检测
        Err("Linux 开机自启动功能尚未实现".to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("当前操作系统不支持开机自启动".to_string())
    }
}

/// 设置开机自启动
#[tauri::command]
pub fn set_autostart(enable: bool) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu
            .open_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                KEY_WRITE,
            )
            .map_err(|e| format!("无法打开注册表写入权限: {}", e))?;

        let app_name = "DevAssistant";

        if enable {
            // 启用自启动 - 写入注册表
            let app_path = get_app_path()?;
            let path_str = format!("\"{}\"", app_path.to_string_lossy());

            run_key
                .set_value(app_name, &path_str)
                .map_err(|e| format!("无法设置自启动: {}", e))?;

            Ok(true)
        } else {
            // 禁用自启动 - 删除注册表项
            match run_key.delete_value(app_name) {
                Ok(_) => Ok(false),
                Err(e) => {
                    // 如果项不存在,也认为是成功的
                    if e.kind() == std::io::ErrorKind::NotFound {
                        Ok(false)
                    } else {
                        Err(format!("无法删除自启动项: {}", e))
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS 实现
        if enable {
            // TODO: 创建 ~/Library/LaunchAgents/com.devassistant.app.plist
            Err("macOS 开机自启动功能尚未实现".to_string())
        } else {
            // TODO: 删除对应的 plist 文件
            Err("macOS 开机自启动功能尚未实现".to_string())
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux 实现
        if enable {
            // TODO: 创建 ~/.config/autostart/devassistant.desktop
            Err("Linux 开机自启动功能尚未实现".to_string())
        } else {
            // TODO: 删除对应的 .desktop 文件
            Err("Linux 开机自启动功能尚未实现".to_string())
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("当前操作系统不支持开机自启动".to_string())
    }
}

/// 切换开机自启动状态
#[tauri::command]
pub fn toggle_autostart() -> Result<bool, String> {
    let status = get_autostart_status()?;
    set_autostart(!status.enabled)
}
