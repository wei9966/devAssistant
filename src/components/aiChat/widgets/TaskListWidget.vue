<template>
  <div class="widget-container task-list-widget">
    <div class="widget-header">
      <div class="widget-title">
        <n-icon size="20" class="widget-icon">
          <CheckboxOutline />
        </n-icon>
        <span>任务列表</span>
      </div>
      <div class="widget-stats">
        共 {{ data.total }} 项任务
      </div>
    </div>

    <div class="widget-content">
      <!-- 按象限分组 -->
      <div v-for="(quadrant, key) in groupedTasks" :key="key" class="quadrant-section">
        <div class="quadrant-header" :style="{ borderColor: QUADRANT_CONFIG[key].borderColor }">
          <n-icon size="16" :color="QUADRANT_CONFIG[key].color">
            <GridOutline />
          </n-icon>
          <span class="quadrant-title" :style="{ color: QUADRANT_CONFIG[key].color }">
            {{ QUADRANT_CONFIG[key].shortLabel }}
          </span>
          <span class="quadrant-count">{{ quadrant.length }}</span>
        </div>

        <div class="task-list">
          <div v-for="task in quadrant" :key="task.id" class="task-item">
            <div class="task-item-header">
              <div class="task-status-dot" :class="`status-${task.status}`"></div>
              <span class="task-title">{{ task.title }}</span>
            </div>

            <div class="task-item-meta">
              <div v-if="task.due_date" class="task-meta-item">
                <n-icon size="12">
                  <TimeOutline />
                </n-icon>
                <span>{{ formatDueDate(task.due_date) }}</span>
              </div>

              <div v-if="task.tags && task.tags.length > 0" class="task-tags">
                <span
                  v-for="(tag, index) in task.tags.slice(0, 2)"
                  :key="index"
                  class="task-tag"
                >
                  {{ tag }}
                </span>
                <span v-if="task.tags.length > 2" class="task-tag-more">
                  +{{ task.tags.length - 2 }}
                </span>
              </div>
            </div>

            <div v-if="task.status !== 'done'" class="task-actions">
              <n-button size="tiny" type="success" @click="handleComplete(task)">
                <template #icon>
                  <n-icon><CheckmarkOutline /></n-icon>
                </template>
                完成
              </n-button>
            </div>
          </div>
        </div>
      </div>

      <div v-if="data.total === 0" class="empty-state">
        <n-icon size="48" class="empty-icon">
          <CheckboxOutline />
        </n-icon>
        <p>暂无任务</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NButton, NIcon } from 'naive-ui';
import { CheckboxOutline, GridOutline, TimeOutline, CheckmarkOutline } from '@vicons/ionicons5';
import { QUADRANT_CONFIG, type TaskQuadrant } from '@/types/task';
import dayjs from 'dayjs';

interface TaskData {
  id: string | number;
  title: string;
  description?: string;
  quadrant: TaskQuadrant;
  status: string;
  due_date?: string;
  completed_at?: string;
  tags?: string[];
  pomodoro_count?: number;
  focus_minutes?: number;
}

interface TasksResponse {
  total: number;
  tasks: TaskData[];
}

const props = defineProps<{
  data: TasksResponse;
}>();

const emit = defineEmits<{
  complete: [task: TaskData];
}>();

// 按象限分组
const groupedTasks = computed(() => {
  const groups: Record<TaskQuadrant, TaskData[]> = {
    urgent_important: [],
    urgent_not_important: [],
    not_urgent_important: [],
    not_urgent_not_important: [],
  };

  props.data.tasks.forEach((task) => {
    if (task.quadrant) {
      groups[task.quadrant].push(task);
    }
  });

  // 只返回有任务的象限
  return Object.fromEntries(
    Object.entries(groups).filter(([_, tasks]) => tasks.length > 0)
  ) as Record<TaskQuadrant, TaskData[]>;
});

const formatDueDate = (date: string) => {
  const now = dayjs();
  const dueDate = dayjs(date);
  const diffDays = dueDate.diff(now, 'day');

  if (diffDays < 0) return `逾期 ${Math.abs(diffDays)} 天`;
  if (diffDays === 0) return '今天到期';
  if (diffDays === 1) return '明天到期';
  if (diffDays <= 7) return `${diffDays} 天后到期`;
  return dueDate.format('MM-DD');
};

const handleComplete = (task: TaskData) => {
  emit('complete', task);
};
</script>

<style scoped>
.widget-container {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 12px;
  padding: 16px;
  transition: all 0.3s;
}

.widget-container:hover {
  background: var(--card-hover-bg);
  border-color: var(--card-hover-border);
  box-shadow: var(--shadow-md);
}

.widget-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-default);
}

.widget-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.widget-icon {
  color: var(--accent-primary);
}

.widget-stats {
  font-size: 12px;
  color: var(--text-muted);
  padding: 4px 12px;
  background: var(--bg-elevated);
  border-radius: 12px;
  border: 1px solid var(--border-default);
}

.widget-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.quadrant-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.quadrant-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: var(--bg-surface);
  border-radius: 8px;
  border-left: 3px solid;
}

.quadrant-title {
  font-size: 13px;
  font-weight: 600;
  flex: 1;
}

.quadrant-count {
  font-size: 11px;
  color: var(--text-muted);
  padding: 2px 8px;
  background: var(--bg-elevated);
  border-radius: 10px;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-item {
  padding: 10px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  transition: all 0.2s;
  cursor: pointer;
}

.task-item:hover {
  background: var(--bg-elevated);
  border-color: var(--border-hover);
  transform: translateX(2px);
}

.task-item-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.task-status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.task-status-dot.status-todo {
  background: var(--text-dim);
}

.task-status-dot.status-active {
  background: var(--accent-primary);
  box-shadow: 0 0 6px var(--accent-glow);
}

.task-status-dot.status-done {
  background: var(--success);
}

.task-title {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 500;
  flex: 1;
  line-height: 1.4;
}

.task-item-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.task-meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
}

.task-tags {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}

.task-tag {
  font-size: 10px;
  padding: 2px 6px;
  background: var(--accent-glow);
  color: var(--accent-primary);
  border-radius: 4px;
  border: 1px solid var(--accent-primary);
}

.task-tag-more {
  font-size: 10px;
  padding: 2px 6px;
  background: var(--bg-elevated);
  color: var(--text-muted);
  border-radius: 4px;
  border: 1px solid var(--border-default);
}

.task-actions {
  display: flex;
  gap: 6px;
  justify-content: flex-end;
  opacity: 0.7;
  transition: opacity 0.2s;
}

.task-item:hover .task-actions {
  opacity: 1;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--text-dim);
}

.empty-icon {
  opacity: 0.3;
  margin-bottom: 12px;
}

.empty-state p {
  margin: 0;
  font-size: 14px;
}
</style>
