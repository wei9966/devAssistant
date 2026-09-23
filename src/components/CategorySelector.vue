<template>
  <div class="category-selector">
    <div class="selector-toolbar">
      <span class="selector-hint">选择任务分类，系统分类可隐藏，自定义分类可编辑</span>
      <div class="toolbar-actions">
        <button v-if="isManaging" type="button" class="create-button" @click="startCreate">
          <n-icon><AddOutline /></n-icon>
          新增分类
        </button>
        <button type="button" class="manage-button" @click="toggleManaging">
          <n-icon><SettingsOutline /></n-icon>
          {{ isManaging ? '完成管理' : '管理分类' }}
        </button>
      </div>
    </div>

    <div v-if="showEditor" class="category-editor" @click.stop>
      <n-input
        v-model:value="editorName"
        maxlength="20"
        show-count
        placeholder="分类名称"
        @keyup.enter="saveCategory"
      />
      <n-color-picker v-model:value="editorColor" :show-alpha="false" :modes="['hex']" />
      <div class="editor-actions">
        <n-button size="small" @click="cancelEditor">取消</n-button>
        <n-button size="small" type="primary" :loading="saving" @click="saveCategory">
          {{ editingCategoryId ? '保存' : '新增' }}
        </n-button>
      </div>
    </div>

    <div class="category-grid" role="radiogroup" aria-label="任务分类">
      <div v-for="category in visibleCategories" :key="category.key" class="category-option">
        <button
          type="button"
          class="category-item"
          :class="{ selected: modelValue === category.key }"
          :style="categoryStyle(category)"
          role="radio"
          :aria-checked="modelValue === category.key"
          @click="handleSelect(category.key)"
        >
          <span class="category-badge"><n-icon :component="resolveIcon(category.icon)" /></span>
          <span class="category-title">{{ category.name }}</span>
          <n-icon v-if="modelValue === category.key" class="selected-check"><CheckmarkCircle /></n-icon>
        </button>

        <div v-if="isManaging && category.id" class="category-actions" @click.stop>
          <button
            v-if="!category.isSystem"
            type="button"
            class="category-action"
            :aria-label="`编辑分类：${category.name}`"
            title="编辑分类"
            @click="startEdit(category)"
          >
            <n-icon><CreateOutline /></n-icon>
          </button>
          <button
            type="button"
            class="category-action"
            :aria-label="`隐藏分类：${category.name}`"
            title="隐藏分类"
            @click="hideCategory(category)"
          >
            <n-icon><EyeOffOutline /></n-icon>
          </button>
          <button
            v-if="!category.isSystem"
            type="button"
            class="category-action danger"
            :aria-label="`删除分类：${category.name}`"
            title="删除分类"
            @click="deleteCategory(category)"
          >
            <n-icon><TrashOutline /></n-icon>
          </button>
        </div>
      </div>
    </div>

    <div v-if="isManaging && hiddenCategories.length" class="hidden-categories">
      <span class="hidden-label">已隐藏</span>
      <button
        v-for="category in hiddenCategories"
        :key="category.key"
        type="button"
        class="restore-button"
        :style="categoryStyle(category)"
        @click="restoreCategory(category)"
      >
        <n-icon><EyeOutline /></n-icon>
        {{ category.name }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { NButton, NColorPicker, NIcon, NInput, useDialog, useMessage } from 'naive-ui';
import {
  AddOutline,
  BulbOutline,
  CheckmarkCircle,
  CodeSlashOutline,
  CreateOutline,
  DocumentTextOutline,
  EllipsisHorizontalOutline,
  EyeOffOutline,
  EyeOutline,
  FolderOutline,
  ServerOutline,
  SettingsOutline,
  TrashOutline,
} from '@vicons/ionicons5';
import { emit as emitTauriEvent } from '@tauri-apps/api/event';
import { useTaskCategoryStore } from '@/stores/taskCategoryStore';
import { CATEGORY_LABELS } from '@/types/task';
import type { TaskCategoryDefinition } from '@/types/task';

const props = withDefaults(defineProps<{ modelValue?: string }>(), { modelValue: 'other' });
const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const categoryStore = useTaskCategoryStore();
const message = useMessage();
const dialog = useDialog();
const isManaging = ref(false);
const showEditor = ref(false);
const editingCategoryId = ref<number | null>(null);
const editorName = ref('');
const editorColor = ref('#6366f1');
const saving = ref(false);

const fallbackCategories = computed<TaskCategoryDefinition[]>(() =>
  Object.entries(CATEGORY_LABELS).map(([key, name], index) => ({
    key,
    name,
    color: '#64748b',
    icon: key,
    isSystem: true,
    isHidden: false,
    sortOrder: (index + 1) * 10,
    usageCount: 0,
  }))
);

const allCategories = computed(() =>
  categoryStore.categories.length ? categoryStore.categories : fallbackCategories.value
);
const visibleCategories = computed(() =>
  allCategories.value.filter(category => !category.isHidden || category.key === props.modelValue)
);
const hiddenCategories = computed(() =>
  allCategories.value.filter(category => category.isHidden && category.key !== props.modelValue)
);

const iconMap = {
  backend: CodeSlashOutline,
  code: CodeSlashOutline,
  database: ServerOutline,
  feature: BulbOutline,
  bulb: BulbOutline,
  docs: DocumentTextOutline,
  document: DocumentTextOutline,
  other: EllipsisHorizontalOutline,
  ellipsis: EllipsisHorizontalOutline,
  folder: FolderOutline,
};

onMounted(async () => {
  try {
    await categoryStore.loadCategories();
  } catch (error) {
    console.error('加载任务分类失败:', error);
    message.warning('自定义分类加载失败，已使用内置分类');
  }
});

function resolveIcon(icon: string) {
  return iconMap[icon as keyof typeof iconMap] || FolderOutline;
}

function categoryStyle(category: TaskCategoryDefinition) {
  return { '--category-color': category.color };
}

async function notifyCategoryChanged() {
  try {
    await emitTauriEvent('task-updated');
  } catch (error) {
    console.error('广播分类更新失败:', error);
  }
}

function handleSelect(category: string) {
  if (!isManaging.value) emit('update:modelValue', category);
}

function toggleManaging() {
  isManaging.value = !isManaging.value;
  if (!isManaging.value) cancelEditor();
}

function startCreate() {
  editingCategoryId.value = null;
  editorName.value = '';
  editorColor.value = '#6366f1';
  showEditor.value = true;
}

function startEdit(category: TaskCategoryDefinition) {
  if (!category.id || category.isSystem) return;
  editingCategoryId.value = category.id;
  editorName.value = category.name;
  editorColor.value = category.color;
  showEditor.value = true;
}

function cancelEditor() {
  showEditor.value = false;
  editingCategoryId.value = null;
  editorName.value = '';
}

async function saveCategory() {
  if (!editorName.value.trim()) {
    message.warning('请输入分类名称');
    return;
  }

  saving.value = true;
  try {
    if (editingCategoryId.value) {
      const category = await categoryStore.updateCategory(
        editingCategoryId.value,
        editorName.value,
        editorColor.value
      );
      if (props.modelValue === category.key) emit('update:modelValue', category.key);
      message.success('分类已更新');
      await notifyCategoryChanged();
    } else {
      const category = await categoryStore.createCategory(editorName.value, editorColor.value);
      emit('update:modelValue', category.key);
      message.success('分类已创建');
      await notifyCategoryChanged();
    }
    cancelEditor();
  } catch (error) {
    console.error('保存分类失败:', error);
    message.error(error instanceof Error ? error.message : String(error));
  } finally {
    saving.value = false;
  }
}

async function hideCategory(category: TaskCategoryDefinition) {
  if (!category.id) return;
  try {
    await categoryStore.setCategoryHidden(category.id, true);
    await notifyCategoryChanged();
    if (props.modelValue === category.key) {
      emit('update:modelValue', categoryStore.visibleCategories[0]?.key || 'other');
    }
  } catch (error) {
    message.error(error instanceof Error ? error.message : String(error));
  }
}

async function restoreCategory(category: TaskCategoryDefinition) {
  if (!category.id) return;
  try {
    await categoryStore.setCategoryHidden(category.id, false);
    await notifyCategoryChanged();
  } catch (error) {
    message.error(error instanceof Error ? error.message : String(error));
  }
}

function deleteCategory(category: TaskCategoryDefinition) {
  if (!category.id || category.isSystem) return;
  dialog.warning({
    title: '删除分类',
    content: category.usageCount
      ? `分类“${category.name}”正在被 ${category.usageCount} 个任务使用。删除后这些任务将归入“其他”，是否继续？`
      : `确定删除分类“${category.name}”吗？`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await categoryStore.deleteCategory(category.id!);
        if (props.modelValue === category.key) emit('update:modelValue', 'other');
        message.success('分类已删除');
        await notifyCategoryChanged();
      } catch (error) {
        message.error(error instanceof Error ? error.message : String(error));
      }
    },
  });
}
</script>

<style scoped>
.category-selector {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}

.selector-toolbar,
.toolbar-actions,
.category-actions,
.editor-actions,
.hidden-categories {
  display: flex;
  align-items: center;
}

.selector-toolbar {
  justify-content: space-between;
  gap: 10px;
  min-height: 30px;
}

.selector-hint,
.hidden-label {
  color: var(--text-muted);
  font-size: 11px;
}

.toolbar-actions,
.editor-actions,
.hidden-categories {
  gap: 7px;
}

.manage-button,
.create-button,
.restore-button,
.category-action {
  border: 0;
  background: transparent;
  color: var(--accent-primary);
  cursor: pointer;
  font: inherit;
}

.manage-button,
.create-button,
.restore-button {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  min-height: 30px;
  padding: 0 8px;
  border-radius: 7px;
  font-size: 11px;
  font-weight: 600;
}

.manage-button:hover,
.create-button:hover,
.restore-button:hover {
  background: var(--bg-hover);
}

.category-editor {
  display: grid;
  grid-template-columns: minmax(140px, 1fr) 132px auto;
  gap: 8px;
  padding: 10px;
  border: 1px solid var(--border-default);
  border-radius: 9px;
  background: var(--bg-elevated);
}

.category-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(126px, 1fr));
  gap: 8px;
}

.category-option {
  position: relative;
  min-width: 0;
}

.category-item {
  width: 100%;
  min-height: 52px;
  padding: 7px 32px 7px 8px;
  border: 1px solid var(--card-border);
  border-radius: 9px;
  background: var(--card-bg);
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: 9px;
  text-align: left;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s, box-shadow 0.2s;
}

.category-item:hover {
  border-color: var(--card-hover-border);
  background: var(--card-hover-bg);
}

.category-item:focus-visible {
  outline: 3px solid color-mix(in srgb, var(--accent-primary) 35%, transparent);
  outline-offset: 2px;
}

.category-item.selected {
  border-color: var(--category-color, var(--accent-primary));
  background: color-mix(in srgb, var(--category-color, var(--accent-primary)) 8%, var(--card-bg));
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--category-color, var(--accent-primary)) 12%, transparent);
}

.category-badge {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  flex: 0 0 34px;
  border-radius: 8px;
  color: var(--category-color, var(--accent-primary));
  background: color-mix(in srgb, var(--category-color, var(--accent-primary)) 12%, var(--bg-elevated));
  font-size: 18px;
}

.category-title {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-size: 12px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.selected-check {
  margin-left: auto;
  color: var(--category-color, var(--accent-primary));
  font-size: 17px;
}

.category-actions {
  position: absolute;
  z-index: 2;
  top: -8px;
  right: -6px;
  gap: 3px;
  padding: 3px;
  border: 1px solid var(--border-default);
  border-radius: 999px;
  background: var(--bg-elevated);
  box-shadow: 0 4px 12px var(--shadow-color);
}

.category-action {
  width: 24px;
  height: 24px;
  display: grid;
  place-items: center;
  border-radius: 50%;
  color: var(--text-secondary);
}

.category-action:hover {
  color: var(--accent-primary);
  background: var(--bg-hover);
}

.category-action.danger:hover {
  color: var(--error);
  background: var(--error-bg);
}

.hidden-categories {
  flex-wrap: wrap;
  padding-top: 8px;
  border-top: 1px dashed var(--border-default);
}

.restore-button {
  color: var(--category-color, var(--text-secondary));
  background: var(--bg-elevated);
}

@media (max-width: 560px) {
  .selector-toolbar {
    align-items: flex-start;
    flex-direction: column;
  }

  .category-editor {
    grid-template-columns: 1fr;
  }

  .editor-actions {
    justify-content: flex-end;
  }
}
</style>
