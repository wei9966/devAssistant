# DevAssistant AI 对话助手 开发文档

> 版本：1.0
>  更新日期：2025-12-25
>  模块定位：软件数据的自然语言交互入口

------

## 目录

- [一、功能概述](https://claude.ai/chat/1d272d42-2987-4874-a96f-72494ca37ea7#一功能概述)
- [二、整体架构](https://claude.ai/chat/1d272d42-2987-4874-a96f-72494ca37ea7#二整体架构)
- [三、Function 定义](https://claude.ai/chat/1d272d42-2987-4874-a96f-72494ca37ea7#三function-定义)
- [四、System Prompt 设计](https://claude.ai/chat/1d272d42-2987-4874-a96f-72494ca37ea7#四system-prompt-设计)
- [五、对话流程](https://claude.ai/chat/1d272d42-2987-4874-a96f-72494ca37ea7#五对话流程)
- [六、UI 设计](https://claude.ai/chat/1d272d42-2987-4874-a96f-72494ca37ea7#六ui-设计)
- [七、对话示例](https://claude.ai/chat/1d272d42-2987-4874-a96f-72494ca37ea7#七对话示例)
- [八、技术实现](https://claude.ai/chat/1d272d42-2987-4874-a96f-72494ca37ea7#八技术实现)
- [九、开发计划](https://claude.ai/chat/1d272d42-2987-4874-a96f-72494ca37ea7#九开发计划)
- [十、注意事项](https://claude.ai/chat/1d272d42-2987-4874-a96f-72494ca37ea7#十注意事项)

------

## 一、功能概述

### 1.1 功能定位

AI 对话助手是 DevAssistant 的智能交互入口，用户可以通过自然语言与软件内所有数据进行交互，实现查询、操作、分析、生成等功能。

**核心价值**：让用户用"说话"的方式操作软件，而不是点击菜单。

### 1.2 能力范围

| 能力类型   | 描述                 | 示例                         |
| ---------- | -------------------- | ---------------------------- |
| **查询类** | 查询软件内各类数据   | "这周我完成了多少任务？"     |
| **操作类** | 执行软件内各种操作   | "帮我创建一个明天到期的任务" |
| **生成类** | 生成日报、周报等内容 | "生成这周的周报"             |
| **分析类** | 分析工作效率和模式   | "分析一下我这周的工作效率"   |
| **搜索类** | 搜索 SQL、任务等记录 | "找一下有 user 表的 SQL"     |

### 1.3 与通用 AI 对话的区别

| 维度     | 通用 AI 对话 | DevAssistant AI 助手 |
| -------- | ------------ | -------------------- |
| 数据访问 | 无           | 可读取软件内所有数据 |
| 操作能力 | 无           | 可执行软件内操作     |
| 上下文   | 仅对话历史   | 对话历史 + 用户数据  |
| 差异化   | 无           | 专属于用户的个人助手 |

------

## 二、整体架构

### 2.1 系统架构图

```
┌─────────────────────────────────────────────────────────────┐
│                      AI 对话助手 UI                          │
│                     (Vue 3 Component)                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      对话管理层                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  • 对话历史管理                                       │   │
│  │  • 上下文构建                                         │   │
│  │  • 消息格式化                                         │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      AI 服务层                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  • API 调用 (OpenAI 兼容)                             │   │
│  │  • Function Calling 处理                              │   │
│  │  • 流式响应处理                                       │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Function 执行层                           │
│  ┌───────────┬───────────┬───────────┬───────────────────┐ │
│  │  查询类    │  操作类    │  生成类    │  分析类           │ │
│  │           │           │           │                   │ │
│  │ get_tasks │create_task│gen_daily  │analyze_efficiency │ │
│  │ get_sql   │update_task│gen_weekly │                   │ │
│  │ get_stats │start_pomo │           │                   │ │
│  └───────────┴───────────┴───────────┴───────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      数据层 (SQLite)                         │
│  ┌─────────┬─────────┬─────────┬─────────┬─────────────┐   │
│  │  Tasks  │SQL Hist │ Stats   │  Logs   │  Pomodoro   │   │
│  └─────────┴─────────┴─────────┴─────────┴─────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 数据流向

```
用户输入
    │
    ▼
┌──────────────────┐
│ 构建请求上下文    │ ← 注入用户数据概况
└──────────────────┘
    │
    ▼
┌──────────────────┐
│ 调用 AI API      │ ← 携带 Functions 定义
└──────────────────┘
    │
    ▼
┌──────────────────┐     ┌──────────────────┐
│ 返回 Function    │────▶│ 执行 Function    │
│ Call?            │     │ (Tauri Command)  │
└──────────────────┘     └──────────────────┘
    │ 否                         │
    ▼                            ▼
┌──────────────────┐     ┌──────────────────┐
│ 直接返回文本回复  │     │ 将结果返回 AI     │
└──────────────────┘     └──────────────────┘
    │                            │
    ▼                            ▼
┌──────────────────────────────────────────┐
│              展示最终回复                  │
└──────────────────────────────────────────┘
```

------

## 三、Function 定义

### 3.1 查询类 Functions

#### 3.1.1 get_tasks - 查询任务

```json
{
  "name": "get_tasks",
  "description": "查询任务列表，支持按状态、象限、日期、关键词筛选",
  "parameters": {
    "type": "object",
    "properties": {
      "status": {
        "type": "string",
        "enum": ["todo", "done", "all"],
        "default": "all",
        "description": "任务状态：todo-待办，done-已完成，all-全部"
      },
      "quadrant": {
        "type": "integer",
        "enum": [1, 2, 3, 4],
        "description": "四象限：1-重要紧急，2-重要不紧急，3-不重要紧急，4-不重要不紧急"
      },
      "date_range": {
        "type": "string",
        "enum": ["today", "yesterday", "this_week", "last_week", "this_month", "custom"],
        "description": "日期范围"
      },
      "start_date": {
        "type": "string",
        "description": "自定义开始日期，格式 YYYY-MM-DD"
      },
      "end_date": {
        "type": "string",
        "description": "自定义结束日期，格式 YYYY-MM-DD"
      },
      "keyword": {
        "type": "string",
        "description": "关键词搜索，匹配任务标题和描述"
      },
      "limit": {
        "type": "integer",
        "default": 20,
        "description": "返回数量限制"
      }
    }
  }
}
```

**Rust 实现接口**：

```rust
#[tauri::command]
async fn ai_get_tasks(
    status: Option<String>,
    quadrant: Option<i32>,
    date_range: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    keyword: Option<String>,
    limit: Option<i32>,
    state: State<'_, AppState>
) -> Result<Vec<Task>, String>
```

**返回数据结构**：

```json
{
  "total": 12,
  "tasks": [
    {
      "id": "task_001",
      "title": "修复登录 bug",
      "description": "用户反馈登录偶尔失败",
      "quadrant": 1,
      "status": "done",
      "due_date": "2025-12-25",
      "completed_at": "2025-12-24 15:30:00",
      "tags": ["bug", "urgent"],
      "pomodoro_count": 2,
      "focus_minutes": 50
    }
  ]
}
```

------

#### 3.1.2 search_sql - 搜索 SQL 历史

```json
{
  "name": "search_sql",
  "description": "搜索 SQL 历史记录，支持按关键词、类型、日期筛选",
  "parameters": {
    "type": "object",
    "properties": {
      "keyword": {
        "type": "string",
        "description": "搜索关键词，如表名、字段名、SQL 片段"
      },
      "sql_type": {
        "type": "string",
        "enum": ["SELECT", "UPDATE", "INSERT", "DELETE", "CREATE", "ALTER", "all"],
        "default": "all",
        "description": "SQL 类型"
      },
      "date_range": {
        "type": "string",
        "enum": ["today", "this_week", "this_month", "all"],
        "default": "all"
      },
      "favorite_only": {
        "type": "boolean",
        "default": false,
        "description": "是否只查询收藏的 SQL"
      },
      "limit": {
        "type": "integer",
        "default": 10
      }
    }
  }
}
```

**返回数据结构**：

```json
{
  "total": 5,
  "sql_list": [
    {
      "id": "sql_001",
      "content": "SELECT u.name, o.* FROM user u JOIN orders o ON u.id = o.user_id WHERE o.status = 1",
      "sql_type": "SELECT",
      "tables": ["user", "orders"],
      "is_favorite": true,
      "created_at": "2025-12-20 14:30:00",
      "note": "查询用户订单"
    }
  ]
}
```

------

#### 3.1.3 get_time_stats - 获取时间统计

```json
{
  "name": "get_time_stats",
  "description": "获取应用使用时间统计数据",
  "parameters": {
    "type": "object",
    "properties": {
      "date_range": {
        "type": "string",
        "enum": ["today", "yesterday", "this_week", "last_week", "this_month"],
        "default": "today"
      },
      "group_by": {
        "type": "string",
        "enum": ["app", "category", "hour", "day"],
        "default": "category",
        "description": "分组方式：app-按应用，category-按类别，hour-按小时，day-按天"
      }
    }
  }
}
```

**返回数据结构**：

```json
{
  "date_range": "today",
  "total_minutes": 494,
  "active_time": "08:55 - 17:09",
  "by_category": [
    { "category": "编程", "minutes": 182, "percentage": 36.8, "apps": ["VS Code", "DataGrip"] },
    { "category": "文档", "minutes": 97, "percentage": 19.6, "apps": ["Word", "Notion"] },
    { "category": "浏览", "minutes": 55, "percentage": 11.1, "apps": ["Chrome"] },
    { "category": "会议", "minutes": 19, "percentage": 3.8, "apps": ["腾讯会议"] },
    { "category": "其他", "minutes": 59, "percentage": 11.9, "apps": ["微信", "其他"] }
  ],
  "by_app": [
    { "app": "VS Code", "minutes": 150, "percentage": 30.4 },
    { "app": "Chrome", "minutes": 55, "percentage": 11.1 }
  ]
}
```

------

#### 3.1.4 get_pomodoro_records - 获取番茄钟记录

```json
{
  "name": "get_pomodoro_records",
  "description": "获取番茄钟专注记录",
  "parameters": {
    "type": "object",
    "properties": {
      "date_range": {
        "type": "string",
        "enum": ["today", "this_week", "this_month"],
        "default": "today"
      },
      "task_id": {
        "type": "string",
        "description": "关联的任务ID，不传则查询全部"
      }
    }
  }
}
```

**返回数据结构**：

```json
{
  "total_count": 8,
  "total_minutes": 200,
  "records": [
    {
      "id": "pomo_001",
      "start_time": "2025-12-25 09:00:00",
      "end_time": "2025-12-25 09:25:00",
      "duration_minutes": 25,
      "task_id": "task_001",
      "task_title": "修复登录 bug",
      "completed": true
    }
  ],
  "by_task": [
    { "task_id": "task_001", "task_title": "修复登录 bug", "count": 3, "minutes": 75 }
  ]
}
```

------

#### 3.1.5 get_daily_logs - 获取日报记录

```json
{
  "name": "get_daily_logs",
  "description": "获取历史日报记录",
  "parameters": {
    "type": "object",
    "properties": {
      "date": {
        "type": "string",
        "description": "指定日期 YYYY-MM-DD，不传则获取最近的"
      },
      "date_range": {
        "type": "string",
        "enum": ["this_week", "last_week", "this_month"]
      },
      "limit": {
        "type": "integer",
        "default": 7
      }
    }
  }
}
```

------

### 3.2 操作类 Functions

#### 3.2.1 create_task - 创建任务

```json
{
  "name": "create_task",
  "description": "创建新的待办任务",
  "parameters": {
    "type": "object",
    "properties": {
      "title": {
        "type": "string",
        "description": "任务标题（必填）"
      },
      "description": {
        "type": "string",
        "description": "任务描述"
      },
      "quadrant": {
        "type": "integer",
        "enum": [1, 2, 3, 4],
        "default": 2,
        "description": "四象限：1-重要紧急，2-重要不紧急，3-不重要紧急，4-不重要不紧急"
      },
      "due_date": {
        "type": "string",
        "description": "截止日期 YYYY-MM-DD"
      },
      "due_time": {
        "type": "string",
        "description": "截止时间 HH:mm"
      },
      "tags": {
        "type": "array",
        "items": { "type": "string" },
        "description": "标签列表"
      },
      "reminder": {
        "type": "boolean",
        "default": false,
        "description": "是否设置提醒"
      }
    },
    "required": ["title"]
  }
}
```

**返回数据结构**：

```json
{
  "success": true,
  "task": {
    "id": "task_new_001",
    "title": "编写技术文档",
    "quadrant": 2,
    "due_date": "2025-12-26",
    "created_at": "2025-12-25 10:30:00"
  },
  "message": "任务创建成功"
}
```

------

#### 3.2.2 update_task - 更新任务

```json
{
  "name": "update_task",
  "description": "更新任务状态或内容",
  "parameters": {
    "type": "object",
    "properties": {
      "task_id": {
        "type": "string",
        "description": "任务ID（必填）"
      },
      "title": {
        "type": "string",
        "description": "新标题"
      },
      "status": {
        "type": "string",
        "enum": ["todo", "done"],
        "description": "任务状态"
      },
      "quadrant": {
        "type": "integer",
        "enum": [1, 2, 3, 4]
      },
      "due_date": {
        "type": "string"
      },
      "description": {
        "type": "string"
      },
      "tags": {
        "type": "array",
        "items": { "type": "string" }
      }
    },
    "required": ["task_id"]
  }
}
```

------

#### 3.2.3 delete_task - 删除任务

```json
{
  "name": "delete_task",
  "description": "删除指定任务（需要确认）",
  "parameters": {
    "type": "object",
    "properties": {
      "task_id": {
        "type": "string",
        "description": "任务ID（必填）"
      },
      "confirm": {
        "type": "boolean",
        "default": false,
        "description": "是否确认删除"
      }
    },
    "required": ["task_id"]
  }
}
```

------

#### 3.2.4 start_pomodoro - 开始番茄钟

```json
{
  "name": "start_pomodoro",
  "description": "开始一个番茄钟计时",
  "parameters": {
    "type": "object",
    "properties": {
      "duration_minutes": {
        "type": "integer",
        "default": 25,
        "description": "专注时长（分钟）"
      },
      "task_id": {
        "type": "string",
        "description": "关联的任务ID"
      },
      "task_title": {
        "type": "string",
        "description": "如果没有 task_id，可以直接传任务标题来匹配"
      }
    }
  }
}
```

**返回数据结构**：

```json
{
  "success": true,
  "pomodoro": {
    "id": "pomo_new_001",
    "duration_minutes": 25,
    "task_id": "task_001",
    "task_title": "修复登录 bug",
    "start_time": "2025-12-25 10:30:00",
    "expected_end_time": "2025-12-25 10:55:00"
  },
  "message": "番茄钟已开始，专注 25 分钟"
}
```

------

#### 3.2.5 stop_pomodoro - 停止番茄钟

```json
{
  "name": "stop_pomodoro",
  "description": "停止当前进行中的番茄钟",
  "parameters": {
    "type": "object",
    "properties": {
      "mark_completed": {
        "type": "boolean",
        "default": false,
        "description": "是否标记为完成（即使没到时间）"
      }
    }
  }
}
```

------

#### 3.2.6 favorite_sql - 收藏/取消收藏 SQL

```json
{
  "name": "favorite_sql",
  "description": "收藏或取消收藏 SQL 记录",
  "parameters": {
    "type": "object",
    "properties": {
      "sql_id": {
        "type": "string",
        "description": "SQL 记录ID（必填）"
      },
      "is_favorite": {
        "type": "boolean",
        "description": "true-收藏，false-取消收藏"
      }
    },
    "required": ["sql_id", "is_favorite"]
  }
}
```

------

### 3.3 生成类 Functions

#### 3.3.1 generate_daily_report - 生成日报

```json
{
  "name": "generate_daily_report",
  "description": "基于已完成任务和时间统计生成工作日报",
  "parameters": {
    "type": "object",
    "properties": {
      "date": {
        "type": "string",
        "default": "today",
        "description": "日期，today/yesterday 或 YYYY-MM-DD"
      },
      "style": {
        "type": "string",
        "enum": ["detailed", "simple", "formal"],
        "default": "simple",
        "description": "风格：detailed-详细，simple-简洁，formal-正式"
      },
      "include_time_stats": {
        "type": "boolean",
        "default": true,
        "description": "是否包含时间统计"
      },
      "include_pomodoro": {
        "type": "boolean",
        "default": true,
        "description": "是否包含番茄钟记录"
      }
    }
  }
}
```

**处理流程**：

```
1. 获取指定日期的已完成任务
2. 获取时间统计数据
3. 获取番茄钟记录
4. 调用 AI 生成结构化日报
5. 返回日报内容
```

**返回数据结构**：

```json
{
  "date": "2025-12-25",
  "report": {
    "summary": "今日完成 5 项任务，专注工作 4.5 小时",
    "work_content": [
      {
        "category": "系统配置与维护",
        "items": [
          "整理所有工厂的排产大屏地址（协作、配置更新）",
          "修改生产节拍大屏系数（数据持久化、配置更新）"
        ]
      },
      {
        "category": "功能优化与修复",
        "items": [
          "修复器械商下单备包所属医院保存需多遍的问题"
        ]
      }
    ],
    "time_stats": {
      "total_hours": 8.2,
      "active_period": "08:55 - 17:09",
      "breakdown": [
        { "category": "编程", "hours": 3.0, "percentage": 37 },
        { "category": "文档", "hours": 1.6, "percentage": 20 }
      ]
    },
    "highlights": [
      "完成排产大屏配置更新，覆盖所有工厂",
      "解决了一个影响用户体验的关键 bug"
    ]
  },
  "markdown": "## 2025-12-25 工作日报\n\n### 工作内容\n..."
}
```

------

#### 3.3.2 generate_weekly_report - 生成周报

```json
{
  "name": "generate_weekly_report",
  "description": "基于本周工作数据生成周报",
  "parameters": {
    "type": "object",
    "properties": {
      "week": {
        "type": "string",
        "default": "this_week",
        "description": "this_week/last_week 或 YYYY-WW 格式"
      },
      "style": {
        "type": "string",
        "enum": ["detailed", "simple", "formal"],
        "default": "simple"
      },
      "include_comparison": {
        "type": "boolean",
        "default": true,
        "description": "是否包含与上周的对比"
      },
      "include_next_week_plan": {
        "type": "boolean",
        "default": true,
        "description": "是否包含下周计划（需要从待办任务获取）"
      }
    }
  }
}
```

**返回数据结构**：

```json
{
  "week": "2025-W52",
  "date_range": "2025-12-23 ~ 2025-12-27",
  "report": {
    "summary": "本周完成 12 项任务，总工作时长 42 小时",
    "completed_tasks": {
      "total": 12,
      "by_category": [
        { "category": "系统配置与维护", "count": 4 },
        { "category": "功能优化与修复", "count": 5 },
        { "category": "文档与计划", "count": 3 }
      ],
      "highlights": [
        "完成所有工厂排产大屏配置",
        "修复 3 个关键 bug"
      ]
    },
    "time_stats": {
      "total_hours": 42,
      "by_category": [
        { "category": "编程", "hours": 24, "percentage": 57 },
        { "category": "会议", "hours": 8, "percentage": 19 },
        { "category": "文档", "hours": 10, "percentage": 24 }
      ]
    },
    "comparison_with_last_week": {
      "tasks_diff": "+2",
      "hours_diff": "-3",
      "efficiency_change": "+12%"
    },
    "next_week_plan": [
      "完成用户模块重构",
      "编写技术文档"
    ]
  },
  "markdown": "## 本周工作周报 (12.23 - 12.27)\n\n..."
}
```

------

### 3.4 分析类 Functions

#### 3.4.1 analyze_efficiency - 效率分析

```json
{
  "name": "analyze_efficiency",
  "description": "分析工作效率，给出数据洞察和改进建议",
  "parameters": {
    "type": "object",
    "properties": {
      "date_range": {
        "type": "string",
        "enum": ["this_week", "last_week", "this_month", "last_month"],
        "default": "this_week"
      },
      "focus": {
        "type": "string",
        "enum": ["time", "tasks", "focus", "all"],
        "default": "all",
        "description": "分析重点：time-时间分配，tasks-任务完成，focus-专注度，all-综合"
      }
    }
  }
}
```

**返回数据结构**：

```json
{
  "date_range": "this_week",
  "analysis": {
    "overview": {
      "total_work_hours": 42,
      "total_tasks_completed": 12,
      "total_pomodoro": 28,
      "avg_daily_focus_hours": 4.2
    },
    "time_analysis": {
      "most_productive_day": "周三",
      "most_productive_hour": "10:00-11:00",
      "coding_percentage": 57,
      "meeting_percentage": 19,
      "fragmentation_score": 35,
      "insight": "你的编程时间集中在上午，效率最高"
    },
    "task_analysis": {
      "completion_rate": 85,
      "quadrant_distribution": {
        "q1": 25,
        "q2": 42,
        "q3": 17,
        "q4": 16
      },
      "avg_task_duration": 45,
      "insight": "重要不紧急的任务占比最高，说明你在关注长期价值"
    },
    "focus_analysis": {
      "avg_pomodoro_per_day": 5.6,
      "completion_rate": 89,
      "best_focus_day": "周二",
      "insight": "你的番茄钟完成率很高，专注力不错"
    },
    "suggestions": [
      "建议把重要的编程任务安排在上午 10 点左右",
      "周五的会议较多，可以考虑集中处理",
      "可以尝试减少第四象限任务的时间投入"
    ]
  }
}
```

------

#### 3.4.2 compare_periods - 周期对比

```json
{
  "name": "compare_periods",
  "description": "对比两个时间段的工作数据",
  "parameters": {
    "type": "object",
    "properties": {
      "period1": {
        "type": "string",
        "description": "第一个时间段：this_week/last_week/this_month/last_month"
      },
      "period2": {
        "type": "string",
        "description": "第二个时间段"
      },
      "metrics": {
        "type": "array",
        "items": {
          "type": "string",
          "enum": ["tasks", "hours", "focus", "efficiency"]
        },
        "default": ["tasks", "hours", "focus"],
        "description": "对比指标"
      }
    },
    "required": ["period1", "period2"]
  }
}
```

------

### 3.5 Functions 汇总表

| 类别     | Function 名称          | 功能描述       | 优先级 |
| -------- | ---------------------- | -------------- | ------ |
| **查询** | get_tasks              | 查询任务列表   | P0     |
| **查询** | search_sql             | 搜索 SQL 历史  | P0     |
| **查询** | get_time_stats         | 获取时间统计   | P0     |
| **查询** | get_pomodoro_records   | 获取番茄钟记录 | P1     |
| **查询** | get_daily_logs         | 获取日报记录   | P1     |
| **操作** | create_task            | 创建任务       | P0     |
| **操作** | update_task            | 更新任务       | P0     |
| **操作** | delete_task            | 删除任务       | P2     |
| **操作** | start_pomodoro         | 开始番茄钟     | P1     |
| **操作** | stop_pomodoro          | 停止番茄钟     | P1     |
| **操作** | favorite_sql           | 收藏 SQL       | P2     |
| **生成** | generate_daily_report  | 生成日报       | P0     |
| **生成** | generate_weekly_report | 生成周报       | P0     |
| **分析** | analyze_efficiency     | 效率分析       | P1     |
| **分析** | compare_periods        | 周期对比       | P2     |

------

## 四、System Prompt 设计

### 4.1 完整 System Prompt

```markdown
你是 DevAssistant 的 AI 助手，一个专为开发者设计的效率工具的智能助理。你可以访问用户在软件中的所有数据，并帮助用户完成各种操作。

## 你的身份

- 名称：DevAssistant AI 助手
- 角色：用户的个人工作助理
- 特点：了解用户的任务、工作时间、专注记录等数据

## 你的能力

### 📋 任务管理
- 查询任务：按状态、象限、日期、关键词筛选任务列表
- 创建任务：帮用户快速创建待办事项，支持设置优先级、截止日期
- 更新任务：修改任务状态、优先级、内容
- 删除任务：删除不需要的任务（需确认）

### 🔍 数据查询
- SQL 历史：搜索用户保存的 SQL 语句，支持按关键词、类型筛选
- 时间统计：查看应用使用时长分布，按天、周、月统计
- 番茄钟记录：查看专注时间记录和统计

### 📊 报告生成
- 日报：基于已完成任务和时间统计自动生成
- 周报：汇总一周工作成果，支持与上周对比
- 支持不同风格：详细、简洁、正式

### 📈 效率分析
- 分析工作时间分布
- 分析任务完成情况
- 分析专注度
- 给出改进建议

### 🍅 专注工具
- 开始番茄钟：可关联任务
- 停止番茄钟

## 回复原则

1. **先理解，再执行**：确保理解用户意图后再调用相应功能
2. **结果要清晰**：查询结果要结构化呈现，突出重点数据
3. **操作要确认**：删除等敏感操作前先确认
4. **语气要友好**：用中文回复，像朋友一样交流
5. **主动引导**：适时给出后续建议或相关操作提示

## 数据呈现格式

- 任务列表：使用编号或分组展示，标注状态和截止日期
- 时间统计：使用进度条或百分比直观展示
- SQL 记录：展示关键信息，提供复制按钮
- 报告内容：结构化分段，突出重点

## 限制

- 只能访问 DevAssistant 软件内的数据
- 不能访问用户电脑的其他文件或应用
- 不能执行软件功能以外的操作

## 当前上下文

- 当前日期：{{current_date}}
- 当前时间：{{current_time}}
- 待办任务数：{{todo_count}}
- 今日已完成：{{today_done_count}}
- 今日专注时长：{{today_focus_minutes}} 分钟
- 本周完成任务：{{weekly_done_count}}

## 示例对话

用户：我还有多少任务没做完？
助手：[调用 get_tasks(status="todo")] 你目前有 8 个待办任务，其中 2 个是重要紧急的...

用户：帮我创建一个任务，明天之前完成技术文档
助手：[调用 create_task(title="完成技术文档", due_date="明天", quadrant=2)] 已创建任务「完成技术文档」，截止日期明天，优先级设为重要不紧急。需要开始一个番茄钟来专注处理吗？

用户：生成这周的周报
助手：[调用 generate_weekly_report(week="this_week")] 好的，我来为你生成本周周报...
```

### 4.2 动态上下文注入

每次对话前，注入用户当前数据概况：

```typescript
async function buildSystemPrompt(): Promise<string> {
  // 获取实时数据
  const stats = await invoke('get_dashboard_stats');
  
  const context = {
    current_date: formatDate(new Date()),
    current_time: formatTime(new Date()),
    todo_count: stats.todoCount,
    today_done_count: stats.todayDoneCount,
    today_focus_minutes: stats.todayFocusMinutes,
    weekly_done_count: stats.weeklyDoneCount
  };
  
  // 替换模板变量
  return SYSTEM_PROMPT.replace(/\{\{(\w+)\}\}/g, (_, key) => context[key] || '');
}
```

------

## 五、对话流程

### 5.1 完整对话流程图

```
┌─────────────────────────────────────────────────────────────┐
│                        用户输入                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   构建请求消息                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  1. System Prompt (含动态上下文)                      │   │
│  │  2. 历史对话记录 (最近 N 轮)                          │   │
│  │  3. 用户当前输入                                      │   │
│  │  4. Functions 定义                                   │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    调用 AI API                              │
│                  (OpenAI 兼容接口)                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  响应类型判断    │
                    └─────────────────┘
                              │
            ┌─────────────────┼─────────────────┐
            ▼                 ▼                 ▼
    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │  纯文本回复   │  │ Function Call │  │    错误     │
    └──────────────┘  └──────────────┘  └──────────────┘
            │                 │                 │
            ▼                 ▼                 ▼
    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │  直接展示    │  │ 执行 Function │  │  错误处理    │
    └──────────────┘  └──────────────┘  └──────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ 获取执行结果     │
                    └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ 将结果返回给 AI  │
                    │ 生成最终回复     │
                    └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   展示最终回复   │
                    └─────────────────┘
```

### 5.2 Function Call 执行流程

```typescript
async function handleFunctionCall(
  functionCall: { name: string; arguments: string }
): Promise<any> {
  const { name, arguments: argsStr } = functionCall;
  const args = JSON.parse(argsStr);
  
  // 映射到 Tauri Command
  const commandMap: Record<string, string> = {
    'get_tasks': 'ai_get_tasks',
    'create_task': 'ai_create_task',
    'search_sql': 'ai_search_sql',
    'get_time_stats': 'ai_get_time_stats',
    'generate_daily_report': 'ai_generate_daily_report',
    // ...
  };
  
  const command = commandMap[name];
  if (!command) {
    throw new Error(`Unknown function: ${name}`);
  }
  
  // 调用 Rust 后端
  const result = await invoke(command, args);
  return result;
}
```

### 5.3 多轮 Function Call 处理

某些复杂请求可能需要多次 Function Call：

```
用户：帮我看看这周完成最多任务的那天，然后生成那天的日报

AI 第一次调用：get_tasks(date_range="this_week", status="done")
→ 返回任务列表，分析得出周三完成最多

AI 第二次调用：generate_daily_report(date="2025-12-24")
→ 返回周三的日报

AI 最终回复：这周三你完成了 5 个任务，是效率最高的一天。这是当天的日报：...
```

------

## 六、UI 设计

### 6.1 整体布局

布局样式请参考D:\desk_code\DevAssistant\example\chat.txt

```
┌─────────────────────────────────────────────────────────────┐
│  🤖 AI 助手                               [设置] [历史] [清空] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                      消息区域                                │
│                    (可滚动区域)                              │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 🤖 欢迎消息                                          │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 👤 用户消息                                          │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 🤖 AI 回复（可能包含富文本、图表等）                    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  💡 快捷提问：                                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │
│  │ 今日任务  │ │ 生成日报  │ │ 效率分析  │ │ 搜索SQL     │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────┐       │
│  │ 输入你想问的...                                   │ [发送] │
│  └─────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 消息类型组件

#### 6.2.1 普通文本消息

```vue
<template>
  <div class="message" :class="{ 'user': isUser, 'assistant': !isUser }">
    <div class="avatar">
      {{ isUser ? '👤' : '🤖' }}
    </div>
    <div class="content">
      <div class="text" v-html="renderedContent"></div>
      <div class="time">{{ formatTime(timestamp) }}</div>
    </div>
  </div>
</template>
```

#### 6.2.2 任务列表消息

```vue
<template>
  <div class="task-list-message">
    <div class="header">
      <span class="title">📋 任务列表</span>
      <span class="count">共 {{ tasks.length }} 个</span>
    </div>
    <div class="tasks">
      <div v-for="task in tasks" :key="task.id" class="task-item">
        <div class="quadrant-badge" :class="'q' + task.quadrant">
          {{ quadrantLabels[task.quadrant] }}
        </div>
        <div class="task-info">
          <div class="task-title">{{ task.title }}</div>
          <div class="task-meta">
            <span v-if="task.due_date">📅 {{ task.due_date }}</span>
            <span v-if="task.tags?.length">🏷️ {{ task.tags.join(', ') }}</span>
          </div>
        </div>
        <div class="task-actions">
          <button @click="completeTask(task.id)">✓ 完成</button>
        </div>
      </div>
    </div>
  </div>
</template>
```

#### 6.2.3 SQL 列表消息

```vue
<template>
  <div class="sql-list-message">
    <div class="header">
      <span class="title">🔍 SQL 搜索结果</span>
      <span class="count">找到 {{ sqlList.length }} 条</span>
    </div>
    <div class="sql-items">
      <div v-for="sql in sqlList" :key="sql.id" class="sql-item">
        <div class="sql-header">
          <span class="sql-type">{{ sql.sql_type }}</span>
          <span class="sql-date">{{ sql.created_at }}</span>
          <span v-if="sql.is_favorite" class="favorite">⭐</span>
        </div>
        <pre class="sql-content"><code>{{ sql.content }}</code></pre>
        <div class="sql-actions">
          <button @click="copySQL(sql.content)">📋 复制</button>
          <button @click="toggleFavorite(sql.id)">
            {{ sql.is_favorite ? '取消收藏' : '收藏' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
```

#### 6.2.4 时间统计消息

```vue
<template>
  <div class="time-stats-message">
    <div class="header">
      <span class="title">📊 时间统计</span>
      <span class="period">{{ dateRange }}</span>
    </div>
    <div class="stats-overview">
      <div class="stat-item">
        <div class="stat-value">{{ totalHours }}h</div>
        <div class="stat-label">总时长</div>
      </div>
      <div class="stat-item">
        <div class="stat-value">{{ activePeriod }}</div>
        <div class="stat-label">活动时段</div>
      </div>
    </div>
    <div class="category-breakdown">
      <div v-for="cat in categories" :key="cat.category" class="category-item">
        <div class="category-name">{{ cat.category }}</div>
        <div class="progress-bar">
          <div class="progress" :style="{ width: cat.percentage + '%' }"></div>
        </div>
        <div class="category-value">{{ cat.hours }}h ({{ cat.percentage }}%)</div>
      </div>
    </div>
  </div>
</template>
```

#### 6.2.5 报告消息

```vue
<template>
  <div class="report-message">
    <div class="header">
      <span class="title">📝 {{ reportType === 'daily' ? '日报' : '周报' }}</span>
      <span class="date">{{ reportDate }}</span>
    </div>
    <div class="report-content" v-html="renderedMarkdown"></div>
    <div class="report-actions">
      <button @click="copyReport">📋 复制</button>
      <button @click="exportReport">📤 导出</button>
      <button @click="editReport">✏️ 编辑</button>
    </div>
  </div>
</template>
```

### 6.3 加载状态

```vue
<template>
  <div class="loading-message">
    <div class="avatar">🤖</div>
    <div class="content">
      <div class="thinking">
        <span class="dot"></span>
        <span class="dot"></span>
        <span class="dot"></span>
      </div>
      <div class="status" v-if="currentAction">
        {{ currentAction }}
      </div>
    </div>
  </div>
</template>

<!-- 状态示例：
  - "正在查询任务..."
  - "正在生成日报..."
  - "正在分析数据..."
-->
```

### 6.4 快捷提问配置

```typescript
const quickQuestions = [
  {
    label: '今日任务',
    icon: '📋',
    prompt: '我今天有哪些任务？'
  },
  {
    label: '生成日报',
    icon: '📝',
    prompt: '帮我生成今天的日报'
  },
  {
    label: '效率分析',
    icon: '📈',
    prompt: '分析一下我这周的工作效率'
  },
  {
    label: '搜索SQL',
    icon: '🔍',
    prompt: '搜索SQL：',  // 需要用户补充关键词
    needInput: true
  },
  {
    label: '开始专注',
    icon: '🍅',
    prompt: '开始一个 25 分钟的番茄钟'
  },
  {
    label: '本周周报',
    icon: '📊',
    prompt: '生成这周的周报'
  }
];
```

------

## 七、对话示例

### 7.1 任务查询与操作

```
用户：我还有多少任务没做完？

AI：你目前有 8 个待办任务：

📌 重要紧急 (2个)
• 修复线上 bug — ⏰ 明天到期
• 提交周报 — ⏰ 今天到期

📋 重要不紧急 (4个)
• 优化数据库查询
• 编写单元测试
• 重构用户模块
• 更新技术文档

📝 其他 (2个)
• 整理会议纪要
• 回复邮件

建议优先处理「提交周报」，今天就到期了。需要我帮你把它标记为完成吗？

---

用户：把提交周报标记完成

AI：✅ 已完成任务「提交周报」

📊 今日进度：
• 已完成：3 个
• 待办：7 个

继续加油！还有什么需要帮忙的吗？

---

用户：帮我创建一个任务，下周三之前完成接口文档

AI：✅ 已创建任务：

📋 接口文档编写
• 优先级：重要不紧急
• 截止日期：2025-12-31（下周三）
• 状态：待办

需要现在开始一个番茄钟来处理这个任务吗？
```

### 7.2 SQL 搜索

```
用户：帮我找一下之前写的那个查用户订单的 SQL

AI：找到 3 条相关 SQL：

1️⃣ 📅 12-20  ⭐已收藏
   SELECT u.name, u.phone, o.* 
   FROM user u 
   JOIN orders o ON u.id = o.user_id 
   WHERE o.status = 1
   ORDER BY o.create_time DESC
   [📋 复制]

2️⃣ 📅 12-18
   SELECT * FROM orders 
   WHERE user_id = 123 
   AND create_time > '2025-12-01'
   [📋 复制]

3️⃣ 📅 12-15
   SELECT count(*) as order_count, user_id 
   FROM orders 
   GROUP BY user_id
   [📋 复制]

第一条是 JOIN 查询，应该是你要找的。要我帮你收藏其他几条吗？

---

用户：把第二条也收藏一下

AI：⭐ 已收藏 SQL：
SELECT * FROM orders WHERE user_id = 123...

你现在有 5 条收藏的 SQL，可以在 SQL 历史页面的收藏标签下快速找到。
```

### 7.3 生成日报

```
用户：帮我生成今天的日报

AI：好的，我来整理今天的工作内容...

📝 2025-12-25 工作日报

━━━━━━━━━━━━━━━━━━━━━━━

【工作内容】

▎系统配置与维护
  • 整理所有工厂的排产大屏地址（协作、配置更新）
  • 修改生产节拍大屏系数（数据持久化、配置更新）
  • 更新松江排产预警时间（数据维护/导入、松江）

▎功能优化与修复
  • 修复器械商下单备包所属医院保存需多遍的问题
   （事务处理、性能优化、数据持久化）

▎文档与计划
  • 整理周报和下周计划

━━━━━━━━━━━━━━━━━━━━━━━

【工作时间】

• 活动时段：08:55 - 17:09（共 8 小时 14 分钟）
• 主要工具：微信、DevAssistant、Chrome、DataGrip、VS Code

📊 时间分布：
  编程     ████████████░░░░  182分钟 (44%)
  文档     ██████░░░░░░░░░░   97分钟 (24%)
  其他     ███░░░░░░░░░░░░░   59分钟 (14%)
  浏览     ███░░░░░░░░░░░░░   55分钟 (13%)
  会议     █░░░░░░░░░░░░░░░   19分钟 (5%)

━━━━━━━━━━━━━━━━━━━━━━━

[📋 复制] [📤 导出] [✏️ 编辑]

需要调整格式或补充内容吗？
```

### 7.4 效率分析

```
用户：分析一下我这周的工作效率

AI：好的，我来分析你这周的工作数据...

📈 本周效率分析报告

━━━━━━━━━━━━━━━━━━━━━━━

【总览】
• 总工作时长：42 小时
• 完成任务：12 个
• 番茄钟：28 个（完成率 89%）
• 日均专注：4.2 小时

━━━━━━━━━━━━━━━━━━━━━━━

【时间分析】

⏰ 效率最高时段：上午 10:00-11:00
📅 效率最高日期：周三（完成 5 个任务）

时间分配：
  编程     ████████████░░░░  24h (57%)
  文档     ██████░░░░░░░░░░  10h (24%)
  会议     ████░░░░░░░░░░░░   8h (19%)

💡 洞察：你的编程时间主要集中在上午，这是你的高效时段

━━━━━━━━━━━━━━━━━━━━━━━

【任务分析】

完成率：85%（12/14）

按象限分布：
  重要紧急     ███░░  25%
  重要不紧急   █████  42%  ← 占比最高 👍
  不重要紧急   ██░░░  17%
  不重要不紧急 ██░░░  16%

💡 洞察：重要不紧急任务占比最高，说明你在关注长期价值

━━━━━━━━━━━━━━━━━━━━━━━

【与上周对比】
• 完成任务：+2 个 ↑
• 工作时长：-3 小时 ↓
• 专注效率：+12% ↑

━━━━━━━━━━━━━━━━━━━━━━━

【改进建议】

1. 💡 建议把重要的编程任务安排在上午 10 点左右
2. 💡 周五会议较多，可以考虑提前处理紧急任务
3. 💡 可以适当减少第四象限任务的时间投入

━━━━━━━━━━━━━━━━━━━━━━━

总体来说，这周效率不错！继续保持 💪
```

### 7.5 番茄钟操作

```
用户：开始一个番茄钟，处理修复 bug 的任务

AI：🍅 番茄钟已开始！

┌─────────────────────────────────┐
│  专注任务：修复登录 bug          │
│  时长：25 分钟                   │
│  预计结束：10:55                 │
│                                 │
│        🍅 25:00                 │
│                                 │
│  [暂停] [停止]                   │
└─────────────────────────────────┘

专心工作，25 分钟后提醒你休息！

---

（25分钟后）

AI：🎉 番茄钟完成！

你刚刚完成了一个 25 分钟的专注时段：
• 关联任务：修复登录 bug
• 今日番茄钟：第 3 个
• 今日总专注：75 分钟

建议休息 5 分钟 ☕

需要把「修复登录 bug」标记为完成吗？
```

### 7.6 复杂查询

```
用户：这周我在哪个任务上花的时间最多？

AI：让我查询一下这周的专注记录...

📊 本周任务专注时长排名

1️⃣ 用户模块重构
   🍅 8 个番茄钟 · ⏱️ 200 分钟
   ████████████████████ 100%

2️⃣ 编写单元测试
   🍅 5 个番茄钟 · ⏱️ 125 分钟
   ████████████░░░░░░░░ 63%

3️⃣ 修复登录 bug
   🍅 3 个番茄钟 · ⏱️ 75 分钟
   ███████░░░░░░░░░░░░░ 38%

4️⃣ 更新技术文档
   🍅 2 个番茄钟 · ⏱️ 50 分钟
   █████░░░░░░░░░░░░░░░ 25%

「用户模块重构」占用了你最多的时间（200 分钟）。
这个任务完成了吗？需要我帮你更新状态吗？
```

------

## 八、技术实现

### 8.1 前端核心代码

#### 8.1.1 Store 定义 (Pinia)

```typescript
// stores/aiChat.ts
import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  type?: 'text' | 'tasks' | 'sql' | 'stats' | 'report';
  data?: any;  // 结构化数据
  loading?: boolean;
}

interface ChatState {
  messages: Message[];
  isLoading: boolean;
  currentAction: string | null;
  error: string | null;
}

export const useAiChatStore = defineStore('aiChat', {
  state: (): ChatState => ({
    messages: [],
    isLoading: false,
    currentAction: null,
    error: null
  }),

  actions: {
    // 发送消息
    async sendMessage(content: string) {
      // 添加用户消息
      const userMessage: Message = {
        id: generateId(),
        role: 'user',
        content,
        timestamp: new Date()
      };
      this.messages.push(userMessage);

      // 添加 AI 占位消息
      const assistantMessage: Message = {
        id: generateId(),
        role: 'assistant',
        content: '',
        timestamp: new Date(),
        loading: true
      };
      this.messages.push(assistantMessage);

      this.isLoading = true;
      this.error = null;

      try {
        // 构建对话历史
        const history = this.messages
          .filter(m => !m.loading)
          .slice(-10)  // 最近 10 轮
          .map(m => ({
            role: m.role,
            content: m.content
          }));

        // 调用后端
        const response = await invoke('ai_chat', {
          messages: history
        });

        // 更新 AI 消息
        const index = this.messages.findIndex(m => m.id === assistantMessage.id);
        if (index !== -1) {
          this.messages[index] = {
            ...assistantMessage,
            content: response.content,
            type: response.type,
            data: response.data,
            loading: false
          };
        }
      } catch (err) {
        this.error = err.message;
        // 移除失败的消息
        this.messages = this.messages.filter(m => m.id !== assistantMessage.id);
      } finally {
        this.isLoading = false;
        this.currentAction = null;
      }
    },

    // 清空对话
    clearMessages() {
      this.messages = [];
    },

    // 设置当前动作状态
    setCurrentAction(action: string | null) {
      this.currentAction = action;
    }
  },

  getters: {
    // 获取对话历史（不包含系统消息）
    chatHistory: (state) => {
      return state.messages.filter(m => m.role !== 'system');
    }
  }
});
```

#### 8.1.2 主对话组件

```vue
<!-- components/AiChat/index.vue -->
<template>
  <div class="ai-chat">
    <!-- 头部 -->
    <div class="chat-header">
      <div class="title">
        <span class="icon">🤖</span>
        <span>AI 助手</span>
      </div>
      <div class="actions">
        <n-button quaternary size="small" @click="showSettings">
          <template #icon><SettingsIcon /></template>
        </n-button>
        <n-button quaternary size="small" @click="showHistory">
          <template #icon><HistoryIcon /></template>
        </n-button>
        <n-button quaternary size="small" @click="clearChat">
          <template #icon><TrashIcon /></template>
        </n-button>
      </div>
    </div>

    <!-- 消息区域 -->
    <div class="chat-messages" ref="messagesContainer">
      <!-- 欢迎消息 -->
      <WelcomeMessage v-if="messages.length === 0" />
      
      <!-- 消息列表 -->
      <template v-for="message in messages" :key="message.id">
        <UserMessage 
          v-if="message.role === 'user'" 
          :message="message" 
        />
        <AssistantMessage 
          v-else 
          :message="message"
          @action="handleAction"
        />
      </template>

      <!-- 加载状态 -->
      <LoadingMessage 
        v-if="isLoading" 
        :action="currentAction" 
      />
    </div>

    <!-- 快捷提问 -->
    <div class="quick-questions">
      <span class="label">💡 快捷提问：</span>
      <div class="questions">
        <n-button
          v-for="q in quickQuestions"
          :key="q.label"
          size="small"
          secondary
          @click="handleQuickQuestion(q)"
        >
          {{ q.icon }} {{ q.label }}
        </n-button>
      </div>
    </div>

    <!-- 输入区域 -->
    <div class="chat-input">
      <n-input
        v-model:value="inputText"
        type="textarea"
        placeholder="输入你想问的..."
        :autosize="{ minRows: 1, maxRows: 4 }"
        @keypress.enter.exact="handleSend"
      />
      <n-button 
        type="primary" 
        :disabled="!inputText.trim() || isLoading"
        @click="handleSend"
      >
        发送
      </n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { useAiChatStore } from '@/stores/aiChat';
import { storeToRefs } from 'pinia';

// Components
import WelcomeMessage from './WelcomeMessage.vue';
import UserMessage from './UserMessage.vue';
import AssistantMessage from './AssistantMessage.vue';
import LoadingMessage from './LoadingMessage.vue';

const store = useAiChatStore();
const { messages, isLoading, currentAction } = storeToRefs(store);

const inputText = ref('');
const messagesContainer = ref<HTMLElement | null>(null);

// 快捷提问
const quickQuestions = [
  { label: '今日任务', icon: '📋', prompt: '我今天有哪些任务？' },
  { label: '生成日报', icon: '📝', prompt: '帮我生成今天的日报' },
  { label: '效率分析', icon: '📈', prompt: '分析一下我这周的工作效率' },
  { label: '搜索SQL', icon: '🔍', prompt: '搜索SQL：', needInput: true },
  { label: '开始专注', icon: '🍅', prompt: '开始一个25分钟的番茄钟' },
];

// 发送消息
async function handleSend() {
  const text = inputText.value.trim();
  if (!text || isLoading.value) return;

  inputText.value = '';
  await store.sendMessage(text);
  scrollToBottom();
}

// 快捷提问
function handleQuickQuestion(q: typeof quickQuestions[0]) {
  if (q.needInput) {
    inputText.value = q.prompt;
    // 聚焦输入框
  } else {
    inputText.value = q.prompt;
    handleSend();
  }
}

// 处理消息中的操作
function handleAction(action: { type: string; payload: any }) {
  switch (action.type) {
    case 'complete_task':
      store.sendMessage(`把任务「${action.payload.title}」标记为完成`);
      break;
    case 'copy_sql':
      navigator.clipboard.writeText(action.payload.content);
      break;
    case 'start_pomodoro':
      store.sendMessage(`开始一个番茄钟，处理「${action.payload.title}」`);
      break;
  }
}

// 滚动到底部
function scrollToBottom() {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
    }
  });
}

// 监听消息变化，自动滚动
watch(messages, () => scrollToBottom(), { deep: true });

// 清空对话
function clearChat() {
  store.clearMessages();
}
</script>

<style scoped lang="scss">
.ai-chat {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-color);
}

.chat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);

  .title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
  }

  .actions {
    display: flex;
    gap: 4px;
  }
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.quick-questions {
  padding: 8px 16px;
  border-top: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;

  .label {
    font-size: 12px;
    color: var(--text-color-secondary);
  }

  .questions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
}

.chat-input {
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
  display: flex;
  gap: 12px;
  align-items: flex-end;
}
</style>
```

### 8.2 后端核心代码

#### 8.2.1 AI 对话主命令

```rust
// src-tauri/src/commands/ai_chat.rs

use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub content: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[tauri::command]
pub async fn ai_chat(
    messages: Vec<ChatMessage>,
    state: State<'_, AppState>,
) -> Result<ChatResponse, String> {
    // 1. 构建系统提示词
    let system_prompt = build_system_prompt(&state).await?;
    
    // 2. 构建完整消息列表
    let mut full_messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        }
    ];
    full_messages.extend(messages);
    
    // 3. 调用 AI API
    let ai_response = call_ai_api(&full_messages, &get_functions()).await?;
    
    // 4. 处理 Function Call
    if let Some(function_call) = ai_response.function_call {
        let function_result = execute_function(&function_call, &state).await?;
        
        // 5. 将结果返回给 AI 生成最终回复
        full_messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: "".to_string(),
            // function_call info...
        });
        full_messages.push(ChatMessage {
            role: "function".to_string(),
            content: serde_json::to_string(&function_result)?,
        });
        
        let final_response = call_ai_api(&full_messages, &[]).await?;
        
        // 6. 构建响应
        Ok(ChatResponse {
            content: final_response.content,
            response_type: function_call.name.clone(),
            data: Some(function_result),
        })
    } else {
        Ok(ChatResponse {
            content: ai_response.content,
            response_type: "text".to_string(),
            data: None,
        })
    }
}

// 执行 Function
async fn execute_function(
    function_call: &FunctionCall,
    state: &State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let args: serde_json::Value = serde_json::from_str(&function_call.arguments)
        .map_err(|e| format!("Parse arguments error: {}", e))?;
    
    match function_call.name.as_str() {
        "get_tasks" => {
            let result = ai_get_tasks(
                args.get("status").and_then(|v| v.as_str()).map(String::from),
                args.get("quadrant").and_then(|v| v.as_i64()).map(|v| v as i32),
                args.get("date_range").and_then(|v| v.as_str()).map(String::from),
                args.get("keyword").and_then(|v| v.as_str()).map(String::from),
                state.clone(),
            ).await?;
            Ok(serde_json::to_value(result).unwrap())
        },
        "create_task" => {
            let result = ai_create_task(
                args.get("title").and_then(|v| v.as_str()).unwrap().to_string(),
                args.get("quadrant").and_then(|v| v.as_i64()).map(|v| v as i32),
                args.get("due_date").and_then(|v| v.as_str()).map(String::from),
                args.get("description").and_then(|v| v.as_str()).map(String::from),
                state.clone(),
            ).await?;
            Ok(serde_json::to_value(result).unwrap())
        },
        "search_sql" => {
            let result = ai_search_sql(
                args.get("keyword").and_then(|v| v.as_str()).map(String::from),
                args.get("sql_type").and_then(|v| v.as_str()).map(String::from),
                args.get("favorite_only").and_then(|v| v.as_bool()).unwrap_or(false),
                state.clone(),
            ).await?;
            Ok(serde_json::to_value(result).unwrap())
        },
        "get_time_stats" => {
            let result = ai_get_time_stats(
                args.get("date_range").and_then(|v| v.as_str()).map(String::from),
                args.get("group_by").and_then(|v| v.as_str()).map(String::from),
                state.clone(),
            ).await?;
            Ok(serde_json::to_value(result).unwrap())
        },
        "generate_daily_report" => {
            let result = ai_generate_daily_report(
                args.get("date").and_then(|v| v.as_str()).map(String::from),
                args.get("style").and_then(|v| v.as_str()).map(String::from),
                state.clone(),
            ).await?;
            Ok(serde_json::to_value(result).unwrap())
        },
        "generate_weekly_report" => {
            let result = ai_generate_weekly_report(
                args.get("week").and_then(|v| v.as_str()).map(String::from),
                args.get("style").and_then(|v| v.as_str()).map(String::from),
                state.clone(),
            ).await?;
            Ok(serde_json::to_value(result).unwrap())
        },
        "start_pomodoro" => {
            let result = ai_start_pomodoro(
                args.get("duration_minutes").and_then(|v| v.as_i64()).map(|v| v as i32),
                args.get("task_id").and_then(|v| v.as_str()).map(String::from),
                state.clone(),
            ).await?;
            Ok(serde_json::to_value(result).unwrap())
        },
        "analyze_efficiency" => {
            let result = ai_analyze_efficiency(
                args.get("date_range").and_then(|v| v.as_str()).map(String::from),
                args.get("focus").and_then(|v| v.as_str()).map(String::from),
                state.clone(),
            ).await?;
            Ok(serde_json::to_value(result).unwrap())
        },
        _ => Err(format!("Unknown function: {}", function_call.name)),
    }
}
```

#### 8.2.2 AI API 调用封装

```rust
// src-tauri/src/services/ai_service.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct AIRequest {
    model: String,
    messages: Vec<AIMessage>,
    functions: Option<Vec<FunctionDefinition>>,
    function_call: Option<String>,
    temperature: f32,
    max_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct AIMessage {
    role: String,
    content: Option<String>,
    function_call: Option<FunctionCallInfo>,
    name: Option<String>,
}

#[derive(Debug, Serialize)]
struct FunctionDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: AIMessage,
}

pub async fn call_ai_api(
    messages: &[ChatMessage],
    functions: &[FunctionDefinition],
) -> Result<AIResponseContent, String> {
    let client = Client::new();
    let config = get_ai_config()?;
    
    let ai_messages: Vec<AIMessage> = messages.iter().map(|m| AIMessage {
        role: m.role.clone(),
        content: Some(m.content.clone()),
        function_call: None,
        name: None,
    }).collect();
    
    let request = AIRequest {
        model: config.model,
        messages: ai_messages,
        functions: if functions.is_empty() { None } else { Some(functions.to_vec()) },
        function_call: if functions.is_empty() { None } else { Some("auto".to_string()) },
        temperature: 0.7,
        max_tokens: 2000,
    };
    
    let response = client
        .post(&format!("{}/v1/chat/completions", config.base_url))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;
    
    let ai_response: AIResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse response error: {}", e))?;
    
    let message = &ai_response.choices[0].message;
    
    Ok(AIResponseContent {
        content: message.content.clone().unwrap_or_default(),
        function_call: message.function_call.clone(),
    })
}
```

#### 8.2.3 Function 实现示例

```rust
// src-tauri/src/commands/ai_functions.rs

// 获取任务列表
#[tauri::command]
pub async fn ai_get_tasks(
    status: Option<String>,
    quadrant: Option<i32>,
    date_range: Option<String>,
    keyword: Option<String>,
    state: State<'_, AppState>,
) -> Result<TasksResponse, String> {
    let db = &state.db;
    
    let mut query = String::from("SELECT * FROM tasks WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
    
    // 构建查询条件
    if let Some(s) = status {
        if s != "all" {
            query.push_str(" AND status = ?");
            params.push(Box::new(s));
        }
    }
    
    if let Some(q) = quadrant {
        query.push_str(" AND quadrant = ?");
        params.push(Box::new(q));
    }
    
    if let Some(range) = date_range {
        let (start, end) = parse_date_range(&range)?;
        query.push_str(" AND created_at BETWEEN ? AND ?");
        params.push(Box::new(start));
        params.push(Box::new(end));
    }
    
    if let Some(kw) = keyword {
        query.push_str(" AND (title LIKE ? OR description LIKE ?)");
        let pattern = format!("%{}%", kw);
        params.push(Box::new(pattern.clone()));
        params.push(Box::new(pattern));
    }
    
    query.push_str(" ORDER BY quadrant ASC, due_date ASC LIMIT 50");
    
    // 执行查询
    let tasks = db.query_tasks(&query, &params)?;
    
    Ok(TasksResponse {
        total: tasks.len(),
        tasks,
    })
}

// 创建任务
#[tauri::command]
pub async fn ai_create_task(
    title: String,
    quadrant: Option<i32>,
    due_date: Option<String>,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<CreateTaskResponse, String> {
    let db = &state.db;
    
    let task = Task {
        id: generate_uuid(),
        title,
        description,
        quadrant: quadrant.unwrap_or(2),
        status: "todo".to_string(),
        due_date,
        created_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };
    
    db.insert_task(&task)?;
    
    Ok(CreateTaskResponse {
        success: true,
        task,
        message: "任务创建成功".to_string(),
    })
}

// 生成日报
#[tauri::command]
pub async fn ai_generate_daily_report(
    date: Option<String>,
    style: Option<String>,
    state: State<'_, AppState>,
) -> Result<DailyReportResponse, String> {
    let db = &state.db;
    let target_date = date.unwrap_or_else(|| "today".to_string());
    let report_style = style.unwrap_or_else(|| "simple".to_string());
    
    // 获取当天完成的任务
    let completed_tasks = db.get_completed_tasks_by_date(&target_date)?;
    
    // 获取时间统计
    let time_stats = db.get_time_stats_by_date(&target_date)?;
    
    // 获取番茄钟记录
    let pomodoro_records = db.get_pomodoro_by_date(&target_date)?;
    
    // 调用 AI 生成报告内容
    let report_content = generate_report_with_ai(
        &completed_tasks,
        &time_stats,
        &pomodoro_records,
        &report_style,
    ).await?;
    
    Ok(DailyReportResponse {
        date: target_date,
        report: report_content,
        markdown: format_report_as_markdown(&report_content),
    })
}
```

------

## 九、开发计划

### 9.1 阶段划分

| 阶段        | 内容                           | 工时   | 优先级 |
| ----------- | ------------------------------ | ------ | ------ |
| **Phase 1** | 基础对话 + 任务查询 + 时间统计 | 1 周   | P0     |
| **Phase 2** | 创建/更新任务 + SQL 搜索       | 1 周   | P0     |
| **Phase 3** | 生成日报/周报                  | 3-5 天 | P0     |
| **Phase 4** | 效率分析 + 番茄钟控制          | 3-5 天 | P1     |
| **Phase 5** | UI 优化 + 对话历史             | 3 天   | P1     |
| **Phase 6** | 快捷提问 + 流式响应            | 2 天   | P2     |

### 9.2 Phase 1 详细任务

**目标**：实现基础对话框架和核心查询功能

| 任务                | 描述                       | 工时 |
| ------------------- | -------------------------- | ---- |
| 创建 AI Chat Store  | Pinia 状态管理             | 2h   |
| 实现对话 UI 组件    | 基础布局、消息列表、输入框 | 4h   |
| 实现 ai_chat 命令   | Rust 后端主命令            | 4h   |
| 实现 AI API 调用    | 封装 OpenAI 兼容接口       | 3h   |
| 实现 get_tasks      | 任务查询 Function          | 3h   |
| 实现 get_time_stats | 时间统计 Function          | 3h   |
| 实现 System Prompt  | 构建系统提示词             | 2h   |
| 任务列表消息组件    | 展示任务查询结果           | 3h   |
| 时间统计消息组件    | 展示时间统计结果           | 3h   |
| 集成测试            | 测试完整对话流程           | 4h   |

### 9.3 Phase 2 详细任务

**目标**：实现操作类功能和 SQL 搜索

| 任务             | 描述                  | 工时 |
| ---------------- | --------------------- | ---- |
| 实现 create_task | 创建任务 Function     | 3h   |
| 实现 update_task | 更新任务 Function     | 3h   |
| 实现 search_sql  | SQL 搜索 Function     | 3h   |
| SQL 列表消息组件 | 展示 SQL 搜索结果     | 4h   |
| 操作确认机制     | 敏感操作二次确认      | 3h   |
| 消息内操作按钮   | 完成任务、复制 SQL 等 | 4h   |
| 错误处理优化     | 友好的错误提示        | 2h   |

### 9.4 Phase 3 详细任务

**目标**：实现日报周报生成

| 任务                        | 描述                   | 工时 |
| --------------------------- | ---------------------- | ---- |
| 实现 generate_daily_report  | 日报生成 Function      | 4h   |
| 实现 generate_weekly_report | 周报生成 Function      | 4h   |
| 报告消息组件                | 展示生成的报告         | 4h   |
| 报告导出功能                | 复制、导出为 Markdown  | 3h   |
| 报告样式模板                | 详细/简洁/正式三种风格 | 3h   |

### 9.5 测试用例

```typescript
// 测试用例示例
describe('AI Chat', () => {
  // 任务查询
  it('should query tasks correctly', async () => {
    const response = await sendMessage('我还有多少任务没做完？');
    expect(response.type).toBe('get_tasks');
    expect(response.data.tasks).toBeDefined();
  });

  // 创建任务
  it('should create task correctly', async () => {
    const response = await sendMessage('帮我创建一个任务，明天之前完成技术文档');
    expect(response.type).toBe('create_task');
    expect(response.data.task.title).toContain('技术文档');
    expect(response.data.task.due_date).toBe(getTomorrow());
  });

  // SQL 搜索
  it('should search SQL correctly', async () => {
    const response = await sendMessage('找一下有 user 表的 SQL');
    expect(response.type).toBe('search_sql');
    expect(response.data.sql_list.length).toBeGreaterThan(0);
  });

  // 生成日报
  it('should generate daily report correctly', async () => {
    const response = await sendMessage('生成今天的日报');
    expect(response.type).toBe('generate_daily_report');
    expect(response.data.markdown).toBeDefined();
  });
});
```

------

## 十、注意事项

### 10.1 安全考虑

| 风险         | 解决方案                     |
| ------------ | ---------------------------- |
| API Key 泄露 | 存储在系统安全区域，不硬编码 |
| 敏感操作     | 删除、批量操作需要二次确认   |
| 数据隐私     | 用户数据只在本地处理，不上传 |
| 输入注入     | 对用户输入进行校验和转义     |

### 10.2 性能优化

| 问题         | 解决方案                      |
| ------------ | ----------------------------- |
| AI 响应慢    | 实现流式响应，边生成边显示    |
| 历史消息过多 | 只发送最近 N 轮对话作为上下文 |
| 大量数据查询 | 分页加载，限制返回数量        |
| 重复请求     | 防抖处理，避免重复发送        |

### 10.3 用户体验

| 场景        | 处理方式                       |
| ----------- | ------------------------------ |
| 网络错误    | 显示友好错误提示，提供重试按钮 |
| AI 无法理解 | 引导用户重新描述或提供示例     |
| 长时间等待  | 显示加载状态和当前操作         |
| 结果过多    | 分组展示，提供筛选选项         |

### 10.4 扩展性考虑

```typescript
// 新增 Function 的标准流程
// 1. 在 AI_FUNCTIONS 中添加定义
// 2. 在 execute_function 中添加处理
// 3. 实现对应的 Tauri Command
// 4. 创建对应的消息展示组件
// 5. 更新 System Prompt 中的能力描述
// 6. 添加测试用例
```

### 10.5 多语言支持

当前版本仅支持中文，后续可考虑：

- System Prompt 多语言版本
- UI 文案国际化
- 根据用户语言自动切换

------

## 附录

### A. 相关文档

- [OpenAI Function Calling 文档](https://platform.openai.com/docs/guides/function-calling)
- [Tauri Commands 文档](https://tauri.app/v1/guides/features/command/)
- [Vue 3 Composition API](https://vuejs.org/guide/extras/composition-api-faq.html)

### B. 参考项目

- [Raycast AI](https://www.raycast.com/ai)
- [Notion AI](https://www.notion.so/product/ai)
- [GitHub Copilot Chat](https://github.com/features/copilot)

### C. 更新日志

| 日期       | 版本 | 内容     |
| ---------- | ---- | -------- |
| 2025-12-25 | 1.0  | 初始版本 |

------

*文档结束*