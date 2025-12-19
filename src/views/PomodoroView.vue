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
        <div class="task-card">
          <!-- Task Selection Area -->
          <div class="task-selection-area">
            <div class="task-selection-label">
              <svg viewBox="0 0 24 24" width="18" height="18">
                <path fill="#6366f1" d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z"/>
              </svg>
              <span>选择专注任务</span>
              <n-tag v-if="selectedTaskCategory" :bordered="false" size="small" class="task-category-tag">
                {{ selectedTaskCategory }}
              </n-tag>
            </div>

            <!-- 任务选择/输入 -->
            <div class="task-input-wrapper">
              <select
                v-if="taskOptions.length > 0"
                v-model="selectedTaskId"
                class="task-native-select"
                @change="handleTaskSelectChange"
              >
                <option :value="null" disabled>-- 选择现有任务 --</option>
                <option v-for="task in taskOptions" :key="task.value" :value="task.value">
                  {{ task.label }}
                </option>
              </select>

              <div class="task-or-divider" v-if="taskOptions.length > 0">或</div>

              <div class="new-task-input-row">
                <input
                  v-model="newTaskName"
                  type="text"
                  class="new-task-input"
                  placeholder="输入新任务名称..."
                  @keyup.enter="handleCreateNewTask"
                />
                <button class="create-task-btn" :disabled="!newTaskName.trim()" @click="handleCreateNewTask">
                  创建
                </button>
              </div>
            </div>

            <!-- 当前选中的任务 -->
            <div v-if="currentTaskDisplay" class="selected-task-display">
              <svg viewBox="0 0 24 24" width="16" height="16">
                <path fill="#10b981" d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
              </svg>
              <span>当前任务: {{ currentTaskDisplay }}</span>
              <button class="clear-task-btn" @click="clearSelectedTask">×</button>
            </div>
          </div>

          <!-- AI Suggestions -->
          <div class="ai-suggestions">
            <div class="ai-header">
              <svg class="sparkle-icon" viewBox="0 0 24 24" width="18" height="18">
                <path fill="#818cf8" d="M12 2L9.5 9.5L2 12l7.5 2.5L12 22l2.5-7.5L22 12l-7.5-2.5L12 2z"/>
              </svg>
              <span class="ai-label">AI 任务执行建议</span>
            </div>
            <!-- 结构化显示 -->
            <div v-if="aiSuggestionData" class="suggestion-structured">
              <!-- 目标 -->
              <div v-if="aiSuggestionData.suggestedGoal" class="suggestion-goal">
                <span class="goal-label">目标：</span>
                <span class="goal-text">{{ aiSuggestionData.suggestedGoal }}</span>
              </div>
              <!-- 子任务列表 -->
              <div v-if="aiSuggestionData.subTasks?.length" class="suggestion-steps">
                <div
                  v-for="(task, index) in aiSuggestionData.subTasks.slice(0, 3)"
                  :key="index"
                  class="suggestion-step"
                >
                  <span class="step-number">{{ index + 1 }}</span>
                  <span class="step-text">{{ task }}</span>
                </div>
              </div>
              <!-- 底部信息 -->
              <div class="suggestion-footer">
                <span v-if="aiSuggestionData.estimatedPomodoros" class="pomodoro-estimate">
                  🍅 预计 {{ aiSuggestionData.estimatedPomodoros }} 个番茄钟
                </span>
                <span v-if="aiSuggestionData.tips" class="suggestion-tip">
                  💡 {{ aiSuggestionData.tips }}
                </span>
              </div>
            </div>
            <!-- 简单显示（无结构化数据时） -->
            <div v-else class="suggestion-list">
              <div class="suggestion-item">
                <div class="suggestion-dot blue"></div>
                <span>{{ aiSuggestion1 }}</span>
              </div>
              <div class="suggestion-item">
                <div class="suggestion-dot purple"></div>
                <span>{{ aiSuggestion2 }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Settings Row -->
        <div class="settings-row">
          <!-- Whitelist -->
          <div class="setting-card">
            <span class="setting-label">白名单</span>
            <div class="app-tags">
              <div
                v-for="app in selectedWhitelistApps"
                :key="app.id"
                class="app-tag"
                @click="removeWhitelistApp(app.id)"
              >
                {{ app.shortName }}
              </div>
              <div class="app-tag add" @click="showAppSelector = true">
                <svg viewBox="0 0 24 24" width="14" height="14">
                  <path fill="currentColor" d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
                </svg>
              </div>
            </div>
          </div>

          <!-- Duration -->
          <div class="setting-card">
            <span class="setting-label">专注时长</span>
            <div class="duration-buttons">
              <button
                v-for="opt in durationOptions"
                :key="opt.value"
                class="duration-btn"
                :class="{ active: duration === opt.value }"
                @click="duration = opt.value"
              >
                {{ opt.value }}分钟
              </button>
            </div>
          </div>
        </div>

        <!-- Start Button -->
        <button class="start-button" :disabled="!canStart || loading" @click="handleStartPrep">
          <svg viewBox="0 0 24 24" width="20" height="20">
            <path fill="currentColor" d="M8 5v14l11-7z"/>
          </svg>
          <span>进入深度模式</span>
        </button>
      </div>
    </div>

    <!-- FOCUSING 专注中 -->
    <div v-else-if="currentPhase === 'focusing'" class="scene-container focusing-scene">
      <div class="focusing-content">
        <!-- Timer Circle -->
        <div class="timer-container">
          <div class="timer-ring">
            <svg viewBox="0 0 320 320" class="progress-svg">
              <circle
                cx="160" cy="160" r="144"
                fill="none"
                stroke="rgba(30, 41, 59, 0.5)"
                stroke-width="12"
              />
              <circle
                cx="160" cy="160" r="144"
                fill="none"
                stroke="#6366f1"
                stroke-width="12"
                stroke-linecap="round"
                :stroke-dasharray="circumference"
                :stroke-dashoffset="progressOffset"
                class="progress-bar"
              />
            </svg>
            <div class="timer-content">
              <div class="time-display">{{ formattedTime }}</div>
              <div class="active-indicator">
                <svg viewBox="0 0 24 24" width="12" height="12">
                  <path fill="#6366f1" d="M13 3v9h7l-8 9v-9H5l8-9z"/>
                </svg>
                <span>{{ currentAppName }} • 活跃中</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Task Title -->
        <div class="focus-info">
          <h2 class="focus-task-title">{{ currentSession?.task?.title || '深度专注中' }}</h2>
          <div class="focus-status">
            <div class="status-dots">
              <span class="dot active"></span>
              <span class="dot"></span>
              <span class="dot"></span>
            </div>
            <span class="status-text">AI 正在守护你的注意力通道</span>
          </div>
        </div>

        <!-- Controls -->
        <div class="focus-controls">
          <button class="control-btn" @click="handlePause">
            <svg v-if="!isPaused" viewBox="0 0 24 24" width="24" height="24">
              <path fill="currentColor" d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" width="24" height="24">
              <path fill="currentColor" d="M8 5v14l11-7z"/>
            </svg>
          </button>
          <button class="control-btn stop" @click="handleStop">
            <svg viewBox="0 0 24 24" width="24" height="24">
              <path fill="currentColor" d="M6 6h12v12H6z"/>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- REPORT 结束报告 -->
    <div v-else-if="currentPhase === 'report'" class="scene-container report-scene">
      <div class="report-card">
        <!-- Header -->
        <div class="report-header">
          <div class="report-icon">
            <svg viewBox="0 0 24 24" width="28" height="28">
              <path fill="#6366f1" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
            </svg>
          </div>
          <div class="report-title-section">
            <h2 class="report-title">{{ reportTaskTitle }}</h2>
            <p class="report-meta">本次专注: {{ sessionDuration }} 分钟 • 专注率 {{ sessionFocusRate }}%</p>
          </div>
          <div class="session-tag">SESSION #{{ todaySessionCount }}</div>
        </div>

        <!-- Progress Update Section -->
        <div v-if="currentSession?.taskId" class="progress-update-section">
          <label class="section-label">更新任务进度</label>
          <div class="progress-slider-wrapper">
            <input
              type="range"
              v-model="progressValue"
              min="0"
              max="100"
              step="5"
              class="progress-slider"
            />
            <span class="progress-display">{{ progressValue }}%</span>
          </div>
        </div>

        <!-- Task Completion Toggle -->
        <div class="task-completion-section">
          <label class="completion-toggle" @click="taskCompleted = !taskCompleted">
            <div class="toggle-checkbox" :class="{ checked: taskCompleted }">
              <svg v-if="taskCompleted" viewBox="0 0 24 24" width="16" height="16">
                <path fill="currentColor" d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
              </svg>
            </div>
            <span class="toggle-label">已完成「{{ currentSession?.task?.title || '当前任务' }}」</span>
          </label>
        </div>

        <!-- Add Milestone Toggle -->
        <div v-if="currentSession?.taskId" class="milestone-section">
          <label class="completion-toggle" @click="addMilestone = !addMilestone">
            <div class="toggle-checkbox milestone-checkbox" :class="{ checked: addMilestone }">
              <svg v-if="addMilestone" viewBox="0 0 24 24" width="16" height="16">
                <path fill="currentColor" d="M14.4 6L14 4H5v17h2v-7h5.6l.4 2h7V6z"/>
              </svg>
            </div>
            <span class="toggle-label">添加里程碑记录</span>
          </label>
          <input
            v-if="addMilestone"
            v-model="milestoneTitle"
            class="milestone-input"
            placeholder="里程碑标题（可选，默认使用本次产出记录）"
          />
        </div>

        <!-- Feedback Input -->
        <div class="feedback-section">
          <label class="feedback-label">记录本次产出</label>
          <textarea
            v-model="feedback"
            class="feedback-textarea"
            :placeholder="`描述一下在「${currentSession?.task?.title || '本次专注'}」中完成了什么...`"
          ></textarea>
        </div>

        <!-- Submit Button -->
        <button class="submit-button" :disabled="loading" @click="handleComplete">
          <svg viewBox="0 0 24 24" width="20" height="20">
            <path fill="currentColor" d="M12 2L9.5 9.5L2 12l7.5 2.5L12 22l2.5-7.5L22 12l-7.5-2.5L12 2z"/>
          </svg>
          <span>{{ taskCompleted ? '完成任务并提交' : '保存进度并提交' }}</span>
        </button>
      </div>
    </div>

    <!-- STATS 统计看板 -->
    <div v-else-if="currentPhase === 'stats'" class="scene-container stats-scene">
      <div class="stats-content">
        <!-- Header -->
        <div class="stats-header">
          <div class="stats-title-row">
            <svg viewBox="0 0 24 24" width="28" height="28">
              <path fill="#6366f1" d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z"/>
            </svg>
            <h2 class="stats-title">深度效能看板</h2>
          </div>
          <div class="date-tag">{{ currentDate }}</div>
        </div>

        <!-- Stats Grid -->
        <div class="stats-grid">
          <div class="stat-item">
            <span class="stat-label">今日番茄</span>
            <span class="stat-value blue">{{ todayStats?.completedSessions || 0 }}<small>个</small></span>
          </div>
          <div class="stat-item">
            <span class="stat-label">专注效率</span>
            <span class="stat-value green">{{ Math.round(todayStats?.avgFocusRate || 0) }}<small>%</small></span>
          </div>
          <div class="stat-item">
            <span class="stat-label">心流时长</span>
            <span class="stat-value white">{{ todayStats?.totalFocusMinutes || 0 }}<small>min</small></span>
          </div>
          <div class="stat-item">
            <span class="stat-label">阻断分心</span>
            <span class="stat-value red">{{ todayStats?.totalDistractions || 0 }}<small>次</small></span>
          </div>
        </div>

        <!-- Charts Row -->
        <div class="charts-row">
          <!-- App Usage -->
          <div class="chart-card">
            <h3 class="chart-title">核心产出工具分布</h3>
            <div class="usage-list">
              <template v-if="appUsageList.length > 0">
                <div v-for="app in appUsageList" :key="app.appName" class="usage-item">
                  <div class="usage-header">
                    <span class="app-name">{{ app.appName }}</span>
                    <span class="app-time">{{ app.minutes }} MIN</span>
                  </div>
                  <div class="usage-bar">
                    <div class="usage-fill" :style="{ width: app.percentage + '%' }"></div>
                  </div>
                </div>
              </template>
              <div v-else class="usage-empty">
                <svg viewBox="0 0 24 24" width="32" height="32">
                  <path fill="#4b5563" d="M13 9h-2V7h2m0 10h-2v-6h2m-1-9A10 10 0 0 0 2 12a10 10 0 0 0 10 10 10 10 0 0 0 10-10A10 10 0 0 0 12 2z"/>
                </svg>
                <p>完成番茄钟后将记录应用使用情况</p>
              </div>
            </div>
          </div>

          <!-- AI Insight -->
          <div class="chart-card ai-card">
            <div class="ai-card-header">
              <svg viewBox="0 0 24 24" width="20" height="20">
                <path fill="#6366f1" d="M12 2L9.5 9.5L2 12l7.5 2.5L12 22l2.5-7.5L22 12l-7.5-2.5L12 2z"/>
              </svg>
              <span>AI COACH DAILY</span>
            </div>
            <p class="ai-insight-text">
              <template v-if="aiInsightLoading">AI 正在分析今日数据...</template>
              <template v-else-if="aiDailyInsight">{{ aiDailyInsight }}</template>
              <template v-else-if="todayStats?.aiInsight">{{ todayStats.aiInsight }}</template>
              <template v-else>完成更多番茄钟后，AI 将为你生成个性化的每日复盘与建议。</template>
            </p>
            <button class="view-report-btn" @click="handleViewWeeklyReport">
              查看完整周报
              <svg viewBox="0 0 24 24" width="14" height="14">
                <path fill="currentColor" d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Exit Button -->
        <button class="exit-button" @click="handleBackToPrep">
          EXIT FOCUS MODE
        </button>
      </div>
    </div>

    <!-- Resume Modal -->
    <div v-if="showResumeModal" class="modal-overlay" @click.self="handleCancelResume">
      <div class="resume-modal">
        <div class="modal-icon">
          <svg viewBox="0 0 24 24" width="32" height="32">
            <path fill="#6366f1" d="M8 5v14l11-7z"/>
          </svg>
        </div>
        <h3 class="modal-title">继续专注</h3>
        <p class="modal-subtitle">已暂停 {{ interruptionDuration }} 分钟</p>

        <div v-if="currentSession?.task" class="modal-task">
          <span class="task-label">当前任务</span>
          <span class="task-name">{{ currentSession.task.title }}</span>
        </div>

        <div class="modal-actions">
          <button class="modal-btn primary" @click="handleConfirmResume">
            <svg viewBox="0 0 24 24" width="18" height="18">
              <path fill="currentColor" d="M8 5v14l11-7z"/>
            </svg>
            继续专注
          </button>
          <button class="modal-btn secondary" @click="handleCancelResume">取消</button>
        </div>
      </div>
    </div>

    <!-- App Selector Modal -->
    <div v-if="showAppSelector" class="modal-overlay" @click.self="showAppSelector = false">
      <div class="app-selector-modal">
        <div class="modal-header">
          <h3>选择白名单应用</h3>
          <button class="close-btn" @click="showAppSelector = false">
            <svg viewBox="0 0 24 24" width="20" height="20">
              <path fill="currentColor" d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
            </svg>
          </button>
        </div>

        <div class="app-list">
          <div class="app-list-title">常用开发工具</div>
          <div class="app-grid">
            <div
              v-for="app in availableApps"
              :key="app.id"
              class="app-item"
              :class="{ selected: isAppSelected(app.id) }"
              @click="toggleAppSelection(app.id)"
            >
              <div class="app-icon-text">{{ app.shortName }}</div>
              <span class="app-name">{{ app.name }}</span>
              <div v-if="isAppSelected(app.id)" class="check-icon">
                <svg viewBox="0 0 24 24" width="16" height="16">
                  <path fill="#6366f1" d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                </svg>
              </div>
            </div>
          </div>
        </div>

        <div class="add-app-section">
          <div class="app-list-title">添加其他应用</div>
          <div class="add-app-row">
            <input
              v-model="appInputName"
              type="text"
              class="add-app-input"
              placeholder="输入应用名称..."
              @keyup.enter="handleAddApp"
            />
            <button class="add-app-btn" :disabled="!appInputName.trim()" @click="handleAddApp">
              添加
            </button>
          </div>
        </div>

        <div class="modal-footer">
          <button class="confirm-btn" @click="showAppSelector = false">确定 ({{ selectedAppCount }} 个)</button>
        </div>
      </div>
    </div>

    <!-- Distraction Alert Modal 分心提醒弹窗 -->
    <div v-if="showDistractionModal" class="modal-overlay distraction-overlay">
      <div class="distraction-modal">
        <div class="distraction-icon">
          <svg viewBox="0 0 24 24" width="40" height="40">
            <path fill="#ef4444" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
          </svg>
        </div>
        <h2 class="distraction-title">注意力偏离！</h2>
        <p class="distraction-desc">
          检测到非工作应用 <strong>{{ distractedAppName }}</strong> 被打开。
        </p>
        <p class="distraction-hint">这是工作中的必要查找，还是需要休息？</p>

        <div class="distraction-actions">
          <button class="distraction-btn primary" @click="handleReturnToWork">
            立即重返工作
          </button>
          <button class="distraction-btn secondary" @click="handleStopTimer">
            停止本次计时
          </button>
        </div>
      </div>
    </div>

    <!-- Scene Switcher (Debug) - 仅开发环境显示 -->
    <div v-if="isDev" class="scene-switcher">
      <button
        v-for="scene in scenes"
        :key="scene"
        class="scene-btn"
        :class="{ active: currentPhase === scene || (scene === 'distracted' && showDistractionModal) }"
        @click="handleSceneSwitch(scene)"
      >
        {{ scene.toUpperCase() }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NTag, useMessage } from 'naive-ui'
import { usePomodoroStore } from '@/stores/pomodoroStore'
import { useTaskStore } from '@/stores/taskStore'
import { aiTaskBreakdown, aiFocusAnalysis, aiDailyReview, getActiveWindowInfo, getRunningApps, type RunningApp } from '@/api/pomodoroApi'
import { taskApi } from '@/api/taskApi'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { PomodoroPhase, FocusApp, AppUsageStats } from '@/types/pomodoro'

const route = useRoute()
const router = useRouter()
const message = useMessage()
const pomodoroStore = usePomodoroStore()
const taskStore = useTaskStore()

// 开发环境判断
const isDev = import.meta.env.DEV

// Phase Control
const currentPhase = ref<PomodoroPhase>('prep')
const scenes = ['prep', 'focusing', 'report', 'stats', 'distracted']

// Task Selection
const selectedTaskId = ref<number | null>(null)
const newTaskName = ref('')

const taskOptions = computed(() => {
  const tasks = [...taskStore.activeTasks, ...taskStore.todoTasks]
  return tasks.map(t => ({
    label: t.title,
    value: t.id
  }))
})

const selectedTask = computed(() => {
  if (selectedTaskId.value !== null) {
    return taskStore.tasks.find(t => t.id === selectedTaskId.value)
  }
  return null
})

const selectedTaskCategory = computed(() => {
  return selectedTask.value?.category || null
})

// 当前任务显示文本
const currentTaskDisplay = computed(() => {
  if (selectedTask.value) {
    return selectedTask.value.title
  }
  return null
})

const canStart = computed(() => {
  return selectedTaskId.value !== null
})

function handleTaskSelectChange() {
  // 原生 select 已经通过 v-model 更新了 selectedTaskId
}

async function handleCreateNewTask() {
  if (!newTaskName.value.trim()) return

  try {
    const taskId = await taskStore.createTask(newTaskName.value.trim())
    selectedTaskId.value = taskId
    message.success('任务已创建')
    newTaskName.value = ''
  } catch (error) {
    message.error('创建任务失败')
  }
}

function clearSelectedTask() {
  selectedTaskId.value = null
}

// AI Suggestions - 从后端获取真正的AI建议
const aiSuggestions = ref<string[]>([])
const aiSuggestionsLoading = ref(false)

// 监听任务选择变化，获取AI建议
watch(selectedTaskId, async (newTaskId) => {
  if (newTaskId && selectedTask.value) {
    await fetchAiSuggestions()
  } else {
    aiSuggestions.value = []
    aiSuggestionData.value = null
  }
})

// 生成默认的任务建议
function getDefaultSuggestions(taskTitle: string): string[] {
  return [
    `专注完成「${taskTitle}」的核心部分`,
    `建议：先理清思路，再动手执行`
  ]
}

// AI建议的结构化数据
interface AiSuggestionData {
  suggestedGoal?: string
  subTasks?: string[]
  estimatedPomodoros?: number
  tips?: string
}

const aiSuggestionData = ref<AiSuggestionData | null>(null)

// 获取AI任务拆解建议
async function fetchAiSuggestions() {
  if (!selectedTask.value) return

  aiSuggestionsLoading.value = true
  aiSuggestionData.value = null
  try {
    const result = await aiTaskBreakdown(
      selectedTask.value.title,
      selectedTask.value.description || undefined
    )
    // 解析AI返回的建议
    try {
      const parsed = JSON.parse(result)
      // 处理结构化的AI返回
      if (parsed.sub_tasks || parsed.subTasks || parsed.suggested_goal) {
        aiSuggestionData.value = {
          suggestedGoal: parsed.suggested_goal || parsed.suggestedGoal,
          subTasks: (parsed.sub_tasks || parsed.subTasks || []).map((t: string) =>
            t.replace(/^\d+\.\s*/, '') // 移除开头的数字编号
          ),
          estimatedPomodoros: parsed.estimated_pomodoros || parsed.estimatedPomodoros,
          tips: parsed.tips
        }
        // 同时设置 aiSuggestions 用于简单显示
        aiSuggestions.value = aiSuggestionData.value.subTasks?.slice(0, 3) || []
      } else if (parsed.suggestions && Array.isArray(parsed.suggestions)) {
        aiSuggestions.value = parsed.suggestions.slice(0, 3)
      } else {
        aiSuggestions.value = [result]
      }
    } catch {
      // 如果不是JSON，直接使用文本
      const lines = result.split('\n').filter((l: string) => l.trim())
      aiSuggestions.value = lines.slice(0, 3)
    }
  } catch (error: any) {
    console.error('获取AI建议失败:', error)
    // 检查是否是 AI 未配置的错误
    const errorMsg = error?.toString() || ''
    if (errorMsg.includes('AI 服务未初始化') || errorMsg.includes('AI 服务未配置')) {
      // AI 未配置，使用默认建议并提示用户
      aiSuggestions.value = [
        `专注完成「${selectedTask.value?.title || '当前任务'}」`,
        '提示：配置 AI 后可获得智能任务拆解建议'
      ]
    } else {
      // 其他错误，使用默认建议
      aiSuggestions.value = getDefaultSuggestions(selectedTask.value?.title || '当前任务')
    }
  } finally {
    aiSuggestionsLoading.value = false
  }
}

const aiSuggestion1 = computed(() => {
  if (aiSuggestionsLoading.value) {
    return 'AI 正在分析任务...'
  }
  if (aiSuggestions.value.length > 0) {
    return aiSuggestions.value[0]
  }
  if (selectedWhitelistApps.value.length > 0) {
    const appNames = selectedWhitelistApps.value.map(a => a.name).join('、')
    return `白名单应用：${appNames}。切换到其他应用时会提醒你。`
  }
  return '选择白名单应用后，切换到其他应用时会提醒你专注。'
})

const aiSuggestion2 = computed(() => {
  if (aiSuggestionsLoading.value) {
    return '请稍候...'
  }
  if (aiSuggestions.value.length > 1) {
    return aiSuggestions.value[1]
  }
  if (selectedTask.value?.title) {
    return `目标：专注完成「${selectedTask.value.title}」`
  }
  return '请选择或输入一个任务，AI 将为你定制专注建议。'
})

// Duration
const duration = ref(25)
const durationOptions = [
  { label: '15 分钟', value: 15 },
  { label: '25 分钟', value: 25 },
  { label: '45 分钟', value: 45 },
  { label: '60 分钟', value: 60 }
]

// App Selection - 白名单应用
const selectedAppProcessNames = ref<string[]>([]) // 改用进程名作为标识
const showAppSelector = ref(false)
const appInputName = ref('')
const runningAppsLoading = ref(false)
const runningAppsList = ref<RunningApp[]>([])

// 可用应用列表（来自运行中的应用）
const availableApps = computed(() => {
  return runningAppsList.value.map((app, index) => ({
    id: index,
    name: app.name,
    shortName: app.name.substring(0, 3).toUpperCase(),
    processName: app.processName
  }))
})

// 加载运行中的应用
async function loadRunningApps() {
  runningAppsLoading.value = true
  try {
    runningAppsList.value = await getRunningApps()
  } catch (error) {
    console.error('获取运行应用失败:', error)
    // 如果获取失败，使用默认应用列表
    runningAppsList.value = [
      { name: 'VS Code', processName: 'Code.exe' },
      { name: 'Chrome', processName: 'chrome.exe' },
      { name: 'Edge', processName: 'msedge.exe' },
      { name: 'Terminal', processName: 'WindowsTerminal.exe' }
    ]
  } finally {
    runningAppsLoading.value = false
  }
}

const selectedWhitelistApps = computed(() => {
  return availableApps.value.filter(app =>
    selectedAppProcessNames.value.includes(app.processName.toLowerCase())
  )
})

// 获取选中应用的名称列表（用于传递给后端）
const selectedAppNames = computed(() => {
  return selectedWhitelistApps.value.map(app => app.name)
})

// 选中的应用数量
const selectedAppCount = computed(() => selectedAppProcessNames.value.length)

function isAppSelected(appId: number) {
  const app = availableApps.value.find(a => a.id === appId)
  if (!app?.processName) return false
  return selectedAppProcessNames.value.includes(app.processName.toLowerCase())
}

function toggleAppSelection(appId: number) {
  const app = availableApps.value.find(a => a.id === appId)
  if (!app?.processName) return

  const processName = app.processName.toLowerCase()
  const index = selectedAppProcessNames.value.indexOf(processName)
  if (index > -1) {
    selectedAppProcessNames.value.splice(index, 1)
  } else {
    selectedAppProcessNames.value.push(processName)
  }
}

function removeWhitelistApp(appId: number) {
  const app = availableApps.value.find(a => a.id === appId)
  if (!app?.processName) return

  const processName = app.processName.toLowerCase()
  const index = selectedAppProcessNames.value.indexOf(processName)
  if (index > -1) {
    selectedAppProcessNames.value.splice(index, 1)
  }
}

// Focus State - 实时活动窗口检测
const currentAppName = ref('检测中...')
const isDistracted = ref(false)
const distractionTimer = ref<number | null>(null)
const windowCheckInterval = ref<number | null>(null)

// 智能分心检测（无白名单时）
const smartDistractionSeconds = ref(0) // 离开DevAssistant的累计秒数
const SMART_DISTRACTION_THRESHOLD = 180 // 3分钟后提醒
const lastSmartReminderTime = ref<number>(0) // 上次提醒时间，避免频繁提醒
const SMART_REMINDER_COOLDOWN = 300000 // 5分钟冷却期

// 分心弹窗状态
const showDistractionModal = ref(false)
const distractedAppName = ref('')

// 获取选中白名单应用的进程名列表
const whitelistProcessNames = computed(() => {
  return selectedWhitelistApps.value
    .filter(app => app.processName)
    .map(app => app.processName!.toLowerCase())
})

// 检测当前窗口是否在白名单中
async function checkActiveWindow() {
  try {
    const windowInfo = await getActiveWindowInfo()

    // 更新当前应用名称
    if (windowInfo.appName) {
      currentAppName.value = windowInfo.appName
    } else if (windowInfo.processName) {
      // 去掉 .exe 后缀显示
      currentAppName.value = windowInfo.processName.replace(/\.exe$/i, '')
    } else {
      currentAppName.value = '未知应用'
    }

    // 只在专注阶段检测分心（且会话状态为 focusing）
    if (currentPhase.value === 'focusing' && currentSession.value?.status === 'focusing' && !showDistractionModal.value) {
      checkDistraction(windowInfo)
    }
  } catch (error) {
    console.error('获取活动窗口失败:', error)
    currentAppName.value = '检测失败'
  }
}

// 单次获取当前窗口（不检测分心，仅更新显示）
async function fetchCurrentWindow() {
  try {
    const windowInfo = await getActiveWindowInfo()
    if (windowInfo.appName) {
      currentAppName.value = windowInfo.appName
    } else if (windowInfo.processName) {
      currentAppName.value = windowInfo.processName.replace(/\.exe$/i, '')
    } else {
      currentAppName.value = '未知应用'
    }
  } catch (error) {
    console.error('获取窗口信息失败:', error)
    currentAppName.value = '无法检测'
  }
}

// 检测是否分心（切换到非白名单应用）
function checkDistraction(windowInfo: { appName: string | null, processName: string | null }) {
  const currentProcess = windowInfo.processName?.toLowerCase().replace(/\.exe$/i, '') || ''
  const currentApp = windowInfo.appName?.toLowerCase() || ''

  // 检查是否在 DevAssistant 内
  const isInDevAssistant = currentProcess.includes('dev-assistant') || currentProcess.includes('devassistant')

  // 忽略系统进程和空进程
  const isSystemProcess = !currentProcess || currentProcess === 'explorer' || currentProcess === 'searchhost'

  // 没有设置白名单时，使用智能分心检测
  if (selectedAppProcessNames.value.length === 0) {
    if (isInDevAssistant || isSystemProcess) {
      // 在 DevAssistant 内或系统进程，重置计时
      smartDistractionSeconds.value = 0
      isDistracted.value = false
      return
    }

    // 累加在外部应用的时间（每次检测间隔约2秒）
    smartDistractionSeconds.value += 2

    // 检查是否超过阈值且不在冷却期
    const now = Date.now()
    if (smartDistractionSeconds.value >= SMART_DISTRACTION_THRESHOLD &&
        (now - lastSmartReminderTime.value) > SMART_REMINDER_COOLDOWN) {
      // 触发智能分心提醒
      lastSmartReminderTime.value = now
      smartDistractionSeconds.value = 0 // 重置计时
      const detectedAppName = windowInfo.appName || windowInfo.processName?.replace(/\.exe$/i, '') || '其他应用'
      handleSmartDistraction(detectedAppName)
    }

    isDistracted.value = false
    return
  }

  // 以下是有白名单时的检测逻辑

  // 忽略本应用自身（DevAssistant）
  if (isInDevAssistant) {
    isDistracted.value = false
    return
  }

  // 忽略系统进程和空进程
  if (isSystemProcess) {
    isDistracted.value = false
    return
  }

  // 改进的白名单匹配逻辑
  const isInWhitelist = selectedWhitelistApps.value.some(app => {
    // 1. 通过进程名匹配（去掉.exe后缀）
    if (app.processName) {
      const whitelistProcess = app.processName.toLowerCase().replace(/\.exe$/i, '')
      // 精确匹配或前缀匹配（如 code 匹配 code, webstorm64 匹配 webstorm64）
      if (currentProcess === whitelistProcess ||
          currentProcess.startsWith(whitelistProcess) ||
          whitelistProcess.startsWith(currentProcess)) {
        return true
      }
    }

    // 2. 通过应用名称匹配
    const whitelistName = app.name.toLowerCase()
    // 检查应用名是否包含白名单名称，或反过来
    if (currentApp && (
        currentApp.includes(whitelistName) ||
        whitelistName.includes(currentApp) ||
        currentProcess.includes(whitelistName.replace(/\s+/g, ''))
    )) {
      return true
    }

    return false
  })

  if (!isInWhitelist) {
    if (!isDistracted.value) {
      isDistracted.value = true
      // 传递检测到的应用名称
      const detectedAppName = windowInfo.appName || windowInfo.processName?.replace(/\.exe$/i, '') || '未知应用'
      handleDistraction(detectedAppName)
    }
  } else {
    isDistracted.value = false
  }
}

// 播放分心提醒音效
function playDistractionSound() {
  try {
    // 使用 Web Audio API 播放简短的警告音
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
    const oscillator = audioContext.createOscillator()
    const gainNode = audioContext.createGain()

    oscillator.connect(gainNode)
    gainNode.connect(audioContext.destination)

    oscillator.frequency.value = 800 // 频率
    oscillator.type = 'sine'
    gainNode.gain.value = 0.3 // 音量

    oscillator.start()
    // 播放两声短促的提示音
    setTimeout(() => {
      gainNode.gain.value = 0
    }, 150)
    setTimeout(() => {
      gainNode.gain.value = 0.3
    }, 200)
    setTimeout(() => {
      oscillator.stop()
      audioContext.close()
    }, 350)
  } catch (error) {
    console.error('播放提示音失败:', error)
  }
}

// 将窗口置顶到前台
async function bringWindowToFront() {
  try {
    const appWindow = getCurrentWindow()
    await appWindow.setFocus()
    await appWindow.unminimize()
  } catch (error) {
    console.error('窗口置顶失败:', error)
  }
}

// 处理分心事件（有白名单时）
async function handleDistraction(appName: string) {
  if (!currentSession.value?.id) return

  // 如果弹窗已经显示，不重复触发
  if (showDistractionModal.value) return

  try {
    // 记录分心的应用名称（直接使用传入的参数）
    distractedAppName.value = appName

    // 暂停计时
    await pomodoroStore.pauseSession(currentSession.value.id)

    // 停止窗口监控（避免重复触发）
    stopWindowMonitoring()

    // 记录分心次数
    await pomodoroStore.recordDistraction(currentSession.value.id)

    // 播放提示音
    playDistractionSound()

    // 将窗口置顶到前台
    await bringWindowToFront()

    // 显示分心弹窗
    showDistractionModal.value = true
  } catch (error) {
    console.error('处理分心失败:', error)
  }
}

// 处理智能分心提醒（无白名单时的温和提醒）
async function handleSmartDistraction(appName: string) {
  if (!currentSession.value?.id) return

  try {
    // 记录分心次数（不暂停会话）
    await pomodoroStore.recordDistraction(currentSession.value.id)

    // 播放轻柔的提示音
    playGentleReminder()

    // 将窗口置顶
    await bringWindowToFront()

    // 显示温和的提示消息
    message.warning(`已在 ${appName} 中使用 3 分钟，记得回来专注哦！`, 5000)
  } catch (error) {
    console.error('智能分心提醒失败:', error)
  }
}

// 播放温和的提醒音效
function playGentleReminder() {
  try {
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
    const oscillator = audioContext.createOscillator()
    const gainNode = audioContext.createGain()

    oscillator.connect(gainNode)
    gainNode.connect(audioContext.destination)

    oscillator.frequency.value = 440 // 更柔和的频率
    oscillator.type = 'sine'
    gainNode.gain.value = 0.15 // 更小的音量

    oscillator.start()
    setTimeout(() => {
      oscillator.stop()
      audioContext.close()
    }, 200)
  } catch (error) {
    console.error('播放提醒音失败:', error)
  }
}

// 立即重返工作
async function handleReturnToWork() {
  if (!currentSession.value?.id) return

  try {
    // 恢复计时
    await pomodoroStore.resumeSession(currentSession.value.id)

    // 关闭弹窗
    showDistractionModal.value = false
    isDistracted.value = false

    message.success('继续专注！请切换回工作应用')

    // 延迟3秒后重新开始窗口监控，给用户时间切换回工作应用
    setTimeout(() => {
      if (currentSession.value?.status === 'focusing') {
        startWindowMonitoring()
      }
    }, 3000)
  } catch (error) {
    console.error('恢复专注失败:', error)
    message.error('恢复失败，请重试')
  }
}

// 停止本次计时
async function handleStopTimer() {
  // 关闭弹窗
  showDistractionModal.value = false
  isDistracted.value = false

  // 跳转到报告页面
  progressValue.value = getCurrentTaskProgress()
  currentPhase.value = 'report'
  message.info('专注已停止，请提交反馈')
}

// 场景切换（调试用）
function handleSceneSwitch(scene: string) {
  if (scene === 'distracted') {
    // 显示分心弹窗（调试用，使用当前检测到的应用名称）
    distractedAppName.value = currentAppName.value || '测试应用'
    showDistractionModal.value = true
  } else {
    showDistractionModal.value = false
    currentPhase.value = scene as PomodoroPhase
  }
}

// 开始窗口监控
function startWindowMonitoring() {
  if (windowCheckInterval.value) {
    clearInterval(windowCheckInterval.value)
  }
  // 每2秒检测一次活动窗口
  windowCheckInterval.value = window.setInterval(checkActiveWindow, 2000)
  // 立即执行一次
  checkActiveWindow()
}

// 停止窗口监控
function stopWindowMonitoring() {
  if (windowCheckInterval.value) {
    clearInterval(windowCheckInterval.value)
    windowCheckInterval.value = null
  }
  // 重置智能分心检测计时
  smartDistractionSeconds.value = 0
}

// Resume Modal
const showResumeModal = ref(false)

// Report
const feedback = ref('')
const taskCompleted = ref(false)
const progressValue = ref(0)
const addMilestone = ref(false)
const milestoneTitle = ref('')
const loading = computed(() => pomodoroStore.loading)

// 获取当前任务的进度（从taskStore获取最新数据）
function getCurrentTaskProgress(): number {
  const taskId = currentSession.value?.taskId
  if (!taskId) return 0
  // 优先从taskStore获取最新进度
  const task = taskStore.tasks.find(t => t.id === taskId)
  if (task?.progress !== undefined) return task.progress
  // 备选：从session中获取
  return currentSession.value?.task?.progress || 0
}

// Report task title
const reportTaskTitle = computed(() => {
  if (currentSession.value?.task?.title) {
    return `「${currentSession.value.task.title}」专注完成`
  }
  if (currentSession.value?.focusGoal) {
    return currentSession.value.focusGoal
  }
  return '专注完成'
})

// Session Data
const currentSession = computed(() => pomodoroStore.currentSession)
const isPaused = computed(() => currentSession.value?.status === 'paused')
const todayStats = computed(() => pomodoroStore.todayStats)
const focusApps = computed(() => pomodoroStore.focusApps)

// Timer
const remainingSeconds = computed(() => pomodoroStore.remainingTime)
const formattedTime = computed(() => {
  const mins = Math.floor(remainingSeconds.value / 60)
  const secs = remainingSeconds.value % 60
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
})

const circumference = 2 * Math.PI * 144
const progressOffset = computed(() => {
  const progress = pomodoroStore.focusProgress
  return circumference - (progress / 100) * circumference
})

// Stats
const sessionDuration = computed(() => currentSession.value?.durationMinutes || 25)
const sessionFocusRate = computed(() => Math.round(currentSession.value?.focusRate || 0))
const todaySessionCount = computed(() => pomodoroStore.todaySessions.length)
const currentDate = computed(() => {
  return new Date().toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
})

// AI 每日复盘
const aiDailyInsight = ref('')
const aiInsightLoading = ref(false)

// 获取 AI 每日复盘
async function fetchAiDailyReview() {
  if (aiInsightLoading.value) return

  aiInsightLoading.value = true
  try {
    const today = new Date().toISOString().split('T')[0]
    const result = await aiDailyReview(today)
    aiDailyInsight.value = result
  } catch (error) {
    console.error('获取 AI 复盘失败:', error)
    aiDailyInsight.value = ''
  } finally {
    aiInsightLoading.value = false
  }
}

// 监听进入统计页面时获取 AI 复盘
watch(currentPhase, async (newPhase) => {
  if (newPhase === 'stats') {
    // 刷新统计数据
    await pomodoroStore.loadTodayStats()
    await pomodoroStore.loadTodaySessions()
    // 获取 AI 每日复盘
    if (!aiDailyInsight.value) {
      fetchAiDailyReview()
    }
  }
})

// 监听计时完成
watch(remainingSeconds, (newValue, oldValue) => {
  // 当剩余时间从大于0变为0时，表示计时完成
  if (oldValue > 0 && newValue <= 0 && currentPhase.value === 'focusing' && currentSession.value?.status === 'focusing') {
    handleTimerComplete()
  }
})

// 播放完成提示音（更悦耳的音效）
function playCompleteSound() {
  try {
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
    const oscillator = audioContext.createOscillator()
    const gainNode = audioContext.createGain()

    oscillator.connect(gainNode)
    gainNode.connect(audioContext.destination)

    oscillator.type = 'sine'
    gainNode.gain.value = 0.3

    // 播放一个上升的音调表示完成
    const now = audioContext.currentTime
    oscillator.frequency.setValueAtTime(523.25, now) // C5
    oscillator.frequency.setValueAtTime(659.25, now + 0.15) // E5
    oscillator.frequency.setValueAtTime(783.99, now + 0.3) // G5

    oscillator.start(now)
    oscillator.stop(now + 0.5)

    setTimeout(() => audioContext.close(), 600)
  } catch (error) {
    console.error('播放完成音效失败:', error)
  }
}

// 处理计时完成
async function handleTimerComplete() {
  // 停止窗口监控
  stopWindowMonitoring()

  // 播放完成音效
  playCompleteSound()

  // 将窗口置顶
  await bringWindowToFront()

  // 显示完成提示
  message.success('🎉 番茄钟完成！请记录你的产出')

  // 刷新任务列表以获取最新进度
  if (currentSession.value?.taskId) {
    await taskStore.loadTasks()
  }

  // 跳转到报告页面
  progressValue.value = getCurrentTaskProgress()
  currentPhase.value = 'report'
}

// 解析后端返回的应用使用统计数据
const appUsageList = computed(() => {
  if (!todayStats.value?.appUsage) {
    return []
  }

  // 后端可能返回 JSON 字符串或对象数组
  let usageData = todayStats.value.appUsage
  if (typeof usageData === 'string') {
    try {
      usageData = JSON.parse(usageData)
    } catch {
      return []
    }
  }

  if (!Array.isArray(usageData) || usageData.length === 0) {
    return []
  }

  // 计算总时长用于百分比
  const totalMinutes = usageData.reduce((sum: number, item: any) => sum + (item.minutes || 0), 0)

  return usageData.map((item: any) => ({
    appName: item.appName || item.app_name || '未知',
    minutes: item.minutes || 0,
    percentage: totalMinutes > 0 ? Math.round((item.minutes || 0) / totalMinutes * 100) : 0
  }))
})
const interruptionDuration = computed(() => pomodoroStore.getInterruptionDuration())

// Methods
async function handleStartPrep() {
  if (!selectedTaskId.value) {
    message.warning('请先选择或创建任务')
    return
  }

  try {
    // 构建专注目标
    const focusGoal = selectedTask.value?.title
      ? `完成「${selectedTask.value.title}」`
      : '专注工作'

    // 获取选中的白名单应用名称
    const focusAppsList = selectedAppNames.value

    const session = await pomodoroStore.createSession({
      taskId: selectedTaskId.value,
      durationMinutes: duration.value,
      focusGoal: focusGoal,
      focusApps: focusAppsList
    })

    if (session.id) {
      await pomodoroStore.startSession(session.id)
      currentPhase.value = 'focusing'
      // 开始窗口监控
      startWindowMonitoring()
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
      showResumeModal.value = true
    } else {
      await pomodoroStore.pauseSession(currentSession.value.id)
      // 暂停时停止窗口监控
      stopWindowMonitoring()
      message.info('已暂停')
    }
  } catch (error) {
    message.error('操作失败')
  }
}

async function handleConfirmResume() {
  if (!currentSession.value?.id) return
  try {
    await pomodoroStore.resumeSession(currentSession.value.id)
    showResumeModal.value = false
    // 恢复时重新开始窗口监控
    startWindowMonitoring()
    message.success('继续专注！')
  } catch (error) {
    message.error('恢复失败')
  }
}

function handleCancelResume() {
  showResumeModal.value = false
}

async function handleStop() {
  if (!currentSession.value?.id) {
    progressValue.value = getCurrentTaskProgress()
    currentPhase.value = 'report'
    return
  }

  try {
    // 停止窗口监控
    stopWindowMonitoring()

    // 暂停当前会话
    await pomodoroStore.pauseSession(currentSession.value.id)

    // 进入报告阶段
    progressValue.value = getCurrentTaskProgress()
    currentPhase.value = 'report'
    message.info('专注已停止，请提交反馈')
  } catch (error) {
    console.error('停止会话失败:', error)
    progressValue.value = getCurrentTaskProgress()
    currentPhase.value = 'report'
  }
}

async function handleComplete() {
  if (!currentSession.value?.id) return

  // 保存任务ID，因为完成后 currentSession 会变成 null
  const sessionTaskId = currentSession.value?.taskId
  const sessionId = currentSession.value.id
  const focusSeconds = currentSession.value.actualFocusSeconds

  try {
    // 先同步最终的专注时间，确保统计数据准确
    await pomodoroStore.updateFocusTime(sessionId, focusSeconds)

    // 完成番茄钟会话
    await pomodoroStore.completeSession(sessionId, {
      feedback: feedback.value,
      progressUpdate: feedback.value
    })

    // 如果有任务关联，更新进度
    if (sessionTaskId) {
      try {
        // 更新任务进度（确保是数字类型）
        const progressNum = Number(progressValue.value)
        await taskApi.updateTaskProgress(sessionTaskId, progressNum)

        // 如果需要添加里程碑
        if (addMilestone.value) {
          const title = milestoneTitle.value.trim() || feedback.value.slice(0, 50) || '番茄钟专注完成'
          await taskApi.createTaskMilestone(
            sessionTaskId,
            title,
            feedback.value || undefined,
            progressNum
          )
        }
      } catch (progressError) {
        console.error('更新进度/里程碑失败:', progressError)
      }
    }

    // 如果用户勾选了任务完成 或 进度达到100%，则标记任务为已完成
    const progressNumForCheck = Number(progressValue.value)
    const shouldCompleteTask = (taskCompleted.value || progressNumForCheck >= 100) && sessionTaskId
    if (shouldCompleteTask) {
      try {
        await taskStore.completeTask(sessionTaskId)
        if (progressNumForCheck >= 100) {
          message.success('🎉 任务100%完成！已自动标记为已完成！')
        } else {
          message.success('🎉 专注完成，任务已标记为已完成！')
        }
      } catch (taskError) {
        console.error('标记任务完成失败:', taskError)
        message.success('专注完成！（任务状态更新失败）')
      }
    } else {
      message.success('专注完成！')
    }

    // 刷新任务列表以更新进度显示
    if (sessionTaskId) {
      await taskStore.loadTasks()
    }

    // 重置状态
    taskCompleted.value = false
    progressValue.value = 0
    addMilestone.value = false
    milestoneTitle.value = ''
    currentPhase.value = 'stats'
  } catch (error) {
    message.error('提交失败')
  }
}

function handleBackToPrep() {
  currentPhase.value = 'prep'
  feedback.value = ''
  taskCompleted.value = false
  progressValue.value = 0
  addMilestone.value = false
  milestoneTitle.value = ''
  selectedTaskId.value = null
}

// 查看完整周报
function handleViewWeeklyReport() {
  router.push('/report-center')
}

function handleAddApp() {
  if (!appInputName.value.trim()) return

  const name = appInputName.value.trim()

  // 创建新的应用条目
  const processName = name.toLowerCase().replace(/\s+/g, '') + '.exe'

  // 添加到本地列表
  runningAppsList.value.push({
    name: name,
    processName: processName
  })

  // 自动选中新添加的应用
  selectedAppProcessNames.value.push(processName.toLowerCase())

  message.success('应用已添加到白名单')
  appInputName.value = ''
}

// Lifecycle
onMounted(async () => {
  // 立即获取当前窗口信息，避免显示"检测中..."
  fetchCurrentWindow()

  await taskStore.loadTasks()

  // 加载运行中的应用作为白名单选项
  await loadRunningApps()

  await pomodoroStore.loadActiveSession()
  await pomodoroStore.loadTodayStats()
  await pomodoroStore.loadTodaySessions()

  // From route
  const taskId = route.query.taskId
  if (taskId) {
    selectedTaskId.value = Number(taskId)
  }

  // Resume active session
  if (currentSession.value) {
    if (currentSession.value.status === 'focusing' || currentSession.value.status === 'paused') {
      currentPhase.value = 'focusing'
      // 如果正在专注，启动窗口监控
      if (currentSession.value.status === 'focusing') {
        startWindowMonitoring()
      }
    }
  }
})

onUnmounted(() => {
  // 清理窗口监控定时器
  stopWindowMonitoring()
})
</script>

<style scoped>
.pomodoro-view {
  width: 100%;
  min-height: 100vh;
  background: linear-gradient(135deg, #020617 0%, #0f172a 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
}

.scene-container {
  width: 100%;
  max-width: 600px;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

/* ===== PREP SCENE ===== */
.prep-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.scene-header {
  margin-bottom: 8px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #6366f1;
  box-shadow: 0 0 12px rgba(99, 102, 241, 0.6);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.status-label {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.15em;
  color: #64748b;
  text-transform: uppercase;
}

.scene-title {
  font-size: 32px;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0;
  letter-spacing: -0.02em;
}

/* Task Card */
.task-card {
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 20px;
  overflow: hidden;
  backdrop-filter: blur(12px);
}

/* Task Selection Area */
.task-selection-area {
  padding: 20px;
  border-bottom: 1px solid rgba(51, 65, 85, 0.3);
}

.task-selection-label {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
  font-size: 14px;
  font-weight: 600;
  color: #94a3b8;
}

.task-category-tag {
  background: rgba(99, 102, 241, 0.15);
  color: #818cf8;
  border: 1px solid rgba(99, 102, 241, 0.3);
  font-size: 11px;
  font-weight: 600;
  margin-left: auto;
}

.task-input-wrapper {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.task-native-select {
  width: 100%;
  height: 48px;
  padding: 0 16px;
  background: rgba(30, 41, 59, 0.6);
  border: 1px solid rgba(71, 85, 105, 0.5);
  border-radius: 12px;
  color: #f1f5f9;
  font-size: 15px;
  cursor: pointer;
  outline: none;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24'%3E%3Cpath fill='%2394a3b8' d='M7 10l5 5 5-5z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
}

.task-native-select:hover {
  border-color: #6366f1;
}

.task-native-select:focus {
  border-color: #6366f1;
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);
}

.task-native-select option {
  background: #1e293b;
  color: #f1f5f9;
  padding: 12px;
}

.task-or-divider {
  text-align: center;
  color: #64748b;
  font-size: 12px;
  position: relative;
}

.task-or-divider::before,
.task-or-divider::after {
  content: '';
  position: absolute;
  top: 50%;
  width: 40%;
  height: 1px;
  background: rgba(71, 85, 105, 0.5);
}

.task-or-divider::before {
  left: 0;
}

.task-or-divider::after {
  right: 0;
}

.new-task-input-row {
  display: flex;
  gap: 10px;
}

.new-task-input {
  flex: 1;
  height: 48px;
  padding: 0 16px;
  background: rgba(30, 41, 59, 0.6);
  border: 1px solid rgba(71, 85, 105, 0.5);
  border-radius: 12px;
  color: #f1f5f9;
  font-size: 15px;
  outline: none;
}

.new-task-input::placeholder {
  color: #64748b;
}

.new-task-input:focus {
  border-color: #6366f1;
}

.create-task-btn {
  height: 48px;
  padding: 0 24px;
  background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%);
  border: none;
  border-radius: 12px;
  color: white;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.create-task-btn:hover:not(:disabled) {
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
}

.create-task-btn:disabled {
  background: #334155;
  opacity: 0.5;
  cursor: not-allowed;
}

.selected-task-display {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid rgba(16, 185, 129, 0.3);
  border-radius: 10px;
  color: #10b981;
  font-size: 14px;
  font-weight: 500;
}

.selected-task-display span {
  flex: 1;
}

.clear-task-btn {
  width: 24px;
  height: 24px;
  background: rgba(248, 113, 113, 0.2);
  border: none;
  border-radius: 6px;
  color: #f87171;
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.clear-task-btn:hover {
  background: rgba(248, 113, 113, 0.3);
}

.task-status-tag {
  background: rgba(99, 102, 241, 0.15);
  color: #818cf8;
  border: 1px solid rgba(99, 102, 241, 0.3);
  font-size: 11px;
  font-weight: 600;
}

/* AI Suggestions */
.ai-suggestions {
  padding: 20px;
  background: rgba(2, 6, 23, 0.4);
}

.ai-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}

.sparkle-icon {
  flex-shrink: 0;
}

.ai-label {
  font-size: 14px;
  font-weight: 600;
  color: #cbd5e1;
}

.suggestion-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.suggestion-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(51, 65, 85, 0.4);
  border-radius: 12px;
  font-size: 14px;
  color: #94a3b8;
  line-height: 1.5;
}

.suggestion-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  margin-top: 6px;
  flex-shrink: 0;
}

.suggestion-dot.blue {
  background: #6366f1;
}

.suggestion-dot.purple {
  background: #a78bfa;
}

.suggestion-item .highlight {
  color: #f1f5f9;
  font-weight: 500;
}

/* 结构化AI建议样式 */
.suggestion-structured {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.suggestion-goal {
  padding: 12px 16px;
  background: rgba(99, 102, 241, 0.15);
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: 10px;
}

.goal-label {
  font-size: 12px;
  font-weight: 600;
  color: #818cf8;
  margin-right: 6px;
}

.goal-text {
  font-size: 14px;
  color: #e2e8f0;
  line-height: 1.5;
}

.suggestion-steps {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.suggestion-step {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 14px;
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(51, 65, 85, 0.4);
  border-radius: 10px;
}

.step-number {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  background: rgba(99, 102, 241, 0.2);
  color: #818cf8;
  border-radius: 50%;
  font-size: 12px;
  font-weight: 600;
  flex-shrink: 0;
}

.step-text {
  font-size: 13px;
  color: #94a3b8;
  line-height: 1.5;
  flex: 1;
}

.suggestion-footer {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  padding-top: 8px;
}

.pomodoro-estimate {
  font-size: 12px;
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.1);
  padding: 4px 10px;
  border-radius: 6px;
}

.suggestion-tip {
  font-size: 12px;
  color: #94a3b8;
  font-style: italic;
  flex: 1;
  line-height: 1.4;
}

/* Settings Row */
.settings-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.setting-card {
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.4);
  border-radius: 16px;
  padding: 16px;
}

.setting-label {
  display: block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: #64748b;
  margin-bottom: 12px;
}

.app-tags {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.app-tag {
  height: 32px;
  padding: 0 12px;
  background: rgba(51, 65, 85, 0.5);
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
  color: #cbd5e1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.app-tag.add {
  background: transparent;
  border: 1px dashed rgba(100, 116, 139, 0.5);
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;
}

.app-tag.add:hover {
  border-color: #6366f1;
  color: #6366f1;
}

.duration-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.duration-btn {
  padding: 8px 12px;
  background: rgba(51, 65, 85, 0.4);
  border: 1px solid rgba(71, 85, 105, 0.4);
  border-radius: 8px;
  color: #94a3b8;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.duration-btn:hover {
  background: rgba(99, 102, 241, 0.1);
  border-color: rgba(99, 102, 241, 0.3);
  color: #c7d2fe;
}

.duration-btn.active {
  background: rgba(99, 102, 241, 0.2);
  border-color: #6366f1;
  color: #a5b4fc;
}

/* Start Button */
.start-button {
  width: 100%;
  height: 56px;
  background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%);
  border: none;
  border-radius: 16px;
  color: white;
  font-size: 16px;
  font-weight: 700;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  transition: all 0.2s;
  box-shadow: 0 8px 24px rgba(99, 102, 241, 0.3);
  margin-top: 8px;
}

.start-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 12px 32px rgba(99, 102, 241, 0.4);
}

.start-button:active:not(:disabled) {
  transform: translateY(0);
}

.start-button:disabled {
  background: #334155;
  box-shadow: none;
  cursor: not-allowed;
  opacity: 0.6;
}

/* ===== FOCUSING SCENE ===== */
.focusing-scene {
  max-width: 500px;
}

.focusing-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 40px;
}

.timer-container {
  margin-bottom: 20px;
}

.timer-ring {
  position: relative;
  width: 320px;
  height: 320px;
}

.progress-svg {
  transform: rotate(-90deg);
}

.progress-bar {
  transition: stroke-dashoffset 1s linear;
  filter: drop-shadow(0 0 12px rgba(99, 102, 241, 0.5));
}

.timer-content {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.time-display {
  font-size: 72px;
  font-weight: 700;
  font-family: 'SF Mono', 'Consolas', monospace;
  color: #f1f5f9;
  letter-spacing: -0.02em;
  text-shadow: 0 4px 24px rgba(0, 0, 0, 0.3);
}

.active-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 12px;
  padding: 6px 14px;
  background: rgba(30, 41, 59, 0.6);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  color: #94a3b8;
  letter-spacing: 0.05em;
}

.focus-info {
  text-align: center;
}

.focus-task-title {
  font-size: 22px;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0 0 12px 0;
}

.focus-status {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}

.status-dots {
  display: flex;
  gap: 4px;
}

.status-dots .dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: #334155;
}

.status-dots .dot.active {
  background: #6366f1;
}

.status-text {
  font-size: 11px;
  font-style: italic;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #64748b;
}

.focus-controls {
  display: flex;
  gap: 24px;
}

.control-btn {
  width: 56px;
  height: 56px;
  background: rgba(15, 23, 42, 0.8);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 16px;
  color: #94a3b8;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.control-btn:hover {
  color: #f1f5f9;
  border-color: #6366f1;
  background: rgba(99, 102, 241, 0.1);
}

.control-btn.stop:hover {
  color: #f87171;
  border-color: #f87171;
  background: rgba(248, 113, 113, 0.1);
}

/* ===== REPORT SCENE ===== */
.report-scene {
  max-width: 640px;
}

.report-card {
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 24px;
  padding: 40px;
  backdrop-filter: blur(12px);
}

.report-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 32px;
}

.report-icon {
  width: 56px;
  height: 56px;
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.report-title-section {
  flex: 1;
}

.report-title {
  font-size: 22px;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0 0 4px 0;
}

.report-meta {
  font-size: 13px;
  color: #64748b;
  margin: 0;
}

.session-tag {
  padding: 8px 14px;
  background: rgba(51, 65, 85, 0.5);
  border: 1px solid rgba(71, 85, 105, 0.5);
  border-radius: 20px;
  font-size: 10px;
  font-weight: 800;
  color: #94a3b8;
  letter-spacing: 0.05em;
}

/* Progress Update Section */
.progress-update-section {
  margin-bottom: 20px;
  padding: 16px;
  background: rgba(99, 102, 241, 0.1);
  border-radius: 16px;
  border: 1px solid rgba(99, 102, 241, 0.2);
}

.section-label {
  display: block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: #6366f1;
  margin-bottom: 12px;
}

.progress-slider-wrapper {
  display: flex;
  align-items: center;
  gap: 16px;
}

.progress-slider {
  flex: 1;
  height: 8px;
  -webkit-appearance: none;
  appearance: none;
  background: rgba(51, 65, 85, 0.5);
  border-radius: 4px;
  outline: none;
}

.progress-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 20px;
  height: 20px;
  background: #6366f1;
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(99, 102, 241, 0.4);
}

.progress-slider::-moz-range-thumb {
  width: 20px;
  height: 20px;
  background: #6366f1;
  border-radius: 50%;
  cursor: pointer;
  border: none;
  box-shadow: 0 2px 8px rgba(99, 102, 241, 0.4);
}

.progress-display {
  font-size: 20px;
  font-weight: 700;
  color: #6366f1;
  min-width: 60px;
  text-align: right;
}

/* Milestone Section */
.milestone-section {
  margin-bottom: 20px;
}

.milestone-checkbox.checked {
  background: #ec4899 !important;
  border-color: #ec4899 !important;
}

.milestone-input {
  width: 100%;
  margin-top: 12px;
  padding: 12px 16px;
  background: rgba(2, 6, 23, 0.5);
  border: 1px solid rgba(236, 72, 153, 0.3);
  border-radius: 12px;
  color: #e2e8f0;
  font-size: 14px;
  outline: none;
  font-family: inherit;
}

.milestone-input::placeholder {
  color: #64748b;
}

.milestone-input:focus {
  border-color: #ec4899;
}

.feedback-section {
  margin-bottom: 24px;
}

.feedback-label {
  display: block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: #64748b;
  margin-bottom: 12px;
}

.feedback-textarea {
  width: 100%;
  height: 140px;
  padding: 16px;
  background: rgba(2, 6, 23, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 16px;
  color: #e2e8f0;
  font-size: 15px;
  line-height: 1.6;
  resize: none;
  outline: none;
  font-family: inherit;
}

.feedback-textarea::placeholder {
  color: #475569;
}

.feedback-textarea:focus {
  border-color: #6366f1;
}

.submit-button {
  width: 100%;
  height: 56px;
  background: #f1f5f9;
  border: none;
  border-radius: 16px;
  color: #0f172a;
  font-size: 16px;
  font-weight: 700;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  transition: all 0.2s;
}

.submit-button:hover:not(:disabled) {
  background: #e2e8f0;
}

.submit-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ===== STATS SCENE ===== */
.stats-scene {
  max-width: 800px;
}

.stats-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.stats-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.stats-title-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stats-title {
  font-size: 26px;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0;
}

.date-tag {
  padding: 8px 14px;
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  color: #94a3b8;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.stat-item {
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 20px;
  padding: 20px;
}

.stat-label {
  display: block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: #64748b;
  margin-bottom: 8px;
}

.stat-value {
  font-size: 28px;
  font-weight: 800;
}

.stat-value small {
  font-size: 12px;
  font-weight: 400;
  opacity: 0.5;
  margin-left: 2px;
}

.stat-value.blue { color: #6366f1; }
.stat-value.green { color: #10b981; }
.stat-value.white { color: #f1f5f9; }
.stat-value.red { color: #f87171; }

.charts-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.chart-card {
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 24px;
  padding: 24px;
}

.chart-title {
  font-size: 14px;
  font-weight: 600;
  color: #cbd5e1;
  margin: 0 0 20px 0;
}

.usage-list {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.usage-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.usage-header {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 700;
}

.app-name {
  color: #94a3b8;
}

.app-time {
  color: #6366f1;
  font-family: monospace;
}

.usage-bar {
  height: 6px;
  background: rgba(51, 65, 85, 0.5);
  border-radius: 3px;
  overflow: hidden;
}

.usage-fill {
  height: 100%;
  background: linear-gradient(90deg, #6366f1, #818cf8);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.usage-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 30px;
  color: #6b7280;
  text-align: center;
}

.usage-empty p {
  margin: 0;
  font-size: 12px;
  color: #9ca3af;
}

.ai-card {
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.1) 0%, rgba(139, 92, 246, 0.05) 100%);
  border-color: rgba(99, 102, 241, 0.2);
}

.ai-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.1em;
  color: #f1f5f9;
}

.ai-insight-text {
  font-size: 13px;
  line-height: 1.7;
  color: #cbd5e1;
  margin: 0 0 20px 0;
}

.view-report-btn {
  background: none;
  border: none;
  color: #6366f1;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0;
  transition: color 0.2s;
}

.view-report-btn:hover {
  color: #818cf8;
}

.exit-button {
  width: 100%;
  padding: 16px;
  background: transparent;
  border: none;
  color: #475569;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.2em;
  cursor: pointer;
  transition: color 0.2s;
  margin-top: 16px;
}

.exit-button:hover {
  color: #94a3b8;
}

/* ===== MODAL ===== */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.resume-modal {
  width: 100%;
  max-width: 380px;
  background: rgba(15, 23, 42, 0.95);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 24px;
  padding: 32px;
  text-align: center;
  backdrop-filter: blur(16px);
}

.modal-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto 16px;
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-title {
  font-size: 22px;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0 0 8px 0;
}

.modal-subtitle {
  font-size: 14px;
  color: #64748b;
  margin: 0 0 24px 0;
}

.modal-task {
  background: rgba(30, 41, 59, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 12px;
  padding: 14px 16px;
  margin-bottom: 24px;
  text-align: left;
}

.modal-task .task-label {
  display: block;
  font-size: 10px;
  font-weight: 600;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 4px;
}

.modal-task .task-name {
  font-size: 15px;
  font-weight: 600;
  color: #e2e8f0;
}

.modal-actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.modal-btn {
  width: 100%;
  height: 48px;
  border-radius: 12px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  transition: all 0.2s;
}

.modal-btn.primary {
  background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%);
  border: none;
  color: white;
}

.modal-btn.primary:hover {
  box-shadow: 0 8px 20px rgba(99, 102, 241, 0.3);
}

.modal-btn.secondary {
  background: transparent;
  border: 1px solid rgba(51, 65, 85, 0.5);
  color: #94a3b8;
}

.modal-btn.secondary:hover {
  border-color: rgba(99, 102, 241, 0.3);
  color: #e2e8f0;
}

/* ===== APP SELECTOR MODAL ===== */
.app-selector-modal {
  width: 100%;
  max-width: 480px;
  background: rgba(15, 23, 42, 0.98);
  border: 1px solid rgba(71, 85, 105, 0.5);
  border-radius: 20px;
  padding: 24px;
  backdrop-filter: blur(16px);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
}

.modal-header h3 {
  font-size: 18px;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0;
}

.close-btn {
  width: 32px;
  height: 32px;
  background: rgba(51, 65, 85, 0.5);
  border: none;
  border-radius: 8px;
  color: #94a3b8;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(248, 113, 113, 0.2);
  color: #f87171;
}

.app-list {
  margin-bottom: 20px;
}

.app-list-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: #64748b;
  margin-bottom: 12px;
}

.app-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

.app-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  background: rgba(30, 41, 59, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.app-item:hover {
  background: rgba(51, 65, 85, 0.5);
  border-color: rgba(99, 102, 241, 0.3);
}

.app-item.selected {
  background: rgba(99, 102, 241, 0.15);
  border-color: #6366f1;
}

.loading-apps,
.no-apps {
  padding: 40px 20px;
  text-align: center;
  color: #64748b;
  font-size: 14px;
}

.app-icon-wrapper {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  overflow: hidden;
  flex-shrink: 0;
}

.app-icon-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.app-icon-text {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: rgba(51, 65, 85, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 800;
  color: #e2e8f0;
  flex-shrink: 0;
}

.app-item.selected .app-icon-text {
  background: rgba(99, 102, 241, 0.3);
  color: #a5b4fc;
}

.app-item .app-name {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  color: #cbd5e1;
}

.check-icon {
  position: absolute;
  top: 8px;
  right: 8px;
}

.add-app-section {
  margin-bottom: 20px;
}

.add-app-row {
  display: flex;
  gap: 10px;
}

.add-app-input {
  flex: 1;
  height: 44px;
  padding: 0 14px;
  background: rgba(30, 41, 59, 0.6);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 10px;
  color: #e2e8f0;
  font-size: 14px;
  outline: none;
}

.add-app-input::placeholder {
  color: #64748b;
}

.add-app-input:focus {
  border-color: #6366f1;
}

.add-app-btn {
  height: 44px;
  padding: 0 20px;
  background: rgba(99, 102, 241, 0.2);
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: 10px;
  color: #a5b4fc;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.add-app-btn:hover:not(:disabled) {
  background: rgba(99, 102, 241, 0.3);
}

.add-app-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.modal-footer {
  padding-top: 16px;
  border-top: 1px solid rgba(51, 65, 85, 0.3);
}

.confirm-btn {
  width: 100%;
  height: 48px;
  background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%);
  border: none;
  border-radius: 12px;
  color: white;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.confirm-btn:hover {
  box-shadow: 0 8px 20px rgba(99, 102, 241, 0.3);
}

/* ===== DISTRACTION MODAL ===== */
.distraction-overlay {
  z-index: 2000;
}

.distraction-modal {
  width: 100%;
  max-width: 420px;
  background: rgba(15, 23, 42, 0.98);
  border: 1px solid rgba(127, 29, 29, 0.5);
  border-radius: 24px;
  padding: 40px 32px;
  text-align: center;
  backdrop-filter: blur(16px);
  animation: shake 0.5s ease-in-out;
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  10%, 30%, 50%, 70%, 90% { transform: translateX(-5px); }
  20%, 40%, 60%, 80% { transform: translateX(5px); }
}

.distraction-icon {
  width: 72px;
  height: 72px;
  margin: 0 auto 20px;
  background: rgba(127, 29, 29, 0.3);
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.distraction-title {
  font-size: 26px;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0 0 16px 0;
}

.distraction-desc {
  font-size: 16px;
  color: #94a3b8;
  margin: 0 0 8px 0;
  line-height: 1.5;
}

.distraction-desc strong {
  color: #f1f5f9;
  text-decoration: underline;
  text-decoration-color: #ef4444;
  text-underline-offset: 3px;
}

.distraction-hint {
  font-size: 14px;
  color: #64748b;
  margin: 0 0 32px 0;
}

.distraction-actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.distraction-btn {
  width: 100%;
  height: 52px;
  border-radius: 14px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.distraction-btn.primary {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  border: none;
  color: white;
}

.distraction-btn.primary:hover {
  box-shadow: 0 8px 24px rgba(59, 130, 246, 0.4);
  transform: translateY(-2px);
}

.distraction-btn.secondary {
  background: transparent;
  border: 1px solid rgba(71, 85, 105, 0.5);
  color: #94a3b8;
}

.distraction-btn.secondary:hover {
  border-color: rgba(248, 113, 113, 0.5);
  color: #f87171;
  background: rgba(248, 113, 113, 0.1);
}

/* ===== SCENE SWITCHER ===== */
.scene-switcher {
  position: fixed;
  bottom: 20px;
  right: 20px;
  background: rgba(15, 23, 42, 0.9);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 14px;
  padding: 6px;
  display: flex;
  gap: 4px;
  backdrop-filter: blur(12px);
  z-index: 100;
}

.scene-btn {
  padding: 8px 14px;
  background: transparent;
  border: none;
  border-radius: 10px;
  color: #64748b;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.08em;
  cursor: pointer;
  transition: all 0.2s;
}

.scene-btn:hover {
  background: rgba(51, 65, 85, 0.5);
  color: #94a3b8;
}

.scene-btn.active {
  background: #6366f1;
  color: white;
}

/* Responsive */
@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .charts-row {
    grid-template-columns: 1fr;
  }

  .settings-row {
    grid-template-columns: 1fr;
  }

  .app-grid {
    grid-template-columns: 1fr;
  }

  .scene-switcher {
    bottom: 10px;
    right: 10px;
    left: 10px;
    justify-content: center;
  }
}
</style>
