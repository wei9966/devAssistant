import { invoke } from '@tauri-apps/api/core';
import type { VlmConfig, VlmConfigResponse } from '@/types/vlm';

/**
 * VLM (Vision Language Model) API •„
 * /* VLM –õFÇICÓVLDeepSeek-VLOpenAIFKimiClaude
 */
export const vlmApi = {
  /**
   * ›X VLM Mn
   */
  async saveConfig(config: VlmConfig): Promise<void> {
    await invoke('vlm_save_config', {
      provider: config.provider,
      apiKey: config.apiKey,
      baseUrl: config.baseUrl,
      model: config.model,
      enabled: config.enabled,
      maxImageSize: config.maxImageSize,
      imageQuality: config.imageQuality,
      timeout: config.timeout,
    });
  },

  /**
   * ∑÷ VLM MnAPI Key Ú1O	
   */
  async getConfig(): Promise<VlmConfigResponse | null> {
    return await invoke('vlm_get_config');
  },

  /**
   * K’ VLM ﬁ•
   */
  async testConnection(): Promise<boolean> {
    return await invoke('vlm_test_connection');
  },

  /**
   * ê˛GÖπ
   * @param imageBase64 Base64 Ñ˛G
   * @param prompt ê–:Õ
   * @returns ê”úá,
   */
  async analyzeImage(imageBase64: string, prompt: string): Promise<string> {
    return await invoke('vlm_analyze_image', { imageBase64, prompt });
  },

  /**
   * ¿Â VLM /&/(
   */
  async isEnabled(): Promise<boolean> {
    return await invoke('vlm_is_enabled');
  },
};
