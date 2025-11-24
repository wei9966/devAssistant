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
          <n-icon size="18" color="#10b981">
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
          <n-icon size="18" color="#06b6d4">
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
          <n-icon size="18" color="#f59e0b">
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
          <n-icon size="18" color="#8b5cf6">
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
          <n-icon size="18" color="#64748b">
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
  background: rgba(30, 41, 59, 0.4);
  border: 2px solid rgba(51, 65, 85, 0.5);
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 50px;
}

.category-item:hover {
  background: rgba(30, 41, 59, 0.7);
  border-color: rgba(71, 85, 105, 0.7);
  transform: translateY(-2px);
  box-shadow: 0 6px 12px -3px rgba(0, 0, 0, 0.3);
}

.category-item.selected {
  background: rgba(30, 41, 59, 0.8);
}

.category-item.category-backend.selected {
  box-shadow: 0 0 0 2px rgba(16, 185, 129, 0.4), 0 6px 12px -3px rgba(0, 0, 0, 0.3);
}

.category-item.category-database.selected {
  box-shadow: 0 0 0 2px rgba(6, 182, 212, 0.4), 0 6px 12px -3px rgba(0, 0, 0, 0.3);
}

.category-item.category-feature.selected {
  box-shadow: 0 0 0 2px rgba(245, 158, 11, 0.4), 0 6px 12px -3px rgba(0, 0, 0, 0.3);
}

.category-item.category-docs.selected {
  box-shadow: 0 0 0 2px rgba(139, 92, 246, 0.4), 0 6px 12px -3px rgba(0, 0, 0, 0.3);
}

.category-item.category-other.selected {
  box-shadow: 0 0 0 2px rgba(100, 116, 139, 0.4), 0 6px 12px -3px rgba(0, 0, 0, 0.3);
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
  background: rgba(16, 185, 129, 0.15);
  color: #6ee7b7;
}

.category-database .category-badge {
  background: rgba(6, 182, 212, 0.15);
  color: #67e8f9;
}

.category-feature .category-badge {
  background: rgba(245, 158, 11, 0.15);
  color: #fcd34d;
}

.category-docs .category-badge {
  background: rgba(139, 92, 246, 0.15);
  color: #c4b5fd;
}

.category-other .category-badge {
  background: rgba(100, 116, 139, 0.15);
  color: #94a3b8;
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
  .category-grid {
    grid-template-columns: 1fr;
  }

  .category-item {
    min-height: 80px;
  }
}
</style>
