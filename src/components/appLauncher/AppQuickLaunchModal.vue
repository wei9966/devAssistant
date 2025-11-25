<template>
  <n-modal
    v-model:show="modalVisible"
    :mask-closable="true"
    :close-on-esc="true"
    :auto-focus="true"
    class="quick-launch-modal"
    transform-origin="center"
  >
    <div class="modal-content" @keydown="handleKeyDown">
      <!-- 搜索框 -->
      <div class="search-box">
        <n-icon size="22" class="search-icon"><SearchOutline /></n-icon>
        <input
          ref="searchInputRef"
          v-model="searchKeyword"
          type="text"
          class="search-input"
          placeholder="输入搜索..."
          autofocus
        />
        <div class="close-hint">
          <kbd>Esc</kbd> 关闭
        </div>
      </div>

      <!-- 搜索结果列表 -->
      <div v-if="searchResults.length > 0" class="results-list">
        <div
          v-for="(app, index) in searchResults"
          :key="app.id"
          class="result-item"
          :class="{ selected: selectedIndex === index }"
          @click="handleLaunch(app)"
          @mouseenter="selectedIndex = index"
        >
          <!-- 应用图标 -->
          <div class="result-icon">
            <img v-if="app.icon" :src="app.icon" :alt="app.name" />
            <n-icon v-else size="28"><AppsOutline /></n-icon>
          </div>

          <!-- 应用信息 -->
          <div class="result-info">
            <div class="result-name" v-html="highlightMatch(app.name)"></div>
            <div class="result-path">{{ app.path }}</div>
          </div>

          <!-- 启动次数 -->
          <div class="result-count">
            <n-icon size="14"><FlashOutline /></n-icon>
            <span>{{ app.launchCount }}</span>
          </div>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else-if="searchKeyword" class="empty-state">
        <n-icon size="48" color="#64748b"><SearchOutline /></n-icon>
        <p>未找到匹配的应用</p>
      </div>

      <!-- 默认状态（显示最近使用和置顶应用） -->
      <div v-else class="default-state">
        <!-- 置顶应用 -->
        <div v-if="pinnedApps.length > 0" class="quick-section">
          <div class="section-title">
            <n-icon size="14"><Pin /></n-icon>
            <span>置顶应用</span>
          </div>
          <div class="quick-apps-grid">
            <div
              v-for="app in pinnedApps"
              :key="app.id"
              class="quick-app-item"
              draggable="true"
              @dragstart="onQuickItemDragStart(app, $event)"
              @dragend="onQuickItemDragEnd"
              @click="handleLaunch(app)"
            >
              <div class="quick-app-icon">
                <img v-if="app.icon" :src="app.icon" :alt="app.name" />
                <n-icon v-else size="24"><AppsOutline /></n-icon>
              </div>
              <div class="quick-app-name">{{ app.name }}</div>
            </div>
          </div>
        </div>

        <!-- 最近使用 -->
        <div v-if="recentApps.length > 0" class="quick-section">
          <div class="section-title">
            <n-icon size="14"><TimeOutline /></n-icon>
            <span>最近使用</span>
          </div>
          <div class="quick-apps-grid">
            <div
              v-for="app in recentApps"
              :key="app.id"
              class="quick-app-item"
              draggable="true"
              @dragstart="onQuickItemDragStart(app, $event)"
              @dragend="onQuickItemDragEnd"
              @click="handleLaunch(app)"
            >
              <div class="quick-app-icon">
                <img v-if="app.icon" :src="app.icon" :alt="app.name" />
                <n-icon v-else size="24"><AppsOutline /></n-icon>
              </div>
              <div class="quick-app-name">{{ app.name }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 快捷键提示 -->
      <div class="keyboard-hints">
        <span><kbd>↑</kbd><kbd>↓</kbd> 选择</span>
        <span><kbd>Enter</kbd> 启动</span>
        <span><kbd>Esc</kbd> 关闭</span>
      </div>
    </div>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { NModal, NIcon } from 'naive-ui';
import { SearchOutline, AppsOutline, FlashOutline, Pin, TimeOutline } from '@vicons/ionicons5';
import type { AppItem } from '@/types/appLauncher';

const props = withDefaults(
  defineProps<{
    show: boolean;
    apps: AppItem[];
  }>(),
  {
    apps: () => [],
  }
);

const emit = defineEmits<{
  'update:show': [value: boolean];
  launch: [app: AppItem];
}>();

const searchInputRef = ref<HTMLInputElement>();
const searchKeyword = ref('');
const selectedIndex = ref(0);

const modalVisible = computed({
  get: () => props.show,
  set: (val) => emit('update:show', val),
});

// 置顶应用
const pinnedApps = computed(() => {
  return props.apps.filter((app) => app.isPinned && !app.isHidden).slice(0, 6);
});

// 最近使用的应用（按最后启动时间排序）
const recentApps = computed(() => {
  return props.apps
    .filter((app) => !app.isHidden && app.lastLaunchedAt)
    .sort((a, b) => (b.lastLaunchedAt || 0) - (a.lastLaunchedAt || 0))
    .slice(0, 6);
});

// 搜索结果（模糊搜索 + 智能排序）
const searchResults = computed(() => {
  if (!searchKeyword.value.trim()) return [];

  const keyword = searchKeyword.value.toLowerCase();
  const results = props.apps
    .filter((app) => !app.isHidden)
    .map((app) => {
      const name = app.name.toLowerCase();
      let score = 0;

      // 精确匹配得分最高
      if (name === keyword) {
        score = 1000;
      }
      // 开头匹配
      else if (name.startsWith(keyword)) {
        score = 500 + app.launchCount;
      }
      // 包含匹配
      else if (name.includes(keyword)) {
        score = 100 + app.launchCount;
      }
      // 拼音首字母匹配（简单实现）
      else if (matchPinyin(app.name, keyword)) {
        score = 50 + app.launchCount;
      }

      // 置顶应用加分
      if (app.isPinned) {
        score += 200;
      }

      return { app, score };
    })
    .filter((item) => item.score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, 10)
    .map((item) => item.app);

  return results;
});

// 简单的拼音首字母匹配（实际项目中应使用专业的拼音库）
const matchPinyin = (text: string, keyword: string): boolean => {
  // TODO: 实现真正的拼音匹配逻辑
  return false;
};

// 高亮匹配文本
const highlightMatch = (text: string): string => {
  if (!searchKeyword.value) return text;
  const keyword = searchKeyword.value;
  const regex = new RegExp(`(${keyword})`, 'gi');
  return text.replace(regex, '<span class="highlight">$1</span>');
};

// 键盘导航
const handleKeyDown = (event: KeyboardEvent) => {
  if (searchResults.value.length === 0) return;

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault();
      selectedIndex.value = (selectedIndex.value + 1) % searchResults.value.length;
      scrollToSelected();
      break;
    case 'ArrowUp':
      event.preventDefault();
      selectedIndex.value =
        selectedIndex.value === 0 ? searchResults.value.length - 1 : selectedIndex.value - 1;
      scrollToSelected();
      break;
    case 'Enter':
      event.preventDefault();
      if (searchResults.value[selectedIndex.value]) {
        handleLaunch(searchResults.value[selectedIndex.value]);
      }
      break;
  }
};

// 滚动到选中项
  const scrollToSelected = () => {
  nextTick(() => {
    const selectedEl = document.querySelector('.result-item.selected');
    if (selectedEl) {
      selectedEl.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  });
};

// 启动应用
const handleLaunch = (app: AppItem) => {
  emit('launch', app);
  modalVisible.value = false;
  searchKeyword.value = '';
  selectedIndex.value = 0;
};

const onQuickItemDragStart = (app: AppItem, e: DragEvent) => {
  if (!e.dataTransfer) return;
  const json = JSON.stringify(app);
  e.dataTransfer.effectAllowed = 'move';
  e.dataTransfer.setData('application/json', json);
  e.dataTransfer.setData('text/plain', app.name);
  const ghost = document.createElement('canvas');
  ghost.width = 1;
  ghost.height = 1;
  e.dataTransfer.setDragImage(ghost, 0, 0);
};

const onQuickItemDragEnd = () => {};

// 监听显示状态，自动聚焦
watch(modalVisible, (visible) => {
  if (visible) {
    nextTick(() => {
      searchInputRef.value?.focus();
    });
  } else {
    searchKeyword.value = '';
    selectedIndex.value = 0;
  }
});

// 监听搜索关键词变化，重置选中索引
watch(searchKeyword, () => {
  selectedIndex.value = 0;
});
</script>

<style scoped>
.quick-launch-modal {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 15vh;
}

.quick-launch-modal :deep(.n-modal) {
  max-width: 640px;
  width: 90%;
  margin: 0;
}

.modal-content {
  background: rgba(15, 23, 42, 0.98);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: 20px;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(99, 102, 241, 0.2);
  overflow: hidden;
}

/* 搜索框 */
.search-box {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 20px 24px;
  border-bottom: 1px solid rgba(51, 65, 85, 0.5);
}

.search-icon {
  color: #6366f1;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 18px;
  color: #e2e8f0;
  font-weight: 400;
}

.search-input::placeholder {
  color: #64748b;
}

.close-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #64748b;
  flex-shrink: 0;
}

.close-hint kbd {
  padding: 3px 8px;
  background: rgba(51, 65, 85, 0.5);
  border: 1px solid rgba(71, 85, 105, 0.6);
  border-radius: 5px;
  font-size: 11px;
  font-family: monospace;
}

/* 搜索结果列表 */
.results-list {
  max-height: 400px;
  overflow-y: auto;
  padding: 8px;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 16px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.result-item:hover,
.result-item.selected {
  background: rgba(99, 102, 241, 0.15);
  border-color: rgba(99, 102, 241, 0.3);
}

.result-icon {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(99, 102, 241, 0.1);
  border-radius: 10px;
  flex-shrink: 0;
}

.result-icon img {
  width: 32px;
  height: 32px;
  object-fit: contain;
}

.result-info {
  flex: 1;
  min-width: 0;
}

.result-name {
  font-size: 15px;
  font-weight: 500;
  color: #e2e8f0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 4px;
}

.result-name :deep(.highlight) {
  color: #6366f1;
  background: rgba(99, 102, 241, 0.2);
  padding: 0 2px;
  border-radius: 3px;
}

.result-path {
  font-size: 11px;
  color: #64748b;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-count {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #94a3b8;
  flex-shrink: 0;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: #64748b;
}

.empty-state p {
  margin-top: 12px;
  font-size: 14px;
}

/* 默认状态 */
.default-state {
  padding: 16px;
  max-height: 400px;
  overflow-y: auto;
}

.quick-section {
  margin-bottom: 20px;
}

.quick-section:last-child {
  margin-bottom: 0;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 12px;
}

.quick-apps-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  gap: 10px;
}

.quick-app-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 14px 10px;
  background: rgba(30, 41, 59, 0.4);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.quick-app-item:hover {
  background: rgba(99, 102, 241, 0.15);
  border-color: rgba(99, 102, 241, 0.4);
  transform: translateY(-2px);
}

.quick-app-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(99, 102, 241, 0.1);
  border-radius: 10px;
}

.quick-app-icon img {
  width: 32px;
  height: 32px;
  object-fit: contain;
}

.quick-app-name {
  font-size: 12px;
  color: #cbd5e1;
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  width: 100%;
}

/* 快捷键提示 */
.keyboard-hints {
  display: flex;
  justify-content: center;
  gap: 16px;
  padding: 12px 24px;
  border-top: 1px solid rgba(51, 65, 85, 0.5);
  font-size: 11px;
  color: #64748b;
}

.keyboard-hints kbd {
  display: inline-block;
  padding: 2px 6px;
  background: rgba(51, 65, 85, 0.5);
  border: 1px solid rgba(71, 85, 105, 0.6);
  border-radius: 4px;
  font-size: 10px;
  font-family: monospace;
  margin: 0 2px;
}
</style>
