<template>
  <div class="task-board">
    <!-- Header -->
    <div class="board-header">
      <h2 class="board-title">任务看板</h2>
      <n-space>
        <n-button class="primary-button" @click="showCreateModal = true">
          <template #icon>
            <n-icon><AddOutline /></n-icon>
          </template>
          新建任务
        </n-button>
        <n-button @click="showImportModal = true">
          <template #icon>
            <n-icon><CloudUploadOutline /></n-icon>
          </template>
          导入任务
        </n-button>
        <n-button @click="handleRefresh">
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          刷新
        </n-button>
      </n-space>
    </div>

    <!-- 筛选器 -->
    <div class="filters-section">
      <n-space align="center" :size="16">
        <!-- 四象限筛选 -->
        <div class="filter-group">
          <span class="filter-label">
            <n-icon size="14"><GridOutline /></n-icon>
            四象限:
          </span>
          <n-select
            v-model:value="filterQuadrant"
            :options="quadrantFilterOptions"
            placeholder="全部"
            clearable
            style="width: 160px"
            size="small"
          />
        </div>

        <!-- 标签筛选 -->
        <div class="filter-group">
          <span class="filter-label">
            <n-icon size="14"><PricetagsOutline /></n-icon>
            标签:
          </span>
          <n-select
            v-model:value="filterTags"
            :options="tagFilterOptions"
            placeholder="全部标签"
            multiple
            clearable
            style="width: 200px"
            size="small"
          />
        </div>

        <!-- 重置筛选 -->
        <n-button
          v-if="filterQuadrant || filterTags.length > 0"
          text
          size="small"
          @click="resetFilters"
        >
          <template #icon>
            <n-icon><CloseCircleOutline /></n-icon>
          </template>
          重置筛选
        </n-button>

        <!-- 筛选结果统计 -->
        <span v-if="isFiltering" class="filter-stats">
          筛选结果: {{ filteredTasksCount }} 个任务
        </span>
      </n-space>
    </div>

    <!-- Three Column Board -->
    <div class="board-columns">
      <!-- 待办列 -->
      <div class="board-column">
        <div class="column-header">
          <div class="column-title">
            <div class="status-dot status-todo"></div>
            <span class="title-text">待办</span>
          </div>
          <span class="task-count">{{ taskStore.todoTasks.length }}</span>
        </div>

        <div class="column-content custom-scrollbar">
          <TaskCard
            v-for="task in filteredTodoTasks"
            :key="task.id"
            :task="task"
            @start="handleStart"
            @complete="handleComplete"
            @edit="handleEdit"
            @delete="handleDelete"
            @defer="handleDefer"
            @click="handleTaskClick"
            class="task-card-item"
          />

          <div v-if="filteredDeferredTasks.length > 0" class="deferred-section">
            <div class="deferred-header">
              <span class="deferred-title">延后任务</span>
              <span class="deferred-count">{{ filteredDeferredTasks.length }}</span>
            </div>
            <TaskCard
              v-for="task in filteredDeferredTasks"
              :key="task.id"
              :task="task"
              @start="handleStart"
              @complete="handleComplete"
              @edit="handleEdit"
              @delete="handleDelete"
              @click="handleTaskClick"
              class="task-card-item deferred-task"
            />
          </div>

          <button class="add-card-button" @click="showCreateModal = true">
            <n-icon size="14"><AddOutline /></n-icon>
            <span>添加卡片</span>
          </button>
        </div>
      </div>

      <!-- 进行中列 -->
      <div class="board-column">
        <div class="column-header">
          <div class="column-title">
            <div class="status-dot status-doing"></div>
            <span class="title-text">进行中</span>
          </div>
          <span class="task-count">{{ taskStore.activeTasks.length }}</span>
        </div>

        <div class="column-content custom-scrollbar">
          <TaskCard
            v-for="task in filteredActiveTasks"
            :key="task.id"
            :task="task"
            @pause="handlePause"
            @complete="handleComplete"
            @edit="handleEdit"
            @click="handleTaskClick"
            class="task-card-item"
          />

          <n-empty
            v-if="filteredActiveTasks.length === 0"
            description="暂无进行中的任务"
            class="empty-placeholder"
          />
        </div>
      </div>

      <!-- 已完成列 -->
      <div class="board-column">
        <div class="column-header">
          <div class="column-title">
            <div class="status-dot status-done"></div>
            <span class="title-text">已完成</span>
          </div>
          <span class="task-count">{{ taskStore.completedTasks.length }}</span>
        </div>

        <div class="column-content custom-scrollbar">
          <TaskCard
            v-for="task in filteredCompletedTasks"
            :key="task.id"
            :task="task"
            @click="handleTaskClick"
            readonly
            class="task-card-item"
          />

          <n-empty
            v-if="filteredCompletedTasks.length === 0"
            description="暂无已完成任务"
            class="empty-placeholder"
          />
        </div>
      </div>
    </div>

    <!-- 创建/编辑任务对话框 -->
    <n-modal v-model:show="showCreateModal" preset="card" :title="isEditing ? '编辑任务' : '新建任务'" style="width: 700px; max-height: 90vh;">
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
                    <n-icon>✨</n-icon>
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
            <n-button @click="handleCancelEdit" :disabled="isCreating">取消</n-button>
            <n-button
              class="primary-button"
              @click="isEditing ? handleUpdate() : handleCreate()"
              :loading="isCreating"
              :disabled="isCreating"
            >
              {{ isCreating ? 'AI 分析中...' : (isEditing ? '保存' : (isAiEnabled ? 'AI 智能创建' : '创建')) }}
            </n-button>
          </n-space>
        </n-space>
      </template>
    </n-modal>

    <!-- 导入任务对话框 -->
    <n-modal v-model:show="showImportModal" :mask-closable="false">
      <TaskImport @close="showImportModal = false" @success="handleImportSuccess" />
    </n-modal>

    <!-- 任务详情弹窗 -->
    <TaskDetailModal
      :task="selectedTask"
      v-model:show="showDetailModal"
      @update="handleTaskUpdate"
    />

    <!-- 标签管理弹窗 -->
    <TagManager
      v-model:show="showTagManager"
      :tags="availableTags"
      @create="handleCreateTag"
      @update="handleUpdateTag"
      @delete="handleDeleteTag"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue';
import { NCard, NSpace, NButton, NIcon, NEmpty, NCollapse, NCollapseItem, NModal, NForm, NFormItem, NInput, NSelect, useMessage } from 'naive-ui';
import { AddOutline, RefreshOutline, CloudUploadOutline, GridOutline, PricetagsOutline, CloseCircleOutline } from '@vicons/ionicons5';
import { useTaskStore } from '@/stores/taskStore';
import { tagApi } from '@/api/tagApi';
import { aiApi } from '@/api/aiApi';
import TaskCard from '@/components/TaskCard.vue';
import TaskImport from '@/components/TaskImport.vue';
import TaskDetailModal from '@/components/TaskDetailModal.vue';
import QuadrantSelector from '@/components/QuadrantSelector.vue';
import TagSelector from '@/components/TagSelector.vue';
import TagManager from '@/components/TagManager.vue';
import PrioritySelector from '@/components/PrioritySelector.vue';
import CategorySelector from '@/components/CategorySelector.vue';
import { CATEGORY_LABELS, PRIORITY_LABELS, QUADRANT_LABELS } from '@/types/task';
import type { Task, TaskQuadrant, Tag } from '@/types/task';

const taskStore = useTaskStore();
const message = useMessage();
const showCreateModal = ref(false);
const showImportModal = ref(false);
const showDetailModal = ref(false);
const showTagManager = ref(false);
const selectedTask = ref<Task | null>(null);
const formRef = ref();
const isEditing = ref(false);
const editingTaskId = ref<number | null>(null);

// AI 功能加载状态
const aiClassifying = ref(false);
const aiEnhancing = ref(false);
const aiGeneratingSubtasks = ref(false);
const isAiEnabled = ref(false);
const isCreating = ref(false); // 创建任务中状态

// AI 增强描述缓存（基于标题和描述内容）
const enhanceCache = new Map<string, string>();
const MAX_CACHE_SIZE = 50; // 最大缓存数量

// 防抖定时器
let aiEnhanceDebounceTimer: ReturnType<typeof setTimeout> | null = null;

// 清理缓存（当缓存超过限制时）
function cleanupCache() {
  if (enhanceCache.size > MAX_CACHE_SIZE) {
    // 保留最近的一半缓存，删除旧的
    const entries = Array.from(enhanceCache.entries());
    const toKeep = entries.slice(-Math.floor(MAX_CACHE_SIZE / 2));
    enhanceCache.clear();
    toKeep.forEach(([key, value]) => enhanceCache.set(key, value));
  }
}

// 标签数据
const availableTags = ref<Tag[]>([]);

// 筛选器状态
const filterQuadrant = ref<TaskQuadrant | null>(null);
const filterTags = ref<number[]>([]);

const formData = reactive({
  title: '',
  description: '',
  category: 'other' as Task['category'],
  priority: 2 as Task['priority'],
  quadrant: 'urgent_not_important' as TaskQuadrant,
  tagIds: [] as number[],
});

const formRules = {
  title: {
    required: true,
    message: '请输入任务标题',
    trigger: 'blur',
  },
};

const categoryOptions = Object.entries(CATEGORY_LABELS).map(([value, label]) => ({
  label,
  value,
}));

const priorityOptions = Object.entries(PRIORITY_LABELS).map(([value, label]) => ({
  label,
  value: Number(value),
}));

// 四象限筛选选项
const quadrantFilterOptions = Object.entries(QUADRANT_LABELS).map(([value, label]) => ({
  label,
  value: value as TaskQuadrant,
}));

// 标签筛选选项（Mock数据，实际应从store获取）
const tagFilterOptions = computed(() => {
  // 从所有任务中提取唯一标签
  const allTags = new Map<number, { id: number; name: string; color: string }>();

  [...taskStore.todoTasks, ...taskStore.activeTasks, ...taskStore.completedTasks, ...taskStore.deferredTasks].forEach(task => {
    task.tags?.forEach(tag => {
      if (tag.id && !allTags.has(tag.id)) {
        allTags.set(tag.id, tag);
      }
    });
  });

  return Array.from(allTags.values()).map(tag => ({
    label: tag.name,
    value: tag.id,
  }));
});

// 筛选逻辑
const filterTasks = (tasks: Task[]) => {
  return tasks.filter(task => {
    // 四象限筛选
    if (filterQuadrant.value && task.quadrant !== filterQuadrant.value) {
      return false;
    }

    // 标签筛选
    if (filterTags.value.length > 0) {
      const taskTagIds = task.tags?.map(t => t.id).filter(id => id !== undefined) || [];
      const hasMatchingTag = filterTags.value.some(tagId => taskTagIds.includes(tagId));
      if (!hasMatchingTag) {
        return false;
      }
    }

    return true;
  });
};

const filteredTodoTasks = computed(() => filterTasks(taskStore.todoTasks));
const filteredActiveTasks = computed(() => filterTasks(taskStore.activeTasks));
const filteredCompletedTasks = computed(() => filterTasks(taskStore.completedTasks));
const filteredDeferredTasks = computed(() => filterTasks(taskStore.deferredTasks));

const isFiltering = computed(() => !!filterQuadrant.value || filterTags.value.length > 0);

const filteredTasksCount = computed(() =>
  filteredTodoTasks.value.length +
  filteredActiveTasks.value.length +
  filteredCompletedTasks.value.length +
  filteredDeferredTasks.value.length
);

const resetFilters = () => {
  filterQuadrant.value = null;
  filterTags.value = [];
};

onMounted(async () => {
  await taskStore.loadTasks();
  await taskStore.loadCompletedTasks(7);

  // 加载标签
  await loadTags();

  // 检查 AI 是否启用
  try {
    isAiEnabled.value = await aiApi.isEnabled();
  } catch (error) {
    console.error('检查 AI 状态失败:', error);
    isAiEnabled.value = false;
  }

  // 检查僵尸任务
  const staleTasks = await taskStore.checkStaleTasks(3);
  if (staleTasks.length > 0) {
    message.warning(`发现 ${staleTasks.length} 个任务超过 3 天未处理`);
  }
});

// 监听快捷键 Ctrl+N 新建任务
onMounted(() => {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.ctrlKey && e.key === 'n') {
      e.preventDefault();
      showCreateModal.value = true;
    }
  };
  window.addEventListener('keydown', handleKeyDown);
  return () => window.removeEventListener('keydown', handleKeyDown);
});

async function handleCreate() {
  try {
    await formRef.value?.validate();

    // 前端额外验证
    if (formData.title.trim().length === 0) {
      message.error('任务标题不能为空');
      return;
    }
    if (formData.title.length > 200) {
      message.error('任务标题不能超过200个字符');
      return;
    }
    if (formData.description && formData.description.length > 10000) {
      message.error('任务描述不能超过10000个字符');
      return;
    }

    // 防止重复提交
    if (isCreating.value) return;
    isCreating.value = true;

    let finalCategory = formData.category;
    let finalPriority = formData.priority;
    let finalQuadrant = formData.quadrant;
    let aiSuggestedTagIds: number[] = [];

    // 如果 AI 启用，自动进行 AI 分类
    if (isAiEnabled.value) {
      try {
        const existingTagNames = availableTags.value.map(t => t.name);
        const result = await aiApi.classifyTask(
          formData.title,
          formData.description || undefined,
          existingTagNames
        );

        // 使用 AI 推荐的分类（如果用户没有手动修改过默认值）
        if (formData.category === 'other' && result.category) {
          finalCategory = result.category as Task['category'];
        }
        if (formData.priority === 2 && result.priority) {
          finalPriority = result.priority as Task['priority'];
        }
        if (formData.quadrant === 'urgent_not_important' && result.quadrant) {
          finalQuadrant = result.quadrant as TaskQuadrant;
        }

        // 处理 AI 推荐的标签
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
        // AI 失败不阻止创建，继续使用用户选择的值
      }
    }

    // 创建任务
    const taskId = await taskStore.createTask(
      formData.title,
      formData.description || undefined,
      finalCategory,
      finalPriority,
      finalQuadrant
    );

    // 合并用户选择的标签和 AI 推荐的标签（去重）
    const allTagIds = [...new Set([...formData.tagIds, ...aiSuggestedTagIds])];

    // 如果有标签,添加标签
    if (allTagIds.length > 0) {
      try {
        await tagApi.addTagsToTask(taskId, allTagIds);
      } catch (error) {
        console.error('添加标签失败:', error);
        message.warning('任务创建成功,但添加标签失败');
      }
    }

    // 刷新任务列表
    await taskStore.loadTasks();

    if (isAiEnabled.value) {
      message.success('任务已通过 AI 智能创建');
    } else {
      message.success('任务创建成功');
    }
    handleCancelEdit();
  } catch (error: any) {
    console.error('创建任务失败:', error);
    message.error(error?.message || '创建任务失败，请检查输入内容');
  } finally {
    isCreating.value = false;
  }
}

async function handleStart(taskId: number) {
  await taskStore.startTask(taskId);
  message.success('任务已开始');
}

async function handlePause(taskId: number) {
  await taskStore.pauseTask(taskId);
  message.success('任务已暂停');
}

async function handleComplete(taskId: number) {
  await taskStore.completeTask(taskId);
  message.success('任务已完成');
}

async function handleEdit(task: Task) {
  isEditing.value = true;
  editingTaskId.value = task.id!;
  formData.title = task.title;
  formData.description = task.description || '';
  formData.category = task.category;
  formData.priority = task.priority;
  formData.quadrant = task.quadrant || 'urgent_not_important';
  formData.tagIds = task.tags?.map(t => t.id!).filter(id => id !== undefined) || [];
  showCreateModal.value = true;
}

async function handleUpdate() {
  try {
    await formRef.value?.validate();

    // 前端额外验证
    if (formData.title.trim().length === 0) {
      message.error('任务标题不能为空');
      return;
    }
    if (formData.title.length > 200) {
      message.error('任务标题不能超过200个字符');
      return;
    }
    if (formData.description && formData.description.length > 10000) {
      message.error('任务描述不能超过10000个字符');
      return;
    }

    // 更新任务基本信息
    await taskStore.updateTask(editingTaskId.value!, {
      title: formData.title,
      description: formData.description || undefined,
      category: formData.category,
      priority: formData.priority,
      quadrant: formData.quadrant,
    });

    // 处理标签更新:先移除所有标签,再添加选中的标签
    try {
      await tagApi.removeAllTagsFromTask(editingTaskId.value!);
      if (formData.tagIds.length > 0) {
        await tagApi.addTagsToTask(editingTaskId.value!, formData.tagIds);
      }
    } catch (error) {
      console.error('更新标签失败:', error);
      message.warning('任务更新成功,但标签更新失败');
    }

    // 刷新任务列表
    await taskStore.loadTasks();

    message.success('任务更新成功');
    handleCancelEdit();
  } catch (error: any) {
    console.error('更新任务失败:', error);
    message.error(error?.message || '更新任务失败，请检查输入内容');
  }
}

function handleCancelEdit() {
  showCreateModal.value = false;
  isEditing.value = false;
  editingTaskId.value = null;
  formData.title = '';
  formData.description = '';
  formData.category = 'other';
  formData.priority = 2;
  formData.quadrant = 'urgent_not_important';
  formData.tagIds = [];

  // 清理防抖定时器
  if (aiEnhanceDebounceTimer) {
    clearTimeout(aiEnhanceDebounceTimer);
    aiEnhanceDebounceTimer = null;
  }
}

async function handleDefer(taskId: number) {
  await taskStore.deferTask(taskId);
  message.success('任务已延后');
}

async function handleDelete(taskId: number) {
  await taskStore.deleteTask(taskId);
  message.success('任务已删除');
}

async function handleRefresh() {
  await taskStore.loadTasks();
  await taskStore.loadCompletedTasks(7);
  message.success('刷新成功');
}

function handleImportSuccess() {
  message.success('任务导入成功');
  handleRefresh();
}

function handleTaskClick(task: Task) {
  selectedTask.value = task;
  showDetailModal.value = true;
}

async function handleTaskUpdate(task: Task, updates: Partial<Task>) {
  try {
    await taskStore.updateTask(task.id!, updates);
    message.success('任务更新成功');

    // 更新selectedTask以反映最新状态
    if (selectedTask.value?.id === task.id) {
      selectedTask.value = { ...selectedTask.value, ...updates };
    }
  } catch (error: any) {
    console.error('更新任务失败:', error);
    message.error(error?.message || '更新任务失败');
  }
}

// 加载标签
async function loadTags() {
  try {
    availableTags.value = await tagApi.getAllTags();
  } catch (error: any) {
    console.error('加载标签失败:', error);
    message.error('加载标签失败');
  }
}

// 标签管理回调
async function handleCreateTag(tag: Omit<Tag, 'id'>) {
  try {
    const tagId = await tagApi.createTag(tag.name, tag.color);
    await loadTags();
    message.success('标签创建成功');
  } catch (error: any) {
    console.error('创建标签失败:', error);
    message.error(error?.message || '创建标签失败');
  }
}

async function handleUpdateTag(id: number, updates: Partial<Tag>) {
  try {
    await tagApi.updateTag(id, updates.name!, updates.color!);
    await loadTags();
    message.success('标签更新成功');
  } catch (error: any) {
    console.error('更新标签失败:', error);
    message.error(error?.message || '更新标签失败');
  }
}

async function handleDeleteTag(id: number) {
  try {
    await tagApi.deleteTag(id);
    await loadTags();
    message.success('标签删除成功');
  } catch (error: any) {
    console.error('删除标签失败:', error);
    message.error(error?.message || '删除标签失败');
  }
}

// AI 智能分类
async function handleAiClassify() {
  if (!formData.title) {
    message.warning('请先输入任务标题');
    return;
  }

  aiClassifying.value = true;

  try {
    // 获取现有标签名称列表
    const existingTagNames = availableTags.value.map(t => t.name);

    // 调用AI分类，传入现有标签
    const result = await aiApi.classifyTask(
      formData.title,
      formData.description || undefined,
      existingTagNames
    );

    // 应用分类结果
    if (result.category) {
      formData.category = result.category as Task['category'];
    }
    if (result.priority) {
      formData.priority = result.priority as Task['priority'];
    }
    if (result.quadrant) {
      formData.quadrant = result.quadrant as TaskQuadrant;
    }

    // 处理建议的标签：匹配现有标签
    if (result.suggestedTags && result.suggestedTags.length > 0) {
      const matchedTagIds: number[] = [];
      const newTags: string[] = [];

      // 遍历AI建议的标签，找到匹配的现有标签
      for (const suggestedTag of result.suggestedTags) {
        const matchedTag = availableTags.value.find(
          t => t.name.toLowerCase() === suggestedTag.toLowerCase()
        );

        if (matchedTag && matchedTag.id) {
          matchedTagIds.push(matchedTag.id);
        } else {
          newTags.push(suggestedTag);
        }
      }

      // 应用匹配到的标签
      if (matchedTagIds.length > 0) {
        formData.tagIds = matchedTagIds;
      }

      // 显示置信度
      const confidencePercent = (result.confidence * 100).toFixed(0);
      message.success(`AI 分类成功（置信度: ${confidencePercent}%）`);

      // 如果有新标签建议，提示用户
      if (newTags.length > 0) {
        message.info(`建议新标签: ${newTags.join(', ')}。可在标签管理中创建。`);
      }
    } else {
      // 没有标签建议时只显示置信度
      const confidencePercent = (result.confidence * 100).toFixed(0);
      message.success(`AI 分类成功（置信度: ${confidencePercent}%）`);
    }
  } catch (error: any) {
    console.error('AI 分类失败:', error);
    message.error(error?.message || 'AI 分类失败，请检查 AI 配置');
  } finally {
    aiClassifying.value = false;
  }
}

// AI 增强描述
async function handleAiEnhance() {
  if (!formData.title) {
    message.warning('请先输入任务标题');
    return;
  }

  // 生成缓存键
  const cacheKey = `${formData.title}_${formData.description || ''}`;

  // 检查缓存
  if (enhanceCache.has(cacheKey)) {
    formData.description = enhanceCache.get(cacheKey)!;
    message.success('已使用缓存的增强描述');
    return;
  }

  // 防抖处理：如果有正在进行的请求，先清除
  if (aiEnhanceDebounceTimer) {
    clearTimeout(aiEnhanceDebounceTimer);
  }

  // 延迟300ms执行，避免重复点击
  aiEnhanceDebounceTimer = setTimeout(async () => {
    aiEnhancing.value = true;

    try {
      const enhancedDesc = await aiApi.enhanceTaskDescription(
        formData.title,
        formData.description || undefined
      );

      formData.description = enhancedDesc;

      // 缓存结果
      enhanceCache.set(cacheKey, enhancedDesc);

      // 清理缓存（如果超过限制）
      cleanupCache();

      message.success('任务描述已优化');
    } catch (error: any) {
      console.error('AI 增强描述失败:', error);
      message.error(error?.message || 'AI 增强描述失败，请检查 AI 配置');
    } finally {
      aiEnhancing.value = false;
      aiEnhanceDebounceTimer = null;
    }
  }, 300);
}

// AI 生成子任务
async function handleAiGenerateSubtasks() {
  if (!formData.title) {
    message.warning('请先输入任务标题');
    return;
  }

  aiGeneratingSubtasks.value = true;

  try {
    const subtasks = await aiApi.generateSubtasks(
      formData.title,
      formData.description || undefined
    );

    if (subtasks.length > 0) {
      // 直接将子任务添加到描述中
      const subtasksSection = '\n\n## 子任务\n' + subtasks.map((task, index) => `${index + 1}. ${task}`).join('\n');
      formData.description = (formData.description || '') + subtasksSection;

      message.success(`已生成 ${subtasks.length} 个子任务并添加到描述中`);
    } else {
      message.warning('AI 未生成子任务');
    }
  } catch (error: any) {
    console.error('AI 生成子任务失败:', error);
    message.error(error?.message || 'AI 生成子任务失败，请检查 AI 配置');
  } finally {
    aiGeneratingSubtasks.value = false;
  }
}
</script>

<style scoped>
.task-board {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 24px;
}

/* Header */
.board-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.board-title {
  font-size: 20px;
  font-weight: 600;
  color: #f1f5f9; /* slate-100 */
  margin: 0;
  letter-spacing: -0.025em;
}

/* 筛选器 */
.filters-section {
  margin-bottom: 20px;
  padding: 16px;
  background: rgba(15, 23, 42, 0.4);
  border-radius: 12px;
  border: 1px solid rgba(51, 65, 85, 0.5);
  backdrop-filter: blur(12px);
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #94a3b8;
  font-weight: 600;
  white-space: nowrap;
}

.filter-stats {
  font-size: 12px;
  color: #6366f1;
  font-weight: 600;
  padding: 4px 10px;
  background: rgba(99, 102, 241, 0.15);
  border-radius: 6px;
  border: 1px solid rgba(99, 102, 241, 0.3);
}

/* Board Columns */
.board-columns {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 24px;
  overflow: hidden;
  min-height: 0;
}

.board-column {
  display: flex;
  flex-direction: column;
  background: rgba(15, 23, 42, 0.4);
  border-radius: 20px;
  border: 1px solid rgba(51, 65, 85, 0.5);
  padding: 8px;
  backdrop-filter: blur(12px);
  min-height: 0;
  transition: all 0.3s ease;
}

.board-column:hover {
  border-color: rgba(71, 85, 105, 0.6);
}

/* Column Header */
.column-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  padding: 12px 8px;
}

.column-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.status-todo {
  background-color: #64748b;
}

.status-dot.status-doing {
  background-color: #6366f1;
  box-shadow: 0 0 10px rgba(99, 102, 241, 0.5);
}

.status-dot.status-done {
  background-color: #10b981;
}

.title-text {
  font-size: 14px;
  font-weight: 700;
  color: #cbd5e1;
  letter-spacing: -0.01em;
}

.task-count {
  font-size: 11px;
  color: #64748b;
  background: rgba(30, 41, 59, 0.8);
  padding: 3px 10px;
  border-radius: 9999px;
  font-weight: 600;
  border: 1px solid rgba(51, 65, 85, 0.5);
  font-family: 'Consolas', 'Monaco', monospace;
}

/* Column Content */
.column-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding-right: 4px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* Custom Scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(51, 65, 85, 0.5);
  border-radius: 3px;
  transition: background 0.2s;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(71, 85, 105, 0.7);
}

/* Task Card Styling */
.task-card-item {
  transition: all 0.2s;
}

.task-card-item:deep(.n-card) {
  background: rgba(30, 41, 59, 0.4);
  border: 1px solid rgba(51, 65, 85, 0.5);
  transition: all 0.2s;
}

.task-card-item:deep(.n-card):hover {
  background: rgba(30, 41, 59, 1);
  border-color: rgba(71, 85, 105, 1);
  transform: translateY(-2px);
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.2), 0 2px 4px -1px rgba(0, 0, 0, 0.1);
}

/* Add Card Button */
.add-card-button {
  width: 100%;
  padding: 12px;
  border: 1px dashed rgba(51, 65, 85, 0.5);
  border-radius: 14px;
  background: transparent;
  color: #64748b;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  margin-top: 4px;
}

.add-card-button:hover {
  background: rgba(30, 41, 59, 0.4);
  color: #a78bfa;
  border-color: rgba(99, 102, 241, 0.3);
  border-style: solid;
}

.add-card-button:hover :deep(.n-icon) {
  transform: scale(1.15) rotate(90deg);
}

.add-card-button :deep(.n-icon) {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.add-card-button:active {
  transform: scale(0.98);
}

/* Empty Placeholder */
.empty-placeholder {
  margin-top: 40px;
}

.empty-placeholder:deep(.n-empty__description) {
  color: #94a3b8; /* slate-400 */
  font-size: 13px;
}

/* Deferred Section */
.deferred-section {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px dashed rgba(51, 65, 85, 0.5);
}

.deferred-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  padding: 0 8px;
}

.deferred-title {
  font-size: 12px;
  font-weight: 600;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.deferred-count {
  font-size: 11px;
  color: #64748b;
  background: rgba(30, 41, 59, 0.6);
  padding: 2px 8px;
  border-radius: 9999px;
  font-weight: 600;
  border: 1px solid rgba(51, 65, 85, 0.5);
}

.deferred-task:deep(.n-card) {
  opacity: 0.7;
  background: rgba(30, 41, 59, 0.3);
}

.deferred-task:deep(.n-card):hover {
  opacity: 1;
  background: rgba(30, 41, 59, 0.7);
}

/* Responsive */
@media (max-width: 900px) {
  .board-columns {
    grid-template-columns: repeat(2, 1fr);
  }

  .board-column:last-child {
    grid-column: 1 / -1;
  }
}

@media (max-width: 768px) {
  .board-columns {
    grid-template-columns: 1fr;
  }

  .board-column:last-child {
    grid-column: auto;
  }

  .task-board {
    padding: 16px;
  }
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

/* AI 按钮样式优化 */
:deep(.n-button.n-button--ghost-type.n-button--primary-type) {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

:deep(.n-button.n-button--ghost-type.n-button--primary-type:hover:not(:disabled)) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
}

:deep(.n-button.n-button--ghost-type.n-button--primary-type:active:not(:disabled)) {
  transform: translateY(0);
}

:deep(.n-button.n-button--loading) {
  opacity: 0.8;
  cursor: wait;
}

/* AI 增强按钮特殊效果 */
:deep(.n-button.n-button--ghost-type.n-button--primary-type .n-icon) {
  animation: sparkle 2s ease-in-out infinite;
}

@keyframes sparkle {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.7;
    transform: scale(1.1);
  }
}

/* AI 状态提示 */
.ai-status-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #a78bfa;
  padding: 6px 12px;
  background: rgba(139, 92, 246, 0.1);
  border-radius: 6px;
  border: 1px solid rgba(139, 92, 246, 0.2);
}

.ai-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #a78bfa;
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
