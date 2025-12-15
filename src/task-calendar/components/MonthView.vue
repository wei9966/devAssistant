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
        <div class="date-header">
          <span class="date-number">{{ day.date }}</span>
          <span v-if="day.taskCount > 0" class="task-count-badge">{{ day.taskCount }}</span>
        </div>

        <div v-if="day.taskCount > 0" class="task-list">
          <div
            v-for="(task, i) in day.tasks.slice(0, 2)"
            :key="task.id"
            class="task-item"
            :title="task.title"
          >
            <div
              class="priority-indicator"
              :class="[task.status, `priority-${task.priority}`]"
            ></div>
            <span class="task-title">{{ task.title }}</span>
          </div>
          <div v-if="day.taskCount > 2" class="more-tasks">
            +{{ day.taskCount - 2 }} 更多
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
  gap: 4px;
  margin-bottom: 8px;
  flex-shrink: 0;
}

.weekday {
  text-align: center;
  font-size: 12px;
  font-weight: 600;
  color: #64748b;
  padding: 6px 0;
  text-transform: uppercase;
}

.date-grid {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  grid-template-rows: repeat(6, 1fr);
  gap: 4px;
  min-height: 0;
  height: 100%;
}

.date-cell {
  display: flex;
  flex-direction: column;
  padding: 8px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.05);
  min-height: 80px;
  overflow: hidden;
  position: relative;
}

.date-cell:hover {
  background: rgba(99, 102, 241, 0.1);
  border-color: rgba(99, 102, 241, 0.2);
}

/* 非当月日期半透明 */
.date-cell.other-month {
  opacity: 0.5;
}

/* 今天的样式 */
.date-cell.today {
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.date-cell.today .date-number {
  background: #10b981;
  color: #ffffff;
}

/* 选中日期发光效果 */
.date-cell.selected {
  background: rgba(99, 102, 241, 0.2);
  border: 1px solid rgba(99, 102, 241, 0.6);
  box-shadow: 0 0 15px rgba(99, 102, 241, 0.3);
}

.date-cell.selected .date-number {
  background: #6366f1;
  color: #ffffff;
}

/* 有任务的日期 */
.date-cell.has-tasks {
  background: rgba(255, 255, 255, 0.03);
}

/* 全部完成的日期 */
.date-cell.all-done {
  background: rgba(16, 185, 129, 0.08);
  border: 1px solid rgba(16, 185, 129, 0.2);
}

/* 日期头部 */
.date-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.date-number {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  font-size: 13px;
  font-weight: 500;
  color: #e2e8f0;
  transition: all 0.2s;
}

.date-cell.other-month .date-number {
  color: #64748b;
}

/* 任务数量徽章 */
.task-count-badge {
  font-size: 10px;
  color: #94a3b8;
  background: rgba(148, 163, 184, 0.2);
  padding: 2px 6px;
  border-radius: 10px;
  font-weight: 500;
}

/* 任务列表 */
.task-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

/* 任务项 */
.task-item {
  display: flex;
  align-items: center;
  gap: 6px;
  overflow: hidden;
}

/* 优先级指示器 */
.priority-indicator {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

/* 高优先级 - 红点 */
.priority-indicator.priority-1 {
  background: #f43f5e;
}

/* 普通优先级 - 蓝点 */
.priority-indicator.priority-2,
.priority-indicator.priority-3 {
  background: #3b82f6;
}

/* 已完成任务的指示器 */
.priority-indicator.done {
  background: #10b981;
}

/* 任务标题 */
.task-title {
  font-size: 10px;
  line-height: 1.3;
  color: #cbd5e1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  opacity: 0.9;
}

.task-item:hover .task-title {
  opacity: 1;
}

/* 已完成任务的标题 */
.task-item .priority-indicator.done ~ .task-title {
  color: #94a3b8;
  text-decoration: line-through;
}

/* 更多任务提示 */
.more-tasks {
  font-size: 10px;
  color: #64748b;
  padding-left: 12px;
  margin-top: 2px;
}

/* 已完成任务的日期单元格样式 */
.date-cell.has-done-tasks::after {
  content: '';
  position: absolute;
  bottom: 4px;
  right: 4px;
  width: 5px;
  height: 5px;
  background: #10b981;
  border-radius: 50%;
  opacity: 0.6;
}
</style>
