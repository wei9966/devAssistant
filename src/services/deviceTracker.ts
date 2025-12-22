/**
 * DevAssistant 设备追踪服务
 *
 * 功能：
 * 1. 收集设备信息（MAC地址、计算机名等）
 * 2. 定期上报心跳到服务器
 * 3. 检查设备禁用状态
 * 4. 提供申诉功能
 */

import { invoke } from '@tauri-apps/api/core'

// 服务器地址配置
const API_BASE_URL = 'http://d.wbdao.cn:9900/api'

// 心跳上报间隔（2小时）
const HEARTBEAT_INTERVAL = 2 * 60 * 60 * 1000

// 设备信息接口
export interface DeviceInfo {
  mac_address: string
  hostname: string
  username: string
  os_info: string
  local_ip: string
}

// 设备状态接口
export interface DeviceStatus {
  status: 'active' | 'banned'
  reason?: string
  ban_time?: string
  can_appeal?: boolean
  has_pending_appeal?: boolean
}

// 申诉结果接口
export interface AppealResult {
  success: boolean
  message: string
  appeal_id?: number
}

// 申诉状态接口
export interface AppealStatus {
  id: number
  status: number
  status_text: string
  admin_reply?: string
  process_time?: string
  created_at: string
}

// 设备追踪器类
class DeviceTracker {
  private deviceInfo: DeviceInfo | null = null
  private heartbeatTimer: number | null = null
  private statusCheckTimer: number | null = null
  private onBannedCallback: ((status: DeviceStatus) => void) | null = null
  private isEnabled: boolean = true

  /**
   * 初始化设备追踪器
   */
  async init(onBanned?: (status: DeviceStatus) => void): Promise<void> {
    if (onBanned) {
      this.onBannedCallback = onBanned
    }

    try {
      // 获取设备信息
      this.deviceInfo = await this.getDeviceInfo()

      // 立即检查状态
      await this.checkStatus()

      // 立即发送心跳
      await this.sendHeartbeat()

      // 启动定时心跳
      this.startHeartbeat()

      console.log('[DeviceTracker] 初始化成功')
    } catch (error) {
      console.error('[DeviceTracker] 初始化失败:', error)
    }
  }

  /**
   * 获取设备信息
   */
  private async getDeviceInfo(): Promise<DeviceInfo> {
    try {
      const info = await invoke<DeviceInfo>('get_device_info')
      return info
    } catch (error) {
      console.error('[DeviceTracker] 获取设备信息失败:', error)
      throw error
    }
  }

  /**
   * 获取应用版本
   */
  private async getAppVersion(): Promise<string> {
    try {
      const version = await invoke<string>('get_app_version')
      return version
    } catch (error) {
      return '1.0.0'
    }
  }

  /**
   * 发送心跳
   */
  async sendHeartbeat(): Promise<boolean> {
    if (!this.isEnabled || !this.deviceInfo) {
      return false
    }

    try {
      const version = await this.getAppVersion()

      const response = await fetch(`${API_BASE_URL}/heartbeat.php`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          mac_address: this.deviceInfo.mac_address,
          hostname: this.deviceInfo.hostname,
          username: this.deviceInfo.username,
          os_info: this.deviceInfo.os_info,
          local_ip: this.deviceInfo.local_ip,
          app_version: version,
        }),
      })

      const result = await response.json()

      if (response.status === 403) {
        // 设备被禁用
        if (this.onBannedCallback) {
          this.onBannedCallback({
            status: 'banned',
            reason: result.data?.reason || '违反使用条款',
            can_appeal: result.data?.can_appeal ?? true,
          })
        }
        return false
      }

      return response.ok
    } catch (error) {
      console.error('[DeviceTracker] 心跳发送失败:', error)
      return false
    }
  }

  /**
   * 检查设备状态
   */
  async checkStatus(): Promise<DeviceStatus> {
    if (!this.deviceInfo) {
      return { status: 'active' }
    }

    try {
      const response = await fetch(`${API_BASE_URL}/check_status.php`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          mac_address: this.deviceInfo.mac_address,
        }),
      })

      const result = await response.json()

      if (response.status === 403) {
        const status: DeviceStatus = {
          status: 'banned',
          reason: result.data?.reason || '违反使用条款',
          ban_time: result.data?.ban_time,
          can_appeal: result.data?.can_appeal ?? true,
          has_pending_appeal: result.data?.has_pending_appeal ?? false,
        }

        if (this.onBannedCallback) {
          this.onBannedCallback(status)
        }

        return status
      }

      return { status: 'active' }
    } catch (error) {
      console.error('[DeviceTracker] 检查状态失败:', error)
      return { status: 'active' }
    }
  }

  /**
   * 提交申诉
   */
  async submitAppeal(contact: string, reason: string): Promise<AppealResult> {
    if (!this.deviceInfo) {
      return { success: false, message: '设备信息不可用' }
    }

    try {
      const response = await fetch(`${API_BASE_URL}/appeal.php`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          mac_address: this.deviceInfo.mac_address,
          contact,
          reason,
        }),
      })

      const result = await response.json()

      if (response.ok) {
        return {
          success: true,
          message: result.message || '申诉提交成功',
          appeal_id: result.data?.appeal_id,
        }
      }

      return {
        success: false,
        message: result.message || '申诉提交失败',
      }
    } catch (error) {
      console.error('[DeviceTracker] 提交申诉失败:', error)
      return {
        success: false,
        message: '网络错误，请稍后重试',
      }
    }
  }

  /**
   * 查询申诉状态
   */
  async getAppealStatus(): Promise<AppealStatus[]> {
    if (!this.deviceInfo) {
      return []
    }

    try {
      const response = await fetch(`${API_BASE_URL}/appeal_status.php`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          mac_address: this.deviceInfo.mac_address,
        }),
      })

      const result = await response.json()

      if (response.ok && result.data?.appeals) {
        return result.data.appeals
      }

      return []
    } catch (error) {
      console.error('[DeviceTracker] 获取申诉状态失败:', error)
      return []
    }
  }

  /**
   * 启动心跳定时器
   */
  private startHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
    }

    this.heartbeatTimer = window.setInterval(async () => {
      await this.sendHeartbeat()
    }, HEARTBEAT_INTERVAL)
  }

  /**
   * 停止心跳
   */
  stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = null
    }
  }

  /**
   * 禁用追踪（用于开发模式）
   */
  disable(): void {
    this.isEnabled = false
    this.stopHeartbeat()
  }

  /**
   * 获取当前设备信息
   */
  getInfo(): DeviceInfo | null {
    return this.deviceInfo
  }
}

// 导出单例
export const deviceTracker = new DeviceTracker()

// 默认导出
export default deviceTracker
