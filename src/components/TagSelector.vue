<template>
  <div class="tag-selector">
    <div class="tag-toolbar">
      <span class="selection-summary">{{ modelValue.length ? `已选 ${modelValue.length} 个` : '可多选' }}</span>
      <n-button text size="small" class="manage-tags-btn" @click="emit('manage')">
        <template #icon><n-icon><SettingsOutline /></n-icon></template>管理标签
      </n-button>
    </div>

    <div v-if="displayTags.length" class="tag-list" aria-label="任务标签">
      <button
        v-for="tag in displayTags"
        :key="tag.id"
        type="button"
        class="tag-item"
        :class="{ selected: isSelected(tag.id!) }"
        :style="{ '--tag-color': tag.color, '--tag-bg': `${tag.color}18`, '--tag-border': `${tag.color}55` }"
        :aria-pressed="isSelected(tag.id!)"
        @click="toggleTag(tag)"
      >
        <span class="tag-dot"></span><span>{{ tag.name }}</span>
        <n-icon v-if="isSelected(tag.id!)"><CheckmarkCircle /></n-icon>
      </button>
      <button v-if="hasMoreTags" type="button" class="expand-button" @click="showAllTags = !showAllTags">
        {{ showAllTags ? '收起' : `+${availableTags.length - compactTags.length} 更多` }}
      </button>
      <button v-if="modelValue.length" type="button" class="clear-button" @click="emit('update:modelValue', [])">清空</button>
    </div>

    <div v-else class="empty-inline">
      <span>暂无标签</span><n-button text size="small" @click="emit('manage')">＋ 新建标签</n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { NButton, NIcon } from 'naive-ui';
import { CheckmarkCircle, SettingsOutline } from '@vicons/ionicons5';
import type { Tag } from '@/types/task';

const props = defineProps<{ modelValue: number[]; availableTags: Tag[] }>();
const emit = defineEmits<{ 'update:modelValue': [value: number[]]; 'manage': [] }>();
const showAllTags = ref(false);

const compactTags = computed(() => {
  const selected = props.availableTags.filter(tag => props.modelValue.includes(tag.id!));
  const favorites = props.availableTags.filter(tag => tag.isFavorite && !props.modelValue.includes(tag.id!));
  const others = props.availableTags.filter(tag => !selected.includes(tag) && !favorites.includes(tag));
  return [...selected, ...favorites, ...others].slice(0, 6);
});
const displayTags = computed(() => showAllTags.value ? props.availableTags : compactTags.value);
const hasMoreTags = computed(() => props.availableTags.length > compactTags.value.length);
const isSelected = (id: number) => props.modelValue.includes(id);
function toggleTag(tag: Tag) {
  const id = tag.id!;
  emit('update:modelValue', isSelected(id) ? props.modelValue.filter(value => value !== id) : [...props.modelValue, id]);
}
</script>

<style scoped>
.tag-selector{display:flex;flex-direction:column;gap:9px}.tag-toolbar{display:flex;align-items:center;justify-content:space-between;min-height:28px}.selection-summary{font-size:12px;color:var(--text-muted)}.manage-tags-btn{color:var(--accent-primary);font-size:12px;font-weight:600}.tag-list{display:flex;flex-wrap:wrap;gap:7px}.tag-item,.expand-button,.clear-button{min-height:36px;border-radius:8px;border:1px solid var(--tag-border,var(--border-default));background:var(--tag-bg,var(--card-bg));color:var(--tag-color,var(--text-secondary));padding:0 10px;display:inline-flex;align-items:center;gap:6px;font-size:12px;font-weight:600;cursor:pointer;transition:border-color .2s,background .2s,box-shadow .2s}.tag-item:hover{border-color:var(--tag-color)}.tag-item:focus-visible,.expand-button:focus-visible,.clear-button:focus-visible{outline:3px solid color-mix(in srgb,var(--accent-primary) 35%,transparent);outline-offset:2px}.tag-item.selected{background:var(--tag-color);border-color:var(--tag-color);color:#fff;box-shadow:0 0 0 2px var(--tag-border)}.tag-dot{width:7px;height:7px;border-radius:50%;background:currentColor}.expand-button{border-style:dashed;color:var(--accent-primary);background:transparent}.clear-button{border-color:transparent;background:transparent;color:var(--text-muted)}.empty-inline{min-height:44px;border:1px dashed var(--border-default);border-radius:8px;display:flex;align-items:center;justify-content:center;gap:8px;color:var(--text-muted);font-size:12px}
</style>
