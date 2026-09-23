<template>
  <div class="quadrant-selector">
    <div class="quadrant-grid" role="radiogroup" aria-label="任务四象限">
      <button
        v-for="item in quadrants"
        :key="item.value"
        type="button"
        class="quadrant-item"
        :class="[`quadrant-${item.value}`, { selected: modelValue === item.value }]"
        role="radio"
        :aria-checked="modelValue === item.value"
        @click="emit('update:modelValue', item.value)"
      >
        <span class="quadrant-badge"><n-icon :component="item.icon" /></span>
        <span class="quadrant-content">
          <strong>{{ item.label }}</strong>
          <small>{{ item.description }}</small>
        </span>
        <n-icon v-if="modelValue === item.value" class="selected-check"><CheckmarkCircle /></n-icon>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NIcon } from 'naive-ui';
import { CalendarOutline, CheckmarkCircle, FlameOutline, RemoveCircleOutline, TimeOutline } from '@vicons/ionicons5';
import { QUADRANT_CONFIG } from '@/types/task';
import type { TaskQuadrant } from '@/types/task';

withDefaults(defineProps<{ modelValue?: TaskQuadrant }>(), { modelValue: 'urgent_not_important' });
const emit = defineEmits<{ 'update:modelValue': [value: TaskQuadrant] }>();

const quadrants: Array<{ value: TaskQuadrant; label: string; description: string; icon: typeof FlameOutline }> = [
  { value: 'urgent_important', label: QUADRANT_CONFIG.urgent_important.label, description: '立即处理', icon: FlameOutline },
  { value: 'not_urgent_important', label: QUADRANT_CONFIG.not_urgent_important.label, description: '计划执行', icon: CalendarOutline },
  { value: 'urgent_not_important', label: QUADRANT_CONFIG.urgent_not_important.label, description: '尽快处理', icon: TimeOutline },
  { value: 'not_urgent_not_important', label: QUADRANT_CONFIG.not_urgent_not_important.label, description: '稍后考虑', icon: RemoveCircleOutline },
];
</script>

<style scoped>
.quadrant-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:8px}.quadrant-item{position:relative;min-height:54px;padding:7px 34px 7px 8px;border:1px solid var(--card-border);border-radius:9px;background:var(--card-bg);color:var(--text-primary);display:flex;align-items:center;gap:9px;text-align:left;cursor:pointer;transition:border-color .2s,background .2s,box-shadow .2s}.quadrant-item:hover{border-color:var(--card-hover-border);background:var(--card-hover-bg)}.quadrant-item:focus-visible{outline:3px solid color-mix(in srgb,var(--accent-primary) 35%,transparent);outline-offset:2px}.quadrant-item.selected{background:var(--bg-active);border-color:var(--accent-primary);box-shadow:0 0 0 2px color-mix(in srgb,var(--accent-primary) 11%,transparent)}.quadrant-badge{width:34px;height:34px;display:grid;place-items:center;flex:0 0 34px;border-radius:8px;font-size:18px;background:var(--bg-elevated);color:var(--text-muted)}.quadrant-content{display:flex;min-width:0;flex-direction:column;gap:2px}.quadrant-content strong{font-size:12px;line-height:1.35}.quadrant-content small{font-size:10px;line-height:1.25;color:var(--text-muted)}.selected-check{position:absolute;right:9px;top:50%;transform:translateY(-50%);font-size:17px;color:var(--accent-primary)}.quadrant-urgent_important.selected{border-color:var(--error)}.quadrant-urgent_important .quadrant-badge{color:var(--error);background:color-mix(in srgb,var(--error) 13%,transparent)}.quadrant-not_urgent_important .quadrant-badge{color:var(--accent-primary);background:color-mix(in srgb,var(--accent-primary) 13%,transparent)}.quadrant-urgent_not_important .quadrant-badge{color:var(--warning);background:color-mix(in srgb,var(--warning) 13%,transparent)}
@media(max-width:560px){.quadrant-grid{grid-template-columns:1fr}}
</style>
