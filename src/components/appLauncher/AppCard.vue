<template>
  <div
    ref="cardRef"
    class="app-card"
    :class="{ pinned: app.isPinned, hidden: app.isHidden, dragging: isBeingDragged }"
    @mousedown="handleMouseDown"
    @click="handleLaunch"
  >
    <!-- Hover Glow Effect -->
    <div class="hover-glow"></div>

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

      <!-- 启动次数 -->
      <div class="app-stats">
        <span class="launch-count">{{ app.launchCount }}次</span>
      </div>
    </div>

    <!-- 操作按钮（悬停显示） -->
    <div class="card-actions" @dragstart.prevent.stop>
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
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { NButton, NIcon } from 'naive-ui';
import { AppsOutline, CreateOutline, TrashOutline, Pin } from '@vicons/ionicons5';
import type { AppItem } from '@/types/appLauncher';

const props = defineProps<{
  app: AppItem;
  isBeingDragged?: boolean;
}>();

const emit = defineEmits<{
  launch: [appId: string];
  pin: [appId: string];
  edit: [app: AppItem];
  delete: [appId: string];
  'mousedown-drag': [app: AppItem, event: MouseEvent, element: HTMLElement];
}>();

const cardRef = ref<HTMLElement | null>(null);
const iconError = ref(false);
const mouseDownTime = ref(0);
const isDragStarted = ref(false);

const handleLaunch = () => {
  // 如果刚刚拖拽过，不触发点击
  if (isDragStarted.value || Date.now() - mouseDownTime.value > 300) {
    isDragStarted.value = false;
    return;
  }
  emit('launch', props.app.id);
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

// 鼠标按下处理 - 用于拖拽
const handleMouseDown = (e: MouseEvent) => {
  // 忽略右键和中键
  if (e.button !== 0) return;

  // 检查是否点击了操作按钮区域
  const target = e.target as HTMLElement;
  if (target.closest('.card-actions')) return;

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
  background: rgba(30, 41, 59, 0.4);
  border: 1px solid rgba(51, 65, 85, 0.5);
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
  background: rgba(30, 41, 59, 0.8);
  border-color: rgba(99, 102, 241, 0.6);
  box-shadow: 0 10px 30px -5px rgba(99, 102, 241, 0.3), 0 4px 6px -2px rgba(0, 0, 0, 0.2);
  transform: translateY(-4px);
}

.app-card.pinned {
  border-left: 3px solid #6366f1;
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
  background: linear-gradient(90deg, transparent 0%, rgba(99, 102, 241, 0.1) 50%, transparent 100%);
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
  background: rgba(99, 102, 241, 0.1);
  border-radius: 12px;
  transition: all 0.3s;
}

.app-card:hover .app-icon {
  background: rgba(99, 102, 241, 0.2);
  transform: scale(1.1);
}

.app-icon img {
  width: 40px;
  height: 40px;
  object-fit: contain;
}

.icon-placeholder {
  color: #6366f1;
}

.pin-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  width: 18px;
  height: 18px;
  background: #6366f1;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

/* 应用名称 */
.app-name {
  font-size: 14px;
  font-weight: 500;
  color: #e2e8f0;
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
  color: #94a3b8;
  display: flex;
  align-items: center;
  gap: 4px;
}

.launch-count {
  padding: 2px 8px;
  background: rgba(99, 102, 241, 0.15);
  border-radius: 10px;
  color: #a5b4fc;
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
  color: #94a3b8;
  background: rgba(15, 23, 42, 0.8);
  backdrop-filter: blur(8px);
  border-radius: 8px;
  padding: 6px;
  transition: all 0.2s;
}

.action-btn:hover {
  color: #e2e8f0;
  background: rgba(51, 65, 85, 0.9);
}

.pin-btn.active {
  color: #6366f1;
}

.pin-btn.active:hover {
  color: #818cf8;
}
</style>
