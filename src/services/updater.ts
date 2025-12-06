import { check, Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'

// 记录更新日志到后端
async function logUpdate(message: string) {
  try {
    await invoke('log_update_info', { message })
  } catch (e) {
    // 静默失败
  }
}

// 存储键名
const SKIPPED_VERSION_KEY = 'skipped_version'
const LAST_CHECK_TIME_KEY = 'last_update_check_time'

// 每1小时检查一次更新（毫秒）- 调试期间缩短间隔
const CHECK_INTERVAL = 1 * 60 * 60 * 1000

export interface UpdateInfo {
  available: boolean
  version?: string
  notes?: string
  date?: string
}

export interface UpdateProgress {
  downloaded: number
  total: number
}

/**
 * 检查更新（考虑跳过的版本）
 */
export async function checkForUpdate(): Promise<UpdateInfo> {
  try {
    // 获取当前版本
    let currentVersion = '未知'
    try {
      currentVersion = await getVersion()
    } catch (e) {
      // 忽略
    }

    await logUpdate(`========== 开始检查更新 ==========`)
    await logUpdate(`当前应用版本: ${currentVersion}`)
    await logUpdate(`调用 Tauri updater check() 方法...`)

    const update = await check()

    if (!update) {
      await logUpdate(`服务器返回: 没有可用更新（当前已是最新版本）`)
      await logUpdate(`========== 更新检查结束 ==========`)
      return { available: false }
    }

    await logUpdate(`服务器返回更新信息:`)
    await logUpdate(`  - 新版本号: ${update.version}`)
    await logUpdate(`  - 发布日期: ${update.date || '未知'}`)
    await logUpdate(`  - 更新说明: ${update.body?.substring(0, 100) || '无'}...`)

    // 检查是否是被跳过的版本
    const skippedVersion = getSkippedVersion()
    if (skippedVersion && update.version === skippedVersion) {
      await logUpdate(`版本 ${update.version} 已被用户跳过，忽略此更新`)
      await logUpdate(`========== 更新检查结束 ==========`)
      return { available: false }
    }

    await logUpdate(`发现新版本: ${update.version}，需要更新`)
    await logUpdate(`========== 更新检查结束 ==========`)

    return {
      available: true,
      version: update.version,
      notes: update.body,
      date: update.date
    }
  } catch (error: any) {
    await logUpdate(`检查更新失败: ${error?.message || error}`)
    await logUpdate(`========== 更新检查结束 ==========`)
    throw error
  }
}

/**
 * 获取跳过的版本
 */
export function getSkippedVersion(): string | null {
  try {
    return localStorage.getItem(SKIPPED_VERSION_KEY)
  } catch (error) {
    console.error('获取跳过版本失败:', error)
    return null
  }
}

/**
 * 设置跳过的版本
 */
export function setSkippedVersion(version: string): void {
  try {
    localStorage.setItem(SKIPPED_VERSION_KEY, version)
  } catch (error) {
    console.error('设置跳过版本失败:', error)
  }
}

/**
 * 清除跳过的版本
 */
export function clearSkippedVersion(): void {
  try {
    localStorage.removeItem(SKIPPED_VERSION_KEY)
  } catch (error) {
    console.error('清除跳过版本失败:', error)
  }
}

/**
 * 下载并安装更新
 */
export async function downloadAndInstall(
  update: Update,
  onProgress?: (progress: UpdateProgress) => void
): Promise<void> {
  try {
    let downloaded = 0
    let contentLength = 0

    // 下载更新
    await update.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength || 0
          if (onProgress) {
            onProgress({
              downloaded: 0,
              total: contentLength
            })
          }
          break
        case 'Progress':
          downloaded += event.data.chunkLength
          if (onProgress) {
            onProgress({
              downloaded,
              total: contentLength
            })
          }
          break
        case 'Finished':
          if (onProgress) {
            onProgress({
              downloaded: contentLength,
              total: contentLength
            })
          }
          break
      }
    })

    // 清除跳过的版本记录
    clearSkippedVersion()
  } catch (error) {
    console.error('下载并安装更新失败:', error)
    throw error
  }
}

/**
 * 重启应用
 */
export async function restartApp(): Promise<void> {
  try {
    await relaunch()
  } catch (error) {
    console.error('重启应用失败:', error)
    throw error
  }
}

/**
 * 获取上次检查时间
 */
export function getLastCheckTime(): number | null {
  try {
    const time = localStorage.getItem(LAST_CHECK_TIME_KEY)
    return time ? parseInt(time, 10) : null
  } catch (error) {
    console.error('获取上次检查时间失败:', error)
    return null
  }
}

/**
 * 设置上次检查时间
 */
export function setLastCheckTime(): void {
  try {
    localStorage.setItem(LAST_CHECK_TIME_KEY, Date.now().toString())
  } catch (error) {
    console.error('设置上次检查时间失败:', error)
  }
}

/**
 * 是否应该检查更新（每天最多检查一次）
 */
export function shouldCheckUpdate(): boolean {
  try {
    const lastCheckTime = getLastCheckTime()

    // 如果从未检查过，返回 true
    if (!lastCheckTime) {
      return true
    }

    // 检查是否已经过了24小时
    const now = Date.now()
    return now - lastCheckTime >= CHECK_INTERVAL
  } catch (error) {
    console.error('判断是否应该检查更新失败:', error)
    return false
  }
}
