import { invoke } from '@tauri-apps/api/core'

export interface ScreenshotRecord {
  id: number
  capturedAt: string
  appName: string | null
  windowTitle: string | null
  activityType: string
  description: string
  keyContent: string | null
  screenshotPath: string | null
  processingTimeMs: number | null
}

export interface ScreenshotListParams {
  startDate?: string
  endDate?: string
  appName?: string
  activityType?: string
  page?: number
  pageSize?: number
}

export interface ScreenshotListResponse {
  records: ScreenshotRecord[]
  total: number
  page: number
  pageSize: number
}

export interface ScreenshotDetail {
  id: number
  capturedAt: string
  appName: string | null
  windowTitle: string | null
  activityType: string
  description: string
  keyContent: string | null
  screenshotPath: string | null
  processingTimeMs: number | null
  imageData: string | null
}

export interface ScreenshotResource {
  id: number
  path: string | null
  capturedAt: string
  appName: string | null
  description: string | null  // 单张截图的描述
}

export interface ActivityGroup {
  id: string
  startTime: string
  endTime: string
  title: string
  description: string
  activityType: string
  screenshots: ScreenshotResource[]
}

export const screenshotApi = {
  async list(params: ScreenshotListParams): Promise<ScreenshotListResponse> {
    // 后端期望参数名为 query
    return invoke('screenshot_list', { query: params })
  },

  async getDetail(id: number): Promise<ScreenshotDetail> {
    // 后端期望参数名为 screenshot_id (snake_case)
    return invoke('screenshot_get_detail', { screenshotId: id })
  },

  async getImage(path: string, thumbnail?: boolean): Promise<string> {
    // 后端期望参数名为 file_path 和 generate_thumbnail
    return invoke('screenshot_get_image', {
      filePath: path,
      generateThumbnail: thumbnail
    })
  },

  async getActivities(date: string): Promise<ActivityGroup[]> {
    // 获取指定日期的活动分组
    return invoke('screenshot_get_activities', { date })
  }
}
