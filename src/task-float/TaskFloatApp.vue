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
                <span class="detail-value description">{{ task.description }}</span>
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
                <span class="detail-value description">{{ task.description }}</span>
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
  await emit('task-float-hidden');
  await currentWindow.hide();
}
</script>

<style scoped>
/* 赛博朋克/极客风格优化 */
.task-float-wrapper {
  width: 100%;
  height: 100%;
  /* 深色半透明背景 + 模糊效果 */
  background: rgba(15, 23, 42, 0.85);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  /* 边框发光效果 */
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-family: 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  /* 强烈的阴影 */
  box-shadow: 
    0 0 0 1px rgba(0, 0, 0, 0.2),
    0 20px 40px rgba(0, 0, 0, 0.6),
    0 0 20px rgba(99, 102, 241, 0.1);
  color: #e2e8f0;
  transition: all 0.3s ease;
}

.drag-handle {
  height: 28px;
  min-height: 28px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: move;
  /* 顶部渐变条 */
  background: linear-gradient(90deg, rgba(99, 102, 241, 0.1), rgba(167, 139, 250, 0.1));
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.handle-dots {
  display: flex;
  gap: 4px;
  opacity: 0.8;
}

.handle-dots span {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: #818cf8;
  box-shadow: 0 0 6px rgba(99, 102, 241, 0.6);
}

.task-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  min-height: 0;
}

/* 滚动条美化 */
.task-list::-webkit-scrollbar {
  width: 4px;
}

.task-list::-webkit-scrollbar-track {
  background: transparent;
}

.task-list::-webkit-scrollbar-thumb {
  background: rgba(99, 102, 241, 0.2);
  border-radius: 2px;
}

.task-list::-webkit-scrollbar-thumb:hover {
  background: rgba(99, 102, 241, 0.4);
}

.task-section {
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 4px 8px;
  font-size: 11px;
  color: #94a3b8;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  flex-shrink: 0;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  position: relative;
}

.status-dot.active {
  background: #6366f1;
  box-shadow: 0 0 8px #6366f1;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { box-shadow: 0 0 0 0 rgba(99, 102, 241, 0.4); }
  70% { box-shadow: 0 0 0 4px rgba(99, 102, 241, 0); }
  100% { box-shadow: 0 0 0 0 rgba(99, 102, 241, 0); }
}

.status-dot.todo {
  background: #64748b;
}

.count {
  margin-left: auto;
  background: rgba(99, 102, 241, 0.1);
  color: #818cf8;
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-family: 'Consolas', monospace;
}

.task-item {
  padding: 10px;
  border-radius: 8px;
  margin-bottom: 8px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  user-select: none;
  border: 1px solid transparent;
  position: relative;
  overflow: hidden;
}

.task-item.active {
  background: rgba(99, 102, 241, 0.1);
  border-color: rgba(99, 102, 241, 0.3);
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.1);
}

.task-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: #6366f1;
  box-shadow: 0 0 8px rgba(99, 102, 241, 0.6);
}

.task-item.todo {
  background: rgba(255, 255, 255, 0.02);
  border-color: rgba(255, 255, 255, 0.03);
}

.task-item:hover {
  transform: translateY(-1px);
  background: rgba(99, 102, 241, 0.08);
  border-color: rgba(99, 102, 241, 0.2);
}

.task-title {
  font-size: 13px;
  color: #f1f5f9;
  font-weight: 500;
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-detail {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  animation: slideDown 0.2s ease-out;
}

@keyframes slideDown {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

.detail-row {
  display: flex;
  gap: 8px;
  font-size: 11px;
  margin-bottom: 4px;
  line-height: 1.5;
  align-items: flex-start;
}

.detail-label {
  color: #64748b;
  flex-shrink: 0;
  font-size: 10px;
  text-transform: uppercase;
}

.detail-value {
  color: #cbd5e1;
  flex: 1;
}

.detail-value.description {
  max-height: 100px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-word;
  padding-right: 4px;
}

.detail-value.description::-webkit-scrollbar {
  width: 3px;
}

.detail-value.description::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.02);
}

.detail-value.description::-webkit-scrollbar-thumb {
  background: rgba(99, 102, 241, 0.3);
  border-radius: 2px;
}

.detail-value.branch {
  color: #a78bfa;
  font-family: 'Consolas', monospace;
  background: rgba(167, 139, 250, 0.1);
  padding: 1px 4px;
  border-radius: 3px;
}

.more-hint {
  font-size: 11px;
  color: #6366f1;
  padding: 8px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
  border-radius: 6px;
}

.more-hint:hover {
  background: rgba(99, 102, 241, 0.1);
  color: #818cf8;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100px;
  color: #64748b;
  font-size: 12px;
  gap: 8px;
}

.empty-state::before {
  content: '';
  width: 32px;
  height: 32px;
  background: rgba(148, 163, 184, 0.1);
  border-radius: 50%;
  mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'%3E%3Cpath d='M9 11l3 3L22 4'/%3E%3Cpath d='M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11'/%3E%3C/svg%3E") no-repeat center;
  -webkit-mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'%3E%3Cpath d='M9 11l3 3L22 4'/%3E%3Cpath d='M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11'/%3E%3C/svg%3E") no-repeat center;
  background-color: currentColor;
}

.action-bar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 8px 12px;
  background: rgba(15, 23, 42, 0.6);
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(4px);
  flex-shrink: 0;
}

.action-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: rgba(255, 255, 255, 0.03);
  color: #94a3b8;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.action-btn:hover {
  background: rgba(99, 102, 241, 0.2);
  color: #fff;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(99, 102, 241, 0.2);
}

.action-btn.close:hover {
  background: rgba(244, 63, 94, 0.2);
  color: #f43f5e;
  box-shadow: 0 2px 8px rgba(244, 63, 94, 0.2);
}

/* 右键菜单优化 */
.context-menu {
  position: absolute;
  background: rgba(30, 41, 59, 0.95);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 8px;
  padding: 6px;
  min-width: 140px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
  z-index: 1000;
  animation: fadeIn 0.1s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  font-size: 12px;
  color: #e2e8f0;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.menu-item:hover {
  background: rgba(99, 102, 241, 0.2);
  color: #fff;
}

.menu-item svg {
  color: #94a3b8;
  transition: color 0.15s;
}

.menu-item:hover svg {
  color: #818cf8;
}
</style>
