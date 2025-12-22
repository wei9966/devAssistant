<template>
  <div class="activity-timeline-item">
    <!-- 活动类型标签 -->
    <div class="activity-header">
      <span class="activity-badge" :class="`badge-${activity.activityType}`">
        {{ getActivityName(activity.activityType) }}
      </span>
      <span class="screenshot-count">{{ activity.screenshots.length }} 张截图</span>
    </div>

    <!-- 活动描述 (可折叠) -->
    <n-popover trigger="hover" placement="top" :width="400">
      <template #trigger>
        <div class="activity-description">
          {{ truncateText(activity.description, 100) }}
        </div>
      </template>
      <div class="description-full">{{ activity.description }}</div>
    </n-popover>

    <!-- 截图缩略图网格 -->
    <div class="screenshots-grid" ref="gridRef">
      <div
        v-for="screenshot in activity.screenshots"
        :key="screenshot.id"
        :ref="el => setThumbRef(screenshot.id, el)"
        class="screenshot-thumb"
        @click="handleScreenshotClick(screenshot)"
      >
        <div v-if="loadingImages[screenshot.id]" class="thumb-loading">
          <n-spin size="small" />
        </div>
        <img
          v-else-if="thumbnails[screenshot.id]"
          :src="`data:image/png;base64,${thumbnails[screenshot.id]}`"
          :alt="screenshot.appName || '截图'"
          class="thumb-image"
        />
        <div v-else class="thumb-placeholder">
          <n-icon :component="ImageOutline" size="20" />
        </div>
        <span class="thumb-time">{{ formatTime(screenshot.capturedAt) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { NPopover, NSpin, NIcon } from 'naive-ui'
import { ImageOutline } from '@vicons/ionicons5'
import { screenshotApi, type ActivityGroup, type ScreenshotResource } from '@/api/screenshotApi'

interface Props {
  activity: ActivityGroup
}

interface Emits {
  (e: 'preview', screenshot: ScreenshotResource): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const thumbnails = ref<Record<number, string>>({})
const loadingImages = ref<Record<number, boolean>>({})
const gridRef = ref<HTMLElement | null>(null)
const thumbRefs = ref<Map<number, HTMLElement>>(new Map())

// 懒加载队列管理器
class LazyLoadQueue {
  private queue: Array<() => Promise<void>> = []
  private running = 0
  private maxConcurrent = 5

  async add(task: () => Promise<void>) {
    this.queue.push(task)
    this.process()
  }

  private async process() {
    if (this.running >= this.maxConcurrent || this.queue.length === 0) {
      return
    }

    this.running++
    const task = this.queue.shift()

    if (task) {
      try {
        await task()
      } catch (error) {
        console.error('Task execution failed:', error)
      } finally {
        this.running--
        this.process()
      }
    }
  }

  clear() {
    this.queue = []
  }
}

const loadQueue = new LazyLoadQueue()
let observer: IntersectionObserver | null = null
const observedElements = new Set<number>()

// 设置缩略图引用
function setThumbRef(id: number, el: any) {
  if (el) {
    thumbRefs.value.set(id, el as HTMLElement)
  } else {
    thumbRefs.value.delete(id)
  }
}

// 加载缩略图
async function loadThumbnail(screenshot: ScreenshotResource) {
  if (!screenshot.path || thumbnails.value[screenshot.id]) return

  loadingImages.value[screenshot.id] = true
  try {
    const base64 = await screenshotApi.getImage(screenshot.path, true)
    thumbnails.value[screenshot.id] = base64
  } catch (error) {
    console.error('加载缩略图失败:', error)
  } finally {
    loadingImages.value[screenshot.id] = false
  }
}

// 初始化 IntersectionObserver
function setupObserver() {
  if (!('IntersectionObserver' in window)) {
    // 降级方案：直接加载所有图片
    props.activity.screenshots.forEach(screenshot => {
      if (screenshot.path && !thumbnails.value[screenshot.id]) {
        loadQueue.add(() => loadThumbnail(screenshot))
      }
    })
    return
  }

  observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          const element = entry.target as HTMLElement
          const screenshotId = parseInt(element.dataset.screenshotId || '0')

          if (screenshotId && !observedElements.has(screenshotId)) {
            observedElements.add(screenshotId)
            const screenshot = props.activity.screenshots.find(s => s.id === screenshotId)

            if (screenshot && screenshot.path && !thumbnails.value[screenshotId]) {
              loadQueue.add(() => loadThumbnail(screenshot))
            }

            // 加载后取消观察
            observer?.unobserve(element)
          }
        }
      })
    },
    {
      root: null,
      rootMargin: '50px',
      threshold: 0.01
    }
  )

  // 观察所有缩略图元素
  nextTick(() => {
    thumbRefs.value.forEach((element, screenshotId) => {
      element.dataset.screenshotId = screenshotId.toString()
      observer?.observe(element)
    })
  })
}

// 清理资源
function cleanup() {
  if (observer) {
    observer.disconnect()
    observer = null
  }
  observedElements.clear()
  loadQueue.clear()
  thumbRefs.value.clear()
}

onMounted(() => {
  setupObserver()
})

onBeforeUnmount(() => {
  cleanup()
})

function handleScreenshotClick(screenshot: ScreenshotResource) {
  emit('preview', screenshot)
}

function getActivityName(type: string): string {
  const names: Record<string, string> = {
    coding: '编码开发',
    browsing: '网页浏览',
    document: '文档处理',
    meeting: '会议沟通',
    communication: '即时通讯',
    other: '其他'
  }
  return names[type] || type
}

function truncateText(text: string, max: number): string {
  return text.length > max ? text.slice(0, max) + '...' : text
}

function formatTime(timestamp: string): string {
  return new Date(timestamp).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit'
  })
}
</script>

<style scoped>
.activity-timeline-item {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: var(--card-bg);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  transition: all 0.3s ease;
}

.activity-timeline-item:hover {
  border-color: color-mix(in srgb, var(--accent-primary) 40%, transparent);
  background: var(--card-hover-bg);
}

.activity-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.activity-badge {
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.badge-coding {
  background: rgba(52, 211, 153, 0.15);
  color: rgb(52, 211, 153);
  border: 1px solid rgba(52, 211, 153, 0.3);
}

.badge-browsing {
  background: rgba(59, 130, 246, 0.15);
  color: rgb(96, 165, 250);
  border: 1px solid rgba(59, 130, 246, 0.3);
}

.badge-document {
  background: rgba(251, 191, 36, 0.15);
  color: rgb(251, 191, 36);
  border: 1px solid rgba(251, 191, 36, 0.3);
}

.badge-meeting {
  background: rgba(239, 68, 68, 0.15);
  color: rgb(248, 113, 113);
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.badge-communication {
  background: rgba(139, 92, 246, 0.15);
  color: rgb(167, 139, 250);
  border: 1px solid rgba(139, 92, 246, 0.3);
}

.badge-other {
  background: rgba(148, 163, 184, 0.15);
  color: rgb(148, 163, 184);
  border: 1px solid rgba(148, 163, 184, 0.3);
}

.screenshot-count {
  font-size: 12px;
  color: var(--text-muted);
  padding: 2px 8px;
  background: var(--bg-surface);
  border-radius: 4px;
}

.activity-description {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
  cursor: pointer;
  transition: color 0.2s;
}

.activity-description:hover {
  color: var(--text-primary);
}

.description-full {
  font-size: 14px;
  line-height: 1.6;
  max-height: 300px;
  overflow-y: auto;
  color: var(--text-secondary);
}

.description-full::-webkit-scrollbar {
  width: 4px;
}

.description-full::-webkit-scrollbar-track {
  background: transparent;
}

.description-full::-webkit-scrollbar-thumb {
  background: var(--border-default);
  border-radius: 2px;
}

.description-full::-webkit-scrollbar-thumb:hover {
  background: var(--border-hover);
}

.screenshots-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.screenshot-thumb {
  position: relative;
  width: 110px;
  height: 70px;
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  transition: all 0.2s ease;
}

.screenshot-thumb:hover {
  transform: scale(1.05);
  box-shadow: 0 4px 12px var(--accent-glow);
  border-color: color-mix(in srgb, var(--accent-primary) 60%, transparent);
  z-index: 1;
}

.thumb-loading,
.thumb-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  color: var(--text-dim);
}

.thumb-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.thumb-time {
  position: absolute;
  bottom: 4px;
  right: 4px;
  font-size: 10px;
  color: white;
  background: rgba(0, 0, 0, 0.7);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'Courier New', monospace;
  font-weight: 500;
  backdrop-filter: blur(4px);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .activity-timeline-item {
    padding: 12px;
  }

  .screenshot-thumb {
    width: 90px;
    height: 60px;
  }

  .activity-description {
    font-size: 13px;
  }
}
</style>
