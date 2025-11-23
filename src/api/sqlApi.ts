import { invoke } from '@tauri-apps/api/core';
import type { SqlRecord } from '@/types/sql';

export const sqlApi = {
  // 保存 SQL
  async saveSql(sqlText: string, source: string): Promise<number> {
    return await invoke('save_sql', { sqlText, source });
  },

  // 获取最近的 SQL
  async getRecentSqls(limit: number): Promise<SqlRecord[]> {
    return await invoke('get_recent_sqls', { limit });
  },

  // 获取收藏的 SQL
  async getFavoriteSqls(): Promise<SqlRecord[]> {
    return await invoke('get_favorite_sqls');
  },

  // 切换收藏状态
  async toggleFavorite(sqlId: number): Promise<void> {
    await invoke('toggle_favorite_sql', { sqlId });
  },

  // 删除 SQL
  async deleteSql(sqlId: number): Promise<void> {
    await invoke('delete_sql', { sqlId });
  },
};
