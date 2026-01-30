import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { taskApi } from '@/api/taskApi';
import type { Task, WorkContext } from '@/types/task';

export const useTaskStore = defineStore('task', () => {
  // 状态
  const tasks = ref<Task[]>([]);
  const completedTasks = ref<Task[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // 计算属性
  const activeTasks = computed(() =>
    tasks.value.filter(t => t.status === 'active')
  );

  const todoTasks = computed(() =>
    tasks.value.filter(t => t.status === 'todo')
  );

  const deferredTasks = computed(() =>
    tasks.value.filter(t => t.status === 'deferred')
  );

  // Actions
  async function loadTasks() {
    loading.value = true;
    error.value = null;
    try {
      tasks.value = await taskApi.getAllTasks();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载任务失败';
      console.error('加载任务失败:', e);
    } finally {
      loading.value = false;
    }
  }

  async function loadCompletedTasks(days: number = 7) {
    try {
      completedTasks.value = await taskApi.getCompletedTasks(days);
    } catch (e) {
      console.error('加载已完成任务失败:', e);
    }
  }

  async function createTask(
    title: string,
    description?: string,
    category: string = 'other',
    priority: number = 2,
    quadrant?: string,
    dueDate?: string,
    registeredAt?: string,
    scheduledStartTime?: string
  ) {
    loading.value = true;
    error.value = null;
    try {
      const taskId = await taskApi.createTask(title, description, category, priority, quadrant, dueDate, registeredAt, scheduledStartTime);
      await loadTasks();
      return taskId;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '创建任务失败';
      console.error('创建任务失败:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function startTask(taskId: number) {
    try {
      await taskApi.startTask(taskId);
      await loadTasks();
    } catch (e) {
      console.error('开始任务失败:', e);
      throw e;
    }
  }

  async function pauseTask(taskId: number, context?: WorkContext) {
    try {
      await taskApi.pauseTask(taskId, context);
      await loadTasks();
    } catch (e) {
      console.error('暂停任务失败:', e);
      throw e;
    }
  }

  async function deferTask(taskId: number) {
    try {
      await taskApi.deferTask(taskId);
      await loadTasks();
    } catch (e) {
      console.error('延后任务失败:', e);
      throw e;
    }
  }

  async function completeTask(taskId: number) {
    try {
      await taskApi.completeTask(taskId);
      await loadTasks();
      await loadCompletedTasks(7);
    } catch (e) {
      console.error('完成任务失败:', e);
      throw e;
    }
  }

  async function reactivateTask(taskId: number, progress: number) {
    try {
      await taskApi.reactivateTask(taskId, progress);
      await loadTasks();
      await loadCompletedTasks(7);
    } catch (e) {
      console.error('重新激活任务失败:', e);
      throw e;
    }
  }

  async function updateTask(
    taskId: number,
    updates: {
      title?: string;
      description?: string;
      category?: string;
      priority?: number;
      gitBranch?: string;
      notes?: string;
      quadrant?: string;
      dueDate?: string;
      registeredAt?: string;
      scheduledStartTime?: string;
    }
  ) {
    try {
      await taskApi.updateTask(taskId, updates);
      await loadTasks();
    } catch (e) {
      console.error('更新任务失败:', e);
      throw e;
    }
  }

  async function deleteTask(taskId: number) {
    try {
      await taskApi.deleteTask(taskId);
      await loadTasks();
    } catch (e) {
      console.error('删除任务失败:', e);
      throw e;
    }
  }

  async function checkStaleTasks(days: number = 3): Promise<Task[]> {
    try {
      return await taskApi.getStaleTasks(days);
    } catch (e) {
      console.error('检查过期任务失败:', e);
      return [];
    }
  }

  async function getCurrentBranch(): Promise<string> {
    try {
      return await taskApi.getCurrentBranch();
    } catch (e) {
      console.error('获取Git分支失败:', e);
      return '';
    }
  }

  return {
    // State
    tasks,
    completedTasks,
    loading,
    error,

    // Computed
    activeTasks,
    todoTasks,
    deferredTasks,

    // Actions
    loadTasks,
    loadCompletedTasks,
    createTask,
    startTask,
    pauseTask,
    deferTask,
    completeTask,
    reactivateTask,
    updateTask,
    deleteTask,
    checkStaleTasks,
    getCurrentBranch,
  };
});
