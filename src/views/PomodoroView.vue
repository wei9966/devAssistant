<template>
  <div class="pomodoro-view">
    <!-- PREP 准备阶段 -->
    <div v-if="currentPhase === 'prep'" class="scene-container prep-scene">
      <div class="prep-content">
        <!-- Header -->
        <div class="scene-header">
          <div class="status-indicator">
            <div class="status-dot"></div>
            <span class="status-label">AI FOCUS ENGINE</span>
          </div>
          <h1 class="scene-title">准备开始深度工作吗？</h1>
        </div>

        <!-- Task Card -->
        <div v-if="selectedTask" class="task-card">
          <div class="task-card-header">
            <div class="task-info">
              <div class="task-checkbox">
                <n-icon size="14"><CheckmarkCircleOutline /></n-icon>
              </div>
              <h4 class="task-title">{{ selectedTask.title }}</h4>
            </div>
            <n-tag :bordered="false" size="small" type="info" class="task-category">
              {{ selectedTask.category }}
            </n-tag>
          </div>

          <!-- AI Suggestions -->
          <div class="ai-suggestions">
            <div class="ai-header">
              <n-icon size="16" color="#60a5fa"><SparklesOutline /></n-icon>
              <span class="ai-label">AI 任务执行建议</span>
            </div>
            <div class="suggestion-list">
              <div class="suggestion-item">
                <div class="suggestion-dot"></div>
                <span>基于任务分析，建议使用番茄工作法保持专注</span>
              </div>
              <div class="suggestion-item">
                <div class="suggestion-dot indigo"></div>
                <span>目标设定：{{ focusGoal || '完成核心功能开发' }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Settings Grid -->
        <div class="settings-grid">
          <!-- Whitelist Apps -->
          <div class="setting-card">
            <span class="setting-label">白名单应用</span>
            <div class="app-icons">
              <div
                v-for="app in selectedApps"
                :key="app.id"
                class="app-icon"
                :title="app.name"
              >
                {{ getAppAbbr(app.name) }}
              </div>
              <div class="app-icon add-icon" @click="showAppSelector = true">+</div>
            </div>
          </div>

          <!-- Duration -->
          <div class="setting-card">
            <span class="setting-label">专注时长</span>
            <n-select
              v-model:value="duration"
              :options="durationOptions"
              size="small"
              style="width: 100%"
            />
          </div>
        </div>

        <!-- Start Button -->
        <n-button
          type="primary"
          size="large"
          block
          class="start-button"
          :loading="loading"
          @click="handleStartPrep"
        >
          <template #icon>
            <n-icon><PlayOutline /></n-icon>
          </template>
          进入深度模式
        </n-button>
      </div>
    </div>

    <!-- FOCUSING 专注中 -->
    <div v-else-if="currentPhase === 'focusing'" class="scene-container focusing-scene">
      <div class="focusing-content">
        <!-- Timer Circle -->
        <div class="timer-container">
          <div class="timer-circle">
            <svg class="progress-ring" width="320" height="320">
              <circle
                class="progress-ring-bg"
                cx="160"
                cy="160"
                r="144"
                fill="none"
                stroke="#18181b"
                stroke-width="12"
              />
              <circle
                class="progress-ring-bar"
                cx="160"
                cy="160"
                r="144"
                fill="none"
                stroke="#3b82f6"
                stroke-width="12"
                stroke-linecap="round"
                :stroke-dasharray="circumference"
                :stroke-dashoffset="progressOffset"
                transform="rotate(-90 160 160)"
              />
            </svg>
            <div class="timer-display">
              <span class="time-text">{{ formattedTime }}</span>
              <div v-if="currentSession?.task" class="active-app-badge">
                <n-icon size="12"><FlashOutline /></n-icon>
                <span>{{ currentAppName || '专注中' }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Task Info -->
        <div class="task-info-section">
          <h2 class="current-task-title">{{ currentSession?.task?.title || '深度专注中' }}</h2>
          <div class="focus-status">
            <div class="status-dots">
              <div class="dot active"></div>
              <div class="dot"></div>
              <div class="dot"></div>
            </div>
            <p class="status-text">AI 正在守护你的注意力通道</p>
          </div>
        </div>

        <!-- Control Buttons -->
        <div class="control-buttons">
          <n-button
            circle
            size="large"
            class="control-btn"
            @click="handlePause"
          >
            <template #icon>
              <n-icon size="20"><PauseOutline v-if="!isPaused" /><PlayOutline v-else /></n-icon>
            </template>
          </n-button>
          <n-button
            circle
            size="large"
            class="control-btn stop-btn"
            @click="handleStop"
          >
            <template #icon>
              <n-icon size="20"><StopOutline /></n-icon>
            </template>
          </n-button>
        </div>
      </div>
    </div>

    <!-- DISTRACTED 分心提醒 -->
    <div v-else-if="currentPhase === 'distracted'" class="scene-container distracted-scene">
      <div class="distracted-overlay">
        <div class="distracted-modal">
          <div class="warning-icon">
            <n-icon size="48"><AlertCircleOutline /></n-icon>
          </div>
          <h2 class="distracted-title">注意力偏离！</h2>
          <p class="distracted-text">
            检测到非工作应用 <span class="highlight-app">{{ distractedApp }}</span> 被打开。
            <br />
            <span class="sub-text">这是工作中的必要查找，还是需要休息？</span>
          </p>
          <div class="distracted-actions">
            <n-button
              type="primary"
              size="large"
              block
              @click="handleReturnFocus"
            >
              立即重返工作
            </n-button>
            <n-button
              size="large"
              block
              class="secondary-btn"
              @click="handleStopFromDistraction"
            >
              停止本次计时
            </n-button>
          </div>
        </div>
      </div>
    </div>

    <!-- REPORT 结束报告 -->
    <div v-else-if="currentPhase === 'report'" class="scene-container report-scene">
      <div class="report-content">
        <div class="report-header">
          <div class="report-icon-box">
            <n-icon size="28"><BulbOutline /></n-icon>
          </div>
          <div class="report-title-section">
            <h2 class="report-title">整理刚才的产出</h2>
            <p class="report-subtitle">
              本次专注: {{ sessionDuration }} 分钟 • 专注率 {{ sessionFocusRate }}%
            </p>
          </div>
          <div class="session-badge">SESSION #{{ todaySessionCount }}</div>
        </div>

        <div class="feedback-section">
          <label class="feedback-label">AI 进度同步反馈</label>
          <n-input
            v-model:value="feedback"
            type="textarea"
            placeholder="完成了核心功能开发，修复了两个关键bug..."
            :rows="6"
            class="feedback-input"
          />
          <div class="feedback-actions">
            <n-button circle class="voice-btn">
              <template #icon>
                <n-icon size="20"><MicOutline /></n-icon>
              </template>
            </n-button>
          </div>
        </div>

        <n-button
          type="primary"
          size="large"
          block
          class="sync-button"
          :loading="loading"
          @click="handleComplete"
        >
          <template #icon>
            <n-icon><SparklesOutline /></n-icon>
          </template>
          AI 同步并生成周报
        </n-button>
      </div>
    </div>

    <!-- STATS 统计看板 -->
    <div v-else-if="currentPhase === 'stats'" class="scene-container stats-scene">
      <div class="stats-content">
        <!-- Header -->
        <div class="stats-header">
          <div class="stats-title-section">
            <n-icon size="28" color="#3b82f6"><BarChartOutline /></n-icon>
            <h2 class="stats-title">深度效能看板</h2>
          </div>
          <div class="date-badge">{{ currentDate }}</div>
        </div>

        <!-- Stats Grid -->
        <div class="stats-grid">
          <div class="stat-card">
            <div class="stat-label">今日番茄</div>
            <div class="stat-value blue">
              {{ todayStats?.completedSessions || 0 }}<span class="stat-unit">个</span>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-label">专注效率</div>
            <div class="stat-value green">
              {{ Math.round(todayStats?.avgFocusRate || 0) }}<span class="stat-unit">%</span>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-label">心流时长</div>
            <div class="stat-value white">
              {{ todayStats?.totalFocusMinutes || 0 }}<span class="stat-unit">min</span>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-label">阻断分心</div>
            <div class="stat-value red">
              {{ todayStats?.totalDistractions || 0 }}<span class="stat-unit">次</span>
            </div>
          </div>
        </div>

        <!-- Charts Section -->
        <div class="charts-section">
          <!-- App Usage -->
          <div class="chart-card">
            <h3 class="chart-title">核心产出工具分布</h3>
            <div class="app-usage-list">
              <div
                v-for="app in appUsageList"
                :key="app.appName"
                class="app-usage-item"
              >
                <div class="app-usage-header">
                  <span class="app-name">{{ app.appName }}</span>
                  <span class="app-time">{{ app.minutes }} MIN</span>
                </div>
                <div class="app-progress-bar">
                  <div
                    class="app-progress-fill"
                    :style="{ width: `${app.percentage}%` }"
                  ></div>
                </div>
              </div>
            </div>
          </div>

          <!-- AI Insight -->
          <div class="chart-card ai-insight-card">
            <div class="ai-insight-bg">
              <n-icon size="120"><BulbOutline /></n-icon>
            </div>
            <div class="ai-insight-content">
              <div class="ai-insight-header">
                <n-icon size="20" color="#3b82f6"><SparklesOutline /></n-icon>
                <span class="ai-insight-label">AI COACH DAILY</span>
              </div>
              <p class="ai-insight-text">
                {{ todayStats?.aiInsight || '继续保持专注，你正在稳步提升工作效率。建议在下午时段安排高难度任务，这是你的最佳状态时间。' }}
              </p>
              <n-button text class="view-more-btn">
                查看完整周报
                <template #icon>
                  <n-icon><ChevronForwardOutline /></n-icon>
                </template>
              </n-button>
            </div>
          </div>
        </div>

        <!-- Back to Prep -->
        <n-button text class="exit-btn" @click="handleBackToPrep">
          EXIT FOCUS MODE
        </n-button>
      </div>
    </div>

    <!-- App Selector Modal -->
    <n-modal
      v-model:show="showAppSelector"
      preset="card"
      title="选择白名单应用"
      style="width: 500px"
    >
      <n-space vertical>
        <n-checkbox-group v-model:value="selectedAppIds">
          <n-space vertical>
            <n-checkbox
              v-for="app in focusApps"
              :key="app.id"
              :value="app.id"
              :label="app.name"
            />
          </n-space>
        </n-checkbox-group>
        <n-divider />
        <n-input
          v-model:value="newAppName"
          placeholder="添加新应用名称"
          @keyup.enter="handleAddApp"
        />
        <n-button @click="handleAddApp" :disabled="!newAppName.trim()">
          添加应用
        </n-button>
      </n-space>
    </n-modal>

    <!-- Resume Modal - AI 恢复建议弹窗 -->
    <n-modal
      v-model:show="showResumeModal"
      :mask-closable="false"
      style="max-width: 520px"
    >
      <div class="resume-modal">
        <div class="resume-header">
          <div class="resume-icon">
            <n-icon size="32"><RefreshOutline /></n-icon>
          </div>
          <h2 class="resume-title">准备恢复专注</h2>
          <p class="resume-subtitle">
            已暂停 {{ interruptionDuration }} 分钟
          </p>
        </div>

        <!-- Loading State -->
        <div v-if="pomodoroStore.resumeLoading" class="resume-loading">
          <n-spin size="large" />
          <p>AI 正在分析您的中断期间活动...</p>
        </div>

        <!-- Quick Resume Tip -->
        <div v-else-if="pomodoroStore.quickResumeTip" class="resume-content">
          <div class="ai-tip-card">
            <div class="ai-tip-header">
              <n-icon size="20" color="#3b82f6"><SparklesOutline /></n-icon>
              <span>AI 快速恢复提示</span>
            </div>
            <p class="ai-tip-text">{{ pomodoroStore.quickResumeTip }}</p>
          </div>
        </div>

        <!-- Full Analysis Result -->
        <div v-else-if="pomodoroStore.resumeSuggestion" class="resume-content">
          <!-- Context Reminder -->
          <div class="resume-section">
            <div class="section-label">
              <n-icon size="16"><TimeOutline /></n-icon>
              <span>上次进度</span>
            </div>
            <p class="section-content">{{ pomodoroStore.resumeSuggestion.contextReminder }}</p>
          </div>

          <!-- Interruption Summary -->
          <div v-if="pomodoroStore.interruptionAnalysis" class="resume-section">
            <div class="section-label">
              <n-icon size="16"><AnalyticsOutline /></n-icon>
              <span>中断分析</span>
            </div>
            <p class="section-content">{{ pomodoroStore.interruptionAnalysis.summary }}</p>
            <div class="interruption-tags">
              <n-tag size="small" :type="getRelevanceType(pomodoroStore.interruptionAnalysis.relevance)">
                {{ pomodoroStore.interruptionAnalysis.relevance }}
              </n-tag>
              <n-tag size="small" v-if="pomodoroStore.interruptionAnalysis.contextSwitch" type="warning">
                上下文切换
              </n-tag>
            </div>
          </div>

          <!-- Next Action -->
          <div class="resume-section highlight">
            <div class="section-label">
              <n-icon size="16"><FlashOutline /></n-icon>
              <span>建议下一步</span>
            </div>
            <p class="section-content bold">{{ pomodoroStore.resumeSuggestion.nextAction }}</p>
          </div>

          <!-- Tips -->
          <div v-if="pomodoroStore.resumeSuggestion.tips?.length" class="resume-tips">
            <div v-for="(tip, index) in pomodoroStore.resumeSuggestion.tips" :key="index" class="tip-item">
              <n-icon size="12" color="#10b981"><CheckmarkOutline /></n-icon>
              <span>{{ tip }}</span>
            </div>
          </div>

          <!-- Adjusted Goal -->
          <div v-if="pomodoroStore.resumeSuggestion.adjustedGoal" class="adjusted-goal">
            <n-icon size="14"><BulbOutline /></n-icon>
            <span>建议调整目标: {{ pomodoroStore.resumeSuggestion.adjustedGoal }}</span>
          </div>
        </div>

        <!-- Initial State - Choose Resume Type -->
        <div v-else class="resume-options">
          <p class="options-hint">选择恢复方式:</p>
          <n-button
            type="primary"
            block
            size="large"
            class="resume-option-btn"
            @click="handleQuickResume"
            :loading="quickResumeLoading"
          >
            <template #icon>
              <n-icon><FlashOutline /></n-icon>
            </template>
            快速恢复
          </n-button>
          <n-button
            block
            size="large"
            class="resume-option-btn secondary"
            @click="handleFullAnalysis"
            :loading="fullAnalysisLoading"
          >
            <template #icon>
              <n-icon><AnalyticsOutline /></n-icon>
            </template>
            AI 深度分析
          </n-button>
        </div>

        <!-- Action Buttons -->
        <div class="resume-actions">
          <n-button
            type="primary"
            size="large"
            block
            @click="handleConfirmResume"
            :disabled="pomodoroStore.resumeLoading"
          >
            继续专注
          </n-button>
          <n-button
            size="large"
            block
            class="cancel-btn"
            @click="handleCancelResume"
          >
            取消
          </n-button>
        </div>
      </div>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { useMessage } from 'naive-ui'
import { usePomodoroStore } from '@/stores/pomodoroStore'
import { useTaskStore } from '@/stores/taskStore'
import type { PomodoroPhase } from '@/types/pomodoro'
import {
  PlayOutline,
  PauseOutline,
  StopOutline,
  CheckmarkCircleOutline,
  SparklesOutline,
  FlashOutline,
  AlertCircleOutline,
  BulbOutline,
  MicOutline,
  BarChartOutline,
  ChevronForwardOutline,
  RefreshOutline,
  TimeOutline,
  AnalyticsOutline,
  CheckmarkOutline
} from '@vicons/ionicons5'

const route = useRoute()
const message = useMessage()
const pomodoroStore = usePomodoroStore()
const taskStore = useTaskStore()

// Phase Control
const currentPhase = ref<PomodoroPhase>('prep')

// PREP phase data
const selectedTask = computed(() => {
  const taskId = route.query.taskId
  if (taskId) {
    return taskStore.tasks.find(t => t.id === Number(taskId))
  }
  return taskStore.activeTasks[0] || null
})

const duration = ref(25)
const durationOptions = [
  { label: '15分钟', value: 15 },
  { label: '25分钟', value: 25 },
  { label: '45分钟', value: 45 },
  { label: '60分钟', value: 60 }
]

const focusGoal = ref('')
const selectedAppIds = ref<number[]>([])
const showAppSelector = ref(false)
const newAppName = ref('')

// FOCUSING phase data
const currentAppName = ref('VS Code')
const distractedApp = ref('Steam')

// REPORT phase data
const feedback = ref('')

// Resume Modal data
const showResumeModal = ref(false)
const quickResumeLoading = ref(false)
const fullAnalysisLoading = ref(false)

// Computed
const loading = computed(() => pomodoroStore.loading)
const currentSession = computed(() => pomodoroStore.currentSession)
const isPaused = computed(() => currentSession.value?.status === 'paused')
const todayStats = computed(() => pomodoroStore.todayStats)
const focusApps = computed(() => pomodoroStore.focusApps)

const selectedApps = computed(() => {
  return focusApps.value.filter(app => selectedAppIds.value.includes(app.id!))
})

// Timer
const remainingSeconds = computed(() => pomodoroStore.remainingTime)

const formattedTime = computed(() => {
  const minutes = Math.floor(remainingSeconds.value / 60)
  const seconds = remainingSeconds.value % 60
  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
})

// Circle progress
const circumference = 2 * Math.PI * 144
const progressOffset = computed(() => {
  const progress = pomodoroStore.focusProgress
  return circumference - (progress / 100) * circumference
})

// Stats
const sessionDuration = computed(() => {
  return currentSession.value?.durationMinutes || 25
})

const sessionFocusRate = computed(() => {
  return Math.round(currentSession.value?.focusRate || 0)
})

const todaySessionCount = computed(() => {
  return pomodoroStore.todaySessions.length
})

const currentDate = computed(() => {
  const today = new Date()
  return today.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
})

const appUsageList = computed(() => {
  return todayStats.value?.appUsage || []
})

// 中断时长
const interruptionDuration = computed(() => {
  return pomodoroStore.getInterruptionDuration()
})

// Methods
function getAppAbbr(name: string): string {
  const words = name.split(' ')
  if (words.length === 1) {
    return name.substring(0, 3).toUpperCase()
  }
  return words.map(w => w[0]).join('').toUpperCase().substring(0, 3)
}

async function handleStartPrep() {
  try {
    const session = await pomodoroStore.createSession({
      taskId: selectedTask.value?.id,
      durationMinutes: duration.value,
      focusGoal: focusGoal.value,
      focusApps: selectedApps.value.map(a => a.name)
    })

    if (session.id) {
      await pomodoroStore.startSession(session.id)
      currentPhase.value = 'focusing'
      message.success('开始专注！')
    }
  } catch (error) {
    message.error('启动失败，请重试')
    console.error(error)
  }
}

async function handlePause() {
  if (!currentSession.value?.id) return

  try {
    if (isPaused.value) {
      // 恢复时显示恢复弹窗
      showResumeModal.value = true
    } else {
      await pomodoroStore.pauseSession(currentSession.value.id)
      message.info('已暂停')
    }
  } catch (error) {
    message.error('操作失败')
    console.error(error)
  }
}

// 快速恢复 - 获取简短提示
async function handleQuickResume() {
  quickResumeLoading.value = true
  try {
    await pomodoroStore.fetchQuickResumeTip()
  } catch (error) {
    console.error('获取快速恢复提示失败:', error)
    message.error('获取恢复提示失败')
  } finally {
    quickResumeLoading.value = false
  }
}

// 完整分析 - 获取详细的中断分析和恢复建议
async function handleFullAnalysis() {
  fullAnalysisLoading.value = true
  try {
    // 这里可以传入实际的活动摘要，目前使用简单描述
    const activitySummaries = '用户在中断期间进行了其他活动'
    await pomodoroStore.fetchResumeSuggestions(activitySummaries)
  } catch (error) {
    console.error('获取恢复建议失败:', error)
    message.error('获取恢复建议失败')
  } finally {
    fullAnalysisLoading.value = false
  }
}

// 确认恢复 - 关闭弹窗并恢复会话
async function handleConfirmResume() {
  if (!currentSession.value?.id) return

  try {
    await pomodoroStore.resumeSession(currentSession.value.id)
    showResumeModal.value = false
    message.success('继续专注！')
  } catch (error) {
    message.error('恢复失败')
    console.error(error)
  }
}

// 取消恢复 - 关闭弹窗，保持暂停状态
function handleCancelResume() {
  showResumeModal.value = false
  pomodoroStore.clearResumeState()
}

// 获取相关性类型用于标签颜色
function getRelevanceType(relevance: string): 'success' | 'warning' | 'error' | 'default' {
  switch (relevance) {
    case '直接相关':
      return 'success'
    case '间接相关':
      return 'warning'
    case '无关':
      return 'error'
    default:
      return 'default'
  }
}

async function handleStop() {
  if (!currentSession.value?.id) return
  currentPhase.value = 'report'
}

function handleReturnFocus() {
  currentPhase.value = 'focusing'
}

async function handleStopFromDistraction() {
  if (!currentSession.value?.id) return
  currentPhase.value = 'report'
}

async function handleComplete() {
  if (!currentSession.value?.id) return

  try {
    await pomodoroStore.completeSession(currentSession.value.id, {
      feedback: feedback.value,
      progressUpdate: feedback.value
    })
    currentPhase.value = 'stats'
    message.success('专注完成！')
  } catch (error) {
    message.error('提交失败')
    console.error(error)
  }
}

function handleBackToPrep() {
  currentPhase.value = 'prep'
  feedback.value = ''
}

async function handleAddApp() {
  if (!newAppName.value.trim()) return

  try {
    await pomodoroStore.addFocusApp(newAppName.value.trim())
    message.success('应用已添加')
    newAppName.value = ''
  } catch (error) {
    message.error('添加失败')
    console.error(error)
  }
}

// Lifecycle
onMounted(async () => {
  await taskStore.loadTasks()
  await pomodoroStore.loadFocusApps()
  await pomodoroStore.loadActiveSession()
  await pomodoroStore.loadTodayStats()
  await pomodoroStore.loadTodaySessions()

  // 如果有活跃会话，切换到对应阶段
  if (currentSession.value) {
    if (currentSession.value.status === 'focusing') {
      currentPhase.value = 'focusing'
    }
  }

  // 预选默认应用
  selectedAppIds.value = focusApps.value.filter(a => a.isDefault).map(a => a.id!)
})

onUnmounted(() => {
  // Cleanup if needed
})
</script>

<style scoped>
.pomodoro-view {
  width: 100%;
  height: 100vh;
  background: #09090b;
  background-image: radial-gradient(circle at top right, rgba(59, 130, 246, 0.05), transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 48px;
  overflow-y: auto;
}

.scene-container {
  width: 100%;
  max-width: 640px;
  animation: fadeIn 0.3s ease-in-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

/* ===== PREP SCENE ===== */
.prep-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.scene-header {
  margin-bottom: 16px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #3b82f6;
  box-shadow: 0 0 8px rgba(59, 130, 246, 0.5);
}

.status-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: #71717a;
}

.scene-title {
  font-size: 32px;
  font-weight: 700;
  color: #ffffff;
  margin: 0;
  letter-spacing: -0.02em;
}

.task-card {
  background: #18181b;
  border: 1px solid #27272a;
  border-radius: 24px;
  overflow: hidden;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
}

.task-card-header {
  padding: 20px;
  border-bottom: 1px solid #27272a;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.task-info {
  display: flex;
  align-items: center;
  gap: 16px;
  flex: 1;
}

.task-checkbox {
  width: 24px;
  height: 24px;
  border: 2px solid #3f3f46;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #27272a;
}

.task-title {
  font-size: 16px;
  font-weight: 600;
  color: #e4e4e7;
  margin: 0;
}

.task-category {
  font-size: 10px;
  font-weight: 700;
}

.ai-suggestions {
  padding: 24px;
  background: rgba(9, 9, 11, 0.3);
}

.ai-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}

.ai-label {
  font-size: 14px;
  font-weight: 700;
  color: #d4d4d8;
}

.suggestion-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.suggestion-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px;
  background: #121214;
  border: 1px solid #27272a;
  border-radius: 16px;
  font-size: 14px;
  color: #a1a1aa;
  line-height: 1.6;
}

.suggestion-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #3b82f6;
  margin-top: 6px;
  flex-shrink: 0;
}

.suggestion-dot.indigo {
  background: #6366f1;
}

.settings-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.setting-card {
  background: #18181b;
  border: 1px solid #27272a;
  border-radius: 24px;
  padding: 16px;
}

.setting-label {
  display: block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: #71717a;
  margin-bottom: 12px;
}

.app-icons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.app-icon {
  width: 32px;
  height: 32px;
  background: #27272a;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 700;
  color: #d4d4d8;
  cursor: default;
}

.app-icon.add-icon {
  border: 1px dashed #3f3f46;
  background: transparent;
  color: #71717a;
  cursor: pointer;
  transition: all 0.2s;
}

.app-icon.add-icon:hover {
  color: #a1a1aa;
  border-color: #52525b;
}

.start-button {
  height: 56px;
  border-radius: 24px;
  font-size: 16px;
  font-weight: 700;
  background: #3b82f6;
  border: none;
  box-shadow: 0 8px 24px rgba(59, 130, 246, 0.3);
  transition: all 0.2s;
}

.start-button:hover {
  background: #2563eb;
  box-shadow: 0 12px 32px rgba(59, 130, 246, 0.4);
}

.start-button:active {
  transform: scale(0.98);
}

/* ===== FOCUSING SCENE ===== */
.focusing-scene {
  max-width: 480px;
}

.focusing-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 48px;
}

.timer-container {
  position: relative;
  margin-bottom: 32px;
}

.timer-circle {
  position: relative;
  width: 320px;
  height: 320px;
}

.progress-ring {
  transform: rotate(-90deg);
}

.progress-ring-bar {
  transition: stroke-dashoffset 1s linear;
  filter: drop-shadow(0 0 8px rgba(59, 130, 246, 0.5));
}

.timer-display {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  z-index: 10;
}

.time-text {
  font-size: 80px;
  font-weight: 700;
  font-family: 'SF Mono', 'Consolas', monospace;
  color: #ffffff;
  display: block;
  line-height: 1;
  letter-spacing: -0.05em;
  text-shadow: 0 4px 24px rgba(0, 0, 0, 0.5);
}

.active-app-badge {
  margin-top: 16px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: rgba(39, 39, 42, 0.5);
  border: 1px solid #3f3f46;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.05em;
  color: #a1a1aa;
  backdrop-filter: blur(8px);
}

.task-info-section {
  text-align: center;
  margin-bottom: 16px;
}

.current-task-title {
  font-size: 22px;
  font-weight: 700;
  color: #ffffff;
  margin: 0 0 12px 0;
  letter-spacing: -0.02em;
}

.focus-status {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.status-dots {
  display: flex;
  gap: 4px;
}

.dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: #27272a;
}

.dot.active {
  background: #3b82f6;
}

.status-text {
  font-size: 11px;
  font-style: italic;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: #71717a;
  margin: 0;
}

.control-buttons {
  display: flex;
  gap: 32px;
}

.control-btn {
  width: 56px;
  height: 56px;
  background: #18181b;
  border: 1px solid #27272a;
  color: #71717a;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  transition: all 0.2s;
}

.control-btn:hover {
  color: #ffffff;
  border-color: #3f3f46;
}

.control-btn.stop-btn:hover {
  color: #ef4444;
  border-color: rgba(239, 68, 68, 0.3);
}

/* ===== DISTRACTED SCENE ===== */
.distracted-scene {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(24px);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.distracted-overlay {
  width: 100%;
  max-width: 480px;
}

.distracted-modal {
  background: #0c0c0e;
  border: 1px solid #27272a;
  border-radius: 40px;
  padding: 48px;
  text-align: center;
  box-shadow: 0 0 100px rgba(0, 0, 0, 0.8);
  animation: zoomIn 0.3s ease-out;
}

@keyframes zoomIn {
  from {
    opacity: 0;
    transform: scale(0.9);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.warning-icon {
  width: 96px;
  height: 96px;
  margin: 0 auto 32px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #ef4444;
  transform: rotate(12deg);
}

.distracted-title {
  font-size: 36px;
  font-weight: 900;
  color: #ffffff;
  margin: 0 0 24px 0;
}

.distracted-text {
  font-size: 16px;
  line-height: 1.8;
  color: #a1a1aa;
  margin: 0 0 48px 0;
}

.highlight-app {
  color: #ffffff;
  font-weight: 700;
  text-decoration: underline;
  text-decoration-color: #ef4444;
  text-decoration-thickness: 2px;
}

.sub-text {
  font-size: 13px;
  opacity: 0.5;
  display: block;
  margin-top: 8px;
}

.distracted-actions {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.secondary-btn {
  background: #18181b;
  border: 1px solid #27272a;
  color: #71717a;
}

.secondary-btn:hover {
  color: #d4d4d8;
}

/* ===== REPORT SCENE ===== */
.report-scene {
  max-width: 720px;
}

.report-content {
  background: #18181b;
  border: 1px solid #27272a;
  border-radius: 32px;
  padding: 48px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
}

.report-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 40px;
}

.report-icon-box {
  width: 56px;
  height: 56px;
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.2);
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #3b82f6;
}

.report-title-section {
  flex: 1;
  margin: 0 24px;
}

.report-title {
  font-size: 24px;
  font-weight: 700;
  color: #ffffff;
  margin: 0 0 4px 0;
}

.report-subtitle {
  font-size: 13px;
  color: #71717a;
  margin: 0;
}

.session-badge {
  padding: 8px 16px;
  background: #27272a;
  border: 1px solid #3f3f46;
  border-radius: 20px;
  font-size: 10px;
  font-weight: 900;
  color: #a1a1aa;
}

.feedback-section {
  margin-bottom: 32px;
  position: relative;
}

.feedback-label {
  display: block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: #71717a;
  margin-bottom: 16px;
}

.feedback-input {
  background: #0c0c0e !important;
  border: 1px solid #27272a !important;
  border-radius: 24px !important;
  font-size: 16px !important;
  line-height: 1.8 !important;
  color: #d4d4d8 !important;
}

.feedback-input:focus {
  border-color: #3b82f6 !important;
}

.feedback-actions {
  position: absolute;
  bottom: 24px;
  right: 24px;
  display: flex;
  gap: 8px;
}

.voice-btn {
  width: 48px;
  height: 48px;
  background: #27272a;
  border: 1px solid #3f3f46;
  color: #a1a1aa;
  transition: all 0.2s;
}

.voice-btn:hover {
  color: #ffffff;
  border-color: #52525b;
}

.sync-button {
  height: 56px;
  border-radius: 24px;
  font-size: 16px;
  font-weight: 900;
  background: #ffffff;
  color: #000000;
  border: none;
  box-shadow: 0 8px 32px rgba(255, 255, 255, 0.2);
  transition: all 0.2s;
}

.sync-button:hover {
  background: #e4e4e7;
}

/* ===== STATS SCENE ===== */
.stats-scene {
  max-width: 960px;
}

.stats-content {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.stats-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.stats-title-section {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stats-title {
  font-size: 28px;
  font-weight: 700;
  color: #ffffff;
  margin: 0;
  letter-spacing: -0.02em;
}

.date-badge {
  padding: 8px 16px;
  background: #18181b;
  border: 1px solid #27272a;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 700;
  color: #a1a1aa;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.stat-card {
  background: #18181b;
  border: 1px solid #27272a;
  border-radius: 24px;
  padding: 24px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
}

.stat-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: #71717a;
  margin-bottom: 8px;
}

.stat-value {
  font-size: 32px;
  font-weight: 900;
}

.stat-value.blue { color: #3b82f6; }
.stat-value.green { color: #10b981; }
.stat-value.white { color: #ffffff; }
.stat-value.red { color: #ef4444; }

.stat-unit {
  font-size: 11px;
  font-weight: 400;
  opacity: 0.4;
  margin-left: 4px;
}

.charts-section {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

.chart-card {
  background: #18181b;
  border: 1px solid #27272a;
  border-radius: 32px;
  padding: 32px;
}

.chart-title {
  font-size: 14px;
  font-weight: 700;
  color: #d4d4d8;
  margin: 0 0 32px 0;
}

.app-usage-list {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.app-usage-item {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.app-usage-header {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 700;
}

.app-name {
  color: #a1a1aa;
}

.app-time {
  color: #3b82f6;
  font-family: 'SF Mono', 'Consolas', monospace;
}

.app-progress-bar {
  width: 100%;
  height: 8px;
  background: #27272a;
  border-radius: 4px;
  overflow: hidden;
}

.app-progress-fill {
  height: 100%;
  background: #3b82f6;
  border-radius: 4px;
  box-shadow: 0 0 10px rgba(59, 130, 246, 0.3);
  transition: width 0.3s ease;
}

.ai-insight-card {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.1), rgba(99, 102, 241, 0.05));
  border: 1px solid rgba(59, 130, 246, 0.2);
  position: relative;
  overflow: hidden;
}

.ai-insight-bg {
  position: absolute;
  top: 0;
  right: 0;
  padding: 32px;
  opacity: 0.1;
  color: #3b82f6;
  transition: transform 0.3s;
}

.ai-insight-card:hover .ai-insight-bg {
  transform: scale(1.1);
}

.ai-insight-content {
  position: relative;
  z-index: 10;
}

.ai-insight-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 24px;
}

.ai-insight-label {
  font-size: 11px;
  font-weight: 900;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: #ffffff;
}

.ai-insight-text {
  font-size: 13px;
  line-height: 1.8;
  color: #d4d4d8;
  margin: 0 0 32px 0;
}

.view-more-btn {
  font-size: 11px;
  font-weight: 700;
  color: #3b82f6;
}

.view-more-btn:hover {
  color: #2563eb;
}

.exit-btn {
  margin-top: 32px;
  padding: 16px 0;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: #52525b;
  transition: color 0.2s;
}

.exit-btn:hover {
  color: #a1a1aa;
}

/* Responsive */
@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .charts-section {
    grid-template-columns: 1fr;
  }

  .settings-grid {
    grid-template-columns: 1fr;
  }
}

/* ===== RESUME MODAL ===== */
.resume-modal {
  background: #0c0c0e;
  border: 1px solid #27272a;
  border-radius: 32px;
  padding: 40px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.resume-header {
  text-align: center;
  margin-bottom: 32px;
}

.resume-icon {
  width: 72px;
  height: 72px;
  margin: 0 auto 16px;
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.2);
  border-radius: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #3b82f6;
}

.resume-title {
  font-size: 24px;
  font-weight: 700;
  color: #ffffff;
  margin: 0 0 8px 0;
}

.resume-subtitle {
  font-size: 14px;
  color: #71717a;
  margin: 0;
}

.resume-loading {
  text-align: center;
  padding: 40px 0;
}

.resume-loading p {
  margin-top: 16px;
  color: #a1a1aa;
  font-size: 14px;
}

.resume-content {
  margin-bottom: 32px;
}

.ai-tip-card {
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.2);
  border-radius: 16px;
  padding: 24px;
}

.ai-tip-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-size: 12px;
  font-weight: 700;
  color: #3b82f6;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.ai-tip-text {
  font-size: 18px;
  font-weight: 600;
  color: #ffffff;
  line-height: 1.6;
  margin: 0;
}

.resume-section {
  background: #18181b;
  border: 1px solid #27272a;
  border-radius: 16px;
  padding: 20px;
  margin-bottom: 16px;
}

.resume-section.highlight {
  background: rgba(16, 185, 129, 0.1);
  border-color: rgba(16, 185, 129, 0.2);
}

.section-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  font-weight: 700;
  color: #71717a;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 12px;
}

.section-content {
  font-size: 14px;
  color: #d4d4d8;
  line-height: 1.6;
  margin: 0;
}

.section-content.bold {
  font-weight: 600;
  color: #10b981;
}

.interruption-tags {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

.resume-tips {
  margin-top: 16px;
}

.tip-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  font-size: 13px;
  color: #a1a1aa;
}

.adjusted-goal {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 16px;
  padding: 12px 16px;
  background: rgba(251, 191, 36, 0.1);
  border: 1px solid rgba(251, 191, 36, 0.2);
  border-radius: 12px;
  font-size: 13px;
  color: #fbbf24;
}

.resume-options {
  margin-bottom: 32px;
}

.options-hint {
  text-align: center;
  color: #71717a;
  font-size: 13px;
  margin-bottom: 16px;
}

.resume-option-btn {
  height: 56px;
  border-radius: 16px;
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 12px;
}

.resume-option-btn.secondary {
  background: #18181b;
  border: 1px solid #27272a;
  color: #d4d4d8;
}

.resume-option-btn.secondary:hover {
  border-color: #3f3f46;
  color: #ffffff;
}

.resume-actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.resume-actions .n-button {
  height: 48px;
  border-radius: 16px;
  font-weight: 600;
}

.cancel-btn {
  background: transparent;
  border: 1px solid #27272a;
  color: #71717a;
}

.cancel-btn:hover {
  border-color: #3f3f46;
  color: #a1a1aa;
}
</style>
