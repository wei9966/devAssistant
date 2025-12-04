import { invoke } from '@tauri-apps/api/core';

export interface ContextSettings {
  captureEnabled: boolean;
  captureInterval: number;
  similarityThreshold: number;
  retentionDays: number;
  excludedApps: string[];
  saveScreenshots: boolean;
}

export interface CaptureStatus {
  isRunning: boolean;
  lastCaptureAt: string | null;
  totalCapturesToday: number;
  skippedCount: number;
}

export interface ScreenContext {
  id: number;
  capturedAt: string;
  appName: string | null;
  windowTitle: string | null;
  activityType: string;
  description: string;
  keyContent: string | null;
}

export interface DayStats {
  totalCount: number;
  appDistribution: Record<string, number>;
  activityDistribution: Record<string, number>;
  timeRange: [string, string] | null;
}

export const contextApi = {
  // 启动采集
  startCapture: () => invoke<void>('context_start_capture'),

  // 停止采集
  stopCapture: () => invoke<void>('context_stop_capture'),

  // 获取采集状态
  getStatus: () => invoke<CaptureStatus>('context_get_status'),

  // 获取设置
  getSettings: () => invoke<ContextSettings>('context_get_settings'),

  // 更新设置
  updateSettings: (settings: ContextSettings) =>
    invoke<void>('context_update_settings', { settings }),

  // 按日期查询上下文
  listByDate: (date: string) =>
    invoke<ScreenContext[]>('context_list_by_date', { date }),

  // 获取当日统计
  getStats: (date: string) =>
    invoke<DayStats>('context_get_stats', { date }),

  // 基于上下文生成日报
  generateSummary: (date: string) =>
    invoke<string>('context_generate_summary', { date }),

  // 删除指定日期数据
  deleteByDate: (date: string) =>
    invoke<number>('context_delete_by_date', { date }),

  // 清理过期数据
  cleanupOld: () => invoke<number>('context_cleanup_old'),
};
