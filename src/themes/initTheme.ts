/**
 * 独立窗口主题初始化
 * 用于在独立窗口（启动器、剪贴板历史、悬浮窗等）中初始化主题
 * 支持跨窗口主题同步（通过 Tauri 事件系统）
 */
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export type ThemeMode = 'light' | 'dark' | 'nord' | 'auto'

// 主题变化事件名称（与 useTheme.ts 保持一致）
const THEME_CHANGE_EVENT = 'theme-changed'

// 保存事件监听器的取消函数
let themeChangeUnlisten: UnlistenFn | null = null

/**
 * 获取系统主题偏好
 */
function getSystemTheme(): 'light' | 'dark' {
  if (typeof window !== 'undefined' && window.matchMedia) {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }
  return 'dark'
}

/**
 * 解析主题模式
 */
function resolveTheme(mode: ThemeMode): 'light' | 'dark' | 'nord' {
  if (mode === 'auto') {
    return getSystemTheme()
  }
  return mode
}

/**
 * 应用主题到 document
 */
function applyTheme(theme: 'light' | 'dark' | 'nord') {
  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-theme', theme)
    console.log('[Theme] Independent window applied theme:', theme)
  }
}

/**
 * 监听主窗口的主题变化事件
 */
async function listenForThemeChanges() {
  // 避免重复监听
  if (themeChangeUnlisten) {
    return
  }

  try {
    themeChangeUnlisten = await listen<{ theme: 'light' | 'dark' | 'nord' }>(
      THEME_CHANGE_EVENT,
      (event) => {
        console.log('[Theme] Received theme change event:', event.payload)
        applyTheme(event.payload.theme)
        // 同时更新 localStorage 以便下次打开时使用最新主题
        try {
          const cached = localStorage.getItem('app-settings')
          if (cached) {
            const settings = JSON.parse(cached)
            // 根据收到的 resolved theme 更新，但不改变原始的 theme 设置
            // 这样 auto 模式仍然保持为 auto
            localStorage.setItem('app-settings', JSON.stringify(settings))
          }
        } catch (e) {
          // 忽略 localStorage 错误
        }
      }
    )
    console.log('[Theme] Started listening for theme changes')
  } catch (error) {
    console.error('[Theme] Failed to listen for theme changes:', error)
  }
}

/**
 * 从后端加载主题设置并应用
 */
export async function initThemeForWindow() {
  try {
    // 从后端获取设置
    const appSettings = await invoke<any>('get_app_settings')
    const themeMode = (appSettings?.theme as ThemeMode) || 'dark'
    const resolvedTheme = resolveTheme(themeMode)
    applyTheme(resolvedTheme)

    // 监听系统主题变化（当设置为 auto 时）
    if (typeof window !== 'undefined' && window.matchMedia) {
      window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', async () => {
        // 重新获取设置以检查是否为 auto
        try {
          const settings = await invoke<any>('get_app_settings')
          if ((settings?.theme as ThemeMode) === 'auto') {
            applyTheme(getSystemTheme())
          }
        } catch (e) {
          console.error('[Theme] Failed to reload settings:', e)
        }
      })
    }

    // 监听主窗口的主题变化事件（实时同步）
    await listenForThemeChanges()
  } catch (error) {
    console.error('[Theme] Failed to init theme for window:', error)
    // 默认使用深色主题
    applyTheme('dark')
    // 即使初始化失败，仍然监听主题变化事件
    await listenForThemeChanges()
  }
}

/**
 * 同步初始化主题（用于无法等待异步的场景）
 * 先尝试从 localStorage 读取，再异步从后端加载
 */
export function initThemeSync() {
  // 先尝试从 localStorage 读取缓存的主题
  try {
    const cached = localStorage.getItem('app-settings')
    if (cached) {
      const settings = JSON.parse(cached)
      const themeMode = (settings?.theme as ThemeMode) || 'dark'
      const resolvedTheme = resolveTheme(themeMode)
      applyTheme(resolvedTheme)
    } else {
      // 如果没有缓存，默认使用深色主题
      applyTheme('dark')
    }
  } catch (e) {
    // 忽略 localStorage 错误，使用默认深色主题
    applyTheme('dark')
  }

  // 然后异步从后端加载最新设置并开始监听主题变化
  initThemeForWindow()
}

/**
 * 清理主题监听器（用于窗口关闭时）
 */
export function cleanupThemeListener() {
  if (themeChangeUnlisten) {
    themeChangeUnlisten()
    themeChangeUnlisten = null
    console.log('[Theme] Cleaned up theme change listener')
  }
}
