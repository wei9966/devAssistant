/**
 * 提示词类型枚举
 */
export enum PromptType {
  SCREENSHOT_ANALYZE = 'screenshot_analyze',
  BATCH_MERGING = 'batch_merging',
  DAILY_REPORT = 'daily_report',
  WEEKLY_REPORT = 'weekly_report',
}

/**
 * 单个提示词配置
 */
export interface PromptConfig {
  system: string;
  user: string;
}

/**
 * 提示词设置
 */
export interface PromptSettings {
  screenshot_analyze: PromptConfig;
  batch_merging: PromptConfig;
  daily_report: PromptConfig;
  weekly_report: PromptConfig;
}

/**
 * 提示词模板
 */
export interface PromptTemplate {
  name: string;
  type: PromptType;
  config: PromptConfig;
  description: string;
}
