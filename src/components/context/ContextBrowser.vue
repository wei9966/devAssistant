<template>
  <div class="context-browser">
    <!-- 日期选择和操作栏 -->
    <section class="browser-header">
      <n-date-picker
        v-model:value="selectedDate"
        type="date"
        placeholder="选择日期"
        @update:value="handleDateChange"
      />
      <n-space>
        <n-button @click="handleGenerateSummary" :loading="generatingSummary" :disabled="!dayStats || dayStats.totalCount === 0">
          生成日报
        </n-button>
        <n-button @click="handleCleanup" :loading="cleaning">
          清理过期数据
        </n-button>
        <n-popconfirm @positive-click="handleDelete" negative-text="取消" positive-text="删除">
          <template #trigger>
            <n-button type="error" :disabled="!dayStats || dayStats.totalCount === 0">
              删除当日数据
            </n-button>
          </template>
          确定要删除 {{ formatDate(selectedDate) }} 的所有上下文数据吗？
        </n-popconfirm>
      </n-space>
    </section>

    <!-- 当日统计卡片 -->
    <section v-if="dayStats" class="stats-section">
      <n-grid :cols="4" :x-gap="16">
        <n-gi>
          <div class="stat-card">
            <div class="stat-icon">
              <n-icon :component="CameraOutline" size="24" />
            </div>
            <div class="stat-value">{{ dayStats.totalCount }}</div>
            <div class="stat-label">总采集数</div>
          </div>
        </n-gi>
        <n-gi>
          <div class="stat-card">
            <div class="stat-icon">
              <n-icon :component="AppsOutline" size="24" />
            </div>
            <div class="stat-value">{{ Object.keys(dayStats.appDistribution).length }}</div>
            <div class="stat-label">应用数量</div>
          </div>
        </n-gi>
        <n-gi>
          <div class="stat-card">
            <div class="stat-icon">
              <n-icon :component="TimeOutline" size="24" />
            </div>
            <div class="stat-value">{{ formatTimeRange(dayStats.timeRange) }}</div>
            <div class="stat-label">活动时段</div>
          </div>
        </n-gi>
        <n-gi>
          <div class="stat-card">
            <div class="stat-icon">
              <n-icon :component="StatsChartOutline" size="24" />
            </div>
            <div class="stat-value">{{ Object.keys(dayStats.activityDistribution).length }}</div>
            <div class="stat-label">活动类型</div>
          </div>
        </n-gi>
      </n-grid>

      <!-- 分布图表 -->
      <n-grid :cols="2" :x-gap="16" style="margin-top: 16px">
        <n-gi>
          <div class="chart-card">
            <h4>应用分布</h4>
            <div v-if="Object.keys(dayStats.appDistribution).length > 0" class="distribution-list">
              <div
                v-for="[app, count] in sortedAppDistribution"
                :key="app"
                class="distribution-item"
              >
                <span class="app-name">{{ app || '未知应用' }}</span>
                <span class="app-count">{{ count }} 次</span>
              </div>
            </div>
            <n-empty v-else description="暂无数据" size="small" />
          </div>
        </n-gi>
        <n-gi>
          <div class="chart-card">
            <h4>活动类型</h4>
            <div v-if="Object.keys(dayStats.activityDistribution).length > 0" class="distribution-list">
              <div
                v-for="[activity, count] in sortedActivityDistribution"
                :key="activity"
                class="distribution-item"
              >
                <span class="activity-name">{{ getActivityName(activity) }}</span>
                <span class="activity-count">{{ count }} 次</span>
              </div>
            </div>
            <n-empty v-else description="暂无数据" size="small" />
          </div>
        </n-gi>
      </n-grid>
    </section>

    <!-- 上下文时间线 -->
    <section class="timeline-section">
      <div class="section-header">
        <h3>上下文时间线</h3>
        <n-tag v-if="contexts.length > 0" type="info">{{ contexts.length }} 条记录</n-tag>
      </div>

      <div v-if="loadingContexts" class="loading-state">
        <n-spin size="small" />
        <span>加载中...</span>
      </div>

      <n-empty v-else-if="contexts.length === 0" description="当日暂无上下文记录" />

      <n-timeline v-else>
        <n-timeline-item
          v-for="context in contexts"
          :key="context.id"
          :time="formatContextTime(context.capturedAt)"
          :type="getActivityColor(context.activityType)"
        >
          <template #icon>
            <n-icon :component="getActivityIcon(context.activityType)" />
          </template>
          <div class="context-item">
            <div class="context-header">
              <span class="activity-badge" :class="`badge-${context.activityType}`">
                {{ getActivityName(context.activityType) }}
              </span>
              <span v-if="context.appName" class="app-badge">{{ context.appName }}</span>
            </div>
            <div class="context-description">{{ context.description }}</div>
            <div v-if="context.windowTitle" class="context-meta">
              窗口：{{ context.windowTitle }}
            </div>
            <div v-if="context.keyContent" class="context-content">
              <strong>关键内容：</strong>{{ context.keyContent }}
            </div>
          </div>
        </n-timeline-item>
      </n-timeline>
    </section>

    <!-- 生成的日报对话框 -->
    <n-modal v-model:show="showSummaryModal" preset="card" title="生成的日报" style="width: 700px">
      <div v-if="generatedSummary" class="summary-content">
        <n-input
          v-model:value="generatedSummary"
          type="textarea"
          :rows="15"
          placeholder="日报内容"
        />
      </div>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showSummaryModal = false">关闭</n-button>
          <n-button type="primary" @click="copySummary">复制到剪贴板</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  NDatePicker,
  NSpace,
  NButton,
  NPopconfirm,
  NGrid,
  NGi,
  NIcon,
  NTag,
  NSpin,
  NEmpty,
  NTimeline,
  NTimelineItem,
  NModal,
  NInput,
  useMessage,
} from 'naive-ui';
import {
  CameraOutline,
  AppsOutline,
  TimeOutline,
  StatsChartOutline,
  CodeSlashOutline,
  GlobeOutline,
  DocumentTextOutline,
  VideocamOutline,
  ChatbubbleOutline,
  EllipsisHorizontalCircleOutline,
} from '@vicons/ionicons5';
import { contextApi } from '@/api/contextApi';
import type { DayStats, ScreenContext } from '@/types/context';

const message = useMessage();

const selectedDate = ref(Date.now());
const loadingContexts = ref(false);
const generatingSummary = ref(false);
const cleaning = ref(false);
const contexts = ref<ScreenContext[]>([]);
const dayStats = ref<DayStats | null>(null);
const showSummaryModal = ref(false);
const generatedSummary = ref('');

const sortedAppDistribution = computed(() => {
  if (!dayStats.value) return [];
  return Object.entries(dayStats.value.appDistribution).sort((a, b) => b[1] - a[1]);
});

const sortedActivityDistribution = computed(() => {
  if (!dayStats.value) return [];
  return Object.entries(dayStats.value.activityDistribution).sort((a, b) => b[1] - a[1]);
});

onMounted(() => {
  loadData();
});

async function loadData() {
  await Promise.all([loadContexts(), loadStats()]);
}

async function loadContexts() {
  loadingContexts.value = true;
  try {
    const dateStr = formatDate(selectedDate.value);
    contexts.value = await contextApi.listByDate(dateStr);
  } catch (error) {
    console.error('加载上下文失败:', error);
    message.error('加载上下文失败');
  } finally {
    loadingContexts.value = false;
  }
}

async function loadStats() {
  try {
    const dateStr = formatDate(selectedDate.value);
    dayStats.value = await contextApi.getStats(dateStr);
  } catch (error) {
    console.error('加载统计失败:', error);
  }
}

function handleDateChange() {
  loadData();
}

async function handleGenerateSummary() {
  generatingSummary.value = true;
  try {
    const dateStr = formatDate(selectedDate.value);
    generatedSummary.value = await contextApi.generateSummary(dateStr);
    showSummaryModal.value = true;
  } catch (error: any) {
    message.error(error || '生成日报失败');
    console.error('生成日报失败:', error);
  } finally {
    generatingSummary.value = false;
  }
}

async function handleCleanup() {
  cleaning.value = true;
  try {
    const count = await contextApi.cleanupOld();
    message.success(`已清理 ${count} 条过期数据`);
    await loadData();
  } catch (error: any) {
    message.error(error || '清理失败');
  } finally {
    cleaning.value = false;
  }
}

async function handleDelete() {
  try {
    const dateStr = formatDate(selectedDate.value);
    const count = await contextApi.deleteByDate(dateStr);
    message.success(`已删除 ${count} 条数据`);
    await loadData();
  } catch (error: any) {
    message.error(error || '删除失败');
  }
}

function copySummary() {
  navigator.clipboard.writeText(generatedSummary.value);
  message.success('已复制到剪贴板');
}

function formatDate(timestamp: number): string {
  // 使用本地时间格式化，避免 toISOString() 的 UTC 时区问题
  const date = new Date(timestamp);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

function formatContextTime(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

function formatTimeRange(range: [string, string] | null): string {
  if (!range) return '--:--';
  const start = new Date(range[0]).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
  const end = new Date(range[1]).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
  return `${start} - ${end}`;
}

function getActivityName(activity: string): string {
  const names: Record<string, string> = {
    coding: '编码开发',
    browsing: '网页浏览',
    document: '文档处理',
    meeting: '会议沟通',
    communication: '即时通讯',
    other: '其他',
  };
  return names[activity] || activity;
}

function getActivityIcon(activity: string) {
  const icons: Record<string, any> = {
    coding: CodeSlashOutline,
    browsing: GlobeOutline,
    document: DocumentTextOutline,
    meeting: VideocamOutline,
    communication: ChatbubbleOutline,
    other: EllipsisHorizontalCircleOutline,
  };
  return icons[activity] || EllipsisHorizontalCircleOutline;
}

function getActivityColor(activity: string): 'success' | 'info' | 'warning' | 'error' | 'default' {
  const colors: Record<string, any> = {
    coding: 'success',
    browsing: 'info',
    document: 'warning',
    meeting: 'error',
    communication: 'info',
    other: 'default',
  };
  return colors[activity] || 'default';
}
</script>

<style scoped>
.context-browser {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 24px 0;
}

.browser-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 12px;
}

.stats-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.stat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 20px;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 12px;
  text-align: center;
}

.stat-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  background: var(--accent-glow);
  border-radius: 12px;
  color: var(--accent-primary);
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
}

.stat-label {
  font-size: 12px;
  color: var(--text-muted);
}

.chart-card {
  padding: 20px;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 12px;
}

.chart-card h4 {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
}

.distribution-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-height: 200px;
  overflow-y: auto;
}

.distribution-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--bg-overlay);
  border-radius: 8px;
}

.app-name,
.activity-name {
  font-size: 13px;
  color: var(--text-secondary);
}

.app-count,
.activity-count {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent-primary);
}

.timeline-section {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 12px;
  padding: 24px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.section-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 32px;
  color: var(--text-muted);
}

.context-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  background: var(--bg-overlay);
  border-radius: 8px;
  border: 1px solid var(--border-default);
}

.context-header {
  display: flex;
  gap: 8px;
  align-items: center;
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

.app-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  background: var(--accent-glow);
  color: var(--accent-primary);
  border: 1px solid var(--accent-glow);
}

.context-description {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.context-meta {
  font-size: 12px;
  color: var(--text-dim);
}

.context-content {
  font-size: 12px;
  color: var(--text-muted);
  padding: 8px;
  background: var(--bg-surface);
  border-radius: 6px;
}

.context-content strong {
  color: var(--text-secondary);
}

.summary-content {
  padding: 16px 0;
}

/* 滚动条样式 */
.distribution-list::-webkit-scrollbar {
  width: 6px;
}

.distribution-list::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
}

.distribution-list::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 3px;
}

/* Naive UI 样式覆盖 */
:deep(.n-date-picker) {
  --n-border: 1px solid var(--input-border);
  --n-border-hover: 1px solid var(--accent-primary);
  --n-border-focus: 1px solid var(--accent-primary);
  --n-color: var(--input-bg);
  --n-text-color: var(--text-primary);
}

:deep(.n-timeline) {
  --n-icon-size: 20px;
}

:deep(.n-button--primary-type) {
  --n-color: var(--accent-primary);
  --n-color-hover: var(--accent-secondary);
  --n-text-color: white;
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2);
}
</style>
