// Tauri API 调用封装
import { invoke } from '@tauri-apps/api/core'

// 将在后续任务中添加具体的API调用函数
export const api = {
  // 示例：调用Rust命令
  // async exampleCommand(params: any) {
  //   return await invoke('example_command', params)
  // }
}

// Export API modules
export * from './screenshotApi';
export * from './aiChatApi';
