<template>
  <div class="screenshot-timeline">
    <!-- 日期导航 -->
    <DateNavigation
      v-model:date="currentDate"
      @change="handleDateChange"
    />

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <n-spin size="large" />
      <span>加载中...</span>
    </div>

    <!-- 时间轴 -->
    <div v-else class="timeline-section">
      <n-empty v-if="activities.length === 0" description="当天暂无活动记录" />

      <n-timeline v-else>
        <n-timeline-item
          v-for="activity in visibleActivities"
          :key="activity.id"
          :time="activity.title"
          type="success"
        >
          <ActivityTimelineItem
            :activity="activity"
            @preview="handlePreview"
          />
        </n-timeline-item>
      </n-timeline>

      <!-- 加载更多sentinel元素 -->
      <div
        v-if="hasMore"
        :ref="observeSentinel"
        class="load-more-sentinel"
      >
        <n-spin size="small" />
        <span>加载更多...</span>
      </div>

      <!-- 已全部加载提示 -->
      <div v-else-if="activities.length > 10" class="all-loaded">
        已加载全部 {{ activities.length }} 个活动分组
      </div>
    </div>

    <!-- 详情模态框 -->
    <ScreenshotDetailModal
      v-model:show="showDetailModal"
      :screenshot="selectedScreenshot"
      :screenshots="allScreenshots"
      @change="handleScreenshotChange"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { NSpin, NTimeline, NTimelineItem, NEmpty, useMessage } from 'naive-ui'
import { screenshotApi, type ActivityGroup, type ScreenshotResource, type ScreenshotRecord } from '@/api/screenshotApi'
import DateNavigation from '@/components/screenshot/DateNavigation.vue'
import ActivityTimelineItem from '@/components/screenshot/ActivityTimelineItem.vue'
import ScreenshotDetailModal from '@/components/screenshot/ScreenshotDetailModal.vue'

const message = useMessage()

const currentDate = ref(new Date())
const activities = ref<ActivityGroup[]>([])
const loading = ref(false)
const showDetailModal = ref(false)
const selectedScreenshot = ref<ScreenshotRecord | null>(null)

// 虚拟滚动相关状态
const visibleCount = ref(10) // 初始显示10个活动分组
const LOAD_MORE_THRESHOLD = 5 // 每次加载5个
const sentinelElement = ref<HTMLElement | null>(null)
let intersectionObserver: IntersectionObserver | null = null

// 可见的活动列表（用于渲染）
const visibleActivities = computed(() => {
  return activities.value.slice(0, visibleCount.value)
})

// 是否还有更多活动
const hasMore = computed(() => {
  return visibleCount.value < activities.value.length
})

// 计算所有截图（用于详情模态框导航）
const allScreenshots = computed(() => {
  return activities.value.flatMap(a =>
    a.screenshots.map(s => ({
      id: s.id,
      capturedAt: s.capturedAt,
      appName: s.appName,
      screenshotPath: s.path,
      windowTitle: null,
      activityType: a.activityType,
      description: s.description || a.description,  // 优先使用单张截图描述，fallback到活动描述
      keyContent: null,
      processingTimeMs: null,
    } as ScreenshotRecord))
  )
})

onMounted(() => {
  loadActivities()
  setupIntersectionObserver()
})

onUnmounted(() => {
  cleanupIntersectionObserver()
})

async function loadActivities() {
  loading.value = true
  try {
    const dateStr = formatDate(currentDate.value)
    activities.value = await screenshotApi.getActivities(dateStr)
    // 重置可见数量
    visibleCount.value = Math.min(10, activities.value.length)
  } catch (error: any) {
    console.error('加载活动失败:', error)
    message.error('加载活动失败: ' + (error.message || error))
  } finally {
    loading.value = false
  }
}

// 设置 IntersectionObserver
function setupIntersectionObserver() {
  cleanupIntersectionObserver()

  intersectionObserver = new IntersectionObserver(
    (entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting && hasMore.value) {
          loadMore()
        }
      })
    },
    {
      root: null,
      rootMargin: '100px', // 提前100px开始加载
      threshold: 0.1
    }
  )
}

// 清理 IntersectionObserver
function cleanupIntersectionObserver() {
  if (intersectionObserver) {
    intersectionObserver.disconnect()
    intersectionObserver = null
  }
}

// 加载更多活动
function loadMore() {
  const newCount = Math.min(
    visibleCount.value + LOAD_MORE_THRESHOLD,
    activities.value.length
  )
  visibleCount.value = newCount
}

// 观察 sentinel 元素
function observeSentinel(el: HTMLElement | null) {
  if (!el) return

  sentinelElement.value = el
  if (intersectionObserver) {
    intersectionObserver.observe(el)
  }
}

function handleDateChange(date: Date) {
  currentDate.value = date
  loadActivities()
}

function handlePreview(screenshot: ScreenshotResource) {
  // 将 ScreenshotResource 转换为 ScreenshotRecord
  const activity = activities.value.find(a =>
    a.screenshots.some(s => s.id === screenshot.id)
  )

  selectedScreenshot.value = {
    id: screenshot.id,
    capturedAt: screenshot.capturedAt,
    appName: screenshot.appName,
    screenshotPath: screenshot.path,
    windowTitle: null,
    activityType: activity?.activityType || 'other',
    description: screenshot.description || activity?.description || '',  // 优先使用单张截图描述
    keyContent: null,
    processingTimeMs: null,
  }
  showDetailModal.value = true

  // 如果预览的截图在可见范围之外，自动加载更多直到该截图可见
  const screenshotIndex = allScreenshots.value.findIndex(s => s.id === screenshot.id)
  if (screenshotIndex !== -1) {
    const activityIndex = activities.value.findIndex(a =>
      a.screenshots.some(s => s.id === screenshot.id)
    )
    if (activityIndex >= visibleCount.value) {
      visibleCount.value = Math.min(activityIndex + 5, activities.value.length)
    }
  }
}

function handleScreenshotChange(screenshot: ScreenshotRecord) {
  selectedScreenshot.value = screenshot
}

function formatDate(date: Date): string {
  // 使用本地时间格式化，避免 toISOString() 的 UTC 时区问题
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}
</script>

<style scoped>
.screenshot-timeline {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 24px;
  height: 100%;
  overflow-y: auto;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 64px;
  color: rgb(148, 163, 184);
}

.timeline-section {
  flex: 1;
}

/* Naive UI Timeline 样式覆盖 */
:deep(.n-timeline) {
  padding-left: 20px;
}

:deep(.n-timeline-item) {
  padding-bottom: 32px;
}

:deep(.n-timeline-item-timeline__line) {
  background: rgba(99, 102, 241, 0.3);
}

:deep(.n-timeline-item-timeline__circle) {
  background: rgb(99, 102, 241);
  border-color: rgba(99, 102, 241, 0.5);
}

:deep(.n-timeline-item-content__time) {
  color: rgb(148, 163, 184);
  font-weight: 600;
  font-size: 14px;
}

:deep(.n-empty) {
  padding: 64px 0;
}

/* 加载更多sentinel */
.load-more-sentinel {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 32px;
  color: rgb(148, 163, 184);
  font-size: 14px;
}

/* 已全部加载提示 */
.all-loaded {
  text-align: center;
  padding: 32px;
  color: rgb(148, 163, 184);
  font-size: 13px;
  border-top: 1px dashed rgba(148, 163, 184, 0.2);
  margin-top: 16px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .screenshot-timeline {
    padding: 16px;
  }

  :deep(.n-timeline) {
    padding-left: 10px;
  }
}
</style>
