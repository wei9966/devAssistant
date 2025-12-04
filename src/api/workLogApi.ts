import { invoke } from '@tauri-apps/api/core';
import type { WorkLog, DayContextSummary, WeekContextSummary } from '@/types/workLog';

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

  // 基于上下文生成日报
  async generateWithContext(date: string): Promise<string> {
    return await invoke('work_log_generate_with_context', { date });
  },

  // 获取当日上下文摘要
  async getContextSummary(date: string): Promise<DayContextSummary> {
    return await invoke('context_get_day_summary', { date });
  },

  // 基于上下文生成周报
  async generateWeeklyWithContext(startDate: string, endDate: string): Promise<string> {
    return await invoke('work_log_generate_weekly_with_context', { startDate, endDate });
  },

  // 获取周上下文摘要
  async getWeekContextSummary(startDate: string, endDate: string): Promise<WeekContextSummary> {
    return await invoke('context_get_week_summary', { startDate, endDate });
  },
};
