import { defineStore } from 'pinia';
import { ref } from 'vue';
import { workLogApi } from '@/api/workLogApi';
import type { WorkLog } from '@/types/workLog';

export const useWorkLogStore = defineStore('workLog', () => {
  // 状态
  const currentLog = ref<WorkLog | null>(null);
  const recentLogs = ref<WorkLog[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Actions
  async function saveWorkLog(
    date: string,
    logType: string = 'daily',
    content: string,
    aiGenerated: boolean = false
  ) {
    loading.value = true;
    error.value = null;
    try {
      await workLogApi.saveWorkLog(date, logType, content, aiGenerated);
      // 保存后刷新数据
      await loadWorkLog(date);
      await loadRecentLogs(7);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '保存工作日志失败';
      console.error('保存工作日志失败:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadWorkLog(date: string) {
    loading.value = true;
    error.value = null;
    try {
      currentLog.value = await workLogApi.getWorkLog(date);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载工作日志失败';
      console.error('加载工作日志失败:', e);
    } finally {
      loading.value = false;
    }
  }

  async function loadRecentLogs(days: number = 7) {
    loading.value = true;
    error.value = null;
    try {
      recentLogs.value = await workLogApi.getRecentWorkLogs(days);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载最近日志失败';
      console.error('加载最近日志失败:', e);
    } finally {
      loading.value = false;
    }
  }

  async function deleteWorkLog(date: string) {
    loading.value = true;
    error.value = null;
    try {
      await workLogApi.deleteWorkLog(date);
      // 删除后刷新数据
      currentLog.value = null;
      await loadRecentLogs(7);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '删除工作日志失败';
      console.error('删除工作日志失败:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadAllLogs() {
    loading.value = true;
    error.value = null;
    try {
      recentLogs.value = await workLogApi.getAllWorkLogs();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载所有日志失败';
      console.error('加载所有日志失败:', e);
    } finally {
      loading.value = false;
    }
  }

  return {
    // State
    currentLog,
    recentLogs,
    loading,
    error,

    // Actions
    saveWorkLog,
    loadWorkLog,
    loadRecentLogs,
    deleteWorkLog,
    loadAllLogs,
  };
});
