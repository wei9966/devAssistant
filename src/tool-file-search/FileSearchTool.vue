<template>
  <div class="file-search-tool">
    <!-- 搜索区域 -->
    <div class="search-section">
      <div class="search-row">
        <div class="input-group keyword-group">
          <label class="input-label">搜索关键词</label>
          <div class="input-wrapper">
            <input
              ref="keywordInputRef"
              type="text"
              v-model="keyword"
              placeholder="输入关键词自动搜索..."
              class="search-input"
              @keydown.enter="handleSearch"
            />
            <button class="search-btn" @click="handleSearch" :disabled="loading || !keyword.trim()">
              <span v-if="loading" class="loading-spinner"></span>
              <span v-else class="search-icon"></span>
              搜索
            </button>
          </div>
          <!-- 状态指示器 -->
          <div class="status-indicator" v-if="isIndexing">
            <span class="status-text indexing">
              <span class="status-dot pulsing"></span>
              {{ indexStatusText }}
            </span>
            <!-- 进度条 -->
            <div class="mini-progress" v-if="indexProgress && indexProgress.progress > 0">
              <div class="mini-progress-bar" :style="{ width: indexProgress.progress + '%' }"></div>
            </div>
          </div>
          <div class="status-indicator" v-else-if="!hasIndex">
            <span class="status-text preparing">
              <span class="status-dot" :class="{ 'pulsing': !needAdminPermission }"></span>
              {{ noIndexStatusText }}
            </span>
          </div>
          <div class="status-indicator ready" v-else-if="hasIndex && indexStats">
            <span class="status-dot ready-dot"></span>
            <span class="ready-text">就绪 - {{ formatNumber(indexStats.totalFiles) }} 文件</span>
          </div>
        </div>
      </div>

      <!-- 高级选项（折叠） -->
      <div class="advanced-options">
        <button class="toggle-options-btn" @click="showAdvanced = !showAdvanced">
          <span class="toggle-icon" :class="{ expanded: showAdvanced }"></span>
          高级选项
        </button>
        <div class="options-content" v-show="showAdvanced">
          <div class="option-group">
            <label class="option-label">结果数量</label>
            <select v-model="maxResults" class="option-select">
              <option :value="100">100 条</option>
              <option :value="500">500 条</option>
              <option :value="1000">1000 条</option>
              <option :value="2000">2000 条</option>
            </select>
          </div>

          <div v-if="hasIndex" class="option-group">
            <label class="option-label">限定驱动器</label>
            <select v-model="selectedDrive" class="option-select">
              <option value="">全部驱动器</option>
              <option v-for="drive in availableDrives" :key="drive" :value="drive">
                {{ drive }}:
              </option>
            </select>
          </div>

          <div v-if="!hasIndex" class="option-group path-group">
            <label class="option-label">搜索路径</label>
            <div class="path-input-wrapper">
              <input
                type="text"
                v-model="searchPath"
                placeholder="留空则搜索全盘，多路径用分号分隔"
                class="path-input"
              />
              <button class="browse-btn" @click="browsePath" title="选择目录">
                <span class="folder-icon"></span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 结果统计 -->
    <div class="stats-bar" v-if="hasSearched && !loading">
      <span class="stats-text">
        找到 <strong>{{ results.length }}</strong> 个文件
        <span v-if="searchTime" class="search-time">(耗时 {{ searchTime }}ms)</span>
      </span>
    </div>

    <!-- 结果表格 -->
    <div class="result-section" v-if="results.length > 0">
      <div class="result-table-wrapper">
        <table class="result-table">
          <thead>
            <tr>
              <th class="col-icon"></th>
              <th class="col-name">文件名</th>
              <th class="col-path">路径</th>
              <th class="col-size">大小</th>
              <th class="col-time">修改时间</th>
              <th class="col-actions">操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(file, index) in results" :key="index" @dblclick="openFile(file.path)">
              <td class="col-icon">
                <span class="file-icon">{{ getFileIcon(file) }}</span>
              </td>
              <td class="col-name">
                <span class="file-name" :title="file.name">{{ file.name }}</span>
              </td>
              <td class="col-path">
                <span class="file-path" :title="file.path">{{ getDisplayPath(file.path) }}</span>
              </td>
              <td class="col-size">
                <span class="file-size">{{ file.isDir ? '-' : formatFileSize(file.size) }}</span>
              </td>
              <td class="col-time">
                <span class="file-time">{{ formatDateTime(file.modifiedTime) }}</span>
              </td>
              <td class="col-actions">
                <div class="action-buttons">
                  <button
                    class="action-btn open-btn"
                    @click.stop="openFile(file.path)"
                    title="打开文件"
                  >
                    打开
                  </button>
                  <button
                    class="action-btn folder-btn"
                    @click.stop="openInFolder(file.path)"
                    title="在文件夹中显示"
                  >
                    文件夹
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="!loading && hasSearched" class="empty-state">
      <span class="empty-icon"></span>
      <span class="empty-text">未找到匹配的文件</span>
      <span class="empty-hint">尝试使用其他关键词或更改搜索路径</span>
    </div>

    <!-- 初始状态 -->
    <div v-else-if="!loading && !hasSearched" class="initial-state">
      <span class="initial-icon"></span>
      <span class="initial-text">输入关键词开始搜索</span>
      <span class="initial-hint">支持文件名模糊匹配</span>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <span class="loading-spinner large"></span>
      <span class="loading-text">正在搜索 "{{ keyword }}"...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { message } from '@tauri-apps/plugin-dialog'
import { open } from '@tauri-apps/plugin-dialog'
import {
  searchFiles as apiSearchFiles,
  openFile as apiOpenFile,
  openFileInFolder as apiOpenFileInFolder,
  type FileSearchResult
} from '@/api/fileSearchApi'
import {
  getIndexStatus,
  getIndexStats,
  startIndexing,
  searchIndexedFiles,
  getAvailableDrives,
  onIndexProgress,
  onIndexCompleted,
  onIndexError,
  type IndexStats,
  type IndexProgressEvent,
  type FileIndexRecord
} from '@/api/fileIndexApi'
import type { UnlistenFn } from '@tauri-apps/api/event'

// 统一的文件结果类型
interface UnifiedFileResult {
  name: string
  path: string
  size: number
  modifiedTime: string
  isDir: boolean
  fileType: string
}

// Refs
const keywordInputRef = ref<HTMLInputElement | null>(null)

// State
const keyword = ref('')
const loading = ref(false)
const results = ref<UnifiedFileResult[]>([])
const hasSearched = ref(false)
const maxResults = ref(500)
const searchPath = ref('')
const searchTime = ref<number | null>(null)
const showAdvanced = ref(false)

// 索引相关状态
const isIndexing = ref(false)
const hasIndex = ref(false)
const indexStats = ref<IndexStats | null>(null)
const indexProgress = ref<IndexProgressEvent | null>(null)
const availableDrives = ref<string[]>([])
const selectedDrive = ref('')
const needAdminPermission = ref(false)
const initCheckDone = ref(false)

// 事件监听器
let unlistenProgress: UnlistenFn | null = null
let unlistenCompleted: UnlistenFn | null = null
let unlistenError: UnlistenFn | null = null

// 防抖搜索定时器
let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

// 监听关键词变化，自动搜索（防抖 400ms）
watch(keyword, (newVal) => {
  // 清除之前的定时器
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer)
  }

  const trimmed = newVal.trim()

  // 如果关键词为空，清空结果
  if (!trimmed) {
    results.value = []
    hasSearched.value = false
    return
  }

  // 至少输入2个字符才触发搜索
  if (trimmed.length < 2) {
    return
  }

  // 设置防抖定时器
  searchDebounceTimer = setTimeout(() => {
    handleSearch()
  }, 400)
})

// 索引状态文字
const indexStatusText = computed(() => {
  if (!indexProgress.value) return '正在准备搜索...'

  const { stage, message: msg, progress } = indexProgress.value

  if (stage === 'starting') return '正在启动索引...'
  if (stage === 'reading') return `正在读取文件系统... ${progress}%`
  if (stage === 'building') return `正在构建路径... ${progress}%`
  if (stage === 'indexing') return `正在建立索引... ${progress}%`
  if (stage === 'completed') return '索引完成'

  return msg || '正在准备搜索...'
})

// 无索引状态文字
const noIndexStatusText = computed(() => {
  if (!initCheckDone.value) return '正在检查...'
  if (needAdminPermission.value) return '使用传统搜索模式（管理员运行可启用快速索引）'
  return '正在准备索引...'
})

// 初始化
async function initialize() {
  try {
    // 获取索引状态
    const status = await getIndexStatus()
    isIndexing.value = status.isIndexing
    hasIndex.value = status.stats?.totalFiles ? status.stats.totalFiles > 0 : false
    indexStats.value = status.stats || null
    needAdminPermission.value = !status.hasAdminPrivilege
    initCheckDone.value = true

    // 获取可用驱动器
    availableDrives.value = await getAvailableDrives()

    // 设置事件监听器
    unlistenProgress = await onIndexProgress((event) => {
      isIndexing.value = true
      indexProgress.value = event
    })

    unlistenCompleted = await onIndexCompleted(async (totalFiles) => {
      isIndexing.value = false
      indexProgress.value = null
      hasIndex.value = true
      // 刷新统计信息
      indexStats.value = await getIndexStats()
    })

    unlistenError = await onIndexError(async (error) => {
      isIndexing.value = false
      indexProgress.value = null
      // 检查是否是权限问题
      if (error.includes('管理员') || error.includes('admin') || error.includes('privilege')) {
        needAdminPermission.value = true
      }
      console.error('索引错误:', error)
    })

    // 如果没有索引且有管理员权限，自动开始建立索引
    if (!hasIndex.value && !isIndexing.value && !needAdminPermission.value) {
      autoStartIndexing()
    }
  } catch (error) {
    console.error('初始化文件索引状态失败:', error)
    initCheckDone.value = true
  }
}

// 自动开始建立索引（无提示，后台静默进行）
async function autoStartIndexing() {
  try {
    isIndexing.value = true
    indexProgress.value = { stage: 'starting', progress: 0, message: '准备中...', processed: 0, total: 0 }
    await startIndexing()
  } catch (error: any) {
    isIndexing.value = false
    indexProgress.value = null
    // 检查是否是权限问题
    const errStr = error.toString()
    if (errStr.includes('管理员') || errStr.includes('admin') || errStr.includes('privilege')) {
      needAdminPermission.value = true
    }
    console.error('自动索引失败:', error)
  }
}


// 搜索方法
async function handleSearch() {
  const trimmedKeyword = keyword.value.trim()
  if (!trimmedKeyword || trimmedKeyword.length < 2) {
    return
  }

  // 如果正在搜索，不重复触发
  if (loading.value) return

  loading.value = true
  hasSearched.value = false
  results.value = []
  searchTime.value = null

  const startTime = Date.now()

  try {
    // 优先使用索引搜索（如果有索引）
    if (hasIndex.value) {
      const searchResults = await searchIndexedFiles(
        trimmedKeyword,
        selectedDrive.value || undefined,
        maxResults.value
      )
      results.value = searchResults.map(normalizeIndexedResult)
    } else {
      // 回退到传统搜索
      const paths = searchPath.value
        .split(';')
        .map(p => p.trim())
        .filter(p => p.length > 0)

      const searchResults = await apiSearchFiles(trimmedKeyword, paths, maxResults.value)
      results.value = searchResults.map(normalizeWalkdirResult)
    }

    hasSearched.value = true
    searchTime.value = Date.now() - startTime
  } catch (error: any) {
    console.error('搜索文件失败:', error)

    if (error.toString().includes('command') && error.toString().includes('not found')) {
      await message('搜索功能需要后端支持', { title: '功能未实现', kind: 'warning' })
    } else {
      await message(`搜索失败: ${error}`, { title: '错误', kind: 'error' })
    }
    hasSearched.value = true
  } finally {
    loading.value = false
  }
}

// 统一索引搜索结果格式
function normalizeIndexedResult(record: FileIndexRecord): UnifiedFileResult {
  return {
    name: record.name,
    path: record.path,
    size: record.size,
    modifiedTime: record.modifiedTime,
    isDir: record.isDir,
    fileType: record.fileType
  }
}

// 统一 walkdir 搜索结果格式
function normalizeWalkdirResult(result: FileSearchResult): UnifiedFileResult {
  return {
    name: result.name,
    path: result.path,
    size: result.size,
    modifiedTime: result.modified_time,
    isDir: result.is_dir,
    fileType: result.file_type
  }
}

// 打开文件
async function openFile(path: string) {
  try {
    await apiOpenFile(path)
  } catch (error: any) {
    console.error('打开文件失败:', error)
    if (error.toString().includes('command open_file not found')) {
      await message('打开文件功能需要后端支持', { title: '功能未实现', kind: 'warning' })
    } else {
      await message(`打开文件失败: ${error}`, { title: '错误', kind: 'error' })
    }
  }
}

// 在文件夹中打开
async function openInFolder(path: string) {
  try {
    await apiOpenFileInFolder(path)
  } catch (error: any) {
    console.error('打开文件夹失败:', error)
    if (error.toString().includes('command open_file_in_folder not found')) {
      await message('打开文件夹功能需要后端支持', { title: '功能未实现', kind: 'warning' })
    } else {
      await message(`打开文件夹失败: ${error}`, { title: '错误', kind: 'error' })
    }
  }
}

// 浏览选择路径
async function browsePath() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择搜索目录'
    })
    if (selected) {
      if (searchPath.value.trim()) {
        searchPath.value = searchPath.value + ';' + selected
      } else {
        searchPath.value = selected as string
      }
    }
  } catch (error) {
    console.error('选择目录失败:', error)
  }
}

// 格式化数字（添加千分位）
function formatNumber(num: number): string {
  return num.toLocaleString('zh-CN')
}

// 格式化文件大小
function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'

  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const k = 1024
  const i = Math.floor(Math.log(bytes) / Math.log(k))

  if (i === 0) return bytes + ' B'

  return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + units[i]
}

// 格式化日期时间
function formatDateTime(dateStr: string): string {
  if (!dateStr || dateStr === '未知') return '-'
  return dateStr
}

// 获取显示路径（截取目录部分）
function getDisplayPath(fullPath: string): string {
  const lastSep = Math.max(fullPath.lastIndexOf('\\'), fullPath.lastIndexOf('/'))
  if (lastSep === -1) return fullPath
  return fullPath.substring(0, lastSep)
}

// 获取文件图标
function getFileIcon(file: UnifiedFileResult): string {
  if (file.isDir) return '\u{1F4C1}'

  const ext = file.fileType.toLowerCase()

  if (['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg', 'ico'].includes(ext)) {
    return '\u{1F5BC}'
  }
  if (['mp4', 'avi', 'mkv', 'mov', 'wmv', 'flv', 'webm'].includes(ext)) {
    return '\u{1F3AC}'
  }
  if (['mp3', 'wav', 'flac', 'aac', 'ogg', 'wma', 'm4a'].includes(ext)) {
    return '\u{1F3B5}'
  }
  if (['doc', 'docx', 'pdf', 'txt', 'rtf', 'odt'].includes(ext)) {
    return '\u{1F4C4}'
  }
  if (['xls', 'xlsx', 'csv'].includes(ext)) {
    return '\u{1F4CA}'
  }
  if (['zip', 'rar', '7z', 'tar', 'gz', 'bz2'].includes(ext)) {
    return '\u{1F4E6}'
  }
  if (['js', 'ts', 'vue', 'jsx', 'tsx', 'py', 'java', 'c', 'cpp', 'h', 'cs', 'go', 'rs', 'rb', 'php', 'html', 'css', 'scss', 'less', 'json', 'xml', 'yaml', 'yml', 'md', 'sql'].includes(ext)) {
    return '\u{1F4BB}'
  }
  if (['exe', 'msi', 'bat', 'cmd', 'sh'].includes(ext)) {
    return '\u{2699}'
  }

  return '\u{1F4C3}'
}

// Lifecycle
onMounted(() => {
  initialize()
  nextTick(() => {
    keywordInputRef.value?.focus()
  })
})

onUnmounted(() => {
  // 清理事件监听器
  unlistenProgress?.()
  unlistenCompleted?.()
  unlistenError?.()
  // 清理防抖定时器
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer)
  }
})
</script>

<style scoped>
.file-search-tool {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

/* Search Section */
.search-section {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.search-row {
  display: flex;
  gap: 12px;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.keyword-group {
  flex: 1;
}

.input-label,
.option-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.input-wrapper {
  display: flex;
  gap: 8px;
}

.search-input {
  flex: 1;
  background: var(--input-bg);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  padding: 10px 14px;
  color: var(--text-primary);
  font-size: 14px;
  transition: all 0.2s;
}

.search-input:focus {
  outline: none;
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px var(--accent-glow);
}

.search-input::placeholder {
  color: var(--text-dim);
}

.search-btn {
  background: var(--accent-glow);
  border: 1px solid color-mix(in srgb, var(--accent-primary) 30%, transparent);
  border-radius: 8px;
  padding: 10px 20px;
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  display: flex;
  align-items: center;
  gap: 6px;
}

.search-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent-primary) 30%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary) 50%, transparent);
}

.search-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.search-icon::before {
  content: '';
  display: inline-block;
  width: 14px;
  height: 14px;
  background: currentColor;
  mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'%3E%3Ccircle cx='11' cy='11' r='8'/%3E%3Cpath d='m21 21-4.35-4.35'/%3E%3C/svg%3E");
  mask-size: contain;
  mask-repeat: no-repeat;
}

/* Status Indicator */
.status-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.status-indicator.ready {
  color: var(--text-dim);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--info);
  flex-shrink: 0;
}

.status-dot.pulsing {
  animation: pulse 1.5s ease-in-out infinite;
}

.status-dot.ready-dot {
  background: var(--success);
  animation: none;
}

.status-text {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-text.indexing {
  color: var(--info);
}

.status-text.preparing {
  color: var(--text-secondary);
}

.ready-text {
  color: var(--text-dim);
}

/* Mini Progress Bar */
.mini-progress {
  flex: 1;
  max-width: 120px;
  height: 4px;
  background: var(--bg-hover);
  border-radius: 2px;
  overflow: hidden;
  margin-left: 8px;
}

.mini-progress-bar {
  height: 100%;
  background: var(--info);
  border-radius: 2px;
  transition: width 0.3s ease;
}


@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.5;
    transform: scale(0.85);
  }
}

/* Advanced Options */
.advanced-options {
  margin-top: 4px;
}

.toggle-options-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: none;
  padding: 4px 0;
  font-size: 12px;
  color: var(--text-dim);
  cursor: pointer;
  transition: color 0.2s;
}

.toggle-options-btn:hover {
  color: var(--text-secondary);
}

.toggle-icon {
  display: inline-block;
  width: 12px;
  height: 12px;
  transition: transform 0.2s;
}

.toggle-icon::before {
  content: '';
  display: block;
  width: 100%;
  height: 100%;
  background: currentColor;
  mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'%3E%3Cpath d='m9 18 6-6-6-6'/%3E%3C/svg%3E");
  mask-size: contain;
  mask-repeat: no-repeat;
}

.toggle-icon.expanded {
  transform: rotate(90deg);
}

.options-content {
  display: flex;
  gap: 16px;
  align-items: flex-end;
  padding: 8px 0;
  flex-wrap: wrap;
}

.option-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.path-group {
  flex: 1;
  min-width: 200px;
}

.option-select {
  background: var(--input-bg);
  border: 1px solid var(--border-default);
  border-radius: 6px;
  padding: 6px 10px;
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.option-select:focus {
  outline: none;
  border-color: var(--accent-primary);
}

.path-input-wrapper {
  display: flex;
  gap: 6px;
}

.path-input {
  flex: 1;
  background: var(--input-bg);
  border: 1px solid var(--border-default);
  border-radius: 6px;
  padding: 6px 10px;
  color: var(--text-primary);
  font-size: 12px;
  transition: all 0.2s;
}

.path-input:focus {
  outline: none;
  border-color: var(--accent-primary);
}

.path-input::placeholder {
  color: var(--text-dim);
}

.browse-btn {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 6px;
  padding: 6px 10px;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.browse-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent-primary);
  color: var(--text-primary);
}

.folder-icon::before {
  content: '';
  display: inline-block;
  width: 14px;
  height: 14px;
  background: currentColor;
  mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'%3E%3Cpath d='M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z'/%3E%3C/svg%3E");
  mask-size: contain;
  mask-repeat: no-repeat;
}

/* Stats Bar */
.stats-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  margin-top: 8px;
  font-size: 13px;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border-default);
}

.stats-text strong {
  color: var(--accent-primary);
}

.search-time {
  color: var(--text-dim);
  font-size: 12px;
}

/* Result Section */
.result-section {
  margin-top: 8px;
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.result-table-wrapper {
  flex: 1;
  overflow: auto;
  border-radius: 8px;
  border: 1px solid var(--border-default);
  min-height: 0;
}

.result-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.result-table thead {
  background: var(--bg-surface);
  position: sticky;
  top: 0;
  z-index: 1;
}

.result-table th {
  padding: 10px 12px;
  text-align: left;
  font-weight: 600;
  color: var(--text-muted);
  border-bottom: 1px solid var(--border-default);
  white-space: nowrap;
}

.result-table tbody tr {
  border-bottom: 1px solid var(--border-default);
  transition: background 0.15s;
  cursor: pointer;
}

.result-table tbody tr:hover {
  background: var(--bg-hover);
}

.result-table td {
  padding: 8px 12px;
  color: var(--text-primary);
}

/* Column widths */
.col-icon {
  width: 40px;
  text-align: center;
}

.col-name {
  min-width: 180px;
  max-width: 250px;
}

.col-path {
  min-width: 200px;
}

.col-size {
  width: 80px;
  text-align: right;
}

.col-time {
  width: 140px;
}

.col-actions {
  width: 140px;
}

/* File info */
.file-icon {
  font-size: 18px;
}

.file-name {
  font-weight: 500;
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-path {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 11px;
  color: var(--text-secondary);
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-size {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.file-time {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
}

/* Action buttons */
.action-buttons {
  display: flex;
  gap: 6px;
}

.action-btn {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--text-secondary);
}

.action-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent-primary);
  color: var(--text-primary);
}

.open-btn:hover {
  background: color-mix(in srgb, var(--info) 15%, transparent);
  border-color: var(--info);
  color: var(--info);
}

.folder-btn:hover {
  background: color-mix(in srgb, var(--warning) 15%, transparent);
  border-color: var(--warning);
  color: var(--warning);
}

/* Empty, Initial & Loading States */
.empty-state,
.initial-state,
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-muted);
  font-size: 14px;
  gap: 8px;
  flex: 1;
}

.empty-icon::before,
.initial-icon::before {
  content: '';
  display: block;
  width: 48px;
  height: 48px;
  margin-bottom: 8px;
  background: var(--text-dim);
  mask-size: contain;
  mask-repeat: no-repeat;
  mask-position: center;
}

.empty-icon::before {
  mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.5'%3E%3Ccircle cx='11' cy='11' r='8'/%3E%3Cpath d='m21 21-4.35-4.35'/%3E%3Cpath d='M8 11h6'/%3E%3C/svg%3E");
}

.initial-icon::before {
  mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.5'%3E%3Cpath d='M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z'/%3E%3C/svg%3E");
}

.empty-text,
.initial-text,
.loading-text {
  font-size: 16px;
  color: var(--text-secondary);
}

.empty-hint,
.initial-hint {
  font-size: 13px;
  color: var(--text-dim);
}

/* Loading Spinner */
.loading-spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid var(--text-dim);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.loading-spinner.large {
  width: 36px;
  height: 36px;
  border-width: 3px;
  margin-bottom: 12px;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Scrollbar */
.result-table-wrapper::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.result-table-wrapper::-webkit-scrollbar-track {
  background: color-mix(in srgb, var(--bg-surface) 30%, transparent);
  border-radius: 4px;
}

.result-table-wrapper::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--accent-primary) 30%, transparent);
  border-radius: 4px;
  transition: background 0.2s;
}

.result-table-wrapper::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--accent-primary) 50%, transparent);
}

/* Responsive */
@media (max-width: 768px) {
  .options-content {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }

  .col-path {
    display: none;
  }

  .col-time {
    display: none;
  }

  .action-buttons {
    flex-direction: column;
    gap: 4px;
  }
}
</style>
