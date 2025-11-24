<template>
  <n-modal
    v-model:show="dialogVisible"
    preset="card"
    :title="isEdit ? '编辑工作流' : '创建工作流'"
    class="workflow-edit-dialog"
    style="width: 700px"
    :mask-closable="false"
  >
    <n-form ref="formRef" :model="formData" :rules="rules" label-placement="left" label-width="100">
      <!-- 工作流名称 -->
      <n-form-item label="工作流名称" path="name">
        <n-input
          v-model:value="formData.name"
          placeholder="例如：前端开发环境"
          @keydown.enter.prevent
        />
      </n-form-item>

      <!-- 启动延迟 -->
      <n-form-item label="启动间隔">
        <n-input-number
          v-model:value="formData.launchDelay"
          :min="0"
          :max="10000"
          :step="100"
          placeholder="毫秒"
          style="width: 100%"
        >
          <template #suffix>ms</template>
        </n-input-number>
        <template #feedback>
          <span style="color: #94a3b8; font-size: 12px">
            应用之间的启动间隔时间（0表示同时启动）
          </span>
        </template>
      </n-form-item>

      <!-- 选择应用 -->
      <n-form-item label="选择应用" path="appIds">
        <div class="app-selector">
          <!-- 已选应用列表 -->
          <div v-if="selectedApps.length > 0" class="selected-apps">
            <TransitionGroup name="list">
              <div
                v-for="(app, index) in selectedApps"
                :key="app.id"
                class="selected-app-item"
              >
                <div class="app-order">{{ index + 1 }}</div>
                <div class="app-icon">
                  <img v-if="app.icon" :src="app.icon" :alt="app.name" />
                  <n-icon v-else size="24"><AppsOutline /></n-icon>
                </div>
                <div class="app-info">
                  <div class="app-name">{{ app.name }}</div>
                  <div class="app-path">{{ app.path }}</div>
                </div>
                <div class="app-actions">
                  <n-button
                    text
                    size="small"
                    :disabled="index === 0"
                    @click="moveUp(index)"
                    title="上移"
                  >
                    <template #icon>
                      <n-icon><ChevronUp /></n-icon>
                    </template>
                  </n-button>
                  <n-button
                    text
                    size="small"
                    :disabled="index === selectedApps.length - 1"
                    @click="moveDown(index)"
                    title="下移"
                  >
                    <template #icon>
                      <n-icon><ChevronDown /></n-icon>
                    </template>
                  </n-button>
                  <n-button text size="small" @click="removeApp(index)" title="移除">
                    <template #icon>
                      <n-icon><CloseOutline /></n-icon>
                    </template>
                  </n-button>
                </div>
              </div>
            </TransitionGroup>
          </div>

          <!-- 添加应用按钮 -->
          <n-button dashed block @click="showAppPicker = true">
            <template #icon>
              <n-icon><AddOutline /></n-icon>
            </template>
            添加应用
          </n-button>
        </div>
      </n-form-item>
    </n-form>

    <!-- 应用选择器 -->
    <n-modal
      v-model:show="showAppPicker"
      preset="card"
      title="选择应用"
      style="width: 500px"
      class="app-picker-modal"
    >
      <n-input
        v-model:value="appSearchKeyword"
        placeholder="搜索应用..."
        clearable
        style="margin-bottom: 16px"
      >
        <template #prefix>
          <n-icon><SearchOutline /></n-icon>
        </template>
      </n-input>

      <div class="app-picker-list">
        <div
          v-for="app in filteredAvailableApps"
          :key="app.id"
          class="app-picker-item"
          :class="{ selected: isAppSelected(app.id) }"
          @click="toggleApp(app)"
        >
          <div class="app-icon-small">
            <img v-if="app.icon" :src="app.icon" :alt="app.name" />
            <n-icon v-else size="20"><AppsOutline /></n-icon>
          </div>
          <div class="app-info-small">
            <div class="app-name-small">{{ app.name }}</div>
            <div class="app-category-small">{{ getCategoryName(app.category) }}</div>
          </div>
          <n-icon v-if="isAppSelected(app.id)" size="20" color="#6366f1">
            <CheckmarkCircle />
          </n-icon>
        </div>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <n-button @click="showAppPicker = false">完成</n-button>
        </div>
      </template>
    </n-modal>

    <template #footer>
      <div class="dialog-footer">
        <n-button @click="handleCancel">取消</n-button>
        <n-button type="primary" @click="handleSubmit" :loading="saving">
          {{ isEdit ? '保存' : '创建' }}
        </n-button>
      </div>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import {
  NModal,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NButton,
  NIcon,
  useMessage,
  type FormInst,
  type FormRules,
} from 'naive-ui';
import {
  AppsOutline,
  AddOutline,
  CloseOutline,
  ChevronUp,
  ChevronDown,
  SearchOutline,
  CheckmarkCircle,
} from '@vicons/ionicons5';
import type { AppItem, Workflow, Category } from '@/types/appLauncher';

interface WorkflowFormData {
  name: string;
  appIds: string[];
  launchDelay: number;
}

const props = withDefaults(
  defineProps<{
    show: boolean;
    workflow?: Workflow | null;
    apps: AppItem[];
    categories: Category[];
  }>(),
  {
    workflow: null,
  }
);

const emit = defineEmits<{
  'update:show': [value: boolean];
  submit: [data: WorkflowFormData];
  cancel: [];
}>();

const message = useMessage();
const formRef = ref<FormInst>();
const saving = ref(false);
const showAppPicker = ref(false);
const appSearchKeyword = ref('');

const dialogVisible = computed({
  get: () => props.show,
  set: (val) => emit('update:show', val),
});

const isEdit = computed(() => !!props.workflow);

// 表单数据
const defaultFormData: WorkflowFormData = {
  name: '',
  appIds: [],
  launchDelay: 500,
};

const formData = ref<WorkflowFormData>({ ...defaultFormData });

// 已选应用列表
const selectedApps = computed(() => {
  return formData.value.appIds
    .map((id) => props.apps.find((app) => app.id === id))
    .filter((app): app is AppItem => !!app);
});

// 可选应用列表（排除已选的）
const availableApps = computed(() => {
  return props.apps.filter((app) => !app.isHidden);
});

// 过滤后的可选应用
const filteredAvailableApps = computed(() => {
  if (!appSearchKeyword.value) return availableApps.value;
  const keyword = appSearchKeyword.value.toLowerCase();
  return availableApps.value.filter((app) => app.name.toLowerCase().includes(keyword));
});

// 表单验证规则
const rules: FormRules = {
  name: [{ required: true, message: '请输入工作流名称', trigger: ['input', 'blur'] }],
  appIds: [
    {
      type: 'array',
      required: true,
      message: '请至少选择一个应用',
      trigger: ['change', 'blur'],
    },
  ],
};

// 监听workflow变化
watch(
  () => props.workflow,
  (newWorkflow) => {
    if (newWorkflow) {
      formData.value = {
        name: newWorkflow.name,
        appIds: [...newWorkflow.appIds],
        launchDelay: newWorkflow.launchDelay || 500,
      };
    } else {
      formData.value = { ...defaultFormData };
    }
  },
  { immediate: true }
);

// 判断应用是否已选
const isAppSelected = (appId: string) => {
  return formData.value.appIds.includes(appId);
};

// 切换应用选择
const toggleApp = (app: AppItem) => {
  const index = formData.value.appIds.indexOf(app.id);
  if (index === -1) {
    formData.value.appIds.push(app.id);
  } else {
    formData.value.appIds.splice(index, 1);
  }
};

// 移除应用
const removeApp = (index: number) => {
  formData.value.appIds.splice(index, 1);
};

// 上移应用
const moveUp = (index: number) => {
  if (index > 0) {
    const temp = formData.value.appIds[index];
    formData.value.appIds[index] = formData.value.appIds[index - 1];
    formData.value.appIds[index - 1] = temp;
  }
};

// 下移应用
const moveDown = (index: number) => {
  if (index < formData.value.appIds.length - 1) {
    const temp = formData.value.appIds[index];
    formData.value.appIds[index] = formData.value.appIds[index + 1];
    formData.value.appIds[index + 1] = temp;
  }
};

// 获取分类名称
const getCategoryName = (categoryId: string) => {
  return props.categories.find((cat) => cat.id === categoryId)?.name || '其他';
};

// 提交表单
const handleSubmit = async () => {
  try {
    await formRef.value?.validate();
    saving.value = true;

    emit('submit', { ...formData.value });

    formData.value = { ...defaultFormData };
    dialogVisible.value = false;
  } catch (error) {
    console.error('表单验证失败:', error);
  } finally {
    saving.value = false;
  }
};

// 取消
const handleCancel = () => {
  formData.value = { ...defaultFormData };
  dialogVisible.value = false;
  emit('cancel');
};
</script>

<style scoped>
.workflow-edit-dialog,
.app-picker-modal {
  background: rgba(15, 23, 42, 0.95);
  backdrop-filter: blur(16px);
}

.workflow-edit-dialog :deep(.n-card),
.app-picker-modal :deep(.n-card) {
  background: rgba(30, 41, 59, 0.8);
  border: 1px solid rgba(51, 65, 85, 0.6);
}

/* 应用选择器 */
.app-selector {
  width: 100%;
}

.selected-apps {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
  max-height: 300px;
  overflow-y: auto;
  padding-right: 4px;
}

.selected-app-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 10px;
  transition: all 0.2s;
}

.selected-app-item:hover {
  background: rgba(30, 41, 59, 0.7);
  border-color: rgba(99, 102, 241, 0.4);
}

.app-order {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(99, 102, 241, 0.2);
  border-radius: 50%;
  font-size: 12px;
  font-weight: 600;
  color: #a5b4fc;
  flex-shrink: 0;
}

.app-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(99, 102, 241, 0.1);
  border-radius: 8px;
  flex-shrink: 0;
}

.app-icon img {
  width: 28px;
  height: 28px;
  object-fit: contain;
}

.app-info {
  flex: 1;
  min-width: 0;
}

.app-name {
  font-size: 14px;
  color: #e2e8f0;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-path {
  font-size: 11px;
  color: #64748b;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-top: 2px;
}

.app-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.app-actions :deep(.n-button) {
  color: #94a3b8;
}

.app-actions :deep(.n-button:hover:not(:disabled)) {
  color: #e2e8f0;
}

/* 应用选择器弹窗 */
.app-picker-list {
  max-height: 400px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.app-picker-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: rgba(15, 23, 42, 0.4);
  border: 1px solid rgba(51, 65, 85, 0.4);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.app-picker-item:hover {
  background: rgba(30, 41, 59, 0.6);
  border-color: rgba(99, 102, 241, 0.4);
}

.app-picker-item.selected {
  background: rgba(99, 102, 241, 0.15);
  border-color: #6366f1;
}

.app-icon-small {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(99, 102, 241, 0.1);
  border-radius: 6px;
  flex-shrink: 0;
}

.app-icon-small img {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.app-info-small {
  flex: 1;
  min-width: 0;
}

.app-name-small {
  font-size: 13px;
  color: #e2e8f0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-category-small {
  font-size: 11px;
  color: #64748b;
  margin-top: 2px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

/* 列表过渡动画 */
.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from {
  opacity: 0;
  transform: translateX(-20px);
}

.list-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

.list-move {
  transition: transform 0.3s ease;
}
</style>
