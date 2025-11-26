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

      <!-- 选择类型 -->
      <n-form-item label="添加类型">
        <n-radio-group v-model:value="selectMode" name="selectMode">
          <n-radio value="file">文件</n-radio>
          <n-radio value="folder">文件夹</n-radio>
        </n-radio-group>
      </n-form-item>

      <!-- 应用路径 -->
      <n-form-item label="路径" path="path">
        <n-input-group>
          <n-input
            v-model:value="formData.path"
            :placeholder="selectMode === 'folder' ? '例如：C:\\Users\\MyFolder' : '例如：C:\\Program Files\\VSCode\\Code.exe'"
            style="flex: 1"
            @keydown.enter.prevent
          />
          <n-button @click="handleSelectFile">
            <template #icon>
              <n-icon><FolderOpenOutline /></n-icon>
            </template>
            {{ selectMode === 'folder' ? '选择文件夹' : '选择文件' }}
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

      <!-- 图标管理 -->
      <n-form-item label="应用图标">
        <div class="icon-section">
          <div v-if="formData.icon" class="icon-preview">
            <img :src="formData.icon" alt="应用图标" />
          </div>
          <div v-else class="icon-placeholder">
            <n-icon size="48"><ImageOutline /></n-icon>
            <span>暂无图标</span>
          </div>
          <div class="icon-actions">
            <n-button size="small" @click="handleUploadIcon">
              <template #icon>
                <n-icon><CloudUploadOutline /></n-icon>
              </template>
              上传自定义图标
            </n-button>
            <n-button v-if="formData.icon" size="small" text @click="handleRemoveIcon">
              <template #icon>
                <n-icon><TrashOutline /></n-icon>
              </template>
              移除图标
            </n-button>
          </div>
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
import { ref, computed, watch, onMounted } from 'vue';
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
  NRadioGroup,
  NRadio,
  useMessage,
  type FormInst,
  type FormRules,
} from 'naive-ui';
import { FolderOpenOutline, TrashOutline, ImageOutline, CloudUploadOutline } from '@vicons/ionicons5';
import type { AppItem, Category, AppLauncherSettings } from '@/types/appLauncher';
import { ItemType } from '@/types/appLauncher';
import { invoke } from '@tauri-apps/api/core';

interface AppFormData {
  name: string;
  path: string;
  category: string;
  tags: string[];
  launchArgs: string;
  isPinned: boolean;
  icon?: string;
  itemType: ItemType;
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
const launcherSettings = ref<AppLauncherSettings>({ allowedExtensions: ['exe', 'lnk'] });
const selectMode = ref<'file' | 'folder'>('file');

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
  itemType: ItemType.Application,
};

const formData = ref<AppFormData>({ ...defaultFormData });

// 加载启动器设置
const loadLauncherSettings = async () => {
  try {
    const settings = await invoke<AppLauncherSettings>('get_launcher_settings');
    launcherSettings.value = settings;
  } catch (error) {
    console.error('加载启动器设置失败:', error);
  }
};

onMounted(() => {
  loadLauncherSettings();
});

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
        itemType: newApp.itemType || ItemType.Application,
      };
      // 根据item type设置选择模式
      selectMode.value = newApp.itemType === ItemType.Folder ? 'folder' : 'file';
    } else {
      formData.value = { ...defaultFormData };
      selectMode.value = 'file';
    }
  },
  { immediate: true }
);

// 监听对话框显示状态，每次打开时重新加载设置（确保新添加的后缀生效）
watch(
  () => props.show,
  async (visible) => {
    if (visible) {
      await loadLauncherSettings();
    }
  }
);

// 确定item type
const determineItemType = (path: string, isFolder: boolean): ItemType => {
  if (isFolder) {
    return ItemType.Folder;
  }

  const ext = path.split('.').pop()?.toLowerCase() || '';

  if (ext === 'exe') return ItemType.Application;
  if (ext === 'lnk') return ItemType.Shortcut;
  if (ext === 'rdp') return ItemType.RemoteDesktop;
  if (ext === 'url') return ItemType.UrlLink;

  return ItemType.File;
};

// 选择文件或文件夹
const handleSelectFile = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');

    if (selectMode.value === 'folder') {
      // 选择文件夹
      const selected = await open({
        multiple: false,
        directory: true,
        title: '选择文件夹',
      });

      if (selected && typeof selected === 'string') {
        formData.value.path = selected;
        formData.value.itemType = ItemType.Folder;

        // 如果名称为空，从路径提取文件夹名
        if (!formData.value.name) {
          const folderName = selected.split(/[\\\/]/).pop() || '';
          formData.value.name = folderName;
        }
      }
    } else {
      // 选择文件
      const extensions = launcherSettings.value.allowedExtensions;
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: '允许的文件',
            extensions: extensions,
          },
        ],
      });

      if (selected && typeof selected === 'string') {
        formData.value.path = selected;
        formData.value.itemType = determineItemType(selected, false);

        // 如果名称为空，从路径提取文件名
        if (!formData.value.name) {
          const fileName = selected.split(/[\\\/]/).pop()?.replace(/\.\w+$/i, '') || '';
          formData.value.name = fileName;
        }

        // TODO: 调用后端API提取图标
        // const icon = await extractIcon(selected);
        // formData.value.icon = icon;
      }
    }
  } catch (error) {
    message.error(`选择${selectMode.value === 'folder' ? '文件夹' : '文件'}失败`);
    console.error('选择失败:', error);
  }
};

// 移除图标
const handleRemoveIcon = () => {
  formData.value.icon = undefined;
};

// 上传自定义图标
const handleUploadIcon = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      multiple: false,
      directory: false,
      title: '选择图标文件',
      filters: [{
        name: '图片文件',
        extensions: ['png', 'jpg', 'jpeg', 'ico', 'svg']
      }]
    });

    if (selected && typeof selected === 'string') {
      // 读取文件并转换为base64
      const { readFile } = await import('@tauri-apps/plugin-fs');
      const fileData = await readFile(selected);

      // 判断文件类型
      const extension = selected.split('.').pop()?.toLowerCase() || 'png';
      const mimeType = extension === 'svg' ? 'image/svg+xml' :
                      extension === 'ico' ? 'image/x-icon' :
                      `image/${extension}`;

      // 转换为base64
      const base64 = btoa(String.fromCharCode.apply(null, Array.from(fileData)));
      formData.value.icon = `data:${mimeType};base64,${base64}`;

      message.success('图标上传成功');
    }
  } catch (error) {
    message.error('上传图标失败: ' + error);
    console.error('上传图标失败:', error);
  }
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

.icon-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.icon-preview {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 80px;
  height: 80px;
  background: rgba(99, 102, 241, 0.1);
  border-radius: 8px;
  border: 1px solid rgba(99, 102, 241, 0.2);
}

.icon-preview img {
  width: 64px;
  height: 64px;
  object-fit: contain;
}

.icon-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 80px;
  height: 80px;
  background: rgba(71, 85, 105, 0.2);
  border-radius: 8px;
  border: 1px dashed rgba(148, 163, 184, 0.3);
  color: rgba(148, 163, 184, 0.6);
  font-size: 12px;
}

.icon-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
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
