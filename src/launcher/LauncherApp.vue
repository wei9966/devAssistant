<template>
  <div class="launcher-window" @keydown="handleKeyDown" @contextmenu.prevent tabindex="0" ref="containerRef">
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
            placeholder="搜索应用（支持拼音）..."
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

      <!-- Body: 三栏布局 -->
      <div class="modal-body">
        <!-- 顶部区域：固定项目 + 最近使用 -->
        <div class="top-section" v-if="!searchQuery.trim()">
          <!-- 固定项目区 -->
          <div class="section-block" v-if="pinnedApps.length > 0">
            <div class="section-header">
              <span class="section-icon">📌</span>
              <span class="section-title">已固定</span>
              <span class="section-count">{{ pinnedApps.length }}</span>
            </div>
            <div class="mini-apps-grid">
              <div
                v-for="(app, index) in pinnedApps"
                :key="app.id"
                :class="['app-item-mini', { selected: currentSection === 'pinned' && selectedIndex === index }]"
                @click="handleLaunchApp(app)"
                @contextmenu.prevent.stop="showContextMenu($event, app)"
                @mouseenter="handleHover('pinned', index)"
              >
                <div :class="['app-icon-mini', getAppColorClass(app.category)]">
                  <img v-if="app.icon && app.type !== 'tool'" :src="app.icon" :alt="app.name" class="app-icon-img-mini" />
                  <span v-else class="app-icon-fallback-mini">{{ app.icon || getAppInitial(app.name) }}</span>
                </div>
                <span class="app-name-mini">{{ app.name }}</span>
              </div>
            </div>
          </div>

          <!-- 最近使用区 -->
          <div class="section-block" v-if="recentApps.length > 0">
            <div class="section-header">
              <span class="section-icon">🕐</span>
              <span class="section-title">最近使用</span>
              <span class="section-count">{{ recentApps.length }}</span>
            </div>
            <div class="mini-apps-grid">
              <div
                v-for="(app, index) in recentApps"
                :key="app.id"
                :class="['app-item-mini', { selected: currentSection === 'recent' && selectedIndex === index }]"
                @click="handleLaunchApp(app)"
                @contextmenu.prevent.stop="showContextMenu($event, app)"
                @mouseenter="handleHover('recent', index)"
              >
                <div :class="['app-icon-mini', getAppColorClass(app.category)]">
                  <img v-if="app.icon && app.type !== 'tool'" :src="app.icon" :alt="app.name" class="app-icon-img-mini" />
                  <span v-else class="app-icon-fallback-mini">{{ app.icon || getAppInitial(app.name) }}</span>
                </div>
                <span class="app-name-mini">{{ app.name }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 分隔线 -->
        <div class="section-divider" v-if="!searchQuery.trim() && (pinnedApps.length > 0 || recentApps.length > 0)"></div>

        <!-- 所有应用区 -->
        <div class="all-apps-section">
          <div class="section-header">
            <span class="section-icon">🗂️</span>
            <span class="section-title">{{ searchQuery.trim() ? '搜索结果' : '所有应用' }}</span>
            <span class="section-count">{{ allApps.length }}</span>
          </div>
          <div class="apps-grid">
            <div
              v-for="(app, index) in allApps"
              :key="app.id"
              :class="['app-item', { selected: currentSection === 'all' && selectedIndex === index, pinned: app.isPinned }]"
              @click="handleLaunchApp(app)"
              @contextmenu.prevent.stop="showContextMenu($event, app)"
              @mouseenter="handleHover('all', index)"
            >
              <div :class="['app-icon', getAppColorClass(app.category)]">
                <img v-if="app.icon && !app.icon.startsWith('🔌') && !app.icon.startsWith('🛠') && !app.icon.startsWith('🧰')" :src="app.icon" :alt="app.name" class="app-icon-img" />
                <span v-else class="app-icon-fallback">{{ app.icon || getAppInitial(app.name) }}</span>
                <!-- 置顶标记 -->
                <span v-if="app.isPinned" class="pin-badge">📌</span>
                <!-- 工具标记 -->
                <span v-if="app.type === 'tool'" class="tool-badge">🧰</span>
              </div>
              <span :class="['app-name', { 'app-name-selected': currentSection === 'all' && selectedIndex === index }]">
                {{ app.name }}
              </span>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-if="allApps.length === 0 && searchQuery.trim()" class="empty-state">
          <span>No results found for "{{ searchQuery }}"</span>
        </div>

        <!-- 加载状态 -->
        <div v-if="loading" class="loading-state">
          <span>Loading...</span>
        </div>

        <!-- 右键菜单 -->
        <div
          v-if="contextMenu.visible"
          class="context-menu"
          :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
          @click.stop
        >
          <div class="context-menu-item" @click="handleTogglePin">
            <span class="menu-icon">{{ contextMenu.item?.isPinned ? '📍' : '📌' }}</span>
            <span>{{ contextMenu.item?.isPinned ? '取消置顶' : '置顶' }}</span>
          </div>
          <div v-if="contextMenu.item?.type !== 'tool'" class="context-menu-item" @click="handleShowInFolder">
            <span class="menu-icon">📂</span>
            <span>打开文件位置</span>
          </div>
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
import { pinyin, match } from 'pinyin-pro'
import { getAllTools, getPinnedTools, pinTool, unpinTool, openToolContainer, type ToolItem as ToolItemType } from '../api/toolApi'

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

/** 统一项目类型（应用 + 工具） */
interface UnifiedItem {
  id: string
  name: string
  icon?: string
  category: string
  isPinned: boolean
  launchCount: number
  lastLaunchedAt?: number
  type: 'app' | 'tool'
  /** 应用路径（仅应用） */
  path?: string
  /** 工具组件名（仅工具） */
  component?: string
  /** 工具描述（仅工具） */
  description?: string
  tags?: string[]
  isHidden?: boolean
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
const currentSection = ref<'pinned' | 'recent' | 'all'>('all')
const activeTag = ref('all')
const loading = ref(true)
const apps = ref<AppItem[]>([])
const tools = ref<ToolItemType[]>([])
const pinnedToolIds = ref<Set<string>>(new Set())
const categories = ref<Category[]>([])
const cpuUsage = ref(0)
const memoryUsed = ref(0)

// 右键菜单状态
const contextMenu = ref<{
  visible: boolean
  x: number
  y: number
  item: UnifiedItem | null
}>({
  visible: false,
  x: 0,
  y: 0,
  item: null
})

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

  // 如果有工具，添加工具箱分类
  const toolTag = tools.value.length > 0
    ? [{ id: 'tools', name: '工具箱', icon: '🧰' }]
    : []

  return [...defaultTags, ...storeCategories, ...toolTag]
})

// 转换工具为统一项目格式
const toolsAsUnified = computed((): UnifiedItem[] => {
  return tools.value.map(tool => ({
    id: `tool:${tool.id}`,
    name: tool.name,
    icon: tool.icon,
    category: 'tools',
    isPinned: pinnedToolIds.value.has(tool.id),
    launchCount: 0,
    lastLaunchedAt: undefined,
    type: 'tool' as const,
    component: tool.component,
    description: tool.description
  }))
})

// 转换应用为统一项目格式
const appsAsUnified = computed((): UnifiedItem[] => {
  return apps.value
    .filter(app => !app.isHidden)
    .map(app => ({
      id: app.id,
      name: app.name,
      icon: app.icon,
      category: app.category,
      isPinned: app.isPinned,
      launchCount: app.launchCount,
      lastLaunchedAt: app.lastLaunchedAt,
      type: 'app' as const,
      path: app.path,
      tags: app.tags,
      isHidden: app.isHidden
    }))
})

// 固定的项目（应用 + 工具，最多8个）
const pinnedApps = computed((): UnifiedItem[] => {
  const pinnedAppItems = appsAsUnified.value.filter(item => item.isPinned)
  const pinnedToolItems = toolsAsUnified.value.filter(item => item.isPinned)

  return [...pinnedAppItems, ...pinnedToolItems]
    .sort((a, b) => b.launchCount - a.launchCount)
    .slice(0, 8)
})

// 最近使用的项目（最多8个，排除已固定的）
const recentApps = computed((): UnifiedItem[] => {
  return appsAsUnified.value
    .filter(item => !item.isPinned && item.lastLaunchedAt)
    .sort((a, b) => (b.lastLaunchedAt || 0) - (a.lastLaunchedAt || 0))
    .slice(0, 8)
})

// 所有应用区的列表（应用 + 工具）
const allApps = computed((): UnifiedItem[] => {
  // 如果选择了工具箱分类，只显示工具
  if (activeTag.value === 'tools') {
    let filteredTools = toolsAsUnified.value

    // 搜索过滤（支持拼音）
    if (searchQuery.value.trim()) {
      const query = searchQuery.value.toLowerCase()
      filteredTools = filteredTools.filter(tool => {
        const nameMatch = tool.name.toLowerCase().includes(query)
        const descMatch = tool.description?.toLowerCase().includes(query) ?? false

        const pinyinFull = pinyin(tool.name, { toneType: 'none', type: 'array' }).join('').toLowerCase()
        const pinyinFullMatch = pinyinFull.includes(query)

        const pinyinFirst = pinyin(tool.name, { pattern: 'first', type: 'array' }).join('').toLowerCase()
        const pinyinFirstMatch = pinyinFirst.includes(query)

        const fuzzyMatch = match(tool.name, query)

        return nameMatch || descMatch || pinyinFullMatch || pinyinFirstMatch || fuzzyMatch
      })
    }

    return filteredTools.slice(0, 20)
  }

  // 其他分类：显示应用（all 时也包含工具）
  let filteredItems: UnifiedItem[] = appsAsUnified.value

  // 分类过滤
  if (activeTag.value !== 'all') {
    filteredItems = filteredItems.filter(item => item.category === activeTag.value)
  } else {
    // all 分类时，将工具也加入
    filteredItems = [...filteredItems, ...toolsAsUnified.value]
  }

  // 搜索过滤（支持拼音）
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    filteredItems = filteredItems.filter(item => {
      // 名称匹配
      const nameMatch = item.name.toLowerCase().includes(query)

      // 标签匹配（仅应用）
      const tagMatch = item.tags?.some(tag => tag.toLowerCase().includes(query)) ?? false

      // 描述匹配（仅工具）
      const descMatch = item.description?.toLowerCase().includes(query) ?? false

      // 拼音全拼匹配
      const pinyinFull = pinyin(item.name, { toneType: 'none', type: 'array' }).join('').toLowerCase()
      const pinyinFullMatch = pinyinFull.includes(query)

      // 拼音首字母匹配
      const pinyinFirst = pinyin(item.name, { pattern: 'first', type: 'array' }).join('').toLowerCase()
      const pinyinFirstMatch = pinyinFirst.includes(query)

      // pinyin-pro 的模糊匹配
      const fuzzyMatch = match(item.name, query)

      return nameMatch || tagMatch || descMatch || pinyinFullMatch || pinyinFirstMatch || fuzzyMatch
    })
  }

  // 排序：置顶 > 使用频率
  filteredItems = [...filteredItems].sort((a, b) => {
    if (a.isPinned && !b.isPinned) return -1
    if (!a.isPinned && b.isPinned) return 1
    return b.launchCount - a.launchCount
  })

  return filteredItems.slice(0, 20)
})

// 获取当前区域的应用列表（用于键盘导航）
const getCurrentSectionApps = computed(() => {
  if (searchQuery.value.trim()) {
    return allApps.value
  }

  switch (currentSection.value) {
    case 'pinned':
      return pinnedApps.value
    case 'recent':
      return recentApps.value
    case 'all':
      return allApps.value
    default:
      return allApps.value
  }
})

// 关闭启动器
const closeLauncher = async () => {
  const window = getCurrentWindow()
  await window.hide()
}

// 鼠标悬停处理
const handleHover = (section: 'pinned' | 'recent' | 'all', index: number) => {
  currentSection.value = section
  selectedIndex.value = index
}

// 键盘导航
const navigate = (direction: 'up' | 'down') => {
  const currentApps = getCurrentSectionApps.value
  if (currentApps.length === 0) return

  // 如果在搜索状态，只导航搜索结果
  if (searchQuery.value.trim()) {
    currentSection.value = 'all'
    const cols = 5
    const total = currentApps.length

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
    return
  }

  // 非搜索状态，支持区域切换
  const cols = currentSection.value === 'all' ? 5 : 4
  const total = currentApps.length

  if (direction === 'down') {
    if (selectedIndex.value + cols < total) {
      selectedIndex.value += cols
    } else {
      // 尝试切换到下一个区域
      if (currentSection.value === 'pinned' && recentApps.value.length > 0) {
        currentSection.value = 'recent'
        selectedIndex.value = 0
      } else if ((currentSection.value === 'pinned' || currentSection.value === 'recent') && allApps.value.length > 0) {
        currentSection.value = 'all'
        selectedIndex.value = 0
      } else if (selectedIndex.value < total - 1) {
        selectedIndex.value = total - 1
      }
    }
  } else {
    if (selectedIndex.value - cols >= 0) {
      selectedIndex.value -= cols
    } else if (selectedIndex.value > 0) {
      selectedIndex.value = 0
    } else {
      // 尝试切换到上一个区域
      if (currentSection.value === 'all' && recentApps.value.length > 0) {
        currentSection.value = 'recent'
        selectedIndex.value = recentApps.value.length - 1
      } else if (currentSection.value === 'all' && pinnedApps.value.length > 0) {
        currentSection.value = 'pinned'
        selectedIndex.value = pinnedApps.value.length - 1
      } else if (currentSection.value === 'recent' && pinnedApps.value.length > 0) {
        currentSection.value = 'pinned'
        selectedIndex.value = pinnedApps.value.length - 1
      }
    }
  }
}

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    e.preventDefault()
    closeLauncher()
  } else if (e.key === 'ArrowLeft') {
    e.preventDefault()
    const currentApps = getCurrentSectionApps.value
    if (selectedIndex.value > 0) {
      selectedIndex.value--
    }
  } else if (e.key === 'ArrowRight') {
    e.preventDefault()
    const currentApps = getCurrentSectionApps.value
    if (selectedIndex.value < currentApps.length - 1) {
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
  const currentApps = getCurrentSectionApps.value
  const app = currentApps[selectedIndex.value]
  if (app) {
    handleLaunchApp(app)
  }
}

const handleLaunchApp = async (item: UnifiedItem) => {
  try {
    // 设置启动标志，防止窗口在启动过程中因失焦而关闭
    isLaunching.value = true

    if (item.type === 'tool') {
      // 工具：打开工具容器窗口
      const toolId = item.id.replace('tool:', '')
      await openToolContainer(toolId)
    } else {
      // 应用：启动应用
      await invoke('launch_app', { appId: item.id })
    }

    closeLauncher()
  } catch (error) {
    console.error('启动失败:', error)
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
    'other': 'bg-slate',
    'tools': 'bg-orange',
    'network': 'bg-cyan',
    'system': 'bg-indigo',
    'development': 'bg-blue',
    'utility': 'bg-teal',
    'file': 'bg-amber'
  }
  return colorMap[category] || 'bg-violet'
}

const getAppInitial = (name: string): string => {
  return name.charAt(0).toUpperCase()
}

// 显示右键菜单
const showContextMenu = (event: MouseEvent, item: UnifiedItem) => {
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    item
  }
}

// 隐藏右键菜单
const hideContextMenu = () => {
  contextMenu.value.visible = false
  contextMenu.value.item = null
}

// 置顶/取消置顶
const handleTogglePin = async () => {
  if (!contextMenu.value.item) return
  try {
    const item = contextMenu.value.item

    if (item.type === 'tool') {
      // 工具：使用工具API进行固定/取消固定
      const toolId = item.id.replace('tool:', '')
      if (item.isPinned) {
        await unpinTool(toolId)
        pinnedToolIds.value.delete(toolId)
      } else {
        await pinTool(toolId)
        pinnedToolIds.value.add(toolId)
      }
    } else {
      // 应用：使用原来的API
      await invoke('toggle_pin_app', { appId: item.id })
      // 更新本地状态
      const app = apps.value.find(a => a.id === item.id)
      if (app) {
        app.isPinned = !app.isPinned
      }
    }
  } catch (error) {
    console.error('置顶失败:', error)
  }
  hideContextMenu()
}

// 打开文件位置（仅应用）
const handleShowInFolder = async () => {
  if (!contextMenu.value.item) return
  // 工具没有文件路径
  if (contextMenu.value.item.type === 'tool') {
    hideContextMenu()
    return
  }
  try {
    await invoke('show_in_folder', { path: contextMenu.value.item.path })
  } catch (error) {
    console.error('打开文件位置失败:', error)
  }
  hideContextMenu()
}

const formatMemory = (bytes: number): string => {
  const gb = bytes / (1024 * 1024 * 1024)
  return gb.toFixed(1) + 'GB'
}

// 加载数据
const loadData = async () => {
  loading.value = true
  try {
    const [appsData, categoriesData, systemInfo, toolsData, pinnedToolsData] = await Promise.all([
      invoke<AppItem[]>('get_all_apps'),
      invoke<Category[]>('get_categories'),
      invoke<SystemInfo>('get_system_info'),
      getAllTools(),
      getPinnedTools()
    ])

    apps.value = appsData
    categories.value = categoriesData
    cpuUsage.value = Math.round(systemInfo.cpu_usage)
    memoryUsed.value = systemInfo.memory_used
    tools.value = toolsData
    pinnedToolIds.value = new Set(pinnedToolsData.map(t => t.id))
  } catch (error) {
    console.error('加载数据失败:', error)
  } finally {
    loading.value = false
  }
}

// 监听搜索变化
watch(searchQuery, () => {
  selectedIndex.value = 0
  // 搜索时自动切换到所有应用区
  if (searchQuery.value.trim()) {
    currentSection.value = 'all'
  } else {
    // 清空搜索时，如果有固定应用则切换到固定区，否则切换到所有应用区
    if (pinnedApps.value.length > 0) {
      currentSection.value = 'pinned'
    } else if (recentApps.value.length > 0) {
      currentSection.value = 'recent'
    } else {
      currentSection.value = 'all'
    }
  }
})

watch(activeTag, () => {
  selectedIndex.value = 0
  currentSection.value = 'all'
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

  // 点击任意位置隐藏右键菜单
  document.addEventListener('click', hideContextMenu)

  // 监听窗口显示/隐藏
  const window = getCurrentWindow()
  window.onFocusChanged(({ payload: focused }) => {
    if (focused) {
      // 窗口获得焦点时重新加载数据并聚焦
      loadData()
      searchQuery.value = ''
      selectedIndex.value = 0
      activeTag.value = 'all'
      // 重置到固定区或所有应用区
      currentSection.value = pinnedApps.value.length > 0 ? 'pinned' : 'all'
      isLaunching.value = false
      hideContextMenu()
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
  background: var(--bg-base);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
  border: 1px solid var(--border-default);
  box-shadow: var(--shadow-lg);
}

/* 装饰背景网格 */
.modal-grid-bg {
  position: absolute;
  inset: 0;
  opacity: 0.15;
  pointer-events: none;
  background-image: radial-gradient(var(--accent-primary) 1px, transparent 1px);
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
  border-bottom: 1px solid var(--border-default);
  padding-bottom: 16px;
}

.search-icon-wrapper {
  flex-shrink: 0;
}

.search-icon {
  width: 28px;
  height: 28px;
  color: var(--accent-primary);
  animation: pulse-glow 2s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%, 100% { opacity: 1; filter: drop-shadow(0 0 4px var(--accent-glow)); }
  50% { opacity: 0.7; filter: drop-shadow(0 0 8px var(--accent-primary)); }
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 24px;
  font-weight: 300;
  color: var(--text-primary);
  letter-spacing: 0.025em;
  height: 48px;
}

.search-input::placeholder {
  color: var(--text-dim);
}

.search-hint {
  display: flex;
  gap: 4px;
  font-size: 11px;
  font-family: 'Courier New', monospace;
  color: var(--text-muted);
  border: 1px solid var(--border-default);
  padding: 4px 8px;
  border-radius: 4px;
  background: var(--bg-overlay);
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
  background: var(--bg-overlay);
  border: 1px solid transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  flex-shrink: 0;
}

.tag-btn:hover {
  color: var(--text-secondary);
  background: var(--bg-hover);
}

.tag-btn.active {
  background: var(--accent-glow);
  border-color: var(--accent-primary);
  color: var(--accent-secondary);
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
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.modal-body::-webkit-scrollbar {
  width: 6px;
  background: transparent;
}

.modal-body::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
  border-radius: 3px;
}

.modal-body::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 3px;
}

.modal-body::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
}

/* 顶部区域：固定 + 最近使用 */
.top-section {
  display: flex;
  gap: 16px;
  padding-bottom: 8px;
}

.section-block {
  flex: 1;
  min-width: 0;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 12px;
  padding: 0 4px;
}

.section-icon {
  font-size: 14px;
  opacity: 0.8;
}

.section-title {
  font-size: 12px;
  color: var(--text-muted);
  font-weight: 500;
  letter-spacing: 0.5px;
}

.section-count {
  font-size: 10px;
  color: var(--text-dim);
  background: var(--bg-overlay);
  padding: 2px 6px;
  border-radius: 8px;
  font-family: 'Courier New', monospace;
}

/* 迷你应用网格（固定 + 最近使用） */
.mini-apps-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}

.app-item-mini {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  padding: 8px 6px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid transparent;
  min-width: 0;
}

.app-item-mini:hover {
  background: var(--bg-hover);
  border-color: var(--border-default);
}

.app-item-mini.selected {
  background: var(--accent-glow);
  border-color: var(--accent-primary);
  box-shadow: 0 0 12px var(--accent-glow);
}

.app-icon-mini {
  width: 40px;
  height: 40px;
  min-width: 40px;
  min-height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 6px;
  box-shadow: var(--shadow-md);
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.app-item-mini:hover .app-icon-mini {
  transform: scale(1.08);
}

.app-icon-img-mini {
  width: 26px;
  height: 26px;
  object-fit: contain;
}

.app-icon-fallback-mini {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-on-accent);
}

.app-name-mini {
  font-size: 10px;
  text-align: center;
  color: var(--text-secondary);
  font-weight: 500;
  width: 100%;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: color 0.2s ease;
  line-height: 1.3;
}

.app-item-mini:hover .app-name-mini {
  color: var(--text-primary);
}

.app-item-mini.selected .app-name-mini {
  color: var(--text-primary);
}

/* 分隔线 */
.section-divider {
  height: 1px;
  background: linear-gradient(
    to right,
    transparent,
    var(--accent-glow) 20%,
    var(--accent-glow) 80%,
    transparent
  );
  margin: 8px 0;
}

/* 所有应用区 */
.all-apps-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
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
  background: var(--bg-hover);
  border-color: var(--border-default);
}

.app-item.selected {
  background: var(--accent-glow);
  border-color: var(--accent-primary);
  box-shadow: 0 0 15px var(--accent-glow);
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
  box-shadow: var(--shadow-md);
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
  color: var(--text-on-accent);
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
.bg-orange { background: linear-gradient(135deg, #ea580c 0%, #f97316 100%); }
.bg-cyan { background: linear-gradient(135deg, #0891b2 0%, #06b6d4 100%); }
.bg-teal { background: linear-gradient(135deg, #0d9488 0%, #14b8a6 100%); }
.bg-indigo { background: linear-gradient(135deg, #4f46e5 0%, #6366f1 100%); }
.bg-amber { background: linear-gradient(135deg, #d97706 0%, #fbbf24 100%); }

.app-name {
  font-size: 11px;
  text-align: center;
  color: var(--text-secondary);
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
  color: var(--text-primary);
}

.app-name-selected {
  color: var(--text-primary);
}

/* 空状态和加载状态 */
.empty-state,
.loading-state {
  grid-column: 1 / -1;
  text-align: center;
  color: var(--text-dim);
  padding: 60px 20px;
  font-size: 14px;
}

/* Footer */
.modal-footer {
  height: 40px;
  background: var(--bg-overlay);
  border-top: 1px solid var(--border-default);
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
  color: var(--text-muted);
}

.stat-icon {
  width: 12px;
  height: 12px;
}

.stat-icon.cpu {
  color: var(--accent-primary);
}

.stat-icon.mem {
  color: var(--info);
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
  color: var(--text-muted);
}

.hint kbd {
  display: inline-block;
  padding: 2px 6px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 3px;
  font-size: 9px;
  color: var(--text-secondary);
}

/* 置顶标记 */
.app-icon {
  position: relative;
}

.pin-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  font-size: 10px;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.5));
}

.tool-badge {
  position: absolute;
  bottom: -4px;
  right: -4px;
  font-size: 10px;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.5));
}

.app-item.pinned {
  border-color: var(--accent-glow);
}

/* 右键菜单 */
.context-menu {
  position: fixed;
  background: var(--bg-elevated);
  border: 1px solid var(--border-active);
  border-radius: 8px;
  padding: 4px 0;
  min-width: 160px;
  box-shadow: var(--shadow-lg);
  z-index: 1000;
  animation: contextMenuFadeIn 0.15s ease;
}

@keyframes contextMenuFadeIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  cursor: pointer;
  color: var(--text-primary);
  font-size: 13px;
  transition: all 0.15s ease;
}

.context-menu-item:hover {
  background: var(--accent-glow);
  color: var(--accent-secondary);
}

.menu-icon {
  font-size: 14px;
  width: 20px;
  text-align: center;
}
</style>
