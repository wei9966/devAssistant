<template>
  <n-modal
    v-model:show="showModal"
    preset="card"
    :title="task?.title || '任务详情'"
    class="task-detail-modal"
    style="width: 700px; max-width: 90vw;"
    :segmented="{
      content: 'soft',
      footer: 'soft'
    }"
  >
    <div v-if="task" class="modal-content">
      <!-- 任务头部信息 -->
      <div class="task-header-section">
        <div class="task-badges-row">
          <Badge :type="getCategoryBadgeType(task.category)">
            {{ CATEGORY_LABELS[task.category] || '其他' }}
          </Badge>
          <div class="priority-badge" :class="`priority-${getPriorityBadgeType(task.priority)}`">
            <div class="priority-dot"></div>
            <span>{{ PRIORITY_LABELS[task.priority] }}</span>
          </div>
          <div class="status-badge" :class="`status-${task.status}`">
            {{ STATUS_LABELS[task.status] }}
          </div>
          <!-- 四象限标识 -->
          <div
            v-if="task.quadrant"
            class="quadrant-badge"
            :style="{
              background: QUADRANT_CONFIG[task.quadrant].bgColor,
              borderColor: QUADRANT_CONFIG[task.quadrant].borderColor,
              color: QUADRANT_CONFIG[task.quadrant].color
            }"
          >
            <n-icon size="14">
              <GridOutline />
            </n-icon>
            <span>{{ QUADRANT_CONFIG[task.quadrant].shortLabel }}</span>
          </div>
        </div>

        <!-- 标签展示 -->
        <div v-if="task.tags && task.tags.length > 0" class="task-tags-row">
          <div
            v-for="tag in task.tags"
            :key="tag.id"
            class="task-tag"
            :style="{
              background: `${tag.color}30`,
              color: tag.color,
              borderColor: `${tag.color}60`
            }"
          >
            <n-icon size="12">
              <PricetagOutline />
            </n-icon>
            <span>{{ tag.name }}</span>
          </div>
        </div>
      </div>

      <!-- 任务进度（仅进行中任务显示） -->
      <div v-if="task.status === 'active'" class="detail-section">
        <div class="section-header">
          <n-icon size="18" class="section-icon">
            <TrendingUpOutline />
          </n-icon>
          <span class="section-title">任务进度</span>
          <n-button
            v-if="!readonly && !isEditingProgress"
            size="tiny"
            quaternary
            type="primary"
            @click="startEditProgress"
            style="margin-left: auto;"
          >
            调整进度
          </n-button>
        </div>
        <div class="progress-section">
          <template v-if="isEditingProgress">
            <div class="progress-edit">
              <n-slider
                v-model:value="editProgress"
                :min="task.progress || 0"
                :max="100"
                :step="5"
                :marks="progressEditMarks"
              />
              <div class="progress-edit-value">{{ editProgress }}%</div>
              <div class="progress-edit-hint">进度不能低于当前值 {{ task.progress || 0 }}%</div>
              <div class="progress-edit-actions">
                <n-button size="small" @click="cancelEditProgress">取消</n-button>
                <n-button size="small" type="primary" @click="saveProgress">保存</n-button>
              </div>
            </div>
          </template>
          <template v-else>
            <n-progress
              type="line"
              :percentage="task.progress || 0"
              :height="20"
              :border-radius="10"
              :fill-border-radius="10"
              indicator-placement="inside"
              :color="getProgressColor(task.progress || 0)"
            />
          </template>
        </div>
      </div>

      <!-- 任务描述 -->
      <div v-if="task.description" class="detail-section">
        <div class="section-header">
          <n-icon size="18" class="section-icon">
            <DocumentTextOutline />
          </n-icon>
          <span class="section-title">任务描述</span>
        </div>
        <div class="section-content">
          <div class="markdown-content" v-html="renderedDescription"></div>
        </div>
      </div>

      <!-- 任务信息网格 -->
      <div class="detail-section">
        <div class="section-header">
          <n-icon size="18" class="section-icon">
            <InformationCircleOutline />
          </n-icon>
          <span class="section-title">任务信息</span>
        </div>
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">任务ID</span>
            <span class="info-value">{{ task.id || 'N/A' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">分类</span>
            <span class="info-value">{{ CATEGORY_LABELS[task.category] || '其他' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">优先级</span>
            <span class="info-value">{{ PRIORITY_LABELS[task.priority] }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">状态</span>
            <span class="info-value">{{ STATUS_LABELS[task.status] }}</span>
          </div>
          <div v-if="task.gitBranch" class="info-item">
            <span class="info-label">Git分支</span>
            <span class="info-value monospace">{{ task.gitBranch }}</span>
          </div>
          <div v-if="task.estimatedHours" class="info-item">
            <span class="info-label">预估时长</span>
            <span class="info-value">{{ task.estimatedHours }} 小时</span>
          </div>
          <div v-if="task.actualHours" class="info-item">
            <span class="info-label">实际时长</span>
            <span class="info-value">{{ task.actualHours }} 小时</span>
          </div>
        </div>
      </div>

      <!-- 时间信息与里程碑 -->
      <div class="detail-section">
        <div class="section-header">
          <n-icon size="18" class="section-icon">
            <TimeOutline />
          </n-icon>
          <span class="section-title">时间轴</span>
          <n-button
            v-if="!readonly && task.status === 'active'"
            size="tiny"
            quaternary
            type="primary"
            @click="showMilestoneForm = !showMilestoneForm"
            style="margin-left: auto;"
          >
            <template #icon>
              <n-icon><AddOutline /></n-icon>
            </template>
            {{ showMilestoneForm ? '取消' : '添加里程碑' }}
          </n-button>
        </div>

        <!-- 新增里程碑表单 -->
        <div v-if="showMilestoneForm" class="milestone-form">
          <n-input
            v-model:value="newMilestoneTitle"
            placeholder="里程碑标题（必填）"
            size="small"
          />
          <n-input
            v-model:value="newMilestoneDesc"
            placeholder="描述（可选）"
            size="small"
            type="textarea"
            :autosize="{ minRows: 2, maxRows: 3 }"
          />
          <div class="milestone-progress-setting">
            <div class="milestone-progress-label">
              <span>设置进度</span>
              <span class="milestone-progress-value">{{ newMilestoneProgress }}%</span>
            </div>
            <n-slider
              v-model:value="newMilestoneProgress"
              :min="task.progress || 0"
              :max="100"
              :step="5"
              :marks="milestoneProgressMarks"
            />
            <div class="milestone-progress-hint">
              进度不能低于当前值 {{ task.progress || 0 }}%
            </div>
          </div>
          <div class="milestone-form-footer">
            <n-button size="small" @click="cancelMilestoneForm">取消</n-button>
            <n-button size="small" type="primary" @click="handleCreateMilestone">
              创建
            </n-button>
          </div>
        </div>

        <div class="timeline">
          <div v-if="task.createdAt" class="timeline-item">
            <div class="timeline-dot"></div>
            <div class="timeline-content">
              <span class="timeline-label">创建时间</span>
              <span class="timeline-value">{{ formatDateTime(task.createdAt) }}</span>
              <span class="timeline-relative">{{ formatRelativeTime(task.createdAt) }}</span>
            </div>
          </div>
          <div v-if="task.scheduledStartTime" class="timeline-item">
            <div class="timeline-dot scheduled"></div>
            <div class="timeline-content">
              <span class="timeline-label">计划开始</span>
              <span class="timeline-value">{{ formatDateTime(task.scheduledStartTime) }}</span>
              <span class="timeline-relative">{{ formatRelativeTime(task.scheduledStartTime) }}</span>
            </div>
          </div>
          <div v-if="task.startedAt" class="timeline-item">
            <div class="timeline-dot active"></div>
            <div class="timeline-content">
              <span class="timeline-label">开始时间</span>
              <span class="timeline-value">{{ formatDateTime(task.startedAt) }}</span>
              <span class="timeline-relative">{{ formatRelativeTime(task.startedAt) }}</span>
            </div>
          </div>

          <!-- 里程碑节点 -->
          <div
            v-for="milestone in milestones"
            :key="milestone.id"
            class="timeline-item milestone-item"
          >
            <div class="timeline-dot milestone"></div>
            <div class="timeline-content">
              <div class="milestone-header">
                <n-icon size="14" class="milestone-icon">
                  <FlagOutline />
                </n-icon>
                <span class="timeline-label milestone-label">{{ milestone.title }}</span>
                <n-popconfirm
                  v-if="!readonly"
                  @positive-click="handleDeleteMilestone(milestone.id!)"
                >
                  <template #trigger>
                    <n-button size="tiny" quaternary type="error" class="milestone-delete">
                      <template #icon>
                        <n-icon size="14"><TrashOutline /></n-icon>
                      </template>
                    </n-button>
                  </template>
                  确定删除这个里程碑吗？
                </n-popconfirm>
              </div>
              <span v-if="milestone.description" class="milestone-desc">{{ milestone.description }}</span>
              <div class="milestone-meta">
                <span v-if="milestone.progressSnapshot !== null && milestone.progressSnapshot !== undefined" class="milestone-progress">
                  进度: {{ milestone.progressSnapshot }}%
                </span>
                <span class="timeline-value">{{ formatDateTime(milestone.createdAt!) }}</span>
              </div>
              <span class="timeline-relative">{{ formatRelativeTime(milestone.createdAt!) }}</span>
            </div>
          </div>

          <div v-if="task.lastActiveAt" class="timeline-item">
            <div class="timeline-dot active"></div>
            <div class="timeline-content">
              <span class="timeline-label">最后活跃</span>
              <span class="timeline-value">{{ formatDateTime(task.lastActiveAt) }}</span>
              <span class="timeline-relative">{{ formatRelativeTime(task.lastActiveAt) }}</span>
            </div>
          </div>
          <div v-if="task.completedAt" class="timeline-item">
            <div class="timeline-dot completed"></div>
            <div class="timeline-content">
              <span class="timeline-label">完成时间</span>
              <span class="timeline-value">{{ formatDateTime(task.completedAt) }}</span>
              <span class="timeline-relative">{{ formatRelativeTime(task.completedAt) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 工作上下文 -->
      <div v-if="task.context" class="detail-section">
        <div class="section-header">
          <n-icon size="18" class="section-icon">
            <CodeSlashOutline />
          </n-icon>
          <span class="section-title">工作上下文</span>
        </div>
        <div class="section-content">
          <ContextViewer :context="task.context" />
        </div>
      </div>

      <!-- 四象限选择器 -->
      <div v-if="!readonly && isEditing" class="detail-section">
        <QuadrantSelector v-model="editQuadrant" />
      </div>

      <!-- 当前处理日期选择器 -->
      <div v-if="!readonly && isEditing && task.status !== 'done'" class="detail-section">
        <div class="section-header">
          <n-icon size="18" class="section-icon">
            <CalendarOutline />
          </n-icon>
          <span class="section-title">日历显示日期</span>
        </div>
        <div class="section-content">
          <n-date-picker
            v-model:formatted-value="editDisplayDate"
            value-format="yyyy-MM-dd"
            type="date"
            clearable
            placeholder="选择日期（在日历中显示此任务）"
            style="width: 100%"
          />
          <div class="date-hint">
            设置后，此任务将在日历的指定日期显示。留空则按登记日期或创建日期显示。
          </div>
        </div>
      </div>

      <!-- 标签选择器 -->
      <div v-if="!readonly && isEditing" class="detail-section">
        <TagSelector
          v-model="editTagIds"
          :available-tags="availableTags"
          @manage="showTagManager = true"
        />
      </div>

      <!-- 任务备注 -->
      <div v-if="task.notes" class="detail-section">
        <div class="section-header">
          <n-icon size="18" class="section-icon">
            <CreateOutline />
          </n-icon>
          <span class="section-title">备注</span>
        </div>
        <div class="section-content">
          <div class="markdown-content notes-content" v-html="renderedNotes"></div>
        </div>
      </div>
    </div>

    <!-- 标签管理弹窗 -->
    <TagManager
      v-model:show="showTagManager"
      :tags="availableTags"
      @create="handleCreateTag"
      @update="handleUpdateTag"
      @delete="handleDeleteTag"
      @toggle-favorite="handleToggleTagFavorite"
    />

    <template #footer>
      <n-space justify="end">
        <n-button v-if="isEditing" @click="handleCancelEdit">取消</n-button>
        <n-button v-else @click="handleClose">关闭</n-button>
        <n-button v-if="!readonly && task?.status !== 'done' && !isEditing" type="primary" @click="startEdit">
          编辑任务
        </n-button>
        <n-button v-if="isEditing" type="primary" @click="handleSaveEdit">
          保存修改
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { NModal, NIcon, NSpace, NButton, NDatePicker, NProgress, NSlider, NInput, NInputGroup, NPopconfirm, useMessage } from 'naive-ui';
import {
  DocumentTextOutline,
  InformationCircleOutline,
  TimeOutline,
  CodeSlashOutline,
  CreateOutline,
  GridOutline,
  PricetagOutline,
  CalendarOutline,
  TrendingUpOutline,
  FlagOutline,
  AddOutline,
  TrashOutline,
} from '@vicons/ionicons5';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/zh-cn';
import { marked } from 'marked';
import { CATEGORY_LABELS, PRIORITY_LABELS, STATUS_LABELS, QUADRANT_CONFIG } from '@/types/task';
import type { Task, TaskQuadrant, Tag, TaskMilestone } from '@/types/task';
import { tagApi } from '@/api/tagApi';
import { taskApi } from '@/api/taskApi';
import ContextViewer from './ContextViewer.vue';
import Badge from './Badge.vue';
import QuadrantSelector from './QuadrantSelector.vue';
import TagSelector from './TagSelector.vue';
import TagManager from './TagManager.vue';

dayjs.extend(relativeTime);
dayjs.locale('zh-cn');

const message = useMessage();

const props = withDefaults(
  defineProps<{
    task: Task | null;
    show: boolean;
    readonly?: boolean;
  }>(),
  {
    readonly: false,
  }
);

const emit = defineEmits<{
  'update:show': [value: boolean];
  'update': [task: Task, updates: Partial<Task>];
}>();

const showModal = computed({
  get: () => props.show,
  set: (value) => emit('update:show', value),
});

// 将描述的 Markdown 转换为 HTML
const renderedDescription = computed(() => {
  if (!props.task?.description) return '';
  try {
    return marked(props.task.description);
  } catch (error) {
    console.error('Markdown 渲染失败:', error);
    return props.task.description;
  }
});

// 将备注的 Markdown 转换为 HTML
const renderedNotes = computed(() => {
  if (!props.task?.notes) return '';
  try {
    return marked(props.task.notes);
  } catch (error) {
    console.error('Markdown 渲染失败:', error);
    return props.task.notes;
  }
});

// 编辑状态
const isEditing = ref(false);
const editQuadrant = ref<TaskQuadrant>('urgent_not_important');
const editTagIds = ref<number[]>([]);
const editDisplayDate = ref<string | null>(null);
const showTagManager = ref(false);

// 标签数据
const availableTags = ref<Tag[]>([]);

// 里程碑数据
const milestones = ref<TaskMilestone[]>([]);
const showMilestoneForm = ref(false);
const newMilestoneTitle = ref('');
const newMilestoneDesc = ref('');
const newMilestoneProgress = ref(0);

// 里程碑进度标记
const milestoneProgressMarks = computed(() => {
  const currentProgress = props.task?.progress || 0;
  const marks: Record<number, string> = {};
  marks[currentProgress] = `${currentProgress}%`;
  if (currentProgress < 50) marks[50] = '50%';
  if (currentProgress < 75) marks[75] = '75%';
  marks[100] = '100%';
  return marks;
});

// 进度编辑
const editProgress = ref(0);
const isEditingProgress = ref(false);

// 进度编辑标记
const progressEditMarks = computed(() => {
  const currentProgress = props.task?.progress || 0;
  const marks: Record<number, string> = {};
  marks[currentProgress] = `${currentProgress}%`;
  if (currentProgress < 50) marks[50] = '50%';
  if (currentProgress < 75) marks[75] = '75%';
  marks[100] = '100%';
  return marks;
});

// 加载标签
onMounted(async () => {
  await loadTags();
});

async function loadTags() {
  try {
    availableTags.value = await tagApi.getAllTags();
  } catch (error) {
    console.error('加载标签失败:', error);
  }
}

// 加载里程碑
async function loadMilestones() {
  if (props.task?.id) {
    try {
      milestones.value = await taskApi.getTaskMilestones(props.task.id);
    } catch (error) {
      console.error('加载里程碑失败:', error);
    }
  }
}

// 创建里程碑
async function handleCreateMilestone() {
  if (!props.task?.id || !newMilestoneTitle.value.trim()) {
    message.warning('请输入里程碑标题');
    return;
  }
  try {
    // 使用用户设置的进度值
    const progressValue = newMilestoneProgress.value;

    await taskApi.createTaskMilestone(
      props.task.id,
      newMilestoneTitle.value.trim(),
      newMilestoneDesc.value.trim() || undefined,
      progressValue
    );

    // 如果进度有变化，同时更新任务进度
    if (progressValue > (props.task.progress || 0)) {
      await taskApi.updateTaskProgress(props.task.id, progressValue);
      emit('update', props.task, { progress: progressValue });
    }

    message.success('里程碑创建成功');
    newMilestoneTitle.value = '';
    newMilestoneDesc.value = '';
    showMilestoneForm.value = false;
    await loadMilestones();
  } catch (error) {
    console.error('创建里程碑失败:', error);
    message.error('创建里程碑失败');
  }
}

// 取消里程碑表单
function cancelMilestoneForm() {
  showMilestoneForm.value = false;
  newMilestoneTitle.value = '';
  newMilestoneDesc.value = '';
  newMilestoneProgress.value = props.task?.progress || 0;
}

// 删除里程碑
async function handleDeleteMilestone(milestoneId: number) {
  try {
    await taskApi.deleteTaskMilestone(milestoneId);
    message.success('里程碑删除成功');
    await loadMilestones();
  } catch (error) {
    console.error('删除里程碑失败:', error);
    message.error('删除里程碑失败');
  }
}

// 开始编辑进度
function startEditProgress() {
  editProgress.value = props.task?.progress || 0;
  isEditingProgress.value = true;
}

// 保存进度
async function saveProgress() {
  if (!props.task?.id) return;
  try {
    await taskApi.updateTaskProgress(props.task.id, editProgress.value);
    message.success('进度更新成功');
    isEditingProgress.value = false;
    // 通知父组件更新
    emit('update', props.task, { progress: editProgress.value });
  } catch (error) {
    console.error('更新进度失败:', error);
    message.error('更新进度失败');
  }
}

// 取消编辑进度
function cancelEditProgress() {
  isEditingProgress.value = false;
  editProgress.value = props.task?.progress || 0;
}

const handleClose = () => {
  isEditing.value = false;
  showModal.value = false;
};

const startEdit = () => {
  if (props.task) {
    isEditing.value = true;
    editQuadrant.value = props.task.quadrant || 'urgent_not_important';
    editTagIds.value = props.task.tags?.map(t => t.id!).filter(id => id !== undefined) || [];
    editDisplayDate.value = props.task.displayDate || null;
  }
};

const handleCancelEdit = () => {
  isEditing.value = false;
};

const handleSaveEdit = async () => {
  if (props.task) {
    const updates: Partial<Task> = {
      quadrant: editQuadrant.value,
      displayDate: editDisplayDate.value || undefined,
    };

    // 调用emit更新任务基本信息(包括四象限和日历显示日期)
    emit('update', props.task, updates);

    // 单独处理标签的更新
    try {
      await tagApi.removeAllTagsFromTask(props.task.id!);
      if (editTagIds.value.length > 0) {
        await tagApi.addTagsToTask(props.task.id!, editTagIds.value);
      }
      message.success('任务更新成功');
    } catch (error) {
      console.error('更新标签失败:', error);
      message.warning('任务更新成功,但标签更新失败');
    }

    isEditing.value = false;
  }
};

// 标签管理回调
const handleCreateTag = async (tag: Omit<Tag, 'id'>) => {
  try {
    await tagApi.createTag(tag.name, tag.color);
    await loadTags();
    message.success('标签创建成功');
  } catch (error) {
    console.error('创建标签失败:', error);
    message.error('创建标签失败');
  }
};

const handleUpdateTag = async (id: number, updates: Partial<Tag>) => {
  try {
    await tagApi.updateTag(id, updates.name!, updates.color!);
    await loadTags();
    message.success('标签更新成功');
  } catch (error) {
    console.error('更新标签失败:', error);
    message.error('更新标签失败');
  }
};

const handleDeleteTag = async (id: number) => {
  try {
    await tagApi.deleteTag(id);
    await loadTags();
    editTagIds.value = editTagIds.value.filter(tagId => tagId !== id);
    message.success('标签删除成功');
  } catch (error) {
    console.error('删除标签失败:', error);
    message.error('删除标签失败');
  }
};

const handleToggleTagFavorite = async (id: number, isFavorite: boolean) => {
  try {
    await tagApi.toggleTagFavorite(id, isFavorite);
    await loadTags();
  } catch (error) {
    console.error('切换常用标签失败:', error);
    message.error('切换常用标签失败');
  }
};

// 重置编辑状态，加载里程碑
watch(() => props.show, async (newVal) => {
  if (newVal) {
    // 打开时加载里程碑
    await loadMilestones();
    editProgress.value = props.task?.progress || 0;
    newMilestoneProgress.value = props.task?.progress || 0;
  } else {
    // 关闭时重置状态
    isEditing.value = false;
    isEditingProgress.value = false;
    showMilestoneForm.value = false;
    newMilestoneTitle.value = '';
    newMilestoneDesc.value = '';
    newMilestoneProgress.value = 0;
  }
});

// 打开里程碑表单时初始化进度值
watch(showMilestoneForm, (newVal) => {
  if (newVal) {
    newMilestoneProgress.value = props.task?.progress || 0;
  }
});

const formatDateTime = (dateString: string) => {
  return dayjs(dateString).format('YYYY-MM-DD HH:mm:ss');
};

const formatRelativeTime = (dateString: string) => {
  return dayjs(dateString).fromNow();
};

const getCategoryBadgeType = (category: string | undefined) => {
  if (!category) return 'default';
  const categoryMap: Record<string, 'Backend' | 'Database' | 'Feature' | 'Docs' | 'default'> = {
    'backend': 'Backend',
    'database': 'Database',
    'feature': 'Feature',
    'docs': 'Docs',
    'dev': 'Backend',
    'ops': 'Database',
    'study': 'Feature',
  };
  return categoryMap[category] || 'default';
};

const getPriorityBadgeType = (priority: number) => {
  const priorityMap: Record<number, 'high' | 'medium' | 'low'> = {
    1: 'high',
    2: 'medium',
    3: 'low',
  };
  return priorityMap[priority] || 'medium';
};

// 获取进度颜色
const getProgressColor = (progress: number) => {
  if (progress < 30) return '#f59e0b';  // 黄色
  if (progress < 70) return '#6366f1';  // 紫色
  return '#10b981';  // 绿色
};
</script>

<style scoped>
.task-detail-modal {
  border-radius: 16px;
}

.modal-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
  color: var(--text-primary);
}

/* 任务头部 */
.task-header-section {
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-default);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.task-badges-row {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  align-items: center;
}

.quadrant-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  border: 1px solid;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.task-tags-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  align-items: center;
}

.task-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  border: 1px solid;
}

.priority-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.priority-badge .priority-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.priority-badge.priority-high {
  background: rgba(244, 63, 94, 0.15);
  color: #fca5a5;
  border: 1px solid rgba(244, 63, 94, 0.3);
}

.priority-badge.priority-high .priority-dot {
  background: #f43f5e;
  box-shadow: 0 0 6px rgba(244, 63, 94, 0.6);
}

.priority-badge.priority-medium {
  background: rgba(245, 158, 11, 0.15);
  color: #fcd34d;
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.priority-badge.priority-medium .priority-dot {
  background: #f59e0b;
  box-shadow: 0 0 6px rgba(245, 158, 11, 0.6);
}

.priority-badge.priority-low {
  background: rgba(16, 185, 129, 0.15);
  color: #6ee7b7;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.priority-badge.priority-low .priority-dot {
  background: #10b981;
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.6);
}

.status-badge {
  display: inline-flex;
  align-items: center;
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
}

.status-badge.status-todo {
  background: rgba(100, 116, 139, 0.15);
  color: #94a3b8;
  border: 1px solid rgba(100, 116, 139, 0.3);
}

.status-badge.status-active {
  background: rgba(99, 102, 241, 0.15);
  color: #a5b4fc;
  border: 1px solid rgba(99, 102, 241, 0.3);
}

.status-badge.status-done {
  background: rgba(16, 185, 129, 0.15);
  color: #6ee7b7;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.status-badge.status-deferred {
  background: rgba(245, 158, 11, 0.15);
  color: #fcd34d;
  border: 1px solid rgba(245, 158, 11, 0.3);
}

/* 详情区块 */
.detail-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.section-icon {
  color: #6366f1;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.section-content {
  padding-left: 26px;
}

/* Markdown 内容样式 */
.markdown-content {
  font-size: 14px;
  line-height: 1.7;
  color: var(--text-primary);
  word-break: break-word;
}

.markdown-content :deep(h1),
.markdown-content :deep(h2),
.markdown-content :deep(h3),
.markdown-content :deep(h4),
.markdown-content :deep(h5),
.markdown-content :deep(h6) {
  color: var(--text-primary);
  font-weight: 600;
  margin-top: 1em;
  margin-bottom: 0.5em;
  line-height: 1.3;
}

.markdown-content :deep(h1) {
  font-size: 1.5em;
  border-bottom: 1px solid var(--border-default);
  padding-bottom: 0.3em;
}

.markdown-content :deep(h2) {
  font-size: 1.3em;
}

.markdown-content :deep(h3) {
  font-size: 1.1em;
}

.markdown-content :deep(p) {
  margin-top: 0.5em;
  margin-bottom: 0.5em;
}

.markdown-content :deep(p:first-child) {
  margin-top: 0;
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  margin-top: 0.5em;
  margin-bottom: 0.5em;
  padding-left: 1.5em;
}

.markdown-content :deep(li) {
  margin-top: 0.25em;
  margin-bottom: 0.25em;
}

.markdown-content :deep(code) {
  background: var(--bg-elevated);
  color: #fbbf24;
  padding: 0.15em 0.4em;
  border-radius: 4px;
  font-size: 0.9em;
  font-family: 'Consolas', 'Monaco', monospace;
}

.markdown-content :deep(pre) {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  padding: 0.8em;
  overflow-x: auto;
  margin: 0.8em 0;
}

.markdown-content :deep(pre code) {
  background: transparent;
  color: var(--text-primary);
  padding: 0;
}

.markdown-content :deep(blockquote) {
  border-left: 3px solid var(--accent-primary);
  margin: 0.8em 0;
  padding-left: 1em;
  color: var(--text-primary);
  font-style: italic;
}

.markdown-content :deep(a) {
  color: var(--accent-primary);
  text-decoration: none;
  transition: color 0.2s;
}

.markdown-content :deep(a:hover) {
  color: var(--accent-secondary);
  text-decoration: underline;
}

.markdown-content :deep(strong) {
  color: var(--text-primary);
  font-weight: 600;
}

.markdown-content :deep(em) {
  color: var(--text-primary);
}

.markdown-content :deep(hr) {
  border: none;
  border-top: 1px solid var(--border-default);
  margin: 1em 0;
}

.markdown-content :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 0.8em 0;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid var(--border-default);
  padding: 0.5em 0.8em;
  text-align: left;
}

.markdown-content :deep(th) {
  background: var(--bg-elevated);
  font-weight: 600;
  color: var(--text-primary);
}

.markdown-content :deep(tr:nth-child(even)) {
  background: var(--bg-surface);
}

/* 备注区块特殊样式 */
.notes-content {
  padding: 12px;
  background: var(--bg-surface);
  border-radius: 8px;
  border-left: 3px solid var(--accent-primary);
}

.notes-content :deep(p:first-child) {
  margin-top: 0;
}

.notes-content :deep(p:last-child) {
  margin-bottom: 0;
}

/* 信息网格 */
.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  padding-left: 26px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.info-value {
  font-size: 14px;
  color: var(--text-primary);
  font-weight: 500;
}

.monospace {
  font-family: 'Consolas', 'Monaco', monospace;
  background: var(--bg-surface);
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid var(--border-default);
}

/* 时间轴 */
.timeline {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-left: 26px;
}

.timeline-item {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  position: relative;
}

.timeline-item:not(:last-child)::after {
  content: '';
  position: absolute;
  left: 5px;
  top: 16px;
  width: 1px;
  height: calc(100% + 16px);
  background: var(--border-default);
}

.timeline-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--text-secondary);
  border: 2px solid var(--bg-base);
  flex-shrink: 0;
  margin-top: 4px;
  position: relative;
  z-index: 1;
}

.timeline-dot.active {
  background: #6366f1;
  box-shadow: 0 0 8px rgba(99, 102, 241, 0.6);
}

.timeline-dot.completed {
  background: #10b981;
  box-shadow: 0 0 8px rgba(16, 185, 129, 0.6);
}

.timeline-dot.scheduled {
  background: #f59e0b;
  box-shadow: 0 0 8px rgba(245, 158, 11, 0.6);
}

.timeline-content {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
}

.timeline-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}

.timeline-value {
  font-size: 14px;
  color: var(--text-primary);
  font-family: 'Consolas', 'Monaco', monospace;
}

.timeline-relative {
  font-size: 12px;
  color: var(--text-secondary);
  font-style: italic;
}

/* 备注文本 */
.notes-text {
  font-size: 13px;
  line-height: 1.7;
  color: var(--text-primary);
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-style: italic;
  padding: 12px;
  background: var(--bg-surface);
  border-radius: 8px;
  border-left: 3px solid var(--accent-primary);
}

/* 日期提示 */
.date-hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

/* 进度区块样式 */
.progress-section {
  padding-left: 26px;
}

.progress-edit {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.progress-edit-value {
  text-align: center;
  font-size: 24px;
  font-weight: 600;
  color: var(--accent-primary);
}

.progress-edit-hint {
  font-size: 11px;
  color: var(--text-secondary);
  text-align: center;
  margin-top: -8px;
}

.progress-edit-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

/* 里程碑表单 */
.milestone-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: var(--bg-surface);
  border-radius: 8px;
  margin-bottom: 16px;
  border: 1px solid var(--border-hover);
}

.milestone-form-footer {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 8px;
}

/* 里程碑进度设置 */
.milestone-progress-setting {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.milestone-progress-label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  color: var(--text-primary);
}

.milestone-progress-value {
  font-size: 18px;
  font-weight: 600;
  color: var(--accent-primary);
}

.milestone-progress-hint {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: -4px;
}

/* 里程碑样式 */
.timeline-dot.milestone {
  background: #ec4899;
  box-shadow: 0 0 8px rgba(236, 72, 153, 0.6);
}

.milestone-item {
  background: rgba(236, 72, 153, 0.05);
  padding: 8px;
  border-radius: 8px;
  margin: -8px;
  margin-left: 0;
}

.milestone-header {
  display: flex;
  align-items: center;
  gap: 6px;
}

.milestone-icon {
  color: #ec4899;
}

.milestone-label {
  color: #f9a8d4 !important;
  font-weight: 600 !important;
  font-size: 13px !important;
}

.milestone-delete {
  margin-left: auto;
  opacity: 0.6;
  transition: opacity 0.2s;
}

.milestone-delete:hover {
  opacity: 1;
}

.milestone-desc {
  font-size: 13px;
  color: var(--text-primary);
  margin-top: 4px;
}

.milestone-meta {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-top: 4px;
}

.milestone-progress {
  font-size: 12px;
  color: #ec4899;
  background: rgba(236, 72, 153, 0.15);
  padding: 2px 8px;
  border-radius: 4px;
}

/* 响应式 */
@media (max-width: 768px) {
  .info-grid {
    grid-template-columns: 1fr;
  }
}
</style>
