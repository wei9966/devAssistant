<template>
  <div class="tag-selector">
    <div class="tag-header">
      <n-button
        text
        size="small"
        class="manage-tags-btn"
        @click="emit('manage')"
      >
        <template #icon>
          <n-icon><SettingsOutline /></n-icon>
        </template>
        管理标签
      </n-button>
    </div>

    <div class="tag-list">
      <div
        v-for="tag in availableTags"
        :key="tag.id"
        class="tag-item"
        :class="{ 'selected': isSelected(tag.id!) }"
        :style="{
          '--tag-color': tag.color,
          '--tag-bg': `${tag.color}20`,
          '--tag-border': `${tag.color}40`,
        }"
        @click="toggleTag(tag)"
      >
        <span class="tag-name">{{ tag.name }}</span>
        <n-icon
          v-if="isSelected(tag.id!)"
          size="14"
          class="check-icon"
        >
          <CheckmarkCircle />
        </n-icon>
      </div>
    </div>

    <!-- 已选标签展示 -->
    <div v-if="selectedTags.length > 0" class="selected-tags">
      <div class="selected-tags-header">
        <span class="selected-count">已选 {{ selectedTags.length }} 个标签</span>
        <n-button text size="tiny" @click="clearAll">
          清空
        </n-button>
      </div>
      <div class="selected-tags-list">
        <div
          v-for="tag in selectedTags"
          :key="tag.id"
          class="selected-tag-chip"
          :style="{ background: `${tag.color}30`, color: tag.color, borderColor: `${tag.color}60` }"
        >
          <span>{{ tag.name }}</span>
          <n-icon
            size="12"
            class="remove-icon"
            @click.stop="removeTag(tag.id!)"
          >
            <CloseOutline />
          </n-icon>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <n-empty
      v-if="availableTags.length === 0"
      description="暂无可用标签"
      class="empty-state"
      size="small"
    >
      <template #extra>
        <n-button size="small" @click="emit('manage')">
          创建标签
        </n-button>
      </template>
    </n-empty>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NIcon, NButton, NEmpty } from 'naive-ui';
import {
  PricetagsOutline,
  SettingsOutline,
  CheckmarkCircle,
  CloseOutline,
} from '@vicons/ionicons5';
import type { Tag } from '@/types/task';

const props = defineProps<{
  modelValue: number[];
  availableTags: Tag[];
}>();

const emit = defineEmits<{
  'update:modelValue': [value: number[]];
  'manage': [];
}>();

const selectedTags = computed(() => {
  return props.availableTags.filter(tag => props.modelValue.includes(tag.id!));
});

const isSelected = (tagId: number) => {
  return props.modelValue.includes(tagId);
};

const toggleTag = (tag: Tag) => {
  const tagId = tag.id!;
  const newValue = isSelected(tagId)
    ? props.modelValue.filter(id => id !== tagId)
    : [...props.modelValue, tagId];
  emit('update:modelValue', newValue);
};

const removeTag = (tagId: number) => {
  emit('update:modelValue', props.modelValue.filter(id => id !== tagId));
};

const clearAll = () => {
  emit('update:modelValue', []);
};
</script>

<style scoped>
.tag-selector {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tag-header {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 4px;
}

.manage-tags-btn {
  color: var(--text-muted);
  font-size: 12px;
  padding: 4px 8px;
  transition: color 0.2s;
}

.manage-tags-btn:hover {
  color: var(--accent-primary);
}

/* 标签列表 */
.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tag-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 5px 10px;
  border-radius: 6px;
  background: var(--tag-bg);
  border: 1px solid var(--tag-border);
  color: var(--tag-color);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  user-select: none;
}

.tag-item:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 8px -2px rgba(0, 0, 0, 0.2);
  border-color: var(--tag-color);
}

.tag-item.selected {
  background: var(--tag-color);
  color: #ffffff;
  border-color: var(--tag-color);
  box-shadow: 0 0 0 2px var(--tag-border);
}

.tag-name {
  line-height: 1;
}

.check-icon {
  flex-shrink: 0;
  animation: check-appear 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes check-appear {
  from {
    opacity: 0;
    transform: scale(0.5);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

/* 已选标签 */
.selected-tags {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--border-default);
}

.selected-tags-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.selected-count {
  font-size: 11px;
  color: var(--text-dim);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.selected-tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.selected-tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  border: 1px solid;
  cursor: default;
  transition: all 0.2s;
}

.selected-tag-chip:hover {
  opacity: 0.8;
}

.remove-icon {
  cursor: pointer;
  opacity: 0.7;
  transition: opacity 0.2s;
}

.remove-icon:hover {
  opacity: 1;
}

/* 空状态 */
.empty-state {
  padding: 20px 0;
}

.empty-state:deep(.n-empty__description) {
  color: var(--text-muted);
  font-size: 12px;
}
</style>
