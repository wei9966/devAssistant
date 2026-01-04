# ReminderService 使用示例

## 概述

`ReminderService` 是一个智能提醒服务，用于检测和提醒用户关于：
- 即将到期的任务
- 长期未处理的高优先级任务
- 记忆中的重要日期

## 基本用法

### 1. 获取所有待办提醒

```rust
use crate::services::ReminderService;

// 获取所有需要提醒的事项
let reminders = ReminderService::get_pending_reminders(&conn)?;

// 遍历提醒
for reminder in &reminders {
    println!("类型: {:?}", reminder.reminder_type);
    println!("标题: {}", reminder.title);
    println!("消息: {}", reminder.message);
    println!("紧急程度: {:?}", reminder.urgency);
    if let Some(task_id) = reminder.related_task_id {
        println!("相关任务ID: {}", task_id);
    }
}
```

### 2. 检查特定类型的提醒

```rust
// 仅检查截止日期提醒
let deadline_reminders = ReminderService::check_deadline_reminders(&conn)?;

// 仅检查高优先级任务提醒
let stale_reminders = ReminderService::check_stale_high_priority_reminders(&conn)?;

// 仅检查记忆日期提醒
let memory_reminders = ReminderService::check_memory_reminders(&conn)?;
```

### 3. 格式化提醒为系统提示词

```rust
// 获取提醒
let reminders = ReminderService::get_pending_reminders(&conn)?;

// 格式化为文本，用于注入 AI 系统提示词
let prompt_text = ReminderService::format_reminders_for_prompt(&reminders);

// 输出示例：
// ## 当前待办提醒
//
// ### 🔴 紧急提醒
// - 任务「完成报告」将在明天到期
// - 任务「修复Bug」已逾期 2 天
//
// ### 🟡 重要提醒
// - 高优先级任务「设计评审」已有 5 天未更新
//
// 请在回复用户时，适时提及这些提醒事项，帮助用户更好地管理任务。
```

## 数据结构

### ReminderType (提醒类型)

```rust
pub enum ReminderType {
    DeadlineApproaching,  // 截止日期临近
    HighPriorityStale,    // 高优先级任务长期未处理
    ImportantDate,        // 记忆中的重要日期
}
```

### Urgency (紧急程度)

```rust
pub enum Urgency {
    High,     // 高紧急（1天内到期、已逾期、7天以上未处理）
    Medium,   // 中紧急（3天内到期、3-7天未处理）
    Low,      // 低紧急（7天内的重要日期）
}
```

### Reminder (提醒)

```rust
pub struct Reminder {
    pub reminder_type: ReminderType,  // 提醒类型
    pub title: String,                // 标题
    pub message: String,              // 详细消息
    pub urgency: Urgency,             // 紧急程度
    pub related_task_id: Option<i64>, // 相关任务ID（如果有）
}
```

## 提醒规则

### 1. 截止日期提醒

- **1天内到期**: 高紧急提醒
- **3天内到期**: 中紧急提醒
- **已逾期**: 高紧急提醒，显示逾期天数

### 2. 高优先级任务提醒

- **3天未更新**: 中紧急提醒
- **7天以上未更新**: 高紧急提醒

### 3. 重要日期提醒

从 `ai_context_memory` 表读取 `context_type = 'fact'` 的记忆，提取日期信息。

示例记忆格式：

```json
{
  "date": "2024-12-30",
  "description": "项目评审会议"
}
```

或：

```json
{
  "event_date": "2024-12-31",
  "event": "年终总结提交"
}
```

提醒时间：
- **当天**: 高紧急
- **1-2天内**: 中紧急
- **3-7天内**: 低紧急

## 在 AI 聊天中集成

在 AI 对话系统中，可以将提醒注入到系统提示词中：

```rust
use crate::services::{AiChatService, ReminderService};

// 获取提醒
let reminders = ReminderService::get_pending_reminders(&conn)?;

// 格式化为提示词
let reminder_prompt = ReminderService::format_reminders_for_prompt(&reminders);

// 在构建系统提示词时添加提醒
let system_prompt = format!(
    "你是一个智能助手，帮助用户管理任务和时间。\n{}\n请根据用户的问题提供帮助。",
    reminder_prompt
);

// 发送给 AI
let response = AiChatService::chat(&conn, session_id, user_message, &system_prompt)?;
```

## 数据库依赖

### tasks 表字段

- `id`: 任务ID
- `title`: 任务标题
- `status`: 任务状态（过滤掉 'completed'）
- `priority`: 优先级（1=高优先级）
- `due_date`: 截止日期
- `created_at`: 创建时间
- `started_at`: 开始时间
- `last_active_at`: 最后活跃时间

### ai_context_memory 表字段

- `id`: 记忆ID
- `context_type`: 上下文类型（'fact' 用于日期提醒）
- `key`: 记忆关键字
- `value`: JSON 值（包含日期和描述）
- `importance`: 重要性（1-10）

## 日期格式支持

服务支持多种日期时间格式：

- `2024-12-29 14:30:00`
- `2024-12-29T14:30:00`
- `2024-12-29 14:30:00.123`
- `2024-12-29T14:30:00.123`
- `2024-12-29` (纯日期，默认 00:00:00)
- `2024/12/29`
- `2024年12月29日`

## 注意事项

1. 已完成的任务（`status = 'completed'`）不会产生提醒
2. 只有高优先级任务（`priority = 1`）才会被检查长期未处理
3. 记忆日期提醒只查找 `context_type = 'fact'` 的记录
4. 时间计算基于本地时区
5. 提醒消息是中文的，适合中文用户界面

## 测试

项目包含完整的单元测试，覆盖所有主要功能：

```bash
cargo test reminder_service --lib
```

测试内容：
- 1天内到期任务提醒
- 3天内到期任务提醒
- 已逾期任务提醒
- 高优先级任务长期未处理提醒
- 记忆日期提醒
- 提醒格式化输出
- 已完成任务不产生提醒
