<template>
  <div class="launcher-window" @keydown="handleKeyDown" tabindex="0" ref="containerRef">
    <!-- 背景遮罩层（点击关闭） -->
    <div class="launcher-backdrop" @click="closeLauncher"></div>

    <!-- 模态框主体 -->
    <div class="launcher-modal">
      <!-- 装饰背景网格 -->
      <div class="modal-grid-bg"></div>

      <!-- Header: 搜索栏 -->
      <div class="modal-header">
        <div class="search-container">
          <!-- 动态图标 -->
          <div class="search-icon-wrapper">
            <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="8"/>
              <path d="m21 21-4.35-4.35"/>
            </svg>
          </div>

          <!-- 输入框 -->
          <input
            ref="searchInputRef"
            type="text"
            v-model="searchQuery"
            placeholder="Search apps, commands, or ask AI..."
            class="search-input"
            @keydown.down.prevent="navigate('down')"
            @keydown.up.prevent="navigate('up')"
            @keydown.enter="handleLaunch"
          />

          <!-- 尾部提示 -->
          <div class="search-hint">
            <span>ESC</span>
          </div>
        </div>

        <!-- 快捷分类标签 -->
        <div class="tags-container">
          <button
            v-for="tag in categoryTags"
            :key="tag.id"
            :class="['tag-btn', { active: activeTag === tag.id }]"
            @click="activeTag = tag.id"
          >
            <span class="tag-icon">{{ tag.icon }}</span>
            <span>{{ tag.name }}</span>
          </button>
        </div>
      </div>

      <!-- Body: 应用网格 -->
      <div class="modal-body">
        <div class="apps-grid">
          <div
            v-for="(app, index) in displayApps"
            :key="app.id"
            :class="['app-item', { selected: selectedIndex === index }]"
            @click="handleLaunchApp(app)"
            @mouseenter="selectedIndex = index"
          >
            <div :class="['app-icon', getAppColorClass(app.category)]">
              <img v-if="app.icon" :src="app.icon" :alt="app.name" class="app-icon-img" />
              <span v-else class="app-icon-fallback">{{ getAppInitial(app.name) }}</span>
            </div>
            <span :class="['app-name', { 'app-name-selected': selectedIndex === index }]">
              {{ app.name }}
            </span>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-if="displayApps.length === 0 && searchQuery" class="empty-state">
          <span>No results found for "{{ searchQuery }}"</span>
        </div>

        <!-- 加载状态 -->
        <div v-if="loading" class="loading-state">
          <span>Loading...</span>
        </div>
      </div>

      <!-- Footer: 状态栏与提示 -->
      <div class="modal-footer">
        <div class="system-stats">
          <span class="stat-item">
            <svg class="stat-icon cpu" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="4" y="4" width="16" height="16" rx="2"/>
              <rect x="9" y="9" width="6" height="6"/>
              <path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 14h3M1 9h3M1 14h3"/>
            </svg>
            <span>CPU {{ cpuUsage }}%</span>
          </span>
          <span class="stat-item">
            <svg class="stat-icon mem" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="6" width="20" height="12" rx="2"/>
              <path d="M6 12h.01M10 12h.01M14 12h.01M18 12h.01"/>
            </svg>
            <span>MEM {{ formatMemory(memoryUsed) }}</span>
          </span>
        </div>

        <div class="keyboard-hints">
          <span class="hint"><kbd>TAB</kbd> 切换</span>
          <span class="hint"><kbd>↑↓</kbd> 选择</span>
          <span class="hint"><kbd>↵</kbd> 打开</span>
          <span class="hint"><kbd>ESC</kbd> 关闭</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

interface AppItem {
  id: string
  name: string
  path: string
  icon?: string
  category: string
  tags: string[]
  launchCount: number
  lastLaunchedAt?: number
  isPinned: boolean
  isHidden: boolean
}

interface Category {
  id: string
  name: string
  icon?: string
}

interface SystemInfo {
  cpu_usage: number
  memory_usage: number
  memory_total: number
  memory_used: number
}

// 状态
const containerRef = ref<HTMLDivElement>()
const searchInputRef = ref<HTMLInputElement>()
const searchQuery = ref('')
const selectedIndex = ref(0)
const activeTag = ref('all')
const loading = ref(true)
const apps = ref<AppItem[]>([])
const categories = ref<Category[]>([])
const cpuUsage = ref(0)
const memoryUsed = ref(0)

// 分类标签
const categoryTags = computed(() => {
  const defaultTags = [
    { id: 'all', name: 'All', icon: '📦' },
  ]

  const storeCategories = categories.value
    .filter(cat => cat.id !== 'all')
    .map(cat => ({
      id: cat.id,
      name: cat.name,
      icon: cat.icon || '📁'
    }))

  return [...defaultTags, ...storeCategories]
})

// 过滤后的应用列表
const displayApps = computed(() => {
  let filteredApps = apps.value.filter(app => !app.isHidden)

  // 分类过滤
  if (activeTag.value !== 'all') {
    filteredApps = filteredApps.filter(app => app.category === activeTag.value)
  }

  // 搜索过滤
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    filteredApps = filteredApps.filter(app =>
      app.name.toLowerCase().includes(query) ||
      app.tags.some(tag => tag.toLowerCase().includes(query))
    )
  }

  // 排序：置顶 > 使用频率
  filteredApps = [...filteredApps].sort((a, b) => {
    if (a.isPinned && !b.isPinned) return -1
    if (!a.isPinned && b.isPinned) return 1
    return b.launchCount - a.launchCount
  })

  return filteredApps.slice(0, 15)
})

// 关闭启动器
const closeLauncher = async () => {
  const window = getCurrentWindow()
  await window.hide()
}

// 键盘导航
const navigate = (direction: 'up' | 'down') => {
  if (displayApps.value.length === 0) return

  const cols = 5
  const total = displayApps.value.length

  if (direction === 'down') {
    if (selectedIndex.value + cols < total) {
      selectedIndex.value += cols
    } else if (selectedIndex.value < total - 1) {
      selectedIndex.value = total - 1
    }
  } else {
    if (selectedIndex.value - cols >= 0) {
      selectedIndex.value -= cols
    } else if (selectedIndex.value > 0) {
      selectedIndex.value = 0
    }
  }
}

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    e.preventDefault()
    closeLauncher()
  } else if (e.key === 'ArrowLeft') {
    e.preventDefault()
    if (selectedIndex.value > 0) {
      selectedIndex.value--
    }
  } else if (e.key === 'ArrowRight') {
    e.preventDefault()
    if (selectedIndex.value < displayApps.value.length - 1) {
      selectedIndex.value++
    }
  } else if (e.key === 'Tab') {
    e.preventDefault()
    const currentIdx = categoryTags.value.findIndex(t => t.id === activeTag.value)
    const nextIdx = (currentIdx + 1) % categoryTags.value.length
    activeTag.value = categoryTags.value[nextIdx].id
  }
}

const handleLaunch = () => {
  const app = displayApps.value[selectedIndex.value]
  if (app) {
    handleLaunchApp(app)
  }
}

const handleLaunchApp = async (app: AppItem) => {
  try {
    // 设置启动标志，防止窗口在启动过程中因失焦而关闭
    isLaunching.value = true
    await invoke('launch_app', { appId: app.id })
    closeLauncher()
  } catch (error) {
    console.error('启动应用失败:', error)
    isLaunching.value = false
  }
}

const getAppColorClass = (category: string): string => {
  const colorMap: Record<string, string> = {
    'dev': 'bg-blue',
    'office': 'bg-green',
    'browser': 'bg-yellow',
    'design': 'bg-purple',
    'media': 'bg-red',
    'game': 'bg-emerald',
    'other': 'bg-slate'
  }
  return colorMap[category] || 'bg-violet'
}

const getAppInitial = (name: string): string => {
  return name.charAt(0).toUpperCase()
}

const formatMemory = (bytes: number): string => {
  const gb = bytes / (1024 * 1024 * 1024)
  return gb.toFixed(1) + 'GB'
}

// 加载数据
const loadData = async () => {
  loading.value = true
  try {
    const [appsData, categoriesData, systemInfo] = await Promise.all([
      invoke<AppItem[]>('get_all_apps'),
      invoke<Category[]>('get_categories'),
      invoke<SystemInfo>('get_system_info')
    ])

    apps.value = appsData
    categories.value = categoriesData
    cpuUsage.value = Math.round(systemInfo.cpu_usage)
    memoryUsed.value = systemInfo.memory_used
  } catch (error) {
    console.error('加载数据失败:', error)
  } finally {
    loading.value = false
  }
}

// 监听搜索变化
watch(searchQuery, () => {
  selectedIndex.value = 0
})

watch(activeTag, () => {
  selectedIndex.value = 0
})

// 是否正在启动应用（防止失焦时关闭窗口）
const isLaunching = ref(false)

// 监听窗口显示事件，重新加载数据
onMounted(async () => {
  await loadData()

  // 聚焦搜索框
  nextTick(() => {
    searchInputRef.value?.focus()
    containerRef.value?.focus()
  })

  // 监听窗口显示/隐藏
  const window = getCurrentWindow()
  window.onFocusChanged(({ payload: focused }) => {
    if (focused) {
      // 窗口获得焦点时重新加载数据并聚焦
      loadData()
      searchQuery.value = ''
      selectedIndex.value = 0
      activeTag.value = 'all'
      isLaunching.value = false
      nextTick(() => {
        searchInputRef.value?.focus()
      })
    } else {
      // 窗口失去焦点时自动隐藏（点击外部时触发）
      // 但如果正在启动应用，则不隐藏（等待启动完成后再隐藏）
      if (!isLaunching.value) {
        closeLauncher()
      }
    }
  })
})
</script>

<style scoped>
/* 启动器窗口容器 */
.launcher-window {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  outline: none;
}

/* 背景遮罩 - 完全透明，仅用于点击关闭（已改用失焦隐藏） */
.launcher-backdrop {
  display: none;
}

/* 模态框主体 */
.launcher-modal {
  width: 100%;
  height: 100%;
  background: rgba(12, 12, 20, 0.98);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow:
    0 0 0 1px rgba(139, 92, 246, 0.3),
    0 25px 50px -12px rgba(0, 0, 0, 0.5);
}

/* 装饰背景网格 */
.modal-grid-bg {
  position: absolute;
  inset: 0;
  opacity: 0.15;
  pointer-events: none;
  background-image: radial-gradient(#4c1d95 1px, transparent 1px);
  background-size: 30px 30px;
}

/* Header */
.modal-header {
  padding: 24px 24px 8px;
  position: relative;
  z-index: 10;
}

.search-container {
  display: flex;
  align-items: center;
  gap: 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  padding-bottom: 16px;
}

.search-icon-wrapper {
  flex-shrink: 0;
}

.search-icon {
  width: 28px;
  height: 28px;
  color: #8b5cf6;
  animation: pulse-glow 2s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%, 100% { opacity: 1; filter: drop-shadow(0 0 4px rgba(139, 92, 246, 0.5)); }
  50% { opacity: 0.7; filter: drop-shadow(0 0 8px rgba(139, 92, 246, 0.8)); }
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 24px;
  font-weight: 300;
  color: #fff;
  letter-spacing: 0.025em;
  height: 48px;
}

.search-input::placeholder {
  color: #475569;
}

.search-hint {
  display: flex;
  gap: 4px;
  font-size: 11px;
  font-family: 'Courier New', monospace;
  color: #64748b;
  border: 1px solid rgba(255, 255, 255, 0.1);
  padding: 4px 8px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.2);
}

/* 分类标签 */
.tags-container {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 16px;
  padding-bottom: 8px;
}

.tag-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 10px;
  border-radius: 9999px;
  font-size: 12px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid transparent;
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  flex-shrink: 0;
}

.tag-btn:hover {
  color: #94a3b8;
  background: rgba(255, 255, 255, 0.08);
}

.tag-btn.active {
  background: rgba(139, 92, 246, 0.2);
  border-color: #8b5cf6;
  color: #c4b5fd;
}

.tag-icon {
  font-size: 14px;
}

/* Body */
.modal-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 16px 24px;
  z-index: 10;
}

.modal-body::-webkit-scrollbar {
  width: 0px;
  background: transparent;
}

.apps-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 12px;
}

.app-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  padding: 12px 8px;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid transparent;
  min-width: 0;
  width: 100%;
  box-sizing: border-box;
}

.app-item:hover {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.1);
}

.app-item.selected {
  background: rgba(139, 92, 246, 0.15);
  border-color: rgba(139, 92, 246, 0.5);
  box-shadow: 0 0 15px rgba(139, 92, 246, 0.2);
}

.app-icon {
  width: 48px;
  height: 48px;
  min-width: 48px;
  min-height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.app-item:hover .app-icon {
  transform: scale(1.1);
}

.app-icon-img {
  width: 32px;
  height: 32px;
  object-fit: contain;
}

.app-icon-fallback {
  font-size: 20px;
  font-weight: 600;
  color: #fff;
}

/* 应用图标背景色 */
.bg-blue { background: linear-gradient(135deg, #2563eb 0%, #3b82f6 100%); }
.bg-green { background: linear-gradient(135deg, #059669 0%, #10b981 100%); }
.bg-yellow { background: linear-gradient(135deg, #d97706 0%, #f59e0b 100%); }
.bg-purple { background: linear-gradient(135deg, #7c3aed 0%, #8b5cf6 100%); }
.bg-red { background: linear-gradient(135deg, #dc2626 0%, #ef4444 100%); }
.bg-emerald { background: linear-gradient(135deg, #059669 0%, #34d399 100%); }
.bg-slate { background: linear-gradient(135deg, #475569 0%, #64748b 100%); }
.bg-violet { background: linear-gradient(135deg, #7c3aed 0%, #a78bfa 100%); }

.app-name {
  font-size: 11px;
  text-align: center;
  color: #94a3b8;
  font-weight: 500;
  width: 100%;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: color 0.2s ease;
  line-height: 1.3;
}

.app-item:hover .app-name {
  color: #e2e8f0;
}

.app-name-selected {
  color: #fff;
}

/* 空状态和加载状态 */
.empty-state,
.loading-state {
  grid-column: 1 / -1;
  text-align: center;
  color: #475569;
  padding: 60px 20px;
  font-size: 14px;
}

/* Footer */
.modal-footer {
  height: 40px;
  background: rgba(0, 0, 0, 0.4);
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  z-index: 10;
}

.system-stats {
  display: flex;
  gap: 16px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  font-family: 'Courier New', monospace;
  color: #64748b;
}

.stat-icon {
  width: 12px;
  height: 12px;
}

.stat-icon.cpu {
  color: #8b5cf6;
}

.stat-icon.mem {
  color: #3b82f6;
}

.keyboard-hints {
  display: flex;
  gap: 16px;
}

.hint {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  font-family: 'Courier New', monospace;
  color: #64748b;
}

.hint kbd {
  display: inline-block;
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  font-size: 9px;
}
</style>
