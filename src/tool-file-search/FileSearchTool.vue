<template>
  <div class="file-search-tool">
    <!-- 管理员权限提示 -->
    <div class="admin-notice" v-if="needAdminPermission && initCheckDone && !isIndexing && !adminNoticeDismissed">
      <div class="notice-icon">⚠️</div>
      <div class="notice-content">
        <div class="notice-title">建议以管理员身份运行</div>
        <div class="notice-desc">
          当前使用传统搜索模式（较慢）。以管理员身份运行程序可启用 NTFS 快速索引，搜索速度将大幅提升。
        </div>
      </div>
      <button class="notice-dismiss" @click="dismissAdminNotice" title="不再提示">×</button>
    </div>

    <!-- 搜索区域 -->
    <div class="search-header">
      <div class="search-bar">
        <div class="search-icon">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/>
            <path d="m21 21-4.35-4.35"/>
          </svg>
        </div>
        <input
          ref="keywordInputRef"
          type="text"
          v-model="keyword"
          placeholder="搜索文件、文件夹..."
          class="search-input"
          @keydown.enter="handleSearch"
        />
        <!-- 状态指示器 -->
        <div class="search-status" v-if="isIndexing || hasIndex">
          <span class="status-badge indexing" v-if="isIndexing">
            <span class="status-dot pulsing"></span>
            {{ indexProgress?.progress || 0 }}%
          </span>
          <span class="status-badge ready" v-else-if="hasIndex">
            <span class="status-dot"></span>
            {{ formatNumber(indexStats?.totalFiles || 0) }} 文件
          </span>
        </div>
      </div>

      <div class="search-actions">
        <!-- 视图切换 -->
        <div class="view-toggle">
          <button
            class="toggle-btn"
            :class="{ active: viewMode === 'list' }"
            @click="viewMode = 'list'"
            title="列表视图"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="8" y1="6" x2="21" y2="6"/>
              <line x1="8" y1="12" x2="21" y2="12"/>
              <line x1="8" y1="18" x2="21" y2="18"/>
              <line x1="3" y1="6" x2="3.01" y2="6"/>
              <line x1="3" y1="12" x2="3.01" y2="12"/>
              <line x1="3" y1="18" x2="3.01" y2="18"/>
            </svg>
          </button>
          <button
            class="toggle-btn"
            :class="{ active: viewMode === 'grid' }"
            @click="viewMode = 'grid'"
            title="网格视图"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="7" height="7"/>
              <rect x="14" y="3" width="7" height="7"/>
              <rect x="14" y="14" width="7" height="7"/>
              <rect x="3" y="14" width="7" height="7"/>
            </svg>
          </button>
        </div>

        <!-- 高级选项按钮 -->
        <button class="options-btn" @click="showAdvanced = !showAdvanced" title="高级选项">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="4" y1="21" x2="4" y2="14"/>
            <line x1="4" y1="10" x2="4" y2="3"/>
            <line x1="12" y1="21" x2="12" y2="12"/>
            <line x1="12" y1="8" x2="12" y2="3"/>
            <line x1="20" y1="21" x2="20" y2="16"/>
            <line x1="20" y1="12" x2="20" y2="3"/>
            <line x1="1" y1="14" x2="7" y2="14"/>
            <line x1="9" y1="8" x2="15" y2="8"/>
            <line x1="17" y1="16" x2="23" y2="16"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- 高级选项面板 -->
    <div class="advanced-panel" v-show="showAdvanced">
      <div class="option-item">
        <label class="option-label">结果数量</label>
        <select v-model="maxResults" class="option-select">
          <option :value="100">100 条</option>
          <option :value="500">500 条</option>
          <option :value="1000">1000 条</option>
          <option :value="2000">2000 条</option>
        </select>
      </div>

      <div v-if="hasIndex" class="option-item">
        <label class="option-label">驱动器</label>
        <select v-model="selectedDrive" class="option-select">
          <option value="">全部</option>
          <option v-for="drive in availableDrives" :key="drive" :value="drive">
            {{ drive }}:
          </option>
        </select>
      </div>

      <div v-if="!hasIndex" class="option-item path-option">
        <label class="option-label">搜索路径</label>
        <div class="path-input-group">
          <input
            type="text"
            v-model="searchPath"
            placeholder="留空搜索全盘，多路径用分号分隔"
            class="path-input"
          />
          <button class="browse-btn" @click="browsePath" title="选择目录">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- 文件类型筛选 -->
    <div class="category-filters">
      <button
        v-for="cat in categories"
        :key="cat.id"
        class="category-btn"
        :class="{ active: selectedCategory === cat.id }"
        @click="selectedCategory = cat.id"
      >
        <span class="category-icon">{{ cat.icon }}</span>
        {{ cat.label }}
      </button>
    </div>

    <!-- 索引进度条 -->
    <div class="index-progress" v-if="isIndexing">
      <div class="progress-info">
        <span class="progress-text">{{ indexStatusText }}</span>
        <span class="progress-percent">{{ indexProgress?.progress || 0 }}%</span>
      </div>
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: (indexProgress?.progress || 0) + '%' }"></div>
      </div>
    </div>

    <!-- 结果统计 -->
    <div class="results-stats" v-if="hasSearched && !loading">
      <span class="stats-count">
        找到 <strong>{{ filteredResults.length }}</strong> 个文件
      </span>
      <span class="stats-time" v-if="searchTime">({{ searchTime }}ms)</span>
    </div>

    <!-- 列表视图 -->
    <div class="results-container" v-if="filteredResults.length > 0 && viewMode === 'list'">
      <div class="results-table-wrapper">
        <table class="results-table">
          <thead>
            <tr>
              <th class="col-name">名称</th>
              <th class="col-size">大小</th>
              <th class="col-time">修改时间</th>
              <th class="col-actions">操作</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(file, index) in filteredResults"
              :key="index"
              :class="{ selected: selectedFile?.path === file.path }"
              @click="selectFile(file)"
              @dblclick="openFile(file.path)"
            >
              <td class="col-name">
                <div class="file-info">
                  <div class="file-icon-wrapper" :class="getFileColorClass(file)">
                    <span class="file-icon">{{ getFileIcon(file) }}</span>
                  </div>
                  <div class="file-details">
                    <div class="file-name" :title="file.name">{{ file.name }}</div>
                    <div class="file-path" :title="file.path">{{ getDisplayPath(file.path) }}</div>
                  </div>
                </div>
              </td>
              <td class="col-size">
                <span class="file-size">{{ file.isDir ? '-' : formatFileSize(file.size) }}</span>
              </td>
              <td class="col-time">
                <span class="file-time">{{ formatDateTime(file.modifiedTime) }}</span>
              </td>
              <td class="col-actions">
                <div class="action-btns">
                  <button class="action-btn primary" @click.stop="openFile(file.path)" title="打开">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                      <polyline points="15 3 21 3 21 9"/>
                      <line x1="10" y1="14" x2="21" y2="3"/>
                    </svg>
                  </button>
                  <button class="action-btn" @click.stop="openInFolder(file.path)" title="打开文件夹">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                    </svg>
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- 网格视图 -->
    <div class="results-grid" v-else-if="filteredResults.length > 0 && viewMode === 'grid'">
      <div
        v-for="(file, index) in filteredResults"
        :key="index"
        class="grid-item"
        :class="{ selected: selectedFile?.path === file.path }"
        @click="selectFile(file)"
        @dblclick="openFile(file.path)"
      >
        <div class="grid-icon-wrapper" :class="getFileColorClass(file)">
          <span class="grid-icon">{{ getFileIcon(file) }}</span>
        </div>
        <div class="grid-name" :title="file.name">{{ file.name }}</div>
        <div class="grid-size">{{ file.isDir ? '文件夹' : formatFileSize(file.size) }}</div>
        <div class="grid-actions">
          <button class="grid-action-btn" @click.stop="openFile(file.path)" title="打开">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
              <polyline points="15 3 21 3 21 9"/>
              <line x1="10" y1="14" x2="21" y2="3"/>
            </svg>
          </button>
          <button class="grid-action-btn" @click.stop="openInFolder(file.path)" title="打开文件夹">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="!loading && hasSearched" class="empty-state">
      <div class="empty-icon">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
          <path d="M8 11h6"/>
        </svg>
      </div>
      <div class="empty-title">未找到相关文件</div>
      <div class="empty-desc">尝试调整搜索词或筛选条件</div>
    </div>

    <!-- 初始状态 -->
    <div v-else-if="!loading && !hasSearched" class="empty-state initial">
      <div class="empty-icon">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
        </svg>
      </div>
      <div class="empty-title">开始搜索</div>
      <div class="empty-desc">输入关键词自动搜索文件</div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <div class="loading-spinner"></div>
      <div class="loading-text">正在搜索 "{{ keyword }}"...</div>
    </div>

    <!-- 文件详情面板 -->
    <Transition name="slide">
      <div class="detail-panel" v-if="selectedFile">
        <div class="detail-header">
          <h3 class="detail-title">文件详情</h3>
          <button class="detail-close" @click="selectedFile = null">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>

        <div class="detail-content">
          <div class="detail-icon-wrapper" :class="getFileColorClass(selectedFile)">
            <span class="detail-icon">{{ getFileIcon(selectedFile) }}</span>
          </div>
          <div class="detail-name">{{ selectedFile.name }}</div>
          <div class="detail-type">{{ selectedFile.isDir ? '文件夹' : selectedFile.fileType.toUpperCase() }}</div>

          <div class="detail-info">
            <div class="info-row">
              <span class="info-label">大小</span>
              <span class="info-value">{{ selectedFile.isDir ? '-' : formatFileSize(selectedFile.size) }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">修改时间</span>
              <span class="info-value">{{ formatDateTime(selectedFile.modifiedTime) }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">路径</span>
              <span class="info-value path-value" :title="selectedFile.path">{{ selectedFile.path }}</span>
            </div>
          </div>
        </div>

        <div class="detail-actions">
          <button class="detail-action-btn" @click="openFile(selectedFile.path)">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
              <polyline points="15 3 21 3 21 9"/>
              <line x1="10" y1="14" x2="21" y2="3"/>
            </svg>
            打开
          </button>
          <button class="detail-action-btn secondary" @click="openInFolder(selectedFile.path)">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            打开文件夹
          </button>
        </div>
      </div>
    </Transition>
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

// 分类配置
const categories = [
  { id: 'all', label: '全部', icon: '📁' },
  { id: 'document', label: '文档', icon: '📄' },
  { id: 'image', label: '图片', icon: '🖼️' },
  { id: 'audio', label: '音频', icon: '🎵' },
  { id: 'video', label: '视频', icon: '🎬' },
  { id: 'code', label: '代码', icon: '💻' },
  { id: 'archive', label: '压缩包', icon: '📦' },
]

// 文件类型映射
const fileTypeMap: Record<string, string> = {
  // 文档
  doc: 'document', docx: 'document', pdf: 'document', txt: 'document', rtf: 'document',
  xls: 'document', xlsx: 'document', ppt: 'document', pptx: 'document', odt: 'document',
  // 图片
  jpg: 'image', jpeg: 'image', png: 'image', gif: 'image', bmp: 'image',
  webp: 'image', svg: 'image', ico: 'image', psd: 'image',
  // 音频
  mp3: 'audio', wav: 'audio', flac: 'audio', aac: 'audio', ogg: 'audio', wma: 'audio', m4a: 'audio',
  // 视频
  mp4: 'video', avi: 'video', mkv: 'video', mov: 'video', wmv: 'video', flv: 'video', webm: 'video',
  // 代码
  js: 'code', ts: 'code', vue: 'code', jsx: 'code', tsx: 'code', py: 'code',
  java: 'code', c: 'code', cpp: 'code', h: 'code', cs: 'code', go: 'code',
  rs: 'code', rb: 'code', php: 'code', html: 'code', css: 'code', scss: 'code',
  less: 'code', json: 'code', xml: 'code', yaml: 'code', yml: 'code', md: 'code', sql: 'code',
  // 压缩包
  zip: 'archive', rar: 'archive', '7z': 'archive', tar: 'archive', gz: 'archive', bz2: 'archive',
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
const viewMode = ref<'list' | 'grid'>('list')
const selectedCategory = ref('all')
const selectedFile = ref<UnifiedFileResult | null>(null)

// 索引相关状态
const isIndexing = ref(false)
const hasIndex = ref(false)
const indexStats = ref<IndexStats | null>(null)
const indexProgress = ref<IndexProgressEvent | null>(null)
const availableDrives = ref<string[]>([])
const selectedDrive = ref('')
const needAdminPermission = ref(false)
const initCheckDone = ref(false)
const adminNoticeDismissed = ref(false)

// 事件监听器
let unlistenProgress: UnlistenFn | null = null
let unlistenCompleted: UnlistenFn | null = null
let unlistenError: UnlistenFn | null = null

// 防抖搜索定时器
let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

// 过滤后的结果
const filteredResults = computed(() => {
  if (selectedCategory.value === 'all') {
    return results.value
  }
  return results.value.filter(file => {
    const ext = file.fileType.toLowerCase()
    const category = fileTypeMap[ext] || 'other'
    return category === selectedCategory.value || (file.isDir && selectedCategory.value === 'all')
  })
})

// 监听关键词变化，自动搜索（防抖 400ms）
watch(keyword, (newVal) => {
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer)
  }

  const trimmed = newVal.trim()

  if (!trimmed) {
    results.value = []
    hasSearched.value = false
    return
  }

  if (trimmed.length < 2) {
    return
  }

  searchDebounceTimer = setTimeout(() => {
    handleSearch()
  }, 400)
})

// 索引状态文字
const indexStatusText = computed(() => {
  if (!indexProgress.value) return '正在准备索引...'

  const { stage, message: msg, progress } = indexProgress.value

  if (stage === 'error') return msg || '索引出错'
  if (stage === 'starting') return '正在启动索引...'
  if (stage === 'reading') {
    if (msg) return msg
    return `正在读取文件系统...`
  }
  if (stage === 'building') return `正在构建路径...`
  if (stage === 'indexing') return `正在建立索引...`
  if (stage === 'completed') return '索引完成'

  return msg || `正在处理...`
})

// 初始化
async function initialize() {
  try {
    const status = await getIndexStatus()
    isIndexing.value = status.isIndexing
    hasIndex.value = status.stats?.totalFiles ? status.stats.totalFiles > 0 : false
    indexStats.value = status.stats || null
    needAdminPermission.value = !status.hasAdminPrivilege
    initCheckDone.value = true

    availableDrives.value = await getAvailableDrives()

    unlistenProgress = await onIndexProgress((event) => {
      isIndexing.value = true
      indexProgress.value = event
    })

    unlistenCompleted = await onIndexCompleted(async (totalFiles) => {
      isIndexing.value = false
      indexProgress.value = null
      hasIndex.value = true
      indexStats.value = await getIndexStats()
    })

    unlistenError = await onIndexError(async (error) => {
      isIndexing.value = false
      indexProgress.value = null
      if (error.includes('管理员') || error.includes('admin') || error.includes('privilege')) {
        needAdminPermission.value = true
      }
      console.error('索引错误:', error)
    })

    if (!hasIndex.value && !isIndexing.value && !needAdminPermission.value) {
      autoStartIndexing()
    }
  } catch (error) {
    console.error('初始化文件索引状态失败:', error)
    initCheckDone.value = true
  }
}

async function autoStartIndexing() {
  try {
    isIndexing.value = true
    indexProgress.value = { stage: 'starting', progress: 0, message: '准备中...', processed: 0, total: 0 }
    await startIndexing()
  } catch (error: any) {
    isIndexing.value = false
    indexProgress.value = null
    const errStr = error.toString()
    if (errStr.includes('管理员') || errStr.includes('admin') || errStr.includes('privilege')) {
      needAdminPermission.value = true
    }
    console.error('自动索引失败:', error)
  }
}

function dismissAdminNotice() {
  adminNoticeDismissed.value = true
}

async function handleSearch() {
  const trimmedKeyword = keyword.value.trim()
  if (!trimmedKeyword || trimmedKeyword.length < 2) {
    return
  }

  if (loading.value) return

  loading.value = true
  hasSearched.value = false
  results.value = []
  searchTime.value = null
  selectedFile.value = null

  const startTime = Date.now()

  try {
    if (hasIndex.value) {
      const searchResults = await searchIndexedFiles(
        trimmedKeyword,
        selectedDrive.value || undefined,
        maxResults.value
      )
      results.value = searchResults.map(normalizeIndexedResult)
    } else {
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

function selectFile(file: UnifiedFileResult) {
  selectedFile.value = selectedFile.value?.path === file.path ? null : file
}

async function openFile(path: string) {
  try {
    await apiOpenFile(path)
  } catch (error: any) {
    console.error('打开文件失败:', error)
    await message(`打开文件失败: ${error}`, { title: '错误', kind: 'error' })
  }
}

async function openInFolder(path: string) {
  try {
    await apiOpenFileInFolder(path)
  } catch (error: any) {
    console.error('打开文件夹失败:', error)
    await message(`打开文件夹失败: ${error}`, { title: '错误', kind: 'error' })
  }
}

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

function formatNumber(num: number): string {
  return num.toLocaleString('zh-CN')
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'

  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const k = 1024
  const i = Math.floor(Math.log(bytes) / Math.log(k))

  if (i === 0) return bytes + ' B'

  return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + units[i]
}

function formatDateTime(dateStr: string): string {
  if (!dateStr || dateStr === '未知') return '-'
  return dateStr
}

function getDisplayPath(fullPath: string): string {
  const lastSep = Math.max(fullPath.lastIndexOf('\\'), fullPath.lastIndexOf('/'))
  if (lastSep === -1) return fullPath
  return fullPath.substring(0, lastSep)
}

function getFileIcon(file: UnifiedFileResult): string {
  if (file.isDir) return '📁'

  const ext = file.fileType.toLowerCase()
  const category = fileTypeMap[ext]

  switch (category) {
    case 'image': return '🖼️'
    case 'video': return '🎬'
    case 'audio': return '🎵'
    case 'document': return '📄'
    case 'code': return '💻'
    case 'archive': return '📦'
    default: return '📃'
  }
}

function getFileColorClass(file: UnifiedFileResult): string {
  if (file.isDir) return 'color-folder'

  const ext = file.fileType.toLowerCase()
  const category = fileTypeMap[ext]

  switch (category) {
    case 'image': return 'color-image'
    case 'video': return 'color-video'
    case 'audio': return 'color-audio'
    case 'document': return 'color-document'
    case 'code': return 'color-code'
    case 'archive': return 'color-archive'
    default: return 'color-default'
  }
}

onMounted(() => {
  initialize()
  nextTick(() => {
    keywordInputRef.value?.focus()
  })
})

onUnmounted(() => {
  unlistenProgress?.()
  unlistenCompleted?.()
  unlistenError?.()
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
  overflow: hidden;
  position: relative;
}

/* 管理员提示 */
.admin-notice {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 16px;
  margin-bottom: 12px;
  background: color-mix(in srgb, var(--warning) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--warning) 30%, transparent);
  border-radius: 12px;
  position: relative;
}

.notice-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.notice-content {
  flex: 1;
  min-width: 0;
}

.notice-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--warning);
  margin-bottom: 4px;
}

.notice-desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.notice-dismiss {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 20px;
  height: 20px;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--text-dim);
  font-size: 16px;
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.notice-dismiss:hover {
  background: color-mix(in srgb, var(--warning) 20%, transparent);
  color: var(--warning);
}

/* 搜索头部 */
.search-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.search-bar {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 14px;
  color: var(--text-dim);
  display: flex;
  align-items: center;
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 12px 14px 12px 44px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  color: var(--text-primary);
  font-size: 14px;
  transition: all 0.2s;
  box-shadow: inset 0 1px 2px color-mix(in srgb, var(--bg-base) 50%, transparent);
}

.search-input:focus {
  outline: none;
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px var(--accent-glow), inset 0 1px 2px transparent;
  background: var(--bg-base);
}

.search-input::placeholder {
  color: var(--text-dim);
}

.search-status {
  position: absolute;
  right: 12px;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 500;
}

.status-badge.indexing {
  background: color-mix(in srgb, var(--info) 15%, transparent);
  color: var(--info);
}

.status-badge.ready {
  background: color-mix(in srgb, var(--success) 15%, transparent);
  color: var(--success);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}

.status-dot.pulsing {
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.8); }
}

.search-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.view-toggle {
  display: flex;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  padding: 3px;
}

.toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-dim);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.toggle-btn:hover {
  color: var(--text-secondary);
}

.toggle-btn.active {
  background: var(--bg-base);
  color: var(--accent-primary);
  box-shadow: 0 1px 3px color-mix(in srgb, var(--bg-base) 30%, transparent);
}

.options-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border: 1px solid var(--border-default);
  background: var(--bg-surface);
  color: var(--text-secondary);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.options-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--accent-primary);
}

/* 高级选项面板 */
.advanced-panel {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  padding: 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  margin-bottom: 12px;
}

.option-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.option-item.path-option {
  flex: 1;
  min-width: 200px;
}

.option-label {
  font-size: 11px;
  font-weight: 500;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.option-select {
  padding: 8px 12px;
  background: var(--bg-base);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.option-select:focus {
  outline: none;
  border-color: var(--accent-primary);
}

.path-input-group {
  display: flex;
  gap: 8px;
}

.path-input {
  flex: 1;
  padding: 8px 12px;
  background: var(--bg-base);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 13px;
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
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: 1px solid var(--border-default);
  background: var(--bg-base);
  color: var(--text-secondary);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.browse-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--accent-primary);
}

/* 分类筛选 */
.category-filters {
  display: flex;
  gap: 8px;
  padding-bottom: 12px;
  overflow-x: auto;
  scrollbar-width: none;
}

.category-filters::-webkit-scrollbar {
  display: none;
}

.category-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: 1px solid var(--border-default);
  background: var(--bg-surface);
  color: var(--text-secondary);
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
}

.category-btn:hover {
  background: var(--bg-hover);
  border-color: var(--border-hover);
}

.category-btn.active {
  background: var(--accent-primary);
  border-color: var(--accent-primary);
  color: white;
  box-shadow: 0 2px 8px color-mix(in srgb, var(--accent-primary) 40%, transparent);
  transform: scale(1.02);
}

.category-icon {
  font-size: 14px;
}

/* 索引进度 */
.index-progress {
  padding: 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  margin-bottom: 12px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.progress-text {
  font-size: 13px;
  color: var(--text-secondary);
}

.progress-percent {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent-primary);
}

.progress-bar {
  height: 6px;
  background: var(--bg-hover);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-primary), var(--info));
  border-radius: 3px;
  transition: width 0.3s ease;
}

/* 结果统计 */
.results-stats {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  font-size: 13px;
  color: var(--text-secondary);
}

.stats-count strong {
  color: var(--accent-primary);
}

.stats-time {
  color: var(--text-dim);
  font-size: 12px;
}

/* 结果容器 - 列表视图 */
.results-container {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.results-table-wrapper {
  flex: 1;
  overflow: auto;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 12px;
}

.results-table {
  width: 100%;
  min-width: 500px;
  border-collapse: collapse;
}

.results-table thead {
  background: var(--bg-hover);
  position: sticky;
  top: 0;
  z-index: 1;
}

.results-table th {
  padding: 12px 16px;
  text-align: left;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--border-default);
}

.results-table tbody tr {
  border-bottom: 1px solid color-mix(in srgb, var(--border-default) 50%, transparent);
  cursor: pointer;
  transition: all 0.15s;
}

.results-table tbody tr:hover {
  background: color-mix(in srgb, var(--accent-primary) 8%, transparent);
}

.results-table tbody tr.selected {
  background: color-mix(in srgb, var(--accent-primary) 12%, transparent);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent-primary) 30%, transparent);
}

.results-table td {
  padding: 12px 16px;
}

.col-name { width: auto; }
.col-size { width: 80px; }
.col-time { width: 130px; }
.col-actions { width: 80px; }

.file-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.file-icon-wrapper {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  flex-shrink: 0;
}

.file-icon {
  font-size: 20px;
}

/* 文件颜色类 */
.color-folder { background: color-mix(in srgb, var(--warning) 15%, transparent); }
.color-image { background: color-mix(in srgb, #9333ea 15%, transparent); }
.color-video { background: color-mix(in srgb, var(--error) 15%, transparent); }
.color-audio { background: color-mix(in srgb, #ec4899 15%, transparent); }
.color-document { background: color-mix(in srgb, var(--info) 15%, transparent); }
.color-code { background: color-mix(in srgb, var(--success) 15%, transparent); }
.color-archive { background: color-mix(in srgb, var(--warning) 15%, transparent); }
.color-default { background: var(--bg-hover); }

.file-details {
  min-width: 0;
  flex: 1;
}

.file-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-path {
  font-size: 12px;
  color: var(--text-dim);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: 'Consolas', 'Monaco', monospace;
}

.file-size, .file-time {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.action-btns {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s;
}

.results-table tbody tr:hover .action-btns {
  opacity: 1;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: none;
  background: var(--bg-hover);
  color: var(--text-secondary);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.action-btn:hover {
  background: var(--bg-base);
  color: var(--text-primary);
}

.action-btn.primary:hover {
  background: color-mix(in srgb, var(--accent-primary) 20%, transparent);
  color: var(--accent-primary);
}

/* 网格视图 */
.results-grid {
  flex: 1;
  overflow: auto;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 12px;
  padding: 4px;
}

.grid-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.grid-item:hover {
  border-color: var(--accent-primary);
  box-shadow: 0 4px 12px color-mix(in srgb, var(--bg-base) 50%, transparent);
  transform: translateY(-2px);
}

.grid-item.selected {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 2px var(--accent-primary), 0 4px 12px color-mix(in srgb, var(--accent-primary) 20%, transparent);
}

.grid-icon-wrapper {
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 14px;
  margin-bottom: 12px;
  transition: transform 0.2s;
}

.grid-item:hover .grid-icon-wrapper {
  transform: scale(1.1);
}

.grid-icon {
  font-size: 28px;
}

.grid-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  text-align: center;
  width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.grid-size {
  font-size: 11px;
  color: var(--text-dim);
}

.grid-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s;
}

.grid-item:hover .grid-actions {
  opacity: 1;
}

.grid-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  background: var(--bg-base);
  color: var(--text-secondary);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.grid-action-btn:hover {
  background: var(--accent-primary);
  color: white;
}

/* 空状态 */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--text-dim);
}

.empty-state.initial .empty-icon {
  color: var(--text-dim);
}

.empty-icon {
  color: var(--text-dim);
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.empty-desc {
  font-size: 13px;
  color: var(--text-dim);
}

/* 加载状态 */
.loading-state {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, var(--bg-base) 80%, transparent);
  backdrop-filter: blur(4px);
  z-index: 10;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border-default);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.loading-text {
  font-size: 14px;
  color: var(--text-secondary);
}

/* 详情面板 */
.detail-panel {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 280px;
  background: var(--bg-surface);
  border-left: 1px solid var(--border-default);
  display: flex;
  flex-direction: column;
  z-index: 20;
  box-shadow: -4px 0 20px color-mix(in srgb, var(--bg-base) 30%, transparent);
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid var(--border-default);
}

.detail-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.detail-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-dim);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.detail-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.detail-content {
  flex: 1;
  overflow: auto;
  padding: 20px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.detail-icon-wrapper {
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 20px;
  margin-bottom: 16px;
}

.detail-icon {
  font-size: 40px;
}

.detail-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  text-align: center;
  word-break: break-all;
  margin-bottom: 4px;
}

.detail-type {
  font-size: 12px;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 24px;
}

.detail-info {
  width: 100%;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 10px 0;
  border-bottom: 1px solid var(--border-default);
}

.info-row:last-child {
  border-bottom: none;
}

.info-label {
  font-size: 13px;
  color: var(--text-dim);
  flex-shrink: 0;
}

.info-value {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 500;
  text-align: right;
  word-break: break-all;
}

.info-value.path-value {
  font-size: 11px;
  font-family: 'Consolas', 'Monaco', monospace;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detail-actions {
  padding: 16px;
  border-top: 1px solid var(--border-default);
  background: var(--bg-hover);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.detail-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px;
  border: none;
  background: var(--accent-primary);
  color: white;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.detail-action-btn:hover {
  filter: brightness(1.1);
}

.detail-action-btn.secondary {
  background: var(--bg-surface);
  color: var(--text-secondary);
  border: 1px solid var(--border-default);
}

.detail-action-btn.secondary:hover {
  background: var(--bg-base);
  color: var(--text-primary);
  border-color: var(--accent-primary);
}

/* 面板过渡动画 */
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.25s ease;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}

/* 滚动条 */
.results-table-wrapper::-webkit-scrollbar,
.results-grid::-webkit-scrollbar,
.detail-content::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.results-table-wrapper::-webkit-scrollbar-track,
.results-grid::-webkit-scrollbar-track,
.detail-content::-webkit-scrollbar-track {
  background: transparent;
}

.results-table-wrapper::-webkit-scrollbar-thumb,
.results-grid::-webkit-scrollbar-thumb,
.detail-content::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--text-dim) 30%, transparent);
  border-radius: 3px;
}

.results-table-wrapper::-webkit-scrollbar-thumb:hover,
.results-grid::-webkit-scrollbar-thumb:hover,
.detail-content::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--text-dim) 50%, transparent);
}

/* 响应式 */
@media (max-width: 700px) {
  .col-time {
    display: none;
  }

  .detail-panel {
    width: 100%;
  }

  .results-grid {
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  }
}

@media (max-width: 500px) {
  .search-header {
    flex-direction: column;
    align-items: stretch;
  }

  .search-actions {
    justify-content: flex-end;
  }

  .col-size {
    display: none;
  }

  .category-filters {
    gap: 6px;
  }

  .category-btn {
    padding: 6px 12px;
    font-size: 12px;
  }
}
</style>
