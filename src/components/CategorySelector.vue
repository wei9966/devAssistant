<template>
  <div class="category-selector">
    <div class="category-grid">
      <!-- 后端开发 -->
      <div
        class="category-item category-backend"
        :class="{ 'selected': modelValue === 'backend' }"
        @click="handleSelect('backend')"
      >
        <div class="category-badge">
          <n-icon size="20">
            <CodeSlashOutline />
          </n-icon>
        </div>
        <div class="category-content">
          <h4 class="category-title">后端开发</h4>
        </div>
        <div v-if="modelValue === 'backend'" class="selected-check">
          <n-icon size="18">
            <CheckmarkCircle />
          </n-icon>
        </div>
      </div>

      <!-- 数据库 -->
      <div
        class="category-item category-database"
        :class="{ 'selected': modelValue === 'database' }"
        @click="handleSelect('database')"
      >
        <div class="category-badge">
          <n-icon size="20">
            <ServerOutline />
          </n-icon>
        </div>
        <div class="category-content">
          <h4 class="category-title">数据库</h4>
        </div>
        <div v-if="modelValue === 'database'" class="selected-check">
          <n-icon size="18">
            <CheckmarkCircle />
          </n-icon>
        </div>
      </div>

      <!-- 功能开发 -->
      <div
        class="category-item category-feature"
        :class="{ 'selected': modelValue === 'feature' }"
        @click="handleSelect('feature')"
      >
        <div class="category-badge">
          <n-icon size="20">
            <BulbOutline />
          </n-icon>
        </div>
        <div class="category-content">
          <h4 class="category-title">功能开发</h4>
        </div>
        <div v-if="modelValue === 'feature'" class="selected-check">
          <n-icon size="18">
            <CheckmarkCircle />
          </n-icon>
        </div>
      </div>

      <!-- 文档 -->
      <div
        class="category-item category-docs"
        :class="{ 'selected': modelValue === 'docs' }"
        @click="handleSelect('docs')"
      >
        <div class="category-badge">
          <n-icon size="20">
            <DocumentTextOutline />
          </n-icon>
        </div>
        <div class="category-content">
          <h4 class="category-title">文档</h4>
        </div>
        <div v-if="modelValue === 'docs'" class="selected-check">
          <n-icon size="18">
            <CheckmarkCircle />
          </n-icon>
        </div>
      </div>

      <!-- 其他 -->
      <div
        class="category-item category-other"
        :class="{ 'selected': modelValue === 'other' }"
        @click="handleSelect('other')"
      >
        <div class="category-badge">
          <n-icon size="20">
            <EllipsisHorizontalOutline />
          </n-icon>
        </div>
        <div class="category-content">
          <h4 class="category-title">其他</h4>
        </div>
        <div v-if="modelValue === 'other'" class="selected-check">
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
  AppsOutline,
  CodeSlashOutline,
  ServerOutline,
  BulbOutline,
  DocumentTextOutline,
  EllipsisHorizontalOutline,
  CheckmarkCircle,
} from '@vicons/ionicons5';

const props = withDefaults(
  defineProps<{
    modelValue?: string;
  }>(),
  {
    modelValue: 'other',
  }
);

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const handleSelect = (category: string) => {
  emit('update:modelValue', category);
};
</script>

<style scoped>
/* 分类网格 */
.category-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 8px;
}

/* 分类项 */
.category-item {
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

.category-item:hover {
  background: var(--card-hover-bg);
  border-color: var(--card-hover-border);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.category-item.selected {
  background: var(--bg-active);
}

.category-item.category-backend.selected {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--success) 40%, transparent), var(--shadow-md);
}

.category-item.category-database.selected {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--info) 40%, transparent), var(--shadow-md);
}

.category-item.category-feature.selected {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--warning) 40%, transparent), var(--shadow-md);
}

.category-item.category-docs.selected {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent-secondary) 40%, transparent), var(--shadow-md);
}

.category-item.category-other.selected {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--text-muted) 40%, transparent), var(--shadow-md);
}

/* 分类徽章 */
.category-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  flex-shrink: 0;
}

.category-backend .category-badge {
  background: color-mix(in srgb, var(--success) 15%, transparent);
  color: var(--success);
}

.category-database .category-badge {
  background: color-mix(in srgb, var(--info) 15%, transparent);
  color: var(--info);
}

.category-feature .category-badge {
  background: color-mix(in srgb, var(--warning) 15%, transparent);
  color: var(--warning);
}

.category-docs .category-badge {
  background: color-mix(in srgb, var(--accent-secondary) 15%, transparent);
  color: var(--accent-secondary);
}

.category-other .category-badge {
  background: color-mix(in srgb, var(--text-muted) 15%, transparent);
  color: var(--text-muted);
}

/* 选中图标颜色 */
.category-backend .selected-check {
  color: var(--success);
}

.category-database .selected-check {
  color: var(--info);
}

.category-feature .selected-check {
  color: var(--warning);
}

.category-docs .selected-check {
  color: var(--accent-secondary);
}

.category-other .selected-check {
  color: var(--text-muted);
}

/* 分类内容 */
.category-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.category-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
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
  .category-grid {
    grid-template-columns: 1fr;
  }

  .category-item {
    min-height: 80px;
  }
}
</style>
