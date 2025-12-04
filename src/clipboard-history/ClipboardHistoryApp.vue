<template>
  <div class="clipboard-window" @keydown="handleKeyDown" tabindex="0" ref="containerRef">
    <!-- 背景遮罩层（点击关闭） -->
    <div class="clipboard-backdrop" @click="closeWindow"></div>

    <!-- 模态框主体 -->
    <div class="clipboard-modal" :class="{ 'show-settings': showSettings }">
      <!-- 装饰背景网格 -->
      <div class="modal-grid-bg"></div>

      <!-- 主内容 -->
      <div v-if="!showSettings" class="main-content">
        <!-- Header: 搜索栏 -->
        <div class="modal-header">
          <div class="search-container">
            <div class="search-icon-wrapper">
              <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"/>
                <path d="m21 21-4.35-4.35"/>
              </svg>
            </div>
            <input
              ref="searchInputRef"
              type="text"
              v-model="searchQuery"
              placeholder="搜索剪切板历史..."
              class="search-input"
              @keydown.down.prevent="navigate('down')"
              @keydown.up.prevent="navigate('up')"
              @keydown.enter="handleCopy"
            />
            <div class="search-hint">
              <span>ESC</span>
            </div>
          </div>

          <!-- 过滤标签 -->
          <div class="tags-container">
            <button
              v-for="tag in filterTags"
              :key="tag.id"
              :class="['tag-btn', { active: activeFilter === tag.id }]"
              @click="activeFilter = tag.id"
            >
              <span class="tag-icon">{{ tag.icon }}</span>
              <span>{{ tag.name }}</span>
            </button>
          </div>
        </div>

        <!-- Body: 历史列表 -->
        <div class="modal-body" ref="listRef">
          <!-- 置顶项 -->
          <div v-if="pinnedItems.length > 0" class="history-section">
            <div class="section-header">
              <span class="section-icon">📌</span>
              <span>置顶项</span>
            </div>
            <div
              v-for="(item, index) in pinnedItems"
              :key="'pinned-' + item.id"
              :class="['history-item', { selected: selectedIndex === index }]"
              @click="handleCopyItem(item)"
              @mouseenter="selectedIndex = index"
              @contextmenu.prevent="showContextMenu($event, item)"
            >
              <!-- 图片预览 -->
              <div v-if="item.content_type === 'image'" class="item-image-preview">
                <img
                  v-if="!imageLoadErrors[item.id]"
                  :src="getImageSrc(item)"
                  alt="preview"
                  @error="handleImageError($event, item.id)"
                />
                <span v-else class="image-placeholder">🖼️</span>
              </div>
              <div v-else class="item-icon" :class="getTypeClass(item.content_type)">
                {{ getTypeIcon(item.content_type) }}
              </div>
              <div class="item-content">
                <div class="item-preview">{{ getPreview(item) }}</div>
                <div class="item-time">{{ formatTime(item.created_at) }}</div>
              </div>
              <button class="item-pin active" @click.stop="togglePin(item.id)" title="取消置顶">
                📌
              </button>
            </div>
          </div>

          <!-- 按日期分组的历史 -->
          <div v-for="group in groupedHistory" :key="group.date" class="history-section">
            <div class="section-header">
              <span class="section-icon">📅</span>
              <span>{{ group.label }}</span>
            </div>
            <div
              v-for="item in group.items"
              :key="item.id"
              :class="['history-item', { selected: selectedIndex === getItemIndex(item) }]"
              @click="handleCopyItem(item)"
              @mouseenter="selectedIndex = getItemIndex(item)"
              @contextmenu.prevent="showContextMenu($event, item)"
            >
              <!-- 图片预览 -->
              <div v-if="item.content_type === 'image'" class="item-image-preview">
                <img
                  v-if="!imageLoadErrors[item.id]"
                  :src="getImageSrc(item)"
                  alt="preview"
                  @error="handleImageError($event, item.id)"
                />
                <span v-else class="image-placeholder">🖼️</span>
              </div>
              <div v-else class="item-icon" :class="getTypeClass(item.content_type)">
                {{ getTypeIcon(item.content_type) }}
              </div>
              <div class="item-content">
                <div class="item-preview">{{ getPreview(item) }}</div>
                <div class="item-time">{{ formatTime(item.created_at) }}</div>
              </div>
              <button class="item-pin" @click.stop="togglePin(item.id)" title="置顶">
                📍
              </button>
            </div>
          </div>

          <!-- 空状态 -->
          <div v-if="filteredHistory.length === 0 && pinnedItems.length === 0 && !loading" class="empty-state">
            <span v-if="searchQuery">未找到匹配 "{{ searchQuery }}" 的记录</span>
            <span v-else>暂无剪切板历史</span>
          </div>

          <!-- 加载状态 -->
          <div v-if="loading" class="loading-state">
            <span>加载中...</span>
          </div>
        </div>

        <!-- Footer: 状态栏 -->
        <div class="modal-footer">
          <div class="footer-stats">
            <span class="stat-item">
              <span class="stat-label">共</span>
              <span class="stat-value">{{ totalCount }}</span>
              <span class="stat-label">条记录</span>
            </span>
          </div>
          <div class="footer-actions">
            <button class="footer-btn" @click="showSettings = true" title="设置">
              ⚙️
            </button>
          </div>
        </div>
      </div>

      <!-- 设置面板 -->
      <div v-else class="settings-panel">
        <div class="settings-header">
          <button class="back-btn" @click="showSettings = false">
            ← 返回
          </button>
          <span class="settings-title">剪切板设置</span>
        </div>

        <div class="settings-body">
          <!-- 基本设置 -->
          <div class="settings-section">
            <div class="section-title">基本设置</div>

            <div class="setting-item">
              <div class="setting-label">启用剪切板监控</div>
              <label class="switch">
                <input type="checkbox" v-model="config.enabled" @change="saveConfig">
                <span class="slider"></span>
              </label>
            </div>

            <div class="setting-item">
              <div class="setting-label">最大历史记录数</div>
              <input
                type="number"
                v-model.number="config.max_history"
                class="setting-input"
                min="10"
                max="1000"
                @change="saveConfig"
              />
            </div>
          </div>

          <!-- 快捷键设置 -->
          <div class="settings-section">
            <div class="section-title">快捷键</div>

            <div class="setting-item">
              <div class="setting-label">呼出剪切板历史</div>
              <div class="shortcut-input" @click="startRecordingShortcut">
                <span v-if="recordingShortcut" class="recording">按下快捷键...</span>
                <span v-else>{{ config.shortcut }}</span>
              </div>
            </div>

            <div class="setting-item">
              <div class="setting-label">复制最近图片路径</div>
              <div class="shortcut-display">Ctrl+Shift+V</div>
            </div>
            <div class="setting-hint" style="padding: 0 0 12px 0; margin-top: -8px;">
              复制图片后保留原图可粘贴到微信，按此快捷键获取路径用于CMD
            </div>
          </div>

          <!-- 路径设置 -->
          <div class="settings-section">
            <div class="section-title">存储路径</div>

            <div class="setting-item column">
              <div class="setting-label">图片保存目录</div>
              <div class="path-input-wrapper">
                <input
                  type="text"
                  v-model="config.image_save_dir"
                  class="setting-input path-input"
                  @change="saveConfig"
                />
                <button class="browse-btn" @click="browseFolder('save')">浏览</button>
              </div>
            </div>

            <div class="setting-item column">
              <div class="setting-label">图片归档目录</div>
              <div class="path-input-wrapper">
                <input
                  type="text"
                  v-model="config.image_archive_dir"
                  class="setting-input path-input"
                  @change="saveConfig"
                />
                <button class="browse-btn" @click="browseFolder('archive')">浏览</button>
              </div>
              <div class="setting-hint">超出最大记录数的旧记录会归档到此目录</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 右键菜单 -->
    <div
      v-if="contextMenu.visible"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <div class="context-item" @click="handleContextCopy">
        <span>📋</span>
        <span>复制</span>
      </div>
      <div v-if="contextMenu.item?.content_type === 'image'" class="context-item" @click="handleCopyImagePath">
        <span>📁</span>
        <span>复制路径</span>
      </div>
      <div class="context-item" @click="handleContextPin">
        <span>{{ contextMenu.item?.is_pinned ? '📍' : '📌' }}</span>
        <span>{{ contextMenu.item?.is_pinned ? '取消置顶' : '置顶' }}</span>
      </div>
      <div class="context-item danger" @click="showDeleteConfirm = true">
        <span>🗑️</span>
        <span>删除</span>
      </div>
    </div>

    <!-- 删除确认对话框 -->
    <div v-if="showDeleteConfirm" class="confirm-overlay" @click="showDeleteConfirm = false">
      <div class="confirm-dialog" @click.stop>
        <div class="confirm-title">确认删除</div>
        <div class="confirm-message">确定要删除这条记录吗？</div>
        <div class="confirm-actions">
          <button class="confirm-btn cancel" @click="showDeleteConfirm = false">取消</button>
          <button class="confirm-btn danger" @click="handleContextDelete">删除</button>
        </div>
      </div>
    </div>

    <!-- 清空确认对话框 -->
    <div v-if="showClearConfirm" class="confirm-overlay" @click="showClearConfirm = false">
      <div class="confirm-dialog" @click.stop>
        <div class="confirm-title">确认清空</div>
        <div class="confirm-message">确定要清空剪切板历史吗？置顶项将被保留。</div>
        <div class="confirm-actions">
          <button class="confirm-btn cancel" @click="showClearConfirm = false">取消</button>
          <button class="confirm-btn danger" @click="handleClear">清空</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import { convertFileSrc } from '@tauri-apps/api/core'
import { clipboardApi } from '@/api/clipboardApi'
import type { ClipboardHistory, ClipboardHistoryGroup, ClipboardContentType, ClipboardConfig } from '@/types/clipboard'

// Refs
const containerRef = ref<HTMLElement | null>(null)
const searchInputRef = ref<HTMLInputElement | null>(null)
const listRef = ref<HTMLElement | null>(null)

// State
const searchQuery = ref('')
const activeFilter = ref('all')
const selectedIndex = ref(0)
const loading = ref(false)
const history = ref<ClipboardHistory[]>([])
const totalCount = ref(0)
const showSettings = ref(false)
const showDeleteConfirm = ref(false)
const showClearConfirm = ref(false)
const recordingShortcut = ref(false)
const imageLoadErrors = ref<Record<number, boolean>>({})

// Config
const config = ref<ClipboardConfig>({
  image_save_dir: '',
  image_archive_dir: '',
  max_history: 1000,
  auto_copy_image_path: true,
  enabled: true,
  shortcut: 'Ctrl+Shift+C'
})

// Context menu
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  item: null as ClipboardHistory | null
})

// Filter tags
const filterTags = [
  { id: 'all', name: '全部', icon: '📋' },
  { id: 'text', name: '文本', icon: '📝' },
  { id: 'image', name: '图片', icon: '🖼️' },
  { id: 'pinned', name: '置顶', icon: '📌' }
]

// Computed
const pinnedItems = computed(() => {
  if (activeFilter.value === 'pinned') {
    return history.value.filter(item => item.is_pinned)
  }
  return history.value.filter(item => item.is_pinned)
})

const filteredHistory = computed(() => {
  let items = history.value.filter(item => !item.is_pinned)

  if (activeFilter.value === 'pinned') {
    return []
  }

  if (activeFilter.value !== 'all') {
    items = items.filter(item => item.content_type === activeFilter.value)
  }

  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    items = items.filter(item =>
      item.content.toLowerCase().includes(query) ||
      (item.preview && item.preview.toLowerCase().includes(query))
    )
  }

  return items
})

const groupedHistory = computed((): ClipboardHistoryGroup[] => {
  const groups: Map<string, ClipboardHistory[]> = new Map()
  const today = new Date().toDateString()
  const yesterday = new Date(Date.now() - 86400000).toDateString()

  for (const item of filteredHistory.value) {
    const date = new Date(item.created_at).toDateString()
    if (!groups.has(date)) {
      groups.set(date, [])
    }
    groups.get(date)!.push(item)
  }

  return Array.from(groups.entries()).map(([date, items]) => ({
    date,
    label: date === today ? '今天' : date === yesterday ? '昨天' : formatDate(date),
    items
  }))
})

const allItems = computed(() => {
  return [...pinnedItems.value, ...filteredHistory.value]
})

// Methods
function getItemIndex(item: ClipboardHistory): number {
  return allItems.value.findIndex(i => i.id === item.id)
}

function getTypeIcon(type: ClipboardContentType): string {
  switch (type) {
    case 'image': return '🖼️'
    case 'file': return '📁'
    default: return '📝'
  }
}

function getTypeClass(type: ClipboardContentType): string {
  return `type-${type}`
}

function getImageSrc(item: ClipboardHistory): string {
  const path = item.image_path || item.content
  // 使用 Tauri 的 convertFileSrc 将本地路径转换为可访问的 URL
  const src = convertFileSrc(path)
  console.log('图片路径转换:', path, '->', src)
  return src
}

function handleImageError(e: Event, itemId: number) {
  imageLoadErrors.value[itemId] = true
}

function getPreview(item: ClipboardHistory): string {
  if (item.content_type === 'image') {
    const filename = (item.image_path || item.content).split(/[/\\]/).pop() || '未知'
    return `[图片] ${filename}`
  }
  const preview = item.preview || item.content
  return preview.length > 80 ? preview.slice(0, 80) + '...' : preview
}

function formatTime(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`
  if (diff < 86400000) return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN', { month: 'long', day: 'numeric' })
}

async function loadHistory() {
  loading.value = true
  try {
    // 使用配置中的 max_history 值，而不是硬编码的 100
    const limit = config.value.max_history || 1000
    history.value = await clipboardApi.getHistory(limit)
    totalCount.value = await clipboardApi.getHistoryCount()
    // 重置图片加载错误状态
    imageLoadErrors.value = {}
  } catch (e) {
    console.error('加载历史失败:', e)
  } finally {
    loading.value = false
  }
}

async function loadConfig() {
  try {
    config.value = await clipboardApi.getConfig()
  } catch (e) {
    console.error('加载配置失败:', e)
  }
}

async function saveConfig() {
  try {
    await clipboardApi.updateConfig(config.value)
  } catch (e) {
    console.error('保存配置失败:', e)
  }
}

async function handleCopyItem(item: ClipboardHistory) {
  try {
    await clipboardApi.copyFromHistory(item.id)
    closeWindow()
  } catch (e) {
    console.error('复制失败:', e)
  }
}

function handleCopy() {
  const item = allItems.value[selectedIndex.value]
  if (item) {
    handleCopyItem(item)
  }
}

async function togglePin(id: number) {
  try {
    await clipboardApi.togglePin(id)
    await loadHistory()
  } catch (e) {
    console.error('置顶失败:', e)
  }
}

async function deleteItem(id: number) {
  try {
    await clipboardApi.deleteItem(id)
    await loadHistory()
  } catch (e) {
    console.error('删除失败:', e)
  }
}

async function handleClear() {
  showClearConfirm.value = false
  try {
    await clipboardApi.clearHistory(true)
    await loadHistory()
  } catch (e) {
    console.error('清空失败:', e)
  }
}

async function browseFolder(type: 'save' | 'archive') {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: type === 'save' ? '选择图片保存目录' : '选择图片归档目录'
    })

    if (selected && typeof selected === 'string') {
      if (type === 'save') {
        config.value.image_save_dir = selected
      } else {
        config.value.image_archive_dir = selected
      }
      await saveConfig()
    }
  } catch (e) {
    console.error('选择目录失败:', e)
  }
}

function startRecordingShortcut() {
  recordingShortcut.value = true
}

function handleShortcutKeydown(e: KeyboardEvent) {
  if (!recordingShortcut.value) return

  e.preventDefault()
  e.stopPropagation()

  const keys: string[] = []
  if (e.ctrlKey) keys.push('Ctrl')
  if (e.shiftKey) keys.push('Shift')
  if (e.altKey) keys.push('Alt')

  // 排除单独的修饰键
  if (!['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
    keys.push(e.key.toUpperCase())

    if (keys.length >= 2) {
      config.value.shortcut = keys.join('+')
      recordingShortcut.value = false
      saveConfig()
    }
  }
}

function navigate(direction: 'up' | 'down') {
  const items = allItems.value
  if (items.length === 0) return

  if (direction === 'down') {
    selectedIndex.value = (selectedIndex.value + 1) % items.length
  } else {
    selectedIndex.value = (selectedIndex.value - 1 + items.length) % items.length
  }

  nextTick(() => {
    const selected = listRef.value?.querySelector('.history-item.selected')
    selected?.scrollIntoView({ block: 'nearest' })
  })
}

function handleKeyDown(e: KeyboardEvent) {
  if (recordingShortcut.value) {
    handleShortcutKeydown(e)
    return
  }

  if (e.key === 'Escape') {
    if (contextMenu.value.visible) {
      hideContextMenu()
    } else if (showSettings.value) {
      showSettings.value = false
    } else if (showDeleteConfirm.value) {
      showDeleteConfirm.value = false
    } else if (showClearConfirm.value) {
      showClearConfirm.value = false
    } else {
      closeWindow()
    }
  }
}

function showContextMenu(e: MouseEvent, item: ClipboardHistory) {
  contextMenu.value = {
    visible: true,
    x: e.clientX,
    y: e.clientY,
    item
  }
}

function hideContextMenu() {
  contextMenu.value.visible = false
  contextMenu.value.item = null
}

function handleContextCopy() {
  if (contextMenu.value.item) {
    handleCopyItem(contextMenu.value.item)
  }
  hideContextMenu()
}

async function handleCopyImagePath() {
  if (contextMenu.value.item && contextMenu.value.item.content_type === 'image') {
    try {
      const path = await clipboardApi.copyImagePath(contextMenu.value.item.id)
      console.log('已复制图片路径:', path)
    } catch (e) {
      console.error('复制图片路径失败:', e)
    }
  }
  hideContextMenu()
  closeWindow()
}

function handleContextPin() {
  if (contextMenu.value.item) {
    togglePin(contextMenu.value.item.id)
  }
  hideContextMenu()
}

function handleContextDelete() {
  if (contextMenu.value.item) {
    deleteItem(contextMenu.value.item.id)
  }
  showDeleteConfirm.value = false
  hideContextMenu()
}

async function closeWindow() {
  try {
    const win = getCurrentWindow()
    await win.hide()
  } catch (e) {
    console.error('关闭窗口失败:', e)
  }
}

// 窗口失去焦点时自动隐藏
async function handleWindowBlur() {
  // 如果正在显示确认框或设置面板，不自动关闭
  if (showDeleteConfirm.value || showClearConfirm.value) {
    return
  }
  await closeWindow()
}

// 窗口获得焦点时刷新数据
async function handleWindowFocus() {
  await loadHistory()
  nextTick(() => {
    searchInputRef.value?.focus()
  })
}

// Lifecycle
onMounted(async () => {
  await loadConfig()
  await loadHistory()
  nextTick(() => {
    searchInputRef.value?.focus()
    containerRef.value?.focus()
  })

  document.addEventListener('click', hideContextMenu)

  // 监听窗口焦点变化
  window.addEventListener('blur', handleWindowBlur)
  window.addEventListener('focus', handleWindowFocus)
})

onUnmounted(() => {
  document.removeEventListener('click', hideContextMenu)
  window.removeEventListener('blur', handleWindowBlur)
  window.removeEventListener('focus', handleWindowFocus)
})

// Watch
watch(activeFilter, () => {
  selectedIndex.value = 0
})

watch(searchQuery, () => {
  selectedIndex.value = 0
})
</script>

<style scoped>
.clipboard-window {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.clipboard-backdrop {
  position: absolute;
  inset: 0;
  background: transparent;
}

.clipboard-modal {
  position: relative;
  width: 450px;
  max-height: 640px;
  background: linear-gradient(145deg, rgba(26, 26, 46, 0.98), rgba(22, 22, 38, 0.98));
  border-radius: 16px;
  border: 1px solid rgba(99, 102, 241, 0.2);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.clipboard-modal.show-settings {
  max-height: 640px;
}

.modal-grid-bg {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(99, 102, 241, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(99, 102, 241, 0.03) 1px, transparent 1px);
  background-size: 20px 20px;
  pointer-events: none;
}

.main-content {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

/* Header */
.modal-header {
  padding: 16px 16px 12px;
  border-bottom: 1px solid rgba(99, 102, 241, 0.1);
}

.search-container {
  display: flex;
  align-items: center;
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 12px;
  padding: 0 12px;
  transition: all 0.2s;
}

.search-container:focus-within {
  border-color: rgba(99, 102, 241, 0.5);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.search-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  margin-right: 8px;
}

.search-icon {
  width: 16px;
  height: 16px;
  color: rgba(148, 163, 184, 0.7);
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 14px;
  color: #e2e8f0;
  padding: 10px 0;
}

.search-input::placeholder {
  color: rgba(148, 163, 184, 0.5);
}

.search-hint span {
  font-size: 10px;
  color: rgba(148, 163, 184, 0.5);
  background: rgba(99, 102, 241, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

/* Tags */
.tags-container {
  display: flex;
  gap: 8px;
  margin-top: 12px;
  flex-wrap: wrap;
}

.tag-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid transparent;
  border-radius: 20px;
  color: rgba(148, 163, 184, 0.8);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.tag-btn:hover {
  background: rgba(99, 102, 241, 0.2);
  color: #e2e8f0;
}

.tag-btn.active {
  background: rgba(99, 102, 241, 0.3);
  border-color: rgba(99, 102, 241, 0.5);
  color: #e2e8f0;
}

/* Body */
.modal-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 8px;
  min-height: 0;
}

.modal-body::-webkit-scrollbar {
  width: 6px;
}

.modal-body::-webkit-scrollbar-track {
  background: transparent;
}

.modal-body::-webkit-scrollbar-thumb {
  background: rgba(99, 102, 241, 0.3);
  border-radius: 3px;
}

/* History sections */
.history-section {
  margin-bottom: 12px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  font-size: 11px;
  color: rgba(148, 163, 184, 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* History items */
.history-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  margin: 4px 0;
  background: rgba(99, 102, 241, 0.05);
  border: 1px solid transparent;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.15s;
}

.history-item:hover {
  background: rgba(99, 102, 241, 0.1);
  border-color: rgba(99, 102, 241, 0.2);
}

.history-item.selected {
  background: rgba(99, 102, 241, 0.15);
  border-color: rgba(99, 102, 241, 0.4);
}

.item-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(99, 102, 241, 0.2);
  border-radius: 8px;
  font-size: 14px;
  flex-shrink: 0;
}

.item-icon.type-image {
  background: rgba(236, 72, 153, 0.2);
}

.item-image-preview {
  width: 48px;
  height: 48px;
  border-radius: 6px;
  overflow: hidden;
  background: rgba(99, 102, 241, 0.1);
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.item-image-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.item-image-preview .image-placeholder {
  font-size: 20px;
  opacity: 0.5;
}

.item-content {
  flex: 1;
  min-width: 0;
}

.item-preview {
  font-size: 13px;
  color: #e2e8f0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.4;
}

.item-time {
  font-size: 11px;
  color: rgba(148, 163, 184, 0.5);
  margin-top: 2px;
}

.item-pin {
  background: transparent;
  border: none;
  font-size: 14px;
  cursor: pointer;
  opacity: 0.3;
  transition: opacity 0.2s;
  padding: 4px;
  flex-shrink: 0;
}

.item-pin:hover,
.item-pin.active {
  opacity: 1;
}

/* Footer */
.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  border-top: 1px solid rgba(99, 102, 241, 0.1);
  background: rgba(0, 0, 0, 0.2);
}

.footer-stats {
  display: flex;
  gap: 12px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
}

.stat-label {
  color: rgba(148, 163, 184, 0.5);
}

.stat-value {
  color: rgba(99, 102, 241, 0.9);
  font-weight: 600;
}

.footer-actions {
  display: flex;
  gap: 8px;
}

.footer-btn {
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid transparent;
  border-radius: 6px;
  padding: 6px 10px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.footer-btn:hover {
  background: rgba(99, 102, 241, 0.2);
  border-color: rgba(99, 102, 241, 0.3);
}

.footer-btn.danger:hover {
  background: rgba(239, 68, 68, 0.2);
  border-color: rgba(239, 68, 68, 0.3);
}

/* Empty & Loading */
.empty-state,
.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: rgba(148, 163, 184, 0.5);
  font-size: 13px;
}

/* Context Menu */
.context-menu {
  position: fixed;
  background: rgba(26, 26, 46, 0.98);
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: 10px;
  padding: 6px;
  min-width: 140px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
  z-index: 1000;
}

.context-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 13px;
  color: #e2e8f0;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.context-item:hover {
  background: rgba(99, 102, 241, 0.2);
}

.context-item.danger:hover {
  background: rgba(239, 68, 68, 0.2);
}

/* Settings Panel */
.settings-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border-bottom: 1px solid rgba(99, 102, 241, 0.1);
}

.back-btn {
  background: rgba(99, 102, 241, 0.1);
  border: none;
  border-radius: 8px;
  padding: 8px 12px;
  color: #e2e8f0;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s;
}

.back-btn:hover {
  background: rgba(99, 102, 241, 0.2);
}

.settings-title {
  font-size: 16px;
  font-weight: 600;
  color: #e2e8f0;
}

.settings-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.settings-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  color: rgba(99, 102, 241, 0.8);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 12px;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid rgba(99, 102, 241, 0.05);
}

.setting-item.column {
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
}

.setting-label {
  font-size: 13px;
  color: #e2e8f0;
}

.setting-hint {
  font-size: 11px;
  color: rgba(148, 163, 184, 0.5);
}

.setting-input {
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 6px;
  padding: 8px 12px;
  color: #e2e8f0;
  font-size: 13px;
  width: 80px;
  text-align: center;
}

.setting-input:focus {
  outline: none;
  border-color: rgba(99, 102, 241, 0.5);
}

.path-input-wrapper {
  display: flex;
  gap: 8px;
  width: 100%;
}

.path-input {
  flex: 1;
  width: auto;
  text-align: left;
}

.browse-btn {
  background: rgba(99, 102, 241, 0.2);
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: 6px;
  padding: 8px 12px;
  color: #e2e8f0;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.browse-btn:hover {
  background: rgba(99, 102, 241, 0.3);
}

.shortcut-input {
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 6px;
  padding: 8px 16px;
  color: #e2e8f0;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.shortcut-input:hover {
  border-color: rgba(99, 102, 241, 0.5);
}

.shortcut-input .recording {
  color: rgba(236, 72, 153, 0.8);
  animation: pulse 1s infinite;
}

.shortcut-display {
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 6px;
  padding: 8px 16px;
  color: rgba(148, 163, 184, 0.8);
  font-size: 13px;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* Switch */
.switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(99, 102, 241, 0.2);
  transition: 0.3s;
  border-radius: 24px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: #e2e8f0;
  transition: 0.3s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: rgba(99, 102, 241, 0.6);
}

input:checked + .slider:before {
  transform: translateX(20px);
}

/* Confirm Dialog */
.confirm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.confirm-dialog {
  background: rgba(26, 26, 46, 0.98);
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: 12px;
  padding: 20px;
  min-width: 280px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
}

.confirm-title {
  font-size: 16px;
  font-weight: 600;
  color: #e2e8f0;
  margin-bottom: 8px;
}

.confirm-message {
  font-size: 13px;
  color: rgba(148, 163, 184, 0.8);
  margin-bottom: 20px;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.confirm-btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.confirm-btn.cancel {
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.2);
  color: #e2e8f0;
}

.confirm-btn.cancel:hover {
  background: rgba(99, 102, 241, 0.2);
}

.confirm-btn.danger {
  background: rgba(239, 68, 68, 0.2);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #ef4444;
}

.confirm-btn.danger:hover {
  background: rgba(239, 68, 68, 0.3);
}
</style>
