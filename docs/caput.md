# DevAssistant 屏幕上下文采集功能

## 开发设计文档

版本: 1.0 | 日期: 2025年12月

------

## 目录

1. 项目概述
2. 功能设计
3. 技术架构
4. **VLM 服务可配置设计** (新增)
5. **日报周报功能整合方案** (新增)
6. 数据库设计
7. 接口设计
8. 开发计划
9. 风险与注意事项

------

## 1. 项目概述

### 1.1 背景与目标

DevAssistant 是一款个人开发效率工具，目前已具备任务管理、SQL历史、工作日志、剪贴板监控等功能。为了进一步提升 AI 生成工作日志的精准度，计划新增「屏幕上下文采集」功能。

该功能通过定时截图并使用视觉语言模型(VLM)理解屏幕内容，自动记录用户的工作活动，从而生成更加准确、详细的工作日报。

### 1.2 核心目标

- 定时采集屏幕截图，记录用户工作轨迹
- 使用 VLM 模型理解截图内容，提取关键信息
- 智能去重，避免存储大量重复截图
- 与现有工作日志模块深度整合
- 本地存储优先，保护用户隐私

### 1.3 现有技术栈

| 层级   | 技术                          |
| ------ | ----------------------------- |
| 前端   | Vue 3 + TypeScript + Naive UI |
| 后端   | Rust + Tauri 2                |
| 数据库 | SQLite (本地存储)             |
| AI服务 | DeepSeek、通义千问            |

------

## 2. 功能设计

### 2.1 功能模块划分

| 模块       | 功能               | 说明                       |
| ---------- | ------------------ | -------------------------- |
| 截图采集   | 定时屏幕截图       | 可配置间隔，默认10秒       |
| 变化检测   | 图片相似度对比     | 感知哈希算法，避免重复     |
| VLM理解    | 截图内容识别       | 提取应用名、操作、关键文字 |
| 上下文存储 | 结构化存储         | 时间戳+摘要+元数据         |
| 日志增强   | 基于上下文生成日报 | 增强现有工作日志功能       |
| 隐私保护   | 敏感内容过滤       | 可配置排除的应用/窗口      |

### 2.2 用户交互流程

1. 用户在设置中开启「屏幕上下文采集」功能
2. 配置采集间隔、排除应用列表、存储保留天数
3. 系统在后台静默采集，用户无感知
4. 用户可在「上下文浏览」页面查看采集记录
5. 在工作日志模块，点击「基于上下文生成日报」
6. AI 基于当日采集的上下文自动生成结构化日报

### 2.3 配置项设计

| 配置项               | 类型        | 默认值 | 说明                   |
| -------------------- | ----------- | ------ | ---------------------- |
| capture_enabled      | bool        | false  | 是否启用采集           |
| capture_interval     | u32         | 10     | 采集间隔(秒)，建议5-30 |
| similarity_threshold | f32         | 0.95   | 相似度阈值，超过则跳过 |
| retention_days       | u32         | 7      | 数据保留天数           |
| excluded_apps        | Vec<String> | []     | 排除的应用列表         |
| save_screenshots     | bool        | false  | 是否保存原始截图       |

### 2.4 活动类型分类

VLM 分析截图后，将活动归类为以下类型：

| 类型          | 典型应用              | 识别特征               |
| ------------- | --------------------- | ---------------------- |
| coding        | VS Code, IDEA, 终端   | 代码编辑器、终端命令行 |
| browsing      | Chrome, Edge, Firefox | 浏览器界面、网页内容   |
| document      | Word, Excel, WPS      | 文档编辑、表格处理     |
| meeting       | 腾讯会议, Zoom, 钉钉  | 视频会议界面           |
| communication | 微信, QQ, 企业微信    | 聊天窗口、消息列表     |
| other         | 其他应用              | 无法归类的活动         |

------

## 3. 技术架构

### 3.1 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                    Vue 3 前端                            │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────┐   │
│  │ContextSettings│ │ContextBrowser│ │ EnhancedLogGen │   │
│  └─────────────┘ └─────────────┘ └─────────────────┘   │
└────────────────────────┬────────────────────────────────┘
                         │ Tauri IPC
┌────────────────────────┴────────────────────────────────┐
│                    Rust 后端                             │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐    │
│  │ScreenCapture │ │ChangeDetector│ │ VLMProcessor │    │
│  └──────────────┘ └──────────────┘ └──────────────┘    │
│  ┌──────────────┐ ┌──────────────┐                      │
│  │ContextManager│ │ ContextStore │                      │
│  └──────────────┘ └──────────────┘                      │
└────────────────────────┬────────────────────────────────┘
                         │
                    ┌────┴────┐
                    │ SQLite  │
                    └─────────┘
```

### 3.2 核心组件职责

| 组件            | 位置      | 职责                       |
| --------------- | --------- | -------------------------- |
| ScreenCapture   | Rust 后端 | 屏幕截图、获取活动窗口信息 |
| ChangeDetector  | Rust 后端 | 图片哈希计算、相似度对比   |
| VLMProcessor    | Rust 后端 | 调用 VLM API、解析响应     |
| ContextManager  | Rust 后端 | 协调各模块、管理定时任务   |
| ContextStore    | Rust 后端 | 数据持久化、清理过期数据   |
| ContextSettings | Vue 前端  | 配置界面                   |
| ContextBrowser  | Vue 前端  | 上下文浏览、时间线展示     |
| EnhancedLogGen  | Vue 前端  | 增强的日志生成界面         |

### 3.3 数据流程

1. 定时器触发 → ScreenCapture 截取屏幕
2. ChangeDetector 计算哈希并对比，判断是否有变化
3. 若有变化 → VLMProcessor 分析截图内容
4. ContextStore 保存结构化数据到 SQLite
5. 用户请求生成日报 → 查询当日上下文 → LLM 生成日报

### 3.4 关键技术选型

| 技术点     | 方案                     | 说明                       |
| ---------- | ------------------------ | -------------------------- |
| 屏幕截图   | xcap (Rust)              | 跨平台截图库，支持多显示器 |
| 图片哈希   | img_hash (Rust)          | 感知哈希算法(pHash)        |
| VLM 服务   | 通义千问VL / DeepSeek-VL | 复用现有 AI 服务配置       |
| 异步运行时 | tokio                    | 处理定时任务和 API 调用    |
| 图片处理   | image (Rust)             | 图片格式转换、压缩         |

------

## 4. VLM 服务可配置设计

### 4.1 设计目标

为支持未来扩展更多 VLM 提供商（如 OpenAI GPT-4V、豆包、Kimi 等），需要设计一套可插拔的 VLM 服务架构。

### 4.2 支持的 VLM 提供商

| 提供商 | 模型 | API 兼容性 | 特点 |
| ------ | ---- | ---------- | ---- |
| 通义千问 | qwen-vl-plus / qwen-vl-max | OpenAI 兼容 | 国内访问快，成本低 |
| DeepSeek | deepseek-vl | OpenAI 兼容 | 性价比高 |
| OpenAI | gpt-4-vision-preview | 原生 | 效果最好，成本较高 |
| 豆包 (字节) | doubao-vision | OpenAI 兼容 | 国内服务稳定 |
| Kimi (月之暗面) | moonshot-v1-vision | OpenAI 兼容 | 长上下文支持 |
| Claude | claude-3-opus/sonnet | 原生 | 多模态能力强 |

### 4.3 VLM 配置数据结构

**前端类型定义** (`src/types/vlm.ts`):

```typescript
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
}

// 提供商预设配置
export const VLM_PROVIDER_PRESETS: Record<VlmProvider, {
  name: string;
  defaultBaseUrl: string;
  defaultModel: string;
  supportsBase64: boolean;
  supportsUrl: boolean;
  maxImageSize: number;
}> = {
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
```

### 4.4 Rust 后端 VLM 服务抽象

**VLM Provider Trait** (`src-tauri/src/services/vlm_provider.rs`):

```rust
use async_trait::async_trait;

/// VLM 提供商通用接口
#[async_trait]
pub trait VlmProvider: Send + Sync {
    /// 分析单张图片
    async fn analyze_image(
        &self,
        image_base64: &str,
        prompt: &str,
    ) -> Result<String, VlmError>;

    /// 批量分析图片
    async fn analyze_images(
        &self,
        images: Vec<&str>,
        prompt: &str,
    ) -> Result<String, VlmError>;

    /// 测试连接
    async fn test_connection(&self) -> Result<bool, VlmError>;

    /// 获取提供商信息
    fn provider_info(&self) -> ProviderInfo;
}

/// 提供商信息
pub struct ProviderInfo {
    pub name: String,
    pub supports_base64: bool,
    pub supports_url: bool,
    pub max_image_size: usize,
}

/// VLM 错误类型
#[derive(Debug)]
pub enum VlmError {
    NetworkError(String),
    AuthError(String),
    RateLimitError(String),
    ImageTooLarge(usize, usize),
    InvalidResponse(String),
    ProviderError(String),
}
```

**OpenAI 兼容实现** (`src-tauri/src/services/vlm_openai_compatible.rs`):

```rust
/// OpenAI 兼容的 VLM 实现 (适用于大多数提供商)
pub struct OpenAiCompatibleVlm {
    config: VlmConfig,
    client: reqwest::Client,
}

#[async_trait]
impl VlmProvider for OpenAiCompatibleVlm {
    async fn analyze_image(
        &self,
        image_base64: &str,
        prompt: &str,
    ) -> Result<String, VlmError> {
        let messages = vec![
            serde_json::json!({
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": prompt
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", image_base64)
                        }
                    }
                ]
            })
        ];

        let response = self.client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&serde_json::json!({
                "model": self.config.model,
                "messages": messages,
                "max_tokens": 1000,
            }))
            .send()
            .await
            .map_err(|e| VlmError::NetworkError(e.to_string()))?;

        // 解析响应...
        Ok(result)
    }
}
```

### 4.5 VLM 服务工厂

```rust
/// VLM 服务工厂 - 根据配置创建对应的提供商实例
pub struct VlmServiceFactory;

impl VlmServiceFactory {
    pub fn create(config: &VlmConfig) -> Box<dyn VlmProvider> {
        match config.provider.as_str() {
            "qwen-vl" | "deepseek-vl" | "openai" | "doubao" | "kimi" | "custom" => {
                Box::new(OpenAiCompatibleVlm::new(config.clone()))
            }
            "claude" => {
                Box::new(ClaudeVlm::new(config.clone()))
            }
            _ => {
                // 默认使用 OpenAI 兼容模式
                Box::new(OpenAiCompatibleVlm::new(config.clone()))
            }
        }
    }
}
```

### 4.6 配置界面设计

前端设置页面需要提供以下配置选项：

```
┌─────────────────────────────────────────────────┐
│  VLM 服务配置                                    │
├─────────────────────────────────────────────────┤
│  提供商:  [▼ 通义千问VL        ]                 │
│                                                  │
│  API Key: [************************]             │
│                                                  │
│  自定义配置 (可选):                              │
│  ├─ Base URL: [                    ]             │
│  └─ 模型名称: [                    ]             │
│                                                  │
│  高级选项:                                        │
│  ├─ 最大图片尺寸: [5] MB                         │
│  ├─ 图片压缩质量: [80] %                         │
│  └─ 请求超时:     [30] 秒                        │
│                                                  │
│  [测试连接]              [保存配置]              │
└─────────────────────────────────────────────────┘
```

### 4.7 新增提供商扩展指南

添加新的 VLM 提供商只需：

1. 在 `VlmProvider` 枚举中添加新类型
2. 在 `VLM_PROVIDER_PRESETS` 中添加预设配置
3. 如果 API 格式特殊，实现新的 `VlmProvider` trait

------

## 5. 日报周报功能整合方案

### 5.1 现有日报功能分析

当前工作日志模块的生成流程：

```
用户点击"AI生成日志"
       ↓
获取当天完成的任务列表
  ├─ 从 taskStore 获取最近7天任务
  ├─ 过滤当天完成的任务
  └─ 提取任务标题和标签
       ↓
调用 aiApi.generateWorkLog()
       ↓
AI 生成 Markdown 格式日志
       ↓
显示在编辑框，用户可编辑保存
```

**现有数据源**：
- 已完成的任务 (tasks 表)
- 执行过的 SQL (可选)
- Git 提交记录 (可选)

### 5.2 整合方案：屏幕上下文增强日报

#### 5.2.1 增强后的数据流

```
                    ┌──────────────────┐
                    │   屏幕上下文采集   │
                    │  (新增功能模块)    │
                    └────────┬─────────┘
                             │
                             ▼
┌─────────────┐    ┌──────────────────┐    ┌─────────────┐
│  任务系统   │───▶│    日报生成引擎   │◀───│  SQL历史   │
└─────────────┘    └────────┬─────────┘    └─────────────┘
                             │
                             ▼
                    ┌──────────────────┐
                    │   增强版工作日报   │
                    └──────────────────┘
```

#### 5.2.2 上下文数据聚合

生成日报时，从 `screen_contexts` 表提取以下信息：

```typescript
interface DayContextSummary {
  // 时间统计
  activeTimeRange: {
    start: string;        // 最早活动时间
    end: string;          // 最晚活动时间
    totalMinutes: number; // 总活跃时长
  };

  // 应用使用统计
  appUsage: {
    appName: string;
    minutes: number;
    percentage: number;
  }[];

  // 活动类型分布
  activityDistribution: {
    coding: number;       // 编码时间占比
    browsing: number;     // 浏览时间占比
    document: number;     // 文档时间占比
    meeting: number;      // 会议时间占比
    communication: number;// 沟通时间占比
    other: number;        // 其他时间占比
  };

  // 关键活动摘要 (VLM 提取的描述)
  keyActivities: {
    time: string;
    description: string;
    appName: string;
  }[];
}
```

#### 5.2.3 增强版 AI Prompt

```
请根据以下信息生成专业的工作日报：

## 日期
{date}

## 完成的任务
{completed_tasks}

## 屏幕活动上下文
- 工作时段: {start_time} - {end_time}，共 {total_hours} 小时
- 应用使用分布:
  {app_usage_list}
- 活动类型占比:
  - 编码开发: {coding_percent}%
  - 网页浏览: {browsing_percent}%
  - 文档处理: {document_percent}%
  - 会议沟通: {meeting_percent}%

## 关键活动记录
{key_activities}

## 生成要求
1. 结合任务完成情况和屏幕活动，生成真实准确的日报
2. 按工作流程组织内容，使用 ### 分组
3. 突出重点工作成果和技术细节
4. 如果检测到大量编码活动，详细描述开发内容
5. 使用 Markdown 格式，简洁专业
```

### 5.3 前端整合方案

#### 5.3.1 WorkLog.vue 界面改造

在现有日报页面添加「上下文增强」选项：

```vue
<template>
  <div class="worklog-page">
    <!-- 现有日期选择器 -->
    <n-date-picker v-model:value="selectedDate" />

    <!-- 新增：上下文增强开关 -->
    <div class="context-enhance-section">
      <n-switch v-model:value="useScreenContext" />
      <span>使用屏幕上下文增强日报</span>

      <!-- 上下文预览面板 -->
      <div v-if="useScreenContext && contextSummary" class="context-preview">
        <h4>今日屏幕活动概览</h4>
        <div class="stats-row">
          <div class="stat-item">
            <span class="label">活动时长</span>
            <span class="value">{{ contextSummary.totalHours }}h</span>
          </div>
          <div class="stat-item">
            <span class="label">主要应用</span>
            <span class="value">{{ contextSummary.topApp }}</span>
          </div>
          <div class="stat-item">
            <span class="label">采集记录</span>
            <span class="value">{{ contextSummary.totalCount }}条</span>
          </div>
        </div>

        <!-- 活动类型饼图 -->
        <div class="activity-chart">
          <PieChart :data="activityDistribution" />
        </div>
      </div>
    </div>

    <!-- 生成按钮 -->
    <n-button @click="handleGenerateWithContext">
      {{ useScreenContext ? '基于上下文生成日报' : 'AI生成日报' }}
    </n-button>

    <!-- 现有编辑器 -->
    <MarkdownEditor v-model="logContent" />
  </div>
</template>
```

#### 5.3.2 新增 API 接口

```typescript
// src/api/workLogApi.ts (扩展)

export const workLogApi = {
  // ...现有方法...

  // 基于上下文生成日报
  async generateWithContext(date: string): Promise<string> {
    return await invoke('work_log_generate_with_context', { date });
  },

  // 获取当日上下文摘要
  async getContextSummary(date: string): Promise<DayContextSummary> {
    return await invoke('context_get_day_summary', { date });
  },
};
```

### 5.4 周报功能整合

#### 5.4.1 周报增强数据

```typescript
interface WeekContextSummary {
  // 每日活动时长趋势
  dailyActiveHours: {
    date: string;
    hours: number;
  }[];

  // 本周应用使用排行
  weeklyAppRanking: {
    appName: string;
    totalMinutes: number;
    trend: 'up' | 'down' | 'stable';
  }[];

  // 本周活动类型总览
  weeklyActivitySummary: {
    type: string;
    totalMinutes: number;
    dailyAverage: number;
  }[];

  // 工作节奏分析
  workPatterns: {
    mostProductiveHour: number;    // 最高效时段
    averageStartTime: string;      // 平均开始时间
    averageEndTime: string;        // 平均结束时间
  };
}
```

#### 5.4.2 周报生成 Prompt

```
请根据以下本周工作信息生成周报：

## 本周完成的任务
{weekly_completed_tasks}

## 本周屏幕活动统计
- 本周总工作时长: {total_hours} 小时
- 日均工作时长: {avg_daily_hours} 小时
- 最高效时段: {most_productive_hour}:00
- 工作时间: {avg_start} - {avg_end}

## 应用使用排行
{app_ranking}

## 活动类型分布
{activity_distribution}

## 生成要求
1. 总结本周主要工作成果
2. 分析工作效率和时间分配
3. 列出遇到的问题和解决方案
4. 规划下周工作重点
5. 使用 Markdown 格式
```

### 5.5 整合后的用户流程

#### 5.5.1 日报生成流程

```
1. 用户进入「工作日志」页面
2. 选择日期
3. 系统自动加载该日的:
   - 已完成任务
   - 屏幕上下文摘要 (如果有)
4. 用户选择是否使用上下文增强
5. 点击生成按钮
6. AI 综合所有数据源生成日报
7. 用户编辑/润色后保存
```

#### 5.5.2 周报生成流程

```
1. 用户切换到「周报」模式
2. 选择周范围
3. 系统加载:
   - 本周完成的任务
   - 本周屏幕活动统计
   - 工作节奏分析
4. 显示可视化图表 (活动趋势、应用分布)
5. 点击生成周报
6. AI 生成包含数据洞察的周报
7. 用户编辑后保存
```

### 5.6 数据库关联设计

```sql
-- 在生成日报时，记录使用了哪些上下文
ALTER TABLE work_logs ADD COLUMN context_ids TEXT;  -- JSON数组，关联的screen_contexts.id

-- 示例查询：获取日报关联的所有上下文
SELECT sc.*
FROM screen_contexts sc
JOIN work_logs wl ON json_each.value = sc.id
WHERE wl.date = '2025-12-04'
  AND json_valid(wl.context_ids);
```

------

## 6. 数据库设计

### 6.1 screen_contexts 表

存储每次采集的屏幕上下文信息：

| 字段               | 类型       | 说明                                 |
| ------------------ | ---------- | ------------------------------------ |
| id                 | INTEGER PK | 主键，自增                           |
| captured_at        | DATETIME   | 采集时间                             |
| app_name           | TEXT       | 应用名称                             |
| window_title       | TEXT       | 窗口标题                             |
| activity_type      | TEXT       | 活动类型: coding/browsing/document等 |
| description        | TEXT       | AI 生成的活动描述                    |
| key_content        | TEXT       | 提取的关键内容                       |
| screenshot_hash    | TEXT       | 截图哈希值(用于去重)                 |
| screenshot_path    | TEXT       | 截图保存路径(可选)                   |
| processing_time_ms | INTEGER    | 处理耗时(毫秒)                       |

### 6.2 context_settings 表

存储上下文采集的配置项（键值对形式）：

| 字段       | 类型        | 说明       |
| ---------- | ----------- | ---------- |
| id         | INTEGER PK  | 主键，自增 |
| key        | TEXT UNIQUE | 配置键名   |
| value      | TEXT        | 配置值     |
| updated_at | DATETIME    | 更新时间   |

### 6.3 daily_summaries 表

存储生成的日报摘要（可选，用于缓存）：

| 字段              | 类型        | 说明                 |
| ----------------- | ----------- | -------------------- |
| id                | INTEGER PK  | 主键，自增           |
| summary_date      | DATE UNIQUE | 日期                 |
| total_contexts    | INTEGER     | 当日采集的上下文数量 |
| app_stats         | TEXT (JSON) | 应用使用统计         |
| activity_timeline | TEXT (JSON) | 活动时间线           |
| ai_summary        | TEXT        | AI 生成的日报内容    |

### 6.4 索引设计

- `screen_contexts(captured_at)` - 按时间查询
- `screen_contexts(app_name)` - 按应用统计
- `screen_contexts(activity_type)` - 按活动类型过滤

------

## 7. 接口设计

### 7.1 Tauri Commands (前后端通信)

| 命令                     | 参数            | 返回                |
| ------------------------ | --------------- | ------------------- |
| context_start_capture    | 无              | Result<(), String>  |
| context_stop_capture     | 无              | Result<(), String>  |
| context_get_status       | 无              | CaptureStatus       |
| context_get_settings     | 无              | ContextSettings     |
| context_update_settings  | ContextSettings | Result<(), String>  |
| context_list_by_date     | date: String    | Vec<ScreenContext>  |
| context_get_stats        | date: String    | DayStats            |
| context_generate_summary | date: String    | String (日报内容)   |
| context_delete_by_date   | date: String    | Result<u32, String> |
| context_cleanup_old      | 无              | Result<u32, String> |

### 7.2 关键数据结构

**ScreenContext**

| 字段          | 类型           | 说明             |
| ------------- | -------------- | ---------------- |
| id            | i64            | 记录ID           |
| captured_at   | String         | 采集时间 ISO8601 |
| app_name      | Option<String> | 应用名           |
| window_title  | Option<String> | 窗口标题         |
| activity_type | String         | 活动类型         |
| description   | String         | 活动描述         |
| key_content   | Option<String> | 关键内容         |

**DayStats**

| 字段                  | 类型                | 说明         |
| --------------------- | ------------------- | ------------ |
| total_count           | u32                 | 总采集数     |
| app_distribution      | HashMap<String,u32> | 应用使用分布 |
| activity_distribution | HashMap<String,u32> | 活动类型分布 |
| time_range            | (String, String)    | 活动时间范围 |

------

## 8. 开发计划

### 8.1 阶段划分

| 阶段 | 内容                | 产出               | 预估工时 |
| ---- | ------------------- | ------------------ | -------- |
| P1   | 截图采集 + 变化检测 | 能定时截图并去重   | 1-2天    |
| P2   | VLM 集成            | 能分析截图内容     | 1天      |
| P3   | 数据存储 + 清理     | 完整的数据管理     | 1天      |
| P4   | 前端设置界面        | 可配置功能         | 1天      |
| P5   | 上下文浏览界面      | 可查看采集记录     | 1-2天    |
| P6   | 日报生成增强        | 基于上下文生成日报 | 1天      |
| P7   | 测试 + 优化         | 稳定可用           | 1-2天    |

**总预估工时: 7-11 天**

### 8.2 MVP 最小可行版本

建议先完成 P1-P3，实现核心功能，验证可行性：

- 定时截图 + 变化检测（避免重复）
- VLM 分析截图内容
- 存储到 SQLite
- 在现有日志模块加一个按钮：基于今日上下文生成日报

### 8.3 Rust 依赖清单

| crate    | 版本    | 用途                |
| -------- | ------- | ------------------- |
| xcap     | 0.0.11+ | 跨平台屏幕截图      |
| image    | 0.25+   | 图片处理、格式转换  |
| img_hash | 4.0+    | 感知哈希算法        |
| tokio    | 1.0+    | 异步运行时（已有）  |
| reqwest  | 0.11+   | HTTP 客户端（已有） |
| base64   | 0.21+   | 图片编码            |

------

## 9. 风险与注意事项

### 9.1 隐私安全

- 所有数据本地存储，不上传到云端（仅 VLM API 调用时传输截图）
- 提供排除应用列表，避免采集敏感应用（如密码管理器）
- 默认不保存原始截图，仅保存 AI 提取的文字摘要
- 自动清理过期数据，避免长期积累

### 9.2 性能考量

- 截图采集在后台线程进行，不阻塞 UI
- VLM 调用异步处理，失败时自动重试或跳过
- 图片压缩后再发送 API，减少网络开销
- 批量处理：队列积压时合并处理

### 9.3 存储管理

- 如果保存截图，每天约 500MB-1GB（取决于间隔）
- 仅保存摘要时，每天约 1-5MB
- 建议默认保留 7 天，可配置
- 提供手动清理功能

### 9.4 API 成本

- 通义千问 VL: 约 0.008 元/张图片
- 假设 10 秒间隔，8 小时工作，每天约 2880 次调用
- 去重后实际约 500-1000 次，日成本约 4-8 元
- 可通过增大间隔或提高相似度阈值降低成本

### 9.5 系统权限

- **macOS**: 需要屏幕录制权限（System Preferences → Privacy）
- **Windows**: 通常无需额外权限
- **Linux**: 可能需要 X11 或 Wayland 相关配置

------

## 附录: 参考资源

- xcap 文档: https://github.com/aspect-apps/xcap
- 通义千问 VL API: https://help.aliyun.com/document_detail/2400395.html
- MineContext 项目（参考）: https://github.com/volcengine/MineContext
- Tauri 2 文档: https://v2.tauri.app/

------

*— 文档结束 —*