import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export interface AppSettings {
  // 通用设置
  theme: 'light' | 'dark' | 'nord' | 'auto';
  language: 'zh-CN' | 'en-US';

  // 窗口设置
  alwaysOnTop: boolean;
  startMinimized: boolean;
  minimizeToTray: boolean;

  // 任务设置
  autoSaveContext: boolean;
  staleTaskDays: number;
  completedTasksRetentionDays: number;

  // SQL 设置
  enableClipboardMonitoring: boolean;
  sqlHistoryLimit: number;
  autoDetectSqlType: boolean;

  // 通知设置
  enableNotifications: boolean;
  notifyOnTaskComplete: boolean;
  notifyOnSqlDetected: boolean;

  // AI 设置
  enableAI: boolean;
  aiApiKey?: string;
  aiModel: string;

  // Git 设置
  enableGitIntegration: boolean;
  autoDetectBranch: boolean;
}

const DEFAULT_SETTINGS: AppSettings = {
  theme: 'auto',
  language: 'zh-CN',
  alwaysOnTop: false,
  startMinimized: false,
  minimizeToTray: true,
  autoSaveContext: true,
  staleTaskDays: 3,
  completedTasksRetentionDays: 7,
  enableClipboardMonitoring: true,
  sqlHistoryLimit: 100,
  autoDetectSqlType: true,
  enableNotifications: true,
  notifyOnTaskComplete: true,
  notifyOnSqlDetected: false,
  enableAI: false,
  aiModel: 'claude-3-sonnet-20240229',
  enableGitIntegration: true,
  autoDetectBranch: true,
};

export const useSettingsStore = defineStore('settings', () => {
  // 状态
  const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS });
  const loading = ref(false);

  // 从 localStorage 加载设置
  function loadSettings() {
    try {
      const saved = localStorage.getItem('app-settings');
      if (saved) {
        const parsed = JSON.parse(saved);
        settings.value = { ...DEFAULT_SETTINGS, ...parsed };
      }
    } catch (error) {
      console.error('加载设置失败:', error);
    }
  }

  // 保存设置到 localStorage
  function saveSettings() {
    try {
      localStorage.setItem('app-settings', JSON.stringify(settings.value));
    } catch (error) {
      console.error('保存设置失败:', error);
    }
  }

  // 更新单个设置项
  function updateSetting<K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K]
  ) {
    settings.value[key] = value;
    saveSettings();
  }

  // 批量更新设置
  function updateSettings(updates: Partial<AppSettings>) {
    settings.value = { ...settings.value, ...updates };
    saveSettings();
  }

  // 重置为默认设置
  function resetSettings() {
    settings.value = { ...DEFAULT_SETTINGS };
    saveSettings();
  }

  // 导出设置
  function exportSettings(): string {
    return JSON.stringify(settings.value, null, 2);
  }

  // 导入设置
  function importSettings(jsonString: string) {
    try {
      const imported = JSON.parse(jsonString);
      settings.value = { ...DEFAULT_SETTINGS, ...imported };
      saveSettings();
    } catch (error) {
      console.error('导入设置失败:', error);
      throw error;
    }
  }

  // 监听设置变化，自动保存
  watch(
    settings,
    () => {
      saveSettings();
    },
    { deep: true }
  );

  // 初始化时加载设置
  loadSettings();

  return {
    settings,
    loading,
    loadSettings,
    saveSettings,
    updateSetting,
    updateSettings,
    resetSettings,
    exportSettings,
    importSettings,
  };
});
