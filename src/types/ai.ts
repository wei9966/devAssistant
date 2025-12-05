// AI 提供商类型
export type AiProvider = 'deepseek' | 'qwen' | 'custom';

// AI 配置
export interface AiConfig {
  provider: AiProvider;
  apiKey: string;
  baseUrl?: string;
  model?: string;
  enabled: boolean;
}

// AI 配置响应（API Key 已脱敏）
export interface AiConfigResponse {
  provider: AiProvider;
  apiKeyPreview: string;  // 只显示部分 API Key，如 "sk-***abc"
  baseUrl?: string;
  model?: string;
  enabled: boolean;
}

// AI 提供商信息
export interface AiProviderInfo {
  id: AiProvider;
  name: string;
  defaultUrl: string;
  defaultModel: string;
}

// AI 提供商列表
export const AI_PROVIDERS: AiProviderInfo[] = [
  {
    id: 'deepseek',
    name: 'DeepSeek',
    defaultUrl: 'https://api.deepseek.com',
    defaultModel: 'deepseek-chat',
  },
  {
    id: 'qwen',
    name: '通义千问 (Qwen)',
    defaultUrl: 'https://dashscope.aliyuncs.com/compatible-mode',
    defaultModel: 'qwen-turbo',
  },
  {
    id: 'custom',
    name: '自定义 (OpenAI 兼容)',
    defaultUrl: '',
    defaultModel: '',
  },
];

// 任务分类结果
export interface TaskClassifyResult {
  category: 'backend' | 'database' | 'feature' | 'docs' | 'other';
  priority: 1 | 2 | 3;
  quadrant: 'urgent_important' | 'urgent_not_important' | 'not_urgent_important' | 'not_urgent_not_important';
  suggestedTags: string[];
  confidence: number;
}

// 应用分类输入
export interface AppClassifyInput {
  id: string;
  name: string;
  path: string;
  currentCategory?: string;
}

// 应用分类结果
export interface AppClassifyResult {
  appId: string;
  category: string;
  tags: string[];
  confidence: number;
}

// 工作流推荐
export interface WorkflowRecommendation {
  name: string;
  appIds: string[];
  reason: string;
}

// 任务总结输入
export interface TaskSummaryInput {
  title: string;
  status: string;
  completedAt?: string;
}

// AI 调用日志
export interface AiLog {
  id: number;
  module: string;
  action: string;
  provider: string;
  model?: string;
  prompt: string;
  response?: string;
  tokensUsed?: number;
  durationMs?: number;
  status: 'success' | 'error';
  errorMessage?: string;
  createdAt: number;
}

// AI 日志查询参数
export interface AiLogQuery {
  module?: string;
  status?: string;
  limit?: number;
  offset?: number;
}

// AI 日志统计
export interface AiLogStats {
  totalCalls: number;
  successCount: number;
  errorCount: number;
  totalTokens: number;
  avgDurationMs: number;
  callsByModule: [string, number][];
}
