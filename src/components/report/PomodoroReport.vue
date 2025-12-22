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

    <!-- 会话详情抽屉 -->
    <n-drawer
      v-model:show="showDetailDrawer"
      :width="480"
      placement="right"
    >
      <n-drawer-content v-if="selectedSession" :title="selectedSession.task?.title || selectedSession.focusGoal || '番茄钟详情'">
        <div class="session-detail">
          <!-- 基本信息 -->
          <div class="detail-section">
            <h4 class="detail-title">📊 基本信息</h4>
            <div class="detail-grid">
              <div class="detail-item">
                <span class="detail-label">专注目标</span>
                <span class="detail-value">{{ selectedSession.focusGoal || '无' }}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">计划时长</span>
                <span class="detail-value">{{ selectedSession.durationMinutes }}分钟</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">实际专注</span>
                <span class="detail-value">{{ Math.round(selectedSession.actualFocusSeconds / 60) }}分钟</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">专注率</span>
                <span class="detail-value highlight">{{ Math.round(selectedSession.focusRate) }}%</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">分心次数</span>
                <span class="detail-value" :class="{ warning: selectedSession.distractionCount > 0 }">{{ selectedSession.distractionCount }}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">完成时间</span>
                <span class="detail-value">{{ formatDateTime(selectedSession.completedAt || selectedSession.createdAt) }}</span>
              </div>
            </div>
          </div>

          <!-- 应用使用统计 -->
          <div v-if="parsedAppUsage.length > 0" class="detail-section">
            <h4 class="detail-title">📱 应用使用分布</h4>
            <div class="app-usage-list">
              <div
                v-for="app in parsedAppUsage"
                :key="app.appName"
                class="app-usage-item"
              >
                <div class="app-info">
                  <span class="app-name">{{ app.appName }}</span>
                  <span class="app-time">{{ app.minutes }}分钟</span>
                </div>
                <div class="app-bar">
                  <div class="app-bar-fill" :style="{ width: `${app.percentage}%` }"></div>
                </div>
                <span class="app-percentage">{{ Math.round(app.percentage) }}%</span>
              </div>
            </div>
          </div>
          <div v-else class="detail-section">
            <h4 class="detail-title">📱 应用使用分布</h4>
            <div class="empty-hint">暂无应用使用记录</div>
          </div>

          <!-- AI建议 (开始时) -->
          <div v-if="parsedAiSuggestion" class="detail-section">
            <h4 class="detail-title">🤖 AI任务建议</h4>
            <div class="ai-suggestion-content">
              <div v-if="parsedAiSuggestion.suggested_goal || parsedAiSuggestion.suggestedGoal" class="suggestion-item">
                <span class="suggestion-label">建议目标:</span>
                <span class="suggestion-value">{{ parsedAiSuggestion.suggested_goal || parsedAiSuggestion.suggestedGoal }}</span>
              </div>
              <div v-if="(parsedAiSuggestion.sub_tasks || parsedAiSuggestion.subTasks)?.length" class="suggestion-item">
                <span class="suggestion-label">子任务:</span>
                <ul class="sub-tasks-list">
                  <li v-for="(task, idx) in (parsedAiSuggestion.sub_tasks || parsedAiSuggestion.subTasks)" :key="idx">{{ task }}</li>
                </ul>
              </div>
              <div v-if="parsedAiSuggestion.tips" class="suggestion-item">
                <span class="suggestion-label">小贴士:</span>
                <span class="suggestion-value">{{ parsedAiSuggestion.tips }}</span>
              </div>
            </div>
          </div>

          <!-- AI分析 (完成后) -->
          <div v-if="selectedSession.status === 'completed'" class="detail-section">
            <h4 class="detail-title">📈 AI专注分析</h4>
            <!-- 加载中 -->
            <div v-if="analysisLoading" class="analysis-loading">
              <n-spin size="small" />
              <span>正在分析专注数据...</span>
            </div>
            <!-- 有分析结果 -->
            <div v-else-if="parsedAiAnalysis" class="ai-analysis-content">
              <div class="analysis-score">
                <div class="score-circle" :class="getScoreClass(parsedAiAnalysis.efficiency_score || parsedAiAnalysis.efficiencyScore || 0)">
                  {{ parsedAiAnalysis.efficiency_score || parsedAiAnalysis.efficiencyScore || 0 }}
                </div>
                <span class="score-label">效率评分</span>
              </div>
              <div v-if="parsedAiAnalysis.relevance_analysis || parsedAiAnalysis.relevanceAnalysis" class="analysis-item">
                <span class="analysis-label">相关性分析:</span>
                <span class="analysis-value">{{ parsedAiAnalysis.relevance_analysis || parsedAiAnalysis.relevanceAnalysis }}</span>
              </div>
              <div v-if="parsedAiAnalysis.efficiency_comment || parsedAiAnalysis.efficiencyComment" class="analysis-item">
                <span class="analysis-label">效率评价:</span>
                <span class="analysis-value">{{ parsedAiAnalysis.efficiency_comment || parsedAiAnalysis.efficiencyComment }}</span>
              </div>
              <div v-if="(parsedAiAnalysis.improvements || []).length" class="analysis-item">
                <span class="analysis-label">改进建议:</span>
                <ul class="improvements-list">
                  <li v-for="(item, idx) in parsedAiAnalysis.improvements" :key="idx">{{ item }}</li>
                </ul>
              </div>
              <div v-if="parsedAiAnalysis.next_action || parsedAiAnalysis.nextAction" class="analysis-item">
                <span class="analysis-label">下次行动:</span>
                <span class="analysis-value">{{ parsedAiAnalysis.next_action || parsedAiAnalysis.nextAction }}</span>
              </div>
            </div>
            <!-- 无分析结果 -->
            <div v-else class="empty-hint">暂无AI分析结果</div>
          </div>

          <!-- 用户反馈 -->
          <div v-if="selectedSession.feedback" class="detail-section">
            <h4 class="detail-title">💬 用户反馈</h4>
            <div class="feedback-content">{{ selectedSession.feedback }}</div>
          </div>
        </div>
      </n-drawer-content>
    </n-drawer>

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
          class="session-item clickable"
          :class="{ completed: session.status === 'completed' }"
          @click="openSessionDetail(session)"
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
import { ref, computed, onMounted, watch } from 'vue'
import { NDatePicker, NButton, NIcon, NDrawer, NDrawerContent, NSpin, useMessage } from 'naive-ui'
import { RefreshOutline } from '@vicons/ionicons5'
import { getStatsRange, getTodaySessions, aiAnalyzeSession, updateAiAnalysis } from '@/api/pomodoroApi'
import type { PomodoroSession, PomodoroDailyStats, AppUsageItem } from '@/types/pomodoro'
import dayjs from 'dayjs'

const message = useMessage()
const loading = ref(false)

// 抽屉状态
const showDetailDrawer = ref(false)
const selectedSession = ref<PomodoroSession | null>(null)
const analysisLoading = ref(false)

// 解析应用使用数据
const parsedAppUsage = computed<AppUsageItem[]>(() => {
  if (!selectedSession.value?.appUsage) return []
  try {
    const data = JSON.parse(selectedSession.value.appUsage)
    if (Array.isArray(data)) {
      return data.map((item: any) => ({
        appName: item.appName || item.app_name || '未知应用',
        count: item.count || 0,
        minutes: item.minutes || 0,
        percentage: item.percentage || 0
      }))
    }
    return []
  } catch {
    return []
  }
})

// 解析AI建议数据
const parsedAiSuggestion = computed(() => {
  if (!selectedSession.value?.aiSuggestion) return null
  try {
    return parseJsonSafe(selectedSession.value.aiSuggestion)
  } catch {
    return null
  }
})

// 解析AI分析数据
const parsedAiAnalysis = computed(() => {
  const session = selectedSession.value as any
  if (!session?.aiAnalysis) return null
  try {
    return parseJsonSafe(session.aiAnalysis)
  } catch {
    return null
  }
})

// 安全解析JSON（处理markdown包装）
function parseJsonSafe(str: string): any {
  if (!str) return null
  try {
    return JSON.parse(str)
  } catch {
    // 尝试提取markdown中的JSON
    const jsonMatch = str.match(/```(?:json)?\s*([\s\S]*?)```/)
    if (jsonMatch && jsonMatch[1]) {
      try {
        return JSON.parse(jsonMatch[1].trim())
      } catch (e) { }
    }
    // 尝试找到JSON边界
    const jsonStart = str.indexOf('{')
    const jsonEnd = str.lastIndexOf('}')
    if (jsonStart !== -1 && jsonEnd > jsonStart) {
      try {
        return JSON.parse(str.substring(jsonStart, jsonEnd + 1))
      } catch (e) { }
    }
    return null
  }
}

// 打开会话详情
async function openSessionDetail(session: PomodoroSession) {
  selectedSession.value = session
  showDetailDrawer.value = true

  // 如果是已完成的会话且没有AI分析结果，则实时请求并保存
  const sessionAny = session as any
  if (session.status === 'completed' && !sessionAny.aiAnalysis && session.id) {
    await fetchAndSaveAiAnalysis(session.id)
  }
}

// 获取并保存AI分析结果
async function fetchAndSaveAiAnalysis(sessionId: number) {
  if (analysisLoading.value) return

  analysisLoading.value = true
  try {
    const result = await aiAnalyzeSession(sessionId)
    console.log('[报表AI分析] 获取成功:', sessionId)

    // 保存到数据库
    await updateAiAnalysis(sessionId, result)
    console.log('[报表AI分析] 已保存到数据库')

    // 更新当前选中的会话数据
    if (selectedSession.value && selectedSession.value.id === sessionId) {
      (selectedSession.value as any).aiAnalysis = result
    }

    // 同时更新sessions列表中的数据
    const idx = sessions.value.findIndex(s => s.id === sessionId)
    if (idx !== -1) {
      (sessions.value[idx] as any).aiAnalysis = result
    }
  } catch (error: any) {
    console.error('[报表AI分析] 获取失败:', error)
    const errorMsg = error?.toString() || ''
    if (!errorMsg.includes('AI 服务未初始化') && !errorMsg.includes('AI 服务未配置')) {
      message.warning('AI分析获取失败')
    }
  } finally {
    analysisLoading.value = false
  }
}

// 根据分数返回样式类
function getScoreClass(score: number): string {
  if (score >= 80) return 'score-high'
  if (score >= 60) return 'score-medium'
  return 'score-low'
}

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
  background: var(--card-bg);
  border: 1px solid var(--card-border);
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
  color: var(--text-primary);
}

.stat-label {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
}

/* 区块 */
.section {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 16px;
  padding: 20px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
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
  color: var(--text-muted);
  margin-top: 8px;
}

.bar-count {
  font-size: 11px;
  font-weight: 600;
  color: var(--accent-primary);
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
  background: var(--bg-overlay);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  padding: 16px;
}

.session-item.clickable {
  cursor: pointer;
  transition: all 0.2s ease;
}

.session-item.clickable:hover {
  background: var(--bg-hover);
  border-color: var(--border-active);
  transform: translateY(-1px);
}

.session-item.completed {
  border-left: 3px solid var(--success);
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
  color: var(--text-primary);
}

.focus-goal {
  font-size: 14px;
  color: var(--text-muted);
  font-style: italic;
}

.no-task {
  font-size: 13px;
  color: var(--text-dim);
}

.session-meta {
  display: flex;
  gap: 8px;
  align-items: center;
}

.session-duration {
  font-size: 12px;
  color: var(--text-muted);
}

.session-status {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;
}

.session-status.completed {
  background: rgba(16, 185, 129, 0.2);
  color: var(--success);
}

.session-status.cancelled {
  background: rgba(239, 68, 68, 0.2);
  color: var(--error);
}

.session-status.paused {
  background: rgba(245, 158, 11, 0.2);
  color: var(--warning);
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
  color: var(--text-dim);
}

.session-stats .stat-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.session-stats .stat.warning .stat-value {
  color: var(--warning);
}

.session-feedback {
  font-size: 13px;
  color: var(--text-muted);
  background: var(--bg-overlay);
  padding: 10px 12px;
  border-radius: 8px;
  line-height: 1.5;
}

.feedback-label {
  color: var(--text-dim);
  font-weight: 500;
}

.session-footer {
  display: flex;
  justify-content: flex-end;
}

.session-time {
  font-size: 11px;
  color: var(--text-dim);
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
  color: var(--text-dim);
}

/* 滚动条 */
.session-list::-webkit-scrollbar {
  width: 6px;
}

.session-list::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
}

.session-list::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 3px;
}

/* 会话详情抽屉样式 */
.session-detail {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.detail-section {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  padding: 16px;
}

.detail-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 12px 0;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-label {
  font-size: 11px;
  color: var(--text-dim);
}

.detail-value {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.detail-value.highlight {
  color: var(--success);
}

.detail-value.warning {
  color: var(--warning);
}

/* 应用使用统计 */
.app-usage-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.app-usage-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.app-info {
  min-width: 100px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.app-name {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 500;
}

.app-time {
  font-size: 11px;
  color: var(--text-dim);
}

.app-bar {
  flex: 1;
  height: 8px;
  background: var(--bg-overlay);
  border-radius: 4px;
  overflow: hidden;
}

.app-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-primary), var(--accent-secondary));
  border-radius: 4px;
  transition: width 0.3s ease;
}

.app-percentage {
  font-size: 12px;
  color: var(--text-muted);
  min-width: 40px;
  text-align: right;
}

.empty-hint {
  font-size: 13px;
  color: var(--text-dim);
  text-align: center;
  padding: 16px;
}

.analysis-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 24px;
  color: var(--text-muted);
  font-size: 13px;
}

/* AI建议样式 */
.ai-suggestion-content,
.ai-analysis-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.suggestion-item,
.analysis-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.suggestion-label,
.analysis-label {
  font-size: 11px;
  color: var(--text-dim);
  font-weight: 500;
}

.suggestion-value,
.analysis-value {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.sub-tasks-list,
.improvements-list {
  margin: 4px 0 0 16px;
  padding: 0;
  list-style: disc;
}

.sub-tasks-list li,
.improvements-list li {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 4px;
  line-height: 1.4;
}

/* AI分析评分 */
.analysis-score {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.score-circle {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: 700;
  color: white;
}

.score-circle.score-high {
  background: linear-gradient(135deg, #10b981, #34d399);
}

.score-circle.score-medium {
  background: linear-gradient(135deg, #f59e0b, #fbbf24);
}

.score-circle.score-low {
  background: linear-gradient(135deg, #ef4444, #f87171);
}

.score-label {
  font-size: 13px;
  color: var(--text-muted);
}

/* 用户反馈 */
.feedback-content {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
  background: var(--bg-overlay);
  padding: 12px;
  border-radius: 8px;
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
