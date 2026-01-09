import { invoke } from '@tauri-apps/api/core';
import type { AiConfig, AiConfigResponse, AiProvider, TaskClassifyResult, TaskSummaryInput, AiLog, AiLogQuery, AiLogStats } from '@/types/ai';

export const aiApi = {
  // 保存 AI 配置
  async saveConfig(config: AiConfig): Promise<void> {
    await invoke('save_ai_config', {
      provider: config.provider,
      apiKey: config.apiKey,
      baseUrl: config.baseUrl,
      model: config.model,
      enabled: config.enabled,
      maxTokens: config.maxTokens,
    });
  },

  // 获取 AI 配置
  async getConfig(): Promise<AiConfigResponse | null> {
    return await invoke('get_ai_config');
  },

  // 测试 AI 连接
  async testConnection(): Promise<boolean> {
    return await invoke('test_ai_connection');
  },

  // 检查 AI 是否启用
  async isEnabled(): Promise<boolean> {
    return await invoke('is_ai_enabled');
  },

  // 生成工作日志
  async generateWorkLog(
    date: string,
    completedTasks: string[],
    executedSqls: string[],
    gitCommits: string[],
    taskMilestones?: Array<{ taskTitle: string; milestones: string[] }>
  ): Promise<string> {
    return await invoke('ai_generate_work_log', {
      date,
      completedTasks,
      executedSqls,
      gitCommits,
      taskMilestones: taskMilestones || null,
    });
  },

  // 润色工作日志
  async polishWorkLog(content: string): Promise<string> {
    return await invoke('ai_polish_work_log', { content });
  },

  // 通用 AI 聊天
  async chat(prompt: string, module?: string): Promise<string> {
    return await invoke('ai_chat', { prompt, module });
  },

  // 生成周报
  async generateWeeklyReport(logs: string[]): Promise<string> {
    return await invoke('ai_generate_weekly_report', { logs });
  },

  // 应用分类
  async classifyApps(
    apps: Array<{ id: string; name: string; path: string }>,
    categories: string[]
  ): Promise<Array<{ appId: string; category: string; tags: string[]; confidence: number }>> {
    return await invoke('ai_classify_apps', { apps, categories });
  },

  // 工作流推荐
  async recommendWorkflows(
    apps: Array<{ id: string; name: string; path: string }>,
    launchHistory: Array<[string, number]>
  ): Promise<Array<{ name: string; appIds: string[]; reason: string }>> {
    return await invoke('ai_recommend_workflows', { apps, launchHistory });
  },

  // 生成应用描述
  async generateAppDescription(appName: string, appPath: string): Promise<string> {
    return await invoke('ai_generate_app_description', { appName, appPath });
  },

  // AI 任务分类
  async classifyTask(title: string, description?: string, existingTags?: string[]): Promise<TaskClassifyResult> {
    return await invoke('ai_classify_task', { title, description, existingTags });
  },

  // AI 增强任务描述
  async enhanceTaskDescription(title: string, description?: string): Promise<string> {
    return await invoke('ai_enhance_task_description', { title, description });
  },

  // AI 生成子任务
  async generateSubtasks(title: string, description?: string): Promise<string[]> {
    return await invoke('ai_generate_subtasks', { title, description });
  },

  // AI 任务总结
  async summarizeTasks(tasks: TaskSummaryInput[]): Promise<string> {
    return await invoke('ai_summarize_tasks', { tasks });
  },

  // ========== AI 日志相关 ==========

  // 查询 AI 调用日志
  async getLogs(query?: AiLogQuery): Promise<AiLog[]> {
    return await invoke('get_ai_logs', {
      module: query?.module,
      status: query?.status,
      limit: query?.limit,
      offset: query?.offset,
    });
  },

  // 获取 AI 调用统计
  async getLogStats(): Promise<AiLogStats> {
    return await invoke('get_ai_log_stats');
  },

  // 清空 AI 日志
  async clearLogs(beforeDays?: number): Promise<number> {
    return await invoke('clear_ai_logs', { beforeDays });
  },
};
