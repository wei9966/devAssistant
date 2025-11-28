import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { weeklyPlanApi } from '@/api/weeklyPlanApi';
import type { WeeklyPlan } from '@/types/workLog';

export const useWeeklyPlanStore = defineStore('weeklyPlan', () => {
  // 状态
  const currentPlan = ref<WeeklyPlan | null>(null);
  const allPlans = ref<WeeklyPlan[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // 计算属性：获取新增的任务ID（相比原始计划）
  const addedTaskIds = computed(() => {
    if (!currentPlan.value) return [];
    const original = new Set(currentPlan.value.originalTaskIds);
    return currentPlan.value.taskIds.filter(id => !original.has(id));
  });

  // Actions
  async function saveWeeklyPlan(
    weekKey: string,
    content: string,
    taskIds: number[],
    status: string = 'draft'
  ) {
    loading.value = true;
    error.value = null;
    try {
      currentPlan.value = await weeklyPlanApi.saveWeeklyPlan(weekKey, content, taskIds, status);
      // 保存后刷新列表
      await loadAllPlans();
      return currentPlan.value;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '保存周计划失败';
      console.error('保存周计划失败:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadWeeklyPlan(weekKey: string) {
    loading.value = true;
    error.value = null;
    try {
      currentPlan.value = await weeklyPlanApi.getWeeklyPlan(weekKey);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载周计划失败';
      console.error('加载周计划失败:', e);
    } finally {
      loading.value = false;
    }
  }

  async function loadAllPlans() {
    loading.value = true;
    error.value = null;
    try {
      allPlans.value = await weeklyPlanApi.getAllWeeklyPlans();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载周计划列表失败';
      console.error('加载周计划列表失败:', e);
    } finally {
      loading.value = false;
    }
  }

  async function loadRecentPlans(limit: number = 10) {
    loading.value = true;
    error.value = null;
    try {
      allPlans.value = await weeklyPlanApi.getRecentWeeklyPlans(limit);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载最近周计划失败';
      console.error('加载最近周计划失败:', e);
    } finally {
      loading.value = false;
    }
  }

  async function updateTaskIds(weekKey: string, taskIds: number[]) {
    loading.value = true;
    error.value = null;
    try {
      await weeklyPlanApi.updateWeeklyPlanTasks(weekKey, taskIds);
      // 更新后刷新当前计划
      await loadWeeklyPlan(weekKey);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '更新任务列表失败';
      console.error('更新任务列表失败:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function updateStatus(weekKey: string, status: string) {
    loading.value = true;
    error.value = null;
    try {
      await weeklyPlanApi.updateWeeklyPlanStatus(weekKey, status);
      // 更新后刷新当前计划
      await loadWeeklyPlan(weekKey);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '更新状态失败';
      console.error('更新状态失败:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function deleteWeeklyPlan(weekKey: string) {
    loading.value = true;
    error.value = null;
    try {
      await weeklyPlanApi.deleteWeeklyPlan(weekKey);
      currentPlan.value = null;
      // 删除后刷新列表
      await loadAllPlans();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '删除周计划失败';
      console.error('删除周计划失败:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  return {
    // State
    currentPlan,
    allPlans,
    loading,
    error,

    // Computed
    addedTaskIds,

    // Actions
    saveWeeklyPlan,
    loadWeeklyPlan,
    loadAllPlans,
    loadRecentPlans,
    updateTaskIds,
    updateStatus,
    deleteWeeklyPlan,
  };
});
