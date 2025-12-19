import { invoke } from '@tauri-apps/api/core'

export interface PortInfo {
  port: number
  pid: number
  processName: string
  protocol: string
  localAddress: string
  state: string
}

/**
 * 检查端口占用
 * @param port 要检查的端口号
 * @returns 占用该端口的进程信息列表
 */
export async function checkPortUsage(port: number): Promise<PortInfo[]> {
  return await invoke('check_port_usage', { port })
}

/**
 * 杀掉进程
 * @param pid 进程ID
 * @returns 是否成功杀掉进程
 */
export async function killProcessByPid(pid: number): Promise<boolean> {
  return await invoke('kill_process_by_pid', { pid })
}

/**
 * 打开工具窗口
 * @param toolId 工具ID
 */
export async function openToolWindow(toolId: string): Promise<void> {
  await invoke('open_tool_window', { toolId })
}
