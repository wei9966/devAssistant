import { invoke } from '@tauri-apps/api/core'
import type { ClipboardHistory, ClipboardConfig, ClipboardContentType } from '@/types/clipboard'

export const clipboardApi = {
  /**
   * 获取剪切板历史记录
   */
  async getHistory(
    limit?: number,
    offset?: number,
    contentType?: ClipboardContentType,
    keyword?: string,
    pinnedOnly?: boolean
  ): Promise<ClipboardHistory[]> {
    return await invoke('get_clipboard_history', {
      limit,
      offset,
      contentType,
      keyword,
      pinnedOnly
    })
  },

  /**
   * 从历史记录中复制内容
   */
  async copyFromHistory(id: number): Promise<string> {
    return await invoke('copy_from_clipboard_history', { id })
  },

  /**
   * 删除剪切板历史记录
   */
  async deleteItem(id: number): Promise<void> {
    return await invoke('delete_clipboard_history_item', { id })
  },

  /**
   * 清空剪切板历史
   */
  async clearHistory(keepPinned: boolean = true): Promise<number> {
    return await invoke('clear_clipboard_history', { keepPinned })
  },

  /**
   * 切换置顶状态
   */
  async togglePin(id: number): Promise<boolean> {
    return await invoke('toggle_clipboard_pin', { id })
  },

  /**
   * 搜索剪切板历史
   */
  async searchHistory(keyword: string, limit?: number): Promise<ClipboardHistory[]> {
    return await invoke('search_clipboard_history', { keyword, limit })
  },

  /**
   * 获取剪切板配置
   */
  async getConfig(): Promise<ClipboardConfig> {
    return await invoke('get_clipboard_config')
  },

  /**
   * 更新剪切板配置
   */
  async updateConfig(config: ClipboardConfig): Promise<void> {
    return await invoke('update_clipboard_config', { config })
  },

  /**
   * 获取历史记录数量
   */
  async getHistoryCount(): Promise<number> {
    return await invoke('get_clipboard_history_count')
  }
}

export default clipboardApi
