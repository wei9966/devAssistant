<template>
  <div class="week-view">
    <!-- 看板式7列布局 -->
    <div class="kanban-columns">
      <div
        v-for="day in weekDays"
        :key="day.dateStr"
        class="kanban-column"
        :class="{
          today: day.isToday,
          selected: day.dateStr === selectedDate
        }"
        @click="$emit('select-date', day.dateStr)"
        @contextmenu.prevent="(e) => $emit('add-task', day.dateStr, e)"
      >
        <!-- 列头部 -->
        <div class="column-header" :class="{ today: day.isToday }">
          <span class="weekday-name">{{ day.weekdayName }}</span>
          <span
            class="day-date"
            :class="{
              'is-today': day.isToday,
              'is-selected': day.dateStr === selectedDate && !day.isToday
            }"
          >
            {{ day.date }}
          </span>
        </div>

        <!-- 任务列表 -->
        <div class="task-list">
          <div
            v-for="task in day.tasks"
            :key="task.id"
            class="task-card"
            :class="[task.status]"
            @click.stop="$emit('task-action', 'detail', task)"
            @contextmenu.prevent.stop="(e) => $emit('task-action', 'context', task, e)"
          >
            <!-- 任务头部：优先级标签 + 完成状态 -->
            <div class="task-card-header">
              <span
                class="priority-tag"
                :class="`priority-${task.priority}`"
              >
                {{ task.priority === 1 ? 'P0' : task.priority === 2 ? 'P1' : 'P2' }}
              </span>
              <svg
                v-if="task.status === 'done'"
                class="done-icon"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                <polyline points="22 4 12 14.01 9 11.01"></polyline>
              </svg>
            </div>
            <!-- 任务标题 -->
            <p class="task-title" :class="{ done: task.status === 'done' }">
              {{ task.title }}
            </p>
          </div>

          <!-- 空状态 -->
          <div v-if="day.tasks.length === 0" class="empty-column">
            <svg class="add-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Task } from '@/types/task';

const props = defineProps<{
  currentDate: Date;
  tasks: Task[];
  selectedDate: string | null;
}>();

const emit = defineEmits<{
  (e: 'select-date', date: string): void;
  (e: 'task-action', action: string, task: Task, event?: MouseEvent): void;
  (e: 'add-task', date: string, event: MouseEvent): void;
}>();

const weekdayNames = ['周一', '周二', '周三', '周四', '周五', '周六', '周日'];

// 格式化日期为 YYYY-MM-DD
function formatDate(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

// 获取周的第一天（周一）
function getWeekStart(date: Date): Date {
  const d = new Date(date);
  const day = d.getDay();
  const diff = d.getDate() - day + (day === 0 ? -6 : 1);
  d.setDate(diff);
  return d;
}

// 获取任务显示日期
// 逻辑：已完成任务按完成日期显示，未完成任务显示在今天（提醒用户去完成）
function getTaskDisplayDate(task: Task): string {
  const today = formatDate(new Date());

  // 已完成任务：按完成日期显示
  if (task.status === 'done' && task.completedAt) {
    return task.completedAt.split(' ')[0];
  }

  // 未完成任务：如果有 displayDate 且是今天或未来，使用它；否则显示在今天
  if (task.status !== 'done') {
    if (task.displayDate && task.displayDate >= today) {
      return task.displayDate;
    }
    return today;
  }

  // 兜底逻辑
  if (task.displayDate) {
    return task.displayDate;
  }
  if (task.registeredAt) {
    return task.registeredAt;
  }
  if (task.createdAt) {
    return task.createdAt.split(' ')[0];
  }
  return today;
}

// 任务排序函数：未完成的在前，完成的在后
const sortTasks = (tasks: Task[]) => {
  return [...tasks].sort((a, b) => {
    // 完成的放后面
    if (a.status === 'done' && b.status !== 'done') return 1;
    if (a.status !== 'done' && b.status === 'done') return -1;
    // 同状态按优先级排序
    return (a.priority || 3) - (b.priority || 3);
  });
};

// 计算周视图的每一天
const weekDays = computed(() => {
  const weekStart = getWeekStart(props.currentDate);
  const today = formatDate(new Date());

  const days: Array<{
    date: number;
    dateStr: string;
    weekdayName: string;
    isToday: boolean;
    tasks: Task[];
    taskCount: number;
  }> = [];

  for (let i = 0; i < 7; i++) {
    const date = new Date(weekStart);
    date.setDate(date.getDate() + i);
    const dateStr = formatDate(date);
    const dayTasks = sortTasks(props.tasks.filter(t => getTaskDisplayDate(t) === dateStr));

    days.push({
      date: date.getDate(),
      dateStr,
      weekdayName: weekdayNames[i],
      isToday: dateStr === today,
      tasks: dayTasks,
      taskCount: dayTasks.length
    });
  }

  return days;
});
</script>

<style scoped>
.week-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 8px;
}

/* 看板式7列布局 */
.kanban-columns {
  flex: 1;
  display: flex;
  gap: 8px;
  min-height: 0;
  overflow: hidden;
}

/* 每一列（每一天） */
.kanban-column {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 12px;
  transition: all 0.3s ease;
  cursor: pointer;
  overflow: hidden;
}

.kanban-column:hover {
  background: var(--bg-hover);
  border-color: var(--border-hover);
}

/* 今天的列 */
.kanban-column.today {
  background: rgba(16, 185, 129, 0.05);
  border-color: rgba(16, 185, 129, 0.3);
}

/* 选中的列 */
.kanban-column.selected {
  background: rgba(99, 102, 241, 0.12);
  border-color: rgba(99, 102, 241, 0.5);
  box-shadow: 0 0 20px rgba(99, 102, 241, 0.15);
}

/* 列头部 */
.column-header {
  padding: 12px 8px;
  text-align: center;
  border-bottom: 1px solid var(--card-border);
  flex-shrink: 0;
}

.column-header.today {
  background: linear-gradient(180deg, rgba(16, 185, 129, 0.1) 0%, transparent 100%);
}

.weekday-name {
  display: block;
  font-size: 11px;
  color: var(--text-secondary);
  margin-bottom: 6px;
  font-weight: 500;
}

.day-date {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  border-radius: 50%;
  transition: all 0.2s;
}

/* 今天的日期圆圈 */
.day-date.is-today {
  background: #10b981;
  color: #ffffff;
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
}

/* 选中的日期圆圈（非今天） */
.day-date.is-selected {
  background: #6366f1;
  color: #ffffff;
}

/* 任务列表区域 */
.task-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-list::-webkit-scrollbar {
  width: 4px;
}

.task-list::-webkit-scrollbar-track {
  background: transparent;
}

.task-list::-webkit-scrollbar-thumb {
  background: rgba(99, 102, 241, 0.3);
  border-radius: 2px;
}

.task-list::-webkit-scrollbar-thumb:hover {
  background: rgba(99, 102, 241, 0.5);
}

/* 任务卡片 */
.task-card {
  background: var(--bg-elevated);
  border: 1px solid var(--card-border);
  border-radius: 8px;
  padding: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.task-card:hover {
  border-color: rgba(99, 102, 241, 0.4);
  transform: translateY(-1px);
}

/* 已完成任务卡片 */
.task-card.done {
  opacity: 0.7;
  background: rgba(16, 185, 129, 0.08);
}

/* 任务卡片头部 */
.task-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

/* 优先级标签 */
.priority-tag {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 4px;
}

.priority-tag.priority-1 {
  background: rgba(244, 63, 94, 0.2);
  color: #f43f5e;
}

.priority-tag.priority-2 {
  background: rgba(245, 158, 11, 0.2);
  color: #f59e0b;
}

.priority-tag.priority-3 {
  background: rgba(100, 116, 139, 0.3);
  color: #94a3b8;
}

/* 完成图标 */
.done-icon {
  width: 14px;
  height: 14px;
  color: #10b981;
}

/* 任务标题 */
.task-title {
  font-size: 12px;
  color: var(--text-primary);
  line-height: 1.4;
  margin: 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
}

.task-title.done {
  color: var(--text-secondary);
  text-decoration: line-through;
}

/* 空状态 */
.empty-column {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 60px;
  opacity: 0;
  transition: opacity 0.2s;
}

.kanban-column:hover .empty-column {
  opacity: 1;
}

.add-icon {
  width: 24px;
  height: 24px;
  color: var(--text-secondary);
  transition: color 0.2s;
}

.add-icon:hover {
  color: #6366f1;
}
</style>
