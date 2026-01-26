import { invoke } from '@tauri-apps/api/core'

export interface SedentaryConfig {
  enabled: boolean
  reminderIntervalMinutes: number
  tips: string[]
}

export interface SedentaryStatus {
  isRunning: boolean
  continuousWorkSeconds: number
  lastReminderTime: number | null
}

// 后端返回的配置格式（蛇形命名）
interface SedentaryConfigRaw {
  enabled: boolean
  reminder_interval_minutes: number
  idle_threshold_seconds?: number
  tips: string[]
}

export const sedentaryApi = {
  // 获取配置
  getConfig: async (): Promise<SedentaryConfig> => {
    const raw = await invoke<SedentaryConfigRaw>('get_sedentary_config')
    return {
      enabled: raw.enabled,
      reminderIntervalMinutes: raw.reminder_interval_minutes,
      tips: raw.tips
    }
  },

  // 保存配置
  saveConfig: (config: SedentaryConfig) =>
    invoke<void>('save_sedentary_config', {
      config: {
        enabled: config.enabled,
        reminder_interval_minutes: config.reminderIntervalMinutes,
        tips: config.tips
      }
    }),

  // 启动提醒服务
  start: () => invoke<void>('start_sedentary_reminder'),

  // 停止提醒服务
  stop: () => invoke<void>('stop_sedentary_reminder'),

  // 获取状态
  getStatus: () => invoke<SedentaryStatus>('get_sedentary_status'),

  // 重置计时器
  resetTimer: () => invoke<void>('reset_sedentary_timer'),
}
