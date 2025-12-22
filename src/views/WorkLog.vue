<template>
  <div class="worklog-page">
    <!-- 左右分栏布局 -->
    <div class="worklog-container">
      <!-- 左侧编辑器区域 -->
      <div class="editor-section">
        <!-- 顶部标题和操作按钮 -->
        <div class="header-actions">
          <h2 class="page-title">工作日志</h2>
          <n-tabs v-model:value="logMode" type="segment" animated class="log-mode-tabs">
            <n-tab-pane name="daily" tab="日报" />
            <n-tab-pane name="weekly" tab="周报" />
            <n-tab-pane name="plan" tab="周计划" />
          </n-tabs>
          <div class="action-buttons">
            <!-- 日报模式的按钮 -->
            <template v-if="logMode === 'daily'">
              <n-button
                type="info"
                secondary
                @click="handleAiGenerate"
                :loading="aiGenerating"
                class="ai-button-primary"
              >
                <template #icon>
                  <n-icon>
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                    </svg>
                  </n-icon>
                </template>
                AI 生成日志
              </n-button>
              <n-button
                type="warning"
                secondary
                @click="handleAiPolish"
                :loading="aiPolishing"
                :disabled="!currentLog.trim()"
                class="ai-button-polish"
              >
                <template #icon>
                  <n-icon>
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                    </svg>
                  </n-icon>
                </template>
                AI 润色
              </n-button>
            </template>

            <!-- 周报模式的按钮 -->
            <template v-else-if="logMode === 'weekly'">
              <n-button
                type="success"
                secondary
                @click="handleGenerateWeekly"
                :loading="generatingWeekly"
                class="ai-button-weekly"
              >
                <template #icon>
                  <n-icon>
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M12 3v18m0-18l4 4m-4-4L8 7" />
                      <path d="M12 21l4-4m-4 4l-4-4" opacity="0.5" />
                    </svg>
                  </n-icon>
                </template>
                AI 生成周报
              </n-button>
            </template>

            <!-- 周计划模式的按钮 -->
            <template v-else-if="logMode === 'plan'">
              <n-button
                type="info"
                secondary
                @click="handleGeneratePlan"
                :loading="generatingPlan"
                class="ai-button-plan"
              >
                <template #icon>
                  <n-icon>
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2" />
                      <rect x="9" y="3" width="6" height="4" rx="1" />
                      <path d="M9 12h6M9 16h6" />
                    </svg>
                  </n-icon>
                </template>
                AI 生成计划
              </n-button>
              <n-button
                type="warning"
                secondary
                @click="handlePolishPlan"
                :loading="generatingPlan"
                :disabled="!currentPlanContent.trim()"
                class="ai-button-polish"
              >
                <template #icon>
                  <n-icon>
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                    </svg>
                  </n-icon>
                </template>
                AI 润色
              </n-button>
            </template>

            <n-button secondary @click="handleArchive">
              历史归档
            </n-button>
          </div>
        </div>

        <!-- 编辑器卡片 -->
        <div class="editor-card">
          <!-- 日报模式 -->
          <template v-if="logMode === 'daily'">
            <!-- 顶部：日期和图标 -->
            <div class="editor-header">
              <div class="date-info">
                <div class="icon-box">
                  <n-icon size="20" color="#a78bfa">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
                      <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
                    </svg>
                  </n-icon>
                </div>
                <div class="date-content">
                  <div class="current-date">{{ formatDateFull(selectedDate) }}</div>
                  <div class="date-subtitle">{{ getDateSubtitle() }}</div>
                </div>
              </div>
              <div class="header-right-actions">
                <n-button
                  text
                  @click="isPreviewMode = !isPreviewMode"
                  class="preview-toggle-button"
                  :disabled="!currentLog.trim()"
                >
                  <template #icon>
                    <n-icon size="20">
                      <svg v-if="!isPreviewMode" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                        <circle cx="12" cy="12" r="3" />
                      </svg>
                      <svg v-else xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                      </svg>
                    </n-icon>
                  </template>
                  {{ isPreviewMode ? '编辑' : '预览' }}
                </n-button>
                <n-date-picker
                  v-model:value="selectedDate"
                  type="date"
                  clearable
                  class="date-picker"
                />
                <n-button
                  v-if="!isToday(dayjs(selectedDate).format('YYYY-MM-DD'))"
                  text
                  @click="goToToday"
                  class="today-button"
                  title="回到今天"
                >
                  <template #icon>
                    <n-icon size="18">
                      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10" />
                        <polyline points="12 6 12 12 16 14" />
                      </svg>
                    </n-icon>
                  </template>
                  今天
                </n-button>
                <n-button
                  text
                  @click="handleSave"
                  :loading="saving"
                  class="save-button"
                >
                  <template #icon>
                    <n-icon size="20">
                      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
                        <polyline points="17 21 17 13 7 13 7 21" />
                        <polyline points="7 3 7 8 15 8" />
                      </svg>
                    </n-icon>
                  </template>
                </n-button>
              </div>
            </div>

            <!-- 中间：可扩展的textarea或预览区域 -->
            <n-spin :show="workLogStore.loading">
              <textarea
                v-if="!isPreviewMode"
                v-model="currentLog"
                class="editor-textarea"
                placeholder="今天做了什么？无论是写代码、开会还是摸鱼，记录下来吧..."
              />
              <div
                v-else
                class="markdown-preview"
                v-html="renderedContent"
              />
            </n-spin>

            <!-- 底部：标签列表 -->
            <div class="editor-footer">
              <div class="footer-label">标签</div>
              <div class="tags-container">
                <n-tag
                  v-for="tag in currentTags"
                  :key="tag"
                  :bordered="false"
                  class="log-tag"
                  closable
                  @close="handleRemoveTag(tag)"
                >
                  {{ tag }}
                </n-tag>
                <n-button
                  text
                  size="small"
                  @click="showAddTag = true"
                  class="add-tag-btn"
                >
                  + 添加标签
                </n-button>
              </div>
            </div>
          </template>

          <!-- 周报模式 -->
          <template v-else-if="logMode === 'weekly'">
            <!-- 顶部：周选择器 -->
            <div class="editor-header">
              <div class="date-info">
                <div class="icon-box">
                  <n-icon size="20" color="#10b981">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
                      <line x1="16" y1="2" x2="16" y2="6" />
                      <line x1="8" y1="2" x2="8" y2="6" />
                      <line x1="3" y1="10" x2="21" y2="10" />
                    </svg>
                  </n-icon>
                </div>
                <div class="date-content">
                  <div class="current-date">{{ weekRangeText }}</div>
                  <div class="date-subtitle">周报视图 · 任务汇总</div>
                </div>
              </div>
              <div class="header-right-actions">
                <n-button
                  text
                  @click="isPreviewMode = !isPreviewMode"
                  class="preview-toggle-button"
                  :disabled="!currentLog.trim()"
                >
                  <template #icon>
                    <n-icon size="20">
                      <svg v-if="!isPreviewMode" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                        <circle cx="12" cy="12" r="3" />
                      </svg>
                      <svg v-else xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                      </svg>
                    </n-icon>
                  </template>
                  {{ isPreviewMode ? '编辑' : '预览' }}
                </n-button>
                <n-select
                  v-model:value="selectedWeekType"
                  :options="weekOptions"
                  class="week-selector"
                  @update:value="handleWeekChange"
                />
                <n-button
                  text
                  @click="handleSave"
                  :loading="saving"
                  class="save-button"
                >
                  <template #icon>
                    <n-icon size="20">
                      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
                        <polyline points="17 21 17 13 7 13 7 21" />
                        <polyline points="7 3 7 8 15 8" />
                      </svg>
                    </n-icon>
                  </template>
                </n-button>
              </div>
            </div>

            <!-- 中间：周任务列表和编辑区 -->
            <n-spin :show="workLogStore.loading || loadingWeeklyTasks">
              <div class="weekly-content">
                <!-- 左侧：完成的任务列表 - 始终显示 -->
                <div class="weekly-tasks">
                  <h3 class="section-title">
                    本周完成的任务 ({{ weeklyTasks.length }})
                    <n-tag v-if="newCompletedCount > 0 && weeklyPlanForReport" size="tiny" type="info" :bordered="false" class="new-tag">
                      +{{ newCompletedCount }} 新增
                    </n-tag>
                  </h3>
                  <!-- 计划完成率提示 -->
                  <div v-if="planCompletionRate" class="plan-progress">
                    <div class="progress-info">
                      <span>计划完成: {{ planCompletionRate.completed }}/{{ planCompletionRate.total }}</span>
                      <span class="progress-percent">{{ planCompletionRate.percent }}%</span>
                    </div>
                    <n-progress
                      type="line"
                      :percentage="planCompletionRate.percent"
                      :height="4"
                      :show-indicator="false"
                      :color="planCompletionRate.percent >= 80 ? '#10b981' : planCompletionRate.percent >= 50 ? '#f59e0b' : '#ef4444'"
                    />
                  </div>
                  <div v-if="weeklyTasks.length > 0" class="task-list">
                    <div
                      v-for="task in weeklyTasks"
                      :key="task.id"
                      class="task-item-simple"
                      :class="{ 'task-new': isNewTask(task.id) && weeklyPlanForReport }"
                    >
                      <div class="task-row">
                        <n-icon size="14" color="#10b981" class="task-check-icon">
                          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <polyline points="20 6 9 17 4 12" />
                          </svg>
                        </n-icon>
                        <span class="task-title-simple">{{ task.title }}</span>
                        <n-tag v-if="isNewTask(task.id) && weeklyPlanForReport" size="tiny" type="info" :bordered="false" class="new-badge">新增</n-tag>
                        <div class="task-tags-simple" v-if="task.tags && task.tags.length > 0">
                          <n-tag
                            v-for="tag in task.tags.slice(0, 2)"
                            :key="tag.id"
                            size="tiny"
                            :bordered="false"
                            :style="{ background: tag.color + '20', color: tag.color }"
                            class="task-tag-simple"
                          >
                            {{ tag.name }}
                          </n-tag>
                        </div>
                        <span class="task-time-simple">{{ formatTaskTime(task.completedAt!) }}</span>
                      </div>
                    </div>
                  </div>
                  <n-empty
                    v-else
                    description="本周暂无完成的任务"
                    size="small"
                    class="empty-tasks"
                  />
                </div>

                <!-- 右侧：周报编辑器/预览 -->
                <div class="weekly-editor">
                  <h3 class="section-title">周报内容</h3>
                  <textarea
                    v-if="!isPreviewMode"
                    v-model="currentLog"
                    class="editor-textarea weekly-textarea"
                    placeholder="点击'AI生成周报'按钮，基于本周完成的任务自动生成周报总结..."
                  />
                  <div
                    v-else
                    class="markdown-preview weekly-preview"
                    v-html="renderedContent"
                  />
                </div>
              </div>
            </n-spin>

            <!-- 底部：统计信息 -->
            <div class="editor-footer">
              <div class="footer-label">统计</div>
              <div class="stats-container">
                <div class="stat-item">
                  <span class="stat-label">完成任务:</span>
                  <span class="stat-value">{{ weeklyTasks.length }} 个</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">时间范围:</span>
                  <span class="stat-value">{{ weekRangeText }}</span>
                </div>
              </div>
            </div>
          </template>

          <!-- 周计划模式 -->
          <template v-else-if="logMode === 'plan'">
            <!-- 顶部：周范围选择和操作 -->
            <div class="editor-header">
              <div class="date-info">
                <div class="icon-box plan-icon-box">
                  <n-icon size="20" color="#3b82f6">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2" />
                      <rect x="9" y="3" width="6" height="4" rx="1" />
                      <path d="M9 12h6M9 16h6" />
                    </svg>
                  </n-icon>
                </div>
                <div class="date-content">
                  <div class="current-date">{{ planWeekRangeText }}</div>
                  <div class="date-subtitle">周计划 · 工作安排</div>
                </div>
              </div>
              <div class="header-right-actions">
                <n-button
                  text
                  @click="isPreviewMode = !isPreviewMode"
                  class="preview-toggle-button"
                  :disabled="!currentPlanContent.trim()"
                >
                  <template #icon>
                    <n-icon size="20">
                      <svg v-if="!isPreviewMode" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                        <circle cx="12" cy="12" r="3" />
                      </svg>
                      <svg v-else xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                      </svg>
                    </n-icon>
                  </template>
                  {{ isPreviewMode ? '编辑' : '预览' }}
                </n-button>
                <n-select
                  v-model:value="selectedPlanWeekType"
                  :options="planWeekOptions"
                  class="week-selector"
                  @update:value="handlePlanWeekChange"
                />
                <n-button
                  text
                  @click="handleSavePlan"
                  :loading="savingPlan"
                  class="save-button"
                >
                  <template #icon>
                    <n-icon size="20">
                      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
                        <polyline points="17 21 17 13 7 13 7 21" />
                        <polyline points="7 3 7 8 15 8" />
                      </svg>
                    </n-icon>
                  </template>
                </n-button>
              </div>
            </div>

            <!-- 中间：任务列表和编辑区 -->
            <n-spin :show="weeklyPlanStore.loading || loadingPlanTasks">
              <div class="weekly-content">
                <!-- 左侧：待办任务列表 -->
                <div class="weekly-tasks">
                  <h3 class="section-title">计划任务 ({{ planTasks.length }})</h3>
                  <div v-if="planTasks.length > 0" class="task-list">
                    <div
                      v-for="task in planTasks"
                      :key="task.id"
                      class="task-item-simple"
                    >
                      <div class="task-row">
                        <n-icon size="14" color="#3b82f6" class="task-check-icon">
                          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="10" />
                            <path d="M12 6v6l4 2" />
                          </svg>
                        </n-icon>
                        <span class="task-title-simple">{{ task.title }}</span>
                        <div class="task-tags-simple" v-if="task.tags && task.tags.length > 0">
                          <n-tag
                            v-for="tag in task.tags.slice(0, 2)"
                            :key="tag.id"
                            size="tiny"
                            :bordered="false"
                            :style="{ background: tag.color + '20', color: tag.color }"
                            class="task-tag-simple"
                          >
                            {{ tag.name }}
                          </n-tag>
                        </div>
                        <span class="task-time-simple" v-if="task.dueDate">截止: {{ dayjs(task.dueDate).format('MM-DD') }}</span>
                      </div>
                    </div>
                  </div>
                  <n-empty
                    v-else
                    description="该周暂无待办任务"
                    size="small"
                    class="empty-tasks"
                  >
                    <template #extra>
                      <span class="empty-hint">请在任务看板中设置任务的截止日期</span>
                    </template>
                  </n-empty>
                </div>

                <!-- 右侧：周计划编辑器/预览 -->
                <div class="weekly-editor">
                  <h3 class="section-title">计划内容</h3>
                  <textarea
                    v-if="!isPreviewMode"
                    v-model="currentPlanContent"
                    class="editor-textarea weekly-textarea"
                    placeholder="点击'AI生成计划'按钮，基于待办任务自动生成工作计划..."
                  />
                  <div
                    v-else
                    class="markdown-preview weekly-preview"
                    v-html="renderedPlanContent"
                  />
                </div>
              </div>
            </n-spin>

            <!-- 底部：统计信息 -->
            <div class="editor-footer">
              <div class="footer-label">统计</div>
              <div class="stats-container">
                <div class="stat-item">
                  <span class="stat-label">计划任务:</span>
                  <span class="stat-value">{{ planTasks.length }} 个</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">时间范围:</span>
                  <span class="stat-value">{{ planWeekRangeText }}</span>
                </div>
              </div>
            </div>
          </template>
        </div>
      </div>

      <!-- 右侧时间轴（大屏显示） -->
      <div class="timeline-section">
        <h3 class="timeline-title">
          {{ logMode === 'daily' ? '日报记录' : (logMode === 'weekly' ? '周报记录' : '周计划记录') }}
        </h3>
        <div class="timeline-container">
          <!-- 垂直线 -->
          <div class="timeline-line" />

          <!-- 日报时间节点 -->
          <template v-if="logMode === 'daily'">
            <div
              v-for="log in dailyLogs"
              :key="log.date"
              class="timeline-item"
              :class="{ 'timeline-item-active': isSelectedDate(log.date) }"
              @click="selectDailyLog(log)"
            >
              <div
                class="timeline-dot"
                :class="{
                  'dot-active': isSelectedDate(log.date),
                  'dot-default': !isSelectedDate(log.date)
                }"
              />
              <div class="timeline-content">
                <div class="timeline-date">{{ formatDate(log.date) }}</div>
                <div class="timeline-text">
                  {{ truncate(log.content, 60) || '暂无内容' }}
                </div>
              </div>
            </div>
            <div v-if="dailyLogs.length === 0" class="timeline-empty">
              <n-empty description="暂无日报记录" size="small" />
            </div>
          </template>

          <!-- 周报时间节点 -->
          <template v-else-if="logMode === 'weekly'">
            <div
              v-for="log in weeklyLogs"
              :key="log.date"
              class="timeline-item"
              :class="{ 'timeline-item-active': isSelectedWeek(log.date) }"
              @click="selectWeeklyLog(log)"
            >
              <div
                class="timeline-dot"
                :class="{
                  'dot-active': isSelectedWeek(log.date),
                  'dot-default': !isSelectedWeek(log.date)
                }"
              />
              <div class="timeline-content">
                <div class="timeline-date">{{ formatWeekDate(log.date) }}</div>
                <div class="timeline-text">
                  {{ truncate(log.content, 60) || '暂无内容' }}
                </div>
              </div>
            </div>
            <div v-if="weeklyLogs.length === 0" class="timeline-empty">
              <n-empty description="暂无周报记录" size="small" />
            </div>
          </template>

          <!-- 周计划时间节点 -->
          <template v-else-if="logMode === 'plan'">
            <div
              v-for="plan in weeklyPlanStore.allPlans"
              :key="plan.weekKey"
              class="timeline-item"
              :class="{ 'timeline-item-active': plan.weekKey === planWeekKey }"
              @click="selectWeeklyPlan(plan)"
            >
              <div
                class="timeline-dot"
                :class="{
                  'dot-active': plan.weekKey === planWeekKey,
                  'dot-default': plan.weekKey !== planWeekKey
                }"
              />
              <div class="timeline-content">
                <div class="timeline-date">
                  {{ formatPlanWeekDate(plan.weekKey) }}
                  <n-tag v-if="plan.status === 'confirmed'" size="tiny" type="success" :bordered="false">已确认</n-tag>
                </div>
                <div class="timeline-text">
                  {{ truncate(plan.content, 60) || '暂无内容' }}
                </div>
              </div>
            </div>
            <div v-if="weeklyPlanStore.allPlans.length === 0" class="timeline-empty">
              <n-empty description="暂无周计划记录" size="small" />
            </div>
          </template>
        </div>
      </div>
    </div>

    <!-- 添加标签对话框 -->
    <n-modal v-model:show="showAddTag" preset="dialog" title="添加标签">
      <n-input
        v-model:value="newTag"
        placeholder="输入标签名称，如 #Frontend"
        @keyup.enter="handleAddTag"
      />
      <template #action>
        <n-button @click="showAddTag = false">取消</n-button>
        <n-button class="primary-button" @click="handleAddTag">添加</n-button>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, onBeforeUnmount, computed } from 'vue';
import {
  NButton,
  NDatePicker,
  NSpin,
  NTag,
  NEmpty,
  NModal,
  NInput,
  NIcon,
  NTabs,
  NTabPane,
  NSelect,
  useMessage,
  useDialog
} from 'naive-ui';
import dayjs from 'dayjs';
import isoWeek from 'dayjs/plugin/isoWeek';
import 'dayjs/locale/zh-cn';
import { marked } from 'marked';

dayjs.extend(isoWeek);
dayjs.locale('zh-cn'); // 设置为中文（周一为一周的开始）

// 生成 ISO 周格式的 key，如 2025-W49
function getIsoWeekKey(timestamp: number): string {
  const d = dayjs(timestamp);
  const year = d.isoWeekYear();
  const week = d.isoWeek();
  return `${year}-W${week.toString().padStart(2, '0')}`;
}
import { useWorkLogStore } from '@/stores/workLogStore';
import { useWeeklyPlanStore } from '@/stores/weeklyPlanStore';
import { useTaskStore } from '@/stores/taskStore';
import { aiApi } from '@/api/aiApi';
import { promptApi } from '@/api/promptApi';
import type { WorkLog, WeeklyPlan } from '@/types/workLog';
import type { Task } from '@/types/task';
import { CATEGORY_LABELS } from '@/types/task';

const message = useMessage();
const dialog = useDialog();
const workLogStore = useWorkLogStore();
const weeklyPlanStore = useWeeklyPlanStore();
const taskStore = useTaskStore();

// 日志模式：日报 / 周报 / 周计划
const logMode = ref<'daily' | 'weekly' | 'plan'>('daily');

// 日报相关状态
const selectedDate = ref<number>(Date.now());
const currentLog = ref('');
const currentTags = ref<string[]>(['#Rust', '#Frontend', '#Bugfix']);
const saving = ref(false);
const aiGenerating = ref(false);
const aiPolishing = ref(false);
const generatingWeekly = ref(false);
const showAddTag = ref(false);
const newTag = ref('');
const isPreviewMode = ref(false);

// 周报相关状态
const selectedWeekType = ref<string>('this_week');
const weekRange = ref<[number, number]>([0, 0]);
const weeklyTasks = ref<Task[]>([]);
const loadingWeeklyTasks = ref(false);
const weeklyPlanForReport = ref<WeeklyPlan | null>(null);  // 当前周的计划（用于对比）

// 周计划相关状态
const selectedPlanWeekType = ref<string>('next_week');
const planWeekRange = ref<[number, number]>([0, 0]);
const planTasks = ref<Task[]>([]);  // 下周截止的任务
const loadingPlanTasks = ref(false);
const currentPlanContent = ref('');  // 周计划内容
const generatingPlan = ref(false);  // AI 生成周计划中
const savingPlan = ref(false);  // 保存周计划中

// 周选择选项
const weekOptions = [
  { label: '本周', value: 'this_week' },
  { label: '上周', value: 'last_week' },
  { label: '前两周', value: 'two_weeks_ago' },
  { label: '前三周', value: 'three_weeks_ago' }
];

// 周计划选择选项
const planWeekOptions = [
  { label: '下周', value: 'next_week' },
  { label: '本周', value: 'this_week' },
  { label: '下两周', value: 'two_weeks_later' }
];

// 将 markdown 转换为 HTML
const renderedContent = computed(() => {
  if (!currentLog.value) return '';
  try {
    return marked(currentLog.value);
  } catch (error) {
    console.error('Markdown 渲染失败:', error);
    return currentLog.value;
  }
});

// 周范围文本显示
const weekRangeText = computed(() => {
  if (!weekRange.value || weekRange.value[0] === 0) {
    return '选择周范围';
  }
  const start = dayjs(weekRange.value[0]).format('MM月DD日');
  const end = dayjs(weekRange.value[1]).format('MM月DD日');
  return `${start} - ${end}`;
});

// 周计划范围文本显示
const planWeekRangeText = computed(() => {
  if (!planWeekRange.value || planWeekRange.value[0] === 0) {
    return '选择周范围';
  }
  const start = dayjs(planWeekRange.value[0]).format('MM月DD日');
  const end = dayjs(planWeekRange.value[1]).format('MM月DD日');
  return `${start} - ${end}`;
});

// 周计划内容渲染
const renderedPlanContent = computed(() => {
  if (!currentPlanContent.value) return '';
  try {
    return marked(currentPlanContent.value);
  } catch (error) {
    console.error('Markdown 渲染失败:', error);
    return currentPlanContent.value;
  }
});

// 获取周计划周的 key（格式：2025-W48，使用 ISO 周格式）
const planWeekKey = computed(() => {
  if (!planWeekRange.value || planWeekRange.value[0] === 0) return '';
  return getIsoWeekKey(planWeekRange.value[0]);
});

// 按类型过滤日志
const dailyLogs = computed(() => {
  return workLogStore.recentLogs.filter(log => {
    // 日报日期格式: YYYY-MM-DD
    return /^\d{4}-\d{2}-\d{2}$/.test(log.date);
  });
});

const weeklyLogs = computed(() => {
  return workLogStore.recentLogs.filter(log => {
    // 周报日期格式: GGGG-WWW (如 2025-W49) 或兼容旧格式 YYYY-XX
    return /^\d{4}-W\d{2}$/.test(log.date) || /^\d{4}-\d{2}$/.test(log.date);
  });
});

// 获取当前周报对应的周计划中的任务ID
const plannedTaskIds = computed(() => {
  if (!weeklyPlanForReport.value) return new Set<number>();
  return new Set(weeklyPlanForReport.value.taskIds);
});

// 判断任务是否在计划中
function isPlannedTask(taskId: number | undefined): boolean {
  if (!taskId) return false;
  return plannedTaskIds.value.has(taskId);
}

// 判断任务是否为新增（不在原始计划中）
function isNewTask(taskId: number | undefined): boolean {
  if (!taskId) return true;
  return !plannedTaskIds.value.has(taskId);
}

// 计划完成率
const planCompletionRate = computed(() => {
  if (!weeklyPlanForReport.value || weeklyPlanForReport.value.taskIds.length === 0) {
    return null;
  }
  const plannedIds = new Set(weeklyPlanForReport.value.taskIds);
  const completedPlannedTasks = weeklyTasks.value.filter(task => task.id && plannedIds.has(task.id));
  return {
    completed: completedPlannedTasks.length,
    total: weeklyPlanForReport.value.taskIds.length,
    percent: Math.round((completedPlannedTasks.length / weeklyPlanForReport.value.taskIds.length) * 100)
  };
});

// 新增完成的任务数量
const newCompletedCount = computed(() => {
  if (!weeklyPlanForReport.value) return weeklyTasks.value.length;
  const plannedIds = new Set(weeklyPlanForReport.value.taskIds);
  return weeklyTasks.value.filter(task => !task.id || !plannedIds.has(task.id)).length;
});

// LocalStorage 键名
const DRAFT_STORAGE_KEY = 'worklog_draft';
const TAGS_STORAGE_KEY = 'worklog_tags';
const PREVIEW_MODE_KEY = 'worklog_preview_mode';

// 保存草稿到 LocalStorage
function saveDraft() {
  const dateStr = dayjs(selectedDate.value).format('YYYY-MM-DD');
  const draft = {
    date: dateStr,
    content: currentLog.value,
    tags: currentTags.value,
    timestamp: Date.now()
  };
  localStorage.setItem(DRAFT_STORAGE_KEY, JSON.stringify(draft));
}

// 从 LocalStorage 加载草稿
function loadDraft() {
  try {
    const draftStr = localStorage.getItem(DRAFT_STORAGE_KEY);
    if (draftStr) {
      const draft = JSON.parse(draftStr);
      const dateStr = dayjs(selectedDate.value).format('YYYY-MM-DD');

      // 只有当日期匹配时才加载草稿
      if (draft.date === dateStr && draft.content) {
        // 如果当前没有内容,使用草稿
        if (!currentLog.value) {
          currentLog.value = draft.content;
          if (draft.tags && draft.tags.length > 0) {
            currentTags.value = draft.tags;
          }
        }
      }
    }
  } catch (error) {
    console.error('加载草稿失败:', error);
  }
}

// 清除草稿
function clearDraft() {
  localStorage.removeItem(DRAFT_STORAGE_KEY);
}

// 计算周范围
function calculateWeekRange(weekType: string): [number, number] {
  const now = dayjs();
  let startOfWeek: dayjs.Dayjs;
  let endOfWeek: dayjs.Dayjs;

  switch (weekType) {
    case 'this_week':
      startOfWeek = now.startOf('week');
      endOfWeek = now.endOf('week');
      break;
    case 'last_week':
      startOfWeek = now.subtract(1, 'week').startOf('week');
      endOfWeek = now.subtract(1, 'week').endOf('week');
      break;
    case 'two_weeks_ago':
      startOfWeek = now.subtract(2, 'week').startOf('week');
      endOfWeek = now.subtract(2, 'week').endOf('week');
      break;
    case 'three_weeks_ago':
      startOfWeek = now.subtract(3, 'week').startOf('week');
      endOfWeek = now.subtract(3, 'week').endOf('week');
      break;
    default:
      startOfWeek = now.startOf('week');
      endOfWeek = now.endOf('week');
  }

  return [startOfWeek.valueOf(), endOfWeek.valueOf()];
}

// 加载周任务
async function loadWeeklyTasks() {
  if (!weekRange.value || weekRange.value[0] === 0) return;

  loadingWeeklyTasks.value = true;
  try {
    // 加载已完成的任务
    await taskStore.loadCompletedTasks(30); // 加载最近30天的完成任务

    // 筛选周范围内的任务
    weeklyTasks.value = taskStore.completedTasks.filter(task => {
      if (!task.completedAt) return false;
      const completedTime = new Date(task.completedAt).getTime();
      return completedTime >= weekRange.value[0] && completedTime <= weekRange.value[1];
    });

    // 加载当前周的计划（用于周报对比）
    const weekKey = getIsoWeekKey(weekRange.value[0]);
    await weeklyPlanStore.loadWeeklyPlan(weekKey);
    weeklyPlanForReport.value = weeklyPlanStore.currentPlan;
  } catch (error) {
    console.error('加载周任务失败:', error);
    message.error('加载周任务失败');
  } finally {
    loadingWeeklyTasks.value = false;
  }
}

// 处理周选择变化
async function handleWeekChange(value: string) {
  weekRange.value = calculateWeekRange(value);
  await loadWeeklyTasks();

  // 尝试加载已保存的周报
  const weekKey = getIsoWeekKey(weekRange.value[0]);
  await workLogStore.loadWorkLog(weekKey);
  currentLog.value = workLogStore.currentLog?.content || '';
}

// 计算周计划的周范围（下周、本周、下两周）
function calculatePlanWeekRange(weekType: string): [number, number] {
  const now = dayjs();
  let startOfWeek: dayjs.Dayjs;
  let endOfWeek: dayjs.Dayjs;

  switch (weekType) {
    case 'next_week':
      startOfWeek = now.add(1, 'week').startOf('week');
      endOfWeek = now.add(1, 'week').endOf('week');
      break;
    case 'this_week':
      startOfWeek = now.startOf('week');
      endOfWeek = now.endOf('week');
      break;
    case 'two_weeks_later':
      startOfWeek = now.add(2, 'week').startOf('week');
      endOfWeek = now.add(2, 'week').endOf('week');
      break;
    default:
      startOfWeek = now.add(1, 'week').startOf('week');
      endOfWeek = now.add(1, 'week').endOf('week');
  }

  return [startOfWeek.valueOf(), endOfWeek.valueOf()];
}

// 加载周计划任务（截止日期在指定周范围内的待办任务）
async function loadPlanTasks() {
  if (!planWeekRange.value || planWeekRange.value[0] === 0) return;

  loadingPlanTasks.value = true;
  try {
    // 加载所有待办和进行中的任务
    await taskStore.loadTasks();

    const startDate = dayjs(planWeekRange.value[0]).format('YYYY-MM-DD');
    const endDate = dayjs(planWeekRange.value[1]).format('YYYY-MM-DD');

    // 筛选截止日期在周范围内的任务（待办或进行中）
    planTasks.value = taskStore.tasks.filter(task => {
      if (task.status === 'done') return false;  // 排除已完成的
      if (!task.dueDate) return false;  // 必须有截止日期
      return task.dueDate >= startDate && task.dueDate <= endDate;
    });
  } catch (error) {
    console.error('加载周计划任务失败:', error);
    message.error('加载周计划任务失败');
  } finally {
    loadingPlanTasks.value = false;
  }
}

// 处理周计划周选择变化
async function handlePlanWeekChange(value: string) {
  planWeekRange.value = calculatePlanWeekRange(value);
  await loadPlanTasks();

  // 尝试加载已保存的周计划
  const weekKey = getIsoWeekKey(planWeekRange.value[0]);
  await weeklyPlanStore.loadWeeklyPlan(weekKey);
  currentPlanContent.value = weeklyPlanStore.currentPlan?.content || '';
}

// 保存周计划
async function handleSavePlan() {
  if (!currentPlanContent.value.trim()) {
    message.warning('周计划内容不能为空');
    return;
  }

  savingPlan.value = true;
  try {
    const taskIds = planTasks.value.map(t => t.id!).filter(id => id !== undefined);
    await weeklyPlanStore.saveWeeklyPlan(
      planWeekKey.value,
      currentPlanContent.value,
      taskIds,
      'draft'
    );
    message.success('周计划已保存');
  } catch (error) {
    console.error('保存周计划失败:', error);
    message.error('保存周计划失败');
  } finally {
    savingPlan.value = false;
  }
}

// AI 生成周计划
async function handleGeneratePlan() {
  if (planTasks.value.length === 0) {
    message.warning('没有找到截止日期在该周的任务，请先在任务看板中添加任务并设置截止日期');
    return;
  }

  generatingPlan.value = true;
  try {
    // 构建任务摘要
    const taskSummary = planTasks.value.map(task => {
      const dueDate = task.dueDate ? dayjs(task.dueDate).format('MM-DD') : '未设置';
      const category = getCategoryLabel(task.category);
      return `[截止:${dueDate}] [${category}] ${task.title}${task.description ? ': ' + task.description : ''}`;
    }).join('\n');

    // 从 API 获取提示词并渲染
    const rendered = await promptApi.renderPromptPreview('weekly_plan_generate', {
      task_summary: taskSummary
    });
    const prompt = rendered.user;

    const result = await aiApi.chat(prompt, 'weekly_plan');
    currentPlanContent.value = result;
    message.success('周计划已生成');
  } catch (error) {
    console.error('生成周计划失败:', error);
    message.error('生成周计划失败，请检查 AI 配置');
  } finally {
    generatingPlan.value = false;
  }
}

// AI 润色周计划
async function handlePolishPlan() {
  if (!currentPlanContent.value.trim()) {
    message.warning('请先填写周计划内容');
    return;
  }

  generatingPlan.value = true;
  try {
    // 从 API 获取提示词并渲染
    const rendered = await promptApi.renderPromptPreview('weekly_plan_polish', {
      content: currentPlanContent.value
    });
    const prompt = rendered.user;

    const result = await aiApi.chat(prompt, 'weekly_plan');
    currentPlanContent.value = result;
    message.success('周计划已润色');
  } catch (error) {
    console.error('润色周计划失败:', error);
    message.error('润色周计划失败，请检查 AI 配置');
  } finally {
    generatingPlan.value = false;
  }
}

// 获取分类标签
function getCategoryLabel(category: Task['category']): string {
  return CATEGORY_LABELS[category] || category;
}

// 格式化任务完成时间
function formatTaskTime(time: string): string {
  return dayjs(time).format('MM-DD HH:mm');
}

onMounted(async () => {
  await loadLogs();
  // 加载草稿
  loadDraft();

  // 初始化周范围
  weekRange.value = calculateWeekRange('this_week');
  await loadWeeklyTasks();

  // 初始化周计划范围
  planWeekRange.value = calculatePlanWeekRange('next_week');
  await loadPlanTasks();

  // 加载周计划列表
  await weeklyPlanStore.loadAllPlans();

  // 加载缓存的预览模式状态，如果没有缓存且有内容则默认预览模式
  const cachedPreviewMode = localStorage.getItem(PREVIEW_MODE_KEY);
  if (cachedPreviewMode !== null) {
    isPreviewMode.value = cachedPreviewMode === 'true';
  } else if (currentLog.value.trim()) {
    // 如果没有缓存但有内容，默认进入预览模式
    isPreviewMode.value = true;
  }
});

// 监听日志模式变化
watch(logMode, async (newMode) => {
  // 清空当前内容
  currentLog.value = '';
  currentPlanContent.value = '';
  isPreviewMode.value = false;

  if (newMode === 'weekly') {
    // 切换到周报模式，加载周任务
    await loadWeeklyTasks();
    // 尝试加载已保存的周报
    const weekKey = getIsoWeekKey(weekRange.value[0]);
    await workLogStore.loadWorkLog(weekKey);
    currentLog.value = workLogStore.currentLog?.content || '';
  } else if (newMode === 'plan') {
    // 切换到周计划模式，加载下周任务
    await loadPlanTasks();
    // 尝试加载已保存的周计划
    const weekKey = getIsoWeekKey(planWeekRange.value[0]);
    await weeklyPlanStore.loadWeeklyPlan(weekKey);
    currentPlanContent.value = weeklyPlanStore.currentPlan?.content || '';
  } else {
    // 切换到日报模式，加载当前日期的日志
    const dateStr = dayjs(selectedDate.value).format('YYYY-MM-DD');
    await workLogStore.loadWorkLog(dateStr);
    currentLog.value = workLogStore.currentLog?.content || '';
    loadDraft();
  }
});

// 监听日期变化,自动加载对应日志 (仅在日报模式下)
watch(selectedDate, async (newDate) => {
  if (newDate && logMode.value === 'daily') {
    const dateStr = dayjs(newDate).format('YYYY-MM-DD');
    await workLogStore.loadWorkLog(dateStr);
    currentLog.value = workLogStore.currentLog?.content || '';
    // 加载草稿
    loadDraft();
    // 如果有内容，默认进入预览模式（除非用户已有缓存设置）
    const cachedPreviewMode = localStorage.getItem(PREVIEW_MODE_KEY);
    if (cachedPreviewMode === null && currentLog.value.trim()) {
      isPreviewMode.value = true;
    }
  }
});

// 监听内容变化,自动保存草稿 (仅日报模式)
watch(currentLog, () => {
  if (currentLog.value && logMode.value === 'daily') {
    saveDraft();
  }
});

// 监听标签变化,自动保存草稿 (仅日报模式)
watch(currentTags, () => {
  if (logMode.value === 'daily') {
    saveDraft();
  }
}, { deep: true });

// 监听预览模式变化，保存到缓存
watch(isPreviewMode, (newValue) => {
  localStorage.setItem(PREVIEW_MODE_KEY, String(newValue));
});

// 页面卸载前保存草稿 (仅日报模式)
onBeforeUnmount(() => {
  if (currentLog.value && logMode.value === 'daily') {
    saveDraft();
  }
});

async function loadLogs() {
  try {
    // 加载所有日志（包括日报和周报）
    await workLogStore.loadAllLogs();

    // 加载当前选中日期的日志
    const dateStr = dayjs(selectedDate.value).format('YYYY-MM-DD');
    await workLogStore.loadWorkLog(dateStr);
    currentLog.value = workLogStore.currentLog?.content || '';
  } catch (error) {
    message.error('加载日志失败');
    console.error(error);
  }
}

async function handleSave() {
  if (!currentLog.value.trim()) {
    message.warning('日志内容不能为空');
    return;
  }

  saving.value = true;
  try {
    let dateStr: string;
    let logType: string;

    if (logMode.value === 'daily') {
      dateStr = dayjs(selectedDate.value).format('YYYY-MM-DD');
      logType = 'daily';
    } else {
      // 周报使用周的起始日期作为key，格式：2025-W49（ISO周格式）
      dateStr = getIsoWeekKey(weekRange.value[0]);
      logType = 'weekly';
    }

    await workLogStore.saveWorkLog(dateStr, logType, currentLog.value, false);
    message.success(`${logMode.value === 'daily' ? '日报' : '周报'}已保存`);

    // 清除草稿
    if (logMode.value === 'daily') {
      clearDraft();
    }

    // 重新加载所有日志
    await workLogStore.loadAllLogs();
  } catch (error) {
    message.error('保存失败');
    console.error(error);
  } finally {
    saving.value = false;
  }
}

// AI 生成工作日志
async function handleAiGenerate() {
  aiGenerating.value = true;
  try {
    // 检查 AI 是否启用
    const isEnabled = await aiApi.isEnabled();
    if (!isEnabled) {
      message.warning('请先在设置中配置并启用 AI 功能');
      return;
    }

    const dateStr = dayjs(selectedDate.value).format('YYYY-MM-DD');

    // 获取已完成任务（包括今天完成的和最近完成的）
    await taskStore.loadCompletedTasks(7);
    const todayStart = dayjs(selectedDate.value).startOf('day');
    const todayEnd = dayjs(selectedDate.value).endOf('day');

    // 只筛选当天完成的任务
    const todayCompletedTasks = taskStore.completedTasks.filter(task => {
      if (task.completedAt) {
        const completedTime = dayjs(task.completedAt);
        return completedTime.isAfter(todayStart) && completedTime.isBefore(todayEnd);
      }
      return false;
    });

    // 构建任务列表（只包含已完成的任务，简洁格式）
    const completedTasks: string[] = todayCompletedTasks.map(task => {
      // 只用任务标题，如果有标签也加上
      const tags = task.tags?.map(t => t.name).join('、') || '';
      return tags ? `${task.title}（${tags}）` : task.title;
    });

    const executedSqls: string[] = [];
    const gitCommits: string[] = [];

    message.loading('AI 正在生成日志...', { duration: 0 });

    const result = await aiApi.generateWorkLog(
      dateStr,
      completedTasks,
      executedSqls,
      gitCommits
    );

    message.destroyAll();
    currentLog.value = result;
    message.success('日志已生成');
  } catch (error: any) {
    message.destroyAll();
    message.error(error?.message || 'AI 生成失败');
    console.error(error);
  } finally {
    aiGenerating.value = false;
  }
}

// AI 润色工作日志
async function handleAiPolish() {
  if (!currentLog.value.trim()) {
    message.warning('请先输入日志内容');
    return;
  }

  aiPolishing.value = true;
  try {
    // 检查 AI 是否启用
    const isEnabled = await aiApi.isEnabled();
    if (!isEnabled) {
      message.warning('请先在设置中配置并启用 AI 功能');
      return;
    }

    message.loading('AI 正在润色日志...', { duration: 0 });

    const result = await aiApi.polishWorkLog(currentLog.value);

    message.destroyAll();
    currentLog.value = result;
    message.success('日志已润色');
  } catch (error: any) {
    message.destroyAll();
    message.error(error?.message || 'AI 润色失败');
    console.error(error);
  } finally {
    aiPolishing.value = false;
  }
}

// AI 生成周报
async function handleGenerateWeekly() {
  generatingWeekly.value = true;
  try {
    // 检查 AI 是否启用
    const isEnabled = await aiApi.isEnabled();
    if (!isEnabled) {
      message.warning('请先在设置中配置并启用 AI 功能');
      return;
    }

    // 检查是否有完成的任务
    if (weeklyTasks.value.length === 0) {
      message.warning('本周没有完成的任务，无法生成周报');
      return;
    }

    message.loading('AI 正在基于任务生成周报...', { duration: 0 });

    // 构建任务摘要
    const taskSummaries = weeklyTasks.value.map(task => {
      const category = getCategoryLabel(task.category);
      const time = dayjs(task.completedAt).format('MM-DD');
      return `[${time}] [${category}] ${task.title}${task.description ? `: ${task.description}` : ''}`;
    });

    // 调用AI生成周报
    const result = await aiApi.generateWeeklyReport(taskSummaries);

    message.destroyAll();
    currentLog.value = result;
    message.success('周报已生成');
  } catch (error: any) {
    message.destroyAll();
    message.error(error?.message || '周报生成失败');
    console.error(error);
  } finally {
    generatingWeekly.value = false;
  }
}

function handleArchive() {
  message.info('历史归档功能开发中');
}

// 选择日报记录
function selectDailyLog(log: WorkLog) {
  const date = new Date(log.date);
  if (!isNaN(date.getTime())) {
    selectedDate.value = date.getTime();
    currentLog.value = log.content;
  }
}

// 选择周报记录
function selectWeeklyLog(log: WorkLog) {
  // 周报日期格式: GGGG-WWW（如 2025-W49），无法直接转换为日期
  // 只更新内容，不更新日期选择器
  currentLog.value = log.content;

  // 尝试解析周数并设置周范围
  const match = log.date.match(/^(\d{4})-W?(\d{2})$/);
  if (match) {
    const year = parseInt(match[1]);
    const week = parseInt(match[2]);
    // 计算该周的起始日期
    const startOfYear = dayjs(`${year}-01-01`);
    const startOfWeek = startOfYear.add(week - 1, 'week').startOf('week');
    const endOfWeek = startOfWeek.endOf('week');
    weekRange.value = [startOfWeek.valueOf(), endOfWeek.valueOf()];
  }
}

// 检查是否为当前选中的日期
function isSelectedDate(date: string) {
  const logDate = dayjs(date);
  const selected = dayjs(selectedDate.value);
  return logDate.isSame(selected, 'day');
}

// 检查是否为当前选中的周
function isSelectedWeek(date: string) {
  const currentWeekKey = getIsoWeekKey(weekRange.value[0]);
  // 处理两种格式: 2025-W49 或 2025-49（兼容旧格式）
  const normalizedDate = date.includes('W') ? date : date.replace(/^(\d{4})-(\d{2})$/, '$1-W$2');
  return normalizedDate === currentWeekKey || date === currentWeekKey.replace('W', '');
}

// 格式化周报日期显示
function formatWeekDate(date: string) {
  // 周报日期格式: GGGG-WWW (如 2025-W49) 或兼容旧格式 YYYY-XX
  const match = date.match(/^(\d{4})-W?(\d{2})$/);
  if (match) {
    return `${match[1]}年 第${match[2]}周`;
  }
  return date;
}

// 选择周计划记录
function selectWeeklyPlan(plan: WeeklyPlan) {
  currentPlanContent.value = plan.content;

  // 尝试解析周数并设置周范围
  const match = plan.weekKey.match(/^(\d{4})-W(\d{2})$/);
  if (match) {
    const year = parseInt(match[1]);
    const week = parseInt(match[2]);
    // 计算该周的起始日期
    const startOfYear = dayjs(`${year}-01-01`);
    const startOfWeek = startOfYear.add(week - 1, 'week').startOf('week');
    const endOfWeek = startOfWeek.endOf('week');
    planWeekRange.value = [startOfWeek.valueOf(), endOfWeek.valueOf()];
  }
}

// 格式化周计划日期显示
function formatPlanWeekDate(weekKey: string) {
  // 周计划日期格式: YYYY-WXX
  const match = weekKey.match(/^(\d{4})-W(\d{2})$/);
  if (match) {
    return `${match[1]}年 第${match[2]}周`;
  }
  return weekKey;
}

function handleAddTag() {
  if (!newTag.value.trim()) {
    return;
  }
  const tag = newTag.value.startsWith('#') ? newTag.value : `#${newTag.value}`;
  if (!currentTags.value.includes(tag)) {
    currentTags.value.push(tag);
  }
  newTag.value = '';
  showAddTag.value = false;
}

function handleRemoveTag(tag: string) {
  const index = currentTags.value.indexOf(tag);
  if (index > -1) {
    currentTags.value.splice(index, 1);
  }
}

function formatDateFull(timestamp: number) {
  return dayjs(timestamp).format('YYYY年 MM月 DD日');
}

function formatDate(date: string) {
  return dayjs(date).format('YYYY-MM-DD');
}

function getDateSubtitle() {
  const day = dayjs(selectedDate.value).format('dddd');
  const dayMap: { [key: string]: string } = {
    'Monday': '星期一',
    'Tuesday': '星期二',
    'Wednesday': '星期三',
    'Thursday': '星期四',
    'Friday': '星期五',
    'Saturday': '星期六',
    'Sunday': '星期日'
  };
  return `${dayMap[day] || day} · 此时此刻`;
}

function isToday(date: string) {
  return dayjs(date).isSame(dayjs(), 'day');
}

// 跳转到今天
function goToToday() {
  selectedDate.value = Date.now();
}

function truncate(text: string, length: number) {
  if (!text) return '';
  return text.length > length ? text.substring(0, length) + '...' : text;
}
</script>

<style scoped>
.worklog-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-base);
  overflow: hidden;
}

.worklog-container {
  flex: 1;
  display: flex;
  gap: 24px;
  padding: 32px;
  padding-top: 16px;
  min-height: 0;
}

/* 左侧编辑器区域 */
.editor-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.header-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  gap: 16px;
}

.page-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
  margin: 0;
}

.log-mode-tabs {
  flex: 1;
  max-width: 300px;
}

.action-buttons {
  display: flex;
  gap: 12px;
}

.ai-button-primary {
  --n-color: color-mix(in srgb, var(--accent-primary) 10%, transparent) !important;
  --n-color-hover: color-mix(in srgb, var(--accent-primary) 20%, transparent) !important;
  --n-text-color: var(--accent-primary) !important;
  --n-border: 1px solid color-mix(in srgb, var(--accent-primary) 20%, transparent) !important;
}

.ai-button-polish {
  --n-color: color-mix(in srgb, var(--warning) 10%, transparent) !important;
  --n-color-hover: color-mix(in srgb, var(--warning) 20%, transparent) !important;
  --n-text-color: var(--warning) !important;
  --n-border: 1px solid color-mix(in srgb, var(--warning) 20%, transparent) !important;
}

.ai-button-weekly {
  --n-color: color-mix(in srgb, var(--success) 10%, transparent) !important;
  --n-color-hover: color-mix(in srgb, var(--success) 20%, transparent) !important;
  --n-text-color: var(--success) !important;
  --n-border: 1px solid color-mix(in srgb, var(--success) 20%, transparent) !important;
}

.editor-card {
  flex: 1;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 20px;
  padding: 24px;
  backdrop-filter: blur(12px);
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  min-height: 0;
  overflow: hidden;
  position: relative;
  transition: all 0.3s ease;
}

.editor-card:focus-within {
  border-color: var(--accent-primary);
  box-shadow: 0 8px 16px -4px rgba(0, 0, 0, 0.2), 0 0 0 1px var(--accent-primary);
}

/* n-spin 容器样式 - 让它填满剩余空间 */
.editor-card :deep(.n-spin-container),
.editor-card :deep(.n-spin-content) {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-default);
  margin-bottom: 16px;
}

.date-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon-box {
  background: var(--accent-primary);
  opacity: 0.1;
  padding: 8px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.plan-icon-box {
  background: var(--info);
  opacity: 0.1;
}

.date-content {
  display: flex;
  flex-direction: column;
}

.current-date {
  color: var(--text-primary);
  font-weight: 500;
  font-size: 15px;
}

.date-subtitle {
  color: var(--text-muted);
  font-size: 12px;
  margin-top: 2px;
}

.header-right-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.date-picker {
  width: 200px;
}

.preview-toggle-button {
  color: var(--text-secondary);
  transition: color 0.2s;
  font-size: 13px;
}

.preview-toggle-button:hover {
  color: var(--accent-primary);
}

.preview-toggle-button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.save-button {
  color: var(--text-secondary);
  transition: color 0.2s;
}

.save-button:hover {
  color: var(--accent-secondary);
}

.today-button {
  color: var(--accent-primary);
  font-size: 13px;
  transition: all 0.2s;
}

.today-button:hover {
  color: var(--accent-primary);
  opacity: 0.8;
}

.editor-textarea {
  flex: 1;
  width: 100%;
  box-sizing: border-box;
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  color: var(--text-primary);
  font-size: 14px;
  line-height: 1.7;
  font-family: inherit;
  padding: 12px;
  overflow-y: auto;
}

.editor-textarea::placeholder {
  color: var(--text-muted);
}

.editor-textarea::-webkit-scrollbar {
  width: 6px;
}

.editor-textarea::-webkit-scrollbar-track {
  background: transparent;
}

.editor-textarea::-webkit-scrollbar-thumb {
  background: var(--bg-elevated);
  border-radius: 3px;
}

.editor-textarea::-webkit-scrollbar-thumb:hover {
  background: var(--border-hover);
}

.markdown-preview {
  flex: 1;
  width: 100%;
  box-sizing: border-box;
  color: var(--text-primary);
  font-size: 14px;
  line-height: 1.7;
  padding: 12px;
  overflow-y: auto;
}

.markdown-preview::-webkit-scrollbar {
  width: 6px;
}

.markdown-preview::-webkit-scrollbar-track {
  background: transparent;
}

.markdown-preview::-webkit-scrollbar-thumb {
  background: var(--bg-elevated);
  border-radius: 3px;
}

.markdown-preview::-webkit-scrollbar-thumb:hover {
  background: var(--border-hover);
}

/* Markdown 内容样式 */
.markdown-preview :deep(h1),
.markdown-preview :deep(h2),
.markdown-preview :deep(h3),
.markdown-preview :deep(h4),
.markdown-preview :deep(h5),
.markdown-preview :deep(h6) {
  color: var(--text-primary);
  font-weight: 600;
  margin-top: 1.5em;
  margin-bottom: 0.5em;
  line-height: 1.3;
}

.markdown-preview :deep(h1) {
  font-size: 1.8em;
  border-bottom: 2px solid var(--border-default);
  padding-bottom: 0.3em;
}

.markdown-preview :deep(h2) {
  font-size: 1.5em;
  border-bottom: 1px solid var(--border-default);
  padding-bottom: 0.3em;
}

.markdown-preview :deep(h3) {
  font-size: 1.3em;
}

.markdown-preview :deep(h4) {
  font-size: 1.1em;
}

.markdown-preview :deep(p) {
  margin-top: 0.8em;
  margin-bottom: 0.8em;
}

.markdown-preview :deep(ul),
.markdown-preview :deep(ol) {
  margin-top: 0.8em;
  margin-bottom: 0.8em;
  padding-left: 2em;
}

.markdown-preview :deep(li) {
  margin-top: 0.3em;
  margin-bottom: 0.3em;
}

.markdown-preview :deep(code) {
  background: var(--bg-elevated);
  color: var(--accent-secondary);
  padding: 0.2em 0.4em;
  border-radius: 4px;
  font-size: 0.9em;
  font-family: 'Consolas', 'Monaco', monospace;
}

.markdown-preview :deep(pre) {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  padding: 1em;
  overflow-x: auto;
  margin: 1em 0;
}

.markdown-preview :deep(pre code) {
  background: transparent;
  color: var(--text-primary);
  padding: 0;
}

.markdown-preview :deep(blockquote) {
  border-left: 4px solid var(--accent-primary);
  margin: 1em 0;
  padding-left: 1em;
  color: var(--text-secondary);
  font-style: italic;
}

.markdown-preview :deep(a) {
  color: var(--accent-primary);
  text-decoration: none;
  transition: color 0.2s;
}

.markdown-preview :deep(a:hover) {
  color: var(--accent-primary);
  opacity: 0.8;
  text-decoration: underline;
}

.markdown-preview :deep(strong) {
  color: var(--text-primary);
  font-weight: 600;
}

.markdown-preview :deep(em) {
  color: var(--text-primary);
}

.markdown-preview :deep(hr) {
  border: none;
  border-top: 1px solid var(--border-default);
  margin: 2em 0;
}

.editor-footer {
  margin-top: auto;
  padding-top: 12px;
  border-top: 1px solid var(--border-default);
  display: flex;
  align-items: center;
  gap: 12px;
}

.footer-label {
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  flex-shrink: 0;
}

.tags-container {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  align-items: center;
  flex: 1;
}

.log-tag {
  background: var(--bg-elevated) !important;
  color: var(--text-secondary) !important;
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.log-tag:hover {
  color: var(--accent-primary) !important;
}

.add-tag-btn {
  color: var(--text-secondary);
  font-size: 12px;
  padding: 4px 8px;
  transition: color 0.2s;
}

.add-tag-btn:hover {
  color: var(--accent-primary);
}

/* 右侧时间轴 */
.timeline-section {
  width: 280px;
  background: var(--bg-surface);
  border-left: 1px solid var(--border-default);
  padding: 20px;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow: hidden;
}

/* 只在很小的屏幕上隐藏时间轴 */
@media (max-width: 768px) {
  .timeline-section {
    display: none;
  }

  .worklog-container {
    padding: 20px;
  }
}

.timeline-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  padding: 0 4px;
  margin: 0 0 24px 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.timeline-container {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 24px;
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding-right: 4px;
}

/* 时间轴滚动条样式 */
.timeline-container::-webkit-scrollbar {
  width: 4px;
}

.timeline-container::-webkit-scrollbar-track {
  background: transparent;
}

.timeline-container::-webkit-scrollbar-thumb {
  background: var(--bg-elevated);
  border-radius: 2px;
}

.timeline-container::-webkit-scrollbar-thumb:hover {
  background: var(--border-hover);
}

.timeline-line {
  position: absolute;
  left: 5px;
  top: 12px;
  bottom: 12px;
  width: 1px;
  background: var(--border-default);
}

.timeline-item {
  position: relative;
  padding-left: 24px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.timeline-item:hover {
  transform: translateX(2px);
}

.timeline-item:hover .timeline-content {
  background: var(--bg-elevated);
  border-color: var(--border-hover);
}

.timeline-dot {
  position: absolute;
  left: 2px;
  top: 6px;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  border: 2px solid var(--bg-base);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 10;
}

.dot-active {
  background: var(--accent-primary);
  box-shadow: 0 0 0 3px var(--accent-primary);
  opacity: 0.8;
  width: 11px;
  height: 11px;
  left: 0;
  top: 4px;
}

.dot-default {
  background: var(--bg-surface);
}

.timeline-item:hover .dot-default {
  background: var(--accent-primary);
  border-color: var(--bg-base);
  box-shadow: 0 0 0 2px var(--accent-primary);
  opacity: 0.6;
}

.timeline-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border-radius: 10px;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.timeline-date {
  font-size: 11px;
  color: var(--text-muted);
  font-family: 'Consolas', 'Monaco', monospace;
  font-weight: 500;
}

.timeline-item-active .timeline-date {
  color: var(--accent-primary);
}

.timeline-text {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}

.timeline-item-active .timeline-text {
  color: var(--text-primary);
  font-weight: 400;
}

.timeline-empty {
  padding: 32px 0;
  display: flex;
  justify-content: center;
  align-items: center;
}

/* 主要按钮样式 */
:deep(.primary-button) {
  background-color: var(--accent-primary) !important;
  border-color: var(--accent-primary) !important;
  color: #ffffff !important;
  box-shadow: var(--shadow-lg) !important;
  transition: all 0.2s !important;
}

:deep(.primary-button:hover) {
  background-color: var(--accent-primary-hover) !important;
  border-color: var(--accent-primary-hover) !important;
}

:deep(.primary-button:active) {
  background-color: var(--accent-primary-hover) !important;
  border-color: var(--accent-primary-hover) !important;
  opacity: 0.9;
}

/* 周报模式样式 */
.week-selector {
  width: 150px;
}

.weekly-content {
  flex: 1;
  display: flex;
  gap: 24px;
  min-height: 300px;
  max-height: calc(100vh - 350px);
  overflow: hidden;
}

.weekly-tasks {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  max-height: 100%;
  overflow: hidden;
}

.weekly-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  max-height: 100%;
  overflow: hidden;
}

.weekly-preview {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 12px;
}

.weekly-preview::-webkit-scrollbar {
  width: 6px;
}

.weekly-preview::-webkit-scrollbar-track {
  background: transparent;
}

.weekly-preview::-webkit-scrollbar-thumb {
  background: var(--bg-elevated);
  border-radius: 3px;
}

.weekly-preview::-webkit-scrollbar-thumb:hover {
  background: var(--border-hover);
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 16px 0;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-default);
}

.task-list {
  flex: 1;
  overflow-y: overlay;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-right: 4px;
}

.task-list::-webkit-scrollbar {
  width: 6px;
}

.task-list::-webkit-scrollbar-track {
  background: transparent;
}

.task-list::-webkit-scrollbar-thumb {
  background: var(--bg-elevated);
  border-radius: 3px;
}

.task-list::-webkit-scrollbar-thumb:hover {
  background: var(--border-hover);
}

.task-item {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 12px;
  padding: 12px;
  transition: all 0.2s;
}

.task-item:hover {
  background: var(--bg-elevated);
  border-color: var(--border-hover);
}

.task-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.task-icon {
  flex-shrink: 0;
}

.task-title {
  flex: 1;
  color: var(--text-primary);
  font-weight: 500;
  font-size: 14px;
}

.task-category {
  flex-shrink: 0;
  background: var(--accent-primary) !important;
  opacity: 0.3;
  color: var(--accent-primary) !important;
  font-size: 11px;
}

.task-description {
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.6;
  margin-bottom: 8px;
  padding-left: 24px;
}

.task-meta {
  padding-left: 24px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.task-time {
  color: var(--text-muted);
  font-size: 11px;
  font-family: 'Consolas', 'Monaco', monospace;
}

.empty-tasks {
  padding: 40px 0;
}

.empty-hint {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 8px;
}

/* 简化的任务项样式 */
.task-item-simple {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 8px;
  padding: 8px 12px;
  transition: all 0.2s;
}

.task-item-simple:hover {
  background: var(--bg-elevated);
  border-color: var(--border-hover);
}

.task-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.task-check-icon {
  flex-shrink: 0;
}

.task-title-simple {
  flex: 1;
  color: var(--text-primary);
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-tags-simple {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.task-tag-simple {
  font-size: 10px !important;
  padding: 0 6px !important;
  height: 18px !important;
  line-height: 18px !important;
}

.task-time-simple {
  color: var(--text-muted);
  font-size: 11px;
  font-family: 'Consolas', 'Monaco', monospace;
  flex-shrink: 0;
  white-space: nowrap;
}

.weekly-textarea {
  flex: 1;
  min-height: 300px;
  overflow-y: overlay;
}

.stats-container {
  display: flex;
  gap: 24px;
  flex: 1;
}

.stat-item {
  display: flex;
  gap: 8px;
  align-items: center;
}

.stat-label {
  color: var(--text-muted);
  font-size: 12px;
}

.stat-value {
  color: var(--accent-secondary);
  font-size: 13px;
  font-weight: 600;
}

/* 周计划完成率进度条 */
.plan-progress {
  margin-bottom: 12px;
  padding: 10px 12px;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 8px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-size: 12px;
  color: var(--text-secondary);
}

.progress-percent {
  color: var(--accent-secondary);
  font-weight: 600;
}

/* 新增任务标记 */
.task-new {
  background: var(--info) !important;
  opacity: 0.15;
  border-color: var(--info) !important;
}

.new-badge {
  background: var(--info) !important;
  opacity: 0.3;
  color: var(--info) !important;
  font-size: 10px !important;
  padding: 0 6px !important;
  height: 16px !important;
  line-height: 16px !important;
  margin-left: 4px;
  flex-shrink: 0;
}

.new-tag {
  background: var(--info) !important;
  opacity: 0.2;
  color: var(--info) !important;
  margin-left: 8px;
}

/* 周计划按钮样式 */
.ai-button-plan {
  --n-color: color-mix(in srgb, var(--info) 10%, transparent) !important;
  --n-color-hover: color-mix(in srgb, var(--info) 20%, transparent) !important;
  --n-text-color: var(--info) !important;
  --n-border: 1px solid color-mix(in srgb, var(--info) 20%, transparent) !important;
}

/* 图标颜色覆盖 - 使用CSS变量 */
.date-info .icon-box :deep(.n-icon) {
  color: var(--accent-primary) !important;
}

.plan-icon-box :deep(.n-icon) {
  color: var(--info) !important;
}

.task-check-icon {
  color: var(--success) !important;
}
</style>
