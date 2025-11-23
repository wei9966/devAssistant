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
                      <div class="info-header">
                        <span class="info-label">CPU Usage</span>
                        <span class="info-value">{{ cpuUsage }}%</span>
                      </div>
                      <div class="progress-bar">
                        <div class="progress-fill" :style="{ width: cpuUsage + '%' }"></div>
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
                  <div data-tauri-drag-region class="drag-region"></div>

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
          </n-dialog-provider>
        </n-notification-provider>
      </n-message-provider>
    </n-loading-bar-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref, h, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import type { MenuOption } from 'naive-ui'
import {
  NConfigProvider,
  NLoadingBarProvider,
  NMessageProvider,
  NNotificationProvider,
  NDialogProvider,
  NMenu,
  darkTheme
} from 'naive-ui'
import {
  CheckboxOutline as TaskIcon,
  DocumentTextOutline as SqlIcon,
  BookOutline as LogIcon,
  SettingsOutline as SettingsIcon
} from '@vicons/ionicons5'

const router = useRouter()
const activeKey = ref<string>('task-board')
const cpuUsage = ref<number>(12)

// 主题配置 - 使用深色主题
const theme = darkTheme

// Naive UI 主题覆盖配置 - 统一使用 indigo 主色调
const themeOverrides = {
  common: {
    primaryColor: '#6366f1',        // indigo-600
    primaryColorHover: '#818cf8',   // indigo-500
    primaryColorPressed: '#4f46e5', // indigo-700
    primaryColorSuppl: '#818cf8',   // indigo-400
  },
  Button: {
    colorPrimary: '#6366f1',
    colorHoverPrimary: '#818cf8',
    colorPressedPrimary: '#4f46e5',
    borderPrimary: '1px solid #6366f1',
    textColorPrimary: '#ffffff',
  },
  Switch: {
    railColorActive: '#6366f1',
  },
  Select: {
    peers: {
      InternalSelection: {
        colorActive: '#6366f1'
      }
    }
  }
}

// 菜单选项
const menuOptions: MenuOption[] = [
  {
    label: '任务看板',
    key: 'task-board',
    icon: () => h(TaskIcon)
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

// CPU使用率模拟（可以后续接入真实系统监控）
let cpuInterval: number | null = null
onMounted(() => {
  cpuInterval = window.setInterval(() => {
    cpuUsage.value = Math.floor(Math.random() * 30) + 5 // 5-35%范围
  }, 3000)
})

onUnmounted(() => {
  if (cpuInterval) {
    clearInterval(cpuInterval)
  }
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
  background: #020617; /* slate-950 */
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
  background-color: #020617; /* slate-950 */
  border-right: 1px solid rgba(148, 163, 184, 0.1); /* slate-800/60 */
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
  color: #f1f5f9; /* slate-100 */
  letter-spacing: -0.025em;
  line-height: 1.2;
  margin: 0;
}

.brand-subtitle {
  font-size: 10px;
  color: #64748b; /* slate-500 */
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
  color: #cbd5e1; /* slate-300 */
}

.nav-menu :deep(.n-menu-item) {
  margin-bottom: 4px;
  border-radius: 12px;
  padding: 12px 16px;
  transition: all 0.2s ease;
  position: relative;
}

.nav-menu :deep(.n-menu-item:not(.n-menu-item--selected):hover) {
  background-color: rgba(30, 41, 59, 0.5); /* slate-800/50 */
  color: #e2e8f0; /* slate-200 */
}

.nav-menu :deep(.n-menu-item.n-menu-item--selected) {
  background-color: rgba(99, 102, 241, 0.1); /* indigo-500/10 */
  color: #a78bfa; /* indigo-400 */
  box-shadow: 0 0 20px rgba(99, 102, 241, 0.1);
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
  background-color: #a78bfa; /* indigo-400 */
  box-shadow: 0 0 8px rgba(99, 102, 241, 0.6);
}

.nav-menu :deep(.n-menu-item-content__icon) {
  color: #94a3b8; /* slate-400 */
  transition: color 0.2s ease;
}

.nav-menu :deep(.n-menu-item:hover .n-menu-item-content__icon) {
  color: #e2e8f0; /* slate-200 */
}

.nav-menu :deep(.n-menu-item.n-menu-item--selected .n-menu-item-content__icon) {
  color: #a78bfa; /* indigo-400 */
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
  background-color: rgba(15, 23, 42, 0.5); /* slate-900/50 */
  border-radius: 12px;
  padding: 12px;
  border: 1px solid rgba(148, 163, 184, 0.08); /* slate-800/50 */
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
  color: #94a3b8; /* slate-400 */
}

.info-value {
  font-size: 12px;
  color: #818cf8; /* indigo-400 */
  font-family: 'Courier New', monospace;
  font-weight: 600;
}

.progress-bar {
  width: 100%;
  height: 6px;
  background-color: #1e293b; /* slate-800 */
  border-radius: 9999px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #6366f1 0%, #8b5cf6 100%); /* indigo-500 to purple-500 */
  border-radius: 9999px;
  transition: width 0.5s ease;
}

.shortcut-hint {
  margin-top: 12px;
  font-size: 10px;
  color: #475569; /* slate-600 */
  display: flex;
  align-items: center;
  gap: 4px;
}

.shortcut-icon {
  width: 10px;
  height: 10px;
  color: #475569; /* slate-600 */
}

/* ==================== 主内容区样式 ==================== */
.main-content {
  flex: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #020617 0%, #0f172a 100%); /* slate-950 to slate-900 */
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
  background: transparent;
}

.content-wrapper::-webkit-scrollbar-thumb {
  background: rgba(148, 163, 184, 0.2);
  border-radius: 4px;
}

.content-wrapper::-webkit-scrollbar-thumb:hover {
  background: rgba(148, 163, 184, 0.3);
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
