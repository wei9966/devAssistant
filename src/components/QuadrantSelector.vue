<template>
  <div class="quadrant-selector">
    <div class="quadrant-grid">
      <!-- 第一象限：不紧急但重要 -->
      <div
        class="quadrant-item quadrant-not-urgent-important"
        :class="{ 'selected': modelValue === 'not_urgent_important' }"
        @click="handleSelect('not_urgent_important')"
      >
        <div class="quadrant-badge">
          <n-icon size="20">
            <CalendarOutline />
          </n-icon>
        </div>
        <div class="quadrant-content">
          <h4 class="quadrant-title">{{ QUADRANT_CONFIG.not_urgent_important.label }}</h4>
        </div>
        <div v-if="modelValue === 'not_urgent_important'" class="selected-check">
          <n-icon size="20">
            <CheckmarkCircle />
          </n-icon>
        </div>
      </div>

      <!-- 第二象限：紧急且重要 -->
      <div
        class="quadrant-item quadrant-urgent-important"
        :class="{ 'selected': modelValue === 'urgent_important' }"
        @click="handleSelect('urgent_important')"
      >
        <div class="quadrant-badge">
          <n-icon size="20">
            <FlameOutline />
          </n-icon>
        </div>
        <div class="quadrant-content">
          <h4 class="quadrant-title">{{ QUADRANT_CONFIG.urgent_important.label }}</h4>
        </div>
        <div v-if="modelValue === 'urgent_important'" class="selected-check">
          <n-icon size="20">
            <CheckmarkCircle />
          </n-icon>
        </div>
      </div>

      <!-- 第三象限：不紧急不重要 -->
      <div
        class="quadrant-item quadrant-not-urgent-not-important"
        :class="{ 'selected': modelValue === 'not_urgent_not_important' }"
        @click="handleSelect('not_urgent_not_important')"
      >
        <div class="quadrant-badge">
          <n-icon size="20">
            <RemoveCircleOutline />
          </n-icon>
        </div>
        <div class="quadrant-content">
          <h4 class="quadrant-title">{{ QUADRANT_CONFIG.not_urgent_not_important.label }}</h4>
        </div>
        <div v-if="modelValue === 'not_urgent_not_important'" class="selected-check">
          <n-icon size="20">
            <CheckmarkCircle />
          </n-icon>
        </div>
      </div>

      <!-- 第四象限：紧急不重要 -->
      <div
        class="quadrant-item quadrant-urgent-not-important"
        :class="{ 'selected': modelValue === 'urgent_not_important' }"
        @click="handleSelect('urgent_not_important')"
      >
        <div class="quadrant-badge">
          <n-icon size="20">
            <TimeOutline />
          </n-icon>
        </div>
        <div class="quadrant-content">
          <h4 class="quadrant-title">{{ QUADRANT_CONFIG.urgent_not_important.label }}</h4>
        </div>
        <div v-if="modelValue === 'urgent_not_important'" class="selected-check">
          <n-icon size="20">
            <CheckmarkCircle />
          </n-icon>
        </div>
      </div>
    </div>

    <!-- 轴标签 -->
    <div class="axis-labels">
      <div class="axis-label vertical-label">
        <span class="arrow">↑</span>
        <span>重要性</span>
      </div>
      <div class="axis-label horizontal-label">
        <span>紧急性</span>
        <span class="arrow">→</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NIcon, NTooltip } from 'naive-ui';
import {
  GridOutline,
  InformationCircleOutline,
  FlameOutline,
  CalendarOutline,
  TimeOutline,
  RemoveCircleOutline,
  CheckmarkCircle,
} from '@vicons/ionicons5';
import { QUADRANT_CONFIG } from '@/types/task';
import type { TaskQuadrant } from '@/types/task';

const props = withDefaults(
  defineProps<{
    modelValue?: TaskQuadrant;
  }>(),
  {
    modelValue: 'urgent_not_important',
  }
);

const emit = defineEmits<{
  'update:modelValue': [value: TaskQuadrant];
}>();

const handleSelect = (quadrant: TaskQuadrant) => {
  emit('update:modelValue', quadrant);
};
</script>

<style scoped>
/* 四象限网格 */
.quadrant-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
  position: relative;
  padding: 12px 0;
}

/* 中心十字线 - 改为4列布局后隐藏 */
.quadrant-grid::before,
.quadrant-grid::after {
  display: none;
}

/* 象限项 */
.quadrant-item {
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
  z-index: 1;
}

.quadrant-item:hover {
  background: var(--card-hover-bg);
  border-color: var(--card-hover-border);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.quadrant-item.selected {
  background: var(--bg-active);
}

/* 不紧急但重要 - 紫色 */
.quadrant-item.quadrant-not-urgent-important.selected {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent-primary) 40%, transparent), var(--shadow-md);
}

/* 紧急且重要 - 红色 */
.quadrant-item.quadrant-urgent-important.selected {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--error) 40%, transparent), var(--shadow-md);
}

/* 不紧急不重要 - 灰色 */
.quadrant-item.quadrant-not-urgent-not-important.selected {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--text-muted) 40%, transparent), var(--shadow-md);
}

/* 紧急不重要 - 橙色 */
.quadrant-item.quadrant-urgent-not-important.selected {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--warning) 40%, transparent), var(--shadow-md);
}

/* 象限徽章 */
.quadrant-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 10px;
  border: 1px solid;
  flex-shrink: 0;
}

/* 不紧急但重要 - 紫色徽章 */
.quadrant-not-urgent-important .quadrant-badge {
  background: color-mix(in srgb, var(--accent-primary) 15%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary) 30%, transparent);
  color: var(--accent-primary);
}

.quadrant-not-urgent-important .selected-check {
  color: var(--accent-primary);
}

/* 紧急且重要 - 红色徽章 */
.quadrant-urgent-important .quadrant-badge {
  background: color-mix(in srgb, var(--error) 15%, transparent);
  border-color: color-mix(in srgb, var(--error) 30%, transparent);
  color: var(--error);
}

.quadrant-urgent-important .selected-check {
  color: var(--error);
}

/* 不紧急不重要 - 灰色徽章 */
.quadrant-not-urgent-not-important .quadrant-badge {
  background: color-mix(in srgb, var(--text-muted) 15%, transparent);
  border-color: color-mix(in srgb, var(--text-muted) 30%, transparent);
  color: var(--text-muted);
}

.quadrant-not-urgent-not-important .selected-check {
  color: var(--text-muted);
}

/* 紧急不重要 - 橙色徽章 */
.quadrant-urgent-not-important .quadrant-badge {
  background: color-mix(in srgb, var(--warning) 15%, transparent);
  border-color: color-mix(in srgb, var(--warning) 30%, transparent);
  color: var(--warning);
}

.quadrant-urgent-not-important .selected-check {
  color: var(--warning);
}

/* 象限内容 */
.quadrant-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.quadrant-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  line-height: 1.4;
}

/* 选中标记 */
.selected-check {
  position: absolute;
  top: 8px;
  right: 8px;
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

/* 轴标签 */
.axis-labels {
  position: relative;
  pointer-events: none;
}

.axis-label {
  position: absolute;
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.vertical-label {
  left: -60px;
  top: -30px;
  flex-direction: column;
  writing-mode: vertical-rl;
  text-orientation: mixed;
}

.horizontal-label {
  bottom: -30px;
  right: 0;
}

.arrow {
  font-size: 14px;
  color: var(--accent-primary);
}

/* 响应式 */
@media (max-width: 768px) {
  .quadrant-grid {
    gap: 8px;
  }

  .quadrant-item {
    padding: 12px;
    min-height: 100px;
  }

  .quadrant-badge {
    width: 36px;
    height: 36px;
  }

  .quadrant-title {
    font-size: 12px;
  }

  .quadrant-desc {
    font-size: 10px;
  }

  .axis-labels {
    display: none;
  }
}
</style>
