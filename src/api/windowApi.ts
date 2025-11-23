import { invoke } from '@tauri-apps/api/core'

/**
 * 显示窗口并聚焦
 */
export async function showWindow(): Promise<void> {
  await invoke('show_window')
}

/**
 * 隐藏窗口
 */
export async function hideWindow(): Promise<void> {
  await invoke('hide_window')
}

/**
 * 切换窗口显示状态
 */
export async function toggleWindow(): Promise<void> {
  await invoke('toggle_window')
}

/**
 * 显示窗口并导航到指定路由
 * @param route 路由路径,如 '/task-board', '/sql-history'
 */
export async function showWindowWithRoute(route: string): Promise<void> {
  await invoke('show_window_with_route', { route })
}
