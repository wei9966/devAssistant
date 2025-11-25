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
          <n-icon size="18" color="#f43f5e">
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
          <n-icon size="18" color="#f59e0b">
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
          <n-icon size="18" color="#10b981">
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
  background: rgba(30, 41, 59, 0.4);
  border: 2px solid rgba(51, 65, 85, 0.5);
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 50px;
}

.priority-item:hover {
  background: rgba(30, 41, 59, 0.7);
  border-color: rgba(71, 85, 105, 0.7);
  transform: translateY(-2px);
  box-shadow: 0 6px 12px -3px rgba(0, 0, 0, 0.3);
}

.priority-item.selected {
  background: rgba(30, 41, 59, 0.8);
}

.priority-item.priority-high.selected {
  box-shadow: 0 0 0 2px rgba(244, 63, 94, 0.4), 0 6px 12px -3px rgba(0, 0, 0, 0.3);
}

.priority-item.priority-medium.selected {
  box-shadow: 0 0 0 2px rgba(245, 158, 11, 0.4), 0 6px 12px -3px rgba(0, 0, 0, 0.3);
}

.priority-item.priority-low.selected {
  box-shadow: 0 0 0 2px rgba(16, 185, 129, 0.4), 0 6px 12px -3px rgba(0, 0, 0, 0.3);
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
  background: rgba(244, 63, 94, 0.15);
  color: #fca5a5;
}

.priority-medium .priority-badge {
  background: rgba(245, 158, 11, 0.15);
  color: #fcd34d;
}

.priority-low .priority-badge {
  background: rgba(16, 185, 129, 0.15);
  color: #6ee7b7;
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
  background: #f43f5e;
  box-shadow: 0 0 6px rgba(244, 63, 94, 0.6);
}

.priority-medium .priority-badge .priority-dot {
  background: #f59e0b;
  box-shadow: 0 0 6px rgba(245, 158, 11, 0.6);
}

.priority-low .priority-badge .priority-dot {
  background: #10b981;
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.6);
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
  color: #e2e8f0;
  margin: 0;
  line-height: 1.4;
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
