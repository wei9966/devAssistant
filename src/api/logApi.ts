import { invoke } from '@tauri-apps/api/core';
import type { WorkLog } from '@/types/context';

export interface WorkLogRecord {
  id?: number;
  date: string;
  logType: 'daily' | 'weekly';
  content: string;
  aiGenerated: boolean;
  createdAt?: string;
  updatedAt?: string;
}

export const logApi = {
  // 获取工作日志
  async getWorkLogs(startDate?: string, endDate?: string): Promise<WorkLogRecord[]> {
    return await invoke('get_work_logs', { startDate, endDate });
  },

  // 获取指定日期的工作日志
  async getWorkLogByDate(date: string): Promise<WorkLogRecord | null> {
    return await invoke('get_work_log_by_date', { date });
  },

  // 创建或更新工作日志
  async saveWorkLog(
    date: string,
    content: string,
    logType: 'daily' | 'weekly' = 'daily',
    aiGenerated: boolean = false
  ): Promise<void> {
    await invoke('save_work_log', { date, content, logType, aiGenerated });
  },

  // 删除工作日志
  async deleteWorkLog(date: string): Promise<void> {
    await invoke('delete_work_log', { date });
  },

  // AI 生成工作日志
  async generateWorkLog(date: string, taskIds?: number[]): Promise<string> {
    return await invoke('generate_work_log', { date, taskIds: taskIds || null });
  },

  // 获取最近的工作日志列表
  async getRecentWorkLogs(limit: number = 10): Promise<WorkLogRecord[]> {
    return await invoke('get_recent_work_logs', { limit });
  },
};
