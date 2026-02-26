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
        <n-button @click="toggleTaskFloat" :type="isTaskFloatVisible ? 'primary' : 'default'">
          <template #icon>
            <n-icon><LayersOutline /></n-icon>
          </template>
          {{ isTaskFloatVisible ? '关闭悬浮窗' : '悬浮窗' }}
        </n-button>
        <n-button @click="toggleCalendarFloat" :type="isCalendarFloatVisible ? 'primary' : 'default'">
          <template #icon>
            <n-icon><CalendarOutline /></n-icon>
          </template>
          {{ isCalendarFloatVisible ? '关闭日历' : '日历' }}
        </n-button>
        <n-badge :value="pendingPredictionCount" :max="99" :offset="[-5, 5]">
          <n-button @click="showPredictionDrawer = true">
            <template #icon>
              <n-icon><BulbOutline /></n-icon>
            </template>
            AI 预测
          </n-button>
        </n-badge>
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
            @cancel="handleCancel"
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
              @cancel="handleCancel"
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
            @adjust-progress="handleAdjustProgress"
            @add-milestone="handleAddMilestone"
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
            @reactivate="handleReactivate"
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

          <!-- 计划开始时间（精确到秒，用于提醒） -->
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

          <!-- 保存并完成时的里程碑输入区域 -->
          <div v-if="showSaveAndCompleteMilestone && !isEditing" class="milestone-input-section">
            <div class="milestone-input-header">完成里程碑（可选）</div>
            <n-input
              v-model:value="saveCompleteTitle"
              placeholder="里程碑标题，如：完成了XX功能开发"
              style="margin-bottom: 8px;"
            />
            <n-input
              v-model:value="saveCompleteDesc"
              type="textarea"
              placeholder="里程碑描述（可选）"
              :rows="2"
            />
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
              v-if="!isEditing"
              type="success"
              @click="handleCreateAndComplete()"
              :loading="isCreating"
              :disabled="isCreating"
            >
              保存并完成
            </n-button>
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
      @toggle-favorite="handleToggleTagFavorite"
    />

    <!-- AI 预测任务列表抽屉 -->
    <n-drawer v-model:show="showPredictionDrawer" width="450" placement="right">
      <n-drawer-content title="AI 预测任务" closable>
        <PredictedTaskList
          ref="predictionListRef"
          @task-accepted="handlePredictionAccepted"
          @refresh="updatePredictionCount"
        />
      </n-drawer-content>
    </n-drawer>

    <!-- 进度调整弹窗 -->
    <n-modal v-model:show="showProgressModal" preset="card" title="调整任务进度" style="width: 450px;">
      <div class="progress-modal-content">
        <div class="progress-task-title">{{ progressTask?.title }}</div>
        <div class="progress-slider-section">
          <n-slider
            v-model:value="progressValue"
            :step="5"
            :marks="{ 0: '0%', 25: '25%', 50: '50%', 75: '75%', 100: '100%' }"
          />
          <div class="progress-value-display">{{ progressValue }}%</div>
        </div>
      </div>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showProgressModal = false">取消</n-button>
          <n-button type="primary" @click="handleSaveProgress">保存</n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 里程碑创建弹窗 -->
    <n-modal v-model:show="showMilestoneModal" preset="card" title="添加里程碑" style="width: 450px;">
      <div class="milestone-modal-content">
        <div class="milestone-task-title">{{ milestoneTask?.title }}</div>
        <n-form-item label="里程碑标题">
          <n-input v-model:value="milestoneTitle" placeholder="请输入里程碑标题" />
        </n-form-item>
        <n-form-item label="描述（可选）">
          <n-input
            v-model:value="milestoneDesc"
            type="textarea"
            placeholder="请输入描述"
            :autosize="{ minRows: 2, maxRows: 4 }"
          />
        </n-form-item>
        <n-form-item label="设置进度">
          <div class="milestone-progress-setting">
            <n-slider
              v-model:value="milestoneProgress"
              :min="milestoneTask?.progress || 0"
              :max="100"
              :step="5"
              :marks="milestoneProgressMarks"
            />
            <div class="milestone-progress-value">{{ milestoneProgress }}%</div>
          </div>
        </n-form-item>
        <div class="milestone-progress-hint">
          进度不能低于当前值 {{ milestoneTask?.progress || 0 }}%
        </div>
      </div>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showMilestoneModal = false">取消</n-button>
          <n-button type="primary" @click="handleSaveMilestone">创建</n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 完成任务里程碑对话框 -->
    <n-modal v-model:show="showCompleteModal" preset="card" title="完成任务" style="width: 450px;">
      <div class="complete-modal-content">
        <div class="complete-task-title">{{ completeTask?.title }}</div>
        <n-form-item label="完成里程碑（可选）">
          <n-input v-model:value="completeTitle" placeholder="记录完成时的里程碑说明" />
        </n-form-item>
        <n-form-item label="描述（可选）">
          <n-input
            v-model:value="completeDesc"
            type="textarea"
            placeholder="补充描述信息"
            :autosize="{ minRows: 2, maxRows: 4 }"
          />
        </n-form-item>
      </div>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showCompleteModal = false">取消</n-button>
          <n-button type="success" @click="handleConfirmComplete">确认完成</n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 重新激活任务对话框 -->
    <n-modal v-model:show="showReactivateModal" preset="card" title="重新激活任务" style="width: 450px;">
      <div class="reactivate-modal-content">
        <div class="reactivate-task-title">{{ reactivateTask?.title }}</div>
        <n-form-item label="激活说明" :required="true">
          <n-input v-model:value="reactivateTitle" placeholder="请说明重新激活的原因" />
        </n-form-item>
        <n-form-item label="描述（可选）">
          <n-input
            v-model:value="reactivateDesc"
            type="textarea"
            placeholder="补充描述信息"
            :autosize="{ minRows: 2, maxRows: 4 }"
          />
        </n-form-item>
        <n-form-item label="设置当前进度">
          <div class="reactivate-progress-setting">
            <n-slider
              v-model:value="reactivateProgress"
              :min="0"
              :max="95"
              :step="5"
              :marks="{ 0: '0%', 25: '25%', 50: '50%', 75: '75%', 95: '95%' }"
            />
            <div class="reactivate-progress-value">{{ reactivateProgress }}%</div>
          </div>
        </n-form-item>
        <div class="reactivate-hint">
          任务将从已完成状态重新激活为进行中，并设置为选定的进度
        </div>
      </div>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showReactivateModal = false">取消</n-button>
          <n-button type="primary" @click="handleConfirmReactivate">确认激活</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed, onUnmounted } from 'vue';
import { Window } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { NCard, NSpace, NButton, NIcon, NEmpty, NCollapse, NCollapseItem, NModal, NForm, NFormItem, NInput, NSelect, NDatePicker, NDrawer, NDrawerContent, NBadge, NSlider, useMessage } from 'naive-ui';
import dayjs from 'dayjs';
import { AddOutline, RefreshOutline, CloudUploadOutline, GridOutline, PricetagsOutline, CloseCircleOutline, LayersOutline, CalendarOutline, BulbOutline } from '@vicons/ionicons5';
import { useTaskStore } from '@/stores/taskStore';
import { tagApi } from '@/api/tagApi';
import { aiApi } from '@/api/aiApi';
import { taskApi } from '@/api/taskApi';
import TaskCard from '@/components/TaskCard.vue';
import TaskImport from '@/components/TaskImport.vue';
import TaskDetailModal from '@/components/TaskDetailModal.vue';
import QuadrantSelector from '@/components/QuadrantSelector.vue';
import TagSelector from '@/components/TagSelector.vue';
import TagManager from '@/components/TagManager.vue';
import PrioritySelector from '@/components/PrioritySelector.vue';
import CategorySelector from '@/components/CategorySelector.vue';
import PredictedTaskList from '@/components/PredictedTaskList.vue';
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

// 悬浮窗状态
const isTaskFloatVisible = ref(false);
const isCalendarFloatVisible = ref(false);

// 预测任务列表状态
const showPredictionDrawer = ref(false);
const predictionListRef = ref<InstanceType<typeof PredictedTaskList> | null>(null);
const pendingPredictionCount = ref(0);

// 进度调整弹窗状态
const showProgressModal = ref(false);
const progressTask = ref<Task | null>(null);
const progressValue = ref(0);

// 里程碑弹窗状态
const showMilestoneModal = ref(false);
const milestoneTask = ref<Task | null>(null);
const milestoneTitle = ref('');
const milestoneDesc = ref('');
const milestoneProgress = ref(0);

// 保存并完成相关状态
const showSaveAndCompleteMilestone = ref(false);
const saveCompleteTitle = ref('');
const saveCompleteDesc = ref('');

// 完成任务弹窗状态
const showCompleteModal = ref(false);
const completeTask = ref<Task | null>(null);
const completeTitle = ref('');
const completeDesc = ref('');

// 重新激活弹窗状态
const showReactivateModal = ref(false);
const reactivateTask = ref<Task | null>(null);
const reactivateTitle = ref('');
const reactivateDesc = ref('');
const reactivateProgress = ref(0);

// 里程碑进度标记
const milestoneProgressMarks = computed(() => {
  const currentProgress = milestoneTask.value?.progress || 0;
  const marks: Record<number, string> = {};
  marks[currentProgress] = `${currentProgress}%`;
  if (currentProgress < 50) marks[50] = '50%';
  if (currentProgress < 75) marks[75] = '75%';
  marks[100] = '100%';
  return marks;
});

// 切换悬浮窗显示
async function toggleTaskFloat() {
  try {
    const floatWindow = await Window.getByLabel('task-float');
    if (floatWindow) {
      if (isTaskFloatVisible.value) {
        await floatWindow.hide();
        isTaskFloatVisible.value = false;
      } else {
        await floatWindow.show();
        isTaskFloatVisible.value = true;
      }
    } else {
      message.error('悬浮窗未初始化');
    }
  } catch (error) {
    console.error('切换悬浮窗失败:', error);
    message.error('切换悬浮窗失败');
  }
}

// 切换日历悬浮窗显示
async function toggleCalendarFloat() {
  try {
    const calendarWindow = await Window.getByLabel('task-calendar');
    if (calendarWindow) {
      if (isCalendarFloatVisible.value) {
        await calendarWindow.hide();
        isCalendarFloatVisible.value = false;
      } else {
        await calendarWindow.show();
        isCalendarFloatVisible.value = true;
      }
    } else {
      message.error('日历悬浮窗未初始化');
    }
  } catch (error) {
    console.error('切换日历悬浮窗失败:', error);
    message.error('切换日历悬浮窗失败');
  }
}

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
  dueDate: null as number | null,        // 截止日期
  registeredAt: Date.now() as number,    // 登记日期，默认为今天
  scheduledStartTime: null as number | null, // 计划开始时间（精确到秒，用于提醒）
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

  // 监听悬浮窗关闭事件
  unlistenFloatHidden = await listen('task-float-hidden', () => {
    isTaskFloatVisible.value = false;
  });

  // 监听日历悬浮窗关闭事件
  unlistenCalendarHidden = await listen('task-calendar-hidden', () => {
    isCalendarFloatVisible.value = false;
  });

  // 加载预测任务数量 (延迟加载，等组件就绪)
  setTimeout(() => {
    updatePredictionCount();
  }, 500);
});

let unlistenFloatHidden: (() => void) | null = null;
let unlistenCalendarHidden: (() => void) | null = null;

onUnmounted(() => {
  if (unlistenFloatHidden) {
    unlistenFloatHidden();
  }
  if (unlistenCalendarHidden) {
    unlistenCalendarHidden();
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

        // 处理 AI 推荐的标签（模糊匹配）
        if (result.suggestedTags && result.suggestedTags.length > 0) {
          for (const suggestedTag of result.suggestedTags) {
            const matchedTag = availableTags.value.find(
              t => t.name.toLowerCase() === suggestedTag.toLowerCase() ||
                   t.name.toLowerCase().includes(suggestedTag.toLowerCase()) ||
                   suggestedTag.toLowerCase().includes(t.name.toLowerCase())
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

    // 格式化日期
    const dueDateStr = formData.dueDate ? dayjs(formData.dueDate).format('YYYY-MM-DD') : undefined;
    const registeredAtStr = formData.registeredAt ? dayjs(formData.registeredAt).format('YYYY-MM-DD') : undefined;
    const scheduledStartTimeStr = formData.scheduledStartTime ? dayjs(formData.scheduledStartTime).format('YYYY-MM-DD HH:mm:ss') : undefined;

    // 创建任务
    const taskId = await taskStore.createTask(
      formData.title,
      formData.description || undefined,
      finalCategory,
      finalPriority,
      finalQuadrant,
      dueDateStr,
      registeredAtStr,
      scheduledStartTimeStr
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

async function handleCreateAndComplete() {
  try {
    await formRef.value?.validate();

    // 前端验证
    if (formData.title.trim().length === 0) {
      message.error('任务标题不能为空');
      return;
    }
    if (formData.title.length > 200) {
      message.error('任务标题不能超过200个字符');
      return;
    }

    // 防止重复提交
    if (isCreating.value) return;
    isCreating.value = true;

    // 展开里程碑输入区域（如果还没展开）
    if (!showSaveAndCompleteMilestone.value) {
      showSaveAndCompleteMilestone.value = true;
      isCreating.value = false;
      return;
    }

    let finalCategory = formData.category;
    let finalPriority = formData.priority;
    let finalQuadrant = formData.quadrant;
    let aiSuggestedTagIds: number[] = [];

    // AI 分类
    if (isAiEnabled.value) {
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
            // 模糊匹配：精确匹配或包含匹配
            const matchedTag = availableTags.value.find(
              t => t.name.toLowerCase() === suggestedTag.toLowerCase() ||
                   t.name.toLowerCase().includes(suggestedTag.toLowerCase()) ||
                   suggestedTag.toLowerCase().includes(t.name.toLowerCase())
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

    // 格式化日期
    const dueDateStr = formData.dueDate ? dayjs(formData.dueDate).format('YYYY-MM-DD') : undefined;
    const registeredAtStr = formData.registeredAt ? dayjs(formData.registeredAt).format('YYYY-MM-DD') : undefined;
    const scheduledStartTimeStr = formData.scheduledStartTime ? dayjs(formData.scheduledStartTime).format('YYYY-MM-DD HH:mm:ss') : undefined;

    // 创建任务
    const taskId = await taskStore.createTask(
      formData.title,
      formData.description || undefined,
      finalCategory,
      finalPriority,
      finalQuadrant,
      dueDateStr,
      registeredAtStr,
      scheduledStartTimeStr
    );

    // 添加标签
    const allTagIds = [...new Set([...formData.tagIds, ...aiSuggestedTagIds])];
    if (allTagIds.length > 0) {
      try {
        await tagApi.addTagsToTask(taskId, allTagIds);
      } catch (error) {
        console.error('添加标签失败:', error);
      }
    }

    // 如果填写了里程碑，创建里程碑
    if (saveCompleteTitle.value.trim()) {
      try {
        await taskApi.createTaskMilestone(
          taskId,
          saveCompleteTitle.value.trim(),
          saveCompleteDesc.value.trim() || undefined,
          100
        );
      } catch (error) {
        console.error('创建里程碑失败:', error);
      }
    }

    // 立即完成任务
    await taskStore.completeTask(taskId);

    // 刷新任务列表
    await taskStore.loadTasks();

    message.success('任务已创建并完成');
    showSaveAndCompleteMilestone.value = false;
    saveCompleteTitle.value = '';
    saveCompleteDesc.value = '';
    handleCancelEdit();
  } catch (error: any) {
    console.error('创建并完成任务失败:', error);
    message.error(error?.message || '操作失败，请检查输入内容');
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

function handleComplete(taskId: number) {
  const task = [...taskStore.tasks, ...taskStore.completedTasks].find(t => t.id === taskId);
  completeTask.value = task || null;
  completeTitle.value = '';
  completeDesc.value = '';
  showCompleteModal.value = true;
}

// 确认完成任务（可选里程碑）
async function handleConfirmComplete() {
  if (!completeTask.value?.id) return;
  try {
    const taskId = completeTask.value.id;

    // 如果填写了里程碑标题，先创建里程碑
    if (completeTitle.value.trim()) {
      await taskApi.createTaskMilestone(
        taskId,
        completeTitle.value.trim(),
        completeDesc.value.trim() || undefined,
        100
      );
    }

    // 完成任务
    await taskStore.completeTask(taskId);
    message.success('任务已完成');
    showCompleteModal.value = false;
  } catch (error: any) {
    console.error('完成任务失败:', error);
    message.error(error?.message || '完成任务失败');
  }
}

// 打开重新激活弹窗
function handleReactivate(task: Task) {
  reactivateTask.value = task;
  reactivateTitle.value = '';
  reactivateDesc.value = '';
  reactivateProgress.value = 0;
  showReactivateModal.value = true;
}

// 确认重新激活
async function handleConfirmReactivate() {
  if (!reactivateTask.value?.id) return;
  if (!reactivateTitle.value.trim()) {
    message.warning('请填写激活说明');
    return;
  }
  try {
    const taskId = reactivateTask.value.id;

    // 创建里程碑记录重新激活
    await taskApi.createTaskMilestone(
      taskId,
      reactivateTitle.value.trim(),
      reactivateDesc.value.trim() || undefined,
      reactivateProgress.value
    );

    // 重新激活任务
    await taskStore.reactivateTask(taskId, reactivateProgress.value);
    message.success('任务已重新激活');
    showReactivateModal.value = false;
  } catch (error: any) {
    console.error('重新激活任务失败:', error);
    message.error(error?.message || '重新激活任务失败');
  }
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
  formData.dueDate = task.dueDate ? new Date(task.dueDate).getTime() : null;
  formData.registeredAt = task.registeredAt ? new Date(task.registeredAt).getTime() : Date.now();
  formData.scheduledStartTime = task.scheduledStartTime ? new Date(task.scheduledStartTime).getTime() : null;
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

    // 格式化日期
    const dueDateStr = formData.dueDate ? dayjs(formData.dueDate).format('YYYY-MM-DD') : undefined;
    const registeredAtStr = formData.registeredAt ? dayjs(formData.registeredAt).format('YYYY-MM-DD') : undefined;
    const scheduledStartTimeStr = formData.scheduledStartTime ? dayjs(formData.scheduledStartTime).format('YYYY-MM-DD HH:mm:ss') : undefined;

    // 更新任务基本信息
    await taskStore.updateTask(editingTaskId.value!, {
      title: formData.title,
      description: formData.description || undefined,
      category: formData.category,
      priority: formData.priority,
      quadrant: formData.quadrant,
      dueDate: dueDateStr,
      registeredAt: registeredAtStr,
      scheduledStartTime: scheduledStartTimeStr,
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
  formData.dueDate = null;
  formData.registeredAt = Date.now();
  formData.scheduledStartTime = null;

  // 重置保存并完成状态
  showSaveAndCompleteMilestone.value = false;
  saveCompleteTitle.value = '';
  saveCompleteDesc.value = '';

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

async function handleCancel(taskId: number) {
  await taskStore.cancelTask(taskId);
  message.success('任务已取消');
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

// 预测任务被接受后刷新任务列表
function handlePredictionAccepted(_taskId: number) {
  handleRefresh();
}

// 更新预测任务数量
function updatePredictionCount() {
  if (predictionListRef.value) {
    pendingPredictionCount.value = predictionListRef.value.pendingCount;
  }
}

function handleTaskClick(task: Task) {
  selectedTask.value = task;
  showDetailModal.value = true;
}

// 打开进度调整弹窗
function handleAdjustProgress(task: Task) {
  progressTask.value = task;
  progressValue.value = task.progress || 0;
  showProgressModal.value = true;
}

// 保存进度
async function handleSaveProgress() {
  if (!progressTask.value?.id) return;
  try {
    await taskApi.updateTaskProgress(progressTask.value.id, progressValue.value);
    message.success('进度更新成功');
    showProgressModal.value = false;
    await taskStore.loadTasks();
  } catch (error: any) {
    console.error('更新进度失败:', error);
    message.error(error?.message || '更新进度失败');
  }
}

// 打开里程碑弹窗
function handleAddMilestone(task: Task) {
  milestoneTask.value = task;
  milestoneTitle.value = '';
  milestoneDesc.value = '';
  milestoneProgress.value = task.progress || 0;
  showMilestoneModal.value = true;
}

// 保存里程碑
async function handleSaveMilestone() {
  if (!milestoneTask.value?.id) return;
  if (!milestoneTitle.value.trim()) {
    message.warning('请输入里程碑标题');
    return;
  }
  try {
    const progressValue = milestoneProgress.value;

    await taskApi.createTaskMilestone(
      milestoneTask.value.id,
      milestoneTitle.value.trim(),
      milestoneDesc.value.trim() || undefined,
      progressValue
    );

    // 如果进度有变化，同时更新任务进度
    if (progressValue > (milestoneTask.value.progress || 0)) {
      await taskApi.updateTaskProgress(milestoneTask.value.id, progressValue);
      // 刷新任务列表
      await taskStore.loadTasks();
    }

    message.success('里程碑创建成功');
    showMilestoneModal.value = false;
  } catch (error: any) {
    console.error('创建里程碑失败:', error);
    message.error(error?.message || '创建里程碑失败');
  }
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

async function handleToggleTagFavorite(id: number, isFavorite: boolean) {
  try {
    await tagApi.toggleTagFavorite(id, isFavorite);
    await loadTags();
  } catch (error: any) {
    console.error('切换常用标签失败:', error);
    message.error(error?.message || '切换常用标签失败');
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
  color: var(--text-primary);
  margin: 0;
  letter-spacing: -0.025em;
}

/* 筛选器 */
.filters-section {
  margin-bottom: 20px;
  padding: 16px;
  background: var(--card-bg);
  border-radius: 12px;
  border: 1px solid var(--card-border);
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
  color: var(--text-primary);
  font-weight: 600;
  white-space: nowrap;
}

.filter-stats {
  font-size: 12px;
  color: var(--accent-primary);
  font-weight: 600;
  padding: 4px 10px;
  background: var(--accent-glow);
  border-radius: 6px;
  border: 1px solid var(--border-active);
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
  background: var(--card-bg);
  border-radius: 20px;
  border: 1px solid var(--card-border);
  padding: 8px;
  backdrop-filter: blur(12px);
  min-height: 0;
  transition: all 0.3s ease;
}

.board-column:hover {
  border-color: var(--border-hover);
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
  background-color: var(--text-secondary);
}

.status-dot.status-doing {
  background-color: var(--accent-primary);
  box-shadow: 0 0 10px var(--accent-glow);
}

.status-dot.status-done {
  background-color: var(--success-color);
}

.title-text {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.task-count {
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--bg-elevated);
  padding: 3px 10px;
  border-radius: 9999px;
  font-weight: 600;
  border: 1px solid var(--border-default);
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
  background: var(--scrollbar-track);
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 3px;
  transition: background 0.2s;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
}

/* Task Card Styling */
.task-card-item {
  transition: all 0.2s;
}

.task-card-item:deep(.n-card) {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  transition: all 0.2s;
}

.task-card-item:deep(.n-card):hover {
  background: var(--card-hover-bg);
  border-color: var(--card-hover-border);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

/* Add Card Button */
.add-card-button {
  width: 100%;
  padding: 12px;
  border: 1px dashed var(--border-default);
  border-radius: 14px;
  background: transparent;
  color: var(--text-primary);
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
  background: var(--bg-hover);
  color: var(--accent-secondary);
  border-color: var(--border-active);
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
  color: var(--text-primary);
  font-size: 13px;
}

/* Deferred Section */
.deferred-section {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px dashed var(--border-default);
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
  color: var(--text-primary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.deferred-count {
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--bg-hover);
  padding: 2px 8px;
  border-radius: 9999px;
  font-weight: 600;
  border: 1px solid var(--border-default);
}

.deferred-task:deep(.n-card) {
  opacity: 0.7;
  background: var(--bg-overlay);
}

.deferred-task:deep(.n-card):hover {
  opacity: 1;
  background: var(--bg-hover);
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
  background-color: var(--accent-primary) !important;
  border-color: var(--accent-primary) !important;
  color: var(--button-text) !important;
  box-shadow: 0 10px 15px -3px var(--shadow-accent) !important;
  transition: all 0.2s !important;
}

:deep(.primary-button:hover) {
  background-color: var(--accent-hover) !important;
  border-color: var(--accent-hover) !important;
}

:deep(.primary-button:active) {
  background-color: var(--accent-active) !important;
  border-color: var(--accent-active) !important;
}

/* AI 按钮样式优化 */
:deep(.n-button.n-button--ghost-type.n-button--primary-type) {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

:deep(.n-button.n-button--ghost-type.n-button--primary-type:hover:not(:disabled)) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px var(--shadow-accent);
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

/* 保存并完成 - 里程碑输入区域 */
.milestone-input-section {
  margin-top: 12px;
  padding: 12px 16px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 8px;
}

.milestone-input-header {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 8px;
}

/* AI 状态提示 */
.ai-status-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--accent-secondary);
  padding: 6px 12px;
  background: var(--accent-glow);
  border-radius: 6px;
  border: 1px solid var(--border-active);
}

.ai-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent-secondary);
  box-shadow: 0 0 8px var(--accent-glow);
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

/* 进度弹窗样式 */
.progress-modal-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.progress-task-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  padding: 12px;
  background: var(--card-bg);
  border-radius: 8px;
  border: 1px solid var(--card-border);
}

.progress-slider-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
  background: var(--bg-overlay);
  border-radius: 8px;
}

.progress-value-display {
  text-align: center;
  font-size: 32px;
  font-weight: 700;
  color: var(--accent-primary);
}

/* 里程碑弹窗样式 */
.milestone-modal-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.milestone-task-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  padding: 12px;
  background: var(--card-bg);
  border-radius: 8px;
  border: 1px solid var(--card-border);
}

.milestone-progress-hint {
  font-size: 12px;
  color: var(--warning-color);
  background: var(--warning-bg);
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--warning-border);
}

.milestone-progress-setting {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
}

.milestone-progress-value {
  text-align: center;
  font-size: 24px;
  font-weight: 600;
  color: var(--accent-primary);
}

/* 完成任务弹窗样式 */
.complete-modal-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.complete-task-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  padding: 12px;
  background: var(--card-bg);
  border-radius: 8px;
  border: 1px solid var(--card-border);
}

/* 重新激活弹窗样式 */
.reactivate-modal-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.reactivate-task-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  padding: 12px;
  background: var(--card-bg);
  border-radius: 8px;
  border: 1px solid var(--card-border);
}

.reactivate-progress-setting {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
}

.reactivate-progress-value {
  text-align: center;
  font-size: 24px;
  font-weight: 600;
  color: var(--accent-primary);
}

.reactivate-hint {
  font-size: 12px;
  color: var(--text-secondary);
  background: var(--bg-overlay);
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--border-default);
}
</style>
