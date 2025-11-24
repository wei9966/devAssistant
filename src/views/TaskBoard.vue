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
        <n-button @click="handleRefresh">
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          刷新
        </n-button>
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
            v-for="task in taskStore.todoTasks"
            :key="task.id"
            :task="task"
            @start="handleStart"
            @complete="handleComplete"
            @edit="handleEdit"
            @delete="handleDelete"
            @defer="handleDefer"
            class="task-card-item"
          />

          <div v-if="taskStore.deferredTasks.length > 0" class="deferred-section">
            <div class="deferred-header">
              <span class="deferred-title">延后任务</span>
              <span class="deferred-count">{{ taskStore.deferredTasks.length }}</span>
            </div>
            <TaskCard
              v-for="task in taskStore.deferredTasks"
              :key="task.id"
              :task="task"
              @start="handleStart"
              @complete="handleComplete"
              @edit="handleEdit"
              @delete="handleDelete"
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
            v-for="task in taskStore.activeTasks"
            :key="task.id"
            :task="task"
            @pause="handlePause"
            @complete="handleComplete"
            @edit="handleEdit"
            class="task-card-item"
          />

          <n-empty
            v-if="taskStore.activeTasks.length === 0"
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
            v-for="task in taskStore.completedTasks"
            :key="task.id"
            :task="task"
            readonly
            class="task-card-item"
          />

          <n-empty
            v-if="taskStore.completedTasks.length === 0"
            description="暂无已完成任务"
            class="empty-placeholder"
          />
        </div>
      </div>
    </div>

    <!-- 创建/编辑任务对话框 -->
    <n-modal v-model:show="showCreateModal" preset="card" :title="isEditing ? '编辑任务' : '新建任务'" style="width: 600px">
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
          <n-input
            v-model:value="formData.description"
            type="textarea"
            placeholder="请输入任务描述（最多10000字符）"
            :rows="3"
            :maxlength="10000"
            show-count
          />
        </n-form-item>
        <n-form-item label="分类" path="category">
          <n-select
            v-model:value="formData.category"
            :options="categoryOptions"
          />
        </n-form-item>
        <n-form-item label="优先级" path="priority">
          <n-select
            v-model:value="formData.priority"
            :options="priorityOptions"
          />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="handleCancelEdit">取消</n-button>
          <n-button class="primary-button" @click="isEditing ? handleUpdate() : handleCreate()">
            {{ isEditing ? '保存' : '创建' }}
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { NCard, NSpace, NButton, NIcon, NEmpty, NCollapse, NCollapseItem, NModal, NForm, NFormItem, NInput, NSelect, useMessage } from 'naive-ui';
import { AddOutline, RefreshOutline } from '@vicons/ionicons5';
import { useTaskStore } from '@/stores/taskStore';
import TaskCard from '@/components/TaskCard.vue';
import { CATEGORY_LABELS, PRIORITY_LABELS } from '@/types/task';
import type { Task } from '@/types/task';

const taskStore = useTaskStore();
const message = useMessage();
const showCreateModal = ref(false);
const formRef = ref();
const isEditing = ref(false);
const editingTaskId = ref<number | null>(null);

const formData = reactive({
  title: '',
  description: '',
  category: 'other' as Task['category'],
  priority: 2 as Task['priority'],
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

onMounted(async () => {
  await taskStore.loadTasks();
  await taskStore.loadCompletedTasks(7);

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

    await taskStore.createTask(
      formData.title,
      formData.description || undefined,
      formData.category,
      formData.priority
    );
    message.success('任务创建成功');
    handleCancelEdit();
  } catch (error: any) {
    console.error('创建任务失败:', error);
    message.error(error?.message || '创建任务失败，请检查输入内容');
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

    await taskStore.updateTask(editingTaskId.value!, {
      title: formData.title,
      description: formData.description || undefined,
      category: formData.category,
      priority: formData.priority,
    });
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
  margin-bottom: 24px;
}

.board-title {
  font-size: 20px;
  font-weight: 600;
  color: #f1f5f9; /* slate-100 */
  margin: 0;
  letter-spacing: -0.025em;
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
</style>
