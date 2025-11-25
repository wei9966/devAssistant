import { invoke } from '@tauri-apps/api/core';
import type { Task, WorkContext } from '@/types/task';

export const taskApi = {
  // 获取所有未完成任务
  async getAllTasks(): Promise<Task[]> {
    return await invoke('get_all_tasks');
  },

  // 获取已完成任务（最近 N 天）
  async getCompletedTasks(days: number): Promise<Task[]> {
    return await invoke('get_completed_tasks', { days });
  },

  // 创建新任务
  async createTask(
    title: string,
    description?: string,
    category: string = 'other',
    priority: number = 2,
    quadrant?: string
  ): Promise<number> {
    const taskId = await invoke<number>('create_task', {
      title,
      description: description || null,
      category,
      priority
    });

    // 如果指定了四象限,更新任务
    if (quadrant && taskId) {
      await invoke('update_task', {
        taskId,
        title: null,
        description: null,
        category: null,
        priority: null,
        gitBranch: null,
        notes: null,
        quadrant
      });
    }

    return taskId;
  },

  // 开始任务
  async startTask(taskId: number): Promise<void> {
    await invoke('start_task', { taskId });
  },

  // 暂停任务
  async pauseTask(taskId: number, context?: WorkContext): Promise<void> {
    await invoke('pause_task', { taskId, context: context || null });
  },

  // 延后任务
  async deferTask(taskId: number): Promise<void> {
    await invoke('defer_task', { taskId });
  },

  // 完成任务
  async completeTask(taskId: number): Promise<void> {
    await invoke('complete_task', { taskId });
  },

  // 更新任务
  async updateTask(
    taskId: number,
    updates: {
      title?: string;
      description?: string;
      category?: string;
      priority?: number;
      gitBranch?: string;
      notes?: string;
      quadrant?: string;
    }
  ): Promise<void> {
    await invoke('update_task', {
      taskId,
      title: updates.title !== undefined ? updates.title : null,
      description: updates.description !== undefined ? updates.description : null,
      category: updates.category !== undefined ? updates.category : null,
      priority: updates.priority !== undefined ? updates.priority : null,
      gitBranch: updates.gitBranch !== undefined ? updates.gitBranch : null,
      notes: updates.notes !== undefined ? updates.notes : null,
      quadrant: updates.quadrant !== undefined ? updates.quadrant : null,
    });
  },

  // 删除任务
  async deleteTask(taskId: number): Promise<void> {
    await invoke('delete_task', { taskId });
  },

  // 获取长时间未处理的任务
  async getStaleTasks(days: number): Promise<Task[]> {
    return await invoke('get_stale_tasks', { days });
  },

  // 获取当前 Git 分支（可选功能）
  async getCurrentBranch(): Promise<string> {
    try {
      return await invoke('get_current_branch');
    } catch (error) {
      return '';
    }
  },

  // 导入任务
  async importTasks(tasks: ImportTask[]): Promise<ImportResult> {
    return await invoke('import_tasks', { tasks });
  },

  // 获取导入模板
  async getImportTemplate(): Promise<string> {
    return await invoke('get_import_template');
  },
};

// 导入任务的数据结构
export interface ImportTask {
  title: string;
  description?: string;
  category?: string;
  priority?: number;
  status?: string;
  gitBranch?: string;
  createdAt?: string;
  startedAt?: string;
  completedAt?: string;
  estimatedHours?: number;
  actualHours?: number;
  notes?: string;
}

// 导入结果
export interface ImportResult {
  success: number;
  failed: number;
  errors: string[];
}
