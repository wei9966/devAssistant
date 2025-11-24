import { invoke } from '@tauri-apps/api/core';
import type { Tag } from '@/types/task';

export const tagApi = {
  // 获取所有标签
  async getAllTags(): Promise<Tag[]> {
    return await invoke('get_all_tags');
  },

  // 创建标签
  async createTag(name: string, color: string): Promise<number> {
    return await invoke('create_tag', { name, color });
  },

  // 更新标签
  async updateTag(id: number, name: string, color: string): Promise<void> {
    await invoke('update_tag', { id, name, color });
  },

  // 删除标签
  async deleteTag(id: number): Promise<void> {
    await invoke('delete_tag', { id });
  },

  // 为任务添加标签
  async addTagToTask(taskId: number, tagId: number): Promise<void> {
    await invoke('add_tag_to_task', { taskId, tagId });
  },

  // 从任务移除标签
  async removeTagFromTask(taskId: number, tagId: number): Promise<void> {
    await invoke('remove_tag_from_task', { taskId, tagId });
  },

  // 批量为任务添加标签
  async addTagsToTask(taskId: number, tagIds: number[]): Promise<void> {
    await invoke('add_tags_to_task', { taskId, tagIds });
  },

  // 移除任务的所有标签
  async removeAllTagsFromTask(taskId: number): Promise<void> {
    await invoke('remove_all_tags_from_task', { taskId });
  },

  // 获取任务的所有标签
  async getTaskTags(taskId: number): Promise<Tag[]> {
    return await invoke('get_task_tags', { taskId });
  },
};
