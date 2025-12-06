<template>
  <div class="predicted-task-list">
    <!-- Header -->
    <div class="list-header">
      <div class="header-left">
        <h3 class="list-title">
          <n-icon :component="BulbOutline" size="18" />
          AI 预测任务
        </h3>
        <n-badge :value="pendingCount" :max="99" type="info" v-if="pendingCount > 0" />
      </div>
      <div class="header-actions">
        <n-button size="small" @click="handleRefresh" :loading="loading">
          <template #icon>
            <n-icon :component="RefreshOutline" />
          </template>
          刷新
        </n-button>
        <n-button
          v-if="predictions.length > 0"
          size="small"
          :type="isAllSelected ? 'primary' : 'default'"
          @click="toggleSelectAll"
        >
          <template #icon>
            <n-icon :component="isAllSelected ? CheckboxOutline : SquareOutline" />
          </template>
          {{ isAllSelected ? '取消全选' : '全选' }}
        </n-button>
        <n-button
          v-if="selectedIds.length > 0"
          size="small"
          type="primary"
          @click="handleBatchAccept"
          :loading="batchAccepting"
        >
          <template #icon>
            <n-icon :component="CheckmarkOutline" />
          </template>
          批量添加 ({{ selectedIds.length }})
        </n-button>
        <n-button
          v-if="selectedIds.length > 0"
          size="small"
          @click="handleBatchIgnore"
          :loading="batchIgnoring"
        >
          <template #icon>
            <n-icon :component="CloseOutline" />
          </template>
          批量忽略
        </n-button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading && predictions.length === 0" class="loading-state">
      <n-spin size="medium" />
      <span>加载预测任务...</span>
    </div>

    <!-- Empty State -->
    <div v-else-if="predictions.length === 0" class="empty-state">
      <n-empty description="暂无 AI 预测任务">
        <template #extra>
          <n-text depth="3">AI 会根据您的活动自动预测可能需要处理的任务</n-text>
        </template>
      </n-empty>
    </div>

    <!-- Prediction List -->
    <div v-else class="prediction-items">
      <div
        v-for="item in predictions"
        :key="item.id"
        class="prediction-card"
        :class="{ 'selected': selectedIds.includes(item.id!) }"
      >
        <div class="card-checkbox">
          <n-checkbox
            :checked="selectedIds.includes(item.id!)"
            @update:checked="(checked) => toggleSelect(item.id!, checked)"
          />
        </div>

        <div class="card-content">
          <div class="card-header">
            <div class="prediction-badges">
              <n-tag :type="getPriorityType(item.priority)" size="small" round>
                {{ getPriorityLabel(item.priority) }}
              </n-tag>
              <n-tag v-if="item.dueDate" type="warning" size="small" round>
                <n-icon :component="TimeOutline" size="12" />
                {{ formatDueDate(item.dueDate) }}
              </n-tag>
            </div>
            <span class="prediction-time">{{ formatTime(item.createdAt) }}</span>
          </div>

          <h4 class="prediction-title">{{ item.description }}</h4>

          <p v-if="item.reason" class="prediction-reason">
            <n-icon :component="InformationCircleOutline" size="14" />
            {{ item.reason }}
          </p>
        </div>

        <div class="card-actions">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button
                circle
                type="primary"
                size="small"
                @click="handleAccept(item)"
                :loading="acceptingId === item.id"
              >
                <template #icon>
                  <n-icon :component="CheckmarkOutline" />
                </template>
              </n-button>
            </template>
            添加到任务
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button
                circle
                size="small"
                @click="handleIgnore(item)"
                :loading="ignoringId === item.id"
              >
                <template #icon>
                  <n-icon :component="CloseOutline" />
                </template>
              </n-button>
            </template>
            忽略
          </n-tooltip>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { NIcon, NBadge, NButton, NCheckbox, NTag, NTooltip, NSpin, NEmpty, NText, useMessage } from 'naive-ui';
import { BulbOutline, RefreshOutline, CheckmarkOutline, CloseOutline, TimeOutline, InformationCircleOutline, CheckboxOutline, SquareOutline } from '@vicons/ionicons5';
import { invoke } from '@tauri-apps/api/core';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/zh-cn';

dayjs.extend(relativeTime);
dayjs.locale('zh-cn');

interface PredictedTask {
  id?: number;
  description: string;
  reason: string;
  priority: string;
  dueDate?: string;
  status?: string;
  taskId?: number;
  createdAt?: string;
}

const emit = defineEmits<{
  taskAccepted: [taskId: number];
  refresh: [];
}>();

const message = useMessage();

const predictions = ref<PredictedTask[]>([]);
const pendingCount = ref(0);
const loading = ref(false);
const selectedIds = ref<number[]>([]);
const acceptingId = ref<number | null>(null);
const ignoringId = ref<number | null>(null);
const batchAccepting = ref(false);
const batchIgnoring = ref(false);

// 是否全选
const isAllSelected = computed(() => {
  return predictions.value.length > 0 && selectedIds.value.length === predictions.value.length;
});

// 是否部分选中
const isIndeterminate = computed(() => {
  return selectedIds.value.length > 0 && selectedIds.value.length < predictions.value.length;
});

// 全选/取消全选
function toggleSelectAll() {
  if (isAllSelected.value) {
    selectedIds.value = [];
  } else {
    selectedIds.value = predictions.value.map(item => item.id!).filter(id => id !== undefined);
  }
}

// 加载预测任务
async function loadPredictions() {
  loading.value = true;
  try {
    const result = await invoke<any[]>('get_pending_predictions');
    predictions.value = result.map(item => ({
      id: item.id,
      description: item.description,
      reason: item.reason,
      priority: item.priority,
      dueDate: item.due_date,
      status: item.status,
      taskId: item.task_id,
      createdAt: item.created_at,
    }));
    pendingCount.value = predictions.value.length;
    selectedIds.value = [];
  } catch (error) {
    console.error('加载预测任务失败:', error);
    message.error('加载预测任务失败');
  } finally {
    loading.value = false;
  }
}

// 接受预测任务
async function handleAccept(item: PredictedTask) {
  if (!item.id) return;
  acceptingId.value = item.id;
  try {
    const taskId = await invoke<number>('accept_prediction', { predictionId: item.id });
    message.success('已添加到任务列表');
    emit('taskAccepted', taskId);
    await loadPredictions();
  } catch (error: any) {
    console.error('接受预测任务失败:', error);
    message.error(error || '添加任务失败');
  } finally {
    acceptingId.value = null;
  }
}

// 忽略预测任务
async function handleIgnore(item: PredictedTask) {
  if (!item.id) return;
  ignoringId.value = item.id;
  try {
    await invoke('ignore_prediction', { predictionId: item.id });
    message.success('已忽略该预测');
    await loadPredictions();
  } catch (error: any) {
    console.error('忽略预测任务失败:', error);
    message.error(error || '忽略失败');
  } finally {
    ignoringId.value = null;
  }
}

// 批量接受
async function handleBatchAccept() {
  if (selectedIds.value.length === 0) return;
  batchAccepting.value = true;
  try {
    const taskIds = await invoke<number[]>('accept_predictions', { predictionIds: selectedIds.value });
    message.success(`已添加 ${taskIds.length} 个任务`);
    taskIds.forEach(id => emit('taskAccepted', id));
    await loadPredictions();
  } catch (error: any) {
    console.error('批量接受预测任务失败:', error);
    message.error(error || '批量添加失败');
  } finally {
    batchAccepting.value = false;
  }
}

// 批量忽略
async function handleBatchIgnore() {
  if (selectedIds.value.length === 0) return;
  batchIgnoring.value = true;
  try {
    await invoke('ignore_predictions', { predictionIds: selectedIds.value });
    message.success(`已忽略 ${selectedIds.value.length} 个预测`);
    await loadPredictions();
  } catch (error: any) {
    console.error('批量忽略预测任务失败:', error);
    message.error(error || '批量忽略失败');
  } finally {
    batchIgnoring.value = false;
  }
}

// 选择/取消选择
function toggleSelect(id: number, checked: boolean) {
  if (checked) {
    selectedIds.value.push(id);
  } else {
    selectedIds.value = selectedIds.value.filter(i => i !== id);
  }
}

// 刷新
function handleRefresh() {
  loadPredictions();
  emit('refresh');
}

// 格式化优先级
function getPriorityType(priority: string): 'error' | 'warning' | 'success' {
  const map: Record<string, 'error' | 'warning' | 'success'> = {
    'high': 'error',
    'medium': 'warning',
    'low': 'success',
  };
  return map[priority] || 'warning';
}

function getPriorityLabel(priority: string): string {
  const map: Record<string, string> = {
    'high': '高优先级',
    'medium': '中优先级',
    'low': '低优先级',
  };
  return map[priority] || '中优先级';
}

// 格式化时间
function formatTime(time?: string): string {
  if (!time) return '';
  return dayjs(time).fromNow();
}

function formatDueDate(date: string): string {
  const d = dayjs(date);
  if (d.isSame(dayjs(), 'day')) return '今天';
  if (d.isSame(dayjs().add(1, 'day'), 'day')) return '明天';
  return d.format('MM/DD');
}

// 暴露方法供父组件调用
defineExpose({
  loadPredictions,
  pendingCount,
});

onMounted(() => {
  loadPredictions();
});
</script>

<style scoped>
.predicted-task-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(51, 65, 85, 0.4);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.list-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: rgb(226, 232, 240);
}

.header-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 40px 20px;
  color: rgb(148, 163, 184);
}

.prediction-items {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.prediction-card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px;
  background: rgba(30, 41, 59, 0.4);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 12px;
  transition: all 0.2s;
}

.prediction-card:hover {
  background: rgba(30, 41, 59, 0.6);
  border-color: rgba(71, 85, 105, 0.6);
}

.prediction-card.selected {
  border-color: rgba(99, 102, 241, 0.5);
  background: rgba(99, 102, 241, 0.1);
}

.card-checkbox {
  padding-top: 2px;
}

.card-content {
  flex: 1;
  min-width: 0;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.prediction-badges {
  display: flex;
  gap: 6px;
  align-items: center;
}

.prediction-time {
  font-size: 12px;
  color: rgb(100, 116, 139);
}

.prediction-title {
  margin: 0 0 8px 0;
  font-size: 14px;
  font-weight: 500;
  color: rgb(226, 232, 240);
  line-height: 1.5;
}

.prediction-reason {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  margin: 0;
  font-size: 12px;
  color: rgb(148, 163, 184);
  line-height: 1.5;
}

.prediction-reason .n-icon {
  flex-shrink: 0;
  margin-top: 2px;
}

.card-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
</style>
