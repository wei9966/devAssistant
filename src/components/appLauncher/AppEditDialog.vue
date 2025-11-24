<template>
  <n-modal
    v-model:show="dialogVisible"
    preset="card"
    :title="isEdit ? '编辑应用' : '添加应用'"
    class="app-edit-dialog"
    style="width: 600px"
    :mask-closable="false"
  >
    <n-form ref="formRef" :model="formData" :rules="rules" label-placement="left" label-width="90">
      <!-- 应用名称 -->
      <n-form-item label="应用名称" path="name">
        <n-input
          v-model:value="formData.name"
          placeholder="例如：Visual Studio Code"
          @keydown.enter.prevent
        />
      </n-form-item>

      <!-- 应用路径 -->
      <n-form-item label="应用路径" path="path">
        <n-input-group>
          <n-input
            v-model:value="formData.path"
            placeholder="例如：C:\Program Files\VSCode\Code.exe"
            style="flex: 1"
            @keydown.enter.prevent
          />
          <n-button @click="handleSelectFile">
            <template #icon>
              <n-icon><FolderOpenOutline /></n-icon>
            </template>
            浏览
          </n-button>
        </n-input-group>
      </n-form-item>

      <!-- 应用分类 -->
      <n-form-item label="应用分类" path="category">
        <n-select
          v-model:value="formData.category"
          :options="categoryOptions"
          placeholder="选择分类"
        />
      </n-form-item>

      <!-- 应用标签 -->
      <n-form-item label="应用标签" path="tags">
        <n-dynamic-tags v-model:value="formData.tags" />
      </n-form-item>

      <!-- 启动参数 -->
      <n-form-item label="启动参数">
        <n-input
          v-model:value="formData.launchArgs"
          placeholder="可选，例如：--new-window"
          @keydown.enter.prevent
        />
      </n-form-item>

      <!-- 置顶选项 -->
      <n-form-item label="置顶显示">
        <n-switch v-model:value="formData.isPinned" />
      </n-form-item>

      <!-- 图标预览 -->
      <n-form-item v-if="formData.icon" label="应用图标">
        <div class="icon-preview">
          <img :src="formData.icon" alt="应用图标" />
          <n-button text @click="handleRemoveIcon">
            <template #icon>
              <n-icon><TrashOutline /></n-icon>
            </template>
            移除图标
          </n-button>
        </div>
      </n-form-item>
    </n-form>

    <template #footer>
      <div class="dialog-footer">
        <n-button @click="handleCancel">取消</n-button>
        <n-button type="primary" @click="handleSubmit" :loading="saving">
          {{ isEdit ? '保存' : '添加' }}
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
  NInputGroup,
  NSelect,
  NDynamicTags,
  NSwitch,
  NButton,
  NIcon,
  useMessage,
  type FormInst,
  type FormRules,
} from 'naive-ui';
import { FolderOpenOutline, TrashOutline } from '@vicons/ionicons5';
import type { AppItem, Category } from '@/types/appLauncher';

interface AppFormData {
  name: string;
  path: string;
  category: string;
  tags: string[];
  launchArgs: string;
  isPinned: boolean;
  icon?: string;
}

const props = withDefaults(
  defineProps<{
    show: boolean;
    app?: AppItem | null;
    categories: Category[];
  }>(),
  {
    app: null,
  }
);

const emit = defineEmits<{
  'update:show': [value: boolean];
  submit: [data: AppFormData];
  cancel: [];
}>();

const message = useMessage();
const formRef = ref<FormInst>();
const saving = ref(false);

const dialogVisible = computed({
  get: () => props.show,
  set: (val) => emit('update:show', val),
});

const isEdit = computed(() => !!props.app);

// 表单数据
const defaultFormData: AppFormData = {
  name: '',
  path: '',
  category: 'other',
  tags: [],
  launchArgs: '',
  isPinned: false,
  icon: undefined,
};

const formData = ref<AppFormData>({ ...defaultFormData });

// 分类选项
const categoryOptions = computed(() => {
  return props.categories
    .filter((cat) => cat.id !== 'all')
    .map((cat) => ({
      label: `${cat.icon || ''} ${cat.name}`,
      value: cat.id,
    }));
});

// 表单验证规则
const rules: FormRules = {
  name: [
    {
      required: true,
      message: '请输入应用名称',
      trigger: ['input', 'blur'],
    },
  ],
  path: [
    {
      required: true,
      message: '请选择应用路径',
      trigger: ['input', 'blur'],
    },
  ],
  category: [
    {
      required: true,
      message: '请选择应用分类',
      trigger: ['change', 'blur'],
    },
  ],
};

// 监听app变化，初始化表单
watch(
  () => props.app,
  (newApp) => {
    if (newApp) {
      formData.value = {
        name: newApp.name,
        path: newApp.path,
        category: newApp.category,
        tags: [...newApp.tags],
        launchArgs: newApp.launchArgs || '',
        isPinned: newApp.isPinned,
        icon: newApp.icon,
      };
    } else {
      formData.value = { ...defaultFormData };
    }
  },
  { immediate: true }
);

// 选择文件
const handleSelectFile = async () => {
  try {
    // 调用Tauri API选择文件
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: '应用程序',
          extensions: ['exe', 'lnk'],
        },
      ],
    });

    if (selected && typeof selected === 'string') {
      formData.value.path = selected;

      // 如果名称为空，从路径提取文件名
      if (!formData.value.name) {
        const fileName = selected.split(/[\\\/]/).pop()?.replace(/\.(exe|lnk)$/i, '') || '';
        formData.value.name = fileName;
      }

      // TODO: 调用后端API提取图标
      // const icon = await extractIcon(selected);
      // formData.value.icon = icon;
    }
  } catch (error) {
    message.error('选择文件失败');
    console.error('选择文件失败:', error);
  }
};

// 移除图标
const handleRemoveIcon = () => {
  formData.value.icon = undefined;
};

// 提交表单
const handleSubmit = async () => {
  try {
    await formRef.value?.validate();
    saving.value = true;

    emit('submit', { ...formData.value });

    // 重置表单
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
.app-edit-dialog {
  background: rgba(15, 23, 42, 0.95);
  backdrop-filter: blur(16px);
}

.app-edit-dialog :deep(.n-card) {
  background: rgba(30, 41, 59, 0.8);
  border: 1px solid rgba(51, 65, 85, 0.6);
}

.app-edit-dialog :deep(.n-card__header) {
  border-bottom: 1px solid rgba(51, 65, 85, 0.5);
  color: #e2e8f0;
  font-weight: 600;
}

.icon-preview {
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon-preview img {
  width: 48px;
  height: 48px;
  object-fit: contain;
  padding: 8px;
  background: rgba(99, 102, 241, 0.1);
  border-radius: 8px;
  border: 1px solid rgba(99, 102, 241, 0.2);
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

/* 表单样式覆盖 */
.app-edit-dialog :deep(.n-form-item-label) {
  color: #cbd5e1;
}

.app-edit-dialog :deep(.n-input) {
  background: rgba(15, 23, 42, 0.5);
  border-color: rgba(51, 65, 85, 0.5);
}

.app-edit-dialog :deep(.n-input:hover) {
  border-color: rgba(99, 102, 241, 0.4);
}

.app-edit-dialog :deep(.n-input:focus) {
  border-color: #6366f1;
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.1);
}

.app-edit-dialog :deep(.n-base-selection) {
  background: rgba(15, 23, 42, 0.5);
  border-color: rgba(51, 65, 85, 0.5);
}

.app-edit-dialog :deep(.n-base-selection:hover) {
  border-color: rgba(99, 102, 241, 0.4);
}

.app-edit-dialog :deep(.n-base-selection.n-base-selection--active) {
  border-color: #6366f1;
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.1);
}
</style>
