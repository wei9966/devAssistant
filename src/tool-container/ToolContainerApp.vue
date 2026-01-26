<template>
  <div class="tool-container-window" @keydown="handleKeyDown" tabindex="0" ref="containerRef">
    <div class="tool-modal">
      <!-- 装饰背景网格 -->
      <div class="modal-grid-bg"></div>

      <!-- Header - 可拖动区域 -->
      <div class="modal-header" data-tauri-drag-region @dblclick="toggleMaximize">
        <div class="header-title">
          <span class="title-icon">{{ currentTool?.icon || '🔧' }}</span>
          <span class="title-text">{{ currentTool?.name || '工具' }}</span>
        </div>
        <div class="header-actions">
          <!-- 固定按钮 -->
          <button
            class="action-btn pin-btn"
            :class="{ pinned: isPinned }"
            @click="togglePin"
            :title="isPinned ? '取消固定' : '固定窗口'"
          >
            {{ isPinned ? '📍' : '📌' }}
          </button>
          <!-- 最大化按钮 -->
          <button
            class="action-btn maximize-btn"
            :class="{ maximized: isMaximized }"
            @click="toggleMaximize"
            :title="isMaximized ? '还原窗口' : '最大化'"
          >
            {{ isMaximized ? '🗗' : '🗖' }}
          </button>
          <!-- 关闭按钮 -->
          <button class="action-btn close-btn" @click="closeWindow" title="关闭 (ESC)">
            ✕
          </button>
        </div>
      </div>

      <!-- Body - 动态工具组件 -->
      <div class="modal-body">
        <component :is="currentToolComponent" v-if="currentToolComponent" />
        <div v-else class="loading-state">
          <span class="loading-spinner">⏳</span>
          <span>加载工具中...</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, markRaw, type Component } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

// 同步导入工具组件（避免开发模式下异步加载慢的问题）
import PortCheckerTool from '../tool-port-checker/PortCheckerTool.vue'
import TextConverterTool from '../tool-text-converter/TextConverterTool.vue'
import FileSearchTool from '../tool-file-search/FileSearchTool.vue'

// 获取当前窗口实例
const appWindow = getCurrentWindow()

// 工具信息接口
interface ToolInfo {
  id: string
  name: string
  icon: string
  component: string
}

// 状态
const containerRef = ref<HTMLElement | null>(null)
const toolId = ref<string>('')
const isPinned = ref(false)
const isMaximized = ref(false)
const currentTool = ref<ToolInfo | null>(null)

// 工具组件映射（使用 markRaw 避免 Vue 响应式代理）
const toolComponents: Record<string, Component> = {
  'PortChecker': markRaw(PortCheckerTool),
  'TextConverter': markRaw(TextConverterTool),
  'FileSearch': markRaw(FileSearchTool)
}

// 当前工具组件
const currentToolComponent = computed(() => {
  if (!currentTool.value) return null
  return toolComponents[currentTool.value.component] || null
})

// 默认工具信息映射（开发时使用）
const defaultToolInfo: Record<string, ToolInfo> = {
  'port-checker': {
    id: 'port-checker',
    name: '端口检查器',
    icon: '🔌',
    component: 'PortChecker'
  },
  'text-converter': {
    id: 'text-converter',
    name: '文本转换器',
    icon: '📝',
    component: 'TextConverter'
  },
  'file-search': {
    id: 'file-search',
    name: '文件搜索',
    icon: '🔍',
    component: 'FileSearch'
  }
}

// 获取工具信息
async function loadToolInfo(id: string) {
  try {
    const tool = await invoke<ToolInfo>('get_tool_by_id', { toolId: id })
    currentTool.value = tool
  } catch (error) {
    console.error('获取工具信息失败:', error)
    // 使用默认工具信息（开发时使用）
    if (defaultToolInfo[id]) {
      currentTool.value = defaultToolInfo[id]
    }
  }
}

// 切换固定状态
async function togglePin() {
  isPinned.value = !isPinned.value
  // 可以调用后端保存固定状态
  try {
    await invoke('set_tool_pinned', { toolId: toolId.value, pinned: isPinned.value })
  } catch (error) {
    console.error('设置固定状态失败:', error)
  }
}

// 切换最大化状态
async function toggleMaximize() {
  try {
    const maximized = await appWindow.isMaximized()
    if (maximized) {
      await appWindow.unmaximize()
      isMaximized.value = false
    } else {
      await appWindow.maximize()
      isMaximized.value = true
    }
  } catch (error) {
    console.error('切换最大化状态失败:', error)
  }
}

// 关闭窗口（不检查置顶状态，直接关闭）
async function closeWindow() {
  try {
    await appWindow.hide()
  } catch (e) {
    console.error('关闭窗口失败:', e)
  }
}

// 尝试关闭窗口（检查置顶状态）
async function tryCloseWindow() {
  // 如果已置顶，不自动关闭
  if (isPinned.value) {
    return
  }
  await closeWindow()
}

// 键盘事件
function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    tryCloseWindow()
  }
}

// 窗口失焦处理
async function handleWindowBlur() {
  // 如果已置顶，不自动关闭
  if (isPinned.value) {
    return
  }
  await closeWindow()
}

// 事件监听器
let unlistenLoadTool: UnlistenFn | null = null
let unlistenFocusChanged: UnlistenFn | null = null
let unlistenResized: UnlistenFn | null = null

// 保存当前工具 ID 到 localStorage
function saveCurrentToolId(id: string) {
  try {
    localStorage.setItem('tool-container-current-tool', id)
  } catch (e) {
    console.error('保存工具 ID 失败:', e)
  }
}

// 从 localStorage 读取工具 ID
function getSavedToolId(): string | null {
  try {
    return localStorage.getItem('tool-container-current-tool')
  } catch (e) {
    return null
  }
}

// 初始化
onMounted(async () => {
  // 优先从 URL 参数获取 toolId，其次从 localStorage，最后使用默认值
  const params = new URLSearchParams(window.location.search)
  const urlToolId = params.get('toolId')
  const savedToolId = getSavedToolId()
  toolId.value = urlToolId || savedToolId || 'port-checker'

  await loadToolInfo(toolId.value)
  saveCurrentToolId(toolId.value)
  containerRef.value?.focus()

  // 同步初始化最大化状态
  try {
    isMaximized.value = await appWindow.isMaximized()
  } catch (e) {
    console.error('获取最大化状态失败:', e)
  }

  // 监听切换工具事件（来自后端）
  unlistenLoadTool = await listen<string>('tool-container:load-tool', async (event) => {
    const newToolId = event.payload
    if (newToolId && newToolId !== toolId.value) {
      toolId.value = newToolId
      await loadToolInfo(newToolId)
      saveCurrentToolId(newToolId)
    }
  })

  // 监听窗口焦点变化（失焦时自动关闭，除非已置顶）
  unlistenFocusChanged = await appWindow.onFocusChanged(({ payload: focused }) => {
    if (!focused) {
      handleWindowBlur()
    }
  })

  // 监听窗口大小变化，同步最大化状态
  unlistenResized = await appWindow.onResized(async () => {
    try {
      isMaximized.value = await appWindow.isMaximized()
    } catch (e) {
      // 忽略错误
    }
  })
})

// 清理
onUnmounted(() => {
  if (unlistenLoadTool) {
    unlistenLoadTool()
  }
  if (unlistenFocusChanged) {
    unlistenFocusChanged()
  }
  if (unlistenResized) {
    unlistenResized()
  }
})
</script>

<style scoped>
.tool-container-window {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
}

.tool-modal {
  width: 100%;
  height: 100%;
  background: linear-gradient(145deg,
    color-mix(in srgb, var(--bg-base, #1a1a2e) 98%, transparent),
    color-mix(in srgb, var(--bg-surface, #161626) 98%, transparent));
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-grid-bg {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(color-mix(in srgb, var(--accent-primary, #6366f1) 3%, transparent) 1px, transparent 1px),
    linear-gradient(90deg, color-mix(in srgb, var(--accent-primary, #6366f1) 3%, transparent) 1px, transparent 1px);
  background-size: 20px 20px;
  pointer-events: none;
}

/* Header */
.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid color-mix(in srgb, var(--border-default, rgba(99, 102, 241, 0.1)) 100%, transparent);
  background: color-mix(in srgb, var(--bg-elevated, rgba(0, 0, 0, 0.2)) 100%, transparent);
  user-select: none;
  flex-shrink: 0;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-icon {
  font-size: 20px;
}

.title-text {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary, #e2e8f0);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.action-btn {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  border-radius: 6px;
  padding: 4px 10px;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.2s;
  line-height: 1;
  color: var(--text-primary, #e2e8f0);
}

.action-btn:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary, #6366f1) 40%, transparent);
}

.pin-btn {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  color: color-mix(in srgb, var(--text-muted, #94a3b8) 80%, transparent);
}

.pin-btn:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 15%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
}

.pin-btn.pinned {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 25%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary, #6366f1) 50%, transparent);
  color: var(--accent-primary, #6366f1);
  box-shadow: 0 0 8px color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
}

.maximize-btn {
  background: color-mix(in srgb, var(--success, #22c55e) 10%, transparent);
  border-color: color-mix(in srgb, var(--success, #22c55e) 20%, transparent);
  color: color-mix(in srgb, var(--success, #22c55e) 80%, transparent);
}

.maximize-btn:hover {
  background: color-mix(in srgb, var(--success, #22c55e) 20%, transparent);
  border-color: color-mix(in srgb, var(--success, #22c55e) 40%, transparent);
  color: var(--success, #22c55e);
}

.maximize-btn.maximized {
  background: color-mix(in srgb, var(--success, #22c55e) 25%, transparent);
  border-color: color-mix(in srgb, var(--success, #22c55e) 50%, transparent);
  color: var(--success, #22c55e);
  box-shadow: 0 0 8px color-mix(in srgb, var(--success, #22c55e) 30%, transparent);
}

.close-btn {
  background: color-mix(in srgb, #ef4444 10%, transparent);
  border-color: color-mix(in srgb, #ef4444 20%, transparent);
  color: #ef4444;
}

.close-btn:hover {
  background: color-mix(in srgb, #ef4444 20%, transparent);
  border-color: color-mix(in srgb, #ef4444 40%, transparent);
}

/* Body */
.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.modal-body::-webkit-scrollbar {
  width: 6px;
}

.modal-body::-webkit-scrollbar-track {
  background: transparent;
}

.modal-body::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  border-radius: 3px;
}

/* Loading State */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: color-mix(in srgb, var(--text-muted, #94a3b8) 60%, transparent);
  font-size: 14px;
  gap: 12px;
  flex: 1;
}

.loading-spinner {
  font-size: 32px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
