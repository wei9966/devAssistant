/**
 * 主题管理 composable
 * 不修改任何组件样式，只管理 data-theme 属性
 * 支持跨窗口主题同步（通过 Tauri 事件系统）
 */
import { computed, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useSettingsStore } from '@/stores/settingsStore'
import { emit } from '@tauri-apps/api/event'

export type ThemeMode = 'light' | 'dark' | 'auto'

// 主题变化事件名称（用于跨窗口同步）
export const THEME_CHANGE_EVENT = 'theme-changed'

export function useTheme() {
  const settingsStore = useSettingsStore()
  const { settings } = storeToRefs(settingsStore)

  // 获取系统主题偏好
  const getSystemTheme = (): 'light' | 'dark' => {
    if (typeof window !== 'undefined' && window.matchMedia) {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    }
    return 'dark'
  }

  // 解析主题模式
  const resolveTheme = (mode: ThemeMode): 'light' | 'dark' => {
    if (mode === 'auto') {
      return getSystemTheme()
    }
    return mode
  }

  // 当前实际主题（解析 auto）
  const currentTheme = computed((): 'light' | 'dark' => {
    const mode = settings.value.theme as ThemeMode || 'dark'
    return resolveTheme(mode)
  })

  // 设置主题
  const setTheme = (mode: ThemeMode) => {
    settingsStore.updateSettings({ theme: mode })
  }

  // 应用主题到 document
  const applyTheme = (theme: 'light' | 'dark') => {
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', theme)
      console.log('[Theme] Applied theme:', theme)
    }
  }

  // 广播主题变化到所有窗口
  const broadcastThemeChange = async (theme: 'light' | 'dark') => {
    try {
      await emit(THEME_CHANGE_EVENT, { theme })
      console.log('[Theme] Broadcasted theme change to all windows:', theme)
    } catch (error) {
      console.error('[Theme] Failed to broadcast theme change:', error)
    }
  }

  // 监听设置中的主题变化（使用 deep watch 确保检测到嵌套属性变化）
  watch(
    () => settings.value.theme,
    (newTheme, oldTheme) => {
      console.log('[Theme] Theme changed:', oldTheme, '->', newTheme)
      const resolved = resolveTheme(newTheme as ThemeMode || 'dark')
      console.log('[Theme] Resolved theme:', resolved)
      applyTheme(resolved)
      // 广播主题变化到其他窗口
      broadcastThemeChange(resolved)
    },
    { immediate: true }
  )

  // 监听系统主题变化（当设置为 auto 时）
  if (typeof window !== 'undefined' && window.matchMedia) {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if ((settings.value.theme as ThemeMode) === 'auto') {
        const systemTheme = getSystemTheme()
        applyTheme(systemTheme)
        broadcastThemeChange(systemTheme)
      }
    })
  }

  return {
    currentTheme,
    setTheme,
    applyTheme: () => applyTheme(currentTheme.value),
    broadcastThemeChange
  }
}
