import { invoke } from '@tauri-apps/api/core';
import type { PromptSettings, PromptConfig } from '@/types/prompts';

/**
 * 提示词配置 API
 * 用于管理 VLM 截图分析、批量合并等功能的提示词模板
 */
export const promptApi = {
  /**
   * 获取提示词配置
   */
  async getConfig(): Promise<PromptSettings> {
    return await invoke('get_prompt_config');
  },

  /**
   * 保存提示词配置
   */
  async saveConfig(settings: PromptSettings): Promise<void> {
    await invoke('save_prompt_config', { settings });
  },

  /**
   * 重置为默认配置
   */
  async resetConfig(): Promise<PromptSettings> {
    return await invoke('reset_prompt_config');
  },

  /**
   * 获取指定类型的提示词模板
   * @param promptType 提示词类型 (screenshot_analyze|batch_merging|daily_report|weekly_report)
   */
  async getTemplate(promptType: string): Promise<PromptConfig> {
    return await invoke('get_prompt_template', { promptType });
  },

  /**
   * 更新指定类型的提示词模板
   * @param promptType 提示词类型
   * @param config 提示词配置
   */
  async updateTemplate(promptType: string, config: PromptConfig): Promise<void> {
    await invoke('update_prompt_template', { promptType, config });
  },
};
