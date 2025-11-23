import { invoke } from '@tauri-apps/api/core';
import type { WorkLog } from '@/types/workLog';

export const workLogApi = {
  // 保存工作日志
  async saveWorkLog(
    date: string,
    logType: string,
    content: string,
    aiGenerated: boolean = false
  ): Promise<void> {
    await invoke('save_work_log', {
      date,
      logType,
      content,
      aiGenerated,
    });
  },

  // 获取指定日期的工作日志
  async getWorkLog(date: string): Promise<WorkLog | null> {
    return await invoke('get_work_log', { date });
  },

  // 获取最近N天的工作日志
  async getRecentWorkLogs(days: number): Promise<WorkLog[]> {
    return await invoke('get_recent_work_logs', { days });
  },

  // 删除指定日期的工作日志
  async deleteWorkLog(date: string): Promise<void> {
    await invoke('delete_work_log', { date });
  },

  // 获取所有工作日志
  async getAllWorkLogs(): Promise<WorkLog[]> {
    return await invoke('get_all_work_logs');
  },
};
