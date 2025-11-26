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
            <template v-else>
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
          <template v-else>
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
              <div v-if="!isPreviewMode" class="weekly-content">
                <!-- 左侧：完成的任务列表 -->
                <div class="weekly-tasks">
                  <h3 class="section-title">本周完成的任务</h3>
                  <div v-if="weeklyTasks.length > 0" class="task-list">
                    <div
                      v-for="task in weeklyTasks"
                      :key="task.id"
                      class="task-item"
                    >
                      <div class="task-header">
                        <n-icon size="16" color="#10b981" class="task-icon">
                          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <polyline points="20 6 9 17 4 12" />
                          </svg>
                        </n-icon>
                        <span class="task-title">{{ task.title }}</span>
                        <n-tag
                          v-if="task.category"
                          size="small"
                          :bordered="false"
                          class="task-category"
                        >
                          {{ getCategoryLabel(task.category) }}
                        </n-tag>
                      </div>
                      <div v-if="task.description" class="task-description">
                        {{ task.description }}
                      </div>
                      <div class="task-meta">
                        <span class="task-time">{{ formatTaskTime(task.completedAt!) }}</span>
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

                <!-- 右侧：周报编辑器 -->
                <div class="weekly-editor">
                  <h3 class="section-title">周报内容</h3>
                  <textarea
                    v-model="currentLog"
                    class="editor-textarea weekly-textarea"
                    placeholder="点击'AI生成周报'按钮，基于本周完成的任务自动生成周报总结..."
                  />
                </div>
              </div>
              <div
                v-else
                class="markdown-preview"
                v-html="renderedContent"
              />
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
        </div>
      </div>

      <!-- 右侧时间轴（大屏显示） -->
      <div class="timeline-section">
        <h3 class="timeline-title">最近记录</h3>
        <div class="timeline-container">
          <!-- 垂直线 -->
          <div class="timeline-line" />

          <!-- 时间节点 -->
          <div
            v-for="(log, index) in workLogStore.recentLogs"
            :key="log.date"
            class="timeline-item"
            :class="{ 'timeline-item-active': isToday(log.date) }"
            @click="selectLog(log)"
          >
            <!-- 圆点 -->
            <div
              class="timeline-dot"
              :class="{
                'dot-active': isToday(log.date),
                'dot-default': !isToday(log.date)
              }"
            />
            <!-- 内容 -->
            <div class="timeline-content">
              <div class="timeline-date">{{ formatDate(log.date) }}</div>
              <div class="timeline-text">
                {{ truncate(log.content, 60) || '暂无内容' }}
              </div>
            </div>
          </div>

          <!-- 空状态 -->
          <div v-if="workLogStore.recentLogs.length === 0" class="timeline-empty">
            <n-empty description="暂无历史记录" size="small" />
          </div>
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
import { marked } from 'marked';
import { useWorkLogStore } from '@/stores/workLogStore';
import { useTaskStore } from '@/stores/taskStore';
import { aiApi } from '@/api/aiApi';
import type { WorkLog } from '@/types/workLog';
import type { Task } from '@/types/task';
import { CATEGORY_LABELS } from '@/types/task';

const message = useMessage();
const dialog = useDialog();
const workLogStore = useWorkLogStore();
const taskStore = useTaskStore();

// 日志模式：日报 / 周报
const logMode = ref<'daily' | 'weekly'>('daily');

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

// 周选择选项
const weekOptions = [
  { label: '本周', value: 'this_week' },
  { label: '上周', value: 'last_week' },
  { label: '前两周', value: 'two_weeks_ago' },
  { label: '前三周', value: 'three_weeks_ago' }
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

// LocalStorage 键名
const DRAFT_STORAGE_KEY = 'worklog_draft';
const TAGS_STORAGE_KEY = 'worklog_tags';

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
  const weekKey = dayjs(weekRange.value[0]).format('YYYY-WW');
  await workLogStore.loadWorkLog(weekKey);
  currentLog.value = workLogStore.currentLog?.content || '';
}

// 获取分类标签
function getCategoryLabel(category: Task['category']): string {
  return CATEGORY_LABELS[category] || category;
}

// 格式化任务完成时间
function formatTaskTime(time: string): string {
  return dayjs(time).format('YYYY-MM-DD HH:mm');
}

onMounted(async () => {
  await loadLogs();
  // 加载草稿
  loadDraft();

  // 初始化周范围
  weekRange.value = calculateWeekRange('this_week');
  await loadWeeklyTasks();
});

// 监听日志模式变化
watch(logMode, async (newMode) => {
  // 清空当前内容
  currentLog.value = '';
  isPreviewMode.value = false;

  if (newMode === 'weekly') {
    // 切换到周报模式，加载周任务
    await loadWeeklyTasks();
    // 尝试加载已保存的周报
    const weekKey = dayjs(weekRange.value[0]).format('YYYY-WW');
    await workLogStore.loadWorkLog(weekKey);
    currentLog.value = workLogStore.currentLog?.content || '';
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

// 页面卸载前保存草稿 (仅日报模式)
onBeforeUnmount(() => {
  if (currentLog.value && logMode.value === 'daily') {
    saveDraft();
  }
});

async function loadLogs() {
  try {
    // 加载最近7天的日志
    await workLogStore.loadRecentLogs(7);

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
      // 周报使用周的起始日期作为key
      dateStr = dayjs(weekRange.value[0]).format('YYYY-WW');
      logType = 'weekly';
    }

    await workLogStore.saveWorkLog(dateStr, logType, currentLog.value, false);
    message.success(`${logMode.value === 'daily' ? '日报' : '周报'}已保存`);

    // 清除草稿
    if (logMode.value === 'daily') {
      clearDraft();
    }

    // 重新加载最近日志
    await workLogStore.loadRecentLogs(7);
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

    // TODO: 获取当天完成的任务、执行的SQL、Git提交
    // 这里使用示例数据，实际应从数据库获取
    const completedTasks: string[] = [];
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

function selectLog(log: WorkLog) {
  selectedDate.value = new Date(log.date).getTime();
  currentLog.value = log.content;
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
  background: linear-gradient(to bottom right, #020617, #0f172a);
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
  color: #f1f5f9;
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
  --n-color: rgba(99, 102, 241, 0.1) !important;
  --n-color-hover: rgba(99, 102, 241, 0.2) !important;
  --n-text-color: #6366f1 !important;
  --n-border: 1px solid rgba(99, 102, 241, 0.2) !important;
}

.ai-button-polish {
  --n-color: rgba(251, 191, 36, 0.1) !important;
  --n-color-hover: rgba(251, 191, 36, 0.2) !important;
  --n-text-color: #fbbf24 !important;
  --n-border: 1px solid rgba(251, 191, 36, 0.2) !important;
}

.ai-button-weekly {
  --n-color: rgba(16, 185, 129, 0.1) !important;
  --n-color-hover: rgba(16, 185, 129, 0.2) !important;
  --n-text-color: #10b981 !important;
  --n-border: 1px solid rgba(16, 185, 129, 0.2) !important;
}

.editor-card {
  flex: 1;
  background: rgba(15, 23, 42, 0.4);
  border: 1px solid rgba(148, 163, 184, 0.1);
  border-radius: 20px;
  padding: 24px;
  backdrop-filter: blur(12px);
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  min-height: 0;
  position: relative;
  transition: all 0.3s ease;
}

.editor-card:focus-within {
  border-color: rgba(99, 102, 241, 0.3);
  box-shadow: 0 8px 16px -4px rgba(0, 0, 0, 0.2), 0 0 0 1px rgba(99, 102, 241, 0.1);
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 16px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
  margin-bottom: 16px;
}

.date-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon-box {
  background: rgba(167, 139, 250, 0.1);
  padding: 8px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.date-content {
  display: flex;
  flex-direction: column;
}

.current-date {
  color: #e2e8f0;
  font-weight: 500;
  font-size: 15px;
}

.date-subtitle {
  color: #64748b;
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
  color: #94a3b8;
  transition: color 0.2s;
  font-size: 13px;
}

.preview-toggle-button:hover {
  color: #a78bfa;
}

.preview-toggle-button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.save-button {
  color: #94a3b8;
  transition: color 0.2s;
}

.save-button:hover {
  color: #10b981;
}

.editor-textarea {
  flex: 1;
  width: 100%;
  box-sizing: border-box;
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  color: #cbd5e1;
  font-size: 14px;
  line-height: 1.7;
  font-family: inherit;
  padding: 12px;
  min-height: 300px;
  overflow-y: auto;
}

.editor-textarea::placeholder {
  color: #64748b; /* slate-500 */
}

.editor-textarea::-webkit-scrollbar {
  width: 6px;
}

.editor-textarea::-webkit-scrollbar-track {
  background: transparent;
}

.editor-textarea::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 3px;
}

.editor-textarea::-webkit-scrollbar-thumb:hover {
  background: #475569;
}

.markdown-preview {
  flex: 1;
  width: 100%;
  box-sizing: border-box;
  color: #cbd5e1;
  font-size: 14px;
  line-height: 1.7;
  padding: 12px;
  min-height: 300px;
  overflow-y: auto;
}

.markdown-preview::-webkit-scrollbar {
  width: 6px;
}

.markdown-preview::-webkit-scrollbar-track {
  background: transparent;
}

.markdown-preview::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 3px;
}

.markdown-preview::-webkit-scrollbar-thumb:hover {
  background: #475569;
}

/* Markdown 内容样式 */
.markdown-preview :deep(h1),
.markdown-preview :deep(h2),
.markdown-preview :deep(h3),
.markdown-preview :deep(h4),
.markdown-preview :deep(h5),
.markdown-preview :deep(h6) {
  color: #e2e8f0;
  font-weight: 600;
  margin-top: 1.5em;
  margin-bottom: 0.5em;
  line-height: 1.3;
}

.markdown-preview :deep(h1) {
  font-size: 1.8em;
  border-bottom: 2px solid rgba(148, 163, 184, 0.2);
  padding-bottom: 0.3em;
}

.markdown-preview :deep(h2) {
  font-size: 1.5em;
  border-bottom: 1px solid rgba(148, 163, 184, 0.15);
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
  background: rgba(51, 65, 85, 0.5);
  color: #fbbf24;
  padding: 0.2em 0.4em;
  border-radius: 4px;
  font-size: 0.9em;
  font-family: 'Consolas', 'Monaco', monospace;
}

.markdown-preview :deep(pre) {
  background: rgba(15, 23, 42, 0.8);
  border: 1px solid rgba(148, 163, 184, 0.1);
  border-radius: 8px;
  padding: 1em;
  overflow-x: auto;
  margin: 1em 0;
}

.markdown-preview :deep(pre code) {
  background: transparent;
  color: #cbd5e1;
  padding: 0;
}

.markdown-preview :deep(blockquote) {
  border-left: 4px solid #6366f1;
  margin: 1em 0;
  padding-left: 1em;
  color: #94a3b8;
  font-style: italic;
}

.markdown-preview :deep(a) {
  color: #6366f1;
  text-decoration: none;
  transition: color 0.2s;
}

.markdown-preview :deep(a:hover) {
  color: #818cf8;
  text-decoration: underline;
}

.markdown-preview :deep(strong) {
  color: #e2e8f0;
  font-weight: 600;
}

.markdown-preview :deep(em) {
  color: #cbd5e1;
}

.markdown-preview :deep(hr) {
  border: none;
  border-top: 1px solid rgba(148, 163, 184, 0.2);
  margin: 2em 0;
}

.editor-footer {
  margin-top: auto;
  padding-top: 12px;
  border-top: 1px solid rgba(148, 163, 184, 0.1);
  display: flex;
  align-items: center;
  gap: 12px;
}

.footer-label {
  color: #64748b;
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
  background: rgba(51, 65, 85, 0.5) !important;
  color: #94a3b8 !important; /* slate-400 */
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.log-tag:hover {
  color: #a78bfa !important;
}

.add-tag-btn {
  color: #94a3b8; /* slate-400 */
  font-size: 12px;
  padding: 4px 8px;
  transition: color 0.2s;
}

.add-tag-btn:hover {
  color: #a78bfa;
}

/* 右侧时间轴 */
.timeline-section {
  width: 280px;
  background: rgba(15, 23, 42, 0.3);
  border-left: 1px solid rgba(148, 163, 184, 0.1);
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
  color: #64748b;
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
  background: rgba(51, 65, 85, 0.5);
  border-radius: 2px;
}

.timeline-container::-webkit-scrollbar-thumb:hover {
  background: rgba(71, 85, 105, 0.7);
}

.timeline-line {
  position: absolute;
  left: 5px;
  top: 12px;
  bottom: 12px;
  width: 1px;
  background: rgba(51, 65, 85, 0.8);
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
  background: rgba(30, 41, 59, 0.6);
  border-color: rgba(71, 85, 105, 0.5);
}

.timeline-dot {
  position: absolute;
  left: 2px;
  top: 6px;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  border: 2px solid #0f172a;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 10;
}

.dot-active {
  background: #6366f1;
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.3);
  width: 11px;
  height: 11px;
  left: 0;
  top: 4px;
}

.dot-default {
  background: #1e293b;
}

.timeline-item:hover .dot-default {
  background: #a78bfa;
  border-color: #0f172a;
  box-shadow: 0 0 0 2px rgba(167, 139, 250, 0.2);
}

.timeline-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border-radius: 10px;
  background: rgba(15, 23, 42, 0.4);
  border: 1px solid rgba(51, 65, 85, 0.3);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.timeline-date {
  font-size: 11px;
  color: #64748b;
  font-family: 'Consolas', 'Monaco', monospace;
  font-weight: 500;
}

.timeline-item-active .timeline-date {
  color: #a78bfa;
}

.timeline-text {
  font-size: 13px;
  color: #94a3b8;
  line-height: 1.6;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}

.timeline-item-active .timeline-text {
  color: #cbd5e1;
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
  background-color: #6366f1 !important;
  border-color: #6366f1 !important;
  color: #ffffff !important;
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2) !important;
  transition: all 0.2s !important;
}

:deep(.primary-button:hover) {
  background-color: #4f46e5 !important;
  border-color: #4f46e5 !important;
}

:deep(.primary-button:active) {
  background-color: #4338ca !important;
  border-color: #4338ca !important;
}

/* 周报模式样式 */
.week-selector {
  width: 150px;
}

.weekly-content {
  flex: 1;
  display: flex;
  gap: 24px;
  min-height: 0;
  overflow: hidden;
}

.weekly-tasks {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.weekly-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #e2e8f0;
  margin: 0 0 16px 0;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
}

.task-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.task-list::-webkit-scrollbar {
  width: 6px;
}

.task-list::-webkit-scrollbar-track {
  background: transparent;
}

.task-list::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 3px;
}

.task-list::-webkit-scrollbar-thumb:hover {
  background: #475569;
}

.task-item {
  background: rgba(30, 41, 59, 0.4);
  border: 1px solid rgba(148, 163, 184, 0.1);
  border-radius: 12px;
  padding: 12px;
  transition: all 0.2s;
}

.task-item:hover {
  background: rgba(30, 41, 59, 0.6);
  border-color: rgba(148, 163, 184, 0.2);
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
  color: #e2e8f0;
  font-weight: 500;
  font-size: 14px;
}

.task-category {
  flex-shrink: 0;
  background: rgba(99, 102, 241, 0.15) !important;
  color: #a5b4fc !important;
  font-size: 11px;
}

.task-description {
  color: #94a3b8;
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
  color: #64748b;
  font-size: 11px;
  font-family: 'Consolas', 'Monaco', monospace;
}

.empty-tasks {
  padding: 40px 0;
}

.weekly-textarea {
  flex: 1;
  min-height: 400px;
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
  color: #64748b;
  font-size: 12px;
}

.stat-value {
  color: #10b981;
  font-size: 13px;
  font-weight: 600;
}
</style>
