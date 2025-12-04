import { invoke } from '@tauri-apps/api/core'

export interface HeatmapData {
  date: string
  count: number
}

export interface DailyTrend {
  date: string
  totalCount: number
  codingCount: number
  browsingCount: number
  documentCount: number
  otherCount: number
}

export interface HourlyDistribution {
  hour: number
  count: number
}

export interface AppUsage {
  appName: string
  count: number
  percentage: number
}

export interface ActivityTypeStats {
  activityType: string
  count: number
  percentage: number
}

export const statisticsApi = {
  async getHeatmap(startDate?: string, endDate?: string): Promise<HeatmapData[]> {
    return invoke('statistics_get_heatmap', { startDate, endDate })
  },
  async getDailyTrend(days?: number): Promise<DailyTrend[]> {
    return invoke('statistics_get_daily_trend', { days })
  },
  async getHourlyDistribution(date?: string): Promise<HourlyDistribution[]> {
    return invoke('statistics_get_hourly_distribution', { date })
  },
  async getAppUsage(startDate?: string, endDate?: string): Promise<AppUsage[]> {
    return invoke('statistics_get_app_usage', { startDate, endDate })
  },
  async getActivityTypes(startDate?: string, endDate?: string): Promise<ActivityTypeStats[]> {
    return invoke('statistics_get_activity_types', { startDate, endDate })
  }
}
