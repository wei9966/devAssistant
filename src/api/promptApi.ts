import { invoke } from '@tauri-apps/api/core';
import type { AiPrompt, RenderedPrompt, PromptUpdateParams } from '@/types/ai';

export const promptApi = {
  // 获取所有提示词
  async getAllPrompts(): Promise<AiPrompt[]> {
    return await invoke('get_all_prompts');
  },

  // 获取单个提示词
  async getPrompt(promptKey: string): Promise<AiPrompt> {
    return await invoke('get_prompt', { promptKey });
  },

  // 按模块获取提示词
  async getPromptsByModule(module: string): Promise<AiPrompt[]> {
    return await invoke('get_prompts_by_module', { module });
  },

  // 更新提示词
  async updatePrompt(promptKey: string, params: PromptUpdateParams): Promise<void> {
    await invoke('update_prompt', {
      promptKey,
      systemPrompt: params.systemPrompt,
      userPrompt: params.userPrompt,
      enabled: params.enabled ?? true,
    });
  },

  // 重置单个提示词
  async resetPrompt(promptKey: string): Promise<void> {
    await invoke('reset_prompt', { promptKey });
  },

  // 重置所有提示词
  async resetAllPrompts(): Promise<void> {
    await invoke('reset_all_prompts');
  },

  // 渲染提示词预览
  async renderPromptPreview(
    promptKey: string,
    variables: Record<string, string>
  ): Promise<RenderedPrompt> {
    return await invoke('render_prompt_preview', { promptKey, variables });
  },

  // 获取提示词变量列表
  async getPromptVariables(promptKey: string): Promise<string[]> {
    return await invoke('get_prompt_variables', { promptKey });
  },

  // 刷新缓存
  async refreshCache(): Promise<void> {
    await invoke('refresh_prompt_cache');
  },
};
