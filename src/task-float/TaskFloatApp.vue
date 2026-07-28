<template>
  <n-config-provider :theme="darkTheme">
    <n-dialog-provider>
      <n-message-provider>
        <div class="task-float-wrapper">
        <!-- 拖动区域 -->
        <div class="drag-handle" data-tauri-drag-region>
          <div class="handle-dots">
            <span></span><span></span><span></span>
          </div>
        </div>

        <!-- 任务列表 -->
        <div class="task-list">
          <!-- 进行中的任务 -->
          <div v-if="activeTasks.length > 0" class="task-section">
            <div class="section-header">
              <div class="status-dot active"></div>
              <span>进行中</span>
              <span class="count">{{ activeTasks.length }}</span>
            </div>
            <div
              v-for="task in activeTasks"
              :key="task.id"
              class="task-item active"
              @click="toggleTaskDetail(task)"
              @contextmenu.prevent="showContextMenu($event, task)"
            >
              <span class="task-title">{{ task.title }}</span>
              <!-- 展开的详情 -->
              <div v-if="expandedTaskId === task.id" class="task-detail">
                <div v-if="task.description" class="detail-row">
                  <span class="detail-label">描述:</span>
                  <span class="detail-value description">{{ task.description }}</span>
                </div>
                <div v-if="task.gitBranch" class="detail-row">
                  <span class="detail-label">分支:</span>
                  <span class="detail-value branch">{{ task.gitBranch }}</span>
                </div>
                <div v-if="task.startedAt" class="detail-row">
                  <span class="detail-label">开始:</span>
                  <span class="detail-value">{{ formatTime(task.startedAt) }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 待办任务 -->
          <div v-if="todoTasks.length > 0" class="task-section">
            <div class="section-header">
              <div class="status-dot todo"></div>
              <span>待办</span>
              <span class="count">{{ todoTasks.length }}</span>
            </div>
            <div
              v-for="task in displayedTodoTasks"
              :key="task.id"
              class="task-item todo"
              @click="toggleTaskDetail(task)"
              @contextmenu.prevent="showContextMenu($event, task)"
            >
              <span class="task-title">{{ task.title }}</span>
              <!-- 展开的详情 -->
              <div v-if="expandedTaskId === task.id" class="task-detail">
                <div v-if="task.description" class="detail-row">
                  <span class="detail-label">描述:</span>
                  <span class="detail-value description">{{ task.description }}</span>
                </div>
                <div v-if="task.dueDate" class="detail-row">
                  <span class="detail-label">截止:</span>
                  <span class="detail-value">{{ formatTime(task.dueDate) }}</span>
                </div>
              </div>
            </div>
            <div v-if="todoTasks.length > maxDisplayTodo && !showAllTodo" class="more-hint" @click="showAllTodo = true">
              点击展开剩余 {{ todoTasks.length - maxDisplayTodo }} 个任务
            </div>
          </div>

          <!-- 空状态 -->
          <div v-if="activeTasks.length === 0 && todoTasks.length === 0" class="empty-state">
            <span>暂无任务</span>
          </div>
        </div>

        <!-- 底部操作栏 -->
        <div class="action-bar">
          <button class="action-btn add" @click="handleAddTask" title="新增任务">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
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
          <!-- 新增任务选项 -->
          <div class="menu-item" @click="handleAddTask">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
            <span>新增任务</span>
          </div>
          <!-- 编辑任务选项 -->
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
        </div>

        <!-- 新增/编辑任务弹框 -->
        <n-modal
          v-model:show="showTaskModal"
          preset="card"
          :title="isEditing ? '编辑任务' : '新建任务'"
          style="width: 780px; max-height: 90vh;"
          to="body"
          :block-scroll="false"
        >
          <div style="max-height: 70vh; overflow-y: auto; padding-right: 8px;">
            <n-form ref="formRef" :model="formData" :rules="formRules">
              <n-form-item label="任务标题" path="title">
                <n-input
                  v-model:value="formData.title"
                  placeholder="请输入任务标题"
                  :maxlength="200"
                  show-count
                />
              </n-form-item>
              <n-form-item label="任务描述" path="description">
                <n-space vertical style="width: 100%;">
                  <n-input
                    v-model:value="formData.description"
                    type="textarea"
                    placeholder="请输入任务描述（最多10000字符）"
                    :rows="3"
                    :maxlength="10000"
                    show-count
                  />
                  <n-space>
                    <n-button
                      size="small"
                      @click="handleAiClassify"
                      :loading="aiClassifying"
                      :disabled="!formData.title"
                    >
                      <template #icon v-if="!aiClassifying">
                        <n-icon><GridOutline /></n-icon>
                      </template>
                      AI 智能分类
                    </n-button>
                    <n-button
                      size="small"
                      @click="handleAiEnhance"
                      :loading="aiEnhancing"
                      :disabled="!formData.title"
                      type="primary"
                      ghost
                    >
                      <template #icon v-if="!aiEnhancing">
                        <span style="font-size: 14px;">✨</span>
                      </template>
                      {{ aiEnhancing ? 'AI 正在思考中...' : 'AI 增强描述' }}
                    </n-button>
                    <n-button
                      size="small"
                      @click="handleAiGenerateSubtasks"
                      :loading="aiGeneratingSubtasks"
                      :disabled="!formData.title"
                    >
                      <template #icon v-if="!aiGeneratingSubtasks">
                        <n-icon><AddOutline /></n-icon>
                      </template>
                      AI 生成子任务
                    </n-button>
                  </n-space>
                </n-space>
              </n-form-item>

              <!-- 分类选择器 -->
              <n-form-item label="分类" path="category">
                <CategorySelector v-model="formData.category" />
              </n-form-item>

              <!-- 四象限选择器 -->
              <n-form-item label="四象限" path="quadrant">
                <QuadrantSelector v-model="formData.quadrant" />
              </n-form-item>

              <!-- 优先级和标签在同一行 -->
              <div style="display: grid; grid-template-columns: 1fr 2fr; gap: 16px;">
                <n-form-item label="优先级" path="priority">
                  <PrioritySelector v-model="formData.priority" />
                </n-form-item>
                <n-form-item label="标签" path="tagIds">
                  <TagSelector
                    v-model="formData.tagIds"
                    :available-tags="availableTags"
                    @manage="showTagManager = true"
                  />
                </n-form-item>
              </div>

              <!-- 日期选择在同一行 -->
              <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px;">
                <n-form-item label="登记日期" path="registeredAt">
                  <n-date-picker
                    v-model:value="formData.registeredAt"
                    type="date"
                    clearable
                    placeholder="选择登记日期"
                    style="width: 100%;"
                  />
                </n-form-item>
                <n-form-item label="截止日期" path="dueDate">
                  <n-date-picker
                    v-model:value="formData.dueDate"
                    type="date"
                    clearable
                    placeholder="选择截止日期（可选）"
                    style="width: 100%;"
                  />
                </n-form-item>
              </div>

              <!-- 计划开始时间 -->
              <n-form-item label="计划开始时间" path="scheduledStartTime">
                <n-date-picker
                  v-model:value="formData.scheduledStartTime"
                  type="datetime"
                  clearable
                  placeholder="选择计划开始时间，到时间会提醒您"
                  style="width: 100%;"
                  format="yyyy-MM-dd HH:mm:ss"
                />
              </n-form-item>
            </n-form>
          </div>
          <template #footer>
            <n-space justify="space-between" style="width: 100%;">
              <span v-if="!isEditing && isAiEnabled" class="ai-status-hint">
                <span class="ai-dot"></span>
                AI 智能分析已启用
              </span>
              <span v-else></span>
              <n-space>
                <n-button @click="handleCancelTask" :disabled="isSubmitting">取消</n-button>
                <n-button
                  type="primary"
                  @click="handleSubmitTask"
                  :loading="isSubmitting"
                  :disabled="isSubmitting"
                >
                  {{ isSubmitting ? 'AI 分析中...' : (isEditing ? '保存' : (isAiEnabled ? 'AI 智能创建' : '创建')) }}
                </n-button>
              </n-space>
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
        </div>
      </n-message-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, watch } from 'vue';
import { NConfigProvider, NDialogProvider, NMessageProvider, NModal, NForm, NFormItem, NInput, NDatePicker, NButton, NSpace, NIcon, darkTheme, useMessage } from 'naive-ui';
import { GridOutline, AddOutline } from '@vicons/ionicons5';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, Window, LogicalSize, LogicalPosition } from '@tauri-apps/api/window';
import { listen, emit } from '@tauri-apps/api/event';
import dayjs from 'dayjs';
import type { Task, TaskQuadrant, Tag } from '@/types/task';
import { aiApi } from '@/api/aiApi';
import { tagApi } from '@/api/tagApi';
import { useTaskCategoryStore } from '@/stores/taskCategoryStore';
import QuadrantSelector from '@/components/QuadrantSelector.vue';
import PrioritySelector from '@/components/PrioritySelector.vue';
import CategorySelector from '@/components/CategorySelector.vue';
import TagSelector from '@/components/TagSelector.vue';
import TagManager from '@/components/TagManager.vue';

const taskCategoryStore = useTaskCategoryStore();
const activeTasks = ref<Task[]>([]);
const todoTasks = ref<Task[]>([]);
const maxDisplayTodo = 5;
const showAllTodo = ref(false);
const expandedTaskId = ref<number | null>(null);

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

// 新增/编辑任务弹框状态
const showTaskModal = ref(false);
const isEditing = ref(false);
const editingTaskId = ref<number | null>(null);
const isSubmitting = ref(false);
const formRef = ref();

// 窗口状态（用于弹框打开/关闭时调整窗口大小）
const originalWindowSize = ref<{ width: number; height: number } | null>(null);
const originalWindowPosition = ref<{ x: number; y: number } | null>(null);
const MODAL_WINDOW_SIZE = { width: 820, height: 750 };
const ORIGINAL_WINDOW_SIZE = { width: 300, height: 550 };

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

const displayedTodoTasks = computed(() => {
  if (showAllTodo.value) {
    return todoTasks.value;
  }
  return todoTasks.value.slice(0, maxDisplayTodo);
});

let unlistenTaskUpdate: (() => void) | null = null;

onMounted(async () => {
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

  // 定时刷新（每30秒）
  setInterval(loadTasks, 30000);

  // 点击其他地方关闭右键菜单
  document.addEventListener('click', hideContextMenu);
});

onUnmounted(() => {
  if (unlistenTaskUpdate) {
    unlistenTaskUpdate();
  }
  document.removeEventListener('click', hideContextMenu);
});

// 监听弹框关闭（用户点击 X 按钮或点击遮罩关闭时）
watch(showTaskModal, async (newVal, oldVal) => {
  if (!newVal && oldVal && originalWindowSize.value) {
    // 弹框被关闭且窗口处于扩大状态，恢复窗口大小
    await restoreWindow();
  }
});

async function loadTasks() {
  try {
    const tasks: Task[] = await invoke('get_all_tasks');
    activeTasks.value = tasks.filter(t => t.status === 'active');
    todoTasks.value = tasks.filter(t => t.status === 'todo');
  } catch (error) {
    console.error('加载任务失败:', error);
  }
}

async function loadTags() {
  try {
    availableTags.value = await tagApi.getAllTags();
  } catch (error) {
    console.error('加载标签失败:', error);
  }
}

function toggleTaskDetail(task: Task) {
  if (expandedTaskId.value === task.id) {
    expandedTaskId.value = null;
  } else {
    expandedTaskId.value = task.id ?? null;
  }
}

function showContextMenu(event: MouseEvent, task: Task) {
  const x = event.clientX;
  const y = event.clientY;

  const menuWidth = 130;
  const menuHeight = 180;
  const maxX = window.innerWidth - menuWidth - 10;
  const maxY = window.innerHeight - menuHeight - 10;

  contextMenu.value = {
    visible: true,
    x: Math.min(x, maxX),
    y: Math.min(y, maxY),
    task,
  };
}

function hideContextMenu() {
  contextMenu.value.visible = false;
}

// 扩大窗口以显示弹框
async function expandWindow() {
  try {
    const currentWindow = getCurrentWindow();

    // 保存当前窗口位置和大小
    const position = await currentWindow.outerPosition();
    const size = await currentWindow.innerSize();

    originalWindowPosition.value = { x: position.x, y: position.y };
    originalWindowSize.value = { width: size.width, height: size.height };

    // 扩大窗口
    await currentWindow.setSize(new LogicalSize(MODAL_WINDOW_SIZE.width, MODAL_WINDOW_SIZE.height));

    // 居中显示
    await currentWindow.center();
  } catch (error) {
    console.error('扩大窗口失败:', error);
  }
}

// 恢复窗口大小
async function restoreWindow() {
  try {
    const currentWindow = getCurrentWindow();

    // 恢复原来的大小
    await currentWindow.setSize(new LogicalSize(ORIGINAL_WINDOW_SIZE.width, ORIGINAL_WINDOW_SIZE.height));

    // 恢复原来的位置
    if (originalWindowPosition.value) {
      await currentWindow.setPosition(new LogicalPosition(originalWindowPosition.value.x, originalWindowPosition.value.y));
    }

    originalWindowSize.value = null;
    originalWindowPosition.value = null;
  } catch (error) {
    console.error('恢复窗口失败:', error);
  }
}

// 新增任务
async function handleAddTask() {
  isEditing.value = false;
  editingTaskId.value = null;
  resetFormData();
  hideContextMenu();

  await expandWindow();
  showTaskModal.value = true;
}

// 编辑任务
async function handleEditTask() {
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

  hideContextMenu();

  await expandWindow();
  showTaskModal.value = true;
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
async function handleCancelTask() {
  showTaskModal.value = false;
  resetFormData();
  await restoreWindow();
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
    await restoreWindow();
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

function formatTime(timeStr: string) {
  if (!timeStr) return '';
  const date = new Date(timeStr);
  return `${date.getMonth() + 1}-${date.getDate()} ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
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
  await emit('task-float-hidden');
  await currentWindow.hide();
}
</script>

<style scoped>
/* 赛博朋克/极客风格优化 */
.task-float-wrapper {
  width: 100%;
  height: 100%;
  max-height: 100vh;
  /* 深色半透明背景 + 模糊效果 */
  background: var(--bg-surface);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  /* 边框发光效果 */
  border: 1px solid var(--accent-glow);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-family: 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  /* 强烈的阴影 */
  box-shadow: var(--shadow-lg), var(--shadow-glow);
  color: var(--text-secondary);
  transition: all 0.3s ease;
}

.drag-handle {
  height: 28px;
  min-height: 28px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: move;
  /* 顶部渐变条 */
  background: linear-gradient(90deg, var(--accent-glow), rgba(167, 139, 250, 0.1));
  border-bottom: 1px solid var(--border-default);
}

.handle-dots {
  display: flex;
  gap: 4px;
  opacity: 0.8;
}

.handle-dots span {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--accent-primary);
  box-shadow: 0 0 6px var(--accent-glow);
}

.task-list {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 12px;
  min-height: 0;
  max-height: calc(100vh - 80px);
}

/* 滚动条美化 */
.task-list::-webkit-scrollbar {
  width: 4px;
}

.task-list::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
}

.task-list::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 2px;
}

.task-list::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
}

.task-section {
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 4px 8px;
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  flex-shrink: 0;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  position: relative;
}

.status-dot.active {
  background: var(--accent-primary);
  box-shadow: 0 0 8px var(--accent-primary);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { box-shadow: 0 0 0 0 rgba(99, 102, 241, 0.4); }
  70% { box-shadow: 0 0 0 4px rgba(99, 102, 241, 0); }
  100% { box-shadow: 0 0 0 0 rgba(99, 102, 241, 0); }
}

.status-dot.todo {
  background: var(--text-dim);
}

.count {
  margin-left: auto;
  background: var(--accent-glow);
  color: var(--accent-primary);
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-family: 'Consolas', monospace;
}

.task-item {
  padding: 10px;
  border-radius: 8px;
  margin-bottom: 8px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  user-select: none;
  border: 1px solid transparent;
  position: relative;
  overflow: hidden;
}

.task-item.active {
  background: var(--accent-glow);
  border-color: var(--border-active);
  box-shadow: var(--shadow-md);
}

.task-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: var(--accent-primary);
  box-shadow: 0 0 8px var(--accent-glow);
}

.task-item.todo {
  background: var(--bg-overlay);
  border-color: var(--border-default);
}

.task-item:hover {
  transform: translateY(-1px);
  background: var(--bg-hover);
  border-color: var(--border-hover);
}

.task-title {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 500;
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-detail {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--border-default);
  animation: slideDown 0.2s ease-out;
}

@keyframes slideDown {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

.detail-row {
  display: flex;
  gap: 8px;
  font-size: 11px;
  margin-bottom: 4px;
  line-height: 1.5;
  align-items: flex-start;
}

.detail-label {
  color: var(--text-dim);
  flex-shrink: 0;
  font-size: 10px;
  text-transform: uppercase;
}

.detail-value {
  color: var(--text-secondary);
  flex: 1;
}

.detail-value.description {
  max-height: 100px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-word;
  padding-right: 4px;
}

.detail-value.description::-webkit-scrollbar {
  width: 3px;
}

.detail-value.description::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
}

.detail-value.description::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 2px;
}

.detail-value.branch {
  color: var(--accent-secondary);
  font-family: 'Consolas', monospace;
  background: rgba(167, 139, 250, 0.1);
  padding: 1px 4px;
  border-radius: 3px;
}

.more-hint {
  font-size: 11px;
  color: var(--accent-primary);
  padding: 8px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
  border-radius: 6px;
}

.more-hint:hover {
  background: var(--accent-glow);
  color: var(--accent-primary-hover);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100px;
  color: var(--text-dim);
  font-size: 12px;
  gap: 8px;
}

.empty-state::before {
  content: '';
  width: 32px;
  height: 32px;
  background: rgba(148, 163, 184, 0.1);
  border-radius: 50%;
  mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'%3E%3Cpath d='M9 11l3 3L22 4'/%3E%3Cpath d='M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11'/%3E%3C/svg%3E") no-repeat center;
  -webkit-mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'%3E%3Cpath d='M9 11l3 3L22 4'/%3E%3Cpath d='M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11'/%3E%3C/svg%3E") no-repeat center;
  background-color: currentColor;
}

.action-bar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-overlay);
  border-top: 1px solid var(--border-default);
  backdrop-filter: blur(4px);
  flex-shrink: 0;
}

.action-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: var(--button-default-bg);
  color: var(--text-muted);
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.action-btn:hover {
  background: var(--accent-glow);
  color: var(--text-primary);
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.action-btn.add:hover {
  background: rgba(34, 197, 94, 0.2);
  color: var(--success);
  box-shadow: 0 2px 8px rgba(34, 197, 94, 0.2);
}

.action-btn.close:hover {
  background: rgba(244, 63, 94, 0.2);
  color: var(--error);
  box-shadow: 0 2px 8px rgba(244, 63, 94, 0.2);
}

/* 右键菜单优化 */
.context-menu {
  position: absolute;
  background: var(--bg-elevated);
  backdrop-filter: blur(12px);
  border: 1px solid var(--border-active);
  border-radius: 8px;
  padding: 6px;
  min-width: 140px;
  box-shadow: var(--shadow-lg);
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
  color: var(--text-secondary);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.menu-item:hover {
  background: var(--accent-glow);
  color: var(--text-primary);
}

.menu-item svg {
  color: var(--text-muted);
  transition: color 0.15s;
}

.menu-item:hover svg {
  color: var(--accent-primary);
}

/* 菜单分隔线 */
.menu-divider {
  height: 1px;
  background: var(--border-default);
  margin: 4px 8px;
}

/* AI 状态提示 */
.ai-status-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--accent-secondary);
  padding: 6px 12px;
  background: rgba(139, 92, 246, 0.1);
  border-radius: 6px;
  border: 1px solid rgba(139, 92, 246, 0.2);
}

.ai-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent-secondary);
  box-shadow: 0 0 8px rgba(167, 139, 250, 0.6);
  animation: pulse-dot 2s ease-in-out infinite;
}

@keyframes pulse-dot {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}
</style>
