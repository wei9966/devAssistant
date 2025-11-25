# DevAssistant AI 功能开发计划

> 版本: 1.0
> 创建时间: 2025-01-25
> 状态: 待开发

---

## 一、概述

### 1.1 背景

DevAssistant 是一个开发者辅助工具，目前包含以下核心功能模块：
- **任务面板（Task Board）**：任务管理、四象限优先级、标签系统
- **应用启动器（App Launcher）**：应用管理、工作流、快速启动
- **工作日志（Work Log）**：每日工作记录
- **SQL 历史（SQL History）**：SQL 语句记录、分类管理

目前 SQL 历史模块已集成 AI 分类功能，但 AI 配置仅限于该模块。本计划旨在：
1. 将 AI 配置升级为全局配置
2. 为其他模块增加 AI 辅助功能

### 1.2 目标

- 统一的 AI 配置管理（支持 DeepSeek、通义千问）
- 各模块 AI 功能增强，提升用户效率
- 良好的用户体验和错误处理

### 1.3 支持的 AI 提供商

| 提供商 | API 地址 | 默认模型 |
|--------|----------|----------|
| DeepSeek | https://api.deepseek.com | deepseek-chat |
| 通义千问 (Qwen) | https://dashscope.aliyuncs.com/compatible-mode | qwen-turbo |

---

## 二、架构设计

### 2.1 系统架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                        Settings (设置页面)                        │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  AI 配置                                                    │  │
│  │  ├─ 提供商选择: DeepSeek / Qwen                            │  │
│  │  ├─ API Key: ********                                      │  │
│  │  ├─ Base URL: (可选，自定义代理)                             │  │
│  │  ├─ Model: (可选，自定义模型)                                │  │
│  │  ├─ 启用状态: 开/关                                         │  │
│  │  └─ [测试连接] [保存配置]                                    │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Global AI Service (全局 AI 服务)               │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Rust 后端 (src-tauri/src/services/ai_service.rs)          │  │
│  │  ├─ configure_ai()      - 配置 AI 服务                      │  │
│  │  ├─ get_ai_config()     - 获取当前配置                      │  │
│  │  ├─ test_connection()   - 测试连接                          │  │
│  │  ├─ chat_completion()   - 通用聊天接口                      │  │
│  │  └─ is_configured()     - 检查是否已配置                    │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                  │
            ┌─────────────────────┼─────────────────────┐
            ▼                     ▼                     ▼
┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐
│   Task AI       │   │  AppLauncher AI │   │   WorkLog AI    │
│                 │   │                 │   │                 │
│ ├─ 智能分类     │   │ ├─ 应用分类     │   │ ├─ 日志生成     │
│ ├─ 描述增强     │   │ ├─ 标签生成     │   │ ├─ 日志润色     │
│ ├─ 子任务生成   │   │ ├─ 工作流推荐   │   │ ├─ 周报生成     │
│ └─ 任务总结     │   │ └─ 应用推荐     │   │ └─ 工作建议     │
└─────────────────┘   └─────────────────┘   └─────────────────┘
            │                     │                     │
            └─────────────────────┼─────────────────────┘
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SQL AI (已实现，需迁移)                        │
│  ├─ SQL 智能分类（多标签）                                        │
│  ├─ SQL 命名                                                     │
│  └─ 分类规则提示词                                                │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 数据流

```
用户操作 → 前端 Store → Tauri Command → Rust Service → AI API
                                              ↓
用户界面 ← 前端 Store ← Tauri Command ← Rust Service ← AI Response
```

---

## 三、数据库设计

### 3.1 新增表：AI 配置表

```sql
-- AI 全局配置表
CREATE TABLE IF NOT EXISTS ai_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider TEXT NOT NULL,           -- 'deepseek' | 'qwen'
    api_key TEXT NOT NULL,            -- API 密钥（加密存储）
    base_url TEXT,                    -- 自定义 API 地址
    model TEXT,                       -- 自定义模型名称
    enabled INTEGER DEFAULT 1,        -- 是否启用 (0/1)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 确保只有一条配置记录
CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_config_single ON ai_config(id);
```

### 3.2 现有表变更

**无需变更**：各模块使用全局 AI 服务，不需要修改现有表结构。

---

## 四、API 设计

### 4.1 全局 AI 服务 API

#### Rust Commands (src-tauri/src/commands/ai_commands.rs)

```rust
// 保存 AI 配置
#[tauri::command]
fn save_ai_config(
    provider: String,
    api_key: String,
    base_url: Option<String>,
    model: Option<String>,
    enabled: bool
) -> Result<(), String>

// 获取 AI 配置（不返回完整 API Key）
#[tauri::command]
fn get_ai_config() -> Result<AiConfigResponse, String>

// 测试 AI 连接
#[tauri::command]
async fn test_ai_connection() -> Result<bool, String>

// 检查 AI 是否已配置且启用
#[tauri::command]
fn is_ai_enabled() -> Result<bool, String>
```

#### 前端 API (src/api/aiApi.ts)

```typescript
export const aiApi = {
  // 保存配置
  saveConfig(config: AiConfig): Promise<void>,

  // 获取配置
  getConfig(): Promise<AiConfigResponse>,

  // 测试连接
  testConnection(): Promise<boolean>,

  // 检查是否启用
  isEnabled(): Promise<boolean>,
}
```

### 4.2 任务 AI API

```rust
// 任务智能分类
#[tauri::command]
async fn ai_classify_task(
    title: String,
    description: Option<String>
) -> Result<TaskClassifyResult, String>

// 任务描述增强
#[tauri::command]
async fn ai_enhance_task_description(
    title: String,
    description: Option<String>
) -> Result<String, String>

// 生成子任务
#[tauri::command]
async fn ai_generate_subtasks(
    title: String,
    description: Option<String>
) -> Result<Vec<String>, String>

// 任务总结
#[tauri::command]
async fn ai_summarize_tasks(
    tasks: Vec<TaskSummaryInput>
) -> Result<String, String>
```

### 4.3 应用启动器 AI API

```rust
// 应用智能分类
#[tauri::command]
async fn ai_classify_apps(
    apps: Vec<AppClassifyInput>,
    categories: Vec<String>
) -> Result<Vec<AppClassifyResult>, String>

// 工作流推荐
#[tauri::command]
async fn ai_recommend_workflows(
    launch_history: Vec<LaunchHistoryItem>
) -> Result<Vec<WorkflowRecommendation>, String>

// 应用描述生成
#[tauri::command]
async fn ai_generate_app_description(
    app_name: String,
    app_path: String
) -> Result<String, String>
```

### 4.4 工作日志 AI API

```rust
// 自动生成工作日志
#[tauri::command]
async fn ai_generate_work_log(
    date: String,
    completed_tasks: Vec<TaskLogInput>,
    executed_sqls: Vec<SqlLogInput>,
    git_commits: Vec<String>
) -> Result<String, String>

// 日志润色
#[tauri::command]
async fn ai_polish_work_log(
    content: String
) -> Result<String, String>

// 生成周报
#[tauri::command]
async fn ai_generate_weekly_report(
    logs: Vec<WorkLogInput>
) -> Result<String, String>
```

---

## 五、功能详细设计

### 5.1 全局 AI 配置（设置页）

#### 界面设计

```
┌─────────────────────────────────────────────────────────┐
│  AI 智能助手配置                                          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  启用 AI 功能    [✓]                                    │
│                                                         │
│  AI 提供商                                              │
│  ┌─────────────────────────────────────────────────┐   │
│  │ ○ DeepSeek                                      │   │
│  │ ○ 通义千问 (Qwen)                               │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  API Key *                                              │
│  ┌─────────────────────────────────────────────────┐   │
│  │ ●●●●●●●●●●●●●●●●●●●●         [显示/隐藏]        │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  自定义 Base URL（可选）                                 │
│  ┌─────────────────────────────────────────────────┐   │
│  │ https://api.deepseek.com                        │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  自定义模型（可选）                                      │
│  ┌─────────────────────────────────────────────────┐   │
│  │ deepseek-chat                                   │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  [测试连接]                    [保存配置]               │
│                                                         │
│  ─────────────────────────────────────────────────────  │
│  提示：                                                 │
│  • DeepSeek API: https://platform.deepseek.com         │
│  • 通义千问 API: https://dashscope.console.aliyun.com  │
└─────────────────────────────────────────────────────────┘
```

#### 交互逻辑

1. 页面加载时获取现有配置
2. 修改后点击"保存配置"保存到数据库
3. "测试连接"验证 API 可用性
4. 关闭 AI 功能后，各模块的 AI 按钮置灰

---

### 5.2 任务面板 AI 功能

#### 5.2.1 智能分类

**触发时机**：新建任务时，输入标题后自动建议

**输入**：
- 任务标题
- 任务描述（可选）

**输出**：
```typescript
interface TaskClassifyResult {
  category: 'backend' | 'database' | 'feature' | 'docs' | 'other';
  priority: 1 | 2 | 3;
  quadrant: TaskQuadrant;
  suggestedTags: string[];
  confidence: number;
}
```

**AI Prompt 模板**：
```
你是一个任务分类助手。请分析以下任务，给出分类建议：

任务标题：{title}
任务描述：{description}

请返回 JSON 格式：
{
  "category": "backend|database|feature|docs|other",
  "priority": 1-3 (1最高),
  "quadrant": "urgent_important|urgent_not_important|not_urgent_important|not_urgent_not_important",
  "suggestedTags": ["标签1", "标签2"],
  "reason": "分类理由"
}

分类说明：
- backend: 后端开发相关
- database: 数据库相关
- feature: 功能开发
- docs: 文档相关
- other: 其他

四象限说明：
- urgent_important: 紧急且重要（立即做）
- urgent_not_important: 紧急不重要（委派或快速处理）
- not_urgent_important: 不紧急但重要（计划做）
- not_urgent_not_important: 不紧急不重要（考虑删除）
```

#### 5.2.2 任务描述增强

**触发时机**：编辑任务时点击"AI 补充"按钮

**功能**：根据标题和现有描述，补充更详细的任务说明

**AI Prompt 模板**：
```
请帮我完善以下任务的描述，使其更加清晰、具体、可执行：

任务标题：{title}
当前描述：{description}

请补充：
1. 具体的实现步骤
2. 需要注意的事项
3. 可能的技术难点
4. 验收标准

直接返回优化后的描述文本，使用 Markdown 格式。
```

#### 5.2.3 子任务生成

**触发时机**：任务详情页点击"生成子任务"

**功能**：将大任务拆分为可执行的小任务

**AI Prompt 模板**：
```
请将以下任务拆分为具体的子任务：

任务标题：{title}
任务描述：{description}

要求：
1. 每个子任务应该是可独立完成的
2. 子任务粒度适中（2-4小时可完成）
3. 子任务之间有清晰的先后顺序

返回 JSON 数组格式：
[
  {"title": "子任务1", "order": 1},
  {"title": "子任务2", "order": 2}
]
```

#### 5.2.4 任务总结

**触发时机**：统计页面点击"生成总结"

**功能**：汇总一段时间内完成的任务，生成工作报告

---

### 5.3 应用启动器 AI 功能

#### 5.3.1 应用智能分类

**触发时机**：扫描新应用后，点击"AI 自动分类"

**输入**：
```typescript
interface AppClassifyInput {
  id: string;
  name: string;
  path: string;
  currentCategory?: string;
  currentTags?: string[];
}
```

**输出**：
```typescript
interface AppClassifyResult {
  appId: string;
  category: string;
  tags: string[];
  confidence: number;
}
```

**AI Prompt 模板**：
```
你是一个应用分类助手。请根据应用名称和路径，为以下应用分配分类和标签。

可用分类：
- dev: 开发工具（IDE、编辑器、终端、数据库工具等）
- office: 办公软件（文档、表格、演示、邮件等）
- browser: 浏览器
- design: 设计工具（图片编辑、UI设计、视频编辑等）
- media: 影音娱乐（音乐、视频播放器等）
- game: 游戏
- system: 系统工具（文件管理、系统设置等）
- other: 其他

应用列表：
{apps_json}

返回 JSON 数组：
[
  {"id": "app_id", "category": "dev", "tags": ["编辑器", "前端"]}
]
```

#### 5.3.2 工作流推荐

**触发时机**：工作流管理页面点击"AI 推荐"

**功能**：分析启动历史，推荐常用应用组合

**AI Prompt 模板**：
```
根据以下应用启动历史，推荐可能的工作流组合：

启动历史（最近30天）：
{launch_history}

应用列表：
{apps_list}

请分析用户的使用习惯，推荐 3-5 个工作流组合，返回 JSON：
[
  {
    "name": "工作流名称",
    "appIds": ["app1", "app2"],
    "reason": "推荐理由"
  }
]
```

#### 5.3.3 应用描述生成

**触发时机**：编辑应用时点击"生成描述"

**功能**：根据应用名称和路径生成简短描述

---

### 5.4 工作日志 AI 功能

#### 5.4.1 自动生成工作日志

**触发时机**：工作日志页面点击"AI 生成"

**输入**：
- 当日完成的任务列表
- 当日执行的 SQL 语句
- 当日的 Git 提交记录（如果有）

**AI Prompt 模板**：
```
请根据以下信息生成今日工作日志：

日期：{date}

完成的任务：
{completed_tasks}

执行的 SQL 操作：
{sql_summary}

Git 提交记录：
{git_commits}

要求：
1. 使用简洁的语言描述工作内容
2. 按工作类型分组（开发、数据库、文档等）
3. 突出重要的成果和进展
4. 使用 Markdown 格式

生成格式：
## {date} 工作日志

### 开发工作
- ...

### 数据库工作
- ...

### 其他
- ...
```

#### 5.4.2 日志润色

**触发时机**：编辑日志时点击"AI 润色"

**功能**：优化日志内容，使其更专业、条理更清晰

#### 5.4.3 周报/月报生成

**触发时机**：日志页面选择日期范围后点击"生成报告"

**AI Prompt 模板**：
```
请根据以下工作日志生成周报：

日志内容：
{logs}

要求：
1. 总结本周主要工作成果
2. 列出遇到的问题和解决方案
3. 规划下周工作重点
4. 使用 Markdown 格式
```

---

### 5.5 SQL 历史 AI 功能（迁移优化）

#### 变更说明

1. 移除 SQL 历史页面的 AI 配置功能
2. 使用全局 AI 配置
3. 保留现有功能：
   - SQL 智能分类（多标签）
   - SQL 命名
   - 分类规则提示词

#### 新增功能

##### 5.5.1 SQL 解释

**触发时机**：选中 SQL 后点击"AI 解释"

**功能**：解释复杂 SQL 的作用和逻辑

##### 5.5.2 SQL 优化建议

**触发时机**：选中 SQL 后点击"优化建议"

**功能**：分析 SQL 性能问题并给出优化建议

---

## 六、开发计划

### 6.1 里程碑概览

| 阶段 | 内容 | 预计工作量 | 状态 |
|------|------|------------|------|
| Phase 1 | 全局 AI 配置 | 2-3天 | 待开发 |
| Phase 2 | SQL 历史迁移 | 1天 | 待开发 |
| Phase 3 | 应用启动器 AI | 2-3天 | 待开发 |
| Phase 4 | 工作日志 AI | 2-3天 | 待开发 |
| Phase 5 | 任务面板 AI | 3-4天 | 待开发 |

### 6.2 Phase 1：全局 AI 配置（基础设施）

**目标**：建立统一的 AI 服务基础设施

#### 任务清单

- [ ] **1.1 数据库迁移**
  - 创建 `ai_config` 表
  - 添加迁移脚本

- [ ] **1.2 后端服务**
  - 创建 `src-tauri/src/services/ai_service.rs`（全局 AI 服务）
  - 实现配置保存/读取
  - 实现通用聊天接口
  - 实现连接测试

- [ ] **1.3 后端命令**
  - 创建 `src-tauri/src/commands/ai_commands.rs`
  - 注册 Tauri 命令

- [ ] **1.4 前端类型**
  - 创建 `src/types/ai.ts`
  - 定义配置和响应类型

- [ ] **1.5 前端 API**
  - 创建 `src/api/aiApi.ts`
  - 封装 Tauri 调用

- [ ] **1.6 前端 Store**
  - 创建 `src/stores/aiStore.ts`
  - 管理 AI 配置状态

- [ ] **1.7 设置页面更新**
  - 在 Settings.vue 添加 AI 配置区域
  - 实现配置表单
  - 实现测试连接功能

### 6.3 Phase 2：SQL 历史迁移

**目标**：将 SQL 历史的 AI 功能迁移到使用全局配置

#### 任务清单

- [ ] **2.1 移除本地 AI 配置**
  - 删除 SqlHistory.vue 中的 AI 配置对话框
  - 删除 sqlStore 中的 AI 配置相关代码

- [ ] **2.2 使用全局服务**
  - 修改 `sql_ai_service.rs` 使用全局配置
  - 或直接使用全局 AI 服务

- [ ] **2.3 UI 调整**
  - AI 配置按钮改为跳转到设置页
  - AI 未配置时显示提示

- [ ] **2.4 新增 SQL 解释功能**（可选）

- [ ] **2.5 新增 SQL 优化建议功能**（可选）

### 6.4 Phase 3：应用启动器 AI

**目标**：为应用启动器增加 AI 辅助功能

#### 任务清单

- [ ] **3.1 后端服务**
  - 在 `ai_service.rs` 添加应用分类方法
  - 添加工作流推荐方法

- [ ] **3.2 后端命令**
  - `ai_classify_apps` - 批量分类应用
  - `ai_recommend_workflows` - 推荐工作流
  - `ai_generate_app_description` - 生成应用描述

- [ ] **3.3 前端集成**
  - AppLauncher.vue 添加 "AI 分类" 按钮
  - 工作流管理添加 "AI 推荐" 功能
  - 应用编辑添加 "生成描述" 功能

- [ ] **3.4 Store 更新**
  - appLauncherStore 添加 AI 相关方法

### 6.5 Phase 4：工作日志 AI

**目标**：为工作日志增加 AI 生成和润色功能

#### 任务清单

- [ ] **4.1 后端服务**
  - 添加日志生成方法
  - 添加日志润色方法
  - 添加周报生成方法

- [ ] **4.2 后端命令**
  - `ai_generate_work_log` - 生成工作日志
  - `ai_polish_work_log` - 润色日志
  - `ai_generate_weekly_report` - 生成周报

- [ ] **4.3 数据收集**
  - 获取当日完成任务
  - 获取当日 SQL 执行记录
  - 获取 Git 提交记录（如果集成）

- [ ] **4.4 前端集成**
  - WorkLog.vue 添加 "AI 生成" 按钮
  - 添加 "AI 润色" 按钮
  - 添加周报生成功能

### 6.6 Phase 5：任务面板 AI

**目标**：为任务管理增加 AI 辅助功能

#### 任务清单

- [ ] **5.1 后端服务**
  - 添加任务分类方法
  - 添加描述增强方法
  - 添加子任务生成方法
  - 添加任务总结方法

- [ ] **5.2 后端命令**
  - `ai_classify_task` - 任务分类建议
  - `ai_enhance_task_description` - 描述增强
  - `ai_generate_subtasks` - 生成子任务
  - `ai_summarize_tasks` - 任务总结

- [ ] **5.3 前端集成**
  - 新建任务对话框添加 AI 建议
  - 任务编辑添加 "AI 补充" 按钮
  - 任务详情添加 "生成子任务" 功能
  - 统计页面添加 "生成总结" 功能

---

## 七、技术实现细节

### 7.1 全局 AI 服务核心代码结构

```rust
// src-tauri/src/services/ai_service.rs

pub struct AiService {
    client: Client,
}

pub struct AiConfig {
    pub provider: AiProvider,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub enabled: bool,
}

pub enum AiProvider {
    DeepSeek,
    Qwen,
}

impl AiService {
    // 从数据库加载配置
    pub fn load_config(conn: &Connection) -> Result<Option<AiConfig>>;

    // 保存配置到数据库
    pub fn save_config(conn: &Connection, config: &AiConfig) -> Result<()>;

    // 通用聊天接口
    pub async fn chat(&self, config: &AiConfig, messages: Vec<Message>) -> Result<String>;

    // 测试连接
    pub async fn test_connection(&self, config: &AiConfig) -> Result<bool>;

    // === 任务相关 ===
    pub async fn classify_task(&self, config: &AiConfig, title: &str, desc: Option<&str>) -> Result<TaskClassifyResult>;
    pub async fn enhance_description(&self, config: &AiConfig, title: &str, desc: Option<&str>) -> Result<String>;
    pub async fn generate_subtasks(&self, config: &AiConfig, title: &str, desc: Option<&str>) -> Result<Vec<String>>;

    // === 应用相关 ===
    pub async fn classify_apps(&self, config: &AiConfig, apps: Vec<AppInput>, categories: Vec<String>) -> Result<Vec<AppClassifyResult>>;
    pub async fn recommend_workflows(&self, config: &AiConfig, history: Vec<LaunchHistory>) -> Result<Vec<WorkflowRecommendation>>;

    // === 日志相关 ===
    pub async fn generate_work_log(&self, config: &AiConfig, data: WorkLogInput) -> Result<String>;
    pub async fn polish_work_log(&self, config: &AiConfig, content: &str) -> Result<String>;

    // === SQL 相关（迁移） ===
    pub async fn classify_sqls(&self, config: &AiConfig, sqls: Vec<SqlInput>, categories: Vec<CategoryWithPrompt>) -> Result<Vec<SqlClassifyResult>>;
}
```

### 7.2 前端 Store 结构

```typescript
// src/stores/aiStore.ts

export const useAiStore = defineStore('ai', () => {
  // 状态
  const config = ref<AiConfig | null>(null);
  const isConfigured = ref(false);
  const isEnabled = ref(false);
  const loading = ref(false);

  // 操作
  async function loadConfig(): Promise<void>;
  async function saveConfig(newConfig: AiConfig): Promise<void>;
  async function testConnection(): Promise<boolean>;

  // 检查 AI 是否可用
  function checkAvailable(): boolean {
    return isConfigured.value && isEnabled.value;
  }

  return {
    config,
    isConfigured,
    isEnabled,
    loading,
    loadConfig,
    saveConfig,
    testConnection,
    checkAvailable,
  };
});
```

### 7.3 错误处理策略

1. **API Key 未配置**：提示用户前往设置页配置
2. **连接失败**：显示具体错误信息，提供重试选项
3. **响应解析失败**：记录日志，返回友好提示
4. **Token 限制**：自动截断过长内容
5. **Rate Limit**：显示限流提示，建议稍后重试

### 7.4 安全考虑

1. **API Key 存储**：考虑加密存储（可选，后续优化）
2. **API Key 显示**：前端显示时部分隐藏
3. **敏感数据**：不将用户敏感数据发送给 AI
4. **错误信息**：不暴露完整 API Key 到错误信息中

---

## 八、测试计划

### 8.1 单元测试

- AI 服务配置保存/读取
- Prompt 构建逻辑
- 响应解析逻辑

### 8.2 集成测试

- 各模块与 AI 服务的集成
- 全局配置变更后各模块响应

### 8.3 E2E 测试

- 完整的用户流程测试
- AI 功能开关测试

---

## 九、后续优化方向

1. **多模型支持**：增加 OpenAI、Claude 等提供商
2. **本地模型**：支持 Ollama 等本地部署模型
3. **Prompt 管理**：允许用户自定义 Prompt 模板
4. **使用统计**：统计 AI 调用次数和 Token 消耗
5. **缓存优化**：缓存相似请求的结果
6. **离线模式**：无网络时的降级处理

---

## 十、附录

### A. 相关文件清单

```
src-tauri/
├── src/
│   ├── services/
│   │   ├── ai_service.rs          # 全局 AI 服务（新建）
│   │   └── sql_ai_service.rs      # SQL AI 服务（修改）
│   ├── commands/
│   │   ├── ai_commands.rs         # AI 命令（新建）
│   │   └── sql_ai_commands.rs     # SQL AI 命令（修改）
│   └── db/
│       └── migrations.rs          # 数据库迁移（修改）

src/
├── types/
│   └── ai.ts                      # AI 类型定义（新建）
├── api/
│   └── aiApi.ts                   # AI API（新建）
├── stores/
│   └── aiStore.ts                 # AI Store（新建）
└── views/
    ├── Settings.vue               # 设置页（修改）
    ├── SqlHistory.vue             # SQL 历史（修改）
    ├── AppLauncher.vue            # 应用启动器（修改）
    ├── WorkLog.vue                # 工作日志（修改）
    └── TaskBoard.vue              # 任务面板（修改）
```

### B. 参考资源

- [DeepSeek API 文档](https://platform.deepseek.com/api-docs)
- [通义千问 API 文档](https://help.aliyun.com/zh/dashscope/developer-reference/api-details)
- [Tauri 官方文档](https://tauri.app/v1/guides/)

---

> 文档维护：每完成一个阶段后更新状态和实际工作量
