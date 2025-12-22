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
                v-if="taskOptions.active.length > 0 || taskOptions.todo.length > 0"
                v-model="selectedTaskId"
                class="task-native-select"
                @change="handleTaskSelectChange"
              >
                <option :value="null" disabled>-- 选择现有任务 --</option>

                <optgroup v-if="taskOptions.active.length > 0" label="进行中">
                  <option v-for="task in taskOptions.active" :key="task.value" :value="task.value">
                    {{ task.label }}
                  </option>
                </optgroup>

                <optgroup v-if="taskOptions.todo.length > 0" label="待办">
                  <option v-for="task in taskOptions.todo" :key="task.value" :value="task.value">
                    {{ task.label }}
                  </option>
                </optgroup>
              </select>

              <div class="task-or-divider" v-if="taskOptions.active.length > 0 || taskOptions.todo.length > 0">或</div>

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
                :key="app.processName"
                class="app-tag"
                @click="removeWhitelistByProcess(app.processName)"
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

        <!-- Task Details Panel (Expandable) -->
        <div class="task-details-panel">
          <button class="panel-toggle-btn" @click="toggleTaskDetailsPanel">
            <svg viewBox="0 0 24 24" width="16" height="16" class="toggle-icon" :class="{ expanded: showTaskDetails }">
              <path fill="currentColor" d="M7 10l5 5 5-5z"/>
            </svg>
            <span>任务详情</span>
          </button>

          <transition name="panel-slide">
            <div v-if="showTaskDetails" class="task-details-content">
              <!-- AI 建议步骤 -->
              <div v-if="aiSuggestionData?.subTasks?.length" class="detail-section">
                <div class="section-header">
                  <svg viewBox="0 0 24 24" width="16" height="16">
                    <path fill="#6366f1" d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                  </svg>
                  <span>AI 建议步骤</span>
                </div>
                <div class="steps-list">
                  <div
                    v-for="(step, index) in aiSuggestionData.subTasks"
                    :key="index"
                    class="step-item"
                  >
                    <span class="step-number">{{ index + 1 }}</span>
                    <span class="step-text">{{ step }}</span>
                  </div>
                </div>
              </div>

              <!-- 任务需求描述 -->
              <div v-if="currentSession?.task?.notes" class="detail-section">
                <div class="section-header">
                  <svg viewBox="0 0 24 24" width="16" height="16">
                    <path fill="#6366f1" d="M14 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8l-6-6zm4 18H6V4h7v5h5v11z"/>
                  </svg>
                  <span>任务需求</span>
                </div>
                <div class="task-notes">{{ currentSession.task.notes }}</div>
              </div>

              <!-- 任务进度 -->
              <div v-if="currentSession?.task?.progress !== undefined" class="detail-section">
                <div class="section-header">
                  <svg viewBox="0 0 24 24" width="16" height="16">
                    <path fill="#6366f1" d="M19 3h-4.18C14.4 1.84 13.3 1 12 1c-1.3 0-2.4.84-2.82 2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-7 0c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zm0 4c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm6 12H6v-1.4c0-2 4-3.1 6-3.1s6 1.1 6 3.1V19z"/>
                  </svg>
                  <span>任务进度</span>
                </div>
                <div class="progress-bar-wrapper">
                  <div class="progress-bar-bg">
                    <div
                      class="progress-bar-fill"
                      :style="{ width: `${currentSession.task.progress}%` }"
                    ></div>
                  </div>
                  <span class="progress-text">{{ currentSession.task.progress }}%</span>
                </div>
              </div>

              <!-- 里程碑记录 -->
              <div v-if="taskMilestones.length > 0" class="detail-section">
                <div class="section-header">
                  <svg viewBox="0 0 24 24" width="16" height="16">
                    <path fill="#6366f1" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
                  </svg>
                  <span>最近里程碑</span>
                </div>
                <div class="milestones-list">
                  <div
                    v-for="milestone in taskMilestones.slice(0, 3)"
                    :key="milestone.id"
                    class="milestone-item"
                  >
                    <div class="milestone-icon"></div>
                    <div class="milestone-content">
                      <div class="milestone-title">{{ milestone.title }}</div>
                      <div class="milestone-meta">
                        <span v-if="milestone.progressSnapshot !== undefined">
                          进度: {{ milestone.progressSnapshot }}%
                        </span>
                        <span v-if="milestone.createdAt" class="milestone-date">
                          {{ formatMilestoneDate(milestone.createdAt) }}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 空状态提示 -->
              <div v-if="!hasAnyTaskDetails" class="empty-state">
                <svg viewBox="0 0 24 24" width="32" height="32">
                  <path fill="#64748b" d="M11 7h2v2h-2zm0 4h2v6h-2zm1-9C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8z"/>
                </svg>
                <p>暂无任务详情</p>
              </div>
            </div>
          </transition>
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

        <!-- App Usage Section -->
        <div class="app-usage-section">
          <div class="section-label">本次专注应用分布</div>
          <div v-if="sessionAppUsage.length > 0" class="session-usage-list">
            <div v-for="app in sessionAppUsage" :key="app.appName" class="session-usage-item">
              <div class="usage-header">
                <span class="app-name">{{ app.appName }}</span>
                <span class="app-time">{{ app.minutes }} MIN</span>
              </div>
              <div class="usage-bar">
                <div class="usage-fill" :style="{ width: app.percentage + '%' }"></div>
              </div>
            </div>
          </div>
          <div v-else class="usage-empty">
            <svg viewBox="0 0 24 24" width="24" height="24">
              <path fill="#9ca3af" d="M13 9h-2V7h2m0 10h-2v-6h2m-1-9A10 10 0 0 0 2 12a10 10 0 0 0 10 10 10 10 0 0 0 10-10A10 10 0 0 0 12 2z"/>
            </svg>
            <span>暂无应用使用记录</span>
          </div>
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

          <!-- Session Analysis -->
          <div class="chart-card ai-card">
            <div class="ai-card-header">
              <svg viewBox="0 0 24 24" width="20" height="20">
                <path fill="#6366f1" d="M12 2L9.5 9.5L2 12l7.5 2.5L12 22l2.5-7.5L22 12l-7.5-2.5L12 2z"/>
              </svg>
              <span>本次专注分析</span>
            </div>

            <!-- 加载状态 -->
            <template v-if="analysisLoading">
              <p class="ai-insight-text">AI 正在分析最近的专注会话...</p>
            </template>

            <!-- 有分析结果 -->
            <template v-else-if="lastSessionAnalysis">
              <div class="session-analysis-wrapper">
                <div class="session-analysis">
                  <!-- 效率评分 -->
                  <div v-if="lastSessionAnalysis.efficiencyScore > 0" class="efficiency-score-section">
                    <div class="score-circle" :class="getScoreClass(lastSessionAnalysis.efficiencyScore)">
                      <span class="score-value">{{ lastSessionAnalysis.efficiencyScore }}</span>
                      <span class="score-label">分</span>
                    </div>
                    <p class="efficiency-comment">{{ lastSessionAnalysis.efficiencyComment }}</p>
                  </div>

                  <div class="analysis-section">
                    <h4 class="analysis-label">任务相关性</h4>
                    <p class="analysis-text">{{ lastSessionAnalysis.relevanceAnalysis }}</p>
                  </div>

                  <div v-if="lastSessionAnalysis.improvements.length > 0" class="analysis-section">
                    <h4 class="analysis-label">改进建议</h4>
                    <ul class="analysis-list">
                      <li v-for="(item, idx) in lastSessionAnalysis.improvements" :key="idx">
                        {{ item }}
                      </li>
                    </ul>
                  </div>

                  <div class="analysis-section">
                    <h4 class="analysis-label">下次行动</h4>
                    <p class="analysis-text highlight">{{ lastSessionAnalysis.nextAction }}</p>
                  </div>
                </div>
              </div>
            </template>

            <!-- 无分析结果 -->
            <template v-else>
              <p class="ai-insight-text">完成番茄钟后，AI 将为你分析本次专注表现。</p>
            </template>

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
              :key="app.processName"
              class="app-item"
              :class="{ selected: isAppSelectedByProcess(app.processName) }"
              @click.stop="toggleAppByProcess(app.processName)"
            >
              <div class="app-icon-text">{{ app.shortName }}</div>
              <span class="app-name">{{ app.name }}</span>
              <div v-if="isAppSelectedByProcess(app.processName)" class="check-icon">
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
        <p class="distraction-hint">请选择接下来的操作：</p>

        <div class="distraction-actions">
          <button class="distraction-btn primary" @click="handleReturnToWork">
            立即重返工作
          </button>
          <button class="distraction-btn whitelist" @click="handleAddToWhitelist">
            <svg viewBox="0 0 24 24" width="16" height="16" style="margin-right: 6px;">
              <path fill="currentColor" d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
            </svg>
            添加到白名单
          </button>
          <button class="distraction-btn secondary" @click="handlePauseTimer">
            暂停计时
          </button>
          <button class="distraction-btn danger" @click="handleStopTimer">
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
import { aiTaskBreakdown, aiFocusAnalysis, aiDailyReview, aiAnalyzeSession, getActiveWindowInfo, getRunningApps, getSessionAppUsage, updateAiAnalysis, type RunningApp } from '@/api/pomodoroApi'
import { taskApi } from '@/api/taskApi'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { PomodoroPhase, FocusApp, AppUsageStats } from '@/types/pomodoro'
import type { TaskMilestone } from '@/types/task'

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
  return {
    active: taskStore.activeTasks.map(t => ({
      label: t.title,
      value: t.id,
      status: 'active'
    })),
    todo: taskStore.todoTasks.map(t => ({
      label: t.title,
      value: t.id,
      status: 'todo'
    }))
  }
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

// 监听任务选择变化，获取AI建议并自动更新任务状态
watch(selectedTaskId, async (newTaskId) => {
  if (newTaskId && selectedTask.value) {
    // 如果选中的任务状态是待办（todo），自动更新为进行中（active）
    if (selectedTask.value.status === 'todo') {
      try {
        await taskStore.startTask(newTaskId)
        message.success(`任务「${selectedTask.value.title}」已开始`)
      } catch (error) {
        console.error('自动开始任务失败:', error)
        message.error('任务状态更新失败')
      }
    }
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
      selectedTask.value.id,
      selectedTask.value.title,
      selectedTask.value.description || undefined,
      selectedTask.value.progress
    )
    console.log('[AI建议] 原始返回:', result)

    // 尝试解析JSON结果
    let parsed: any = null
    try {
      // 1. 先尝试直接解析
      parsed = JSON.parse(result)
    } catch {
      // 2. 尝试从markdown代码块中提取JSON
      const jsonMatch = result.match(/```(?:json)?\s*([\s\S]*?)```/)
      if (jsonMatch && jsonMatch[1]) {
        try {
          parsed = JSON.parse(jsonMatch[1].trim())
          console.log('[AI建议] 从markdown提取JSON成功')
        } catch (e) {
          console.log('[AI建议] markdown JSON解析失败:', e)
        }
      }

      // 3. 尝试查找JSON对象边界
      if (!parsed) {
        const jsonStart = result.indexOf('{')
        const jsonEnd = result.lastIndexOf('}')
        if (jsonStart !== -1 && jsonEnd > jsonStart) {
          try {
            parsed = JSON.parse(result.substring(jsonStart, jsonEnd + 1))
            console.log('[AI建议] 从文本提取JSON成功')
          } catch (e) {
            console.log('[AI建议] 文本JSON解析失败:', e)
          }
        }
      }
    }

    // 处理解析结果
    if (parsed && typeof parsed === 'object') {
      // 处理结构化的AI返回
      if (parsed.sub_tasks || parsed.subTasks || parsed.suggested_goal || parsed.suggestedGoal) {
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
        console.log('[AI建议] 解析成功:', aiSuggestionData.value)
      } else if (parsed.suggestions && Array.isArray(parsed.suggestions)) {
        aiSuggestions.value = parsed.suggestions.slice(0, 3)
      } else {
        // JSON格式但不符合预期结构，使用原始文本
        const lines = result.split('\n').filter((l: string) => l.trim() && !l.startsWith('```'))
        aiSuggestions.value = lines.slice(0, 3)
      }
    } else {
      // 如果完全无法解析为JSON，直接使用文本（过滤掉markdown标记）
      console.log('[AI建议] 无法解析为JSON，使用原始文本')
      const lines = result.split('\n').filter((l: string) => {
        const trimmed = l.trim()
        return trimmed && !trimmed.startsWith('```') && trimmed !== '{' && trimmed !== '}'
      })
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

// Session Data (需要在 Task Details Panel 之前声明)
const currentSession = computed(() => pomodoroStore.currentSession)

// Task Details Panel
const showTaskDetails = ref(false)
const taskMilestones = ref<TaskMilestone[]>([])

// 切换任务详情面板
function toggleTaskDetailsPanel() {
  showTaskDetails.value = !showTaskDetails.value
}

// 判断是否有任何任务详情
const hasAnyTaskDetails = computed(() => {
  return (
    (aiSuggestionData.value?.subTasks?.length ?? 0) > 0 ||
    !!currentSession.value?.task?.notes ||
    currentSession.value?.task?.progress !== undefined ||
    taskMilestones.value.length > 0
  )
})

// 格式化里程碑日期
function formatMilestoneDate(dateStr: string) {
  try {
    const date = new Date(dateStr)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const days = Math.floor(diff / (1000 * 60 * 60 * 24))

    if (days === 0) {
      return '今天'
    } else if (days === 1) {
      return '昨天'
    } else if (days < 7) {
      return `${days}天前`
    } else {
      return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
    }
  } catch {
    return ''
  }
}

// 加载任务里程碑
async function loadTaskMilestones(taskId: number) {
  try {
    const milestones = await taskApi.getTaskMilestones(taskId)
    taskMilestones.value = milestones.sort((a, b) => {
      const dateA = new Date(a.createdAt || 0).getTime()
      const dateB = new Date(b.createdAt || 0).getTime()
      return dateB - dateA // 最新的在前面
    })
  } catch (error) {
    console.error('加载里程碑失败:', error)
    taskMilestones.value = []
  }
}

// 监听当前会话任务变化，加载里程碑
watch(() => currentSession.value?.taskId, async (taskId) => {
  if (taskId) {
    await loadTaskMilestones(taskId)
  } else {
    taskMilestones.value = []
  }
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

// 直接通过进程名判断是否选中（更可靠）
function isAppSelectedByProcess(processName: string) {
  if (!processName) return false
  return selectedAppProcessNames.value.includes(processName.toLowerCase())
}

// 直接通过进程名切换选中状态（更可靠）
function toggleAppByProcess(processName: string) {
  if (!processName) return

  const lowerName = processName.toLowerCase()
  const isSelected = selectedAppProcessNames.value.includes(lowerName)

  if (isSelected) {
    // 取消选中 - 创建新数组以确保响应式更新
    selectedAppProcessNames.value = selectedAppProcessNames.value.filter(name => name !== lowerName)
  } else {
    // 选中 - 创建新数组以确保响应式更新
    selectedAppProcessNames.value = [...selectedAppProcessNames.value, lowerName]
  }
}

// 直接通过进程名移除白名单应用
function removeWhitelistByProcess(processName: string) {
  if (!processName) return
  const lowerName = processName.toLowerCase()
  selectedAppProcessNames.value = selectedAppProcessNames.value.filter(name => name !== lowerName)
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

// 应用使用记录（番茄钟期间记录每次检测到的应用）
interface AppRecord {
  appName: string
  timestamp: number
}
const appUsageRecords = ref<AppRecord[]>([])

// 智能分心检测（无白名单时）
const smartDistractionSeconds = ref(0) // 离开DevAssistant的累计秒数
const SMART_DISTRACTION_THRESHOLD = 180 // 3分钟后提醒
const lastSmartReminderTime = ref<number>(0) // 上次提醒时间，避免频繁提醒
const SMART_REMINDER_COOLDOWN = 300000 // 5分钟冷却期

// 分心弹窗状态
const showDistractionModal = ref(false)
const distractedAppName = ref('')
const distractedProcessName = ref('') // 保存分心应用的进程名，用于添加白名单

// 白名单分心检测延迟（30秒后才弹窗）
const WHITELIST_DISTRACTION_DELAY = 30 // 秒
const whitelistDistractionSeconds = ref(0) // 在非白名单应用的累计秒数
const pendingDistractionApp = ref<{ appName: string, processName: string } | null>(null)

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
    let appName = '未知应用'
    if (windowInfo.appName) {
      appName = windowInfo.appName
      currentAppName.value = windowInfo.appName
    } else if (windowInfo.processName) {
      // 去掉 .exe 后缀显示
      appName = windowInfo.processName.replace(/\.exe$/i, '')
      currentAppName.value = appName
    } else {
      currentAppName.value = '未知应用'
    }

    // 记录应用使用（用于统计）
    if (currentPhase.value === 'focusing' && currentSession.value?.status === 'focusing') {
      appUsageRecords.value.push({
        appName: appName,
        timestamp: Date.now()
      })
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
    // 记录当前分心应用信息
    const detectedAppName = windowInfo.appName || windowInfo.processName?.replace(/\.exe$/i, '') || '未知应用'
    const detectedProcessName = windowInfo.processName || ''

    // 累加在非白名单应用的时间（每次检测间隔约2秒）
    whitelistDistractionSeconds.value += 2
    pendingDistractionApp.value = { appName: detectedAppName, processName: detectedProcessName }

    // 检查是否超过延迟阈值（30秒）
    if (whitelistDistractionSeconds.value >= WHITELIST_DISTRACTION_DELAY) {
      if (!isDistracted.value && !showDistractionModal.value) {
        isDistracted.value = true
        handleDistraction(detectedAppName, detectedProcessName)
      }
    }
  } else {
    // 回到白名单应用，重置计时
    isDistracted.value = false
    whitelistDistractionSeconds.value = 0
    pendingDistractionApp.value = null
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
async function handleDistraction(appName: string, processName: string = '') {
  if (!currentSession.value?.id) return

  // 如果弹窗已经显示，不重复触发
  if (showDistractionModal.value) return

  try {
    // 记录分心的应用名称和进程名
    distractedAppName.value = appName
    distractedProcessName.value = processName

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

    // 重置分心计时
    whitelistDistractionSeconds.value = 0

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

  // 先计算本地应用使用统计（在停止监控前计算）
  const localAppUsage = calculateLocalAppUsage()

  // 停止窗口监控
  stopWindowMonitoring()

  // 获取应用使用统计
  if (currentSession.value?.id) {
    try {
      const backendAppUsage = await getSessionAppUsage(currentSession.value.id)
      // 优先使用后端数据，如果为空则使用本地记录
      const parsedBackend = JSON.parse(backendAppUsage || '[]')
      if (parsedBackend.length > 0) {
        fetchedAppUsage.value = backendAppUsage
      } else {
        fetchedAppUsage.value = localAppUsage
      }
    } catch (e) {
      console.error('获取应用使用统计失败:', e)
      fetchedAppUsage.value = localAppUsage
    }
  } else {
    fetchedAppUsage.value = localAppUsage
  }

  // 跳转到报告页面
  progressValue.value = getCurrentTaskProgress()
  currentPhase.value = 'report'
  message.info('专注已停止，请提交反馈')
}

// 暂停计时（保持会话暂停状态，稍后继续）
function handlePauseTimer() {
  // 关闭弹窗，但保持会话暂停状态
  showDistractionModal.value = false
  isDistracted.value = false

  // 不恢复计时，让用户手动恢复
  message.info('计时已暂停，处理完事务后可继续')

  // 停止窗口监控
  stopWindowMonitoring()
}

// 将分心应用添加到白名单
async function handleAddToWhitelist() {
  if (!distractedProcessName.value && !distractedAppName.value) {
    message.warning('无法获取应用信息')
    return
  }

  // 获取进程名，如果没有则用应用名生成
  let processName = distractedProcessName.value
  if (!processName) {
    processName = distractedAppName.value.toLowerCase().replace(/\s+/g, '') + '.exe'
  }

  // 添加到本地列表（如果不存在）
  const existingApp = runningAppsList.value.find(
    app => app.processName.toLowerCase() === processName.toLowerCase()
  )

  if (!existingApp) {
    runningAppsList.value.push({
      name: distractedAppName.value,
      processName: processName
    })
  }

  // 添加到选中的白名单
  const lowerProcessName = processName.toLowerCase()
  if (!selectedAppProcessNames.value.includes(lowerProcessName)) {
    selectedAppProcessNames.value = [...selectedAppProcessNames.value, lowerProcessName]
  }

  message.success(`已将 ${distractedAppName.value} 添加到白名单`)

  // 恢复计时并关闭弹窗
  if (currentSession.value?.id) {
    try {
      await pomodoroStore.resumeSession(currentSession.value.id)
    } catch (error) {
      console.error('恢复会话失败:', error)
    }
  }

  showDistractionModal.value = false
  isDistracted.value = false

  // 延迟后重新开始监控
  setTimeout(() => {
    if (currentSession.value?.status === 'focusing') {
      startWindowMonitoring()
    }
  }, 2000)
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
  // 清空应用使用记录
  appUsageRecords.value = []
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

// 从本地记录计算应用使用统计
function calculateLocalAppUsage(): string {
  if (appUsageRecords.value.length === 0) {
    return '[]'
  }

  // 统计每个应用的出现次数
  const appCounts: Record<string, number> = {}
  for (const record of appUsageRecords.value) {
    appCounts[record.appName] = (appCounts[record.appName] || 0) + 1
  }

  const totalCount = appUsageRecords.value.length

  // 转换为统计数组
  const usageItems = Object.entries(appCounts)
    .map(([appName, count]) => ({
      appName,
      count,
      minutes: Math.round((count * 2) / 60), // 每次检测间隔2秒
      percentage: Math.round((count / totalCount) * 100)
    }))
    .sort((a, b) => b.percentage - a.percentage)

  return JSON.stringify(usageItems)
}

// Resume Modal
const showResumeModal = ref(false)

// Report
const feedback = ref('')
const taskCompleted = ref(false)
const progressValue = ref(0)
const addMilestone = ref(false)
const milestoneTitle = ref('')
const fetchedAppUsage = ref<string | null>(null)  // 手动停止时获取的app_usage
const lastCompletedSessionId = ref<number | null>(null)  // 刚完成的会话ID，用于AI分析
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

// 根据效率评分获取样式类
function getScoreClass(score: number): string {
  if (score >= 80) return 'score-excellent'
  if (score >= 60) return 'score-good'
  if (score >= 40) return 'score-average'
  return 'score-low'
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

// Session Data (currentSession 已在前面声明)
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

// 会话分析结果类型（匹配后端返回的JSON格式）
interface SessionAnalysis {
  relevanceAnalysis: string    // 应用使用与任务相关性分析
  efficiencyScore: number      // 专注效率评分 (1-100)
  efficiencyComment: string    // 专注效率评估说明
  improvements: string[]       // 改进建议
  nextAction: string           // 下次行动建议
}

// 最近会话的AI分析
const lastSessionAnalysis = ref<SessionAnalysis | null>(null)
const analysisLoading = ref(false)

// AI 每日复盘（保留用于右侧AI分析的备选显示）
const aiDailyInsight = ref('')
const aiInsightLoading = ref(false)

// 获取最近完成的会话的AI分析
async function loadLastSessionAnalysis(sessionId?: number) {
  if (analysisLoading.value) return

  analysisLoading.value = true
  lastSessionAnalysis.value = null

  try {
    // 优先使用传入的sessionId（刚完成的会话）
    let targetSessionId = sessionId

    // 如果没有传入，尝试从currentSession获取
    if (!targetSessionId && currentSession.value?.id) {
      targetSessionId = currentSession.value.id
    }

    // 如果还没有，从今日已完成的会话中查找最近的
    if (!targetSessionId) {
      const completedSessions = pomodoroStore.todaySessions.filter(
        s => s.status === 'completed'
      )

      if (completedSessions.length === 0) {
        return
      }

      // 按完成时间排序，获取最近完成的会话
      const sortedSessions = [...completedSessions].sort((a, b) => {
        const timeA = a.completedAt ? new Date(a.completedAt).getTime() : 0
        const timeB = b.completedAt ? new Date(b.completedAt).getTime() : 0
        return timeB - timeA  // 降序，最新的在前
      })

      targetSessionId = sortedSessions[0]?.id
    }

    if (!targetSessionId) {
      return
    }

    console.log('[AI分析] 分析会话ID:', targetSessionId)

    // 调用AI分析接口
    const result = await aiAnalyzeSession(targetSessionId)
    console.log('[AI分析] 原始返回:', result)

    // 尝试解析JSON结果
    let parsed: any = null
    try {
      // 1. 先尝试直接解析
      parsed = JSON.parse(result)
    } catch {
      // 2. 尝试从markdown代码块中提取JSON
      const jsonMatch = result.match(/```(?:json)?\s*([\s\S]*?)```/)
      if (jsonMatch && jsonMatch[1]) {
        try {
          parsed = JSON.parse(jsonMatch[1].trim())
          console.log('[AI分析] 从markdown提取JSON成功')
        } catch (e) {
          console.log('[AI分析] markdown JSON解析失败:', e)
        }
      }

      // 3. 尝试查找JSON对象边界
      if (!parsed) {
        const jsonStart = result.indexOf('{')
        const jsonEnd = result.lastIndexOf('}')
        if (jsonStart !== -1 && jsonEnd > jsonStart) {
          try {
            parsed = JSON.parse(result.substring(jsonStart, jsonEnd + 1))
            console.log('[AI分析] 从文本提取JSON成功')
          } catch (e) {
            console.log('[AI分析] 文本JSON解析失败:', e)
          }
        }
      }
    }

    if (parsed && typeof parsed === 'object') {
      lastSessionAnalysis.value = {
        relevanceAnalysis: parsed.relevance_analysis || parsed.relevanceAnalysis || '分析中...',
        efficiencyScore: parsed.efficiency_score || parsed.efficiencyScore || 0,
        efficiencyComment: parsed.efficiency_comment || parsed.efficiencyComment || '分析中...',
        improvements: parsed.improvements || [],
        nextAction: parsed.next_action || parsed.nextAction || '继续保持专注'
      }
      console.log('[AI分析] 解析成功:', lastSessionAnalysis.value)

      // 保存AI分析结果到数据库
      if (targetSessionId) {
        try {
          await updateAiAnalysis(targetSessionId, result)
          console.log('[AI分析] 已保存到数据库')
        } catch (saveError) {
          console.error('[AI分析] 保存到数据库失败:', saveError)
        }
      }
    } else {
      // 如果完全无法解析为JSON，使用原始文本
      console.log('[AI分析] 无法解析为JSON，使用原始文本')
      lastSessionAnalysis.value = {
        relevanceAnalysis: result.length > 200 ? result.substring(0, 200) + '...' : result,
        efficiencyScore: 0,
        efficiencyComment: '分析结果格式异常',
        improvements: [],
        nextAction: '请重试或检查AI配置'
      }
    }
  } catch (error: any) {
    console.error('获取会话分析失败:', error)
    const errorMsg = error?.toString() || ''
    if (errorMsg.includes('AI 服务未初始化') || errorMsg.includes('AI 服务未配置')) {
      // AI未配置时不显示错误
      lastSessionAnalysis.value = null
    }
  } finally {
    analysisLoading.value = false
  }
}

// 获取 AI 每日复盘（备用）
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

// 监听进入统计页面时加载数据和分析
watch(currentPhase, async (newPhase) => {
  if (newPhase === 'stats') {
    // 刷新统计数据
    await pomodoroStore.loadTodayStats()
    await pomodoroStore.loadTodaySessions()
    // 获取最近会话的AI分析（优先使用刚完成的会话ID）
    const sessionIdToAnalyze = lastCompletedSessionId.value || undefined
    loadLastSessionAnalysis(sessionIdToAnalyze)
    // 清空已使用的ID
    lastCompletedSessionId.value = null
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
  // 先计算本地应用使用统计（在停止监控前计算）
  const localAppUsage = calculateLocalAppUsage()
  console.log('[计时完成] 本地应用统计:', localAppUsage)

  // 停止窗口监控
  stopWindowMonitoring()

  // 获取应用使用统计（与handleStop相同逻辑）
  if (currentSession.value?.id) {
    try {
      const backendAppUsage = await getSessionAppUsage(currentSession.value.id)
      const parsedBackend = JSON.parse(backendAppUsage || '[]')
      if (parsedBackend.length > 0) {
        fetchedAppUsage.value = backendAppUsage
        console.log('[计时完成] 使用后端数据')
      } else {
        fetchedAppUsage.value = localAppUsage
        console.log('[计时完成] 使用本地数据')
      }
    } catch (e) {
      console.error('[计时完成] 获取应用使用统计失败:', e)
      fetchedAppUsage.value = localAppUsage
    }
  } else {
    fetchedAppUsage.value = localAppUsage
  }

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

// 当前会话或今日完成会话的应用使用数据
// 优先显示刚完成的当前会话数据，如果没有则显示最近完成的会话数据
const appUsageList = computed(() => {
  // 优先使用刚获取的当前会话应用数据（手动停止或计时完成时获取的）
  if (fetchedAppUsage.value) {
    try {
      const usageData = JSON.parse(fetchedAppUsage.value)
      if (Array.isArray(usageData) && usageData.length > 0) {
        const totalMinutes = usageData.reduce((sum: number, item: any) => sum + (item.minutes || 0), 0)
        if (totalMinutes > 0) {
          return usageData.map((item: any) => ({
            appName: item.appName || item.app_name || '未知',
            minutes: Math.round(item.minutes || 0),
            percentage: Math.round(((item.minutes || 0) / totalMinutes) * 100)
          })).sort((a: any, b: any) => b.minutes - a.minutes)
        }
      }
    } catch {
      // 解析失败，继续使用下面的逻辑
    }
  }

  // 获取今日已完成的番茄钟会话
  const completedSessions = pomodoroStore.todaySessions.filter(
    s => s.status === 'completed'
  )

  if (completedSessions.length === 0) {
    return []
  }

  // 获取最近完成的会话的应用数据（而不是汇总所有会话）
  const lastSession = completedSessions[completedSessions.length - 1]
  if (lastSession?.appUsage) {
    let usageData: any = lastSession.appUsage
    if (typeof usageData === 'string') {
      try {
        usageData = JSON.parse(usageData)
      } catch {
        usageData = null
      }
    }
    if (Array.isArray(usageData) && usageData.length > 0) {
      const totalMinutes = usageData.reduce((sum: number, item: any) => sum + (item.minutes || 0), 0)
      if (totalMinutes > 0) {
        return usageData.map((item: any) => ({
          appName: item.appName || item.app_name || '未知',
          minutes: Math.round(item.minutes || 0),
          percentage: Math.round(((item.minutes || 0) / totalMinutes) * 100)
        })).sort((a: any, b: any) => b.minutes - a.minutes)
      }
    }
  }

  // 如果最近会话没有应用数据，显示空
  return []
})

// 解析当前会话的应用使用统计数据（用于 report 阶段展示）
const sessionAppUsage = computed(() => {
  // 优先使用手动获取的app_usage（handleStop时获取）
  // 如果没有，再尝试使用currentSession中的appUsage
  const rawData = fetchedAppUsage.value || currentSession.value?.appUsage
  if (!rawData) {
    return []
  }

  // 后端可能返回 JSON 字符串或对象数组
  let usageData = rawData
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

    // 将AI建议序列化为JSON字符串保存到数据库
    let aiSuggestionJson: string | undefined = undefined
    if (aiSuggestionData.value) {
      // 如果有结构化数据，保存结构化数据
      try {
        aiSuggestionJson = JSON.stringify(aiSuggestionData.value)
      } catch (e) {
        console.error('序列化AI建议失败:', e)
      }
    } else if (aiSuggestions.value.length > 0) {
      // 如果只有简单建议，保存为简单格式
      try {
        aiSuggestionJson = JSON.stringify({
          suggestions: aiSuggestions.value
        })
      } catch (e) {
        console.error('序列化AI建议失败:', e)
      }
    }

    const session = await pomodoroStore.createSession({
      taskId: selectedTaskId.value,
      durationMinutes: duration.value,
      focusGoal: focusGoal,
      focusApps: focusAppsList,
      aiSuggestion: aiSuggestionJson
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

  const sessionId = currentSession.value.id

  // 先计算本地应用使用统计（在停止监控前计算）
  const localAppUsage = calculateLocalAppUsage()

  try {
    // 停止窗口监控
    stopWindowMonitoring()

    // 尝试从后端获取应用使用统计（基于screen_contexts，如果开启了VLM采集）
    let backendAppUsage = '[]'
    try {
      backendAppUsage = await getSessionAppUsage(sessionId)
      console.log('后端应用使用统计:', backendAppUsage)
    } catch (e) {
      console.error('获取后端应用使用统计失败:', e)
    }

    // 优先使用后端数据，如果为空则使用本地记录
    const parsedBackend = JSON.parse(backendAppUsage || '[]')
    if (parsedBackend.length > 0) {
      fetchedAppUsage.value = backendAppUsage
    } else {
      fetchedAppUsage.value = localAppUsage
      console.log('使用本地应用使用统计:', localAppUsage)
    }

    // 暂停当前会话
    await pomodoroStore.pauseSession(sessionId)

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

    // 完成番茄钟会话（传入应用使用统计，保存到数据库）
    await pomodoroStore.completeSession(sessionId, {
      feedback: feedback.value,
      progressUpdate: feedback.value,
      appUsage: fetchedAppUsage.value || undefined
    })

    // 如果有任务关联，更新进度并创建里程碑
    if (sessionTaskId) {
      try {
        // 更新任务进度（确保是数字类型）
        const progressNum = Number(progressValue.value)

        // 获取当前任务进度用于比较
        const currentTask = taskStore.tasks.find(t => t.id === sessionTaskId)
        const previousProgress = currentTask?.progress || 0

        await taskApi.updateTaskProgress(sessionTaskId, progressNum)

        // 当进度有变化时，自动创建里程碑记录
        if (progressNum > previousProgress) {
          // 用户自定义标题 > 反馈内容前50字 > 默认标题
          const title = milestoneTitle.value.trim() ||
                       (feedback.value ? feedback.value.slice(0, 50) : '') ||
                       `番茄钟专注 - 进度${progressNum}%`
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

    // 保存刚完成的会话ID，用于AI分析
    lastCompletedSessionId.value = sessionId

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

  // 从路由参数获取任务ID并自动选择
  const taskIdFromRoute = route.query.taskId
  if (taskIdFromRoute) {
    const taskId = Number(taskIdFromRoute)
    // 确保任务存在
    const task = taskStore.tasks.find(t => t.id === taskId)
    if (task) {
      selectedTaskId.value = taskId
      // 清除路由参数，避免刷新页面时重复选择
      router.replace({ query: {} })
    }
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
  background: linear-gradient(135deg, var(--bg-base, #020617) 0%, var(--bg-surface, #0f172a) 100%);
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
  background: var(--accent-primary, #6366f1);
  box-shadow: 0 0 12px color-mix(in srgb, var(--accent-primary, #6366f1) 60%, transparent);
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
  color: var(--text-muted, #64748b);
  text-transform: uppercase;
}

.scene-title {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary, #f1f5f9);
  margin: 0;
  letter-spacing: -0.02em;
}

/* Task Card */
.task-card {
  background: color-mix(in srgb, var(--card-bg, #0f172a) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 50%, transparent);
  border-radius: 20px;
  overflow: hidden;
  backdrop-filter: blur(12px);
}

/* Task Selection Area */
.task-selection-area {
  padding: 20px;
  border-bottom: 1px solid color-mix(in srgb, var(--border-default, #334155) 30%, transparent);
}

.task-selection-label {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary, #94a3b8);
}

.task-category-tag {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 15%, transparent);
  color: var(--accent-secondary, #818cf8);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
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
  background: color-mix(in srgb, var(--input-bg, #1e293b) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--input-border, #475569) 50%, transparent);
  border-radius: 12px;
  color: var(--text-primary, #f1f5f9);
  font-size: 15px;
  cursor: pointer;
  outline: none;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24'%3E%3Cpath fill='%2394a3b8' d='M7 10l5 5 5-5z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
}

.task-native-select:hover {
  border-color: var(--border-hover, #6366f1);
}

.task-native-select:focus {
  border-color: var(--accent-primary, #6366f1);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
}

.task-native-select option {
  background: var(--input-bg, #1e293b);
  color: var(--text-primary, #f1f5f9);
  padding: 12px;
}

.task-native-select optgroup {
  font-weight: 600;
  font-size: 13px;
  color: var(--text-secondary, #94a3b8);
  background: var(--bg-surface, #0f172a);
  padding: 8px 0;
}

.task-native-select optgroup[label="进行中"] {
  color: #60a5fa;
}

.task-native-select optgroup[label="待办"] {
  color: #94a3b8;
}

.task-or-divider {
  text-align: center;
  color: var(--text-muted, #64748b);
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
  background: color-mix(in srgb, var(--border-default, #475569) 50%, transparent);
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
  background: color-mix(in srgb, var(--input-bg, #1e293b) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--input-border, #475569) 50%, transparent);
  border-radius: 12px;
  color: var(--text-primary, #f1f5f9);
  font-size: 15px;
  outline: none;
}

.new-task-input::placeholder {
  color: var(--text-muted, #64748b);
}

.new-task-input:focus {
  border-color: var(--accent-primary, #6366f1);
}

.create-task-btn {
  height: 48px;
  padding: 0 24px;
  background: linear-gradient(135deg, var(--accent-primary, #6366f1) 0%, var(--accent-primary, #4f46e5) 100%);
  border: none;
  border-radius: 12px;
  color: white;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.create-task-btn:hover:not(:disabled) {
  box-shadow: 0 4px 12px color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
}

.create-task-btn:disabled {
  background: var(--bg-elevated, #334155);
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
  background: color-mix(in srgb, var(--bg-base, #020617) 40%, transparent);
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
  color: var(--text-secondary, #cbd5e1);
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
  background: color-mix(in srgb, var(--card-bg, #0f172a) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 40%, transparent);
  border-radius: 12px;
  font-size: 14px;
  color: var(--text-secondary, #94a3b8);
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
  background: var(--accent-primary, #6366f1);
}

.suggestion-dot.purple {
  background: var(--accent-secondary, #a78bfa);
}

.suggestion-item .highlight {
  color: var(--text-primary, #f1f5f9);
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
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  border-radius: 10px;
}

.goal-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent-secondary, #818cf8);
  margin-right: 6px;
}

.goal-text {
  font-size: 14px;
  color: var(--text-primary, #e2e8f0);
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
  background: color-mix(in srgb, var(--card-bg, #0f172a) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 40%, transparent);
  border-radius: 10px;
}

.step-number {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  color: var(--accent-secondary, #818cf8);
  border-radius: 50%;
  font-size: 12px;
  font-weight: 600;
  flex-shrink: 0;
}

.step-text {
  font-size: 13px;
  color: var(--text-secondary, #94a3b8);
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
  background: color-mix(in srgb, var(--card-bg, #0f172a) 50%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 40%, transparent);
  border-radius: 16px;
  padding: 16px;
}

.setting-label {
  display: block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted, #64748b);
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
  background: color-mix(in srgb, var(--bg-elevated, #334155) 50%, transparent);
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
  color: var(--text-secondary, #cbd5e1);
  display: flex;
  align-items: center;
  justify-content: center;
}

.app-tag.add {
  background: transparent;
  border: 1px dashed color-mix(in srgb, var(--border-default, #64748b) 50%, transparent);
  color: var(--text-muted, #64748b);
  cursor: pointer;
  transition: all 0.2s;
}

.app-tag.add:hover {
  border-color: var(--accent-primary, #6366f1);
  color: var(--accent-primary, #6366f1);
}

.duration-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.duration-btn {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--bg-elevated, #334155) 40%, transparent);
  border: 1px solid color-mix(in srgb, var(--border-default, #475569) 40%, transparent);
  border-radius: 8px;
  color: var(--text-secondary, #94a3b8);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.duration-btn:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  color: var(--text-primary, #c7d2fe);
}

.duration-btn.active {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  border-color: var(--accent-primary, #6366f1);
  color: var(--accent-secondary, #a5b4fc);
}

/* Start Button */
.start-button {
  width: 100%;
  height: 56px;
  background: linear-gradient(135deg, var(--accent-primary, #6366f1) 0%, var(--accent-primary, #4f46e5) 100%);
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
  box-shadow: 0 8px 24px color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  margin-top: 8px;
}

.start-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 12px 32px color-mix(in srgb, var(--accent-primary, #6366f1) 40%, transparent);
}

.start-button:active:not(:disabled) {
  transform: translateY(0);
}

.start-button:disabled {
  background: var(--bg-elevated, #334155);
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
  color: var(--text-primary, #f1f5f9);
  letter-spacing: -0.02em;
  text-shadow: 0 4px 24px rgba(0, 0, 0, 0.3);
}

.active-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 12px;
  padding: 6px 14px;
  background: color-mix(in srgb, var(--input-bg, #1e293b) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--border-default, #334155) 50%, transparent);
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary, #94a3b8);
  letter-spacing: 0.05em;
}

.focus-info {
  text-align: center;
}

.focus-task-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary, #f1f5f9);
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
  background: var(--bg-elevated, #334155);
}

.status-dots .dot.active {
  background: var(--accent-primary, #6366f1);
}

.status-text {
  font-size: 11px;
  font-style: italic;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-muted, #64748b);
}

.focus-controls {
  display: flex;
  gap: 24px;
}

.control-btn {
  width: 56px;
  height: 56px;
  background: color-mix(in srgb, var(--card-bg, #0f172a) 80%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 50%, transparent);
  border-radius: 16px;
  color: var(--text-secondary, #94a3b8);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.control-btn:hover {
  color: var(--text-primary, #f1f5f9);
  border-color: var(--accent-primary, #6366f1);
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
}

.control-btn.stop:hover {
  color: #f87171;
  border-color: #f87171;
  background: color-mix(in srgb, #f87171 10%, transparent);
}

/* ===== TASK DETAILS PANEL ===== */
.task-details-panel {
  width: 100%;
  margin-top: 20px;
}

.panel-toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 12px 16px;
  background: color-mix(in srgb, var(--card-bg, #0f172a) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 50%, transparent);
  border-radius: 12px;
  color: var(--text-secondary, #94a3b8);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.panel-toggle-btn:hover {
  color: var(--text-primary, #f1f5f9);
  border-color: var(--accent-primary, #6366f1);
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
}

.toggle-icon {
  transition: transform 0.3s ease;
}

.toggle-icon.expanded {
  transform: rotate(180deg);
}

.task-details-content {
  margin-top: 16px;
  padding: 20px;
  background: color-mix(in srgb, var(--card-bg, #0f172a) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 50%, transparent);
  border-radius: 16px;
  overflow: hidden;
}

.detail-section {
  margin-bottom: 20px;
}

.detail-section:last-child {
  margin-bottom: 0;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary, #cbd5e1);
}

.steps-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.step-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 10px 12px;
  background: color-mix(in srgb, var(--input-bg, #1e293b) 40%, transparent);
  border: 1px solid color-mix(in srgb, var(--border-default, #334155) 30%, transparent);
  border-radius: 8px;
}

.step-number {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  border-radius: 6px;
  color: var(--accent-secondary, #818cf8);
  font-size: 11px;
  font-weight: 600;
}

.step-text {
  flex: 1;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-primary, #e2e8f0);
}

.task-notes {
  padding: 12px;
  background: color-mix(in srgb, var(--input-bg, #1e293b) 40%, transparent);
  border: 1px solid color-mix(in srgb, var(--border-default, #334155) 30%, transparent);
  border-radius: 8px;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-secondary, #cbd5e1);
  white-space: pre-wrap;
}

.progress-bar-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
}

.progress-bar-bg {
  flex: 1;
  height: 8px;
  background: rgba(30, 41, 59, 0.6);
  border-radius: 4px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #6366f1 0%, #818cf8 100%);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.progress-text {
  flex-shrink: 0;
  font-size: 13px;
  font-weight: 600;
  color: #818cf8;
  min-width: 42px;
  text-align: right;
}

.milestones-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.milestone-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  background: rgba(30, 41, 59, 0.4);
  border: 1px solid rgba(51, 65, 85, 0.3);
  border-radius: 8px;
}

.milestone-icon {
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  margin-top: 6px;
  background: #6366f1;
  border-radius: 50%;
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);
}

.milestone-content {
  flex: 1;
}

.milestone-title {
  font-size: 13px;
  font-weight: 500;
  color: #e2e8f0;
  margin-bottom: 4px;
}

.milestone-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 11px;
  color: #94a3b8;
}

.milestone-date {
  color: #64748b;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  text-align: center;
}

.empty-state svg {
  margin-bottom: 12px;
  opacity: 0.5;
}

.empty-state p {
  margin: 0;
  font-size: 13px;
  color: #64748b;
}

/* Panel Slide Animation */
.panel-slide-enter-active,
.panel-slide-leave-active {
  transition: all 0.3s ease;
}

.panel-slide-enter-from {
  opacity: 0;
  transform: translateY(-10px);
}

.panel-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* ===== REPORT SCENE ===== */
.report-scene {
  max-width: 640px;
}

.report-card {
  background: color-mix(in srgb, var(--card-bg, #0f172a) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 50%, transparent);
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
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
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
  color: var(--text-primary, #f1f5f9);
  margin: 0 0 4px 0;
}

.report-meta {
  font-size: 13px;
  color: var(--text-muted, #64748b);
  margin: 0;
}

.session-tag {
  padding: 8px 14px;
  background: color-mix(in srgb, var(--bg-elevated, #334155) 50%, transparent);
  border: 1px solid color-mix(in srgb, var(--border-default, #475569) 50%, transparent);
  border-radius: 20px;
  font-size: 10px;
  font-weight: 800;
  color: var(--text-secondary, #94a3b8);
  letter-spacing: 0.05em;
}

/* Progress Update Section */
.progress-update-section {
  margin-bottom: 20px;
  padding: 16px;
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
  border-radius: 16px;
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
}

.section-label {
  display: block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: var(--accent-primary, #6366f1);
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
  background: color-mix(in srgb, var(--bg-elevated, #334155) 50%, transparent);
  border-radius: 4px;
  outline: none;
}

.progress-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 20px;
  height: 20px;
  background: var(--accent-primary, #6366f1);
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 2px 8px color-mix(in srgb, var(--accent-primary, #6366f1) 40%, transparent);
}

.progress-slider::-moz-range-thumb {
  width: 20px;
  height: 20px;
  background: var(--accent-primary, #6366f1);
  border-radius: 50%;
  cursor: pointer;
  border: none;
  box-shadow: 0 2px 8px color-mix(in srgb, var(--accent-primary, #6366f1) 40%, transparent);
}

.progress-display {
  font-size: 20px;
  font-weight: 700;
  color: var(--accent-primary, #6366f1);
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
  background: color-mix(in srgb, var(--bg-base, #020617) 50%, transparent);
  border: 1px solid color-mix(in srgb, #ec4899 30%, transparent);
  border-radius: 12px;
  color: var(--text-primary, #e2e8f0);
  font-size: 14px;
  outline: none;
  font-family: inherit;
}

.milestone-input::placeholder {
  color: var(--text-muted, #64748b);
}

.milestone-input:focus {
  border-color: #ec4899;
}

/* App Usage Section in Report */
.app-usage-section {
  margin-bottom: 24px;
  padding: 20px;
  background: color-mix(in srgb, var(--bg-base, #020617) 50%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 50%, transparent);
  border-radius: 16px;
}

.app-usage-section .section-label {
  display: block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: var(--text-muted, #64748b);
  margin-bottom: 16px;
}

.session-usage-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.session-usage-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.app-usage-section .usage-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 20px;
  color: #6b7280;
  font-size: 13px;
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
  color: var(--text-muted, #64748b);
  margin-bottom: 12px;
}

.feedback-textarea {
  width: 100%;
  height: 140px;
  padding: 16px;
  background: color-mix(in srgb, var(--bg-base, #020617) 50%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 50%, transparent);
  border-radius: 16px;
  color: var(--text-primary, #e2e8f0);
  font-size: 15px;
  line-height: 1.6;
  resize: none;
  outline: none;
  font-family: inherit;
}

.feedback-textarea::placeholder {
  color: var(--text-dim, #475569);
}

.feedback-textarea:focus {
  border-color: var(--accent-primary, #6366f1);
}

.submit-button {
  width: 100%;
  height: 56px;
  background: var(--text-primary, #f1f5f9);
  border: none;
  border-radius: 16px;
  color: var(--bg-surface, #0f172a);
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
  background: var(--text-primary, #e2e8f0);
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
  color: var(--text-primary, #f1f5f9);
  margin: 0;
}

.date-tag {
  padding: 8px 14px;
  background: color-mix(in srgb, var(--card-bg, #0f172a) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 50%, transparent);
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary, #94a3b8);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.stat-item {
  background: color-mix(in srgb, var(--card-bg, #0f172a) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 50%, transparent);
  border-radius: 20px;
  padding: 20px;
}

.stat-label {
  display: block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted, #64748b);
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

.stat-value.blue { color: var(--accent-primary, #6366f1); }
.stat-value.green { color: #10b981; }
.stat-value.white { color: var(--text-primary, #f1f5f9); }
.stat-value.red { color: #f87171; }

.charts-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.chart-card {
  background: color-mix(in srgb, var(--card-bg, #0f172a) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 50%, transparent);
  border-radius: 24px;
  padding: 24px;
}

.chart-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary, #cbd5e1);
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
  color: var(--text-secondary, #94a3b8);
}

.app-time {
  color: var(--accent-primary, #6366f1);
  font-family: monospace;
}

.usage-bar {
  height: 6px;
  background: color-mix(in srgb, var(--bg-elevated, #334155) 50%, transparent);
  border-radius: 3px;
  overflow: hidden;
}

.usage-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-primary, #6366f1), var(--accent-secondary, #818cf8));
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
  color: var(--text-primary, #f1f5f9);
}

.ai-insight-text {
  font-size: 13px;
  line-height: 1.7;
  color: var(--text-secondary, #cbd5e1);
  margin: 0 0 20px 0;
}

/* 分析结果外层容器 - 添加高度限制和滚动 */
.session-analysis-wrapper {
  max-height: 300px;
  overflow-y: auto;
  margin-bottom: 16px;
  padding-right: 8px;
}

.session-analysis-wrapper::-webkit-scrollbar {
  width: 4px;
}

.session-analysis-wrapper::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 2px;
}

.session-analysis-wrapper::-webkit-scrollbar-thumb {
  background: rgba(99, 102, 241, 0.4);
  border-radius: 2px;
}

.session-analysis-wrapper::-webkit-scrollbar-thumb:hover {
  background: rgba(99, 102, 241, 0.6);
}

.session-analysis {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 效率评分展示 */
.efficiency-score-section {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px;
  background: rgba(99, 102, 241, 0.1);
  border-radius: 8px;
  margin-bottom: 8px;
}

.score-circle {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.score-circle.score-excellent {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
}

.score-circle.score-good {
  background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%);
}

.score-circle.score-average {
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
}

.score-circle.score-low {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
}

.score-value {
  font-size: 18px;
  font-weight: 700;
  color: #ffffff;
  line-height: 1;
}

.score-label {
  font-size: 10px;
  color: rgba(255, 255, 255, 0.8);
  margin-top: 2px;
}

.efficiency-comment {
  flex: 1;
  font-size: 13px;
  line-height: 1.5;
  color: #cbd5e1;
  margin: 0;
}

.analysis-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.analysis-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--accent-primary, #6366f1);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0;
}

.analysis-text {
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-secondary, #cbd5e1);
  margin: 0;
}

.analysis-text.highlight {
  color: var(--accent-secondary, #a5b4fc);
  font-weight: 500;
}

.analysis-list {
  margin: 0;
  padding-left: 20px;
  list-style: none;
}

.analysis-list li {
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-secondary, #cbd5e1);
  position: relative;
  margin-bottom: 6px;
}

.analysis-list li::before {
  content: "•";
  color: var(--accent-primary, #6366f1);
  font-weight: bold;
  position: absolute;
  left: -15px;
}

.view-report-btn {
  background: none;
  border: none;
  color: var(--accent-primary, #6366f1);
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
  color: var(--accent-secondary, #818cf8);
}

.exit-button {
  width: 100%;
  padding: 16px;
  background: transparent;
  border: none;
  color: var(--text-dim, #475569);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.2em;
  cursor: pointer;
  transition: color 0.2s;
  margin-top: 16px;
}

.exit-button:hover {
  color: var(--text-secondary, #94a3b8);
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
  background: color-mix(in srgb, var(--card-bg, #0f172a) 95%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  border-radius: 24px;
  padding: 32px;
  text-align: center;
  backdrop-filter: blur(16px);
}

.modal-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto 16px;
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  border-radius: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary, #f1f5f9);
  margin: 0 0 8px 0;
}

.modal-subtitle {
  font-size: 14px;
  color: var(--text-muted, #64748b);
  margin: 0 0 24px 0;
}

.modal-task {
  background: color-mix(in srgb, var(--input-bg, #1e293b) 50%, transparent);
  border: 1px solid color-mix(in srgb, var(--border-default, #334155) 50%, transparent);
  border-radius: 12px;
  padding: 14px 16px;
  margin-bottom: 24px;
  text-align: left;
}

.modal-task .task-label {
  display: block;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-muted, #64748b);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 4px;
}

.modal-task .task-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary, #e2e8f0);
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
  background: linear-gradient(135deg, var(--accent-primary, #6366f1) 0%, var(--accent-primary, #4f46e5) 100%);
  border: none;
  color: white;
}

.modal-btn.primary:hover {
  box-shadow: 0 8px 20px color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
}

.modal-btn.secondary {
  background: transparent;
  border: 1px solid color-mix(in srgb, var(--border-default, #334155) 50%, transparent);
  color: var(--text-secondary, #94a3b8);
}

.modal-btn.secondary:hover {
  border-color: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  color: var(--text-primary, #e2e8f0);
}

/* ===== APP SELECTOR MODAL ===== */
.app-selector-modal {
  width: 100%;
  max-width: 480px;
  background: color-mix(in srgb, var(--card-bg, #0f172a) 98%, transparent);
  border: 1px solid color-mix(in srgb, var(--border-default, #475569) 50%, transparent);
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
  color: var(--text-primary, #f1f5f9);
  margin: 0;
}

.close-btn {
  width: 32px;
  height: 32px;
  background: color-mix(in srgb, var(--bg-elevated, #334155) 50%, transparent);
  border: none;
  border-radius: 8px;
  color: var(--text-secondary, #94a3b8);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.close-btn:hover {
  background: color-mix(in srgb, #f87171 20%, transparent);
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
  color: var(--text-muted, #64748b);
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
  background: color-mix(in srgb, var(--input-bg, #1e293b) 50%, transparent);
  border: 1px solid color-mix(in srgb, var(--border-default, #334155) 50%, transparent);
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.app-item:hover {
  background: color-mix(in srgb, var(--bg-elevated, #334155) 50%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
}

.app-item.selected {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 15%, transparent);
  border-color: var(--accent-primary, #6366f1);
}

.loading-apps,
.no-apps {
  padding: 40px 20px;
  text-align: center;
  color: var(--text-muted, #64748b);
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
  background: color-mix(in srgb, var(--bg-elevated, #334155) 80%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 800;
  color: var(--text-primary, #e2e8f0);
  flex-shrink: 0;
}

.app-item.selected .app-icon-text {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  color: var(--accent-secondary, #a5b4fc);
}

.app-item .app-name {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary, #cbd5e1);
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
  background: color-mix(in srgb, var(--input-bg, #1e293b) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--input-border, #334155) 50%, transparent);
  border-radius: 10px;
  color: var(--text-primary, #e2e8f0);
  font-size: 14px;
  outline: none;
}

.add-app-input::placeholder {
  color: var(--text-muted, #64748b);
}

.add-app-input:focus {
  border-color: var(--accent-primary, #6366f1);
}

.add-app-btn {
  height: 44px;
  padding: 0 20px;
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  border-radius: 10px;
  color: var(--accent-secondary, #a5b4fc);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.add-app-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
}

.add-app-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.modal-footer {
  padding-top: 16px;
  border-top: 1px solid color-mix(in srgb, var(--border-default, #334155) 30%, transparent);
}

.confirm-btn {
  width: 100%;
  height: 48px;
  background: linear-gradient(135deg, var(--accent-primary, #6366f1) 0%, var(--accent-primary, #4f46e5) 100%);
  border: none;
  border-radius: 12px;
  color: white;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.confirm-btn:hover {
  box-shadow: 0 8px 20px color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
}

/* ===== DISTRACTION MODAL ===== */
.distraction-overlay {
  z-index: 2000;
}

.distraction-modal {
  width: 100%;
  max-width: 420px;
  background: color-mix(in srgb, var(--card-bg, #0f172a) 98%, transparent);
  border: 1px solid color-mix(in srgb, #7f1d1d 50%, transparent);
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
  background: color-mix(in srgb, #7f1d1d 30%, transparent);
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.distraction-title {
  font-size: 26px;
  font-weight: 700;
  color: var(--text-primary, #f1f5f9);
  margin: 0 0 16px 0;
}

.distraction-desc {
  font-size: 16px;
  color: var(--text-secondary, #94a3b8);
  margin: 0 0 8px 0;
  line-height: 1.5;
}

.distraction-desc strong {
  color: var(--text-primary, #f1f5f9);
  text-decoration: underline;
  text-decoration-color: #ef4444;
  text-underline-offset: 3px;
}

.distraction-hint {
  font-size: 14px;
  color: var(--text-muted, #64748b);
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
  border: 1px solid color-mix(in srgb, var(--border-default, #475569) 50%, transparent);
  color: var(--text-secondary, #94a3b8);
}

.distraction-btn.secondary:hover {
  border-color: color-mix(in srgb, var(--accent-primary, #6366f1) 50%, transparent);
  color: var(--accent-secondary, #a5b4fc);
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
}

.distraction-btn.whitelist {
  background: color-mix(in srgb, #22c55e 10%, transparent);
  border: 1px solid color-mix(in srgb, #22c55e 30%, transparent);
  color: #4ade80;
  display: flex;
  align-items: center;
  justify-content: center;
}

.distraction-btn.whitelist:hover {
  border-color: color-mix(in srgb, #22c55e 60%, transparent);
  background: color-mix(in srgb, #22c55e 20%, transparent);
  box-shadow: 0 4px 12px color-mix(in srgb, #22c55e 20%, transparent);
}

.distraction-btn.danger {
  background: transparent;
  border: 1px solid color-mix(in srgb, #ef4444 30%, transparent);
  color: #f87171;
}

.distraction-btn.danger:hover {
  border-color: color-mix(in srgb, #ef4444 60%, transparent);
  background: color-mix(in srgb, #ef4444 10%, transparent);
}

/* ===== SCENE SWITCHER ===== */
.scene-switcher {
  position: fixed;
  bottom: 20px;
  right: 20px;
  background: color-mix(in srgb, var(--card-bg, #0f172a) 90%, transparent);
  border: 1px solid color-mix(in srgb, var(--card-border, #334155) 50%, transparent);
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
  color: var(--text-muted, #64748b);
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.08em;
  cursor: pointer;
  transition: all 0.2s;
}

.scene-btn:hover {
  background: color-mix(in srgb, var(--bg-elevated, #334155) 50%, transparent);
  color: var(--text-secondary, #94a3b8);
}

.scene-btn.active {
  background: var(--accent-primary, #6366f1);
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
