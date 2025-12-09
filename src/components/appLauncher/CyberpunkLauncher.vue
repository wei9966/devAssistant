<template>
  <!-- 背景遮罩 (Backdrop) -->
  <Teleport to="body">
    <Transition name="backdrop-fade">
      <div
        v-if="isVisible"
        class="cyberpunk-backdrop"
        @click.self="close"
      >
        <!-- 模态框主体 (Modal Container) -->
        <Transition name="cyber-fade">
          <div
            v-if="isVisible"
            class="cyberpunk-modal"
            @keydown="handleKeyDown"
          >
            <!-- 装饰背景噪点/网格 -->
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
                  placeholder="尝试搜索应用，命令，或者AI（支持拼音）"
                  class="search-input"
                  @keydown.down.prevent="navigate('down')"
                  @keydown.up.prevent="navigate('up')"
                  @keydown.enter="handleLaunch"
                />

                <!-- 尾部提示 -->
                <div class="search-hint">
                  <span>CMD</span>
                  <span>K</span>
                </div>
              </div>

              <!-- 快捷分类标签 (Capsule Buttons) -->
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
              <div v-if="displayApps.length === 0" class="empty-state">
                <span>No results found for "{{ searchQuery }}"</span>
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
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useAppLauncherStore } from '@/stores/appLauncherStore'
import type { AppItem, Category } from '@/types/appLauncher'
import { invoke } from '@tauri-apps/api/core'
import { pinyin, match } from 'pinyin-pro'

interface SystemInfo {
  cpu_usage: number
  memory_usage: number
  memory_total: number
  memory_used: number
}

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'launch': [app: AppItem]
}>()

const store = useAppLauncherStore()

// 状态
const searchInputRef = ref<HTMLInputElement>()
const searchQuery = ref('')
const selectedIndex = ref(0)
const activeTag = ref('all')
const cpuUsage = ref(0)
const memoryUsed = ref(0)
const memoryTotal = ref(0)

// 计算属性
const isVisible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

// 分类标签
const categoryTags = computed(() => {
  const defaultTags = [
    { id: 'all', name: 'All', icon: '📦' },
  ]

  // 从 store 获取分类，排除 'all'
  const storeCategories = store.categories
    .filter(cat => cat.id !== 'all')
    .slice(0, 4)
    .map(cat => ({
      id: cat.id,
      name: cat.name,
      icon: cat.icon || '📁'
    }))

  return [...defaultTags, ...storeCategories]
})

// 过滤后的应用列表
const displayApps = computed(() => {
  let apps = store.apps.filter(app => !app.isHidden)

  // 分类过滤
  if (activeTag.value !== 'all') {
    apps = apps.filter(app => app.category === activeTag.value)
  }

  // 搜索过滤
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    apps = apps.filter(app => {
      // 应用名称匹配
      const nameMatch = app.name.toLowerCase().includes(query)

      // 标签匹配
      const tagMatch = app.tags?.some(tag => tag.toLowerCase().includes(query)) ?? false

      // 拼音全拼匹配
      const pinyinFull = pinyin(app.name, { toneType: 'none', type: 'array' }).join('').toLowerCase()
      const pinyinFullMatch = pinyinFull.includes(query)

      // 拼音首字母匹配
      const pinyinFirst = pinyin(app.name, { pattern: 'first', type: 'array' }).join('').toLowerCase()
      const pinyinFirstMatch = pinyinFirst.includes(query)

      // pinyin-pro 的模糊匹配
      const fuzzyMatch = match(app.name, query)

      return nameMatch || tagMatch || pinyinFullMatch || pinyinFirstMatch || fuzzyMatch
    })
  }

  // 按使用频率和置顶排序
  apps = [...apps].sort((a, b) => {
    if (a.isPinned && !b.isPinned) return -1
    if (!a.isPinned && b.isPinned) return 1
    return b.launchCount - a.launchCount
  })

  return apps.slice(0, 15) // 最多显示15个
})

// 方法
const close = () => {
  isVisible.value = false
}

const navigate = (direction: 'up' | 'down') => {
  if (displayApps.value.length === 0) return

  const cols = 5 // 每行5个
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
    close()
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
    // 切换分类
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
    await store.launchApp(app.id)
    emit('launch', app)
    close()
  } catch (error) {
    console.error('启动应用失败:', error)
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

// 获取系统信息
const getSystemInfo = async () => {
  try {
    const info = await invoke<SystemInfo>('get_system_info')
    cpuUsage.value = Math.round(info.cpu_usage)
    memoryUsed.value = info.memory_used
    memoryTotal.value = info.memory_total
  } catch (error) {
    console.error('获取系统信息失败:', error)
  }
}

// 监听显示状态
watch(isVisible, async (visible) => {
  if (visible) {
    searchQuery.value = ''
    selectedIndex.value = 0
    activeTag.value = 'all'

    // 加载应用列表
    if (store.apps.length === 0) {
      await store.loadApps()
    }
    if (store.categories.length === 0) {
      await store.loadCategories()
    }

    // 获取系统信息
    getSystemInfo()

    // 聚焦搜索框
    nextTick(() => {
      searchInputRef.value?.focus()
    })
  }
})

// 监听搜索变化，重置选中索引
watch(searchQuery, () => {
  selectedIndex.value = 0
})

watch(activeTag, () => {
  selectedIndex.value = 0
})

// 系统信息定时更新
let systemInfoTimer: number | null = null
onMounted(() => {
  systemInfoTimer = window.setInterval(getSystemInfo, 5000)
})

onUnmounted(() => {
  if (systemInfoTimer) {
    clearInterval(systemInfoTimer)
  }
})
</script>

<style scoped>
/* ============================================ */
/* 核心：赛博朋克霓虹光效定义                    */
/* ============================================ */

/* 背景遮罩 */
.cyberpunk-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(5, 5, 8, 0.6);
  backdrop-filter: blur(20px);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

/* 模态框主体 */
.cyberpunk-modal {
  width: 750px;
  height: 520px;
  background: rgba(12, 12, 20, 0.95);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.1);
  /* 霓虹边框光晕 */
  box-shadow:
    0 0 0 1px rgba(139, 92, 246, 0.3),
    0 0 20px rgba(139, 92, 246, 0.15),
    0 0 40px rgba(6, 182, 212, 0.1),
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

/* ============================================ */
/* Header: 搜索栏                               */
/* ============================================ */
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

/* ============================================ */
/* Body: 应用网格                               */
/* ============================================ */
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

/* 空状态 */
.empty-state {
  grid-column: 1 / -1;
  text-align: center;
  color: #475569;
  padding: 60px 20px;
  font-size: 14px;
}

/* ============================================ */
/* Footer: 状态栏与提示                          */
/* ============================================ */
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

/* ============================================ */
/* 动画                                         */
/* ============================================ */

/* 背景遮罩过渡 */
.backdrop-fade-enter-active,
.backdrop-fade-leave-active {
  transition: opacity 0.3s ease;
}

.backdrop-fade-enter-from,
.backdrop-fade-leave-to {
  opacity: 0;
}

/* 模态框进出动画 */
.cyber-fade-enter-active,
.cyber-fade-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.cyber-fade-enter-from,
.cyber-fade-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(10px);
  filter: blur(10px);
}

.cyber-fade-enter-to,
.cyber-fade-leave-from {
  opacity: 1;
  transform: scale(1) translateY(0);
  filter: blur(0);
}

/* ============================================ */
/* 响应式                                       */
/* ============================================ */
@media (max-width: 800px) {
  .cyberpunk-modal {
    width: 95vw;
    height: 85vh;
  }

  .apps-grid {
    grid-template-columns: repeat(4, 1fr);
  }

  .search-input {
    font-size: 18px;
  }
}

@media (max-width: 640px) {
  .apps-grid {
    grid-template-columns: repeat(3, 1fr);
  }

  .keyboard-hints {
    display: none;
  }
}
</style>
