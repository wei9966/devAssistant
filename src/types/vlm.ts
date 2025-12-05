// VLM 提供商枚举
export type VlmProvider =
  | 'qwen-vl'      // 通义千问VL
  | 'deepseek-vl'  // DeepSeek VL
  | 'openai'       // OpenAI GPT-4V
  | 'doubao'       // 豆包
  | 'kimi'         // Kimi/Moonshot
  | 'claude'       // Claude
  | 'custom';      // 自定义 OpenAI 兼容接口

// VLM 配置
export interface VlmConfig {
  provider: VlmProvider;
  apiKey: string;
  baseUrl?: string;           // 自定义 API 地址
  model?: string;             // 自定义模型名
  enabled: boolean;
  maxImageSize?: number;      // 最大图片尺寸 (KB)
  imageQuality?: number;      // 图片压缩质量 (0-100)
  timeout?: number;           // 请求超时 (秒)
  maxTokens?: number;         // 最大输出 token 数
}

// VLM 配置响应（API Key 已脱敏）
export interface VlmConfigResponse {
  provider: VlmProvider;
  apiKeyPreview: string;      // 只显示部分 API Key
  baseUrl?: string;
  model?: string;
  enabled: boolean;
  maxImageSize?: number;
  imageQuality?: number;
  timeout?: number;
  maxTokens?: number;         // 最大输出 token 数
}

// 提供商预设配置
export interface VlmProviderPreset {
  name: string;
  defaultBaseUrl: string;
  defaultModel: string;
  supportsBase64: boolean;
  supportsUrl: boolean;
  maxImageSize: number;
}

export const VLM_PROVIDER_PRESETS: Record<VlmProvider, VlmProviderPreset> = {
  'qwen-vl': {
    name: '通义千问VL',
    defaultBaseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    defaultModel: 'qwen-vl-plus',
    supportsBase64: true,
    supportsUrl: true,
    maxImageSize: 10240,  // 10MB
  },
  'deepseek-vl': {
    name: 'DeepSeek VL',
    defaultBaseUrl: 'https://api.deepseek.com/v1',
    defaultModel: 'deepseek-vl',
    supportsBase64: true,
    supportsUrl: false,
    maxImageSize: 4096,
  },
  'openai': {
    name: 'OpenAI GPT-4V',
    defaultBaseUrl: 'https://api.openai.com/v1',
    defaultModel: 'gpt-4-vision-preview',
    supportsBase64: true,
    supportsUrl: true,
    maxImageSize: 20480,
  },
  'doubao': {
    name: '豆包',
    defaultBaseUrl: 'https://ark.cn-beijing.volces.com/api/v3',
    defaultModel: 'doubao-vision-pro-32k',
    supportsBase64: true,
    supportsUrl: true,
    maxImageSize: 10240,
  },
  'kimi': {
    name: 'Kimi',
    defaultBaseUrl: 'https://api.moonshot.cn/v1',
    defaultModel: 'moonshot-v1-8k-vision-preview',
    supportsBase64: true,
    supportsUrl: true,
    maxImageSize: 10240,
  },
  'claude': {
    name: 'Claude',
    defaultBaseUrl: 'https://api.anthropic.com/v1',
    defaultModel: 'claude-3-sonnet-20240229',
    supportsBase64: true,
    supportsUrl: false,
    maxImageSize: 5120,
  },
  'custom': {
    name: '自定义',
    defaultBaseUrl: '',
    defaultModel: '',
    supportsBase64: true,
    supportsUrl: true,
    maxImageSize: 10240,
  },
};

// VLM 响应类型
export interface VlmResponse {
  success: boolean;
  content?: string;
  error?: string;
  tokensUsed?: number;
  durationMs?: number;
}

// VLM 图片分析输入
export interface VlmAnalyzeInput {
  imageBase64: string;
  prompt: string;
  provider?: VlmProvider;  // 可选：指定使用的提供商（不指定则使用配置的默认提供商）
}
