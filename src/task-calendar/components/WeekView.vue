<template>
  <div class="week-view">
    <!-- 日期头部 -->
    <div class="day-headers">
      <div
        v-for="day in weekDays"
        :key="day.dateStr"
        class="day-header"
        :class="{ today: day.isToday, selected: day.dateStr === selectedDate }"
        @click="$emit('select-date', day.dateStr)"
      >
        <span class="weekday-name">{{ day.weekdayName }}</span>
        <span class="day-date">{{ day.date }}</span>
        <span v-if="day.taskCount > 0" class="task-badge">{{ day.taskCount }}</span>
      </div>
    </div>

    <!-- 任务列表区域 -->
    <div class="task-columns">
      <div
        v-for="day in weekDays"
        :key="day.dateStr"
        class="task-column"
        :class="{ today: day.isToday, selected: day.dateStr === selectedDate }"
        @click="$emit('select-date', day.dateStr)"
      >
        <div
          v-for="task in day.tasks"
          :key="task.id"
          class="task-item"
          :class="[task.status, `priority-${task.priority}`]"
          @click.stop="$emit('task-action', 'detail', task)"
          @contextmenu.prevent="$emit('task-action', 'context', task)"
        >
          <div class="task-status-dot" :class="task.status"></div>
          <span class="task-title">{{ task.title }}</span>
        </div>
        <div v-if="day.tasks.length === 0" class="empty-day">
          <span>-</span>
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
  (e: 'task-action', action: string, task: Task): void;
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

.day-headers {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 4px;
  margin-bottom: 8px;
  flex-shrink: 0;
}

.day-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 6px 4px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  background: rgba(255, 255, 255, 0.02);
  position: relative;
}

.day-header:hover {
  background: rgba(99, 102, 241, 0.15);
}

.day-header.today {
  background: rgba(99, 102, 241, 0.2);
  border: 1px solid rgba(99, 102, 241, 0.4);
}

.day-header.selected {
  background: rgba(99, 102, 241, 0.3);
  border: 1px solid rgba(99, 102, 241, 0.6);
}

.weekday-name {
  font-size: 10px;
  color: #64748b;
  text-transform: uppercase;
}

.day-date {
  font-size: 14px;
  font-weight: 600;
  color: #e2e8f0;
  margin-top: 2px;
}

.day-header.today .day-date {
  color: #818cf8;
}

.task-badge {
  position: absolute;
  top: 2px;
  right: 2px;
  min-width: 14px;
  height: 14px;
  background: rgba(99, 102, 241, 0.8);
  color: #fff;
  font-size: 9px;
  font-weight: 600;
  border-radius: 7px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 3px;
}

.task-columns {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 4px;
  min-height: 0;
  height: 100%;
  overflow: hidden;
}

.task-column {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 4px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.02);
  overflow-y: auto;
  cursor: pointer;
  transition: all 0.2s;
}

.task-column:hover {
  background: rgba(99, 102, 241, 0.05);
}

.task-column.today {
  background: rgba(99, 102, 241, 0.08);
}

.task-column.selected {
  background: rgba(99, 102, 241, 0.12);
  border: 1px solid rgba(99, 102, 241, 0.3);
}

.task-column::-webkit-scrollbar {
  width: 3px;
}

.task-column::-webkit-scrollbar-track {
  background: transparent;
}

.task-column::-webkit-scrollbar-thumb {
  background: rgba(99, 102, 241, 0.2);
  border-radius: 2px;
}

.task-item {
  display: flex;
  align-items: flex-start;
  gap: 4px;
  padding: 4px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.03);
  cursor: default;
  transition: all 0.2s;
}

.task-item:hover {
  background: rgba(99, 102, 241, 0.15);
}

.task-status-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  flex-shrink: 0;
  margin-top: 4px;
}

.task-status-dot.active {
  background: #6366f1;
  box-shadow: 0 0 4px #6366f1;
}

.task-status-dot.todo {
  background: #64748b;
}

.task-status-dot.done {
  background: #10b981;
  box-shadow: 0 0 4px rgba(16, 185, 129, 0.5);
}

/* 已完成任务样式 */
.task-item.done {
  opacity: 0.7;
  background: rgba(16, 185, 129, 0.08);
}

.task-item.done .task-title {
  text-decoration: line-through;
  color: #94a3b8;
}

.task-title {
  font-size: 12px;
  color: #e2e8f0;
  line-height: 1.4;
  word-break: break-word;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.task-item.priority-1 {
  border-left: 2px solid #f43f5e;
}

.task-item.priority-2 {
  border-left: 2px solid #f59e0b;
}

.task-item.priority-3 {
  border-left: 2px solid #10b981;
}

.empty-day {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #475569;
  font-size: 12px;
}
</style>
