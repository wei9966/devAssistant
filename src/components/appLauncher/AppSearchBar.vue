<template>
  <div class="search-bar-container">
    <div class="search-bar">
      <div class="search-icon">
        <n-icon size="20"><SearchOutline /></n-icon>
      </div>
      <input
        ref="searchInputRef"
        v-model="searchKeyword"
        type="text"
        class="search-input"
        placeholder="搜索应用或网址... (支持拼音首字母)"
        @input="handleSearch"
        @keydown.enter="handleEnter"
        @keydown.esc="handleEscape"
      />
      <div v-if="searchKeyword" class="clear-btn" @click="handleClear">
        <n-icon size="18"><CloseCircle /></n-icon>
      </div>
      <div class="shortcut-hint">
        <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Space</kbd>
      </div>
    </div>

    <!-- 搜索结果数量 -->
    <div v-if="searchKeyword && resultCount !== null" class="search-result-info">
      找到 <span class="result-count">{{ resultCount }}</span> 个应用
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { NIcon } from 'naive-ui';
import { SearchOutline, CloseCircle } from '@vicons/ionicons5';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    resultCount?: number | null;
    autoFocus?: boolean;
  }>(),
  {
    resultCount: null,
    autoFocus: false,
  }
);

const emit = defineEmits<{
  'update:modelValue': [value: string];
  search: [keyword: string];
  enter: [];
  clear: [];
}>();

const searchInputRef = ref<HTMLInputElement>();
const searchKeyword = ref(props.modelValue);

// 监听外部变化
watch(
  () => props.modelValue,
  (newVal) => {
    searchKeyword.value = newVal;
  }
);

// 监听内部变化
watch(searchKeyword, (newVal) => {
  emit('update:modelValue', newVal);
});

// 自动聚焦
if (props.autoFocus) {
  setTimeout(() => {
    searchInputRef.value?.focus();
  }, 100);
}

const handleSearch = () => {
  emit('search', searchKeyword.value);
};

const handleEnter = () => {
  emit('enter');
};

const handleEscape = () => {
  if (searchKeyword.value) {
    handleClear();
  }
};

const handleClear = () => {
  searchKeyword.value = '';
  emit('clear');
  searchInputRef.value?.focus();
};

// 暴露方法给父组件
defineExpose({
  focus: () => searchInputRef.value?.focus(),
  blur: () => searchInputRef.value?.blur(),
  clear: handleClear,
});
</script>

<style scoped>
.search-bar-container {
  width: 100%;
}

.search-bar {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  background: var(--input-bg);
  border: 2px solid var(--input-border);
  border-radius: 16px;
  padding: 14px 20px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  backdrop-filter: blur(8px);
}

.search-bar:focus-within {
  background: var(--bg-surface);
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px var(--accent-glow), var(--shadow-md);
}

.search-icon {
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  transition: color 0.3s;
  flex-shrink: 0;
}

.search-bar:focus-within .search-icon {
  color: var(--accent-primary);
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 15px;
  color: var(--text-primary);
  font-weight: 400;
  min-width: 0;
}

.search-input::placeholder {
  color: var(--text-dim);
}

.clear-btn {
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  transition: all 0.2s;
  flex-shrink: 0;
  padding: 4px;
  border-radius: 50%;
}

.clear-btn:hover {
  color: var(--text-primary);
  background: var(--accent-glow);
}

.shortcut-hint {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-secondary);
  flex-shrink: 0;
  margin-left: 8px;
  padding-left: 12px;
  border-left: 1px solid var(--border-default);
}

.shortcut-hint kbd {
  display: inline-block;
  padding: 2px 6px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 4px;
  font-size: 10px;
  font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace;
  color: var(--text-secondary);
  box-shadow: var(--shadow-sm);
}

.search-result-info {
  margin-top: 12px;
  font-size: 13px;
  color: var(--text-secondary);
  text-align: center;
}

.result-count {
  color: var(--accent-primary);
  font-weight: 600;
  font-size: 14px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .shortcut-hint {
    display: none;
  }

  .search-bar {
    padding: 12px 16px;
  }

  .search-input {
    font-size: 14px;
  }
}
</style>
