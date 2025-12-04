<template>
  <div class="context-timeline">
    <!-- 过滤器和操作栏 -->
    <section class="settings-card filter-section">
      <div class="card-header">
        <h3 class="card-title">时间线</h3>
        <n-space>
          <n-button
            @click="handleRefresh"
            :loading="loading"
            size="small"
          >
            <template #icon>
              <n-icon :component="RefreshOutline" />
            </template>
            刷新
          </n-button>
          <n-button
            type="primary"
            @click="handleGenerateReport"
            :loading="generatingReport"
            :disabled="!selectedDate"
            size="small"
          >
            <template #icon>
              <n-icon :component="DocumentTextOutline" />
            </template>
            生成日报
          </n-button>
        </n-space>
      </div>
      <div class="card-content filter-controls">
        <div class="filter-row">
          <!-- 日期选择器 -->
          <div class="filter-item">
            <span class="filter-label">日期</span>
            <n-date-picker
              v-model:value="selectedDateTimestamp"
              type="date"
              clearable
              style="width: 200px"
              @update:value="handleDateChange"
            />
          </div>

          <!-- 活动类型过滤 -->
          <div class="filter-item">
            <span class="filter-label">活动类型</span>
            <n-select
              v-model:value="selectedActivityTypes"
              :options="activityTypeOptions"
              multiple
              clearable
              placeholder="全部类型"
              style="min-width: 200px"
              @update:value="handleFilterChange"
            />
          </div>

          <!-- 最低重要性 -->
          <div class="filter-item">
            <span class="filter-label">最低重要性</span>
            <n-slider
              v-model:value="minImportance"
              :min="1"
              :max="5"
              :step="1"
              :marks="importanceMarks"
              style="width: 150px"
              @update:value="handleFilterChange"
            />
          </div>
        </div>
      </div>
    </section>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <n-spin size="medium" />
      <span>加载时间线数据...</span>
    </div>

    <!-- 空状态 -->
    <n-empty
      v-else-if="!timelineGroups.length"
      description="暂无活动记录"
      class="empty-state"
    />

    <!-- 时间线内容 -->
    <div v-else class="timeline-content">
      <!-- 按日期分组 -->
      <section
        v-for="group in timelineGroups"
        :key="group.date"
        class="settings-card timeline-group"
      >
        <div class="card-header group-header">
          <div class="group-info">
            <h3 class="card-title">{{ group.displayDate }}</h3>
            <span class="item-count">{{ group.items.length }} 个活动</span>
          </div>
          <n-button
            v-if="group.summary"
            text
            @click="showSummary(group)"
          >
            <template #icon>
              <n-icon :component="EyeOutline" />
            </template>
            查看总结
          </n-button>
        </div>

        <div class="card-content">
          <n-timeline>
            <n-timeline-item
              v-for="item in group.items"
              :key="item.id"
              :type="getTimelineItemType(item.activityType)"
              :title="formatTime(item.startTime)"
            >
              <template #icon>
                <n-icon :component="getActivityIcon(item.activityType)" :size="20" />
              </template>

              <div class="timeline-item-content">
                <!-- 标题和应用名称 -->
                <div class="item-header">
                  <h4 class="item-title">{{ item.title }}</h4>
                  <div class="item-meta">
                    <n-tag
                      :type="getActivityTagType(item.activityType)"
                      size="small"
                      round
                    >
                      {{ getActivityTypeName(item.activityType) }}
                    </n-tag>
                    <span v-if="item.appName" class="app-name">
                      {{ item.appName }}
                    </span>
                    <n-tag
                      v-if="item.merged"
                      type="info"
                      size="small"
                      round
                    >
                      合并 {{ item.mergedCount }} 项
                    </n-tag>
                  </div>
                </div>

                <!-- 重要性指示器 -->
                <div class="importance-indicator">
                  <n-rate
                    :value="item.importance"
                    readonly
                    size="small"
                    :count="5"
                  />
                </div>

                <!-- 摘要 -->
                <p class="item-summary">{{ item.summary }}</p>

                <!-- 关键词标签 -->
                <div v-if="item.keywords.length" class="item-keywords">
                  <n-tag
                    v-for="keyword in item.keywords"
                    :key="keyword"
                    size="small"
                    :bordered="false"
                  >
                    {{ keyword }}
                  </n-tag>
                </div>

                <!-- 操作按钮 -->
                <div class="item-actions">
                  <n-button
                    v-if="item.thumbnailPath"
                    text
                    size="small"
                    @click="viewScreenshot(item)"
                  >
                    <template #icon>
                      <n-icon :component="ImageOutline" />
                    </template>
                    查看截图
                  </n-button>
                  <n-button
                    text
                    size="small"
                    @click="toggleDetails(item.id)"
                  >
                    <template #icon>
                      <n-icon :component="expandedItems.has(item.id) ? ChevronUpOutline : ChevronDownOutline" />
                    </template>
                    {{ expandedItems.has(item.id) ? '收起' : '详情' }}
                  </n-button>
                </div>

                <!-- 展开的详情 -->
                <div v-if="expandedItems.has(item.id)" class="item-details">
                  <div class="detail-row">
                    <span class="detail-label">开始时间:</span>
                    <span class="detail-value">{{ formatFullTime(item.startTime) }}</span>
                  </div>
                  <div v-if="item.endTime" class="detail-row">
                    <span class="detail-label">结束时间:</span>
                    <span class="detail-value">{{ formatFullTime(item.endTime) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">持续时长:</span>
                    <span class="detail-value">{{ calculateDuration(item.startTime, item.endTime) }}</span>
                  </div>
                </div>
              </div>
            </n-timeline-item>
          </n-timeline>
        </div>
      </section>
    </div>

    <!-- 日报预览弹窗 -->
    <n-modal
      v-model:show="showReportModal"
      preset="card"
      title="日报预览"
      style="width: 800px; max-height: 80vh"
      :bordered="false"
    >
      <div class="report-content" v-html="reportContent"></div>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showReportModal = false">关闭</n-button>
          <n-button type="primary" @click="copyReport">复制</n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 总结预览弹窗 -->
    <n-modal
      v-model:show="showSummaryModal"
      preset="card"
      title="日期总结"
      style="width: 600px"
      :bordered="false"
    >
      <div class="summary-content">{{ currentSummary }}</div>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showSummaryModal = false">关闭</n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 截图预览弹窗 -->
    <n-modal
      v-model:show="showScreenshotModal"
      preset="card"
      title="截图预览"
      style="width: 90vw; max-width: 1200px"
      :bordered="false"
    >
      <div class="screenshot-content">
        <img v-if="currentScreenshot" :src="currentScreenshot" alt="截图" />
      </div>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showScreenshotModal = false">关闭</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  NDatePicker,
  NSelect,
  NSlider,
  NButton,
  NIcon,
  NSpin,
  NEmpty,
  NTimeline,
  NTimelineItem,
  NTag,
  NRate,
  NModal,
  NSpace,
  useMessage,
} from 'naive-ui';
import {
  RefreshOutline,
  DocumentTextOutline,
  EyeOutline,
  ImageOutline,
  ChevronUpOutline,
  ChevronDownOutline,
  CodeOutline,
  GlobeOutline,
  ChatbubbleOutline,
  DocumentOutline,
  BrushOutline,
  EllipsisHorizontalCircleOutline,
} from '@vicons/ionicons5';
import { timelineApi } from '@/api/timelineApi';
import type { TimelineGroup, TimelineItem, TimelineFilter } from '@/types/timeline';
import { convertFileSrc } from '@tauri-apps/api/core';

const message = useMessage();

const loading = ref(true);
const generatingReport = ref(false);
const timelineGroups = ref<TimelineGroup[]>([]);
const expandedItems = ref(new Set<string>());

// 过滤器状态
const selectedDateTimestamp = ref<number | null>(null);
const selectedDate = computed(() => {
  if (!selectedDateTimestamp.value) return null;
  // 使用本地时间格式化，避免 toISOString() 的 UTC 时区问题
  const date = new Date(selectedDateTimestamp.value);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
});
const selectedActivityTypes = ref<string[]>([]);
const minImportance = ref(1);

// 弹窗状态
const showReportModal = ref(false);
const reportContent = ref('');
const showSummaryModal = ref(false);
const currentSummary = ref('');
const showScreenshotModal = ref(false);
const currentScreenshot = ref('');

// 活动类型选项
const activityTypeOptions = [
  { label: '编码', value: 'coding' },
  { label: '浏览', value: 'browsing' },
  { label: '聊天', value: 'chatting' },
  { label: '文档', value: 'document' },
  { label: '设计', value: 'design' },
  { label: '其他', value: 'other' },
];

// 重要性标记
const importanceMarks = {
  1: '1',
  2: '2',
  3: '3',
  4: '4',
  5: '5',
};

onMounted(async () => {
  // 默认加载今天的数据
  selectedDateTimestamp.value = Date.now();
  await loadTimeline();
});

async function loadTimeline() {
  loading.value = true;
  try {
    const filter: TimelineFilter = {
      dateRange: selectedDate.value ? [selectedDate.value, selectedDate.value] : undefined,
      activityTypes: selectedActivityTypes.value.length ? selectedActivityTypes.value : undefined,
      minImportance: minImportance.value > 1 ? minImportance.value : undefined,
    };

    timelineGroups.value = await timelineApi.getTimeline(filter);
  } catch (error: any) {
    console.error('加载时间线失败:', error);
    message.error('加载时间线失败');
  } finally {
    loading.value = false;
  }
}

function handleDateChange() {
  loadTimeline();
}

function handleFilterChange() {
  loadTimeline();
}

function handleRefresh() {
  loadTimeline();
}

async function handleGenerateReport() {
  if (!selectedDate.value) {
    message.warning('请选择日期');
    return;
  }

  generatingReport.value = true;
  try {
    const report = await timelineApi.generateDailyReport(selectedDate.value);
    reportContent.value = report;
    showReportModal.value = true;
  } catch (error: any) {
    console.error('生成日报失败:', error);
    message.error('生成日报失败');
  } finally {
    generatingReport.value = false;
  }
}

function copyReport() {
  navigator.clipboard.writeText(reportContent.value);
  message.success('已复制到剪贴板');
}

function showSummary(group: TimelineGroup) {
  currentSummary.value = group.summary || '暂无总结';
  showSummaryModal.value = true;
}

function viewScreenshot(item: TimelineItem) {
  if (item.thumbnailPath) {
    currentScreenshot.value = convertFileSrc(item.thumbnailPath);
    showScreenshotModal.value = true;
  }
}

function toggleDetails(itemId: string) {
  if (expandedItems.value.has(itemId)) {
    expandedItems.value.delete(itemId);
  } else {
    expandedItems.value.add(itemId);
  }
}

function formatTime(timeStr: string): string {
  const date = new Date(timeStr);
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
}

function formatFullTime(timeStr: string): string {
  const date = new Date(timeStr);
  return date.toLocaleString('zh-CN');
}

function calculateDuration(startTime: string, endTime?: string): string {
  const start = new Date(startTime);
  const end = endTime ? new Date(endTime) : new Date();
  const durationMs = end.getTime() - start.getTime();

  const minutes = Math.floor(durationMs / 60000);
  if (minutes < 60) {
    return `${minutes} 分钟`;
  }

  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  return `${hours} 小时 ${remainingMinutes} 分钟`;
}

function getActivityTypeName(type: string): string {
  const names: Record<string, string> = {
    coding: '编码',
    browsing: '浏览',
    chatting: '聊天',
    document: '文档',
    design: '设计',
    other: '其他',
  };
  return names[type] || type;
}

function getActivityTagType(type: string): 'default' | 'success' | 'info' | 'warning' | 'error' {
  const types: Record<string, 'default' | 'success' | 'info' | 'warning' | 'error'> = {
    coding: 'success',
    browsing: 'info',
    chatting: 'warning',
    document: 'default',
    design: 'error',
    other: 'default',
  };
  return types[type] || 'default';
}

function getActivityIcon(type: string) {
  const icons: Record<string, any> = {
    coding: CodeOutline,
    browsing: GlobeOutline,
    chatting: ChatbubbleOutline,
    document: DocumentOutline,
    design: BrushOutline,
    other: EllipsisHorizontalCircleOutline,
  };
  return icons[type] || EllipsisHorizontalCircleOutline;
}

function getTimelineItemType(type: string): 'default' | 'success' | 'info' | 'warning' | 'error' {
  return getActivityTagType(type);
}
</script>

<style scoped>
.context-timeline {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* 过滤器区域 */
.filter-section {
  position: sticky;
  top: 0;
  z-index: 10;
}

.filter-controls {
  padding: 16px 24px !important;
}

.filter-row {
  display: flex;
  align-items: center;
  gap: 24px;
  flex-wrap: wrap;
}

.filter-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.filter-label {
  font-size: 13px;
  font-weight: 500;
  color: rgb(148, 163, 184);
  white-space: nowrap;
}

/* 加载和空状态 */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 64px;
  color: rgb(148, 163, 184);
}

.empty-state {
  padding: 64px;
}

/* 时间线内容 */
.timeline-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.timeline-group {
  overflow: visible;
}

.group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.group-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.item-count {
  font-size: 12px;
  color: rgb(100, 116, 139);
  padding: 4px 8px;
  background: rgba(30, 41, 59, 0.5);
  border-radius: 4px;
}

/* 时间线项 */
.timeline-item-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
}

.item-title {
  font-size: 15px;
  font-weight: 500;
  color: rgb(226, 232, 240);
  margin: 0;
  flex: 1;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.app-name {
  font-size: 12px;
  color: rgb(148, 163, 184);
  padding: 2px 8px;
  background: rgba(30, 41, 59, 0.5);
  border-radius: 4px;
}

.importance-indicator {
  margin: -4px 0;
}

.item-summary {
  font-size: 13px;
  color: rgb(148, 163, 184);
  line-height: 1.6;
  margin: 0;
}

.item-keywords {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.item-actions {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.item-details {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  background: rgba(30, 41, 59, 0.3);
  border-radius: 8px;
  border: 1px solid rgba(51, 65, 85, 0.4);
  margin-top: 4px;
}

.detail-row {
  display: flex;
  gap: 12px;
  font-size: 13px;
}

.detail-label {
  color: rgb(100, 116, 139);
  font-weight: 500;
  min-width: 80px;
}

.detail-value {
  color: rgb(203, 213, 225);
}

/* 弹窗内容 */
.report-content {
  padding: 16px;
  background: rgba(30, 41, 59, 0.3);
  border-radius: 8px;
  border: 1px solid rgba(51, 65, 85, 0.4);
  color: rgb(203, 213, 225);
  line-height: 1.8;
  max-height: 60vh;
  overflow-y: auto;
  white-space: pre-wrap;
}

.summary-content {
  padding: 16px;
  color: rgb(203, 213, 225);
  line-height: 1.8;
  white-space: pre-wrap;
}

.screenshot-content {
  display: flex;
  justify-content: center;
  align-items: center;
  background: rgba(0, 0, 0, 0.5);
  border-radius: 8px;
  padding: 16px;
}

.screenshot-content img {
  max-width: 100%;
  max-height: 70vh;
  border-radius: 4px;
}

/* 卡片样式（复用VlmSettings样式） */
.settings-card {
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.6);
  border-radius: 16px;
  overflow: hidden;
  backdrop-filter: blur(8px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  border-bottom: 1px solid rgba(51, 65, 85, 0.6);
  background: rgba(15, 23, 42, 0.3);
}

.card-title {
  font-size: 14px;
  font-weight: 500;
  color: rgb(226, 232, 240);
  margin: 0;
}

.card-content {
  padding: 24px;
}

/* Naive UI 样式覆盖 */
:deep(.n-date-picker) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-border-focus: 1px solid rgb(99, 102, 241);
  --n-color: rgb(2, 6, 23);
  --n-text-color: rgb(203, 213, 225);
}

:deep(.n-select) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-border-focus: 1px solid rgb(99, 102, 241);
  --n-color: rgb(2, 6, 23);
  --n-text-color: rgb(203, 213, 225);
}

:deep(.n-slider) {
  --n-fill-color: rgb(99, 102, 241);
  --n-fill-color-hover: rgb(79, 70, 229);
  --n-handle-color: rgb(255, 255, 255);
  --n-rail-color: rgb(51, 65, 85);
}

:deep(.n-timeline) {
  --n-icon-size-timeline: 20px;
}

:deep(.n-timeline-item) {
  --n-color: transparent;
}

:deep(.n-timeline-item__timeline) {
  padding-left: 8px;
}

:deep(.n-timeline-item__header) {
  color: rgb(100, 116, 139);
  font-size: 12px;
  font-weight: 500;
}

:deep(.n-rate) {
  --n-item-color: rgb(51, 65, 85);
  --n-item-color-active: rgb(251, 191, 36);
}

:deep(.n-tag) {
  --n-border: transparent;
}

:deep(.n-button--primary-type) {
  --n-color: rgb(99, 102, 241);
  --n-color-hover: rgb(79, 70, 229);
  --n-text-color: rgb(255, 255, 255);
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2);
}

:deep(.n-modal) {
  --n-color: rgb(15, 23, 42);
  --n-title-text-color: rgb(226, 232, 240);
  --n-border-color: rgba(51, 65, 85, 0.6);
}

/* 滚动条样式 */
.report-content::-webkit-scrollbar {
  width: 6px;
}

.report-content::-webkit-scrollbar-track {
  background: transparent;
}

.report-content::-webkit-scrollbar-thumb {
  background: rgba(51, 65, 85, 0.5);
  border-radius: 3px;
}

.report-content::-webkit-scrollbar-thumb:hover {
  background: rgba(71, 85, 105, 0.7);
}
</style>
