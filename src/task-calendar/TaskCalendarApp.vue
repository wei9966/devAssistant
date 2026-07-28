<template>
  <n-config-provider :theme="darkTheme">
    <div class="calendar-wrapper">
      <!-- 主容器 - 双栏布局 -->
      <div class="main-container">
        <!-- 左侧区域：日历 -->
        <div class="left-section">
          <!-- 拖动区域和头部 -->
          <div class="calendar-header" data-tauri-drag-region>
            <div class="header-left">
              <div class="handle-dots">
                <span></span><span></span><span></span>
              </div>
              <span class="title">任务日历</span>
            </div>
            <div class="header-actions">
              <button class="nav-btn" @click="navigatePrev">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                  <polyline points="15 18 9 12 15 6"></polyline>
                </svg>
              </button>
              <span class="date-display">{{ dateDisplayText }}</span>
              <button class="nav-btn" @click="navigateNext">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                  <polyline points="9 18 15 12 9 6"></polyline>
                </svg>
              </button>
            </div>
          </div>

          <!-- 工具栏 -->
          <div class="toolbar">
            <!-- 搜索框 -->
            <div class="search-box">
              <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
                <circle cx="11" cy="11" r="8"></circle>
                <path d="m21 21-4.35-4.35"></path>
              </svg>
              <input
                type="text"
                class="search-input"
                placeholder="搜索历史任务..."
                v-model="searchQuery"
              />
            </div>

            <!-- 视图切换 -->
            <div class="view-switch">
              <button class="view-btn" :class="{ active: viewMode === 'week' }" @click="viewMode = 'week'">周</button>
              <button class="view-btn" :class="{ active: viewMode === 'month' }" @click="viewMode = 'month'">月</button>
            </div>
          </div>

          <!-- 搜索结果 -->
          <div v-if="searchQuery && searchResults.length > 0" class="search-results">
            <div class="search-results-header">
              <span>搜索结果 ({{ searchResults.length }})</span>
            </div>
            <div class="search-results-list">
              <div
                v-for="task in searchResults"
                :key="task.id"
                class="search-result-item"
                @click="handleSearchResultClick(task)"
              >
                <div class="task-status-dot" :class="task.status"></div>
                <div class="search-result-content">
                  <span class="search-result-title">{{ task.title }}</span>
                  <span class="search-result-date">{{ task.createdAt?.split(' ')[0] || task.displayDate }}</span>
                </div>
                <span class="task-category">{{ getCategoryLabel(task.category) }}</span>
              </div>
            </div>
          </div>

          <!-- 日历内容 -->
          <div v-else class="calendar-content">
            <!-- 月视图 -->
            <MonthView
              v-if="viewMode === 'month'"
              :currentDate="currentDate"
              :tasks="tasks"
              :selectedDate="selectedDate"
              @select-date="handleSelectDate"
              @task-action="handleTaskAction"
              @add-task="handleAddTaskFromDate"
            />
            <!-- 周视图 -->
            <WeekView
              v-else
              :currentDate="currentDate"
              :tasks="tasks"
              :selectedDate="selectedDate"
              @select-date="handleSelectDate"
              @task-action="handleTaskAction"
              @add-task="handleAddTaskFromDate"
            />
          </div>
        </div>

        <!-- 右侧区域：任务详情面板 -->
        <div class="right-section">
          <div class="panel-header">
            <div class="panel-date-info">
              <span class="panel-date">{{ formatSelectedDate }}</span>
              <span class="task-count">{{ selectedDateTasks.length }} 个任务</span>
            </div>
          </div>

          <div class="task-list">
            <div
              v-for="task in selectedDateTasks"
              :key="task.id"
              class="task-item"
              :class="[task.status, `priority-${task.priority}`]"
              @click="showTaskDetail(task)"
              @contextmenu.prevent="showContextMenu($event, task)"
            >
              <div class="task-status-dot" :class="task.status"></div>
              <div class="task-item-content">
                <span class="task-title">{{ task.title }}</span>
                <span class="task-category">{{ getCategoryLabel(task.category) }}</span>
              </div>
            </div>
            <div v-if="selectedDateTasks.length === 0" class="empty-hint">
              暂无任务
            </div>
          </div>
        </div>
      </div>

      <!-- 底部操作栏 -->
      <div class="action-bar">
        <!-- 尺寸选择按钮 -->
        <div class="size-control">
          <button class="action-btn size-btn" @click.stop="showSizeMenu = !showSizeMenu" title="调整尺寸">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"></path>
            </svg>
          </button>
          <div v-if="showSizeMenu" class="size-menu" @click.stop>
            <div class="size-menu-title">窗口尺寸</div>
            <div
              v-for="(preset, key) in SIZE_PRESETS"
              :key="key"
              class="size-option"
              :class="{ active: currentSizePreset === key }"
              @click="setPresetSize(key as keyof typeof SIZE_PRESETS)"
            >
              <span class="size-label">{{ preset.label }}</span>
              <span class="size-dim">{{ preset.width }} × {{ preset.height }}</span>
            </div>
            <div class="size-hint">拖动边框可自定义尺寸</div>
          </div>
        </div>

        <!-- 透明度调节按钮 -->
        <div class="opacity-control">
          <button class="action-btn opacity-btn" @click.stop="showOpacityMenu = !showOpacityMenu" title="调整透明度">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <circle cx="12" cy="12" r="10"></circle>
              <path d="M12 2a10 10 0 0 1 0 20" fill="currentColor" opacity="0.3"></path>
            </svg>
          </button>
          <div v-if="showOpacityMenu" class="opacity-menu" @click.stop>
            <div class="opacity-menu-title">透明度</div>
            <div class="opacity-slider">
              <input
                type="range"
                min="30"
                max="100"
                :value="windowOpacity"
                @input="setOpacity(Number(($event.target as HTMLInputElement).value))"
              />
              <span class="opacity-value">{{ windowOpacity }}%</span>
            </div>
            <div class="opacity-presets">
              <button
                v-for="preset in [100, 80, 60, 40]"
                :key="preset"
                class="opacity-preset-btn"
                :class="{ active: windowOpacity === preset }"
                @click="setOpacity(preset)"
              >
                {{ preset }}%
              </button>
            </div>
          </div>
        </div>

        <!-- 置顶按钮 -->
        <button
          class="action-btn"
          :class="{ active: isAlwaysOnTop }"
          @click="toggleAlwaysOnTop"
          :title="isAlwaysOnTop ? '取消置顶' : '窗口置顶'"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <path d="M12 2l0 5"></path>
            <path d="M12 22l0-5"></path>
            <path d="M4.93 4.93l3.54 3.54"></path>
            <path d="M15.54 15.54l3.53 3.53"></path>
            <path d="M2 12l5 0"></path>
            <path d="M17 12l5 0"></path>
            <circle cx="12" cy="12" r="4" :fill="isAlwaysOnTop ? 'currentColor' : 'none'"></circle>
          </svg>
        </button>

        <button class="action-btn" @click="goToToday" title="回到今天">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <circle cx="12" cy="12" r="10"></circle>
            <polyline points="12 6 12 12 16 14"></polyline>
          </svg>
        </button>
        <button class="action-btn" @click="openMainWindow" title="打开主窗口">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
            <line x1="9" y1="3" x2="9" y2="21"></line>
          </svg>
        </button>
        <button class="action-btn" @click="refreshTasks" title="刷新">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <polyline points="23 4 23 10 17 10"></polyline>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
          </svg>
        </button>
        <button class="action-btn close" @click="hideWindow" title="隐藏">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <!-- 右键菜单 -->
      <div
        v-if="contextMenu.visible"
        class="context-menu"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
        @mouseleave="hideContextMenu"
      >
        <!-- 新增任务 -->
        <div class="menu-item" @click="handleAddTask">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          <span>新增任务</span>
        </div>
        <!-- 编辑任务（仅当有选中任务时） -->
        <div v-if="contextMenu.task" class="menu-item" @click="handleEditTask">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
          </svg>
          <span>编辑任务</span>
        </div>
        <div class="menu-divider"></div>
        <template v-if="contextMenu.task?.status === 'active'">
          <div class="menu-item" @click="handleComplete">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
            <span>完成任务</span>
          </div>
          <div class="menu-item" @click="handlePause">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <rect x="6" y="4" width="4" height="16"></rect>
              <rect x="14" y="4" width="4" height="16"></rect>
            </svg>
            <span>暂停任务</span>
          </div>
        </template>
        <template v-else-if="contextMenu.task?.status === 'todo'">
          <div class="menu-item" @click="handleStart">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <polygon points="5 3 19 12 5 21 5 3"></polygon>
            </svg>
            <span>开始任务</span>
          </div>
          <div class="menu-item" @click="handleComplete">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
            <span>完成任务</span>
          </div>
        </template>
        <div v-if="contextMenu.task" class="menu-divider"></div>
        <div v-if="contextMenu.task" class="menu-item" @click="handleMoveToToday">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <circle cx="12" cy="12" r="10"></circle>
            <polyline points="12 6 12 12 16 14"></polyline>
          </svg>
          <span>移至今天</span>
        </div>
      </div>

      <!-- 调整大小的边缘区域 -->
      <div class="resize-edge resize-right" @mousedown="(e) => startResize(e, 'East')"></div>
      <div class="resize-edge resize-bottom" @mousedown="(e) => startResize(e, 'South')"></div>
      <div class="resize-edge resize-corner" @mousedown="(e) => startResize(e, 'SouthEast')"></div>
      <div class="resize-edge resize-left" @mousedown="(e) => startResize(e, 'West')"></div>
      <div class="resize-edge resize-top" @mousedown="(e) => startResize(e, 'North')"></div>
    </div>

    <!-- 任务详情弹窗 -->
    <n-message-provider>
      <n-dialog-provider>
        <TaskDetailModal
          :task="detailTask"
          :show="showDetailModal"
          :readonly="true"
          @update:show="showDetailModal = $event"
        />

        <!-- 新增/编辑任务弹窗 -->
        <n-modal
          v-model:show="showTaskModal"
          preset="card"
          :title="isEditing ? '编辑任务' : '新增任务'"
          class="task-modal"
          style="width: 680px; max-width: 90vw;"
          :mask-closable="false"
          :segmented="{
            content: 'soft',
            footer: 'soft'
          }"
          to="body"
        >
          <div class="task-form-container">
            <n-form
              ref="formRef"
              :model="formData"
              :rules="formRules"
              label-placement="top"
              label-width="auto"
              size="medium"
            >
              <!-- 任务标题 -->
              <n-form-item label="任务标题" path="title">
                <div class="input-with-ai">
                  <n-input
                    v-model:value="formData.title"
                    placeholder="请输入任务标题"
                    :maxlength="100"
                    show-count
                  />
                  <n-button
                    v-if="isAiEnabled"
                    quaternary
                    type="primary"
                    size="small"
                    :loading="aiClassifying"
                    :disabled="!formData.title"
                    @click="handleAiClassify"
                    title="AI 智能分类"
                    class="ai-btn"
                  >
                    <template #icon>
                      <n-icon><GridOutline /></n-icon>
                    </template>
                    AI 分类
                  </n-button>
                </div>
              </n-form-item>

              <!-- 任务描述 -->
              <n-form-item label="任务描述" path="description">
                <div class="input-with-ai-vertical">
                  <n-input
                    v-model:value="formData.description"
                    type="textarea"
                    placeholder="请输入任务描述（可选）"
                    :autosize="{ minRows: 3, maxRows: 6 }"
                    :maxlength="1000"
                    show-count
                  />
                  <div class="ai-actions" v-if="isAiEnabled">
                    <n-button
                      quaternary
                      size="tiny"
                      :loading="aiEnhancing"
                      :disabled="!formData.title"
                      @click="handleAiEnhance"
                      class="ai-btn-small"
                    >
                      AI 增强描述
                    </n-button>
                    <n-button
                      quaternary
                      size="tiny"
                      :loading="aiGeneratingSubtasks"
                      :disabled="!formData.title"
                      @click="handleAiGenerateSubtasks"
                      class="ai-btn-small"
                    >
                      AI 生成子任务
                    </n-button>
                  </div>
                </div>
              </n-form-item>

              <!-- 四象限选择 -->
              <n-form-item label="四象限" path="quadrant">
                <QuadrantSelector v-model:value="formData.quadrant" />
              </n-form-item>

              <!-- 优先级选择 -->
              <n-form-item label="优先级" path="priority">
                <PrioritySelector v-model:value="formData.priority" />
              </n-form-item>

              <!-- 分类选择 -->
              <n-form-item label="分类" path="category">
                <CategorySelector v-model="formData.category" />
              </n-form-item>

              <!-- 标签选择 -->
              <n-form-item label="标签">
                <TagSelector
                  v-model:modelValue="formData.tagIds"
                  :available-tags="availableTags"
                  @manage="showTagManager = true"
                />
              </n-form-item>

              <!-- 日期选择 -->
              <div class="date-row">
                <n-form-item label="登记日期" path="registeredAt">
                  <n-date-picker
                    v-model:value="formData.registeredAt"
                    type="date"
                    clearable
                    style="width: 100%"
                  />
                </n-form-item>
                <n-form-item label="截止日期" path="dueDate">
                  <n-date-picker
                    v-model:value="formData.dueDate"
                    type="date"
                    clearable
                    style="width: 100%"
                  />
                </n-form-item>
              </div>

              <n-form-item label="计划开始时间" path="scheduledStartTime">
                <n-date-picker
                  v-model:value="formData.scheduledStartTime"
                  type="datetime"
                  clearable
                  style="width: 100%"
                />
              </n-form-item>
            </n-form>
          </div>

          <template #footer>
            <n-space justify="end">
              <n-button @click="handleCancelTask" :disabled="isSubmitting">取消</n-button>
              <n-button
                type="primary"
                :loading="isSubmitting"
                :disabled="isSubmitting"
                @click="handleSubmitTask"
              >
                {{ isSubmitting ? 'AI 分析中...' : (isEditing ? '保存' : (isAiEnabled ? 'AI 智能创建' : '创建')) }}
              </n-button>
            </n-space>
          </template>
        </n-modal>

        <!-- 标签管理弹窗 -->
        <TagManager
          v-model:show="showTagManager"
          :tags="availableTags"
          @create="handleCreateTag"
          @update="handleUpdateTag"
          @delete="handleDeleteTag"
          @toggle-favorite="handleToggleTagFavorite"
        />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, watch } from 'vue';
import { NConfigProvider, NMessageProvider, NDialogProvider, NModal, NForm, NFormItem, NInput, NDatePicker, NButton, NSpace, NIcon, darkTheme } from 'naive-ui';
import { GridOutline, AddOutline } from '@vicons/ionicons5';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, Window, LogicalSize, LogicalPosition } from '@tauri-apps/api/window';
import { listen, emit } from '@tauri-apps/api/event';
import dayjs from 'dayjs';
import type { Task, TaskQuadrant, Tag } from '@/types/task';
import { useTaskCategoryStore } from '@/stores/taskCategoryStore';
import MonthView from './components/MonthView.vue';
import WeekView from './components/WeekView.vue';
import TaskDetailModal from '@/components/TaskDetailModal.vue';
import QuadrantSelector from '@/components/QuadrantSelector.vue';
import PrioritySelector from '@/components/PrioritySelector.vue';
import CategorySelector from '@/components/CategorySelector.vue';
import TagSelector from '@/components/TagSelector.vue';
import TagManager from '@/components/TagManager.vue';
import { aiApi } from '@/api/aiApi';
import { tagApi } from '@/api/tagApi';

// 日历窗口尺寸配置key
const CALENDAR_SIZE_KEY = 'task_calendar_size';
const CALENDAR_POSITION_KEY = 'task_calendar_position';
const taskCategoryStore = useTaskCategoryStore();

// 预设尺寸 (双栏布局需要更宽的尺寸)
const SIZE_PRESETS = {
  small: { width: 650, height: 450, label: '小' },
  medium: { width: 750, height: 560, label: '中' },
  large: { width: 900, height: 680, label: '大' },
  xlarge: { width: 1050, height: 800, label: '特大' }
};

// 视图模式
const viewMode = ref<'week' | 'month'>('month');

// 当前日期（用于导航）
const currentDate = ref(new Date());

// 选中的日期
const selectedDate = ref<string | null>(null);

// 任务数据
const tasks = ref<Task[]>([]);

// 搜索功能
const searchQuery = ref('');

// 右键菜单状态
const contextMenu = ref<{
  visible: boolean;
  x: number;
  y: number;
  task: Task | null;
}>({
  visible: false,
  x: 0,
  y: 0,
  task: null,
});

let unlistenTaskUpdate: (() => void) | null = null;
let unlistenResize: (() => void) | null = null;
let resizeTimer: ReturnType<typeof setTimeout> | null = null;

// 当前尺寸预设
const currentSizePreset = ref<keyof typeof SIZE_PRESETS | 'custom'>('medium');

// 显示尺寸选择菜单
const showSizeMenu = ref(false);

// 任务详情弹窗
const showDetailModal = ref(false);
const detailTask = ref<Task | null>(null);

// 新增/编辑任务弹框状态
const showTaskModal = ref(false);
const isEditing = ref(false);
const editingTaskId = ref<number | null>(null);
const isSubmitting = ref(false);
const formRef = ref();

// AI 功能状态
const aiClassifying = ref(false);
const aiEnhancing = ref(false);
const aiGeneratingSubtasks = ref(false);
const isAiEnabled = ref(false);

// 标签数据
const availableTags = ref<Tag[]>([]);
const showTagManager = ref(false);

// 表单数据
const formData = reactive({
  title: '',
  description: '',
  category: 'other' as Task['category'],
  priority: 2 as Task['priority'],
  quadrant: 'urgent_not_important' as TaskQuadrant,
  tagIds: [] as number[],
  dueDate: null as number | null,
  registeredAt: Date.now() as number,
  scheduledStartTime: null as number | null,
});

// 表单验证规则
const formRules = {
  title: {
    required: true,
    message: '请输入任务标题',
    trigger: 'blur',
  },
};

// 窗口置顶状态
const isAlwaysOnTop = ref(false);

// 透明度控制
const showOpacityMenu = ref(false);
const windowOpacity = ref(100); // 0-100

// 本地存储 key
const CALENDAR_ALWAYS_ON_TOP_KEY = 'task_calendar_always_on_top';
const CALENDAR_OPACITY_KEY = 'task_calendar_opacity';

// 计算显示文本
const dateDisplayText = computed(() => {
  const year = currentDate.value.getFullYear();
  const month = currentDate.value.getMonth() + 1;

  if (viewMode.value === 'month') {
    return `${year}年${month}月`;
  } else {
    // 周视图：显示周范围
    const weekStart = getWeekStart(currentDate.value);
    const weekEnd = new Date(weekStart);
    weekEnd.setDate(weekEnd.getDate() + 6);

    const startMonth = weekStart.getMonth() + 1;
    const endMonth = weekEnd.getMonth() + 1;

    if (startMonth === endMonth) {
      return `${year}年${startMonth}月 第${getWeekNumber(currentDate.value)}周`;
    } else {
      return `${startMonth}/${weekStart.getDate()} - ${endMonth}/${weekEnd.getDate()}`;
    }
  }
});

// 格式化选中日期
const formatSelectedDate = computed(() => {
  if (!selectedDate.value) return '';
  const date = new Date(selectedDate.value);
  const weekDays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
  return `${date.getMonth() + 1}月${date.getDate()}日 ${weekDays[date.getDay()]}`;
});

// 选中日期的任务（排序：未完成在前，完成在后）
const selectedDateTasks = computed(() => {
  if (!selectedDate.value) return [];
  const dayTasks = tasks.value.filter(task => getTaskDisplayDate(task) === selectedDate.value);
  // 排序：未完成的在前，完成的在后，同状态按优先级排序
  return dayTasks.sort((a, b) => {
    if (a.status === 'done' && b.status !== 'done') return 1;
    if (a.status !== 'done' && b.status === 'done') return -1;
    return (a.priority || 3) - (b.priority || 3);
  });
});

// 搜索结果
const searchResults = computed(() => {
  if (!searchQuery.value.trim()) return [];
  const query = searchQuery.value.toLowerCase().trim();
  return tasks.value.filter(task =>
    task.title.toLowerCase().includes(query) ||
    task.description?.toLowerCase().includes(query)
  );
});

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
    // 如果设置了 displayDate 且是未来日期，使用它（用于计划性任务）
    if (task.displayDate && task.displayDate >= today) {
      return task.displayDate;
    }
    // 否则所有未完成任务都显示在今天
    return today;
  }

  // 兜底逻辑（理论上不会走到这里）
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

// 获取周的第一天（周一）
function getWeekStart(date: Date): Date {
  const d = new Date(date);
  const day = d.getDay();
  const diff = d.getDate() - day + (day === 0 ? -6 : 1);
  d.setDate(diff);
  return d;
}

// 获取周数
function getWeekNumber(date: Date): number {
  const d = new Date(date);
  d.setHours(0, 0, 0, 0);
  d.setDate(d.getDate() + 4 - (d.getDay() || 7));
  const yearStart = new Date(d.getFullYear(), 0, 1);
  return Math.ceil((((d.getTime() - yearStart.getTime()) / 86400000) + 1) / 7);
}

// 格式化日期为 YYYY-MM-DD
function formatDate(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

// 导航到上一个周期
function navigatePrev() {
  const newDate = new Date(currentDate.value);
  if (viewMode.value === 'month') {
    newDate.setMonth(newDate.getMonth() - 1);
  } else {
    newDate.setDate(newDate.getDate() - 7);
  }
  currentDate.value = newDate;
}

// 导航到下一个周期
function navigateNext() {
  const newDate = new Date(currentDate.value);
  if (viewMode.value === 'month') {
    newDate.setMonth(newDate.getMonth() + 1);
  } else {
    newDate.setDate(newDate.getDate() + 7);
  }
  currentDate.value = newDate;
}

// 回到今天
function goToToday() {
  currentDate.value = new Date();
  selectedDate.value = formatDate(new Date());
}

// 选择日期
function handleSelectDate(date: string) {
  selectedDate.value = date;
}

// 处理搜索结果点击
function handleSearchResultClick(task: Task) {
  const taskDate = getTaskDisplayDate(task);
  selectedDate.value = taskDate;

  // 跳转到任务所在的日期
  const date = new Date(taskDate);
  currentDate.value = date;

  // 清空搜索
  searchQuery.value = '';

  // 显示任务详情
  showTaskDetail(task);
}

// 获取分类标签
function getCategoryLabel(category: Task['category']): string {
  return taskCategoryStore.getLabel(category);
}

// 加载任务
async function loadTasks() {
  try {
    let startDate: string;
    let endDate: string;

    if (viewMode.value === 'month') {
      // 月视图：获取当月及前后各一周的数据
      const firstDay = new Date(currentDate.value.getFullYear(), currentDate.value.getMonth(), 1);
      const lastDay = new Date(currentDate.value.getFullYear(), currentDate.value.getMonth() + 1, 0);

      const start = new Date(firstDay);
      start.setDate(start.getDate() - 7);
      const end = new Date(lastDay);
      end.setDate(end.getDate() + 7);

      startDate = formatDate(start);
      endDate = formatDate(end);
    } else {
      // 周视图
      const weekStart = getWeekStart(currentDate.value);
      const weekEnd = new Date(weekStart);
      weekEnd.setDate(weekEnd.getDate() + 6);

      startDate = formatDate(weekStart);
      endDate = formatDate(weekEnd);
    }

    tasks.value = await invoke('get_tasks_by_date_range', { startDate, endDate });
  } catch (error) {
    console.error('加载任务失败:', error);
  }
}

// 处理任务操作
function handleTaskAction(action: string, task: Task, event?: MouseEvent) {
  contextMenu.value.task = task;
  switch (action) {
    case 'start':
      handleStart();
      break;
    case 'pause':
      handlePause();
      break;
    case 'complete':
      handleComplete();
      break;
    case 'detail':
      showTaskDetail(task);
      break;
    case 'context':
      // 从视图组件中触发的右键菜单
      if (event) {
        showContextMenu(event, task);
      } else {
        // 如果没有事件，使用一个默认位置
        contextMenu.value = {
          visible: true,
          x: 100,
          y: 100,
          task,
        };
      }
      break;
  }
}

// 显示任务详情弹窗
function showTaskDetail(task: Task) {
  detailTask.value = task;
  showDetailModal.value = true;
}

// 显示右键菜单
function showContextMenu(event: MouseEvent, task: Task) {
  const menuWidth = 150;
  const menuHeight = 120;
  const maxX = window.innerWidth - menuWidth - 10;
  const maxY = window.innerHeight - menuHeight - 10;

  contextMenu.value = {
    visible: true,
    x: Math.min(event.clientX, maxX),
    y: Math.min(event.clientY, maxY),
    task,
  };
}

function hideContextMenu() {
  contextMenu.value.visible = false;
}

async function handleStart() {
  if (!contextMenu.value.task?.id) return;
  try {
    await invoke('start_task', { taskId: contextMenu.value.task.id });
    await loadTasks();
    await emit('task-updated');
  } catch (error) {
    console.error('开始任务失败:', error);
  }
  hideContextMenu();
}

async function handlePause() {
  if (!contextMenu.value.task?.id) return;
  try {
    await invoke('pause_task', { taskId: contextMenu.value.task.id, context: null });
    await loadTasks();
    await emit('task-updated');
  } catch (error) {
    console.error('暂停任务失败:', error);
  }
  hideContextMenu();
}

async function handleComplete() {
  if (!contextMenu.value.task?.id) return;
  try {
    await invoke('complete_task', { taskId: contextMenu.value.task.id });
    await loadTasks();
    await emit('task-updated');
  } catch (error) {
    console.error('完成任务失败:', error);
  }
  hideContextMenu();
}

async function handleMoveToToday() {
  if (!contextMenu.value.task?.id) return;
  try {
    await invoke('move_task_to_today', { taskId: contextMenu.value.task.id });
    await loadTasks();
    await emit('task-updated');
  } catch (error) {
    console.error('移至今天失败:', error);
  }
  hideContextMenu();
}

// ========== 任务新增/编辑相关函数 ==========

// 加载标签
async function loadTags() {
  try {
    const tags = await tagApi.getAllTags();
    availableTags.value = tags || [];
  } catch (error) {
    console.error('加载标签失败:', error);
    availableTags.value = [];
  }
}

// 从日期单元格添加任务（月视图右键）
function handleAddTaskFromDate(date: string, event: MouseEvent) {
  // 先选中这个日期
  selectedDate.value = date;
  // 显示右键菜单（无任务选中状态）
  contextMenu.value = {
    visible: true,
    x: Math.min(event.clientX, window.innerWidth - 160),
    y: Math.min(event.clientY, window.innerHeight - 120),
    task: null,
  };
}

// 新增任务
function handleAddTask() {
  isEditing.value = false;
  editingTaskId.value = null;
  resetFormData();
  // 如果有选中的日期，设置为登记日期
  if (selectedDate.value) {
    formData.registeredAt = new Date(selectedDate.value).getTime();
  }
  showTaskModal.value = true;
  hideContextMenu();
}

// 编辑任务
function handleEditTask() {
  if (!contextMenu.value.task) return;

  isEditing.value = true;
  editingTaskId.value = contextMenu.value.task.id ?? null;

  formData.title = contextMenu.value.task.title;
  formData.description = contextMenu.value.task.description || '';
  formData.category = contextMenu.value.task.category;
  formData.priority = contextMenu.value.task.priority;
  formData.quadrant = contextMenu.value.task.quadrant || 'urgent_not_important';
  formData.tagIds = contextMenu.value.task.tags?.map(t => t.id!).filter(id => id !== undefined) || [];
  formData.dueDate = contextMenu.value.task.dueDate ? new Date(contextMenu.value.task.dueDate).getTime() : null;
  formData.registeredAt = contextMenu.value.task.registeredAt ? new Date(contextMenu.value.task.registeredAt).getTime() : Date.now();
  formData.scheduledStartTime = contextMenu.value.task.scheduledStartTime ? new Date(contextMenu.value.task.scheduledStartTime).getTime() : null;

  showTaskModal.value = true;
  hideContextMenu();
}

// 重置表单数据
function resetFormData() {
  formData.title = '';
  formData.description = '';
  formData.category = 'other';
  formData.priority = 2;
  formData.quadrant = 'urgent_not_important';
  formData.tagIds = [];
  formData.dueDate = null;
  formData.registeredAt = Date.now();
  formData.scheduledStartTime = null;
}

// 取消任务编辑
function handleCancelTask() {
  showTaskModal.value = false;
  resetFormData();
}

// 提交任务
async function handleSubmitTask() {
  try {
    await formRef.value?.validate();

    if (formData.title.trim().length === 0) {
      console.error('任务标题不能为空');
      return;
    }

    isSubmitting.value = true;

    let finalCategory = formData.category;
    let finalPriority = formData.priority;
    let finalQuadrant = formData.quadrant;
    let aiSuggestedTagIds: number[] = [];

    // 如果 AI 启用且是新建，自动进行 AI 分类
    if (isAiEnabled.value && !isEditing.value) {
      try {
        const existingTagNames = availableTags.value.map(t => t.name);
        const result = await aiApi.classifyTask(
          formData.title,
          formData.description || undefined,
          existingTagNames
        );

        if (formData.category === 'other' && result.category) {
          finalCategory = result.category as Task['category'];
        }
        if (formData.priority === 2 && result.priority) {
          finalPriority = result.priority as Task['priority'];
        }
        if (formData.quadrant === 'urgent_not_important' && result.quadrant) {
          finalQuadrant = result.quadrant as TaskQuadrant;
        }

        if (result.suggestedTags && result.suggestedTags.length > 0) {
          for (const suggestedTag of result.suggestedTags) {
            const matchedTag = availableTags.value.find(
              t => t.name.toLowerCase() === suggestedTag.toLowerCase()
            );
            if (matchedTag && matchedTag.id) {
              aiSuggestedTagIds.push(matchedTag.id);
            }
          }
        }
      } catch (error) {
        console.error('AI 分类失败，使用默认值:', error);
      }
    }

    const dueDateStr = formData.dueDate ? dayjs(formData.dueDate).format('YYYY-MM-DD') : undefined;
    const registeredAtStr = formData.registeredAt ? dayjs(formData.registeredAt).format('YYYY-MM-DD') : undefined;
    const scheduledStartTimeStr = formData.scheduledStartTime ? dayjs(formData.scheduledStartTime).format('YYYY-MM-DD HH:mm:ss') : undefined;

    if (isEditing.value && editingTaskId.value) {
      await invoke('update_task', {
        taskId: editingTaskId.value,
        title: formData.title,
        description: formData.description || null,
        category: formData.category,
        priority: formData.priority,
        quadrant: formData.quadrant,
        dueDate: dueDateStr,
        registeredAt: registeredAtStr,
        scheduledStartTime: scheduledStartTimeStr,
      });

      // 更新标签
      try {
        await tagApi.removeAllTagsFromTask(editingTaskId.value);
        if (formData.tagIds.length > 0) {
          await tagApi.addTagsToTask(editingTaskId.value, formData.tagIds);
        }
      } catch (error) {
        console.error('更新标签失败:', error);
      }
    } else {
      const taskId = await invoke<number>('create_task', {
        title: formData.title,
        description: formData.description || null,
        category: finalCategory,
        priority: finalPriority,
        quadrant: finalQuadrant,
        dueDate: dueDateStr,
        registeredAt: registeredAtStr,
        scheduledStartTime: scheduledStartTimeStr,
      });

      // 合并标签
      const allTagIds = [...new Set([...formData.tagIds, ...aiSuggestedTagIds])];
      if (allTagIds.length > 0) {
        try {
          await tagApi.addTagsToTask(taskId, allTagIds);
        } catch (error) {
          console.error('添加标签失败:', error);
        }
      }
    }

    await loadTasks();
    await emit('task-updated');

    showTaskModal.value = false;
    resetFormData();
  } catch (error) {
    console.error(isEditing.value ? '更新任务失败:' : '创建任务失败:', error);
  } finally {
    isSubmitting.value = false;
  }
}

// AI 智能分类
async function handleAiClassify() {
  if (!formData.title) return;

  aiClassifying.value = true;

  try {
    const existingTagNames = availableTags.value.map(t => t.name);
    const result = await aiApi.classifyTask(
      formData.title,
      formData.description || undefined,
      existingTagNames
    );

    if (result.category) {
      formData.category = result.category as Task['category'];
    }
    if (result.priority) {
      formData.priority = result.priority as Task['priority'];
    }
    if (result.quadrant) {
      formData.quadrant = result.quadrant as TaskQuadrant;
    }

    if (result.suggestedTags && result.suggestedTags.length > 0) {
      const matchedTagIds: number[] = [];
      for (const suggestedTag of result.suggestedTags) {
        const matchedTag = availableTags.value.find(
          t => t.name.toLowerCase() === suggestedTag.toLowerCase()
        );
        if (matchedTag && matchedTag.id) {
          matchedTagIds.push(matchedTag.id);
        }
      }
      if (matchedTagIds.length > 0) {
        formData.tagIds = matchedTagIds;
      }
    }
  } catch (error) {
    console.error('AI 分类失败:', error);
  } finally {
    aiClassifying.value = false;
  }
}

// AI 增强描述
async function handleAiEnhance() {
  if (!formData.title) return;

  aiEnhancing.value = true;

  try {
    const enhancedDesc = await aiApi.enhanceTaskDescription(
      formData.title,
      formData.description || undefined
    );
    formData.description = enhancedDesc;
  } catch (error) {
    console.error('AI 增强描述失败:', error);
  } finally {
    aiEnhancing.value = false;
  }
}

// AI 生成子任务
async function handleAiGenerateSubtasks() {
  if (!formData.title) return;

  aiGeneratingSubtasks.value = true;

  try {
    const subtasks = await aiApi.generateSubtasks(
      formData.title,
      formData.description || undefined
    );

    if (subtasks.length > 0) {
      const subtasksSection = '\n\n## 子任务\n' + subtasks.map((task, index) => `${index + 1}. ${task}`).join('\n');
      formData.description = (formData.description || '') + subtasksSection;
    }
  } catch (error) {
    console.error('AI 生成子任务失败:', error);
  } finally {
    aiGeneratingSubtasks.value = false;
  }
}

// 标签管理回调
async function handleCreateTag(tag: Omit<Tag, 'id'>) {
  try {
    await tagApi.createTag(tag.name, tag.color);
    await loadTags();
  } catch (error) {
    console.error('创建标签失败:', error);
  }
}

async function handleUpdateTag(id: number, updates: Partial<Tag>) {
  try {
    await tagApi.updateTag(id, updates.name!, updates.color!);
    await loadTags();
  } catch (error) {
    console.error('更新标签失败:', error);
  }
}

async function handleDeleteTag(id: number) {
  try {
    await tagApi.deleteTag(id);
    await loadTags();
  } catch (error) {
    console.error('删除标签失败:', error);
  }
}

async function handleToggleTagFavorite(id: number, isFavorite: boolean) {
  try {
    await tagApi.toggleTagFavorite(id, isFavorite);
    await loadTags();
  } catch (error) {
    console.error('切换常用标签失败:', error);
  }
}

async function openMainWindow() {
  try {
    const mainWindow = await Window.getByLabel('main');
    if (mainWindow) {
      await mainWindow.show();
      await mainWindow.setFocus();
    }
  } catch (error) {
    console.error('打开主窗口失败:', error);
  }
}

async function refreshTasks() {
  await loadTasks();
}

async function hideWindow() {
  const currentWindow = getCurrentWindow();
  await emit('task-calendar-hidden');
  await currentWindow.hide();
}

// 切换置顶状态
async function toggleAlwaysOnTop() {
  try {
    const currentWindow = getCurrentWindow();
    isAlwaysOnTop.value = !isAlwaysOnTop.value;
    await currentWindow.setAlwaysOnTop(isAlwaysOnTop.value);
    localStorage.setItem(CALENDAR_ALWAYS_ON_TOP_KEY, String(isAlwaysOnTop.value));
  } catch (error) {
    console.error('设置置顶失败:', error);
  }
}

// 设置窗口透明度
async function setOpacity(opacity: number) {
  try {
    windowOpacity.value = opacity;
    // 应用透明度到 wrapper 元素
    const wrapper = document.querySelector('.calendar-wrapper') as HTMLElement;
    if (wrapper) {
      wrapper.style.opacity = String(opacity / 100);
    }
    localStorage.setItem(CALENDAR_OPACITY_KEY, String(opacity));
  } catch (error) {
    console.error('设置透明度失败:', error);
  }
}

// 恢复置顶和透明度设置
async function restoreWindowSettings() {
  try {
    const currentWindow = getCurrentWindow();

    // 恢复置顶状态（默认不置顶）
    const savedOnTop = localStorage.getItem(CALENDAR_ALWAYS_ON_TOP_KEY);
    isAlwaysOnTop.value = savedOnTop === 'true';
    await currentWindow.setAlwaysOnTop(isAlwaysOnTop.value);

    // 恢复透明度
    const savedOpacity = localStorage.getItem(CALENDAR_OPACITY_KEY);
    if (savedOpacity) {
      windowOpacity.value = parseInt(savedOpacity);
      const wrapper = document.querySelector('.calendar-wrapper') as HTMLElement;
      if (wrapper) {
        wrapper.style.opacity = String(windowOpacity.value / 100);
      }
    }
  } catch (error) {
    console.error('恢复窗口设置失败:', error);
  }
}

// 开始调整窗口大小
async function startResize(e: MouseEvent, direction: string) {
  e.preventDefault();
  e.stopPropagation();

  const currentWindow = getCurrentWindow();

  try {
    const startSize = await currentWindow.innerSize();
    const startPos = await currentWindow.outerPosition();
    const startMouseX = e.screenX;
    const startMouseY = e.screenY;

    const handleMouseMove = async (moveEvent: MouseEvent) => {
      const deltaX = moveEvent.screenX - startMouseX;
      const deltaY = moveEvent.screenY - startMouseY;

      let newWidth = startSize.width;
      let newHeight = startSize.height;
      let newX = startPos.x;
      let newY = startPos.y;

      // 根据方向计算新尺寸 (双栏布局最小宽度650px)
      if (direction.includes('East')) {
        newWidth = Math.max(650, startSize.width + deltaX);
      }
      if (direction.includes('West')) {
        const widthDelta = Math.min(deltaX, startSize.width - 650);
        newWidth = startSize.width - widthDelta;
        newX = startPos.x + widthDelta;
      }
      if (direction.includes('South')) {
        newHeight = Math.max(450, startSize.height + deltaY);
      }
      if (direction.includes('North')) {
        const heightDelta = Math.min(deltaY, startSize.height - 450);
        newHeight = startSize.height - heightDelta;
        newY = startPos.y + heightDelta;
      }

      try {
        await currentWindow.setSize(new LogicalSize(newWidth, newHeight));
        if (direction.includes('West') || direction.includes('North')) {
          await currentWindow.setPosition(new LogicalPosition(newX, newY));
        }
      } catch (err) {
        // 忽略错误
      }
    };

    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      debouncedSaveSize();
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  } catch (error) {
    console.error('调整窗口大小失败:', error);
  }
}

// 保存窗口尺寸
async function saveWindowSize() {
  try {
    const currentWindow = getCurrentWindow();
    const size = await currentWindow.innerSize();
    const position = await currentWindow.outerPosition();

    localStorage.setItem(CALENDAR_SIZE_KEY, JSON.stringify({
      width: size.width,
      height: size.height
    }));

    localStorage.setItem(CALENDAR_POSITION_KEY, JSON.stringify({
      x: position.x,
      y: position.y
    }));

    // 检测当前是哪个预设
    detectSizePreset(size.width, size.height);
  } catch (error) {
    console.error('保存窗口尺寸失败:', error);
  }
}

// 检测当前尺寸对应的预设
function detectSizePreset(width: number, height: number) {
  for (const [key, preset] of Object.entries(SIZE_PRESETS)) {
    if (Math.abs(preset.width - width) < 20 && Math.abs(preset.height - height) < 20) {
      currentSizePreset.value = key as keyof typeof SIZE_PRESETS;
      return;
    }
  }
  currentSizePreset.value = 'custom';
}

// 恢复窗口尺寸
async function restoreWindowSize() {
  try {
    const currentWindow = getCurrentWindow();

    // 恢复尺寸（添加验证，防止无效尺寸导致崩溃）
    const sizeStr = localStorage.getItem(CALENDAR_SIZE_KEY);
    if (sizeStr) {
      const size = JSON.parse(sizeStr);
      // 验证尺寸有效性：必须大于最小值且小于合理最大值
      const minWidth = 650;
      const minHeight = 450;
      const maxWidth = 2000;
      const maxHeight = 1500;

      if (size.width >= minWidth && size.width <= maxWidth &&
          size.height >= minHeight && size.height <= maxHeight) {
        await currentWindow.setSize(new LogicalSize(size.width, size.height));
        detectSizePreset(size.width, size.height);
      } else {
        // 尺寸无效，清除保存的数据并使用默认尺寸
        console.warn('保存的窗口尺寸无效，使用默认尺寸:', size);
        localStorage.removeItem(CALENDAR_SIZE_KEY);
        await currentWindow.setSize(new LogicalSize(SIZE_PRESETS.medium.width, SIZE_PRESETS.medium.height));
      }
    }

    // 恢复位置
    const posStr = localStorage.getItem(CALENDAR_POSITION_KEY);
    if (posStr) {
      const pos = JSON.parse(posStr);
      // 验证位置有效性
      if (pos.x >= -100 && pos.y >= -100 && pos.x < 5000 && pos.y < 3000) {
        await currentWindow.setPosition(new LogicalPosition(pos.x, pos.y));
      } else {
        localStorage.removeItem(CALENDAR_POSITION_KEY);
      }
    }
  } catch (error) {
    console.error('恢复窗口尺寸失败:', error);
    // 出错时清除保存的数据
    localStorage.removeItem(CALENDAR_SIZE_KEY);
    localStorage.removeItem(CALENDAR_POSITION_KEY);
  }
}

// 设置预设尺寸
async function setPresetSize(preset: keyof typeof SIZE_PRESETS) {
  try {
    const currentWindow = getCurrentWindow();
    const size = SIZE_PRESETS[preset];
    await currentWindow.setSize(new LogicalSize(size.width, size.height));
    currentSizePreset.value = preset;
    showSizeMenu.value = false;

    // 保存尺寸
    localStorage.setItem(CALENDAR_SIZE_KEY, JSON.stringify({
      width: size.width,
      height: size.height
    }));
  } catch (error) {
    console.error('设置尺寸失败:', error);
  }
}

// 防抖保存尺寸
function debouncedSaveSize() {
  if (resizeTimer) {
    clearTimeout(resizeTimer);
  }
  resizeTimer = setTimeout(() => {
    saveWindowSize();
  }, 500);
}

// 监听视图模式和当前日期变化
watch([viewMode, currentDate], () => {
  loadTasks();
});

onMounted(async () => {
  // 默认选中今天
  selectedDate.value = formatDate(new Date());

  // 恢复窗口尺寸和位置
  await restoreWindowSize();

  // 恢复置顶和透明度设置
  await restoreWindowSettings();

  await taskCategoryStore.loadCategories();
  await loadTasks();
  await loadTags();

  // 检查 AI 是否启用
  try {
    isAiEnabled.value = await aiApi.isEnabled();
  } catch (error) {
    console.error('检查 AI 状态失败:', error);
    isAiEnabled.value = false;
  }

  // 监听任务更新事件
  unlistenTaskUpdate = await listen('task-updated', async () => {
    await taskCategoryStore.loadCategories(true);
    await loadTasks();
  });

  // 监听窗口尺寸变化
  const currentWindow = getCurrentWindow();
  unlistenResize = await currentWindow.onResized(() => {
    debouncedSaveSize();
  });

  // 定时刷新（每60秒）
  setInterval(loadTasks, 60000);

  // 点击其他地方关闭右键菜单和各种菜单
  document.addEventListener('click', (e) => {
    hideContextMenu();
    // 检查是否点击在尺寸菜单外
    const sizeMenu = document.querySelector('.size-menu');
    const sizeBtn = document.querySelector('.size-btn');
    if (sizeMenu && !sizeMenu.contains(e.target as Node) && !sizeBtn?.contains(e.target as Node)) {
      showSizeMenu.value = false;
    }
    // 检查是否点击在透明度菜单外
    const opacityMenu = document.querySelector('.opacity-menu');
    const opacityBtn = document.querySelector('.opacity-btn');
    if (opacityMenu && !opacityMenu.contains(e.target as Node) && !opacityBtn?.contains(e.target as Node)) {
      showOpacityMenu.value = false;
    }
  });
});

onUnmounted(() => {
  if (unlistenTaskUpdate) {
    unlistenTaskUpdate();
  }
  if (unlistenResize) {
    unlistenResize();
  }
  if (resizeTimer) {
    clearTimeout(resizeTimer);
  }
});
</script>

<style scoped>
.calendar-wrapper {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, var(--bg-base, rgba(15, 23, 42, 0.95)), var(--bg-surface, rgba(30, 41, 59, 0.92)));
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-family: 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  box-shadow:
    0 0 0 1px rgba(0, 0, 0, 0.2),
    0 20px 40px rgba(0, 0, 0, 0.6),
    0 0 20px color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
  color: var(--text-primary, #e2e8f0);
}

/* 主容器 - 双栏布局 */
.main-container {
  display: flex;
  flex: 1;
  overflow: hidden;
  min-height: 0;
}

/* 左侧区域 */
.left-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-right: 1px solid var(--border-default, rgba(255, 255, 255, 0.08));
}

/* 右侧区域 - 固定宽度 320px */
.right-section {
  width: 320px;
  display: flex;
  flex-direction: column;
  background: color-mix(in srgb, var(--bg-base, #0f172a) 60%, transparent);
  backdrop-filter: blur(10px);
  overflow: hidden;
}

/* 头部区域 */
.calendar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: linear-gradient(90deg, color-mix(in srgb, var(--accent-primary, #6366f1) 12%, transparent), color-mix(in srgb, var(--accent-secondary, #a78bfa) 12%, transparent));
  border-bottom: 1px solid var(--border-default, rgba(255, 255, 255, 0.08));
  cursor: move;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.handle-dots {
  display: flex;
  gap: 3px;
  opacity: 0.8;
}

.handle-dots span {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--accent-primary, #818cf8);
  box-shadow: 0 0 6px color-mix(in srgb, var(--accent-primary, #6366f1) 60%, transparent);
}

.title {
  font-size: 14px;
  font-weight: 700;
  background: linear-gradient(135deg, var(--accent-primary, #818cf8), var(--accent-secondary, #a78bfa));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.nav-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: color-mix(in srgb, var(--text-primary, #fff) 6%, transparent);
  color: var(--text-secondary, #94a3b8);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.nav-btn:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 25%, transparent);
  color: var(--text-primary, #fff);
  transform: translateY(-1px);
}

.date-display {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary, #e2e8f0);
  min-width: 120px;
  text-align: center;
}

/* 工具栏 */
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 5%, transparent);
  border-bottom: 1px solid var(--border-default, rgba(255, 255, 255, 0.05));
  flex-shrink: 0;
}

/* 搜索框 */
.search-box {
  position: relative;
  flex: 1;
  max-width: 300px;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-dim, #64748b);
  pointer-events: none;
  transition: color 0.3s;
}

.search-input {
  width: 100%;
  padding: 8px 12px 8px 38px;
  background: color-mix(in srgb, var(--text-primary, #fff) 5%, transparent);
  border: 1px solid var(--border-default, rgba(255, 255, 255, 0.1));
  border-radius: 10px;
  color: var(--text-primary, #e2e8f0);
  font-size: 13px;
  outline: none;
  transition: all 0.3s;
}

.search-input::placeholder {
  color: var(--text-dim, #64748b);
}

.search-input:focus {
  background: color-mix(in srgb, var(--text-primary, #fff) 8%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary, #6366f1) 50%, transparent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
}

.search-input:focus ~ .search-icon,
.search-box:focus-within .search-icon {
  color: var(--accent-primary, #818cf8);
}

/* 视图切换 */
.view-switch {
  display: flex;
  gap: 4px;
  background: color-mix(in srgb, var(--text-primary, #fff) 5%, transparent);
  padding: 4px;
  border-radius: 8px;
}

.view-btn {
  padding: 6px 14px;
  border: none;
  background: transparent;
  color: var(--text-secondary, #94a3b8);
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s;
}

.view-btn.active {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  color: var(--text-primary, #fff);
  box-shadow: 0 2px 8px color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
}

.view-btn:hover:not(.active) {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 15%, transparent);
  color: var(--text-primary, #e2e8f0);
}

/* 搜索结果 */
.search-results {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.search-results-header {
  padding: 12px 16px;
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 8%, transparent);
  border-bottom: 1px solid var(--border-default, rgba(255, 255, 255, 0.05));
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary, #94a3b8);
}

.search-results-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.search-results-list::-webkit-scrollbar {
  width: 6px;
}

.search-results-list::-webkit-scrollbar-track {
  background: transparent;
}

.search-results-list::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  border-radius: 3px;
}

.search-result-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  margin-bottom: 6px;
  background: color-mix(in srgb, var(--text-primary, #fff) 3%, transparent);
  border: 1px solid var(--border-default, rgba(255, 255, 255, 0.05));
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
}

.search-result-item:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 15%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  transform: translateX(4px);
}

.search-result-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.search-result-title {
  font-size: 13px;
  color: var(--text-primary, #e2e8f0);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.search-result-date {
  font-size: 11px;
  color: var(--text-dim, #64748b);
}

/* 日历内容区 */
.calendar-content {
  flex: 1;
  overflow: hidden;
  min-height: 0;
}

/* 右侧面板 */
.panel-header {
  padding: 16px;
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 8%, transparent);
  border-bottom: 1px solid var(--border-default, rgba(255, 255, 255, 0.08));
  flex-shrink: 0;
}

.panel-date-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.panel-date {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary, #e2e8f0);
}

.task-count {
  font-size: 12px;
  color: var(--text-dim, #64748b);
  display: flex;
  align-items: center;
  gap: 6px;
}

.task-count::before {
  content: '';
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--text-dim, #64748b);
}

/* 任务列表 */
.task-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.task-list::-webkit-scrollbar {
  width: 6px;
}

.task-list::-webkit-scrollbar-track {
  background: transparent;
}

.task-list::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  border-radius: 3px;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  border-radius: 10px;
  margin-bottom: 8px;
  background: color-mix(in srgb, var(--text-primary, #fff) 4%, transparent);
  border: 1px solid var(--border-default, rgba(255, 255, 255, 0.05));
  cursor: pointer;
  transition: all 0.2s;
}

.task-item:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 15%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  transform: translateX(4px);
}

.task-item-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.task-status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.task-status-dot.active {
  background: #6366f1;
  box-shadow: 0 0 8px rgba(99, 102, 241, 0.6);
}

.task-status-dot.todo {
  background: #64748b;
  box-shadow: 0 0 6px rgba(100, 116, 139, 0.4);
}

.task-status-dot.done {
  background: #10b981;
  box-shadow: 0 0 8px rgba(16, 185, 129, 0.5);
}

/* 已完成任务样式 */
.task-item.done {
  opacity: 0.7;
  background: rgba(16, 185, 129, 0.08);
  border-color: rgba(16, 185, 129, 0.2);
}

.task-item.done .task-title {
  text-decoration: line-through;
  color: #94a3b8;
}

.task-item.done .task-category {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
  border-color: rgba(16, 185, 129, 0.3);
}

.task-title {
  font-size: 13px;
  color: var(--text-primary, #e2e8f0);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-category {
  font-size: 10px;
  color: var(--text-dim, #64748b);
  padding: 3px 8px;
  background: color-mix(in srgb, var(--text-primary, #fff) 8%, transparent);
  border: 1px solid var(--border-default, rgba(255, 255, 255, 0.1));
  border-radius: 6px;
  flex-shrink: 0;
  align-self: flex-start;
}

.task-item.priority-1 {
  border-left: 3px solid #f43f5e;
}

.task-item.priority-2 {
  border-left: 3px solid #f59e0b;
}

.task-item.priority-3 {
  border-left: 3px solid #10b981;
}

.empty-hint {
  text-align: center;
  color: var(--text-dim, #64748b);
  font-size: 13px;
  padding: 40px 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.empty-hint::before {
  content: '📅';
  font-size: 48px;
  opacity: 0.3;
}

.action-bar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 8px 12px;
  background: color-mix(in srgb, var(--bg-base, #0f172a) 60%, transparent);
  border-top: 1px solid var(--border-default, rgba(255, 255, 255, 0.05));
  backdrop-filter: blur(4px);
  flex-shrink: 0;
}

.action-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: color-mix(in srgb, var(--text-primary, #fff) 3%, transparent);
  color: var(--text-secondary, #94a3b8);
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.action-btn:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  color: var(--text-primary, #fff);
  transform: translateY(-1px);
  box-shadow: 0 2px 8px color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
}

.action-btn.close:hover {
  background: rgba(244, 63, 94, 0.2);
  color: #f43f5e;
  box-shadow: 0 2px 8px rgba(244, 63, 94, 0.2);
}

.context-menu {
  position: absolute;
  background: color-mix(in srgb, var(--bg-surface, #1e293b) 95%, transparent);
  backdrop-filter: blur(12px);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  border-radius: 8px;
  padding: 6px;
  min-width: 140px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
  z-index: 1000;
  animation: fadeIn 0.1s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  font-size: 12px;
  color: var(--text-primary, #e2e8f0);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.menu-item:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  color: var(--text-primary, #fff);
}

.menu-item svg {
  color: var(--text-secondary, #94a3b8);
  transition: color 0.15s;
}

.menu-item:hover svg {
  color: var(--accent-primary, #818cf8);
}

.menu-divider {
  height: 1px;
  background: var(--border-default, rgba(255, 255, 255, 0.1));
  margin: 4px 0;
}

/* 尺寸选择菜单 */
.size-control {
  position: relative;
}

.size-btn.active {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  color: var(--accent-primary, #818cf8);
}

.size-menu {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 8px;
  background: color-mix(in srgb, var(--bg-surface, #1e293b) 98%, transparent);
  backdrop-filter: blur(12px);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 25%, transparent);
  border-radius: 10px;
  padding: 8px;
  min-width: 160px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  z-index: 1000;
  animation: slideUp 0.15s ease-out;
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.size-menu-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-dim, #64748b);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 4px 8px 8px;
  border-bottom: 1px solid var(--border-default, rgba(255, 255, 255, 0.08));
  margin-bottom: 4px;
}

.size-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.size-option:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 15%, transparent);
}

.size-option.active {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 25%, transparent);
}

.size-option.active .size-label {
  color: var(--accent-primary, #818cf8);
  font-weight: 600;
}

.size-label {
  font-size: 12px;
  color: var(--text-primary, #e2e8f0);
}

.size-dim {
  font-size: 10px;
  color: var(--text-dim, #64748b);
  font-family: 'Consolas', 'Monaco', monospace;
}

.size-hint {
  font-size: 10px;
  color: var(--text-muted, #475569);
  text-align: center;
  padding: 8px 4px 4px;
  border-top: 1px solid var(--border-default, rgba(255, 255, 255, 0.05));
  margin-top: 4px;
}

/* 透明度控制 */
.opacity-control {
  position: relative;
}

.opacity-btn.active {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  color: var(--accent-primary, #818cf8);
}

.opacity-menu {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 8px;
  background: color-mix(in srgb, var(--bg-surface, #1e293b) 98%, transparent);
  backdrop-filter: blur(12px);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 25%, transparent);
  border-radius: 10px;
  padding: 10px;
  min-width: 180px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  z-index: 1000;
  animation: slideUp 0.15s ease-out;
}

.opacity-menu-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-dim, #64748b);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 2px 4px 8px;
  border-bottom: 1px solid var(--border-default, rgba(255, 255, 255, 0.08));
  margin-bottom: 8px;
}

.opacity-slider {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.opacity-slider input[type="range"] {
  flex: 1;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  border-radius: 2px;
  outline: none;
}

.opacity-slider input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  background: var(--accent-primary, #6366f1);
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 0 6px color-mix(in srgb, var(--accent-primary, #6366f1) 50%, transparent);
  transition: transform 0.15s;
}

.opacity-slider input[type="range"]::-webkit-slider-thumb:hover {
  transform: scale(1.2);
}

.opacity-value {
  font-size: 11px;
  color: var(--text-secondary, #94a3b8);
  min-width: 36px;
  text-align: right;
  font-family: 'Consolas', 'Monaco', monospace;
}

.opacity-presets {
  display: flex;
  gap: 4px;
}

.opacity-preset-btn {
  flex: 1;
  padding: 5px 8px;
  border: none;
  background: color-mix(in srgb, var(--text-primary, #fff) 5%, transparent);
  color: var(--text-secondary, #94a3b8);
  border-radius: 4px;
  font-size: 10px;
  cursor: pointer;
  transition: all 0.15s;
}

.opacity-preset-btn:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  color: var(--text-primary, #e2e8f0);
}

.opacity-preset-btn.active {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  color: var(--accent-primary, #818cf8);
}

/* 置顶按钮激活状态 */
.action-btn.active {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  color: var(--accent-primary, #818cf8);
  box-shadow: 0 0 8px color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
}

/* 调整大小的边缘区域 */
.resize-edge {
  position: absolute;
  z-index: 100;
}

.resize-right {
  top: 20px;
  right: 0;
  width: 6px;
  height: calc(100% - 40px);
  cursor: ew-resize;
}

.resize-left {
  top: 20px;
  left: 0;
  width: 6px;
  height: calc(100% - 40px);
  cursor: ew-resize;
}

.resize-bottom {
  bottom: 0;
  left: 20px;
  width: calc(100% - 40px);
  height: 6px;
  cursor: ns-resize;
}

.resize-top {
  top: 0;
  left: 20px;
  width: calc(100% - 40px);
  height: 6px;
  cursor: ns-resize;
}

.resize-corner {
  bottom: 0;
  right: 0;
  width: 16px;
  height: 16px;
  cursor: nwse-resize;
  background: linear-gradient(135deg, transparent 50%, color-mix(in srgb, var(--accent-primary, #6366f1) 40%, transparent) 50%);
  border-radius: 0 0 16px 0;
}

.resize-edge:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
}

.resize-corner:hover {
  background: linear-gradient(135deg, transparent 40%, color-mix(in srgb, var(--accent-primary, #6366f1) 60%, transparent) 40%);
}

/* ========== 任务表单样式 ========== */
.task-form-container {
  max-height: 65vh;
  overflow-y: auto;
  padding-right: 8px;
}

.task-form-container::-webkit-scrollbar {
  width: 6px;
}

.task-form-container::-webkit-scrollbar-track {
  background: transparent;
}

.task-form-container::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  border-radius: 3px;
}

.input-with-ai {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  width: 100%;
}

.input-with-ai :deep(.n-input) {
  flex: 1;
}

.input-with-ai-vertical {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.ai-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.ai-btn {
  flex-shrink: 0;
  font-size: 12px;
}

.ai-btn-small {
  font-size: 11px;
  color: var(--accent-primary, #818cf8);
}

.ai-btn-small:hover {
  color: var(--accent-secondary, #a5b4fc);
}

.date-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

/* 任务表单弹窗深色主题适配 */
:deep(.task-modal) {
  background: color-mix(in srgb, var(--bg-surface, #1e293b) 98%, transparent);
  backdrop-filter: blur(20px);
  border: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
}

:deep(.task-modal .n-card-header) {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
  border-bottom: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
}

:deep(.task-modal .n-card-header__main) {
  color: var(--text-primary, #e2e8f0);
  font-weight: 600;
}

:deep(.task-modal .n-card__content) {
  background: transparent;
  padding: 20px !important;
}

:deep(.task-modal .n-card__footer) {
  background: color-mix(in srgb, var(--bg-base, #0f172a) 50%, transparent);
  border-top: 1px solid color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  padding: 12px 20px !important;
}

:deep(.n-form-item-label__text) {
  color: var(--text-secondary, #94a3b8);
  font-weight: 500;
}
</style>
