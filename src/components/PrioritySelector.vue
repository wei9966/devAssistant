<template>
  <div class="priority-selector">
    <div class="priority-grid">
      <!-- 高优先级 -->
      <div
        class="priority-item priority-high"
        :class="{ 'selected': modelValue === 1 }"
        @click="handleSelect(1)"
      >
        <div class="priority-badge">
          <div class="priority-dot"></div>
          <n-icon size="20">
            <ArrowUpOutline />
          </n-icon>
        </div>
        <div class="priority-content">
          <h4 class="priority-title">高优先级</h4>
        </div>
        <div v-if="modelValue === 1" class="selected-check">
          <n-icon size="18">
            <CheckmarkCircle />
          </n-icon>
        </div>
      </div>

      <!-- 中优先级 -->
      <div
        class="priority-item priority-medium"
        :class="{ 'selected': modelValue === 2 }"
        @click="handleSelect(2)"
      >
        <div class="priority-badge">
          <div class="priority-dot"></div>
          <n-icon size="20">
            <RemoveOutline />
          </n-icon>
        </div>
        <div class="priority-content">
          <h4 class="priority-title">中优先级</h4>
        </div>
        <div v-if="modelValue === 2" class="selected-check">
          <n-icon size="18">
            <CheckmarkCircle />
          </n-icon>
        </div>
      </div>

      <!-- 低优先级 -->
      <div
        class="priority-item priority-low"
        :class="{ 'selected': modelValue === 3 }"
        @click="handleSelect(3)"
      >
        <div class="priority-badge">
          <div class="priority-dot"></div>
          <n-icon size="20">
            <ArrowDownOutline />
          </n-icon>
        </div>
        <div class="priority-content">
          <h4 class="priority-title">低优先级</h4>
        </div>
        <div v-if="modelValue === 3" class="selected-check">
          <n-icon size="18">
            <CheckmarkCircle />
          </n-icon>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NIcon } from 'naive-ui';
import {
  FlagOutline,
  ArrowUpOutline,
  RemoveOutline,
  ArrowDownOutline,
  CheckmarkCircle,
} from '@vicons/ionicons5';

const props = withDefaults(
  defineProps<{
    modelValue?: number;
  }>(),
  {
    modelValue: 2,
  }
);

const emit = defineEmits<{
  'update:modelValue': [value: number];
}>();

const handleSelect = (priority: number) => {
  emit('update:modelValue', priority);
};
</script>

<style scoped>
/* 优先级网格 */
.priority-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

/* 优先级项 */
.priority-item {
  position: relative;
  padding: 8px;
  border-radius: 8px;
  background: var(--card-bg);
  border: 2px solid var(--card-border);
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 50px;
}

.priority-item:hover {
  background: var(--card-hover-bg);
  border-color: var(--card-hover-border);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.priority-item.selected {
  background: var(--bg-active);
}

.priority-item.priority-high.selected {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--error) 40%, transparent), var(--shadow-md);
}

.priority-item.priority-medium.selected {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--warning) 40%, transparent), var(--shadow-md);
}

.priority-item.priority-low.selected {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--success) 40%, transparent), var(--shadow-md);
}

/* 优先级徽章 */
.priority-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  flex-shrink: 0;
  position: relative;
}

.priority-high .priority-badge {
  background: color-mix(in srgb, var(--error) 15%, transparent);
  color: var(--error);
}

.priority-medium .priority-badge {
  background: color-mix(in srgb, var(--warning) 15%, transparent);
  color: var(--warning);
}

.priority-low .priority-badge {
  background: color-mix(in srgb, var(--success) 15%, transparent);
  color: var(--success);
}

.priority-badge .priority-dot {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.priority-high .priority-badge .priority-dot {
  background: var(--error);
  box-shadow: 0 0 6px color-mix(in srgb, var(--error) 60%, transparent);
}

.priority-medium .priority-badge .priority-dot {
  background: var(--warning);
  box-shadow: 0 0 6px color-mix(in srgb, var(--warning) 60%, transparent);
}

.priority-low .priority-badge .priority-dot {
  background: var(--success);
  box-shadow: 0 0 6px color-mix(in srgb, var(--success) 60%, transparent);
}

/* 优先级内容 */
.priority-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.priority-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  line-height: 1.4;
}

/* 选中图标颜色 */
.priority-high .selected-check {
  color: var(--error);
}

.priority-medium .selected-check {
  color: var(--warning);
}

.priority-low .selected-check {
  color: var(--success);
}

/* 选中标记 */
.selected-check {
  position: absolute;
  top: 6px;
  right: 6px;
  animation: checkmark-appear 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes checkmark-appear {
  from {
    opacity: 0;
    transform: scale(0.5);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

/* 响应式 */
@media (max-width: 768px) {
  .priority-grid {
    grid-template-columns: 1fr;
  }

  .priority-item {
    min-height: 80px;
  }
}
</style>
