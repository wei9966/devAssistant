<template>
  <n-card class="task-card" :class="`status-${task.status}`" @click="handleCardClick">
    <!-- Hover Glow Effect -->
    <div class="hover-glow"></div>

    <div class="card-header">
      <div class="task-badges">
        <Badge :type="getCategoryBadgeType(task.category)">{{ CATEGORY_LABELS[task.category] || 'Other' }}</Badge>
        <!-- 四象限标识 -->
        <div
          v-if="task.quadrant"
          class="quadrant-mini-badge"
          :style="{
            background: QUADRANT_CONFIG[task.quadrant].bgColor,
            borderColor: QUADRANT_CONFIG[task.quadrant].borderColor,
            color: QUADRANT_CONFIG[task.quadrant].color
          }"
          :title="QUADRANT_CONFIG[task.quadrant].label"
        >
          <n-icon size="12">
            <GridOutline />
          </n-icon>
        </div>
      </div>
      <div class="task-actions-menu" @click.stop>
        <n-dropdown :options="dropdownOptions" @select="handleDropdownSelect">
          <n-button text size="small" class="more-btn">
            <template #icon>
              <n-icon><EllipsisHorizontal /></n-icon>
            </template>
          </n-button>
        </n-dropdown>
      </div>
    </div>

    <h3 class="task-title">{{ task.title }}</h3>

    <!-- 进行中任务显示进度 -->
    <div v-if="task.status === 'active' && (task.progress ?? 0) > 0" class="task-progress-bar">
      <n-progress
        type="line"
        :percentage="task.progress || 0"
        :show-indicator="false"
        :height="4"
        :border-radius="2"
        :rail-color="'rgba(51, 65, 85, 0.5)'"
        :color="getProgressColor(task.progress || 0)"
      />
      <span class="progress-text">{{ task.progress }}%</span>
    </div>

    <!-- 标签展示 -->
    <div v-if="task.tags && task.tags.length > 0" class="task-tags" @click.stop>
      <div
        v-for="tag in task.tags.slice(0, 3)"
        :key="tag.id"
        class="task-tag-mini"
        :style="{
          background: `${tag.color}40`,
          color: tag.color,
          borderColor: `${tag.color}60`
        }"
        :title="tag.name"
      >
        <span>{{ tag.name }}</span>
      </div>
      <div v-if="task.tags.length > 3" class="task-tag-more" :title="`还有 ${task.tags.length - 3} 个标签`">
        +{{ task.tags.length - 3 }}
      </div>
    </div>

    <div class="task-footer" @click.stop>
      <div class="priority-section">
        <div class="priority-dot" :class="`priority-${getPriorityBadgeType(task.priority)}`"></div>
        <span class="priority-label">{{ PRIORITY_LABELS[task.priority] }}</span>
      </div>
      <div class="task-actions">
        <n-button v-if="task.status === 'todo'" class="primary-button" size="small" @click="handleStart">开始</n-button>
        <n-button v-if="task.status === 'active'" type="warning" size="small" @click="handlePause">暂停</n-button>
        <n-button v-if="task.status !== 'done'" type="success" size="small" @click="handleComplete">完成</n-button>
        <n-button v-if="!readonly" type="error" text size="small" @click="handleDelete">删除</n-button>
      </div>
    </div>
  </n-card>
</template>

<script setup lang="ts">
import { ref, computed, h } from 'vue';
import { NCard, NButton, NIcon, NDropdown, NProgress, useDialog } from 'naive-ui';
import { EllipsisHorizontal, CreateOutline, TrashOutline, TimeOutline, GridOutline, TrendingUpOutline, FlagOutline } from '@vicons/ionicons5';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/zh-cn';
import { CATEGORY_LABELS, PRIORITY_LABELS, QUADRANT_CONFIG } from '@/types/task';
import type { Task } from '@/types/task';
import Badge from './Badge.vue';

dayjs.extend(relativeTime);
dayjs.locale('zh-cn');

const props = withDefaults(
  defineProps<{
    task: Task;
    readonly?: boolean;
  }>(),
  {
    readonly: false,
  }
);

const emit = defineEmits<{
  start: [taskId: number];
  pause: [taskId: number];
  complete: [taskId: number];
  edit: [task: Task];
  delete: [taskId: number];
  defer: [taskId: number];
  click: [task: Task];
  adjustProgress: [task: Task];
  addMilestone: [task: Task];
}>();

const dialog = useDialog();

const dropdownOptions = computed(() => {
  const options = [];

  if (!props.readonly) {
    options.push({
      label: '编辑',
      key: 'edit',
      icon: () => h(NIcon, null, { default: () => h(CreateOutline) })
    });

    // 进行中任务可以调整进度和添加里程碑
    if (props.task.status === 'active') {
      options.push({
        label: '调整进度',
        key: 'adjustProgress',
        icon: () => h(NIcon, null, { default: () => h(TrendingUpOutline) })
      });
      options.push({
        label: '添加里程碑',
        key: 'addMilestone',
        icon: () => h(NIcon, null, { default: () => h(FlagOutline) })
      });
    }

    if (props.task.status === 'todo') {
      options.push({
        label: '延后',
        key: 'defer',
        icon: () => h(NIcon, null, { default: () => h(TimeOutline) })
      });
    }

    options.push({
      label: '删除',
      key: 'delete',
      icon: () => h(NIcon, null, { default: () => h(TrashOutline) })
    });
  }

  return options;
});

const isStale = computed(() => {
  if (props.task.status !== 'todo') return false;
  if (!props.task.createdAt) return false;
  const days = dayjs().diff(dayjs(props.task.createdAt), 'day');
  return days > 3;
});

const handleStart = () => {
  emit('start', props.task.id!);
};

const handlePause = () => {
  emit('pause', props.task.id!);
};

const handleComplete = () => {
  emit('complete', props.task.id!);
};

const handleEdit = () => {
  emit('edit', props.task);
};

const handleDelete = () => {
  dialog.warning({
    title: '删除任务',
    content: `确定要删除任务 "${props.task.title}" 吗？`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: () => {
      emit('delete', props.task.id!);
    },
  });
};

const handleCardClick = () => {
  emit('click', props.task);
};

const handleDropdownSelect = (key: string) => {
  switch (key) {
    case 'edit':
      handleEdit();
      break;
    case 'defer':
      emit('defer', props.task.id!);
      break;
    case 'delete':
      handleDelete();
      break;
    case 'adjustProgress':
      emit('adjustProgress', props.task);
      break;
    case 'addMilestone':
      emit('addMilestone', props.task);
      break;
  }
};

const formatTime = (time: string) => {
  return dayjs(time).fromNow();
};

const getCategoryBadgeType = (category: string | undefined) => {
  if (!category) return 'default';
  const categoryMap: Record<string, 'Backend' | 'Database' | 'Feature' | 'Docs' | 'default'> = {
    'backend': 'Backend',
    'database': 'Database',
    'feature': 'Feature',
    'docs': 'Docs',
  };
  return categoryMap[category] || 'default';
};

const getPriorityBadgeType = (priority: number) => {
  const priorityMap: Record<number, 'high' | 'medium' | 'low'> = {
    3: 'high',
    2: 'medium',
    1: 'low',
  };
  return priorityMap[priority] || 'medium';
};

const getProgressColor = (progress: number) => {
  if (progress >= 80) return '#10b981'; // green
  if (progress >= 50) return '#6366f1'; // indigo
  if (progress >= 20) return '#f59e0b'; // amber
  return '#94a3b8'; // gray
};
</script>

<style scoped>
.task-card {
  margin-bottom: 0;
  background: rgba(30, 41, 59, 0.4);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 14px !important;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: visible;
  cursor: pointer;
  height: auto;
}

/* 确保 n-card 内部内容正确布局 */
.task-card :deep(.n-card__content) {
  display: flex;
  flex-direction: column;
  padding: 16px !important;
  height: auto;
}

.task-card:hover {
  background: rgba(30, 41, 59, 0.8);
  border-color: rgba(71, 85, 105, 0.6);
  box-shadow: 0 10px 20px -5px rgba(0, 0, 0, 0.3), 0 4px 6px -2px rgba(0, 0, 0, 0.2);
  transform: translateY(-2px);
}

/* Hover Glow Effect */
.hover-glow {
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent 0%, rgba(255, 255, 255, 0.05) 50%, transparent 100%);
  transform: translateX(-100%);
  transition: transform 0.7s cubic-bezier(0.4, 0, 0.2, 1);
  pointer-events: none;
}

.task-card:hover .hover-glow {
  transform: translateX(100%);
}

.task-card.status-active {
  border-left: 3px solid #6366f1;
  background: rgba(30, 41, 59, 0.5);
}

.task-card.status-done {
  opacity: 0.7;
}

.task-card.status-done:hover {
  opacity: 0.85;
}

.task-card.status-deferred {
  border-left: 3px solid #f59e0b;
  opacity: 0.75;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.task-badges {
  display: flex;
  gap: 6px;
  align-items: center;
}

.quadrant-mini-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 6px;
  border: 1px solid;
  flex-shrink: 0;
  transition: all 0.2s;
}

.quadrant-mini-badge:hover {
  transform: scale(1.1);
}

.task-actions-menu {
  opacity: 0.5;
  transition: opacity 0.2s;
}

.task-card:hover .task-actions-menu {
  opacity: 1;
}

.more-btn {
  color: #94a3b8; /* slate-400 */
  transition: color 0.2s;
}

.more-btn:hover {
  color: #e2e8f0; /* slate-200 */
}

.task-title {
  font-size: 14px;
  font-weight: 500;
  color: #e2e8f0;
  line-height: 1.6;
  margin: 0 0 8px 0;
  flex-shrink: 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
}

/* 进度条 */
.task-progress-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.task-progress-bar :deep(.n-progress) {
  flex: 1;
}

.progress-text {
  font-size: 11px;
  font-weight: 600;
  color: #6366f1;
  min-width: 32px;
  text-align: right;
}

/* 标签展示 */
.task-tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  align-items: center;
  margin-bottom: 8px;
}

.task-tag-mini {
  display: inline-flex;
  align-items: center;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  border: 1px solid;
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: all 0.2s;
}

.task-tag-mini:hover {
  transform: translateY(-1px);
  max-width: none;
}

.task-tag-more {
  display: inline-flex;
  align-items: center;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  background: rgba(100, 116, 139, 0.2);
  color: #94a3b8;
  border: 1px solid rgba(100, 116, 139, 0.4);
  cursor: help;
}

.task-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid rgba(51, 65, 85, 0.3);
  flex-shrink: 0;
}

.priority-section {
  display: flex;
  align-items: center;
  gap: 8px;
}

.priority-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.priority-dot.priority-high {
  background: #f43f5e;
  box-shadow: 0 0 6px rgba(244, 63, 94, 0.5);
}

.priority-dot.priority-medium {
  background: #f59e0b;
  box-shadow: 0 0 6px rgba(245, 158, 11, 0.5);
}

.priority-dot.priority-low {
  background: #10b981;
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.5);
}

.priority-label {
  font-size: 11px;
  color: #64748b;
  text-transform: uppercase;
  font-weight: 600;
  letter-spacing: 0.05em;
}

.task-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  opacity: 0.6;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  align-items: center;
}

/* 响应式优化: 在小屏幕上保持按钮可见 */
@media (max-width: 1200px) {
  .task-actions {
    opacity: 1;
  }

  .task-footer {
    gap: 6px;
  }

  .priority-section {
    flex-shrink: 0;
  }
}

.task-card:hover .task-actions {
  opacity: 1;
  transform: scale(1.05);
}

/* 主要按钮样式 */
:deep(.primary-button) {
  background-color: #6366f1;
  border-color: #6366f1;
  color: #ffffff;
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2);
  transition: all 0.2s;
}

:deep(.primary-button:hover) {
  background-color: #4f46e5;
  border-color: #4f46e5;
}

:deep(.primary-button:active) {
  background-color: #4338ca;
  border-color: #4338ca;
}
</style>
