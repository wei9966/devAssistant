export type NotificationType = 'tip' | 'daily_report' | 'weekly_report'

export interface Notification {
  id: number
  notificationType: NotificationType
  title: string
  content: string
  isRead: boolean
  createdAt: string
}

export interface NotificationSettings {
  tipsEnabled: boolean
  tipsIntervalMinutes: number
  tipsMaxPerDay: number
  dailyReportEnabled: boolean
  dailyReportTime: string
  weeklyReportEnabled: boolean
  weeklyReportDay: number  // 0=周日
  weeklyReportTime: string
}
