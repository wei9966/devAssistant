import { invoke } from '@tauri-apps/api/core';
import type { Task, WorkContext, TaskMilestone } from '@/types/task';

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
    quadrant?: string,
    dueDate?: string,
    registeredAt?: string,
    scheduledStartTime?: string
  ): Promise<number> {
    const taskId = await invoke<number>('create_task', {
      title,
      description: description || null,
      category,
      priority
    });

    // 如果指定了四象限或日期字段,更新任务
    if ((quadrant || dueDate || registeredAt || scheduledStartTime) && taskId) {
      await invoke('update_task', {
        taskId,
        title: null,
        description: null,
        category: null,
        priority: null,
        gitBranch: null,
        notes: null,
        quadrant: quadrant || null,
        dueDate: dueDate || null,
        registeredAt: registeredAt || null,
        displayDate: null,
        scheduledStartTime: scheduledStartTime || null
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
      dueDate?: string;
      registeredAt?: string;
      displayDate?: string;
      scheduledStartTime?: string;
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
      dueDate: updates.dueDate !== undefined ? updates.dueDate : null,
      registeredAt: updates.registeredAt !== undefined ? updates.registeredAt : null,
      displayDate: updates.displayDate !== undefined ? updates.displayDate : null,
      scheduledStartTime: updates.scheduledStartTime !== undefined ? updates.scheduledStartTime : null,
    });
  },

  // 取消任务（软删除）
  async cancelTask(taskId: number): Promise<void> {
    await invoke('cancel_task', { taskId });
  },

  // 删除任务（物理删除，保留备用）
  async deleteTask(taskId: number): Promise<void> {
    await invoke('delete_task', { taskId });
  },

  // 获取历史任务（已完成 + 已取消）
  async getHistoryTasks(days: number, statusFilter?: string): Promise<Task[]> {
    return await invoke('get_history_tasks', {
      days,
      statusFilter: statusFilter || null,
    });
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

  // 获取日期范围内的任务（用于日历显示）
  async getTasksByDateRange(startDate: string, endDate: string): Promise<Task[]> {
    return await invoke('get_tasks_by_date_range', { startDate, endDate });
  },

  // 更新任务的日历显示日期
  async updateTaskDisplayDate(taskId: number, displayDate: string): Promise<void> {
    await invoke('update_task_display_date', { taskId, displayDate });
  },

  // 将任务移至今天
  async moveTaskToToday(taskId: number): Promise<void> {
    await invoke('move_task_to_today', { taskId });
  },

  // 更新任务进度
  async updateTaskProgress(taskId: number, progress: number): Promise<void> {
    await invoke('update_task_progress', { taskId, progress });
  },

  // 创建任务里程碑
  async createTaskMilestone(
    taskId: number,
    title: string,
    description?: string,
    progressSnapshot?: number
  ): Promise<TaskMilestone> {
    return await invoke('create_task_milestone', {
      taskId,
      title,
      description: description || null,
      progressSnapshot: progressSnapshot ?? null,
    });
  },

  // 获取任务的所有里程碑
  async getTaskMilestones(taskId: number): Promise<TaskMilestone[]> {
    return await invoke('get_task_milestones', { taskId });
  },

  // 删除任务里程碑
  async deleteTaskMilestone(milestoneId: number): Promise<void> {
    await invoke('delete_task_milestone', { milestoneId });
  },

  // 获取日期范围内的里程碑（含任务标题）
  async getMilestonesByDateRange(startDate: string, endDate: string): Promise<MilestoneWithTask[]> {
    return await invoke('get_milestones_by_date_range', { startDate, endDate });
  },

  // 重新激活已完成的任务
  async reactivateTask(taskId: number, progress: number): Promise<void> {
    await invoke('reactivate_task', { taskId, progress });
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

// 带任务标题的里程碑
export interface MilestoneWithTask {
  id?: number;
  taskId: number;
  title: string;
  description?: string;
  progressSnapshot?: number;
  createdAt?: string;
  taskTitle: string;
}
