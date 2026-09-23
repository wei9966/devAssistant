import { invoke } from '@tauri-apps/api/core';
import type { TaskCategoryDefinition } from '@/types/task';

export const taskCategoryApi = {
  async getAll(): Promise<TaskCategoryDefinition[]> {
    return await invoke('get_task_categories');
  },

  async create(name: string, color: string): Promise<TaskCategoryDefinition> {
    return await invoke('create_task_category', { name, color });
  },

  async update(id: number, name: string, color: string): Promise<TaskCategoryDefinition> {
    return await invoke('update_task_category', { id, name, color });
  },

  async setHidden(id: number, isHidden: boolean): Promise<TaskCategoryDefinition> {
    return await invoke('set_task_category_hidden', { id, isHidden });
  },

  async delete(id: number): Promise<void> {
    await invoke('delete_task_category', { id });
  },
};
