<template>
  <div class="month-view">
    <!-- 星期头部 -->
    <div class="weekday-header">
      <div v-for="day in weekDays" :key="day" class="weekday">{{ day }}</div>
    </div>

    <!-- 日期网格 -->
    <div class="date-grid">
      <div
        v-for="(day, index) in calendarDays"
        :key="index"
        class="date-cell"
        :class="{
          'other-month': !day.isCurrentMonth,
          'today': day.isToday,
          'selected': day.dateStr === selectedDate,
          'has-tasks': day.taskCount > 0,
          'has-done-tasks': day.hasDoneTasks,
          'all-done': day.allDone
        }"
        @click="$emit('select-date', day.dateStr)"
      >
        <span class="date-number">{{ day.date }}</span>
        <div v-if="day.taskCount > 0" class="task-indicators">
          <div
            v-for="(task, i) in day.tasks.slice(0, 3)"
            :key="task.id"
            class="task-dot"
            :class="[task.status, `priority-${task.priority}`]"
            :title="task.title"
          ></div>
          <span v-if="day.taskCount > 3" class="more-count">+{{ day.taskCount - 3 }}</span>
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

const weekDays = ['一', '二', '三', '四', '五', '六', '日'];

// 格式化日期为 YYYY-MM-DD
function formatDate(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

// 获取任务显示日期
function getTaskDisplayDate(task: Task): string {
  if (task.status === 'done' && task.completedAt) {
    return task.completedAt.split(' ')[0];
  }
  if (task.displayDate) {
    return task.displayDate;
  }
  if (task.registeredAt) {
    return task.registeredAt;
  }
  if (task.createdAt) {
    return task.createdAt.split(' ')[0];
  }
  return formatDate(new Date());
}

// 计算日历天数
const calendarDays = computed(() => {
  const year = props.currentDate.getFullYear();
  const month = props.currentDate.getMonth();

  // 当月第一天
  const firstDay = new Date(year, month, 1);
  // 当月最后一天
  const lastDay = new Date(year, month + 1, 0);

  // 第一天是周几（0是周日，转换为周一开始）
  let startWeekday = firstDay.getDay();
  startWeekday = startWeekday === 0 ? 6 : startWeekday - 1;

  // 今天
  const today = formatDate(new Date());

  const days: Array<{
    date: number;
    dateStr: string;
    isCurrentMonth: boolean;
    isToday: boolean;
    tasks: Task[];
    taskCount: number;
    hasDoneTasks: boolean;
    allDone: boolean;
  }> = [];

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

  // 上月的天数
  const prevMonthLastDay = new Date(year, month, 0).getDate();
  for (let i = startWeekday - 1; i >= 0; i--) {
    const date = prevMonthLastDay - i;
    const dateStr = formatDate(new Date(year, month - 1, date));
    const dayTasks = sortTasks(props.tasks.filter(t => getTaskDisplayDate(t) === dateStr));
    const doneTasks = dayTasks.filter(t => t.status === 'done');
    days.push({
      date,
      dateStr,
      isCurrentMonth: false,
      isToday: dateStr === today,
      tasks: dayTasks,
      taskCount: dayTasks.length,
      hasDoneTasks: doneTasks.length > 0,
      allDone: dayTasks.length > 0 && doneTasks.length === dayTasks.length
    });
  }

  // 当月天数
  for (let date = 1; date <= lastDay.getDate(); date++) {
    const dateStr = formatDate(new Date(year, month, date));
    const dayTasks = sortTasks(props.tasks.filter(t => getTaskDisplayDate(t) === dateStr));
    const doneTasks = dayTasks.filter(t => t.status === 'done');
    days.push({
      date,
      dateStr,
      isCurrentMonth: true,
      isToday: dateStr === today,
      tasks: dayTasks,
      taskCount: dayTasks.length,
      hasDoneTasks: doneTasks.length > 0,
      allDone: dayTasks.length > 0 && doneTasks.length === dayTasks.length
    });
  }

  // 下月的天数（填满6行）
  const totalCells = Math.ceil(days.length / 7) * 7;
  let nextDate = 1;
  while (days.length < totalCells) {
    const dateStr = formatDate(new Date(year, month + 1, nextDate));
    const dayTasks = sortTasks(props.tasks.filter(t => getTaskDisplayDate(t) === dateStr));
    const doneTasks = dayTasks.filter(t => t.status === 'done');
    days.push({
      date: nextDate,
      dateStr,
      isCurrentMonth: false,
      isToday: dateStr === today,
      tasks: dayTasks,
      taskCount: dayTasks.length,
      hasDoneTasks: doneTasks.length > 0,
      allDone: dayTasks.length > 0 && doneTasks.length === dayTasks.length
    });
    nextDate++;
  }

  return days;
});
</script>

<style scoped>
.month-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 8px;
}

.weekday-header {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
  margin-bottom: 4px;
  flex-shrink: 0;
}

.weekday {
  text-align: center;
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  padding: 4px 0;
  text-transform: uppercase;
}

.date-grid {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  grid-template-rows: repeat(6, 1fr);
  gap: 2px;
  min-height: 0;
}

.date-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 2px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  background: rgba(255, 255, 255, 0.02);
  min-height: 0;
  overflow: hidden;
  position: relative;
}

.date-cell:hover {
  background: rgba(99, 102, 241, 0.15);
}

.date-cell.other-month {
  opacity: 0.4;
}

.date-cell.today {
  background: rgba(99, 102, 241, 0.2);
  border: 1px solid rgba(99, 102, 241, 0.4);
}

.date-cell.selected {
  background: rgba(99, 102, 241, 0.3);
  border: 1px solid rgba(99, 102, 241, 0.6);
}

.date-cell.has-tasks {
  background: rgba(99, 102, 241, 0.08);
}

/* 全部完成的日期 - 绿色标记 */
.date-cell.all-done {
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.date-cell.all-done .date-number {
  color: #10b981;
}

.date-number {
  font-size: 12px;
  font-weight: 500;
  color: #e2e8f0;
  margin-bottom: 2px;
}

.date-cell.today .date-number {
  color: #818cf8;
  font-weight: 700;
}

.date-cell.other-month .date-number {
  color: #64748b;
}

.task-indicators {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-wrap: wrap;
  justify-content: center;
}

.task-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.task-dot.active {
  background: #6366f1;
  box-shadow: 0 0 4px #6366f1;
}

.task-dot.todo {
  background: #64748b;
}

.task-dot.done {
  background: #10b981;
  box-shadow: 0 0 4px rgba(16, 185, 129, 0.5);
}

/* 已完成任务的日期单元格样式 */
.date-cell.has-done-tasks::after {
  content: '';
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 4px;
  height: 4px;
  background: #10b981;
  border-radius: 50%;
}

.task-dot.priority-1 {
  border: 1px solid #f43f5e;
}

.task-dot.priority-2 {
  border: 1px solid #f59e0b;
}

.more-count {
  font-size: 9px;
  color: #94a3b8;
}
</style>
