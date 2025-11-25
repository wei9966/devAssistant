import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { aiApi } from '@/api/aiApi';
import type { AiConfig, AiConfigResponse, AiProvider } from '@/types/ai';

export const useAiStore = defineStore('ai', () => {
  // 状态
  const config = ref<AiConfigResponse | null>(null);
  const isConfigured = ref(false);
  const isEnabled = ref(false);
  const loading = ref(false);
  const testing = ref(false);

  // 计算属性
  const hasApiKey = computed(() => {
    return config.value?.apiKeyPreview && config.value.apiKeyPreview.length > 0;
  });

  const currentProvider = computed(() => {
    return config.value?.provider || 'deepseek';
  });

  // 检查 AI 是否可用
  function checkAvailable(): boolean {
    return isConfigured.value && isEnabled.value;
  }

  // 加载配置
  async function loadConfig(): Promise<void> {
    loading.value = true;
    try {
      const result = await aiApi.getConfig();
      config.value = result;
      isConfigured.value = result !== null && result.apiKeyPreview.length > 0;
      isEnabled.value = result?.enabled ?? false;
    } catch (error) {
      console.error('加载 AI 配置失败:', error);
      config.value = null;
      isConfigured.value = false;
      isEnabled.value = false;
    } finally {
      loading.value = false;
    }
  }

  // 保存配置
  async function saveConfig(newConfig: AiConfig): Promise<void> {
    loading.value = true;
    try {
      await aiApi.saveConfig(newConfig);
      // 重新加载配置以获取更新后的状态
      await loadConfig();
    } finally {
      loading.value = false;
    }
  }

  // 测试连接
  async function testConnection(): Promise<boolean> {
    testing.value = true;
    try {
      return await aiApi.testConnection();
    } finally {
      testing.value = false;
    }
  }

  // 检查启用状态
  async function checkEnabled(): Promise<boolean> {
    try {
      const enabled = await aiApi.isEnabled();
      isEnabled.value = enabled;
      return enabled;
    } catch {
      isEnabled.value = false;
      return false;
    }
  }

  return {
    // 状态
    config,
    isConfigured,
    isEnabled,
    loading,
    testing,
    // 计算属性
    hasApiKey,
    currentProvider,
    // 方法
    checkAvailable,
    loadConfig,
    saveConfig,
    testConnection,
    checkEnabled,
  };
});
