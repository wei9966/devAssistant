# DevAssistant AI 智能体集成方案

> 版本: 1.0 | 日期: 2026-01-28 | 状态: 设计阶段

## 目录

1. [设计目标](#1-设计目标)
2. [整体架构](#2-整体架构)
3. [兼容性设计](#3-兼容性设计)
4. [工具系统设计](#4-工具系统设计)
5. [Agent 核心设计](#5-agent-核心设计)
6. [前端交互设计](#6-前端交互设计)
7. [数据流设计](#7-数据流设计)
8. [安全与限制](#8-安全与限制)
9. [实现计划](#9-实现计划)
10. [风险评估](#10-风险评估)

---

## 1. 设计目标

### 1.1 核心目标

| 目标 | 说明 |
|------|------|
| **智能化** | 用户通过自然语言即可完成复杂的多步骤操作 |
| **兼容性** | 不开启AI时，所有现有功能正常运行 |
| **渐进式** | AI功能可逐步启用，从简单到复杂 |
| **透明性** | 用户可以看到AI的思考过程和执行的每一步 |
| **可控性** | 用户可以中断、修改、确认AI的操作 |

### 1.2 典型使用场景

```
场景1: 工作整理
用户: "帮我整理今天的工作，生成日报"
Agent:
  ① 获取今日完成的任务 (3个)
  ② 获取今日执行的SQL (8条)
  ③ 获取今日番茄钟记录 (4个，共100分钟)
  ④ 生成工作日报
  ⑤ 返回日报内容，询问是否保存

场景2: 智能任务创建
用户: "下周五之前要完成用户登录功能的开发"
Agent:
  ① 解析意图：创建任务
  ② 解析截止日期：下周五 → 2026-02-06
  ③ 分析任务分类：后端开发
  ④ 推荐优先级和象限
  ⑤ 生成子任务建议
  ⑥ 创建任务并返回确认

场景3: 数据查询
用户: "这周我的工作效率怎么样？"
Agent:
  ① 获取本周时间统计
  ② 分析任务完成情况
  ③ 计算专注时长和效率指标
  ④ 生成效率分析报告
  ⑤ 可视化展示统计数据

场景4: 快速操作
用户: "开始做那个登录功能的任务"
Agent:
  ① 识别代词"那个" → 查找最近提到的登录相关任务
  ② 找到任务 #42 "用户登录功能开发"
  ③ 启动25分钟番茄钟
  ④ 返回确认信息
```

---

## 2. 整体架构

### 2.1 系统架构图

```
┌─────────────────────────────────────────────────────────────────────┐
│                         DevAssistant 应用                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                     前端 (Vue 3)                             │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │   │
│  │  │ AiChat组件  │  │ AgentPanel  │  │  其他功能模块       │  │   │
│  │  │ (现有)      │  │ (新增)      │  │  (任务/SQL/番茄等)  │  │   │
│  │  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │   │
│  │         │                │                    │              │   │
│  │  ┌──────┴────────────────┴────────────────────┴──────────┐  │   │
│  │  │              agentStore (Pinia)                        │  │   │
│  │  │  - agentEnabled    - executionSteps                   │  │   │
│  │  │  - isExecuting     - pendingConfirmation              │  │   │
│  │  └───────────────────────┬───────────────────────────────┘  │   │
│  │                          │                                   │   │
│  │  ┌───────────────────────┴───────────────────────────────┐  │   │
│  │  │              agentApi.ts (新增)                        │  │   │
│  │  │  - executeAgent()  - confirmAction()                   │  │   │
│  │  │  - cancelExecution() - getAgentStatus()               │  │   │
│  │  └───────────────────────┬───────────────────────────────┘  │   │
│  └──────────────────────────┼───────────────────────────────────┘   │
│                             │ Tauri IPC                             │
│  ┌──────────────────────────┼───────────────────────────────────┐   │
│  │                     后端 (Rust)                               │   │
│  │  ┌───────────────────────┴───────────────────────────────┐   │   │
│  │  │           agent_commands.rs (新增)                     │   │   │
│  │  │  - execute_agent   - confirm_action                    │   │   │
│  │  │  - cancel_execution - get_agent_status                │   │   │
│  │  └───────────────────────┬───────────────────────────────┘   │   │
│  │                          │                                    │   │
│  │  ┌───────────────────────┴───────────────────────────────┐   │   │
│  │  │           agent_service.rs (新增) - 核心               │   │   │
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐   │   │   │
│  │  │  │ AgentLoop   │  │ ToolRegistry│  │ ContextMgr   │   │   │   │
│  │  │  │ (执行循环)  │  │ (工具注册)  │  │ (上下文管理) │   │   │   │
│  │  │  └──────┬──────┘  └──────┬──────┘  └──────┬───────┘   │   │   │
│  │  │         │                │                │            │   │   │
│  │  │  ┌──────┴────────────────┴────────────────┴──────────┐│   │   │
│  │  │  │              ToolExecutor (工具执行器)             ││   │   │
│  │  │  └────────────────────────┬───────────────────────────┘│   │   │
│  │  └───────────────────────────┼────────────────────────────┘   │   │
│  │                              │                                 │   │
│  │  ┌───────────────────────────┴────────────────────────────┐   │   │
│  │  │     现有服务层 (ai_functions_service.rs 等)             │   │   │
│  │  │  - get_tasks      - create_task    - search_sql        │   │   │
│  │  │  - get_time_stats - start_pomodoro - generate_report   │   │   │
│  │  └───────────────────────────┬────────────────────────────┘   │   │
│  │                              │                                 │   │
│  │  ┌───────────────────────────┴────────────────────────────┐   │   │
│  │  │                    数据库 (SQLite)                      │   │   │
│  │  └────────────────────────────────────────────────────────┘   │   │
│  └───────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 核心模块说明

| 模块 | 位置 | 职责 |
|------|------|------|
| **AgentPanel** | 前端组件 | Agent交互界面，展示执行步骤 |
| **agentStore** | Pinia Store | 管理Agent状态、执行进度 |
| **agentApi** | 前端API | 封装Agent相关的IPC调用 |
| **agent_commands** | Rust命令 | 暴露给前端的Agent命令 |
| **agent_service** | Rust服务 | Agent核心逻辑（循环、工具调用） |
| **ToolRegistry** | Rust组件 | 管理所有可用工具的定义和元数据 |
| **ToolExecutor** | Rust组件 | 执行具体工具，处理结果 |
| **ContextManager** | Rust组件 | 管理对话上下文、操作历史 |

---

## 3. 兼容性设计

### 3.1 三层模式设计

```
┌───────────────────────────────────────────────────────────────┐
│                        用户体验层级                            │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  Level 0: 无AI模式 (AI未配置或禁用)                           │
│  ├─ 所有功能手动操作                                          │
│  ├─ AI聊天入口隐藏或显示"未配置"提示                          │
│  ├─ 任务/SQL/番茄等功能完全正常                               │
│  └─ 无任何AI相关的网络请求                                    │
│                                                               │
│  Level 1: 基础AI模式 (AI已配置，Agent未启用)                  │
│  ├─ 现有AI功能正常 (任务分类、日志生成等)                     │
│  ├─ AI聊天可用，但只支持简单问答                              │
│  ├─ 不支持自动工具调用                                        │
│  └─ 与当前版本行为一致                                        │
│                                                               │
│  Level 2: 智能Agent模式 (Agent已启用)                         │
│  ├─ 支持自然语言完成复杂任务                                  │
│  ├─ 自动规划和执行多步骤操作                                  │
│  ├─ 透明展示执行过程                                          │
│  └─ 支持确认/中断/回退操作                                    │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### 3.2 配置开关设计

```typescript
// 设置页面新增配置项
interface AgentConfig {
  // Agent总开关
  enabled: boolean;

  // 执行模式
  executionMode: 'auto' | 'confirm' | 'preview';
  // auto: 自动执行所有操作
  // confirm: 每步操作前需要确认
  // preview: 只展示计划，不执行

  // 允许的操作类型
  allowedActions: {
    query: boolean;      // 查询类操作（默认开启）
    create: boolean;     // 创建类操作（需确认）
    update: boolean;     // 更新类操作（需确认）
    delete: boolean;     // 删除类操作（默认关闭）
    external: boolean;   // 外部操作如启动应用（需确认）
  };

  // 单次执行限制
  maxStepsPerExecution: number;  // 默认10步
  maxTokensPerExecution: number; // 默认4000

  // 自动保存执行历史
  saveExecutionHistory: boolean;
}
```

### 3.3 渐进式启用流程

```
用户首次使用AI功能:
┌─────────────────────────────────────────────────────────────┐
│  1. 检测AI配置状态                                          │
│     ├─ 未配置 → 引导配置AI (设置页面)                       │
│     └─ 已配置 → 继续                                        │
│                                                             │
│  2. 检测Agent启用状态                                       │
│     ├─ 未启用 → 使用基础AI模式                              │
│     │           (显示"升级到Agent模式"入口)                 │
│     └─ 已启用 → 使用智能Agent模式                           │
│                                                             │
│  3. 首次启用Agent时:                                        │
│     ├─ 显示功能介绍和风险提示                               │
│     ├─ 默认使用"确认模式"(每步确认)                         │
│     └─ 用户可在设置中调整为自动模式                         │
└─────────────────────────────────────────────────────────────┘
```

### 3.4 降级策略

```typescript
// 自动降级处理
async function handleAgentRequest(input: string) {
  // 检查AI可用性
  if (!aiStore.isEnabled) {
    return {
      type: 'fallback',
      message: 'AI功能未启用，请在设置中配置'
    };
  }

  // 检查Agent可用性
  if (!agentStore.isEnabled) {
    // 降级到基础AI模式
    return basicAiChat(input);
  }

  // 检查网络连接
  const isOnline = await checkAiConnection();
  if (!isOnline) {
    return {
      type: 'offline',
      message: 'AI服务暂时不可用，您可以继续使用其他功能'
    };
  }

  // 正常执行Agent
  return executeAgent(input);
}
```

---

## 4. 工具系统设计

### 4.1 工具定义结构

```rust
// src-tauri/src/agent/tools/mod.rs

/// 工具定义结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具唯一标识
    pub name: String,

    /// 工具描述（供LLM理解）
    pub description: String,

    /// 工具分类
    pub category: ToolCategory,

    /// 操作类型（用于权限控制）
    pub action_type: ActionType,

    /// 参数定义（JSON Schema格式）
    pub parameters: ParameterSchema,

    /// 是否需要确认
    pub requires_confirmation: bool,

    /// 是否在Agent禁用时仍可单独使用
    pub standalone_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCategory {
    Task,       // 任务相关
    Pomodoro,   // 番茄钟相关
    Sql,        // SQL相关
    Report,     // 报告相关
    Statistics, // 统计相关
    App,        // 应用启动器相关
    System,     // 系统相关
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Query,      // 查询（只读）
    Create,     // 创建
    Update,     // 更新
    Delete,     // 删除
    Execute,    // 执行（如启动应用、开始番茄钟）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    pub properties: HashMap<String, ParameterDef>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub param_type: String,       // string, number, boolean, array
    pub description: String,
    pub enum_values: Option<Vec<String>>,
    pub default: Option<serde_json::Value>,
}
```

### 4.2 完整工具清单

```rust
// 基于现有 ai_functions_service.rs 扩展

pub fn get_all_tools() -> Vec<ToolDefinition> {
    vec![
        // ═══════════════════════════════════════════
        // 任务管理工具 (Task)
        // ═══════════════════════════════════════════
        ToolDefinition {
            name: "get_tasks".into(),
            description: "获取任务列表。可按状态、象限、日期范围、关键词筛选。".into(),
            category: ToolCategory::Task,
            action_type: ActionType::Query,
            parameters: json_schema!({
                "status": {
                    "type": "string",
                    "enum": ["todo", "active", "done", "deferred", "all"],
                    "description": "任务状态筛选"
                },
                "quadrant": {
                    "type": "integer",
                    "enum": [1, 2, 3, 4],
                    "description": "四象限: 1=重要紧急, 2=重要不紧急, 3=紧急不重要, 4=不重要不紧急"
                },
                "date_range": {
                    "type": "string",
                    "enum": ["today", "yesterday", "this_week", "last_week", "this_month"],
                    "description": "日期范围"
                },
                "keyword": {
                    "type": "string",
                    "description": "搜索关键词"
                },
                "limit": {
                    "type": "integer",
                    "default": 20,
                    "description": "返回数量限制"
                }
            }),
            requires_confirmation: false,
            standalone_available: true,
        },

        ToolDefinition {
            name: "create_task".into(),
            description: "创建新任务。支持自然语言日期如'下周五'、'3天后'。".into(),
            category: ToolCategory::Task,
            action_type: ActionType::Create,
            parameters: json_schema!({
                "title": {
                    "type": "string",
                    "required": true,
                    "description": "任务标题"
                },
                "description": {
                    "type": "string",
                    "description": "任务详细描述"
                },
                "quadrant": {
                    "type": "integer",
                    "enum": [1, 2, 3, 4],
                    "description": "四象限分类"
                },
                "due_date": {
                    "type": "string",
                    "description": "截止日期，支持自然语言"
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "标签列表"
                }
            }),
            requires_confirmation: true,
            standalone_available: true,
        },

        ToolDefinition {
            name: "update_task".into(),
            description: "更新任务信息。可修改标题、状态、象限、截止日期等。".into(),
            category: ToolCategory::Task,
            action_type: ActionType::Update,
            parameters: json_schema!({
                "task_id": {
                    "type": "integer",
                    "required": true,
                    "description": "任务ID"
                },
                "title": { "type": "string" },
                "status": {
                    "type": "string",
                    "enum": ["active", "done", "deferred"]
                },
                "quadrant": { "type": "integer", "enum": [1, 2, 3, 4] },
                "due_date": { "type": "string" }
            }),
            requires_confirmation: true,
            standalone_available: true,
        },

        ToolDefinition {
            name: "complete_task".into(),
            description: "将任务标记为完成".into(),
            category: ToolCategory::Task,
            action_type: ActionType::Update,
            parameters: json_schema!({
                "task_id": {
                    "type": "integer",
                    "required": true,
                    "description": "要完成的任务ID"
                }
            }),
            requires_confirmation: false, // 完成任务风险低
            standalone_available: true,
        },

        ToolDefinition {
            name: "delete_task".into(),
            description: "删除任务（危险操作）".into(),
            category: ToolCategory::Task,
            action_type: ActionType::Delete,
            parameters: json_schema!({
                "task_id": {
                    "type": "integer",
                    "required": true
                }
            }),
            requires_confirmation: true, // 必须确认
            standalone_available: false, // 不允许单独调用
        },

        // ═══════════════════════════════════════════
        // 番茄钟工具 (Pomodoro)
        // ═══════════════════════════════════════════
        ToolDefinition {
            name: "get_pomodoro_records".into(),
            description: "获取番茄钟记录".into(),
            category: ToolCategory::Pomodoro,
            action_type: ActionType::Query,
            parameters: json_schema!({
                "date_range": {
                    "type": "string",
                    "enum": ["today", "yesterday", "this_week", "last_week", "this_month"]
                },
                "task_id": {
                    "type": "integer",
                    "description": "指定任务的番茄钟"
                }
            }),
            requires_confirmation: false,
            standalone_available: true,
        },

        ToolDefinition {
            name: "start_pomodoro".into(),
            description: "开始一个番茄钟计时，可关联到指定任务".into(),
            category: ToolCategory::Pomodoro,
            action_type: ActionType::Execute,
            parameters: json_schema!({
                "duration_minutes": {
                    "type": "integer",
                    "default": 25,
                    "description": "时长（分钟）"
                },
                "task_id": {
                    "type": "integer",
                    "description": "关联的任务ID"
                }
            }),
            requires_confirmation: false, // 启动番茄钟风险低
            standalone_available: true,
        },

        ToolDefinition {
            name: "stop_pomodoro".into(),
            description: "停止当前番茄钟".into(),
            category: ToolCategory::Pomodoro,
            action_type: ActionType::Execute,
            parameters: json_schema!({}),
            requires_confirmation: true, // 中断需要确认
            standalone_available: true,
        },

        // ═══════════════════════════════════════════
        // SQL工具
        // ═══════════════════════════════════════════
        ToolDefinition {
            name: "search_sql".into(),
            description: "搜索SQL历史记录".into(),
            category: ToolCategory::Sql,
            action_type: ActionType::Query,
            parameters: json_schema!({
                "keyword": {
                    "type": "string",
                    "description": "搜索关键词"
                },
                "sql_type": {
                    "type": "string",
                    "enum": ["SELECT", "INSERT", "UPDATE", "DELETE", "CREATE", "ALTER"]
                },
                "favorite_only": {
                    "type": "boolean",
                    "default": false
                },
                "limit": {
                    "type": "integer",
                    "default": 20
                }
            }),
            requires_confirmation: false,
            standalone_available: true,
        },

        // ═══════════════════════════════════════════
        // 统计工具
        // ═══════════════════════════════════════════
        ToolDefinition {
            name: "get_time_stats".into(),
            description: "获取时间统计数据，包括专注时长、任务完成情况等".into(),
            category: ToolCategory::Statistics,
            action_type: ActionType::Query,
            parameters: json_schema!({
                "date_range": {
                    "type": "string",
                    "enum": ["today", "yesterday", "this_week", "last_week", "this_month"],
                    "required": true
                },
                "group_by": {
                    "type": "string",
                    "enum": ["day", "total"],
                    "default": "total"
                }
            }),
            requires_confirmation: false,
            standalone_available: true,
        },

        ToolDefinition {
            name: "analyze_efficiency".into(),
            description: "分析工作效率，生成效率报告".into(),
            category: ToolCategory::Statistics,
            action_type: ActionType::Query,
            parameters: json_schema!({
                "date_range": {
                    "type": "string",
                    "enum": ["today", "this_week", "last_week", "this_month"]
                },
                "focus": {
                    "type": "string",
                    "enum": ["overview", "focus_time", "task_completion", "productivity_trend"],
                    "default": "overview"
                }
            }),
            requires_confirmation: false,
            standalone_available: true,
        },

        // ═══════════════════════════════════════════
        // 报告工具
        // ═══════════════════════════════════════════
        ToolDefinition {
            name: "generate_daily_report".into(),
            description: "生成指定日期的工作日报".into(),
            category: ToolCategory::Report,
            action_type: ActionType::Create,
            parameters: json_schema!({
                "date": {
                    "type": "string",
                    "description": "日期，默认今天"
                },
                "style": {
                    "type": "string",
                    "enum": ["brief", "detailed", "bullet_points"],
                    "default": "detailed"
                },
                "include_time_stats": {
                    "type": "boolean",
                    "default": true
                }
            }),
            requires_confirmation: false,
            standalone_available: true,
        },

        ToolDefinition {
            name: "generate_weekly_report".into(),
            description: "生成周报".into(),
            category: ToolCategory::Report,
            action_type: ActionType::Create,
            parameters: json_schema!({
                "week": {
                    "type": "string",
                    "enum": ["this_week", "last_week"],
                    "default": "this_week"
                },
                "style": {
                    "type": "string",
                    "enum": ["brief", "detailed"],
                    "default": "detailed"
                },
                "include_comparison": {
                    "type": "boolean",
                    "default": true,
                    "description": "是否包含与上周的对比"
                }
            }),
            requires_confirmation: false,
            standalone_available: true,
        },

        // ═══════════════════════════════════════════
        // 应用启动器工具
        // ═══════════════════════════════════════════
        ToolDefinition {
            name: "search_apps".into(),
            description: "搜索已配置的应用".into(),
            category: ToolCategory::App,
            action_type: ActionType::Query,
            parameters: json_schema!({
                "keyword": {
                    "type": "string",
                    "description": "应用名称关键词"
                },
                "category": {
                    "type": "string",
                    "description": "应用分类"
                }
            }),
            requires_confirmation: false,
            standalone_available: true,
        },

        ToolDefinition {
            name: "launch_app".into(),
            description: "启动指定应用".into(),
            category: ToolCategory::App,
            action_type: ActionType::Execute,
            parameters: json_schema!({
                "app_id": {
                    "type": "string",
                    "required": true,
                    "description": "应用ID"
                }
            }),
            requires_confirmation: true, // 启动外部程序需确认
            standalone_available: true,
        },

        ToolDefinition {
            name: "launch_workflow".into(),
            description: "启动工作流（批量启动多个应用）".into(),
            category: ToolCategory::App,
            action_type: ActionType::Execute,
            parameters: json_schema!({
                "workflow_id": {
                    "type": "string",
                    "required": true
                }
            }),
            requires_confirmation: true,
            standalone_available: true,
        },

        // ═══════════════════════════════════════════
        // 系统工具
        // ═══════════════════════════════════════════
        ToolDefinition {
            name: "get_current_context".into(),
            description: "获取当前操作上下文（最近操作的任务、番茄钟等）".into(),
            category: ToolCategory::System,
            action_type: ActionType::Query,
            parameters: json_schema!({}),
            requires_confirmation: false,
            standalone_available: false, // 仅供Agent内部使用
        },

        ToolDefinition {
            name: "get_today_summary".into(),
            description: "获取今日工作概要（快速了解今天做了什么）".into(),
            category: ToolCategory::System,
            action_type: ActionType::Query,
            parameters: json_schema!({}),
            requires_confirmation: false,
            standalone_available: true,
        },
    ]
}
```

### 4.3 工具执行器设计

```rust
// src-tauri/src/agent/executor.rs

pub struct ToolExecutor {
    db: Arc<Mutex<Connection>>,
    config: AgentConfig,
}

impl ToolExecutor {
    /// 执行工具调用
    pub async fn execute(
        &self,
        tool_name: &str,
        args: serde_json::Value,
        context: &ExecutionContext,
    ) -> Result<ToolResult, AgentError> {

        // 1. 获取工具定义
        let tool = self.get_tool_definition(tool_name)?;

        // 2. 权限检查
        self.check_permission(&tool, context)?;

        // 3. 参数验证
        self.validate_parameters(&tool, &args)?;

        // 4. 执行前处理（如日期解析）
        let processed_args = self.preprocess_args(&tool, args)?;

        // 5. 执行工具
        let result = match tool_name {
            "get_tasks" => self.execute_get_tasks(processed_args).await,
            "create_task" => self.execute_create_task(processed_args).await,
            "update_task" => self.execute_update_task(processed_args).await,
            "complete_task" => self.execute_complete_task(processed_args).await,
            "get_pomodoro_records" => self.execute_get_pomodoro(processed_args).await,
            "start_pomodoro" => self.execute_start_pomodoro(processed_args).await,
            "search_sql" => self.execute_search_sql(processed_args).await,
            "get_time_stats" => self.execute_get_time_stats(processed_args).await,
            "analyze_efficiency" => self.execute_analyze_efficiency(processed_args).await,
            "generate_daily_report" => self.execute_generate_daily_report(processed_args).await,
            "generate_weekly_report" => self.execute_generate_weekly_report(processed_args).await,
            "search_apps" => self.execute_search_apps(processed_args).await,
            "launch_app" => self.execute_launch_app(processed_args).await,
            "get_current_context" => self.execute_get_context(context).await,
            "get_today_summary" => self.execute_get_today_summary().await,
            _ => Err(AgentError::UnknownTool(tool_name.to_string())),
        }?;

        // 6. 后处理（格式化输出）
        let formatted_result = self.postprocess_result(&tool, result)?;

        // 7. 记录执行日志
        self.log_execution(tool_name, &processed_args, &formatted_result, context)?;

        Ok(formatted_result)
    }

    /// 权限检查
    fn check_permission(&self, tool: &ToolDefinition, context: &ExecutionContext) -> Result<(), AgentError> {
        let allowed = match tool.action_type {
            ActionType::Query => self.config.allowed_actions.query,
            ActionType::Create => self.config.allowed_actions.create,
            ActionType::Update => self.config.allowed_actions.update,
            ActionType::Delete => self.config.allowed_actions.delete,
            ActionType::Execute => self.config.allowed_actions.external,
        };

        if !allowed {
            return Err(AgentError::PermissionDenied {
                tool: tool.name.clone(),
                action_type: tool.action_type.clone(),
            });
        }

        Ok(())
    }
}
```

---

## 5. Agent 核心设计

### 5.1 Agent执行流程

```
┌─────────────────────────────────────────────────────────────────┐
│                      Agent 执行流程                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   用户输入: "帮我整理今天的工作，生成日报"                        │
│                              │                                  │
│                              ▼                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │ Step 1: 意图理解                                         │   │
│   │ • 解析用户意图                                           │   │
│   │ • 识别涉及的数据类型（任务、SQL、番茄钟）                  │   │
│   │ • 确定目标输出（日报）                                    │   │
│   └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │ Step 2: 任务规划                                         │   │
│   │ • 生成执行计划:                                          │   │
│   │   1. get_tasks(status=done, date_range=today)            │   │
│   │   2. search_sql(date_range=today)                        │   │
│   │   3. get_pomodoro_records(date_range=today)              │   │
│   │   4. generate_daily_report(date=today)                   │   │
│   └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │ Step 3: 循环执行                                         │   │
│   │                                                          │   │
│   │  ┌──────────────┐                                        │   │
│   │  │ 检查是否需要  │──── 是 ────▶ 暂停等待用户确认           │   │
│   │  │ 用户确认     │                     │                  │   │
│   │  └──────┬───────┘                     │                  │   │
│   │         │ 否                          │                  │   │
│   │         ▼                             │                  │   │
│   │  ┌──────────────┐                     │                  │   │
│   │  │ 执行工具调用  │ ◀─────────────────┘                   │   │
│   │  └──────┬───────┘                                        │   │
│   │         │                                                │   │
│   │         ▼                                                │   │
│   │  ┌──────────────┐                                        │   │
│   │  │ 收集执行结果  │                                        │   │
│   │  └──────┬───────┘                                        │   │
│   │         │                                                │   │
│   │         ▼                                                │   │
│   │  ┌──────────────┐                                        │   │
│   │  │ LLM分析结果  │                                        │   │
│   │  │ 决定下一步   │                                        │   │
│   │  └──────┬───────┘                                        │   │
│   │         │                                                │   │
│   │         ▼                                                │   │
│   │  ┌──────────────┐                                        │   │
│   │  │ 是否完成？   │──── 否 ────▶ 继续循环                   │   │
│   │  └──────┬───────┘                                        │   │
│   │         │ 是                                             │   │
│   │         ▼                                                │   │
│   └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │ Step 4: 输出结果                                         │   │
│   │ • 格式化最终输出                                         │   │
│   │ • 展示执行摘要                                           │   │
│   │ • 提供后续操作建议                                        │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Agent核心代码

```rust
// src-tauri/src/agent/core.rs

use tokio::sync::mpsc;

pub struct AgentCore {
    executor: ToolExecutor,
    ai_service: AiService,
    config: AgentConfig,
    event_sender: mpsc::Sender<AgentEvent>,
}

/// Agent执行事件（用于前端实时更新）
#[derive(Debug, Clone, Serialize)]
pub enum AgentEvent {
    /// 开始执行
    Started { request_id: String },
    /// 思考中
    Thinking { step: u32, message: String },
    /// 准备执行工具
    ToolPending {
        step: u32,
        tool_name: String,
        args: serde_json::Value,
        requires_confirmation: bool,
    },
    /// 工具执行中
    ToolExecuting { step: u32, tool_name: String },
    /// 工具执行完成
    ToolCompleted {
        step: u32,
        tool_name: String,
        result_summary: String,
    },
    /// 需要用户确认
    ConfirmationRequired {
        step: u32,
        tool_name: String,
        description: String,
        args: serde_json::Value,
    },
    /// 执行完成
    Completed {
        request_id: String,
        final_response: String,
        steps_executed: u32,
    },
    /// 执行失败
    Failed {
        request_id: String,
        error: String,
        step: Option<u32>,
    },
    /// 用户取消
    Cancelled { request_id: String },
}

/// 执行上下文
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub request_id: String,
    pub session_id: Option<String>,
    pub operation_context: OperationContext,  // 代词解析上下文
    pub execution_history: Vec<ExecutionStep>,
    pub pending_confirmation: Option<PendingAction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionStep {
    pub step_number: u32,
    pub tool_name: String,
    pub args: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub status: StepStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub enum StepStatus {
    Pending,
    Executing,
    Completed,
    Failed(String),
    Cancelled,
    AwaitingConfirmation,
}

impl AgentCore {
    /// 执行Agent
    pub async fn execute(&mut self, user_input: &str, context: &mut ExecutionContext) -> Result<String, AgentError> {
        let request_id = context.request_id.clone();

        // 发送开始事件
        self.emit(AgentEvent::Started { request_id: request_id.clone() }).await;

        // 构建系统提示词
        let system_prompt = self.build_system_prompt();

        // 构建消息历史
        let mut messages = vec![
            ChatMessage::system(&system_prompt),
            ChatMessage::user(user_input),
        ];

        let mut step = 0u32;
        let max_steps = self.config.max_steps_per_execution;

        loop {
            step += 1;

            // 防止无限循环
            if step > max_steps {
                return Err(AgentError::MaxStepsExceeded(max_steps));
            }

            // 发送思考事件
            self.emit(AgentEvent::Thinking {
                step,
                message: "分析中...".into()
            }).await;

            // 调用LLM
            let response = self.ai_service.chat_with_tools(
                &messages,
                &self.get_tool_schemas(),
            ).await?;

            // 检查是否有工具调用
            if let Some(tool_calls) = response.tool_calls {
                for tool_call in tool_calls {
                    // 获取工具定义
                    let tool_def = self.executor.get_tool_definition(&tool_call.name)?;

                    // 检查是否需要确认
                    if self.requires_confirmation(&tool_def) {
                        self.emit(AgentEvent::ConfirmationRequired {
                            step,
                            tool_name: tool_call.name.clone(),
                            description: tool_def.description.clone(),
                            args: tool_call.arguments.clone(),
                        }).await;

                        // 等待用户确认
                        context.pending_confirmation = Some(PendingAction {
                            step,
                            tool_name: tool_call.name.clone(),
                            args: tool_call.arguments.clone(),
                        });

                        return Err(AgentError::AwaitingConfirmation);
                    }

                    // 执行工具
                    self.emit(AgentEvent::ToolExecuting {
                        step,
                        tool_name: tool_call.name.clone()
                    }).await;

                    let result = self.executor.execute(
                        &tool_call.name,
                        tool_call.arguments.clone(),
                        context,
                    ).await?;

                    // 记录执行步骤
                    context.execution_history.push(ExecutionStep {
                        step_number: step,
                        tool_name: tool_call.name.clone(),
                        args: tool_call.arguments.clone(),
                        result: Some(result.data.clone()),
                        status: StepStatus::Completed,
                        started_at: chrono::Utc::now(),
                        completed_at: Some(chrono::Utc::now()),
                    });

                    // 发送完成事件
                    self.emit(AgentEvent::ToolCompleted {
                        step,
                        tool_name: tool_call.name.clone(),
                        result_summary: result.summary.clone(),
                    }).await;

                    // 将结果添加到消息历史
                    messages.push(ChatMessage::tool(
                        &tool_call.id,
                        &serde_json::to_string(&result.data)?,
                    ));
                }

                // 继续循环，让LLM决定下一步
                continue;
            }

            // 没有工具调用，说明LLM完成了任务
            let final_response = response.content.unwrap_or_default();

            // 发送完成事件
            self.emit(AgentEvent::Completed {
                request_id,
                final_response: final_response.clone(),
                steps_executed: step,
            }).await;

            return Ok(final_response);
        }
    }

    /// 用户确认后继续执行
    pub async fn confirm_and_continue(&mut self, context: &mut ExecutionContext) -> Result<String, AgentError> {
        if let Some(pending) = context.pending_confirmation.take() {
            // 执行之前等待确认的工具
            let result = self.executor.execute(
                &pending.tool_name,
                pending.args,
                context,
            ).await?;

            // 继续Agent循环
            // ... (将结果加入消息历史，继续执行)
        }

        Err(AgentError::NoPendingConfirmation)
    }

    /// 取消执行
    pub async fn cancel(&mut self, context: &mut ExecutionContext) -> Result<(), AgentError> {
        self.emit(AgentEvent::Cancelled {
            request_id: context.request_id.clone()
        }).await;

        context.pending_confirmation = None;
        Ok(())
    }

    /// 构建系统提示词
    fn build_system_prompt(&self) -> String {
        format!(r#"
你是 DevAssistant 的AI助手，帮助用户管理开发任务、追踪工作时间和生成报告。

## 可用工具
你可以使用以下工具来完成用户的请求：

### 任务管理
- get_tasks: 查询任务列表
- create_task: 创建新任务
- update_task: 更新任务
- complete_task: 完成任务

### 时间追踪
- get_pomodoro_records: 获取番茄钟记录
- start_pomodoro: 开始番茄钟
- get_time_stats: 获取时间统计

### SQL管理
- search_sql: 搜索SQL历史

### 报告生成
- generate_daily_report: 生成日报
- generate_weekly_report: 生成周报
- analyze_efficiency: 效率分析

### 应用启动
- search_apps: 搜索应用
- launch_app: 启动应用

## 执行原则
1. 理解用户意图，拆解为具体步骤
2. 优先使用查询工具收集信息
3. 创建/修改操作前先确认数据
4. 给出清晰的执行摘要
5. 对于模糊的请求，先询问澄清

## 日期理解
支持自然语言日期：今天、明天、下周五、3天后等

## 代词理解
用户可能使用"这个任务"、"刚才的"等代词，请结合上下文理解。

当前时间: {current_time}
"#, current_time = chrono::Local::now().format("%Y-%m-%d %H:%M"))
    }

    /// 判断是否需要确认
    fn requires_confirmation(&self, tool: &ToolDefinition) -> bool {
        if self.config.execution_mode == ExecutionMode::Auto {
            return false;
        }

        if self.config.execution_mode == ExecutionMode::Confirm {
            return tool.requires_confirmation;
        }

        // Preview模式：所有操作都需要确认
        true
    }

    /// 发送事件
    async fn emit(&self, event: AgentEvent) {
        let _ = self.event_sender.send(event).await;
    }
}
```

### 5.3 LLM工具调用格式

```rust
// 发送给LLM的工具定义格式（OpenAI兼容）
fn get_tool_schemas(&self) -> Vec<serde_json::Value> {
    self.tools.iter().map(|tool| {
        json!({
            "type": "function",
            "function": {
                "name": tool.name,
                "description": tool.description,
                "parameters": {
                    "type": "object",
                    "properties": tool.parameters.properties,
                    "required": tool.parameters.required,
                }
            }
        })
    }).collect()
}
```

---

## 6. 前端交互设计

### 6.1 UI组件结构

```
┌─────────────────────────────────────────────────────────────────┐
│                     AiChat / AgentPanel                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 消息区域                                                 │   │
│  │ ┌─────────────────────────────────────────────────────┐ │   │
│  │ │ 👤 帮我整理今天的工作，生成日报                      │ │   │
│  │ └─────────────────────────────────────────────────────┘ │   │
│  │                                                         │   │
│  │ ┌─────────────────────────────────────────────────────┐ │   │
│  │ │ 🤖 正在处理您的请求...                              │ │   │
│  │ │                                                     │ │   │
│  │ │ ┌─────────────────────────────────────────────────┐ │ │   │
│  │ │ │ 📋 执行步骤                                      │ │ │   │
│  │ │ │                                                 │ │ │   │
│  │ │ │ ✅ Step 1: 获取今日完成的任务                   │ │ │   │
│  │ │ │    └─ 找到 3 个已完成任务                       │ │ │   │
│  │ │ │                                                 │ │ │   │
│  │ │ │ ✅ Step 2: 获取今日SQL记录                      │ │ │   │
│  │ │ │    └─ 找到 8 条SQL记录                          │ │ │   │
│  │ │ │                                                 │ │ │   │
│  │ │ │ ⏳ Step 3: 获取番茄钟记录                       │ │ │   │
│  │ │ │    └─ 执行中...                                 │ │ │   │
│  │ │ │                                                 │ │ │   │
│  │ │ │ ⬜ Step 4: 生成日报                             │ │ │   │
│  │ │ └─────────────────────────────────────────────────┘ │ │   │
│  │ └─────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 输入区域                                                 │   │
│  │ ┌───────────────────────────────────────────┐ ┌───────┐ │   │
│  │ │ 输入消息...                               │ │ 发送  │ │   │
│  │ └───────────────────────────────────────────┘ └───────┘ │   │
│  │                                                         │   │
│  │ 💡 快捷指令: [整理今日工作] [生成周报] [效率分析]        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 确认对话框设计

```
┌─────────────────────────────────────────────────────────────────┐
│                        确认操作                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ⚠️ Agent 需要执行以下操作：                                    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 🔧 创建任务                                              │   │
│  │                                                         │   │
│  │ 标题: 用户登录功能开发                                   │   │
│  │ 截止日期: 2026-02-06 (下周五)                           │   │
│  │ 象限: 重要且紧急                                        │   │
│  │ 标签: [后端, 功能开发]                                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ ☐ 不再询问此类操作                                      │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│           ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│           │   取消   │  │   修改   │  │   确认   │             │
│           └──────────┘  └──────────┘  └──────────┘             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 6.3 前端状态管理

```typescript
// src/stores/agentStore.ts

interface AgentState {
  // 配置
  config: AgentConfig;
  isEnabled: boolean;

  // 执行状态
  isExecuting: boolean;
  currentRequestId: string | null;
  executionSteps: ExecutionStep[];

  // 确认状态
  pendingConfirmation: PendingAction | null;

  // 历史
  executionHistory: ExecutionRecord[];
}

interface ExecutionStep {
  stepNumber: number;
  toolName: string;
  toolDisplayName: string;
  args: Record<string, any>;
  status: 'pending' | 'executing' | 'completed' | 'failed' | 'awaiting_confirmation';
  resultSummary?: string;
  error?: string;
}

interface PendingAction {
  step: number;
  toolName: string;
  toolDisplayName: string;
  description: string;
  args: Record<string, any>;
}

export const useAgentStore = defineStore('agent', () => {
  // 状态
  const config = ref<AgentConfig>(getDefaultConfig());
  const isEnabled = computed(() => config.value.enabled);
  const isExecuting = ref(false);
  const executionSteps = ref<ExecutionStep[]>([]);
  const pendingConfirmation = ref<PendingAction | null>(null);

  // 执行Agent
  async function execute(input: string) {
    if (!isEnabled.value) {
      throw new Error('Agent未启用');
    }

    isExecuting.value = true;
    executionSteps.value = [];

    try {
      // 监听事件
      const unlisten = await listen<AgentEvent>('agent-event', (event) => {
        handleAgentEvent(event.payload);
      });

      // 执行
      const result = await invoke('execute_agent', { input });

      unlisten();
      return result;
    } finally {
      isExecuting.value = false;
    }
  }

  // 处理Agent事件
  function handleAgentEvent(event: AgentEvent) {
    switch (event.type) {
      case 'Thinking':
        // 更新思考状态
        break;

      case 'ToolPending':
        executionSteps.value.push({
          stepNumber: event.step,
          toolName: event.tool_name,
          toolDisplayName: getToolDisplayName(event.tool_name),
          args: event.args,
          status: 'pending',
        });
        break;

      case 'ToolExecuting':
        updateStepStatus(event.step, 'executing');
        break;

      case 'ToolCompleted':
        updateStepStatus(event.step, 'completed', event.result_summary);
        break;

      case 'ConfirmationRequired':
        pendingConfirmation.value = {
          step: event.step,
          toolName: event.tool_name,
          toolDisplayName: getToolDisplayName(event.tool_name),
          description: event.description,
          args: event.args,
        };
        updateStepStatus(event.step, 'awaiting_confirmation');
        break;

      case 'Completed':
        isExecuting.value = false;
        break;

      case 'Failed':
        isExecuting.value = false;
        if (event.step) {
          updateStepStatus(event.step, 'failed', undefined, event.error);
        }
        break;
    }
  }

  // 确认操作
  async function confirmAction() {
    if (!pendingConfirmation.value) return;

    pendingConfirmation.value = null;
    await invoke('confirm_agent_action');
  }

  // 取消操作
  async function cancelAction() {
    pendingConfirmation.value = null;
    await invoke('cancel_agent_execution');
  }

  return {
    config,
    isEnabled,
    isExecuting,
    executionSteps,
    pendingConfirmation,
    execute,
    confirmAction,
    cancelAction,
  };
});
```

### 6.4 执行步骤组件

```vue
<!-- src/components/aiChat/AgentExecutionSteps.vue -->
<template>
  <div class="agent-steps" v-if="steps.length > 0">
    <div class="steps-header">
      <span class="steps-icon">📋</span>
      <span class="steps-title">执行步骤</span>
      <span class="steps-count">{{ completedCount }}/{{ steps.length }}</span>
    </div>

    <div class="steps-list">
      <div
        v-for="step in steps"
        :key="step.stepNumber"
        class="step-item"
        :class="step.status"
      >
        <div class="step-status-icon">
          <template v-if="step.status === 'completed'">✅</template>
          <template v-else-if="step.status === 'executing'">
            <n-spin size="small" />
          </template>
          <template v-else-if="step.status === 'failed'">❌</template>
          <template v-else-if="step.status === 'awaiting_confirmation'">⏸️</template>
          <template v-else>⬜</template>
        </div>

        <div class="step-content">
          <div class="step-name">
            Step {{ step.stepNumber }}: {{ step.toolDisplayName }}
          </div>
          <div class="step-result" v-if="step.resultSummary">
            └─ {{ step.resultSummary }}
          </div>
          <div class="step-error" v-if="step.error">
            └─ 错误: {{ step.error }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NSpin } from 'naive-ui';

interface ExecutionStep {
  stepNumber: number;
  toolDisplayName: string;
  status: string;
  resultSummary?: string;
  error?: string;
}

const props = defineProps<{
  steps: ExecutionStep[];
}>();

const completedCount = computed(() =>
  props.steps.filter(s => s.status === 'completed').length
);
</script>

<style scoped>
.agent-steps {
  background: var(--bg-elevated);
  border-radius: 8px;
  padding: 12px;
  margin: 8px 0;
}

.steps-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  color: var(--text-primary);
}

.steps-title {
  font-weight: 500;
}

.steps-count {
  color: var(--text-secondary);
  font-size: 12px;
}

.step-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px 0;
}

.step-item.executing {
  color: var(--accent-primary);
}

.step-item.completed {
  color: var(--text-secondary);
}

.step-item.failed {
  color: var(--color-error);
}

.step-status-icon {
  width: 20px;
  text-align: center;
}

.step-content {
  flex: 1;
}

.step-name {
  color: var(--text-primary);
}

.step-result {
  color: var(--text-secondary);
  font-size: 12px;
  margin-top: 2px;
}

.step-error {
  color: var(--color-error);
  font-size: 12px;
  margin-top: 2px;
}
</style>
```

---

## 7. 数据流设计

### 7.1 完整数据流

```
┌─────────────────────────────────────────────────────────────────┐
│                        数据流图                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────┐      ┌──────────┐      ┌──────────────────────┐  │
│  │ 用户输入  │ ──▶ │ agentApi │ ──▶ │ agent_commands.rs    │  │
│  └──────────┘      └──────────┘      │ (Tauri Command)      │  │
│                                      └───────────┬──────────┘  │
│                                                  │              │
│                                                  ▼              │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    agent_service.rs                      │   │
│  │  ┌───────────────────────────────────────────────────┐  │   │
│  │  │ 1. 构建消息                                        │  │   │
│  │  │    - 系统提示词                                    │  │   │
│  │  │    - 用户输入                                      │  │   │
│  │  │    - 操作上下文                                    │  │   │
│  │  └───────────────────────────────────────────────────┘  │   │
│  │                          │                               │   │
│  │                          ▼                               │   │
│  │  ┌───────────────────────────────────────────────────┐  │   │
│  │  │ 2. 调用LLM                                         │  │   │
│  │  │    POST /v1/chat/completions                       │  │   │
│  │  │    - messages: [...]                               │  │   │
│  │  │    - tools: [工具定义JSON Schema]                  │  │   │
│  │  │    - tool_choice: "auto"                           │  │   │
│  │  └───────────────────────────────────────────────────┘  │   │
│  │                          │                               │   │
│  │                          ▼                               │   │
│  │  ┌───────────────────────────────────────────────────┐  │   │
│  │  │ 3. 解析响应                                        │  │   │
│  │  │    - 如果有 tool_calls → 执行工具                  │  │   │
│  │  │    - 如果是 content → 返回最终结果                 │  │   │
│  │  └───────────────────────────────────────────────────┘  │   │
│  │                          │                               │   │
│  │              ┌───────────┴───────────┐                   │   │
│  │              ▼                       ▼                   │   │
│  │  ┌─────────────────────┐  ┌─────────────────────────┐   │   │
│  │  │ 工具调用分支         │  │ 最终响应分支            │   │   │
│  │  │                     │  │                         │   │   │
│  │  │ 4. 执行工具         │  │ 7. 格式化输出           │   │   │
│  │  │    ToolExecutor     │  │    返回给前端           │   │   │
│  │  └──────────┬──────────┘  └─────────────────────────┘   │   │
│  │             │                                            │   │
│  │             ▼                                            │   │
│  │  ┌─────────────────────────────────────────────────┐    │   │
│  │  │ 5. 工具结果                                      │    │   │
│  │  │    - 查询数据库                                  │    │   │
│  │  │    - 执行操作                                    │    │   │
│  │  │    - 返回结构化数据                              │    │   │
│  │  └─────────────────────────────────────────────────┘    │   │
│  │             │                                            │   │
│  │             ▼                                            │   │
│  │  ┌─────────────────────────────────────────────────┐    │   │
│  │  │ 6. 更新消息历史                                  │    │   │
│  │  │    messages.push(tool_result)                    │    │   │
│  │  │    → 返回步骤2，继续循环                         │    │   │
│  │  └─────────────────────────────────────────────────┘    │   │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    事件流 (实时)                          │  │
│  │                                                          │  │
│  │  agent_service ──▶ Tauri Event ──▶ agentStore ──▶ UI    │  │
│  │                                                          │  │
│  │  AgentEvent::Thinking        →  更新思考状态             │  │
│  │  AgentEvent::ToolExecuting   →  更新步骤状态             │  │
│  │  AgentEvent::ToolCompleted   →  显示结果摘要             │  │
│  │  AgentEvent::Confirmation    →  弹出确认对话框           │  │
│  │  AgentEvent::Completed       →  显示最终结果             │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 7.2 LLM请求示例

```json
// 发送给LLM的请求
{
  "model": "deepseek-chat",
  "messages": [
    {
      "role": "system",
      "content": "你是 DevAssistant 的AI助手..."
    },
    {
      "role": "user",
      "content": "帮我整理今天的工作，生成日报"
    }
  ],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "get_tasks",
        "description": "获取任务列表",
        "parameters": {
          "type": "object",
          "properties": {
            "status": { "type": "string", "enum": ["todo", "active", "done", "all"] },
            "date_range": { "type": "string", "enum": ["today", "yesterday", "this_week"] }
          }
        }
      }
    }
    // ... 其他工具
  ],
  "tool_choice": "auto"
}

// LLM响应 - 工具调用
{
  "choices": [{
    "message": {
      "role": "assistant",
      "tool_calls": [
        {
          "id": "call_001",
          "type": "function",
          "function": {
            "name": "get_tasks",
            "arguments": "{\"status\": \"done\", \"date_range\": \"today\"}"
          }
        }
      ]
    }
  }]
}

// 执行工具后，将结果加入消息历史
{
  "role": "tool",
  "tool_call_id": "call_001",
  "content": "{\"total\": 3, \"tasks\": [{\"id\": 1, \"title\": \"完成登录功能\"}, ...]}"
}

// 继续调用LLM，LLM可能继续调用工具或返回最终结果
```

---

## 8. 安全与限制

### 8.1 安全措施

```rust
// 安全配置
pub struct SecurityConfig {
    // 敏感操作白名单
    pub allowed_delete_patterns: Vec<String>,  // 例如: ["test_*", "temp_*"]

    // 速率限制
    pub max_tool_calls_per_minute: u32,  // 默认30
    pub max_tokens_per_minute: u32,      // 默认10000

    // 数据限制
    pub max_query_results: u32,          // 单次查询最大返回数
    pub max_batch_operations: u32,       // 批量操作最大数量

    // 沙箱设置
    pub sandbox_mode: bool,              // 沙箱模式（只读）
}

impl AgentCore {
    /// 安全检查
    fn security_check(&self, tool: &ToolDefinition, args: &Value) -> Result<(), SecurityError> {
        // 1. 检查工具是否在黑名单
        if self.is_tool_blocked(tool) {
            return Err(SecurityError::ToolBlocked(tool.name.clone()));
        }

        // 2. 检查参数是否包含危险模式
        if self.contains_dangerous_pattern(args) {
            return Err(SecurityError::DangerousPattern);
        }

        // 3. 检查速率限制
        if self.rate_limiter.is_exceeded() {
            return Err(SecurityError::RateLimitExceeded);
        }

        // 4. 沙箱模式下禁止写操作
        if self.config.sandbox_mode && tool.action_type != ActionType::Query {
            return Err(SecurityError::SandboxModeViolation);
        }

        Ok(())
    }
}
```

### 8.2 错误处理

```rust
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("AI服务不可用: {0}")]
    AiServiceUnavailable(String),

    #[error("未知工具: {0}")]
    UnknownTool(String),

    #[error("参数验证失败: {0}")]
    ValidationError(String),

    #[error("权限不足: 工具 {tool} 需要 {action_type:?} 权限")]
    PermissionDenied { tool: String, action_type: ActionType },

    #[error("超过最大执行步数: {0}")]
    MaxStepsExceeded(u32),

    #[error("等待用户确认")]
    AwaitingConfirmation,

    #[error("没有待确认的操作")]
    NoPendingConfirmation,

    #[error("用户取消")]
    UserCancelled,

    #[error("工具执行失败: {tool} - {message}")]
    ToolExecutionFailed { tool: String, message: String },

    #[error("速率限制: 请稍后重试")]
    RateLimited,
}
```

### 8.3 限制配置

| 限制项 | 默认值 | 说明 |
|--------|--------|------|
| 单次最大步数 | 10 | 防止无限循环 |
| 单次最大Token | 4000 | 控制成本 |
| 每分钟最大调用 | 30 | 速率限制 |
| 单次查询最大结果 | 50 | 防止数据过载 |
| 批量操作最大数量 | 10 | 防止误操作 |

---

## 9. 实现计划

### 9.1 阶段划分

```
┌─────────────────────────────────────────────────────────────────┐
│                        实现阶段                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Phase 1: 基础设施 (预计 2-3 天)                                │
│  ├─ 创建 agent 模块目录结构                                     │
│  ├─ 实现 ToolDefinition 和 ToolRegistry                        │
│  ├─ 实现 ToolExecutor 基础框架                                  │
│  ├─ 添加 AgentConfig 配置项                                     │
│  └─ 单元测试                                                    │
│                                                                 │
│  Phase 2: Agent核心 (预计 3-4 天)                               │
│  ├─ 实现 AgentCore 执行循环                                     │
│  ├─ 实现 LLM 工具调用解析                                       │
│  ├─ 实现事件发送机制                                            │
│  ├─ 实现确认/取消流程                                           │
│  └─ 集成测试                                                    │
│                                                                 │
│  Phase 3: 前端集成 (预计 2-3 天)                                │
│  ├─ 实现 agentStore                                            │
│  ├─ 实现 agentApi                                              │
│  ├─ 实现 AgentExecutionSteps 组件                              │
│  ├─ 实现 ConfirmationDialog 组件                               │
│  ├─ 集成到 AiChat 组件                                         │
│  └─ 添加设置页面配置项                                          │
│                                                                 │
│  Phase 4: 优化与完善 (预计 2-3 天)                              │
│  ├─ 添加更多工具                                                │
│  ├─ 优化提示词                                                  │
│  ├─ 添加快捷指令                                                │
│  ├─ 完善错误处理                                                │
│  ├─ 性能优化                                                    │
│  └─ 文档更新                                                    │
│                                                                 │
│  总计: 9-13 天                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 9.2 文件结构

```
新增文件:

src-tauri/src/
├── agent/
│   ├── mod.rs                 # 模块导出
│   ├── core.rs                # Agent核心逻辑
│   ├── config.rs              # Agent配置
│   ├── events.rs              # 事件定义
│   ├── errors.rs              # 错误类型
│   ├── tools/
│   │   ├── mod.rs             # 工具模块导出
│   │   ├── registry.rs        # 工具注册表
│   │   ├── executor.rs        # 工具执行器
│   │   ├── definitions.rs     # 工具定义
│   │   └── schemas.rs         # JSON Schema
│   └── security.rs            # 安全检查
├── commands/
│   └── agent_commands.rs      # 新增: Agent命令

src/
├── api/
│   └── agentApi.ts            # 新增: Agent API
├── stores/
│   └── agentStore.ts          # 新增: Agent Store
├── components/
│   └── aiChat/
│       ├── AgentExecutionSteps.vue    # 新增: 执行步骤
│       ├── AgentConfirmDialog.vue     # 新增: 确认对话框
│       └── AgentQuickCommands.vue     # 新增: 快捷指令
└── types/
    └── agent.ts               # 新增: Agent类型定义

修改文件:
├── src-tauri/src/main.rs      # 注册新命令
├── src-tauri/src/lib.rs       # 导出agent模块
├── src/components/aiChat/AiChat.vue       # 集成Agent功能
├── src/components/aiChat/AiChatDrawer.vue # 集成Agent功能
├── src/views/Settings.vue     # 添加Agent配置
└── src/stores/aiStore.ts      # 扩展配置
```

### 9.3 任务清单

#### Phase 1: 基础设施

- [ ] 创建 `src-tauri/src/agent/` 目录结构
- [ ] 实现 `ToolDefinition` 结构体
- [ ] 实现 `ToolRegistry` 工具注册表
- [ ] 实现 `ToolExecutor` 基础框架
- [ ] 复用 `ai_functions_service.rs` 中的工具实现
- [ ] 添加 `AgentConfig` 到设置系统
- [ ] 编写单元测试

#### Phase 2: Agent核心

- [ ] 实现 `AgentCore` 执行循环
- [ ] 实现 `AgentEvent` 事件系统
- [ ] 实现 LLM 工具调用请求构建
- [ ] 实现 LLM 响应解析（tool_calls）
- [ ] 实现确认等待机制
- [ ] 实现取消执行机制
- [ ] 实现 `agent_commands.rs`
- [ ] 编写集成测试

#### Phase 3: 前端集成

- [ ] 创建 `src/types/agent.ts`
- [ ] 创建 `src/api/agentApi.ts`
- [ ] 创建 `src/stores/agentStore.ts`
- [ ] 创建 `AgentExecutionSteps.vue`
- [ ] 创建 `AgentConfirmDialog.vue`
- [ ] 创建 `AgentQuickCommands.vue`
- [ ] 集成到 `AiChat.vue`
- [ ] 添加设置页面配置

#### Phase 4: 优化完善

- [ ] 添加更多工具（应用启动、截图分析等）
- [ ] 优化系统提示词
- [ ] 添加快捷指令预设
- [ ] 完善错误处理和用户提示
- [ ] 性能优化（缓存、并发）
- [ ] 更新文档

---

## 10. 风险评估

### 10.1 技术风险

| 风险 | 影响 | 可能性 | 缓解措施 |
|------|------|--------|----------|
| LLM工具调用不稳定 | 高 | 中 | 添加重试机制、降级到基础模式 |
| 执行循环死循环 | 高 | 低 | 设置最大步数限制、超时机制 |
| 工具执行失败 | 中 | 中 | 完善错误处理、回滚机制 |
| 性能问题 | 中 | 中 | 异步执行、分批处理 |

### 10.2 用户体验风险

| 风险 | 影响 | 可能性 | 缓解措施 |
|------|------|--------|----------|
| 操作不符合预期 | 高 | 中 | 默认确认模式、预览功能 |
| 等待时间过长 | 中 | 中 | 进度展示、可取消 |
| 学习成本高 | 低 | 低 | 渐进式引导、快捷指令 |

### 10.3 兼容性风险

| 风险 | 影响 | 可能性 | 缓解措施 |
|------|------|--------|----------|
| 破坏现有功能 | 高 | 低 | 独立模块、开关控制 |
| AI提供商变更 | 中 | 低 | 抽象接口、多提供商支持 |

---

## 附录

### A. 工具显示名称映射

```typescript
const toolDisplayNames: Record<string, string> = {
  'get_tasks': '获取任务',
  'create_task': '创建任务',
  'update_task': '更新任务',
  'complete_task': '完成任务',
  'delete_task': '删除任务',
  'get_pomodoro_records': '获取番茄钟记录',
  'start_pomodoro': '开始番茄钟',
  'stop_pomodoro': '停止番茄钟',
  'search_sql': '搜索SQL',
  'get_time_stats': '获取时间统计',
  'analyze_efficiency': '效率分析',
  'generate_daily_report': '生成日报',
  'generate_weekly_report': '生成周报',
  'search_apps': '搜索应用',
  'launch_app': '启动应用',
  'launch_workflow': '启动工作流',
  'get_current_context': '获取上下文',
  'get_today_summary': '今日概要',
};
```

### B. 快捷指令预设

```typescript
const quickCommands = [
  { label: '整理今日工作', prompt: '帮我整理今天的工作，生成日报' },
  { label: '生成周报', prompt: '生成本周的工作周报' },
  { label: '效率分析', prompt: '分析一下我这周的工作效率' },
  { label: '待办推荐', prompt: '我现在有空，推荐一个任务给我' },
  { label: '今日概要', prompt: '总结一下今天做了什么' },
];
```

### C. 提示词模板位置

```
src-tauri/src/prompts/
├── agent_system.txt           # Agent系统提示词
├── agent_tool_selection.txt   # 工具选择提示词
└── agent_response.txt         # 响应生成提示词
```

---

## 总结

本方案设计了一个**渐进式、兼容性强、可控性高**的AI智能体系统：

1. **三层模式**：无AI → 基础AI → 智能Agent，用户可按需启用
2. **工具系统**：复用现有功能，统一封装为可调用工具
3. **执行循环**：标准的 ReAct 模式，支持多步骤任务
4. **透明可控**：实时展示执行步骤，支持确认/取消
5. **安全设计**：权限控制、速率限制、沙箱模式

下一步：等待确认后开始实现。
