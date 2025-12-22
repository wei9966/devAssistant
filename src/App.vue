<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides">
    <n-loading-bar-provider>
      <n-message-provider>
        <n-notification-provider>
          <n-dialog-provider>
            <div id="app">
              <div class="app-container">
                <!-- 侧边导航栏 -->
                <div class="sidebar">
                  <!-- 品牌Logo区域 -->
                  <div class="brand-section">
                    <div class="logo-icon">
                      <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon">
                        <rect x="3" y="3" width="7" height="7" rx="1" stroke="currentColor" stroke-width="2"/>
                        <rect x="14" y="3" width="7" height="7" rx="1" stroke="currentColor" stroke-width="2"/>
                        <rect x="3" y="14" width="7" height="7" rx="1" stroke="currentColor" stroke-width="2"/>
                        <rect x="14" y="14" width="7" height="7" rx="1" stroke="currentColor" stroke-width="2"/>
                      </svg>
                    </div>
                    <div class="brand-text">
                      <h1 class="brand-title">DevAssistant</h1>
                      <p class="brand-subtitle">PERSONAL HUB</p>
                    </div>
                  </div>

                  <!-- 导航菜单 -->
                  <nav class="nav-menu">
                    <n-menu
                      v-model:value="activeKey"
                      :options="menuOptions"
                      @update:value="handleMenuSelect"
                      :indent="16"
                    />
                  </nav>

                  <!-- 底部信息区 -->
                  <div class="sidebar-footer">
                    <div class="info-card">
                      <!-- CPU Usage -->
                      <div class="info-header">
                        <span class="info-label">CPU Usage</span>
                        <span class="info-value">{{ cpuUsage }}%</span>
                      </div>
                      <div class="progress-bar">
                        <div class="progress-fill" :style="{ width: cpuUsage + '%' }"></div>
                      </div>

                      <!-- Memory Usage -->
                      <div class="info-header" style="margin-top: 12px;">
                        <span class="info-label">Memory</span>
                        <span class="info-value">{{ memoryUsage }}%</span>
                      </div>
                      <div class="progress-bar">
                        <div class="progress-fill-memory" :style="{ width: memoryUsage + '%' }"></div>
                      </div>
                      <div class="memory-details">
                        <span class="memory-text">{{ formatMemory(memoryUsed) }} / {{ formatMemory(memoryTotal) }}</span>
                      </div>

                      <div class="shortcut-hint">
                        <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" class="shortcut-icon">
                          <rect x="3" y="3" width="18" height="18" rx="2" stroke="currentColor" stroke-width="2"/>
                          <path d="M7 7L17 17M17 7L7 17" stroke="currentColor" stroke-width="2"/>
                        </svg>
                        <span>Ctrl + K to search</span>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- 主内容区 -->
                <div class="main-content">
                  <!-- Tauri窗口拖拽区域 -->
                  <div data-tauri-drag-region class="drag-region">
                    <NotificationBell />
                  </div>

                  <div class="content-wrapper">
                    <router-view v-slot="{ Component, route }">
                      <transition name="fade" mode="out-in">
                        <component :is="Component" :key="route.path" />
                      </transition>
                    </router-view>
                  </div>
                </div>
              </div>
            </div>
            <!-- 赛博朋克全屏启动器 -->
            <CyberpunkLauncher
              v-model="showCyberpunkLauncher"
              @launch="handleLauncherLaunch"
            />

            <!-- 快速任务创建模态框 -->
            <QuickTaskModal
              v-model:show="showQuickTaskModal"
              @created="handleTaskCreated"
            />

            <!-- 赛博朋克SQL查询模态框 -->
            <CyberpunkSqlModal
              v-model="showCyberpunkSql"
              @edit="handleSqlEdit"
            />

            <!-- 更新提示弹框 -->
            <UpdateDialog
              v-model:show="showUpdateDialog"
              :version="updateInfo.version || ''"
              :notes="updateInfo.notes || ''"
              :date="updateInfo.date || ''"
              :downloading="isUpdating"
              :progress="updateProgress"
              @update="handleUpdate"
              @later="handleUpdateLater"
              @skip="handleUpdateSkip"
            />
          </n-dialog-provider>
        </n-notification-provider>
      </n-message-provider>
    </n-loading-bar-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref, shallowRef, h, onMounted, onUnmounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import type { MenuOption } from 'naive-ui'
import {
  NConfigProvider,
  NLoadingBarProvider,
  NMessageProvider,
  NNotificationProvider,
  NDialogProvider,
  NMenu,
  darkTheme,
  type GlobalTheme
} from 'naive-ui'
import {
  CheckboxOutline as TaskIcon,
  DocumentTextOutline as SqlIcon,
  BookOutline as LogIcon,
  RocketOutline as LauncherIcon,
  NotificationsOutline as NotificationIcon,
  SettingsOutline as SettingsIcon,
  PieChartOutline as ReportIcon,
  TimerOutline as TimerIcon,
  ConstructOutline as ToolboxIcon
} from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import CyberpunkLauncher from '@/components/appLauncher/CyberpunkLauncher.vue'
import QuickTaskModal from '@/components/QuickTaskModal.vue'
import CyberpunkSqlModal from '@/components/sql/CyberpunkSqlModal.vue'
import NotificationBell from '@/components/notification/NotificationBell.vue'
import UpdateDialog from '@/components/UpdateDialog.vue'
import {
  checkForUpdate,
  setSkippedVersion,
  setLastCheckTime,
  shouldCheckUpdate,
  type UpdateInfo
} from '@/services/updater'
import { relaunch } from '@tauri-apps/plugin-process'
// 主题系统
import { useTheme } from '@/themes'
import { useSettingsStore } from '@/stores/settingsStore'

// 初始化主题系统
const { currentTheme } = useTheme()

// 获取全局设置 store
const settingsStore = useSettingsStore()

// 从后端加载主题设置并同步到 store
const loadThemeFromBackend = async () => {
  try {
    const appSettings = await invoke<any>('get_app_settings')
    if (appSettings && appSettings.theme) {
      settingsStore.updateSettings({ theme: appSettings.theme as 'light' | 'dark' | 'auto' })
    }
  } catch (error) {
    console.error('加载主题设置失败:', error)
  }
}

interface SystemInfo {
  cpu_usage: number
  memory_usage: number
  memory_total: number
  memory_used: number
}

const router = useRouter()
const activeKey = ref<string>('task-board')
const cpuUsage = ref<number>(0)
const memoryUsage = ref<number>(0)
const memoryTotal = ref<number>(0)
const memoryUsed = ref<number>(0)

// 赛博朋克启动器状态
const showCyberpunkLauncher = ref(false)

// 快速任务模态框状态
const showQuickTaskModal = ref(false)

// 赛博朋克SQL模态框状态
const showCyberpunkSql = ref(false)

// 更新弹框状态
const showUpdateDialog = ref(false)
const updateInfo = ref<UpdateInfo>({ available: false })
const isUpdating = ref(false)
const updateProgress = ref(0)
const currentUpdate = shallowRef<any>(null)

// 打开赛博朋克启动器
const openCyberpunkLauncher = () => {
  showCyberpunkLauncher.value = true
}

// 打开快速任务模态框
const openQuickTaskModal = () => {
  showQuickTaskModal.value = true
}

// 打开赛博朋克SQL模态框
const openCyberpunkSql = () => {
  showCyberpunkSql.value = true
}

// 处理启动器启动应用事件
const handleLauncherLaunch = () => {
  // 应用启动后的回调，可以添加通知等
}

// 处理任务创建完成事件
const handleTaskCreated = (taskId: number) => {
  console.log('任务已创建:', taskId)
  // 可以在这里添加刷新任务列表等逻辑
}

// 处理SQL编辑事件
const handleSqlEdit = (sql: any) => {
  console.log('编辑SQL:', sql)
  // 跳转到SQL历史页面进行编辑
  router.push({ name: 'sql-history' })
}

// 检查应用更新
const checkAppUpdate = async () => {
  try {
    // 记录自动更新检查开始
    await invoke('log_update_info', { message: '[自动更新] 应用启动，开始检查更新...' })

    // 检查是否应该检查更新（每24小时一次）
    if (!shouldCheckUpdate()) {
      await invoke('log_update_info', { message: '[自动更新] 24小时内已检查过，跳过本次检查' })
      return
    }

    await invoke('log_update_info', { message: '[自动更新] 开始执行更新检查...' })
    const info = await checkForUpdate()
    setLastCheckTime()

    if (info.available && info.version && info.update) {
      updateInfo.value = info
      // 直接使用 checkForUpdate 返回的 Update 对象
      currentUpdate.value = info.update
      showUpdateDialog.value = true
    }
  } catch (error) {
    console.error('检查更新失败:', error)
  }
}

// 处理立即更新
const handleUpdate = async () => {
  console.log('[更新] 点击立即更新按钮')
  console.log('[更新] currentUpdate:', currentUpdate.value)
  console.log('[更新] isUpdating:', isUpdating.value)

  if (!currentUpdate.value) {
    console.error('[更新] currentUpdate 为空，无法执行更新')
    return
  }

  if (isUpdating.value) {
    console.log('[更新] 已在更新中，跳过')
    return
  }

  isUpdating.value = true
  updateProgress.value = 0

  try {
    console.log('[更新] 开始下载并安装更新...')
    let downloaded = 0
    let contentLength = 0

    await currentUpdate.value.downloadAndInstall((event: any) => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength || 0
          break
        case 'Progress':
          downloaded += event.data.chunkLength
          if (contentLength > 0) {
            updateProgress.value = (downloaded / contentLength) * 100
          }
          break
        case 'Finished':
          updateProgress.value = 100
          break
      }
    })

    // 下载完成，重启应用
    console.log('更新下载完成，准备重启...')
    await relaunch()
  } catch (error) {
    console.error('更新失败:', error)
    isUpdating.value = false
    updateProgress.value = 0
  }
}

// 处理稍后提醒
const handleUpdateLater = () => {
  showUpdateDialog.value = false
  isUpdating.value = false
  updateProgress.value = 0
}

// 处理跳过版本
const handleUpdateSkip = () => {
  if (updateInfo.value.version) {
    setSkippedVersion(updateInfo.value.version)
    console.log('已跳过版本:', updateInfo.value.version)
  }
  showUpdateDialog.value = false
  isUpdating.value = false
  updateProgress.value = 0
}

// 主题配置 - 根据当前主题动态切换
// dark 和 nord 都是深色系主题，需要使用 Naive UI 的 darkTheme
const theme = computed<GlobalTheme | null>(() => {
  return (currentTheme.value === 'dark' || currentTheme.value === 'nord') ? darkTheme : null
})

// Naive UI 主题覆盖配置 - 根据当前主题动态调整
const themeOverrides = computed(() => {
  // dark 和 nord 都是深色系主题
  const isDark = currentTheme.value === 'dark' || currentTheme.value === 'nord'
  const isNord = currentTheme.value === 'nord'

  // Nord 主题使用 Frost 冰蓝色系，默认使用 indigo 紫色系
  const primaryColor = isNord ? '#88c0d0' : '#6366f1'
  const primaryColorHover = isNord ? '#8fbcbb' : '#818cf8'
  const primaryColorPressed = isNord ? '#81a1c1' : '#4f46e5'

  return {
    common: {
      primaryColor,
      primaryColorHover,
      primaryColorPressed,
      primaryColorSuppl: primaryColorHover,
      // 浅色主题需要调整背景色
      ...(isDark ? {} : {
        bodyColor: '#f8fafc',
        cardColor: '#ffffff',
        modalColor: '#ffffff',
        popoverColor: '#ffffff',
        tableColor: '#ffffff',
        inputColor: '#ffffff',
      })
    },
    Button: {
      colorPrimary: primaryColor,
      colorHoverPrimary: primaryColorHover,
      colorPressedPrimary: primaryColorPressed,
      borderPrimary: `1px solid ${primaryColor}`,
      textColorPrimary: '#ffffff',
    },
    Switch: {
      railColorActive: primaryColor,
    },
    Select: {
      peers: {
        InternalSelection: {
          colorActive: primaryColor
        }
      }
    },
    Menu: isDark ? {} : {
      color: 'transparent',
      itemColorHover: 'rgba(226, 232, 240, 0.6)',
      itemColorActive: `rgba(${isNord ? '136, 192, 208' : '99, 102, 241'}, 0.1)`,
      itemColorActiveHover: `rgba(${isNord ? '136, 192, 208' : '99, 102, 241'}, 0.15)`,
      itemTextColor: '#334155',
      itemTextColorHover: '#0f172a',
      itemTextColorActive: primaryColor,
      itemIconColor: '#64748b',
      itemIconColorHover: '#0f172a',
      itemIconColorActive: primaryColor,
    }
  }
})

// 菜单选项
const menuOptions: MenuOption[] = [
  {
    label: '任务看板',
    key: 'task-board',
    icon: () => h(TaskIcon)
  },
  {
    label: '专注时钟',
    key: 'pomodoro',
    icon: () => h(TimerIcon)
  },
  {
    label: 'SQL历史',
    key: 'sql-history',
    icon: () => h(SqlIcon)
  },
  {
    label: '工作日志',
    key: 'work-log',
    icon: () => h(LogIcon)
  },
  {
    label: '应用启动器',
    key: 'app-launcher',
    icon: () => h(LauncherIcon)
  },
  {
    label: '报表中心',
    key: 'report-center',
    icon: () => h(ReportIcon)
  },
  {
    label: '通知中心',
    key: 'notification-center',
    icon: () => h(NotificationIcon)
  },
  {
    label: '工具箱',
    key: 'toolbox',
    icon: () => h(ToolboxIcon)
  },
  {
    label: '设置',
    key: 'settings',
    icon: () => h(SettingsIcon)
  }
]

// 菜单选择处理
const handleMenuSelect = (key: string) => {
  router.push({ name: key })
}

// 初始化：根据当前路由设置激活的菜单项
router.afterEach((to) => {
  if (to.name) {
    activeKey.value = to.name as string
  }
})

// 获取系统信息
const getSystemInfo = async () => {
  try {
    const info = await invoke<SystemInfo>('get_system_info')
    cpuUsage.value = Math.round(info.cpu_usage)
    memoryUsage.value = Math.round(info.memory_usage)
    memoryTotal.value = info.memory_total
    memoryUsed.value = info.memory_used
  } catch (error) {
    console.error('获取系统信息失败:', error)
  }
}

// 格式化内存大小
const formatMemory = (bytes: number): string => {
  const gb = bytes / (1024 * 1024 * 1024)
  return gb.toFixed(1) + ' GB'
}

// 定时获取系统信息
let systemInterval: number | null = null

// 监听全局快捷键事件
const handleOpenLauncher = () => {
  openCyberpunkLauncher()
}

// 监听打开快速任务模态框事件
const handleOpenQuickTask = () => {
  openQuickTaskModal()
}

// 监听打开SQL模态框事件
const handleOpenSqlModal = () => {
  openCyberpunkSql()
}

onMounted(() => {
  // 从后端加载主题设置
  loadThemeFromBackend()

  getSystemInfo() // 立即获取一次
  systemInterval = window.setInterval(() => {
    getSystemInfo()
  }, 3000)

  // 监听打开启动器的自定义事件
  window.addEventListener('open-cyberpunk-launcher', handleOpenLauncher)
  // 监听打开快速任务模态框的自定义事件
  window.addEventListener('open-quick-task-modal', handleOpenQuickTask)
  // 监听打开SQL模态框的自定义事件
  window.addEventListener('open-cyberpunk-sql', handleOpenSqlModal)

  // 启动时检查更新（延迟3秒，等待应用初始化完成）
  setTimeout(() => {
    checkAppUpdate()
  }, 3000)
})

onUnmounted(() => {
  if (systemInterval) {
    clearInterval(systemInterval)
  }
  // 移除事件监听
  window.removeEventListener('open-cyberpunk-launcher', handleOpenLauncher)
  window.removeEventListener('open-quick-task-modal', handleOpenQuickTask)
  window.removeEventListener('open-cyberpunk-sql', handleOpenSqlModal)
})
</script>

<style scoped>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

#app {
  width: 100%;
  height: 100vh;
  overflow: hidden;
  background: var(--bg-base);
}

.app-container {
  display: flex;
  width: 100%;
  height: 100%;
}

/* ==================== 侧边栏样式 ==================== */
.sidebar {
  width: 256px;
  height: 100%;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--sidebar-border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  padding: 16px;
}

/* 品牌Logo区域 */
.brand-section {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  margin-bottom: 24px;
}

.logo-icon {
  width: 32px;
  height: 32px;
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%); /* indigo-500 to purple-500 */
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 8px 20px rgba(99, 102, 241, 0.2);
  flex-shrink: 0;
}

.logo-icon .icon {
  width: 18px;
  height: 18px;
  color: #ffffff;
}

.brand-text {
  flex: 1;
  min-width: 0;
}

.brand-title {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.025em;
  line-height: 1.2;
  margin: 0;
}

.brand-subtitle {
  font-size: 10px;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-weight: 600;
  margin: 0;
  margin-top: 2px;
}

/* 导航菜单 */
.nav-menu {
  flex: 1;
  overflow-y: auto;
  margin-bottom: 16px;
}

/* 自定义Naive UI Menu样式 */
.nav-menu :deep(.n-menu) {
  background-color: transparent;
  color: var(--text-secondary);
}

.nav-menu :deep(.n-menu-item) {
  margin-bottom: 4px;
  border-radius: 12px;
  padding: 12px 16px;
  transition: all 0.2s ease;
  position: relative;
}

.nav-menu :deep(.n-menu-item:not(.n-menu-item--selected):hover) {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.nav-menu :deep(.n-menu-item.n-menu-item--selected) {
  background-color: var(--accent-glow);
  color: var(--accent-secondary);
  box-shadow: 0 0 20px var(--accent-glow);
}

.nav-menu :deep(.n-menu-item.n-menu-item--selected::after) {
  content: '';
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: var(--accent-secondary);
  box-shadow: 0 0 8px var(--accent-glow);
}

.nav-menu :deep(.n-menu-item-content__icon) {
  color: var(--text-muted);
  transition: color 0.2s ease;
}

.nav-menu :deep(.n-menu-item:hover .n-menu-item-content__icon) {
  color: var(--text-primary);
}

.nav-menu :deep(.n-menu-item.n-menu-item--selected .n-menu-item-content__icon) {
  color: var(--accent-secondary);
}

.nav-menu :deep(.n-menu-item-content) {
  padding-left: 0 !important;
}

.nav-menu :deep(.n-menu-item-content-header) {
  font-size: 14px;
  font-weight: 500;
  letter-spacing: 0.025em;
}

/* 底部信息区 */
.sidebar-footer {
  padding: 16px;
  margin-top: auto;
}

.info-card {
  background-color: var(--card-bg);
  border-radius: 12px;
  padding: 12px;
  border: 1px solid var(--card-border);
}

.info-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.info-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
}

.info-value {
  font-size: 12px;
  color: var(--accent-primary);
  font-family: 'Courier New', monospace;
  font-weight: 600;
}

.progress-bar {
  width: 100%;
  height: 6px;
  background-color: var(--progress-bg);
  border-radius: 9999px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #6366f1 0%, #8b5cf6 100%); /* indigo-500 to purple-500 */
  border-radius: 9999px;
  transition: width 0.5s ease;
}

.progress-fill-memory {
  height: 100%;
  background: linear-gradient(90deg, #10b981 0%, #34d399 100%); /* green-500 to green-400 */
  border-radius: 9999px;
  transition: width 0.5s ease;
}

.memory-details {
  margin-top: 6px;
  display: flex;
  justify-content: center;
}

.memory-text {
  font-size: 10px;
  color: var(--text-dim);
  font-family: 'Courier New', monospace;
  font-weight: 500;
}

.shortcut-hint {
  margin-top: 12px;
  font-size: 10px;
  color: var(--text-dim);
  display: flex;
  align-items: center;
  gap: 4px;
}

.shortcut-icon {
  width: 10px;
  height: 10px;
  color: var(--text-dim);
}

/* ==================== 主内容区样式 ==================== */
.main-content {
  flex: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-surface);
  position: relative;
  min-width: 0;
}

/* Tauri拖拽区域 */
.drag-region {
  height: 32px;
  width: 100%;
  flex-shrink: 0;
  -webkit-app-region: drag;
  app-region: drag;
  display: flex;
  justify-content: flex-end;
  align-items: center;
  padding-right: 16px;
}

.drag-region > * {
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.content-wrapper {
  flex: 1;
  padding: 32px;
  padding-top: 8px;
  overflow: auto;
}

/* 自定义滚动条 */
.content-wrapper::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.content-wrapper::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
}

.content-wrapper::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 4px;
}

.content-wrapper::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
}

/* ==================== 路由切换动画 ==================== */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

/* ==================== 响应式设计 ==================== */
@media (max-width: 768px) {
  .sidebar {
    width: 80px;
  }

  .brand-text {
    display: none;
  }

  .sidebar-footer {
    display: none;
  }
}
</style>
