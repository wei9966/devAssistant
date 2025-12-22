<template>
  <n-modal
    v-model:show="dialogVisible"
    preset="card"
    title="分类管理"
    class="category-manager"
    style="width: 700px"
    :mask-closable="false"
  >
    <div class="manager-content">
      <!-- 添加新分类 -->
      <div class="add-section">
        <n-form inline :model="newCategory">
          <n-form-item label="分类名称">
            <n-input
              v-model:value="newCategory.name"
              placeholder="例如：开发工具"
              style="width: 150px"
              @keydown.enter.prevent="handleAddCategory"
            />
          </n-form-item>
          <n-form-item label="图标">
            <n-input
              v-model:value="newCategory.icon"
              placeholder="📁"
              style="width: 80px"
              @keydown.enter.prevent="handleAddCategory"
            />
          </n-form-item>
          <n-form-item label="颜色">
            <n-color-picker
              v-model:value="newCategory.color"
              :modes="['hex']"
              style="width: 100px"
            />
          </n-form-item>
          <n-form-item>
            <n-button type="primary" @click="handleAddCategory">
              <template #icon>
                <n-icon><AddOutline /></n-icon>
              </template>
              添加
            </n-button>
          </n-form-item>
        </n-form>
      </div>

      <!-- 分类列表 -->
      <div class="category-list">
        <n-data-table
          :columns="columns"
          :data="editableCategories"
          :bordered="false"
          :single-line="false"
          striped
        />
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <n-button @click="handleCancel">取消</n-button>
        <n-button type="primary" @click="handleSave">保存</n-button>
      </div>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, h } from 'vue';
import {
  NModal,
  NForm,
  NFormItem,
  NInput,
  NColorPicker,
  NButton,
  NIcon,
  NDataTable,
  NSpace,
  useMessage,
  type DataTableColumns,
} from 'naive-ui';
import { AddOutline, CreateOutline, TrashOutline } from '@vicons/ionicons5';
import type { Category } from '@/types/appLauncher';

const props = withDefaults(
  defineProps<{
    show: boolean;
    categories: Category[];
  }>(),
  {
    show: false,
  }
);

const emit = defineEmits<{
  'update:show': [value: boolean];
  save: [categories: Category[]];
  cancel: [];
}>();

const message = useMessage();

const dialogVisible = computed({
  get: () => props.show,
  set: (val) => emit('update:show', val),
});

// 可编辑的分类列表（包含所有分类）
const editableCategories = ref<Category[]>([]);

// 默认分类ID集合（用于标记默认分类）
const defaultCategoryIds = new Set(['all', 'dev', 'office', 'browser', 'design', 'media', 'game', 'other']);

// 新分类表单
const newCategory = ref({
  name: '',
  icon: '',
  color: '#6366f1',
});

// 监听props变化，初始化可编辑列表
const initCategories = () => {
  // 包含所有分类，包括默认分类
  console.log('初始化分类列表，分类数量:', props.categories.length);
  editableCategories.value = props.categories.map((cat) => ({ ...cat }));
  console.log('可编辑分类列表:', editableCategories.value);
};

// 表格列定义
const columns: DataTableColumns<Category> = [
  {
    title: '图标',
    key: 'icon',
    width: 80,
    render: (row) => {
      return h('div', { class: 'category-icon' }, row.icon || '📁');
    },
  },
  {
    title: '分类名称',
    key: 'name',
    render: (row) => {
      const isDefault = defaultCategoryIds.has(row.id);
      return h('div', { style: 'display: flex; align-items: center; gap: 8px;' }, [
        h(NInput, {
          value: row.name,
          onUpdateValue: (val: string) => {
            row.name = val;
          },
        }),
        isDefault ? h('span', {
          style: 'font-size: 11px; color: #94a3b8; white-space: nowrap;'
        }, '(默认)') : null,
      ]);
    },
  },
  {
    title: '图标emoji',
    key: 'icon',
    width: 120,
    render: (row) => {
      return h(NInput, {
        value: row.icon,
        placeholder: '📁',
        onUpdateValue: (val: string) => {
          row.icon = val;
        },
      });
    },
  },
  {
    title: '颜色',
    key: 'color',
    width: 120,
    render: (row) => {
      return h(NColorPicker, {
        value: row.color,
        modes: ['hex'],
        onUpdateValue: (val: string) => {
          row.color = val;
        },
      });
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 100,
    render: (row) => {
      return h(
        NSpace,
        {},
        {
          default: () => [
            h(
              NButton,
              {
                text: true,
                type: 'error',
                onClick: () => handleDeleteCategory(row.id),
              },
              {
                icon: () => h(NIcon, {}, { default: () => h(TrashOutline) }),
              }
            ),
          ],
        }
      );
    },
  },
];

// 添加新分类
const handleAddCategory = () => {
  if (!newCategory.value.name.trim()) {
    message.warning('请输入分类名称');
    return;
  }

  // 计算新分类的 sortOrder（最大值+1）
  const maxSortOrder = Math.max(...editableCategories.value.map(cat => cat.sortOrder), 0);

  const newCat: Category = {
    id: `cat_${Date.now()}`,
    name: newCategory.value.name.trim(),
    icon: newCategory.value.icon || '📁',
    color: newCategory.value.color,
    sortOrder: maxSortOrder + 1,
    createdAt: Date.now(),
  };

  editableCategories.value.push(newCat);

  // 重置表单
  newCategory.value = {
    name: '',
    icon: '',
    color: '#6366f1',
  };

  message.success('分类已添加');
};

// 删除分类
const handleDeleteCategory = (categoryId: string) => {
  const index = editableCategories.value.findIndex((cat) => cat.id === categoryId);
  if (index !== -1) {
    editableCategories.value.splice(index, 1);
    message.success('分类已删除');
  }
};

// 保存
const handleSave = () => {
  // 直接保存所有编辑后的分类（包括默认分类）
  emit('save', editableCategories.value);
  dialogVisible.value = false;
};

// 取消
const handleCancel = () => {
  dialogVisible.value = false;
  emit('cancel');
};

// 监听对话框显示状态，当显示时初始化分类列表
watch(() => props.show, (newValue) => {
  if (newValue) {
    console.log('分类管理对话框打开，接收到的分类:', props.categories);
    initCategories();
  }
}, { immediate: true });
</script>

<style scoped>
.category-manager {
  background: var(--bg-overlay);
  backdrop-filter: blur(16px);
}

.category-manager :deep(.n-card) {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
}

.category-manager :deep(.n-card__header) {
  border-bottom: 1px solid var(--border-default);
  color: var(--text-primary);
  font-weight: 600;
}

.manager-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.add-section {
  padding: 16px;
  background: var(--bg-overlay);
  border-radius: 8px;
  border: 1px solid var(--border-default);
}

.category-list {
  max-height: 400px;
  overflow-y: auto;
}

.category-icon {
  font-size: 24px;
  text-align: center;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

/* 表单样式覆盖 */
.category-manager :deep(.n-form-item-label) {
  color: var(--text-secondary);
}

.category-manager :deep(.n-input) {
  background: var(--input-bg);
  border-color: var(--input-border);
}

.category-manager :deep(.n-input:hover) {
  border-color: var(--border-hover);
}

.category-manager :deep(.n-input:focus) {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 2px var(--accent-glow);
}

/* 表格样式 */
.category-manager :deep(.n-data-table) {
  background: transparent;
}

.category-manager :deep(.n-data-table-th) {
  background: var(--bg-overlay);
  color: var(--text-secondary);
  border-color: var(--border-default);
}

.category-manager :deep(.n-data-table-td) {
  background: transparent;
  border-color: var(--border-default);
  color: var(--text-primary);
}

.category-manager :deep(.n-data-table-tr:hover .n-data-table-td) {
  background: var(--bg-hover);
}

/* 滚动条样式 */
.category-list::-webkit-scrollbar {
  width: 8px;
}

.category-list::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
  border-radius: 4px;
}

.category-list::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 4px;
}

.category-list::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
}
</style>
