<template>
  <n-modal
    v-model:show="showModal"
    preset="card"
    title="标签管理"
    class="tag-manager-modal"
    style="width: 600px; max-width: 90vw;"
    :segmented="{
      content: 'soft',
      footer: 'soft'
    }"
    to="body"
  >
    <div class="tag-manager">
      <!-- 创建新标签 -->
      <div class="create-section">
        <h3 class="section-title">创建新标签</h3>
        <n-form ref="formRef" :model="formData" :rules="formRules">
          <n-space vertical :size="12">
            <n-form-item label="标签名称" path="name">
              <n-input
                v-model:value="formData.name"
                placeholder="请输入标签名称"
                :maxlength="20"
                show-count
              />
            </n-form-item>
            <n-form-item label="标签颜色" path="color">
              <div class="color-picker-wrapper">
                <n-color-picker
                  v-model:value="formData.color"
                  :modes="['hex']"
                  :show-alpha="false"
                />
                <div class="color-presets">
                  <div
                    v-for="color in COLOR_PRESETS"
                    :key="color"
                    class="color-preset"
                    :class="{ 'active': formData.color === color }"
                    :style="{ background: color }"
                    @click="formData.color = color"
                  />
                </div>
              </div>
            </n-form-item>
            <n-form-item label="预览">
              <div class="tag-preview" :style="{
                background: `${formData.color}30`,
                color: formData.color,
                borderColor: `${formData.color}60`
              }">
                <n-icon size="14">
                  <PricetagOutline />
                </n-icon>
                <span>{{ formData.name || '标签名称' }}</span>
              </div>
            </n-form-item>
          </n-space>
        </n-form>
        <n-space justify="end" style="margin-top: 12px;">
          <n-button @click="resetForm">重置</n-button>
          <n-button
            v-if="!isEditing"
            type="primary"
            @click="handleCreate"
          >
            创建标签
          </n-button>
          <n-button
            v-else
            type="primary"
            @click="handleUpdate"
          >
            保存修改
          </n-button>
        </n-space>
      </div>

      <!-- 已有标签列表 -->
      <div class="tags-list-section">
        <h3 class="section-title">
          已有标签
          <span class="tag-count">{{ tags.length }}</span>
        </h3>

        <div v-if="tags.length > 0" class="tags-list">
          <div
            v-for="tag in tags"
            :key="tag.id"
            class="tag-item"
            :style="{
              background: `${tag.color}20`,
              borderColor: `${tag.color}40`
            }"
          >
            <div class="tag-info">
              <div
                class="tag-color-dot"
                :style="{ background: tag.color }"
              />
              <span class="tag-name" :style="{ color: tag.color }">{{ tag.name }}</span>
            </div>
            <div class="tag-actions">
              <n-button
                text
                size="small"
                @click="handleEdit(tag)"
              >
                <template #icon>
                  <n-icon><CreateOutline /></n-icon>
                </template>
              </n-button>
              <n-button
                text
                size="small"
                type="error"
                @click="handleDelete(tag.id!)"
              >
                <template #icon>
                  <n-icon><TrashOutline /></n-icon>
                </template>
              </n-button>
            </div>
          </div>
        </div>

        <n-empty
          v-else
          description="暂无标签"
          size="small"
          class="empty-state"
        />
      </div>
    </div>

    <template #footer>
      <n-space justify="end">
        <n-button @click="handleClose">关闭</n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import {
  NModal,
  NForm,
  NFormItem,
  NInput,
  NColorPicker,
  NSpace,
  NButton,
  NIcon,
  NEmpty,
  useMessage,
  useDialog,
} from 'naive-ui';
import {
  PricetagOutline,
  CreateOutline,
  TrashOutline,
} from '@vicons/ionicons5';
import type { Tag } from '@/types/task';

const props = defineProps<{
  show: boolean;
  tags: Tag[];
}>();

const emit = defineEmits<{
  'update:show': [value: boolean];
  'create': [tag: Omit<Tag, 'id'>];
  'update': [id: number, tag: Partial<Tag>];
  'delete': [id: number];
}>();

const message = useMessage();
const dialog = useDialog();
const formRef = ref();
const isEditing = ref(false);
const editingTagId = ref<number | null>(null);

const showModal = computed({
  get: () => props.show,
  set: (value) => emit('update:show', value),
});

const formData = reactive({
  name: '',
  color: '#6366f1',
});

const formRules = {
  name: {
    required: true,
    message: '请输入标签名称',
    trigger: 'blur',
  },
};

// 预设颜色
const COLOR_PRESETS = [
  '#f43f5e', // 红色
  '#f59e0b', // 橙色
  '#eab308', // 黄色
  '#10b981', // 绿色
  '#06b6d4', // 青色
  '#6366f1', // 蓝色
  '#8b5cf6', // 紫色
  '#ec4899', // 粉色
  '#64748b', // 灰色
];

const resetForm = () => {
  formData.name = '';
  formData.color = '#6366f1';
  isEditing.value = false;
  editingTagId.value = null;
  formRef.value?.restoreValidation();
};

const handleCreate = async () => {
  try {
    await formRef.value?.validate();

    // 检查名称是否重复
    if (props.tags.some(tag => tag.name === formData.name.trim())) {
      message.error('标签名称已存在');
      return;
    }

    emit('create', {
      name: formData.name.trim(),
      color: formData.color,
    });

    message.success('标签创建成功');
    resetForm();
  } catch (error) {
    console.error('创建标签失败:', error);
  }
};

const handleEdit = (tag: Tag) => {
  isEditing.value = true;
  editingTagId.value = tag.id!;
  formData.name = tag.name;
  formData.color = tag.color;
};

const handleUpdate = async () => {
  try {
    await formRef.value?.validate();

    // 检查名称是否与其他标签重复
    if (props.tags.some(tag => tag.id !== editingTagId.value && tag.name === formData.name.trim())) {
      message.error('标签名称已存在');
      return;
    }

    emit('update', editingTagId.value!, {
      name: formData.name.trim(),
      color: formData.color,
    });

    message.success('标签更新成功');
    resetForm();
  } catch (error) {
    console.error('更新标签失败:', error);
  }
};

const handleDelete = (tagId: number) => {
  dialog.warning({
    title: '删除标签',
    content: '确定要删除此标签吗？删除后，所有使用此标签的任务都会移除该标签。',
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: () => {
      emit('delete', tagId);
      message.success('标签已删除');
    },
  });
};

const handleClose = () => {
  resetForm();
  showModal.value = false;
};

// 监听弹窗关闭，重置表单
watch(() => props.show, (newVal) => {
  if (!newVal) {
    resetForm();
  }
});
</script>

<style scoped>
.tag-manager {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #cbd5e1;
  margin: 0 0 16px 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.tag-count {
  font-size: 11px;
  color: #64748b;
  background: rgba(30, 41, 59, 0.8);
  padding: 3px 10px;
  border-radius: 9999px;
  font-weight: 600;
  border: 1px solid rgba(51, 65, 85, 0.5);
  font-family: 'Consolas', 'Monaco', monospace;
}

/* 创建区域 */
.create-section {
  padding: 20px;
  background: rgba(15, 23, 42, 0.4);
  border-radius: 12px;
  border: 1px solid rgba(51, 65, 85, 0.5);
}

.color-picker-wrapper {
  display: flex;
  align-items: center;
  gap: 16px;
  width: 100%;
}

.color-presets {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  flex: 1;
}

.color-preset {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  border: 2px solid transparent;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.color-preset:hover {
  transform: scale(1.1);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
}

.color-preset.active {
  border-color: #ffffff;
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.4);
}

.tag-preview {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  border: 1px solid;
}

/* 标签列表 */
.tags-list-section {
  display: flex;
  flex-direction: column;
}

.tags-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 300px;
  overflow-y: auto;
  padding-right: 4px;
}

/* 自定义滚动条 */
.tags-list::-webkit-scrollbar {
  width: 6px;
}

.tags-list::-webkit-scrollbar-track {
  background: transparent;
  border-radius: 3px;
}

.tags-list::-webkit-scrollbar-thumb {
  background: rgba(51, 65, 85, 0.5);
  border-radius: 3px;
  transition: background 0.2s;
}

.tags-list::-webkit-scrollbar-thumb:hover {
  background: rgba(71, 85, 105, 0.7);
}

.tag-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-radius: 10px;
  border: 1px solid;
  transition: all 0.2s;
}

.tag-item:hover {
  background: rgba(30, 41, 59, 0.3) !important;
  transform: translateX(4px);
}

.tag-info {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
}

.tag-color-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  flex-shrink: 0;
  box-shadow: 0 0 8px rgba(0, 0, 0, 0.3);
}

.tag-name {
  font-size: 13px;
  font-weight: 600;
}

.tag-actions {
  display: flex;
  gap: 4px;
  opacity: 0.6;
  transition: opacity 0.2s;
}

.tag-item:hover .tag-actions {
  opacity: 1;
}

/* 空状态 */
.empty-state {
  padding: 40px 0;
}

.empty-state:deep(.n-empty__description) {
  color: #94a3b8;
  font-size: 13px;
}
</style>
