/**
 * 主题系统入口
 *
 * 使用说明：
 * 1. 主窗口：使用 useTheme() composable 管理主题
 * 2. 独立窗口：在 main.ts 中调用 initThemeSync() 初始化
 * 3. 主题变化会自动通过 Tauri 事件系统同步到所有窗口
 *
 * 新增主题支持：
 * 1. 在 variables.css 中添加 [data-theme="new-theme"] 选择器
 * 2. 定义该主题下的所有 CSS 变量
 * 3. 在 useTheme.ts 中更新 ThemeMode 类型
 */
export { useTheme, THEME_CHANGE_EVENT } from './useTheme'
export type { ThemeMode } from './useTheme'
export { initThemeForWindow, initThemeSync, cleanupThemeListener } from './initTheme'

// 导入主题变量 - 必须在所有组件之前导入
import './variables.css'
