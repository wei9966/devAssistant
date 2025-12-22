import { invoke } from '@tauri-apps/api/core'

export interface PortInfo {
  port: number
  pid: number
  processName: string
  protocol: string
  localAddress: string
  state: string
}

/** 工具分类 */
export type ToolCategory = 'network' | 'system' | 'development' | 'file' | 'utility' | 'other'

/** 工具项 */
export interface ToolItem {
  id: string
  name: string
  icon: string
  description: string
  component: string
  isPinnable: boolean
  multiInstance: boolean
  category: ToolCategory
  shortcut?: string
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

/**
 * 获取所有工具列表
 * @returns 工具列表
 */
export async function getAllTools(): Promise<ToolItem[]> {
  return await invoke('get_all_tools')
}

/**
 * 根据ID获取工具
 * @param toolId 工具ID
 * @returns 工具详情
 */
export async function getToolById(toolId: string): Promise<ToolItem | null> {
  return await invoke('get_tool_by_id', { toolId })
}

/**
 * 固定工具
 * @param toolId 工具ID
 * @returns 是否成功
 */
export async function pinTool(toolId: string): Promise<boolean> {
  return await invoke('pin_tool', { toolId })
}

/**
 * 取消固定工具
 * @param toolId 工具ID
 * @returns 是否成功
 */
export async function unpinTool(toolId: string): Promise<boolean> {
  return await invoke('unpin_tool', { toolId })
}

/**
 * 获取固定的工具列表
 * @returns 固定的工具列表
 */
export async function getPinnedTools(): Promise<ToolItem[]> {
  return await invoke('get_pinned_tools')
}

/**
 * 记录工具使用
 * @param toolId 工具ID
 * @returns 是否成功
 */
export async function recordToolUsage(toolId: string): Promise<boolean> {
  return await invoke('record_tool_usage', { toolId })
}

/**
 * 获取最近使用的工具
 * @param limit 返回数量
 * @returns 最近使用的工具列表
 */
export async function getRecentTools(limit: number): Promise<ToolItem[]> {
  return await invoke('get_recent_tools', { limit })
}

/**
 * 打开工具容器窗口
 * @param toolId 工具ID
 */
export async function openToolContainer(toolId: string): Promise<void> {
  await invoke('open_tool_container', { toolId })
}
