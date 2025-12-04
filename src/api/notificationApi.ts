import { invoke } from '@tauri-apps/api/core'
import type { Notification, NotificationSettings, NotificationType } from '@/types/notification'

export interface NotificationListParams {
  notificationType?: NotificationType
  isRead?: boolean
  limit?: number
}

export const notificationApi = {
  // 获取通知列表
  list: (params?: NotificationListParams) =>
    invoke<Notification[]>('notification_list', params || {}),

  // 获取未读数量
  getUnreadCount: () =>
    invoke<number>('notification_get_unread_count'),

  // 标记已读
  markRead: (id: number) =>
    invoke<void>('notification_mark_read', { id }),

  // 全部已读
  markAllRead: () =>
    invoke<void>('notification_mark_all_read'),

  // 删除通知
  delete: (id: number) =>
    invoke<void>('notification_delete', { id }),

  // 清空所有
  clearAll: () =>
    invoke<void>('notification_clear_all'),

  // 获取设置
  getSettings: () =>
    invoke<NotificationSettings>('notification_get_settings'),

  // 更新设置
  updateSettings: (settings: NotificationSettings) =>
    invoke<void>('notification_update_settings', { settings }),

  // 手动生成日报
  generateDailyReport: () =>
    invoke<void>('notification_generate_daily_report'),

  // 手动生成周报
  generateWeeklyReport: () =>
    invoke<void>('notification_generate_weekly_report'),
}
