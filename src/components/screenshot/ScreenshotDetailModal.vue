<template>
  <n-modal
    v-model:show="showModal"
    preset="card"
    :title="modalTitle"
    class="screenshot-modal"
    :style="{ width: '90vw', maxWidth: '1200px' }"
    :segmented="{ content: 'soft' }"
    @after-leave="handleAfterLeave"
  >
    <div v-if="currentScreenshot" class="modal-content">
      <!-- 图片预览区 -->
      <div class="image-section">
        <div v-if="loadingImage" class="image-loading">
          <n-spin size="large" />
          <span>加载中...</span>
        </div>
        <img
          v-else-if="currentImage"
          :src="`data:image/png;base64,${currentImage}`"
          :alt="currentScreenshot.description"
          class="preview-image"
        />
        <div v-else class="image-placeholder">
          <n-icon :component="ImageOutline" size="64" />
          <span>无图片</span>
        </div>
      </div>

      <!-- 信息区 -->
      <div class="info-section">
        <div class="info-group">
          <div class="info-label">捕获时间</div>
          <div class="info-value">{{ formatFullTime(currentScreenshot.capturedAt) }}</div>
        </div>

        <div class="info-group">
          <div class="info-label">活动类型</div>
          <div class="info-value">
            <span class="activity-badge" :class="`badge-${currentScreenshot.activityType}`">
              {{ getActivityName(currentScreenshot.activityType) }}
            </span>
          </div>
        </div>

        <div v-if="currentScreenshot.appName" class="info-group">
          <div class="info-label">应用程序</div>
          <div class="info-value app-value">{{ currentScreenshot.appName }}</div>
        </div>

        <div v-if="currentScreenshot.windowTitle" class="info-group">
          <div class="info-label">窗口标题</div>
          <div class="info-value">{{ currentScreenshot.windowTitle }}</div>
        </div>

        <div class="info-group">
          <div class="info-label">描述</div>
          <div class="info-value description-value">{{ currentScreenshot.description }}</div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="modal-footer">
        <div class="navigation-buttons">
          <n-button
            :disabled="!canGoPrevious"
            @click="handlePrevious"
            secondary
          >
            <template #icon>
              <n-icon :component="ChevronBackOutline" />
            </template>
            上一张
          </n-button>
          <n-button
            :disabled="!canGoNext"
            @click="handleNext"
            secondary
          >
            下一张
            <template #icon>
              <n-icon :component="ChevronForwardOutline" />
            </template>
          </n-button>
        </div>
        <n-button type="primary" @click="handleClose">关闭</n-button>
      </div>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { NModal, NButton, NSpin, NIcon } from 'naive-ui'
import {
  ImageOutline,
  ChevronBackOutline,
  ChevronForwardOutline,
} from '@vicons/ionicons5'
import { screenshotApi, type ScreenshotRecord } from '@/api/screenshotApi'

interface Props {
  show: boolean
  screenshot: ScreenshotRecord | null
  screenshots: ScreenshotRecord[]
}

interface Emits {
  (e: 'update:show', value: boolean): void
  (e: 'change', screenshot: ScreenshotRecord): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const showModal = computed({
  get: () => props.show,
  set: (value) => emit('update:show', value),
})

const currentScreenshot = ref<ScreenshotRecord | null>(null)
const currentImage = ref<string>('')
const loadingImage = ref(false)

const currentIndex = computed(() => {
  if (!currentScreenshot.value) return -1
  return props.screenshots.findIndex((s) => s.id === currentScreenshot.value!.id)
})

const canGoPrevious = computed(() => currentIndex.value > 0)
const canGoNext = computed(() => currentIndex.value < props.screenshots.length - 1)

const modalTitle = computed(() => {
  if (!currentScreenshot.value) return '截图详情'
  return `截图详情 (${currentIndex.value + 1} / ${props.screenshots.length})`
})

watch(
  () => props.screenshot,
  (newScreenshot) => {
    if (newScreenshot) {
      currentScreenshot.value = newScreenshot
      loadImage(newScreenshot)
    }
  },
  { immediate: true }
)

async function loadImage(screenshot: ScreenshotRecord) {
  if (!screenshot.screenshotPath) {
    currentImage.value = ''
    return
  }

  loadingImage.value = true
  try {
    const base64 = await screenshotApi.getImage(screenshot.screenshotPath, false)
    currentImage.value = base64
  } catch (error) {
    console.error('加载图片失败:', error)
    currentImage.value = ''
  } finally {
    loadingImage.value = false
  }
}

function handlePrevious() {
  if (canGoPrevious.value) {
    const newScreenshot = props.screenshots[currentIndex.value - 1]
    currentScreenshot.value = newScreenshot
    loadImage(newScreenshot)
    emit('change', newScreenshot)
  }
}

function handleNext() {
  if (canGoNext.value) {
    const newScreenshot = props.screenshots[currentIndex.value + 1]
    currentScreenshot.value = newScreenshot
    loadImage(newScreenshot)
    emit('change', newScreenshot)
  }
}

function handleClose() {
  showModal.value = false
}

function handleAfterLeave() {
  currentScreenshot.value = null
  currentImage.value = ''
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

function formatFullTime(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}
</script>

<style scoped>
.screenshot-modal {
  max-height: 90vh;
}

.modal-content {
  display: flex;
  gap: 24px;
  min-height: 400px;
}

.image-section {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(30, 41, 59, 0.3);
  border-radius: 12px;
  overflow: hidden;
  position: relative;
  min-height: 400px;
}

.image-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: rgb(148, 163, 184);
}

.preview-image {
  max-width: 100%;
  max-height: 600px;
  object-fit: contain;
}

.image-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: rgb(100, 116, 139);
}

.info-section {
  width: 320px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  flex-shrink: 0;
}

.info-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-label {
  font-size: 12px;
  font-weight: 600;
  color: rgb(148, 163, 184);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.info-value {
  font-size: 14px;
  color: rgb(226, 232, 240);
  line-height: 1.5;
}

.app-value {
  font-weight: 600;
  color: rgb(99, 102, 241);
}

.description-value {
  padding: 12px;
  background: rgba(30, 41, 59, 0.5);
  border-radius: 8px;
  border: 1px solid rgba(51, 65, 85, 0.4);
}

.activity-badge {
  display: inline-block;
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 12px;
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

.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.navigation-buttons {
  display: flex;
  gap: 8px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .modal-content {
    flex-direction: column;
  }

  .info-section {
    width: 100%;
  }
}

/* Naive UI 样式覆盖 */
:deep(.n-card__content) {
  padding: 24px;
}

:deep(.n-button--secondary-type) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-color: transparent;
  --n-color-hover: rgba(99, 102, 241, 0.1);
}
</style>
