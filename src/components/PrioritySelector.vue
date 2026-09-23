<template>
  <div class="priority-selector">
    <div class="priority-grid" role="radiogroup" aria-label="任务优先级">
      <button
        v-for="item in priorities"
        :key="item.value"
        type="button"
        class="priority-item"
        :class="[`priority-${item.name}`, { selected: modelValue === item.value }]"
        role="radio"
        :aria-checked="modelValue === item.value"
        @click="emit('update:modelValue', item.value)"
      >
        <span class="priority-dot"></span>{{ item.label }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ modelValue?: number }>(), { modelValue: 2 });
const emit = defineEmits<{ 'update:modelValue': [value: number] }>();
const priorities = [
  { value: 1, label: '高', name: 'high' },
  { value: 2, label: '中', name: 'medium' },
  { value: 3, label: '低', name: 'low' },
];
</script>

<style scoped>
.priority-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:7px}.priority-item{min-height:44px;padding:0 10px;border-radius:8px;border:1px solid var(--card-border);background:var(--card-bg);color:var(--text-secondary);display:flex;align-items:center;justify-content:center;gap:7px;font-size:12px;font-weight:600;cursor:pointer;transition:border-color .2s,background .2s,box-shadow .2s}.priority-item:hover{border-color:var(--card-hover-border);background:var(--card-hover-bg)}.priority-item:focus-visible{outline:3px solid color-mix(in srgb,var(--accent-primary) 35%,transparent);outline-offset:2px}.priority-item.selected{background:var(--bg-active);font-weight:700}.priority-dot{width:7px;height:7px;border-radius:50%;background:var(--text-muted)}.priority-high .priority-dot{background:var(--error)}.priority-medium .priority-dot{background:var(--warning)}.priority-low .priority-dot{background:var(--success)}.priority-high.selected{border-color:var(--error);color:var(--error)}.priority-medium.selected{border-color:var(--warning);color:var(--warning)}.priority-low.selected{border-color:var(--success);color:var(--success)}
</style>
