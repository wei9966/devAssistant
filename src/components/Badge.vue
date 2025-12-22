<template>
  <span :class="['badge', badgeClass]">
    <slot></slot>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    type?: 'high' | 'medium' | 'low' | 'Backend' | 'Database' | 'Feature' | 'Docs' | 'default';
  }>(),
  {
    type: 'default'
  }
);

const badgeClass = computed(() => {
  const classMap = {
    // 优先级徽章
    high: 'badge-high',
    medium: 'badge-medium',
    low: 'badge-low',
    // 分类徽章
    Backend: 'badge-backend',
    Database: 'badge-database',
    Feature: 'badge-feature',
    Docs: 'badge-docs',
    // 默认
    default: 'badge-default'
  };
  return classMap[props.type] || classMap.default;
});
</script>

<style scoped>
.badge {
  display: inline-block;
  font-size: 10px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid;
  text-transform: uppercase;
  letter-spacing: 0.025em;
}

/* 优先级徽章 */
.badge-high {
  background: rgba(244, 63, 94, 0.1);
  color: rgb(251, 113, 133);
  border-color: rgba(244, 63, 94, 0.2);
}

.badge-medium {
  background: rgba(245, 158, 11, 0.1);
  color: rgb(251, 191, 36);
  border-color: rgba(245, 158, 11, 0.2);
}

.badge-low {
  background: rgba(16, 185, 129, 0.1);
  color: rgb(52, 211, 153);
  border-color: rgba(16, 185, 129, 0.2);
}

/* 分类徽章 */
.badge-backend {
  background: rgba(59, 130, 246, 0.1);
  color: rgb(96, 165, 250);
  border-color: rgba(59, 130, 246, 0.2);
}

.badge-database {
  background: rgba(168, 85, 247, 0.1);
  color: rgb(192, 132, 252);
  border-color: rgba(168, 85, 247, 0.2);
}

.badge-feature {
  background: rgba(236, 72, 153, 0.1);
  color: rgb(244, 114, 182);
  border-color: rgba(236, 72, 153, 0.2);
}

.badge-docs {
  background: var(--bg-surface);
  color: var(--text-secondary);
  border-color: var(--border-default);
}

/* 默认 */
.badge-default {
  background: var(--bg-overlay);
  color: var(--text-muted);
  border-color: var(--border-default);
}
</style>
