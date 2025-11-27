<template>
  <n-config-provider :theme="darkTheme">
    <div class="task-float-wrapper">
      <!-- 拖动区域 -->
      <div class="drag-handle" data-tauri-drag-region>
        <div class="handle-dots">
          <span></span><span></span><span></span>
        </div>
      </div>

      <!-- 任务列表 -->
      <div class="task-list">
        <!-- 进行中的任务 -->
        <div v-if="activeTasks.length > 0" class="task-section">
          <div class="section-header">
            <div class="status-dot active"></div>
            <span>进行中</span>
            <span class="count">{{ activeTasks.length }}</span>
          </div>
          <div
            v-for="task in activeTasks"
            :key="task.id"
            class="task-item active"
            @click="toggleTaskDetail(task)"
            @contextmenu.prevent="showContextMenu($event, task)"
          >
            <span class="task-title">{{ task.title }}</span>
            <!-- 展开的详情 -->
            <div v-if="expandedTaskId === task.id" class="task-detail">
              <div v-if="task.description" class="detail-row">
                <span class="detail-label">描述:</span>
                <span class="detail-value">{{ task.description }}</span>
              </div>
              <div v-if="task.gitBranch" class="detail-row">
                <span class="detail-label">分支:</span>
                <span class="detail-value branch">{{ task.gitBranch }}</span>
              </div>
              <div v-if="task.startedAt" class="detail-row">
                <span class="detail-label">开始:</span>
                <span class="detail-value">{{ formatTime(task.startedAt) }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 待办任务 -->
        <div v-if="todoTasks.length > 0" class="task-section">
          <div class="section-header">
            <div class="status-dot todo"></div>
            <span>待办</span>
            <span class="count">{{ todoTasks.length }}</span>
          </div>
          <div
            v-for="task in displayedTodoTasks"
            :key="task.id"
            class="task-item todo"
            @click="toggleTaskDetail(task)"
            @contextmenu.prevent="showContextMenu($event, task)"
          >
            <span class="task-title">{{ task.title }}</span>
            <!-- 展开的详情 -->
            <div v-if="expandedTaskId === task.id" class="task-detail">
              <div v-if="task.description" class="detail-row">
                <span class="detail-label">描述:</span>
                <span class="detail-value">{{ task.description }}</span>
              </div>
              <div v-if="task.dueDate" class="detail-row">
                <span class="detail-label">截止:</span>
                <span class="detail-value">{{ formatTime(task.dueDate) }}</span>
              </div>
            </div>
          </div>
          <div v-if="todoTasks.length > maxDisplayTodo && !showAllTodo" class="more-hint" @click="showAllTodo = true">
            点击展开剩余 {{ todoTasks.length - maxDisplayTodo }} 个任务
          </div>
        </div>

        <!-- 空状态 -->
        <div v-if="activeTasks.length === 0 && todoTasks.length === 0" class="empty-state">
          <span>暂无任务</span>
        </div>
      </div>

      <!-- 底部操作栏 -->
      <div class="action-bar">
        <button class="action-btn" @click="openMainWindow" title="打开主窗口">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
            <line x1="9" y1="3" x2="9" y2="21"></line>
          </svg>
        </button>
        <button class="action-btn" @click="refreshTasks" title="刷新">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <polyline points="23 4 23 10 17 10"></polyline>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
          </svg>
        </button>
        <button class="action-btn close" @click="hideWindow" title="隐藏">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <!-- 右键菜单 -->
      <div
        v-if="contextMenu.visible"
        class="context-menu"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
        @mouseleave="hideContextMenu"
      >
        <template v-if="contextMenu.task?.status === 'active'">
          <div class="menu-item" @click="handleComplete">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
            <span>完成任务</span>
          </div>
          <div class="menu-item" @click="handlePause">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <rect x="6" y="4" width="4" height="16"></rect>
              <rect x="14" y="4" width="4" height="16"></rect>
            </svg>
            <span>暂停任务</span>
          </div>
        </template>
        <template v-else-if="contextMenu.task?.status === 'todo'">
          <div class="menu-item" @click="handleStart">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <polygon points="5 3 19 12 5 21 5 3"></polygon>
            </svg>
            <span>开始任务</span>
          </div>
          <div class="menu-item" @click="handleComplete">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
            <span>完成任务</span>
          </div>
        </template>
      </div>
    </div>
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { NConfigProvider, darkTheme } from 'naive-ui';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, Window } from '@tauri-apps/api/window';
import { listen, emit } from '@tauri-apps/api/event';
import type { Task } from '@/types/task';

const activeTasks = ref<Task[]>([]);
const todoTasks = ref<Task[]>([]);
const maxDisplayTodo = 5;
const showAllTodo = ref(false);
const expandedTaskId = ref<number | null>(null);

// 右键菜单状态
const contextMenu = ref<{
  visible: boolean;
  x: number;
  y: number;
  task: Task | null;
}>({
  visible: false,
  x: 0,
  y: 0,
  task: null,
});

const displayedTodoTasks = computed(() => {
  if (showAllTodo.value) {
    return todoTasks.value;
  }
  return todoTasks.value.slice(0, maxDisplayTodo);
});

let unlistenTaskUpdate: (() => void) | null = null;

onMounted(async () => {
  await loadTasks();

  // 监听任务更新事件
  unlistenTaskUpdate = await listen('task-updated', async () => {
    await loadTasks();
  });

  // 定时刷新（每30秒）
  setInterval(loadTasks, 30000);

  // 点击其他地方关闭右键菜单
  document.addEventListener('click', hideContextMenu);
});

onUnmounted(() => {
  if (unlistenTaskUpdate) {
    unlistenTaskUpdate();
  }
  document.removeEventListener('click', hideContextMenu);
});

async function loadTasks() {
  try {
    const tasks: Task[] = await invoke('get_all_tasks');
    activeTasks.value = tasks.filter(t => t.status === 'active');
    todoTasks.value = tasks.filter(t => t.status === 'todo');
  } catch (error) {
    console.error('加载任务失败:', error);
  }
}

function toggleTaskDetail(task: Task) {
  if (expandedTaskId.value === task.id) {
    expandedTaskId.value = null;
  } else {
    expandedTaskId.value = task.id ?? null;
  }
}

function showContextMenu(event: MouseEvent, task: Task) {
  // 获取鼠标相对于窗口的位置
  const x = event.clientX;
  const y = event.clientY;

  // 确保菜单不会超出窗口边界
  const menuWidth = 130;
  const menuHeight = 80;
  const maxX = window.innerWidth - menuWidth - 10;
  const maxY = window.innerHeight - menuHeight - 10;

  contextMenu.value = {
    visible: true,
    x: Math.min(x, maxX),
    y: Math.min(y, maxY),
    task,
  };
}

function hideContextMenu() {
  contextMenu.value.visible = false;
}

async function handleStart() {
  if (!contextMenu.value.task?.id) return;
  try {
    await invoke('start_task', { taskId: contextMenu.value.task.id });
    await loadTasks();
    await emit('task-updated');
  } catch (error) {
    console.error('开始任务失败:', error);
  }
  hideContextMenu();
}

async function handlePause() {
  if (!contextMenu.value.task?.id) return;
  try {
    await invoke('pause_task', { taskId: contextMenu.value.task.id, context: null });
    await loadTasks();
    await emit('task-updated');
  } catch (error) {
    console.error('暂停任务失败:', error);
  }
  hideContextMenu();
}

async function handleComplete() {
  if (!contextMenu.value.task?.id) return;
  try {
    await invoke('complete_task', { taskId: contextMenu.value.task.id });
    await loadTasks();
    await emit('task-updated');
  } catch (error) {
    console.error('完成任务失败:', error);
  }
  hideContextMenu();
}

function formatTime(timeStr: string) {
  if (!timeStr) return '';
  const date = new Date(timeStr);
  return `${date.getMonth() + 1}-${date.getDate()} ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
}

async function openMainWindow() {
  try {
    const mainWindow = await Window.getByLabel('main');
    if (mainWindow) {
      await mainWindow.show();
      await mainWindow.setFocus();
    }
  } catch (error) {
    console.error('打开主窗口失败:', error);
  }
}

async function refreshTasks() {
  await loadTasks();
}

async function hideWindow() {
  const currentWindow = getCurrentWindow();
  await currentWindow.hide();
}
</script>

<style scoped>
.task-float-wrapper {
  width: 100%;
  height: 100%;
  background: #0f172a;
  border: 1px solid rgba(100, 116, 139, 0.5);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(0, 0, 0, 0.3);
  position: relative;
}

.drag-handle {
  height: 20px;
  min-height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: move;
  background: rgba(30, 41, 59, 0.5);
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
}

.handle-dots {
  display: flex;
  gap: 3px;
}

.handle-dots span {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: rgba(148, 163, 184, 0.4);
}

.task-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
  min-height: 0;
}

.task-list::-webkit-scrollbar {
  width: 4px;
}

.task-list::-webkit-scrollbar-track {
  background: transparent;
}

.task-list::-webkit-scrollbar-thumb {
  background: rgba(148, 163, 184, 0.3);
  border-radius: 2px;
}

.task-section {
  margin-bottom: 8px;
}

.task-section:last-child {
  margin-bottom: 0;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  font-size: 11px;
  color: #64748b;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.status-dot.active {
  background: #6366f1;
  box-shadow: 0 0 6px rgba(99, 102, 241, 0.5);
}

.status-dot.todo {
  background: #64748b;
}

.count {
  margin-left: auto;
  background: rgba(148, 163, 184, 0.15);
  padding: 1px 6px;
  border-radius: 8px;
  font-size: 10px;
}

.task-item {
  padding: 6px 8px;
  border-radius: 6px;
  margin-bottom: 4px;
  transition: all 0.2s;
  cursor: pointer;
  user-select: none;
}

.task-item.active {
  background: rgba(99, 102, 241, 0.1);
  border-left: 2px solid #6366f1;
}

.task-item.todo {
  background: rgba(148, 163, 184, 0.05);
  border-left: 2px solid transparent;
}

.task-item:hover {
  background: rgba(148, 163, 184, 0.15);
}

.task-title {
  font-size: 12px;
  color: #e2e8f0;
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 任务详情展开样式 */
.task-detail {
  margin-top: 6px;
  padding-top: 6px;
  border-top: 1px solid rgba(148, 163, 184, 0.1);
}

.detail-row {
  display: flex;
  gap: 6px;
  font-size: 11px;
  margin-bottom: 4px;
  line-height: 1.4;
}

.detail-row:last-child {
  margin-bottom: 0;
}

.detail-label {
  color: #64748b;
  flex-shrink: 0;
}

.detail-value {
  color: #94a3b8;
  word-break: break-all;
}

.detail-value.branch {
  color: #a78bfa;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 10px;
}

.more-hint {
  font-size: 11px;
  color: #6366f1;
  padding: 4px 8px;
  text-align: center;
  cursor: pointer;
  transition: color 0.2s;
}

.more-hint:hover {
  color: #818cf8;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 60px;
  color: #64748b;
  font-size: 12px;
}

.action-bar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
  padding: 6px 8px;
  background: rgba(30, 41, 59, 0.5);
  border-top: 1px solid rgba(148, 163, 184, 0.1);
  min-height: 36px;
}

.action-btn {
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: #64748b;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.action-btn:hover {
  background: rgba(148, 163, 184, 0.15);
  color: #e2e8f0;
}

.action-btn.close:hover {
  background: rgba(244, 63, 94, 0.15);
  color: #f43f5e;
}

/* 右键菜单样式 */
.context-menu {
  position: absolute;
  background: #1e293b;
  border: 1px solid rgba(148, 163, 184, 0.2);
  border-radius: 8px;
  padding: 4px;
  min-width: 120px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  z-index: 1000;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 12px;
  color: #e2e8f0;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;
}

.menu-item:hover {
  background: rgba(99, 102, 241, 0.2);
}

.menu-item svg {
  color: #94a3b8;
}
</style>
