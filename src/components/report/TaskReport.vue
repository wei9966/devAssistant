<template>
  <div class="task-report">
    <!-- 过滤器区域 -->
    <section class="settings-card filter-section">
      <div class="card-header">
        <h3 class="card-title">任务报表</h3>
        <n-space>
          <n-button @click="handleRefresh" :loading="loading" size="small">
            <template #icon><n-icon :component="RefreshOutline" /></template>
            刷新
          </n-button>
          <n-button type="primary" @click="handleExport" size="small">
            <template #icon><n-icon :component="DownloadOutline" /></template>
            导出报表
          </n-button>
        </n-space>
      </div>
      <div class="card-content filter-controls">
        <div class="filter-row">
          <div class="filter-item">
            <span class="filter-label">时间范围</span>
            <n-date-picker
              v-model:value="dateRange"
              type="daterange"
              clearable
              style="width: 280px"
              @update:value="handleFilterChange"
            />
          </div>
          <div class="filter-item">
            <span class="filter-label">标签筛选</span>
            <n-select
              v-model:value="selectedTags"
              :options="tagOptions"
              multiple
              clearable
              placeholder="选择标签"
              style="min-width: 200px"
              @update:value="handleFilterChange"
            />
          </div>
          <div class="filter-item">
            <span class="filter-label">四象限</span>
            <n-select
              v-model:value="selectedQuadrant"
              :options="quadrantOptions"
              clearable
              placeholder="全部"
              style="width: 150px"
              @update:value="handleFilterChange"
            />
          </div>
        </div>
      </div>
    </section>

    <!-- 统计卡片 -->
    <section class="stats-section">
      <n-grid :cols="4" :x-gap="16">
        <n-gi>
          <div class="stat-card">
            <div class="stat-icon completed">
              <n-icon :component="CheckmarkCircleOutline" size="24" />
            </div>
            <div class="stat-value">{{ stats.completed }}</div>
            <div class="stat-label">已完成</div>
          </div>
        </n-gi>
        <n-gi>
          <div class="stat-card">
            <div class="stat-icon in-progress">
              <n-icon :component="HourglassOutline" size="24" />
            </div>
            <div class="stat-value">{{ stats.inProgress }}</div>
            <div class="stat-label">进行中</div>
          </div>
        </n-gi>
        <n-gi>
          <div class="stat-card">
            <div class="stat-icon pending">
              <n-icon :component="TimeOutline" size="24" />
            </div>
            <div class="stat-value">{{ stats.pending }}</div>
            <div class="stat-label">待办</div>
          </div>
        </n-gi>
        <n-gi>
          <div class="stat-card">
            <div class="stat-icon total">
              <n-icon :component="LayersOutline" size="24" />
            </div>
            <div class="stat-value">{{ stats.total }}</div>
            <div class="stat-label">总任务</div>
          </div>
        </n-gi>
      </n-grid>
    </section>

    <!-- 任务列表 -->
    <section class="settings-card task-list-section">
      <div class="card-header">
        <h3 class="card-title">已完成任务详情</h3>
        <n-tag type="success">{{ filteredTasks.length }} 条记录</n-tag>
      </div>
      <div class="card-content">
        <div v-if="loading" class="loading-state">
          <n-spin size="medium" />
          <span>加载中...</span>
        </div>
        <n-empty v-else-if="filteredTasks.length === 0" description="暂无已完成的任务" />
        <div v-else class="task-table-wrapper">
          <table class="task-table">
            <thead>
              <tr>
                <th style="width: 300px;">任务名称</th>
                <th style="width: 120px;">标签</th>
                <th style="width: 100px;">四象限</th>
                <th style="width: 150px;">开始时间</th>
                <th style="width: 150px;">完成时间</th>
                <th style="width: 100px;">耗时</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="task in filteredTasks" :key="task.id">
                <td>
                  <div class="task-name">{{ task.title }}</div>
                  <div v-if="task.description" class="task-desc-wrapper">
                    <div
                      :class="['task-desc', { 'expanded': expandedTasks.has(task.id) }]"
                      @click="toggleDescription(task.id)"
                    >
                      {{ task.description }}
                    </div>
                    <span
                      v-if="task.description.length > 50"
                      class="expand-btn"
                      @click="toggleDescription(task.id)"
                    >
                      {{ expandedTasks.has(task.id) ? '收起' : '展开' }}
                    </span>
                  </div>
                </td>
                <td>
                  <div class="tag-list">
                    <n-tag
                      v-for="tag in task.tags"
                      :key="tag.id"
                      size="small"
                      :style="{ backgroundColor: tag.color || '#6366f1', color: '#fff' }"
                    >
                      {{ tag.name }}
                    </n-tag>
                    <span v-if="!task.tags || task.tags.length === 0" class="no-tag">-</span>
                  </div>
                </td>
                <td>
                  <span class="quadrant-badge" :class="`quadrant-${task.quadrant || 4}`">
                    {{ getQuadrantName(task.quadrant) }}
                  </span>
                </td>
                <td>{{ formatDateTime(task.startedAt) }}</td>
                <td>{{ formatDateTime(task.completedAt) }}</td>
                <td>{{ calculateDuration(task.startedAt, task.completedAt) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  NButton,
  NIcon,
  NSpace,
  NDatePicker,
  NSelect,
  NGrid,
  NGi,
  NTag,
  NSpin,
  NEmpty,
  useMessage
} from 'naive-ui';
import {
  RefreshOutline,
  DownloadOutline,
  CheckmarkCircleOutline,
  HourglassOutline,
  TimeOutline,
  LayersOutline
} from '@vicons/ionicons5';
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import { taskApi } from '@/api/taskApi';
import { tagApi } from '@/api/tagApi';
import type { Task, Tag } from '@/types/task';

const message = useMessage();

const loading = ref(false);
const dateRange = ref<[number, number] | null>(null);
const selectedTags = ref<number[]>([]);
const selectedQuadrant = ref<number | null>(null);
const availableTags = ref<Tag[]>([]);
// 所有历史完成任务
const allCompletedTasks = ref<Task[]>([]);
// 展开的任务描述
const expandedTasks = ref<Set<number>>(new Set());

// 四象限选项
const quadrantOptions = [
  { label: '重要紧急', value: 1 },
  { label: '重要不紧急', value: 2 },
  { label: '不重要紧急', value: 3 },
  { label: '不重要不紧急', value: 4 }
];

// 标签选项
const tagOptions = computed(() => {
  return availableTags.value.map(tag => ({
    label: tag.name,
    value: tag.id
  }));
});

// 统计数据 - 基于所有历史完成任务
const stats = computed(() => {
  const tasks = allCompletedTasks.value;
  return {
    total: tasks.length,
    completed: tasks.length,  // 所有任务都是已完成的
    inProgress: 0,  // 报表只显示已完成任务
    pending: 0
  };
});

// 过滤后的已完成任务
const filteredTasks = computed(() => {
  let tasks = [...allCompletedTasks.value];

  // 按时间范围筛选
  if (dateRange.value) {
    const [start, end] = dateRange.value;
    tasks = tasks.filter(t => {
      if (!t.completedAt) return false;
      const completedTime = new Date(t.completedAt).getTime();
      return completedTime >= start && completedTime <= end + 86400000; // 加一天以包含结束日期
    });
  }

  // 按标签筛选
  if (selectedTags.value.length > 0) {
    tasks = tasks.filter(t =>
      t.tags?.some(tag => selectedTags.value.includes(tag.id))
    );
  }

  // 按四象限筛选
  if (selectedQuadrant.value !== null) {
    tasks = tasks.filter(t => t.quadrant === selectedQuadrant.value);
  }

  // 按完成时间倒序排序
  return tasks.sort((a, b) => {
    const timeA = a.completedAt ? new Date(a.completedAt).getTime() : 0;
    const timeB = b.completedAt ? new Date(b.completedAt).getTime() : 0;
    return timeB - timeA;
  });
});

onMounted(async () => {
  await loadData();
});

async function loadData() {
  loading.value = true;
  try {
    // 使用 days=0 获取所有历史完成任务
    allCompletedTasks.value = await taskApi.getCompletedTasks(0);
    await loadTags();
  } catch (error) {
    console.error('加载数据失败:', error);
    message.error('加载历史任务失败');
  } finally {
    loading.value = false;
  }
}

async function loadTags() {
  try {
    availableTags.value = await tagApi.getAllTags();
  } catch (error) {
    console.error('加载标签失败:', error);
  }
}

function handleRefresh() {
  loadData();
}

function handleFilterChange() {
  // 过滤条件变化时自动触发computed重新计算
}

function toggleDescription(taskId: number) {
  if (expandedTasks.value.has(taskId)) {
    expandedTasks.value.delete(taskId);
  } else {
    expandedTasks.value.add(taskId);
  }
  // 触发响应式更新
  expandedTasks.value = new Set(expandedTasks.value);
}

async function handleExport() {
  if (filteredTasks.value.length === 0) {
    message.warning('没有可导出的数据');
    return;
  }

  try {
    // 使用 Tauri 文件保存对话框选择保存位置
    const filePath = await save({
      title: '导出任务报表',
      defaultPath: `任务报表_${formatDateForFile(new Date())}.csv`,
      filters: [{
        name: 'CSV文件',
        extensions: ['csv']
      }]
    });

    if (!filePath) {
      // 用户取消了保存
      return;
    }

    // 构建CSV内容
    const headers = ['任务名称', '描述', '标签', '四象限', '开始时间', '完成时间', '耗时'];
    const rows = filteredTasks.value.map(task => [
      task.title,
      task.description || '',
      task.tags?.map(t => t.name).join('、') || '',  // 多个标签用顿号拼接
      getQuadrantName(task.quadrant),
      formatDateTime(task.startedAt),
      formatDateTime(task.completedAt),
      calculateDuration(task.startedAt, task.completedAt)
    ]);

    // 转义CSV特殊字符
    const escapeCSV = (cell: string) => {
      if (cell.includes(',') || cell.includes('"') || cell.includes('\n')) {
        return `"${cell.replace(/"/g, '""')}"`;
      }
      return cell;
    };

    const csvContent = [
      headers.map(escapeCSV).join(','),
      ...rows.map(row => row.map(escapeCSV).join(','))
    ].join('\n');

    // 添加BOM以支持中文，写入文件
    await writeTextFile(filePath, '\uFEFF' + csvContent);

    message.success(`导出成功: ${filePath}`);
  } catch (error) {
    console.error('导出失败:', error);
    message.error('导出失败: ' + (error as Error).message);
  }
}

function getQuadrantName(quadrant: number | undefined): string {
  const names: Record<number, string> = {
    1: '重要紧急',
    2: '重要不紧急',
    3: '不重要紧急',
    4: '不重要不紧急'
  };
  return names[quadrant || 4] || '未分类';
}

function formatDateTime(dateStr: string | undefined): string {
  if (!dateStr) return '-';
  const date = new Date(dateStr);
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  });
}

function formatDateForFile(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}${month}${day}`;
}

function calculateDuration(startStr: string | undefined, endStr: string | undefined): string {
  if (!startStr || !endStr) return '-';

  const start = new Date(startStr).getTime();
  const end = new Date(endStr).getTime();
  const diffMs = end - start;

  if (diffMs < 0) return '-';

  const minutes = Math.floor(diffMs / 60000);
  if (minutes < 60) {
    return `${minutes}分钟`;
  }

  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;

  if (hours < 24) {
    return remainingMinutes > 0 ? `${hours}小时${remainingMinutes}分钟` : `${hours}小时`;
  }

  const days = Math.floor(hours / 24);
  const remainingHours = hours % 24;
  return remainingHours > 0 ? `${days}天${remainingHours}小时` : `${days}天`;
}
</script>

<style scoped>
.task-report {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 24px;
}

/* 卡片样式 */
.settings-card {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 16px;
  overflow: hidden;
  backdrop-filter: blur(8px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  border-bottom: 1px solid var(--border-default);
  background: var(--bg-overlay);
}

.card-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  margin: 0;
}

.card-content {
  padding: 24px;
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
  color: var(--text-primary);
  white-space: nowrap;
}

/* 统计卡片 */
.stats-section {
  margin-bottom: 0;
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
  border-radius: 12px;
}

.stat-icon.completed {
  background: rgba(52, 211, 153, 0.15);
  color: var(--success);
}

.stat-icon.in-progress {
  background: rgba(251, 191, 36, 0.15);
  color: var(--warning);
}

.stat-icon.pending {
  background: rgba(96, 165, 250, 0.15);
  color: var(--info);
}

.stat-icon.total {
  background: var(--accent-glow);
  color: var(--accent-primary);
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
}

.stat-label {
  font-size: 12px;
  color: var(--text-primary);
  font-weight: 500;
}

/* 加载状态 */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 64px;
  color: var(--text-primary);
}

/* 任务表格 */
.task-table-wrapper {
  overflow-x: auto;
}

.task-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.task-table thead {
  background: var(--bg-overlay);
}

.task-table th {
  padding: 12px 16px;
  text-align: left;
  font-weight: 500;
  color: var(--text-primary);
  border-bottom: 1px solid var(--border-default);
}

.task-table td {
  padding: 16px;
  border-bottom: 1px solid var(--border-default);
  color: var(--text-primary);
}

.task-table tbody tr:hover {
  background: var(--bg-hover);
}

.task-name {
  font-weight: 500;
  color: var(--text-primary);
}

.task-desc-wrapper {
  margin-top: 4px;
}

.task-desc {
  font-size: 12px;
  color: var(--text-secondary);
  max-height: 40px;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  cursor: pointer;
  transition: max-height 0.3s ease;
}

.task-desc.expanded {
  max-height: none;
  -webkit-line-clamp: unset;
}

.expand-btn {
  font-size: 11px;
  color: var(--accent-primary);
  cursor: pointer;
  margin-left: 4px;
}

.expand-btn:hover {
  text-decoration: underline;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.no-tag {
  color: var(--text-secondary);
}

.quadrant-badge {
  display: inline-block;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.quadrant-1 {
  background: rgba(239, 68, 68, 0.15);
  color: var(--error);
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.quadrant-2 {
  background: rgba(251, 191, 36, 0.15);
  color: var(--warning);
  border: 1px solid rgba(251, 191, 36, 0.3);
}

.quadrant-3 {
  background: rgba(96, 165, 250, 0.15);
  color: var(--info);
  border: 1px solid rgba(96, 165, 250, 0.3);
}

.quadrant-4 {
  background: var(--bg-overlay);
  color: var(--text-secondary);
  border: 1px solid var(--border-default);
}

/* Naive UI 样式覆盖 */
:deep(.n-date-picker) {
  --n-border: 1px solid var(--input-border);
  --n-border-hover: 1px solid var(--accent-primary);
  --n-border-focus: 1px solid var(--accent-primary);
  --n-color: var(--input-bg);
  --n-text-color: var(--text-primary);
}

:deep(.n-select) {
  --n-border: 1px solid var(--input-border);
  --n-border-hover: 1px solid var(--accent-primary);
  --n-border-focus: 1px solid var(--accent-primary);
  --n-color: var(--input-bg);
  --n-text-color: var(--text-primary);
}

:deep(.n-button--primary-type) {
  --n-color: var(--accent-primary);
  --n-color-hover: var(--accent-secondary);
  --n-text-color: #ffffff;
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2);
}

/* 滚动条样式 */
.task-table-wrapper::-webkit-scrollbar {
  height: 6px;
}

.task-table-wrapper::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
}

.task-table-wrapper::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 3px;
}

.task-table-wrapper::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
}
</style>
