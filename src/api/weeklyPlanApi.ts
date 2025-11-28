import { invoke } from '@tauri-apps/api/core';
import type { WeeklyPlan } from '@/types/workLog';

export const weeklyPlanApi = {
  // 保存周计划
  async saveWeeklyPlan(
    weekKey: string,
    content: string,
    taskIds: number[],
    status: string = 'draft'
  ): Promise<WeeklyPlan> {
    return await invoke('save_weekly_plan', {
      weekKey,
      content,
      taskIds,
      status,
    });
  },

  // 获取指定周的计划
  async getWeeklyPlan(weekKey: string): Promise<WeeklyPlan | null> {
    return await invoke('get_weekly_plan', { weekKey });
  },

  // 获取所有周计划
  async getAllWeeklyPlans(): Promise<WeeklyPlan[]> {
    return await invoke('get_all_weekly_plans');
  },

  // 获取最近N周的计划
  async getRecentWeeklyPlans(limit: number): Promise<WeeklyPlan[]> {
    return await invoke('get_recent_weekly_plans', { limit });
  },

  // 更新周计划的任务列表
  async updateWeeklyPlanTasks(weekKey: string, taskIds: number[]): Promise<void> {
    await invoke('update_weekly_plan_tasks', { weekKey, taskIds });
  },

  // 更新周计划状态
  async updateWeeklyPlanStatus(weekKey: string, status: string): Promise<void> {
    await invoke('update_weekly_plan_status', { weekKey, status });
  },

  // 删除指定周的计划
  async deleteWeeklyPlan(weekKey: string): Promise<void> {
    await invoke('delete_weekly_plan', { weekKey });
  },
};
