<template>
  <div
    ref="cardRef"
    class="app-card"
    :class="{
      pinned: app.isPinned,
      hidden: app.isHidden,
      dragging: isBeingDragged,
      'batch-mode': batchSelectMode,
      selected: isSelected
    }"
    @mousedown="handleMouseDown"
    @click="handleClick"
    @contextmenu.prevent="handleContextMenu"
  >
    <!-- Hover Glow Effect -->
    <div class="hover-glow"></div>

    <!-- 批量选择复选框 -->
    <div v-if="batchSelectMode" class="select-checkbox" @click.stop="handleToggleSelect">
      <n-checkbox :checked="isSelected" />
    </div>

    <div class="card-content">
      <!-- 应用图标 -->
      <div class="app-icon">
        <img v-if="app.icon" :src="app.icon" :alt="app.name" @error="handleIconError" draggable="false" />
        <div v-else class="icon-placeholder">
          <n-icon size="32"><AppsOutline /></n-icon>
        </div>
        <!-- 置顶标记 -->
        <div v-if="app.isPinned" class="pin-badge">
          <n-icon size="12"><Pin /></n-icon>
        </div>
      </div>

      <!-- 应用名称 -->
      <h3 class="app-name" :title="app.name">{{ app.name }}</h3>

      <!-- 启动次数和来源标签 -->
      <div class="app-stats">
        <span class="launch-count">{{ app.launchCount }}次</span>
        <span v-if="app.appSource && app.appSource !== 'manual'" class="source-tag" :class="'source-' + app.appSource">
          {{ getSourceName(app.appSource) }}
        </span>
      </div>
    </div>

    <!-- 操作按钮（悬停显示） -->
    <div v-if="!batchSelectMode" class="card-actions" @dragstart.prevent.stop>
      <n-button
        text
        size="small"
        class="action-btn pin-btn"
        :class="{ active: app.isPinned }"
        @click.stop="handlePin"
        :title="app.isPinned ? '取消置顶' : '置顶'"
      >
        <template #icon>
          <n-icon><Pin /></n-icon>
        </template>
      </n-button>
      <n-button text size="small" class="action-btn" @click.stop="handleEdit" title="编辑">
        <template #icon>
          <n-icon><CreateOutline /></n-icon>
        </template>
      </n-button>
      <n-button text size="small" class="action-btn" @click.stop="handleDelete" title="删除">
        <template #icon>
          <n-icon><TrashOutline /></n-icon>
        </template>
      </n-button>
    </div>

    <!-- 右键菜单 -->
    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      :x="contextMenuX"
      :y="contextMenuY"
      :options="contextMenuOptions"
      :show="showContextMenu"
      @select="handleContextMenuSelect"
      @clickoutside="showContextMenu = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h } from 'vue';
import { NButton, NIcon, NCheckbox, NDropdown, NTag } from 'naive-ui';
import { AppsOutline, CreateOutline, TrashOutline, Pin, FolderOpenOutline } from '@vicons/ionicons5';
import type { AppItem } from '@/types/appLauncher';
import { APP_SOURCE_NAMES, AppSource } from '@/types/appLauncher';

const props = defineProps<{
  app: AppItem;
  isBeingDragged?: boolean;
  batchSelectMode?: boolean;
  isSelected?: boolean;
}>();

const emit = defineEmits<{
  launch: [appId: string];
  pin: [appId: string];
  edit: [app: AppItem];
  delete: [appId: string];
  'mousedown-drag': [app: AppItem, event: MouseEvent, element: HTMLElement];
  'toggle-select': [appId: string];
  'show-in-folder': [path: string];
}>();

const cardRef = ref<HTMLElement | null>(null);
const iconError = ref(false);
const mouseDownTime = ref(0);
const isDragStarted = ref(false);

// 右键菜单状态
const showContextMenu = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);

// 右键菜单选项
const contextMenuOptions = computed(() => [
  {
    label: '打开文件位置',
    key: 'show-in-folder',
    icon: () => h(NIcon, null, { default: () => h(FolderOpenOutline) }),
  },
  {
    type: 'divider',
    key: 'd1',
  },
  {
    label: props.app.isPinned ? '取消置顶' : '置顶',
    key: 'pin',
    icon: () => h(NIcon, null, { default: () => h(Pin) }),
  },
  {
    label: '编辑',
    key: 'edit',
    icon: () => h(NIcon, null, { default: () => h(CreateOutline) }),
  },
  {
    label: '删除',
    key: 'delete',
    icon: () => h(NIcon, null, { default: () => h(TrashOutline) }),
  },
]);

// 处理点击事件
const handleClick = () => {
  // 批量选择模式下点击切换选中状态
  if (props.batchSelectMode) {
    emit('toggle-select', props.app.id);
    return;
  }

  // 如果刚刚拖拽过，不触发点击
  if (isDragStarted.value || Date.now() - mouseDownTime.value > 300) {
    isDragStarted.value = false;
    return;
  }
  emit('launch', props.app.id);
};

// 切换选中状态
const handleToggleSelect = () => {
  emit('toggle-select', props.app.id);
};

// 右键菜单
const handleContextMenu = (e: MouseEvent) => {
  // 批量模式下不显示右键菜单
  if (props.batchSelectMode) return;

  contextMenuX.value = e.clientX;
  contextMenuY.value = e.clientY;
  showContextMenu.value = true;
};

// 右键菜单选择处理
const handleContextMenuSelect = (key: string) => {
  showContextMenu.value = false;
  switch (key) {
    case 'show-in-folder':
      emit('show-in-folder', props.app.path);
      break;
    case 'pin':
      handlePin();
      break;
    case 'edit':
      handleEdit();
      break;
    case 'delete':
      handleDelete();
      break;
  }
};

const handlePin = () => {
  emit('pin', props.app.id);
};

const handleEdit = () => {
  emit('edit', props.app);
};

const handleDelete = () => {
  // 直接触发删除事件，由父组件处理确认逻辑
  emit('delete', props.app.id);
};

const handleIconError = () => {
  iconError.value = true;
};

// 获取来源名称
const getSourceName = (source: string): string => {
  const sourceMap: Record<string, string> = {
    'start_menu': '开始菜单',
    'registry': '注册表',
    'shell_apps': '系统',
    'uwp': 'UWP',
    'manual': '手动',
  };
  return sourceMap[source] || source;
};

// 鼠标按下处理 - 用于拖拽
const handleMouseDown = (e: MouseEvent) => {
  // 忽略右键和中键
  if (e.button !== 0) return;

  // 批量选择模式下不启用拖拽
  if (props.batchSelectMode) return;

  // 检查是否点击了操作按钮区域或复选框
  const target = e.target as HTMLElement;
  if (target.closest('.card-actions') || target.closest('.select-checkbox')) return;

  mouseDownTime.value = Date.now();
  isDragStarted.value = false;

  // 使用延迟来区分点击和拖拽
  const startX = e.clientX;
  const startY = e.clientY;

  const onMouseMove = (moveEvent: MouseEvent) => {
    const dx = moveEvent.clientX - startX;
    const dy = moveEvent.clientY - startY;
    const distance = Math.sqrt(dx * dx + dy * dy);

    // 移动超过 5px 才开始拖拽
    if (distance > 5 && !isDragStarted.value && cardRef.value) {
      isDragStarted.value = true;
      document.removeEventListener('mousemove', onMouseMove);
      document.removeEventListener('mouseup', onMouseUp);
      emit('mousedown-drag', props.app, e, cardRef.value);
    }
  };

  const onMouseUp = () => {
    document.removeEventListener('mousemove', onMouseMove);
    document.removeEventListener('mouseup', onMouseUp);
  };

  document.addEventListener('mousemove', onMouseMove);
  document.addEventListener('mouseup', onMouseUp);
};
</script>

<style scoped>
.app-card {
  position: relative;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 16px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
  cursor: grab;
  height: 160px;
  padding: 20px;
  user-select: none;
  display: flex;
  flex-direction: column;
}

.app-card.dragging {
  opacity: 0.6;
  cursor: grabbing !important;
}

.app-card:hover:not(.dragging) {
  background: var(--card-hover-bg);
  border-color: var(--border-active);
  box-shadow: var(--shadow-lg), var(--shadow-glow);
  transform: translateY(-4px);
}

.app-card.pinned {
  border-left: 3px solid var(--accent-primary);
}

.app-card.hidden {
  opacity: 0.5;
}

.app-card.hidden:hover {
  opacity: 0.7;
}

/* Hover Glow Effect */
.hover-glow {
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent 0%, var(--accent-glow) 50%, transparent 100%);
  transform: translateX(-100%);
  transition: transform 0.7s cubic-bezier(0.4, 0, 0.2, 1);
  pointer-events: none;
}

.app-card.dragging * :not(.card-actions):not(.card-actions *) {
  pointer-events: none !important;
}

.app-card:hover .hover-glow {
  transform: translateX(100%);
}

.card-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  position: relative;
}

/* 应用图标 */
.app-icon {
  position: relative;
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-glow);
  border-radius: 12px;
  transition: all 0.3s;
}

.app-card:hover .app-icon {
  background: var(--bg-hover);
  transform: scale(1.1);
}

.app-icon img {
  width: 40px;
  height: 40px;
  object-fit: contain;
}

.icon-placeholder {
  color: var(--accent-primary);
}

.pin-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  width: 18px;
  height: 18px;
  background: var(--accent-primary);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: var(--shadow-sm);
}

/* 应用名称 */
.app-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  text-align: center;
  line-height: 1.4;
  margin: 0;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

/* 启动次数 */
.app-stats {
  font-size: 11px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 4px;
}

.launch-count {
  padding: 2px 8px;
  background: var(--accent-glow);
  border-radius: 10px;
  color: var(--accent-secondary);
}

/* 来源标签样式 */
.source-tag {
  padding: 2px 6px;
  border-radius: 8px;
  font-size: 10px;
  font-weight: 500;
}

.source-start_menu {
  background: rgba(16, 185, 129, 0.15);
  color: var(--success);
}

.source-registry {
  background: rgba(59, 130, 246, 0.15);
  color: var(--info);
}

.source-shell_apps {
  background: rgba(245, 158, 11, 0.15);
  color: var(--warning);
}

.source-uwp {
  background: rgba(139, 92, 246, 0.15);
  color: var(--accent-secondary);
}

/* 操作按钮 */
.card-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
  opacity: 0;
  transform: translateY(-4px);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.app-card:hover .card-actions {
  opacity: 1;
  transform: translateY(0);
}

.action-btn {
  color: var(--text-secondary);
  background: var(--bg-overlay);
  backdrop-filter: blur(8px);
  border-radius: 8px;
  padding: 6px;
  transition: all 0.2s;
}

.action-btn:hover {
  color: var(--text-primary);
  background: var(--bg-elevated);
}

.pin-btn.active {
  color: var(--accent-primary);
}

.pin-btn.active:hover {
  color: var(--accent-secondary);
}

/* 批量选择模式样式 */
.app-card.batch-mode {
  cursor: pointer;
}

.app-card.batch-mode:hover {
  border-color: var(--border-hover);
}

.app-card.selected {
  border-color: var(--accent-primary);
  background: var(--accent-glow);
}

.app-card.selected:hover {
  border-color: var(--accent-secondary);
  background: var(--bg-hover);
}

.select-checkbox {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: var(--bg-overlay);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.select-checkbox:hover {
  background: var(--accent-glow);
}

.app-card.selected .select-checkbox {
  background: var(--accent-glow);
}
</style>
