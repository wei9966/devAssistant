<template>
  <div class="pomodoro-report">
    <!-- 日期选择器 -->
    <div class="report-controls">
      <div class="date-range">
        <n-date-picker
          v-model:value="dateRange"
          type="daterange"
          clearable
          :default-value="defaultDateRange"
          @update:value="handleDateChange"
        />
      </div>
      <n-button type="primary" @click="loadData" :loading="loading">
        <template #icon><n-icon :component="RefreshOutline" /></template>
        刷新
      </n-button>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-cards">
      <div class="stat-card">
        <div class="stat-icon tomato">🍅</div>
        <div class="stat-content">
          <div class="stat-value">{{ totalStats.totalSessions }}</div>
          <div class="stat-label">完成番茄</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon time">⏱️</div>
        <div class="stat-content">
          <div class="stat-value">{{ formatMinutes(totalStats.totalFocusMinutes) }}</div>
          <div class="stat-label">专注时长</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon focus">🎯</div>
        <div class="stat-content">
          <div class="stat-value">{{ totalStats.avgFocusRate }}%</div>
          <div class="stat-label">平均专注率</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon task">✅</div>
        <div class="stat-content">
          <div class="stat-value">{{ completedTaskCount }}</div>
          <div class="stat-label">完成任务</div>
        </div>
      </div>
    </div>

    <!-- 每日趋势 -->
    <div class="section">
      <h3 class="section-title">每日趋势</h3>
      <div class="daily-chart">
        <div
          v-for="day in dailyStats"
          :key="day.date"
          class="day-bar"
          :title="`${day.date}: ${day.completedSessions}个番茄, ${day.totalFocusMinutes}分钟`"
        >
          <div
            class="bar-fill"
            :style="{ height: `${Math.min(100, (day.totalFocusMinutes / maxDailyMinutes) * 100)}%` }"
          ></div>
          <div class="bar-label">{{ formatDateLabel(day.date) }}</div>
          <div class="bar-count">{{ day.completedSessions }}</div>
        </div>
      </div>
    </div>

    <!-- 番茄钟记录列表 -->
    <div class="section">
      <h3 class="section-title">专注记录</h3>
      <div v-if="sessions.length === 0" class="empty-state">
        <div class="empty-icon">🍅</div>
        <div class="empty-text">该时间段内没有番茄钟记录</div>
      </div>
      <div v-else class="session-list">
        <div
          v-for="session in sessions"
          :key="session.id"
          class="session-item"
          :class="{ completed: session.status === 'completed' }"
        >
          <div class="session-header">
            <div class="session-task">
              <span v-if="session.task" class="task-title">{{ session.task.title }}</span>
              <span v-else-if="session.focusGoal" class="focus-goal">{{ session.focusGoal }}</span>
              <span v-else class="no-task">无关联任务</span>
            </div>
            <div class="session-meta">
              <span class="session-duration">{{ session.durationMinutes }}分钟</span>
              <span class="session-status" :class="session.status">
                {{ getStatusText(session.status) }}
              </span>
            </div>
          </div>
          <div class="session-body">
            <div class="session-stats">
              <span class="stat">
                <span class="stat-label">专注率</span>
                <span class="stat-value">{{ Math.round(session.focusRate) }}%</span>
              </span>
              <span class="stat">
                <span class="stat-label">实际时长</span>
                <span class="stat-value">{{ Math.round(session.actualFocusSeconds / 60) }}分钟</span>
              </span>
              <span v-if="session.distractionCount > 0" class="stat warning">
                <span class="stat-label">分心次数</span>
                <span class="stat-value">{{ session.distractionCount }}</span>
              </span>
            </div>
            <div v-if="session.feedback" class="session-feedback">
              <span class="feedback-label">反馈：</span>
              {{ session.feedback }}
            </div>
          </div>
          <div class="session-footer">
            <span class="session-time">{{ formatDateTime(session.completedAt || session.createdAt) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NDatePicker, NButton, NIcon, useMessage } from 'naive-ui'
import { RefreshOutline } from '@vicons/ionicons5'
import { getStatsRange, getTodaySessions } from '@/api/pomodoroApi'
import type { PomodoroSession, PomodoroDailyStats } from '@/types/pomodoro'
import dayjs from 'dayjs'

const message = useMessage()
const loading = ref(false)

// 日期范围（默认最近7天）
const now = Date.now()
const weekAgo = now - 7 * 24 * 60 * 60 * 1000
const defaultDateRange: [number, number] = [weekAgo, now]
const dateRange = ref<[number, number] | null>(defaultDateRange)

// 数据
const dailyStats = ref<PomodoroDailyStats[]>([])
const sessions = ref<PomodoroSession[]>([])

// 计算统计
const totalStats = computed(() => {
  const stats = dailyStats.value
  if (stats.length === 0) {
    return {
      totalSessions: 0,
      totalFocusMinutes: 0,
      avgFocusRate: 0
    }
  }
  const totalSessions = stats.reduce((sum, s) => sum + s.completedSessions, 0)
  const totalFocusMinutes = stats.reduce((sum, s) => sum + s.totalFocusMinutes, 0)
  const avgFocusRate = Math.round(
    stats.reduce((sum, s) => sum + s.avgFocusRate, 0) / stats.length
  )
  return { totalSessions, totalFocusMinutes, avgFocusRate }
})

const completedTaskCount = computed(() => {
  const taskIds = new Set<number>()
  sessions.value.forEach(s => {
    if (s.taskId && s.status === 'completed') {
      taskIds.add(s.taskId)
    }
  })
  return taskIds.size
})

const maxDailyMinutes = computed(() => {
  if (dailyStats.value.length === 0) return 60
  return Math.max(...dailyStats.value.map(s => s.totalFocusMinutes), 60)
})

// 加载数据
async function loadData() {
  if (!dateRange.value) return

  loading.value = true
  try {
    const [start, end] = dateRange.value
    const startDate = dayjs(start).format('YYYY-MM-DD')
    const endDate = dayjs(end).format('YYYY-MM-DD')

    // 获取日期范围统计
    dailyStats.value = await getStatsRange(startDate, endDate)

    // 获取今日会话（用于详细记录）
    // TODO: 需要后端添加按日期范围获取会话的API
    sessions.value = await getTodaySessions()
  } catch (error) {
    console.error('加载番茄钟数据失败:', error)
    message.error('加载数据失败')
  } finally {
    loading.value = false
  }
}

function handleDateChange(value: [number, number] | null) {
  dateRange.value = value
  if (value) {
    loadData()
  }
}

function formatMinutes(minutes: number): string {
  if (minutes < 60) return `${minutes}分钟`
  const hours = Math.floor(minutes / 60)
  const mins = minutes % 60
  return mins > 0 ? `${hours}小时${mins}分` : `${hours}小时`
}

function formatDateLabel(date: string): string {
  const d = dayjs(date)
  const today = dayjs()
  if (d.isSame(today, 'day')) return '今天'
  if (d.isSame(today.subtract(1, 'day'), 'day')) return '昨天'
  return d.format('MM/DD')
}

function formatDateTime(dateStr?: string): string {
  if (!dateStr) return ''
  return dayjs(dateStr).format('MM-DD HH:mm')
}

function getStatusText(status: string): string {
  const map: Record<string, string> = {
    completed: '已完成',
    focusing: '进行中',
    paused: '已暂停',
    cancelled: '已取消'
  }
  return map[status] || status
}

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.pomodoro-report {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.report-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.date-range {
  flex: 1;
  max-width: 320px;
}

/* 统计卡片 */
.stats-cards {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: rgba(30, 41, 59, 0.6);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 16px;
}

.stat-icon {
  font-size: 32px;
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
}

.stat-icon.tomato { background: rgba(239, 68, 68, 0.15); }
.stat-icon.time { background: rgba(99, 102, 241, 0.15); }
.stat-icon.focus { background: rgba(16, 185, 129, 0.15); }
.stat-icon.task { background: rgba(245, 158, 11, 0.15); }

.stat-content {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: #f1f5f9;
}

.stat-label {
  font-size: 12px;
  color: #94a3b8;
  margin-top: 4px;
}

/* 区块 */
.section {
  background: rgba(30, 41, 59, 0.4);
  border: 1px solid rgba(51, 65, 85, 0.4);
  border-radius: 16px;
  padding: 20px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: #e2e8f0;
  margin: 0 0 16px 0;
}

/* 每日趋势图表 */
.daily-chart {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  height: 150px;
  padding: 10px 0;
}

.day-bar {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  height: 100%;
  position: relative;
}

.bar-fill {
  width: 100%;
  max-width: 40px;
  background: linear-gradient(to top, #6366f1, #818cf8);
  border-radius: 6px 6px 0 0;
  margin-top: auto;
  transition: height 0.3s ease;
  min-height: 4px;
}

.bar-label {
  font-size: 10px;
  color: #94a3b8;
  margin-top: 8px;
}

.bar-count {
  font-size: 11px;
  font-weight: 600;
  color: #6366f1;
  position: absolute;
  top: 0;
}

/* 会话列表 */
.session-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-height: 400px;
  overflow-y: auto;
}

.session-item {
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.4);
  border-radius: 12px;
  padding: 16px;
}

.session-item.completed {
  border-left: 3px solid #10b981;
}

.session-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.task-title {
  font-size: 14px;
  font-weight: 600;
  color: #e2e8f0;
}

.focus-goal {
  font-size: 14px;
  color: #94a3b8;
  font-style: italic;
}

.no-task {
  font-size: 13px;
  color: #64748b;
}

.session-meta {
  display: flex;
  gap: 8px;
  align-items: center;
}

.session-duration {
  font-size: 12px;
  color: #94a3b8;
}

.session-status {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;
}

.session-status.completed {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
}

.session-status.cancelled {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.session-status.paused {
  background: rgba(245, 158, 11, 0.2);
  color: #f59e0b;
}

.session-body {
  margin-bottom: 12px;
}

.session-stats {
  display: flex;
  gap: 20px;
  margin-bottom: 8px;
}

.session-stats .stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.session-stats .stat-label {
  font-size: 10px;
  color: #64748b;
}

.session-stats .stat-value {
  font-size: 14px;
  font-weight: 600;
  color: #e2e8f0;
}

.session-stats .stat.warning .stat-value {
  color: #f59e0b;
}

.session-feedback {
  font-size: 13px;
  color: #94a3b8;
  background: rgba(51, 65, 85, 0.3);
  padding: 10px 12px;
  border-radius: 8px;
  line-height: 1.5;
}

.feedback-label {
  color: #64748b;
  font-weight: 500;
}

.session-footer {
  display: flex;
  justify-content: flex-end;
}

.session-time {
  font-size: 11px;
  color: #64748b;
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 40px 20px;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 12px;
  opacity: 0.5;
}

.empty-text {
  font-size: 14px;
  color: #64748b;
}

/* 滚动条 */
.session-list::-webkit-scrollbar {
  width: 6px;
}

.session-list::-webkit-scrollbar-track {
  background: transparent;
}

.session-list::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 3px;
}

/* 响应式 */
@media (max-width: 1200px) {
  .stats-cards {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .stats-cards {
    grid-template-columns: 1fr;
  }

  .report-controls {
    flex-direction: column;
    align-items: stretch;
  }

  .date-range {
    max-width: none;
  }
}
</style>
