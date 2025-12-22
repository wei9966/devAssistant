<template>
  <div class="screenshot-grid">
    <n-empty v-if="screenshots.length === 0" description="暂无截图记录" />

    <div v-else class="grid-container">
      <div
        v-for="screenshot in screenshots"
        :key="screenshot.id"
        class="grid-item"
        @click="handleItemClick(screenshot)"
      >
        <div class="thumbnail-wrapper">
          <div v-if="loadingImages[screenshot.id]" class="thumbnail-loading">
            <n-spin size="small" />
          </div>
          <img
            v-else-if="thumbnails[screenshot.id]"
            :src="`data:image/png;base64,${thumbnails[screenshot.id]}`"
            :alt="screenshot.description"
            class="thumbnail-image"
            loading="lazy"
          />
          <div v-else class="thumbnail-placeholder">
            <n-icon :component="ImageOutline" size="32" />
          </div>
        </div>

        <div class="item-info">
          <div class="info-header">
            <span class="activity-badge" :class="`badge-${screenshot.activityType}`">
              {{ getActivityName(screenshot.activityType) }}
            </span>
            <span class="time-text">{{ formatTime(screenshot.capturedAt) }}</span>
          </div>
          <div v-if="screenshot.appName" class="app-name">{{ screenshot.appName }}</div>
          <div class="description">{{ truncateText(screenshot.description, 80) }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { NEmpty, NSpin, NIcon } from 'naive-ui'
import { ImageOutline } from '@vicons/ionicons5'
import { screenshotApi, type ScreenshotRecord } from '@/api/screenshotApi'

interface Props {
  screenshots: ScreenshotRecord[]
}

interface Emits {
  (e: 'select', screenshot: ScreenshotRecord): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const thumbnails = ref<Record<number, string>>({})
const loadingImages = ref<Record<number, boolean>>({})

watch(
  () => props.screenshots,
  async (newScreenshots) => {
    // 并行加载缩略图（提升加载性能）
    const screenshotsToLoad = newScreenshots.filter(
      s => s.screenshotPath && !thumbnails.value[s.id]
    )

    // 分批并行加载，每批最多10张，避免同时请求过多
    const batchSize = 10
    for (let i = 0; i < screenshotsToLoad.length; i += batchSize) {
      const batch = screenshotsToLoad.slice(i, i + batchSize)
      await Promise.all(
        batch.map(s => loadThumbnail(s.id, s.screenshotPath!))
      )
    }
  },
  { immediate: true }
)

async function loadThumbnail(id: number, path: string) {
  loadingImages.value[id] = true
  try {
    const base64 = await screenshotApi.getImage(path, true)
    thumbnails.value[id] = base64
  } catch (error) {
    console.error('加载缩略图失败:', error)
  } finally {
    loadingImages.value[id] = false
  }
}

function handleItemClick(screenshot: ScreenshotRecord) {
  emit('select', screenshot)
}

function getActivityName(activity: string): string {
  const names: Record<string, string> = {
    coding: '编码开发',
    browsing: '网页浏览',
    document: '文档处理',
    meeting: '会议沟通',
    communication: '即时通讯',
    other: '其他',
  }
  return names[activity] || activity
}

function formatTime(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  })
}

function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.substring(0, maxLength) + '...'
}
</script>

<style scoped>
.screenshot-grid {
  width: 100%;
}

.grid-container {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.grid-item {
  background: var(--card-bg);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.3s ease;
}

.grid-item:hover {
  border-color: color-mix(in srgb, var(--accent-primary) 60%, transparent);
  box-shadow: 0 8px 20px var(--accent-glow);
  transform: translateY(-2px);
}

.thumbnail-wrapper {
  position: relative;
  width: 100%;
  padding-top: 56.25%; /* 16:9 aspect ratio */
  background: var(--bg-surface);
  overflow: hidden;
}

.thumbnail-loading,
.thumbnail-placeholder {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-dim);
}

.thumbnail-image {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.item-info {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.activity-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
}

.badge-coding {
  background: rgba(52, 211, 153, 0.1);
  color: rgb(52, 211, 153);
  border: 1px solid rgba(52, 211, 153, 0.2);
}

.badge-browsing {
  background: rgba(59, 130, 246, 0.1);
  color: rgb(96, 165, 250);
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.badge-document {
  background: rgba(251, 191, 36, 0.1);
  color: rgb(251, 191, 36);
  border: 1px solid rgba(251, 191, 36, 0.2);
}

.badge-meeting {
  background: rgba(239, 68, 68, 0.1);
  color: rgb(248, 113, 113);
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.badge-communication {
  background: rgba(139, 92, 246, 0.1);
  color: rgb(167, 139, 250);
  border: 1px solid rgba(139, 92, 246, 0.2);
}

.badge-other {
  background: rgba(148, 163, 184, 0.1);
  color: rgb(148, 163, 184);
  border: 1px solid rgba(148, 163, 184, 0.2);
}

.time-text {
  font-size: 11px;
  color: var(--text-muted);
  font-family: 'Courier New', monospace;
}

.app-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent-primary);
}

.description {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .grid-container {
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  }
}

@media (max-width: 768px) {
  .grid-container {
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 12px;
  }
}
</style>
