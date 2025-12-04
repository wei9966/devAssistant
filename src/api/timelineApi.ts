import { invoke } from '@tauri-apps/api/core';
import type { TimelineItem, TimelineGroup, TimelineFilter } from '@/types/timeline';

export const timelineApi = {
  // 获取时间线数据
  async getTimeline(filter?: TimelineFilter): Promise<TimelineGroup[]> {
    return invoke('get_timeline', { filter });
  },

  // 生成日报
  async generateDailyReport(date: string): Promise<string> {
    return invoke('generate_daily_report', { date });
  },

  // 手动合并选中项
  async mergeItems(itemIds: string[]): Promise<TimelineItem> {
    return invoke('merge_timeline_items', { itemIds });
  },

  // 获取指定日期的活动
  async getActivitiesByDate(date: string): Promise<TimelineItem[]> {
    return invoke('get_activities_by_date', { date });
  }
};
