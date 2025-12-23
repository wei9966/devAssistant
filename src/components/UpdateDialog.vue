<template>
  <n-modal
    v-model:show="modalVisible"
    :mask-closable="false"
    :close-on-esc="false"
    class="update-dialog"
    transform-origin="center"
  >
    <div class="update-container">
      <!-- 主内容区 -->
      <div class="update-content">
        <!-- 顶部标题区 -->
        <div class="header-section">
          <div class="icon-wrapper">
            <n-icon size="32" color="#fff">
              <RocketOutline />
            </n-icon>
          </div>
          <div class="header-text">
            <h2 class="title">发现新版本</h2>
            <p class="subtitle">DevAssistant 有可用更新</p>
          </div>
        </div>

        <!-- 版本信息区 -->
        <div class="version-section">
          <div class="version-badge">
            <span class="version-label">新版本</span>
            <span class="version-number">v{{ version }}</span>
          </div>
          <div class="version-date">
            <n-icon size="14"><TimeOutline /></n-icon>
            <span>{{ formatDate(date) }}</span>
          </div>
        </div>

        <!-- 更新日志区 -->
        <div class="changelog-section">
          <div class="changelog-header">
            <n-icon size="16"><DocumentTextOutline /></n-icon>
            <span>更新内容</span>
          </div>
          <div class="changelog-content" v-html="renderedNotes"></div>
        </div>

        <!-- 下载进度区 (仅在下载时显示) -->
        <div v-if="isDownloading" class="progress-section">
          <div class="progress-header">
            <div class="progress-info">
              <n-icon size="16" class="spinner"><SyncOutline /></n-icon>
              <span>正在下载更新...</span>
            </div>
            <span class="progress-percent">{{ downloadProgress }}%</span>
          </div>
          <n-progress
            type="line"
            :percentage="downloadProgress"
            :show-indicator="false"
            :height="6"
            :border-radius="3"
            :color="progressColor"
            :rail-color="'var(--bg-hover)'"
          />
        </div>

        <!-- 底部操作区 -->
        <div class="actions-section">
          <div class="action-buttons">
            <button
              class="btn-secondary"
              @click="handleSkip"
              :disabled="isDownloading"
              title="不再提醒此版本"
            >
              <n-icon size="16"><CloseCircleOutline /></n-icon>
              跳过此版本
            </button>
            <button
              class="btn-tertiary"
              @click="handleLater"
              :disabled="isDownloading"
              title="稍后在设置中更新"
            >
              <n-icon size="16"><TimeOutline /></n-icon>
              稍后提醒
            </button>
            <button
              class="btn-primary"
              @click="handleUpdate"
              :disabled="isDownloading"
              :class="{ downloading: isDownloading }"
            >
              <n-icon size="16" :class="{ spinner: isDownloading }">
                <DownloadOutline v-if="!isDownloading" />
                <SyncOutline v-else />
              </n-icon>
              {{ isDownloading ? '下载中...' : '立即更新' }}
            </button>
          </div>
        </div>
      </div>

      <!-- 底部提示 -->
      <div class="footer-hint">
        <div class="hint-item">
          <span class="status-dot"></span>
          <span>更新过程中请勿关闭应用</span>
        </div>
      </div>
    </div>
  </n-modal>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NModal, NIcon, NProgress } from 'naive-ui'
import {
  RocketOutline,
  TimeOutline,
  DocumentTextOutline,
  DownloadOutline,
  CloseCircleOutline,
  SyncOutline,
} from '@vicons/ionicons5'
import { marked } from 'marked'

const props = withDefaults(
  defineProps<{
    show: boolean
    version: string
    notes: string
    date: string
    downloading?: boolean
    progress?: number
  }>(),
  {
    show: false,
    version: '',
    notes: '',
    date: '',
    downloading: false,
    progress: 0
  }
)

const emit = defineEmits<{
  'update:show': [value: boolean]
  'update': []
  'later': []
  'skip': []
}>()

// 计算属性
const modalVisible = computed({
  get: () => props.show,
  set: (val) => emit('update:show', val)
})

// 使用 props 中的下载状态
const isDownloading = computed(() => props.downloading)
const downloadProgress = computed(() => Math.round(props.progress))

// 渲染 Markdown 内容
const renderedNotes = computed(() => {
  if (!props.notes) return ''

  // 配置 marked
  marked.setOptions({
    breaks: true,
    gfm: true,
  })

  return marked(props.notes)
})

// 进度条颜色
const progressColor = computed(() => {
  if (downloadProgress.value < 30) return '#6366f1' // indigo-500
  if (downloadProgress.value < 70) return '#8b5cf6' // purple-500
  return '#22c55e' // green-500
})

// 方法
const formatDate = (dateStr: string): string => {
  if (!dateStr) return ''

  try {
    const date = new Date(dateStr)
    const year = date.getFullYear()
    const month = String(date.getMonth() + 1).padStart(2, '0')
    const day = String(date.getDate()).padStart(2, '0')
    return `${year}-${month}-${day}`
  } catch {
    return dateStr
  }
}

const handleUpdate = () => {
  if (isDownloading.value) return
  emit('update')
}

const handleLater = () => {
  if (isDownloading.value) return
  emit('later')
  modalVisible.value = false
}

const handleSkip = () => {
  if (isDownloading.value) return
  emit('skip')
  modalVisible.value = false
}
</script>

<style scoped>
.update-dialog {
  display: flex;
  align-items: center;
  justify-content: center;
}

.update-dialog :deep(.n-modal) {
  max-width: 600px;
  width: 90%;
  margin: 0;
  background: transparent;
  box-shadow: none;
}

.update-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.update-content {
  width: 100%;
  background: var(--card-bg);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: 20px;
  box-shadow:
    0 25px 50px -12px rgba(0, 0, 0, 0.5),
    0 0 0 1px rgba(99, 102, 241, 0.2),
    0 0 60px rgba(99, 102, 241, 0.15);
  overflow: hidden;
}

/* 顶部标题区 */
.header-section {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 24px 28px;
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.1) 0%, rgba(139, 92, 246, 0.1) 100%);
  border-bottom: 1px solid var(--border-default);
}

.icon-wrapper {
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  border-radius: 14px;
  box-shadow: 0 8px 20px rgba(99, 102, 241, 0.4);
  flex-shrink: 0;
}

.header-text {
  flex: 1;
}

.title {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 4px 0;
  letter-spacing: -0.025em;
}

.subtitle {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
}

/* 版本信息区 */
.version-section {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 28px;
  background: var(--bg-hover);
  border-bottom: 1px solid var(--border-default);
}

.version-badge {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: 10px;
}

.version-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.version-number {
  font-size: 16px;
  font-weight: 700;
  color: #818cf8;
  font-family: 'Courier New', monospace;
}

.version-date {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-muted);
}

/* 更新日志区 */
.changelog-section {
  padding: 20px 28px;
  max-height: 400px;
  overflow-y: auto;
}

.changelog-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 16px;
}

.changelog-content {
  color: #cbd5e1;
  font-size: 14px;
  line-height: 1.7;
}

/* Markdown 样式 */
.changelog-content :deep(h1),
.changelog-content :deep(h2),
.changelog-content :deep(h3) {
  color: var(--text-primary);
  margin: 16px 0 10px 0;
  font-weight: 600;
}

.changelog-content :deep(h1) {
  font-size: 18px;
}

.changelog-content :deep(h2) {
  font-size: 16px;
}

.changelog-content :deep(h3) {
  font-size: 14px;
}

.changelog-content :deep(p) {
  margin: 10px 0;
  color: #cbd5e1;
}

.changelog-content :deep(ul),
.changelog-content :deep(ol) {
  margin: 10px 0;
  padding-left: 24px;
}

.changelog-content :deep(li) {
  margin: 6px 0;
  color: #cbd5e1;
}

.changelog-content :deep(li::marker) {
  color: #6366f1;
}

.changelog-content :deep(code) {
  padding: 2px 6px;
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 4px;
  color: #a78bfa;
  font-family: 'Courier New', monospace;
  font-size: 13px;
}

.changelog-content :deep(pre) {
  margin: 12px 0;
  padding: 12px;
  background: var(--bg-hover);
  border: 1px solid var(--card-border);
  border-radius: 8px;
  overflow-x: auto;
}

.changelog-content :deep(pre code) {
  background: transparent;
  border: none;
  padding: 0;
}

.changelog-content :deep(strong) {
  color: var(--text-primary);
  font-weight: 600;
}

.changelog-content :deep(a) {
  color: #818cf8;
  text-decoration: none;
  transition: color 0.2s ease;
}

.changelog-content :deep(a:hover) {
  color: #a78bfa;
  text-decoration: underline;
}

.changelog-content :deep(blockquote) {
  margin: 12px 0;
  padding: 12px 16px;
  border-left: 3px solid #6366f1;
  background: rgba(99, 102, 241, 0.05);
  color: var(--text-secondary);
  font-style: italic;
}

/* 自定义滚动条 */
.changelog-section::-webkit-scrollbar {
  width: 8px;
}

.changelog-section::-webkit-scrollbar-track {
  background: transparent;
}

.changelog-section::-webkit-scrollbar-thumb {
  background: rgba(148, 163, 184, 0.2);
  border-radius: 4px;
}

.changelog-section::-webkit-scrollbar-thumb:hover {
  background: rgba(148, 163, 184, 0.3);
}

/* 下载进度区 */
.progress-section {
  padding: 20px 28px;
  background: rgba(99, 102, 241, 0.05);
  border-top: 1px solid var(--border-default);
  border-bottom: 1px solid var(--border-default);
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.progress-info {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #a78bfa;
  font-weight: 500;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.progress-percent {
  font-size: 14px;
  font-weight: 600;
  color: #818cf8;
  font-family: 'Courier New', monospace;
}

/* 底部操作区 */
.actions-section {
  padding: 20px 28px;
  background: var(--bg-hover);
  border-top: 1px solid var(--border-default);
}

.action-buttons {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.btn-secondary,
.btn-tertiary,
.btn-primary {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 500;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
  outline: none;
}

.btn-secondary {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-default);
}

.btn-secondary:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
  color: #fca5a5;
}

.btn-tertiary {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-default);
}

.btn-tertiary:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--border-hover);
  color: var(--text-primary);
}

.btn-primary {
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  color: #ffffff;
  box-shadow: 0 8px 20px rgba(99, 102, 241, 0.3);
}

.btn-primary:hover:not(:disabled) {
  background: linear-gradient(135deg, #818cf8 0%, #a78bfa 100%);
  transform: translateY(-1px);
  box-shadow: 0 10px 25px rgba(99, 102, 241, 0.4);
}

.btn-primary.downloading {
  background: linear-gradient(135deg, #818cf8 0%, #a78bfa 100%);
  cursor: not-allowed;
}

.btn-secondary:disabled,
.btn-tertiary:disabled,
.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 底部提示 */
.footer-hint {
  display: flex;
  justify-content: center;
  gap: 20px;
  padding: 12px;
}

.hint-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: var(--text-muted);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #6366f1;
  box-shadow: 0 0 8px rgba(99, 102, 241, 0.6);
  animation: pulse-dot 2s ease-in-out infinite;
}

@keyframes pulse-dot {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.7;
    transform: scale(1.1);
  }
}

/* 响应式 */
@media (max-width: 768px) {
  .update-dialog :deep(.n-modal) {
    max-width: 95%;
  }

  .action-buttons {
    flex-direction: column;
  }

  .btn-secondary,
  .btn-tertiary,
  .btn-primary {
    width: 100%;
    justify-content: center;
  }

  .changelog-section {
    max-height: 300px;
  }
}
</style>
