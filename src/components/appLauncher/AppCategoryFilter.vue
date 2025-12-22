<template>
  <div class="category-filter">
    <div class="category-tabs" ref="categoryTabsRef">
      <!-- 分类标签 -->
      <div
        v-for="category in categories"
        :key="category.id"
        class="category-tab"
        :class="{
          active: selectedCategory === category.id,
          'drag-over': isDragging && hoveredCategoryId === category.id && category.id !== 'all',
          'drag-disabled': isDragging && hoveredCategoryId === category.id && category.id === 'all',
          'drop-target': isDragging && category.id !== 'all'
        }"
        :data-category-id="category.id"
        @click="handleSelectCategory(category.id)"
      >
        <span v-if="category.icon" class="category-icon">{{ category.icon }}</span>
        <span class="category-name">{{ category.name }}</span>
        <span v-if="getCategoryCount(category.id) > 0" class="category-count">
          {{ getCategoryCount(category.id) }}
        </span>
      </div>

      <!-- 管理分类按钮 -->
      <div class="category-tab manage-btn" @click="handleManageCategories">
        <n-icon size="16"><SettingsOutline /></n-icon>
        <span class="category-name">管理</span>
      </div>
    </div>

    <!-- 滚动指示器（当分类过多时） -->
    <div v-if="showScrollIndicator" class="scroll-indicator">
      <div class="scroll-left" @click="scrollLeft">
        <n-icon><ChevronBack /></n-icon>
      </div>
      <div class="scroll-right" @click="scrollRight">
        <n-icon><ChevronForward /></n-icon>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { NIcon } from 'naive-ui';
import { SettingsOutline, ChevronBack, ChevronForward } from '@vicons/ionicons5';
import type { Category } from '@/types/appLauncher';

const props = withDefaults(
  defineProps<{
    categories: Category[];
    selectedCategory: string;
    categoryCounts?: Record<string, number>;
    hoveredCategoryId?: string | null;
    isDragging?: boolean;
  }>(),
  {
    categoryCounts: () => ({}),
    hoveredCategoryId: null,
    isDragging: false,
  }
);

const emit = defineEmits<{
  'update:selectedCategory': [categoryId: string];
  select: [categoryId: string];
  manage: [];
}>();

const categoryTabsRef = ref<HTMLDivElement>();
const showScrollIndicator = ref(false);

// 获取分类的应用数量
const getCategoryCount = (categoryId: string): number => {
  return props.categoryCounts[categoryId] || 0;
};

// 选择分类
const handleSelectCategory = (categoryId: string) => {
  emit('update:selectedCategory', categoryId);
  emit('select', categoryId);
};

// 管理分类
const handleManageCategories = () => {
  emit('manage');
};

// 滚动控制
const scrollLeft = () => {
  if (categoryTabsRef.value) {
    categoryTabsRef.value.scrollBy({ left: -200, behavior: 'smooth' });
  }
};

const scrollRight = () => {
  if (categoryTabsRef.value) {
    categoryTabsRef.value.scrollBy({ left: 200, behavior: 'smooth' });
  }
};

// 检查是否需要显示滚动指示器
const checkScrollIndicator = () => {
  if (categoryTabsRef.value) {
    showScrollIndicator.value =
      categoryTabsRef.value.scrollWidth > categoryTabsRef.value.clientWidth;
  }
};

onMounted(() => {
  checkScrollIndicator();
  window.addEventListener('resize', checkScrollIndicator);
});
</script>

<style scoped>
.category-filter {
  position: relative;
  width: 100%;
}

.category-tabs {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  overflow-y: hidden;
  padding: 4px 0;
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* IE/Edge */
}

.category-tabs::-webkit-scrollbar {
  display: none; /* Chrome/Safari */
}

.category-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  white-space: nowrap;
  flex-shrink: 0;
  position: relative;
  overflow: hidden;
  pointer-events: auto;  /* 确保可以接收事件 */
  z-index: 10;  /* 确保在其他元素之上 */
}

.category-tab::before {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent, var(--accent-glow), transparent);
  transform: translateX(-100%);
  transition: transform 0.6s;
}

.category-tab:hover::before {
  transform: translateX(100%);
}

.category-tab:hover {
  background: var(--card-hover-bg);
  border-color: var(--border-hover);
  transform: translateY(-2px);
}

.category-tab.active {
  background: var(--accent-glow);
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 1px var(--accent-glow), var(--shadow-md);
}

.category-tab.drag-over {
  background: linear-gradient(135deg, rgba(34, 197, 94, 0.2), rgba(34, 197, 94, 0.1));
  border-color: var(--success);
  box-shadow: 0 0 0 2px rgba(34, 197, 94, 0.3), 0 4px 16px rgba(34, 197, 94, 0.2);
  transform: scale(1.05);
  animation: pulse 1s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 0 0 2px rgba(34, 197, 94, 0.3), 0 4px 16px rgba(34, 197, 94, 0.2);
  }
  50% {
    box-shadow: 0 0 0 4px rgba(34, 197, 94, 0.4), 0 4px 20px rgba(34, 197, 94, 0.3);
  }
}

.category-tab.drag-disabled {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.2), rgba(239, 68, 68, 0.1));
  border-color: var(--error);
  cursor: not-allowed;
  opacity: 0.7;
}

.category-tab.drop-target {
  border-style: dashed;
  border-color: var(--border-active);
}

.category-icon {
  font-size: 16px;
  line-height: 1;
  pointer-events: none;  /* 让事件穿透到父元素 */
}

.category-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  transition: color 0.3s;
  pointer-events: none;  /* 让事件穿透到父元素 */
}

.category-tab:hover .category-name,
.category-tab.active .category-name {
  color: var(--text-primary);
}

.category-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  height: 20px;
  pointer-events: none;  /* 让事件穿透到父元素 */
  padding: 0 6px;
  background: var(--accent-glow);
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
  color: var(--accent-secondary);
}

.category-tab.active .category-count {
  background: var(--bg-hover);
  color: var(--accent-primary);
}

.manage-btn {
  border-style: dashed;
  border-color: var(--border-hover);
  color: var(--text-muted);
}

.manage-btn:hover {
  border-color: var(--accent-primary);
  color: var(--accent-secondary);
}

/* 滚动指示器 */
.scroll-indicator {
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  transform: translateY(-50%);
  pointer-events: none;
  display: flex;
  justify-content: space-between;
  padding: 0 8px;
}

.scroll-left,
.scroll-right {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-overlay);
  border: 1px solid var(--border-default);
  border-radius: 50%;
  color: var(--text-muted);
  cursor: pointer;
  pointer-events: auto;
  transition: all 0.2s;
  backdrop-filter: blur(8px);
}

.scroll-left:hover,
.scroll-right:hover {
  background: var(--bg-elevated);
  border-color: var(--accent-primary);
  color: var(--accent-secondary);
}

.scroll-left {
  box-shadow: var(--shadow-md);
}

.scroll-right {
  box-shadow: var(--shadow-md);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .category-tab {
    padding: 8px 12px;
  }

  .category-name {
    font-size: 12px;
  }

  .category-icon {
    font-size: 14px;
  }
}
</style>
