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

// SQL模板
export interface SqlTemplate {
  id: number;
  templateText: string;
  templateHash: string;
  originalSqlSample?: string;
  tableNames: string[];
  sqlType?: string;
  businessScene?: string;
  usageCount: number;
  variantCount: number;
  isFavorite: boolean;
  createdAt?: string;
  updatedAt?: string;
}

// 整合结果
export interface ConsolidateResult {
  totalProcessed: number;
  templatesCreated: number;
  templatesUpdated: number;
  errors: string[];
}

// 分组统计
export interface TemplateGroupStats {
  byScene: GroupCount[];
  byType: GroupCount[];
  byTable: GroupCount[];
  totalTemplates: number;
  totalVariants: number;
}

export interface GroupCount {
  name: string;
  count: number;
}

// 分组类型
export type TemplateGroupBy = 'business_scene' | 'sql_type' | 'table' | null;
