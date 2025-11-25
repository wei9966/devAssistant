// SQL 分类简要信息（用于显示标签）
export interface SqlCategoryInfo {
  id: number;
  name: string;
  color: string;
}

export interface SqlRecord {
  id?: number;
  sqlText: string;
  sqlType?: string;
  databaseName?: string;
  executedAt?: string;
  executionSource?: string;
  isFavorite: boolean;
  tags?: string;
  description?: string;
  usageCount?: number;
  createdAt?: string;
  name?: string;
  // 多标签分类
  categories: SqlCategoryInfo[];
}

export interface SqlCategory {
  id?: number;
  name: string;
  description?: string;
  color?: string;
  icon?: string;
  aiPrompt?: string;  // AI 识别规则提示词
  sortOrder?: number;
  isSystem?: boolean;  // 是否是系统预设分类
  createdAt?: string;
  updatedAt?: string;
}

export interface SqlClassifyResult {
  sqlId: number;
  name: string;
  categories: string[];  // 多个分类名称
  confidence: number;
}

export type AiProvider = 'deepseek' | 'qwen';

export interface SqlAiConfig {
  provider: AiProvider;
  apiKey: string;
  baseUrl?: string;
  model?: string;
}
