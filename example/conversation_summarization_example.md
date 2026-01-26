# 对话历史摘要功能使用示例

## 功能概述

当对话历史过长时，自动将旧消息压缩为摘要，保留关键信息同时减少 token 消耗。

## 使用方法

### Rust 后端使用

```rust
use crate::services::ai_chat_service::AiChatService;
use crate::services::ai_service::ChatMessage;

// 假设你有一个很长的对话历史
let messages = vec![
    ChatMessage::user("你好".to_string()),
    ChatMessage::assistant("你好！有什么可以帮你的吗？".to_string()),
    ChatMessage::user("帮我创建一个任务：写周报".to_string()),
    ChatMessage::assistant("好的，已创建任务「写周报」".to_string()),
    ChatMessage::user("开始番茄钟".to_string()),
    ChatMessage::assistant("番茄钟已开始！专注 25 分钟".to_string()),
    // ... 更多消息
];

// 当消息数超过 20 条时，自动压缩
let max_messages = 20;
let summarized = AiChatService::summarize_conversation_history(&messages, max_messages);

// 返回结果：
// - 如果消息数 <= 20，返回原消息
// - 如果消息数 > 20，返回压缩后的消息（1条摘要 + 最近10条消息）
```

### 前端集成示例

在发送消息给 AI 前，先进行历史压缩：

```typescript
// src/stores/aiChatStore.ts

async function sendMessage(content: string) {
  // 添加用户消息
  const userMessage: ChatMessage = {
    id: generateId(),
    role: 'user',
    content,
    timestamp: new Date()
  };

  messages.push(userMessage);

  // 在发送给后端前，检查消息历史长度
  let messagesToSend = [...messages];

  // 如果消息过多，可以在前端先调用后端的摘要接口
  if (messagesToSend.length > 20) {
    // 可选：调用后端摘要 API
    const summarized = await invoke('summarize_chat_history', {
      messages: messagesToSend,
      maxMessages: 20
    });
    messagesToSend = summarized;
  }

  // 发送给 AI API
  const response = await invoke('ai_chat', {
    messages: messagesToSend
  });

  // 处理响应...
}
```

## 摘要策略

### 1. 消息筛选

只提取重要信息，忽略以下内容：
- 纯问候语（"你好"、"谢谢"等）
- 简单确认（"好的"、"嗯"等）
- 过短的消息（少于5个字符）

### 2. 关键信息提取

自动提取以下类型的操作：

#### 任务操作
```
用户: "帮我创建一个任务：完成项目文档"
摘要: "- 创建了任务: 帮我创建一个任务：完成项目文档"
```

#### 番茄钟操作
```
用户: "开始番茄钟，专注工作"
摘要: "- 开始番茄钟，专注工作"
```

#### 查询操作
```
用户: "查询一下今天的任务列表"
摘要: "- 查询了: 查询一下今天的任务列表"
```

#### 用户偏好
```
用户: "我希望每天早上9点开始工作"
摘要: "- 用户偏好: 我希望每天早上9点开始工作"
```

### 3. 摘要格式

最终生成的摘要消息格式：

```
[对话摘要] 之前讨论了：
- 创建了任务: 帮我创建一个任务：写周报
- 开始番茄钟
- 查询了: 查询今天的任务
- 用户偏好: 我希望每天早上9点开始工作
```

## 实际效果

### 压缩前（40 条消息，约 2000 tokens）

```
[系统消息]
用户: 你好
助手: 你好！
用户: 今天天气怎么样？
助手: 我不知道天气...
用户: 好的
助手: 还有什么可以帮你的吗？
用户: 帮我创建一个任务：写周报
助手: 好的，已创建
... 32 条更多消息 ...
```

### 压缩后（11 条消息，约 500 tokens）

```
[系统消息: 对话摘要] 之前讨论了：
- 创建了任务: 帮我创建一个任务：写周报
- 开始番茄钟
- 查询了: 查询今天的任务

[最近 10 条消息保持原样]
用户: 完成番茄钟
助手: 番茄钟已完成！
...
```

**Token 节省率：约 75%**

## 配置建议

### 推荐阈值

- **max_messages = 20**: 适合一般对话
- **max_messages = 30**: 适合需要更多上下文的对话
- **max_messages = 10**: 适合快速问答场景

### 何时触发压缩

建议在以下情况触发：

1. **每次发送前检查**: 如果 `messages.length > max_messages`，先压缩
2. **定期压缩**: 每 5 条消息检查一次
3. **主动压缩**: 用户点击"清理历史"按钮时

## 注意事项

1. **纯本地处理**: 不调用 AI API，避免额外 token 消耗
2. **简单规则**: 使用关键词匹配，不需要复杂的 NLP
3. **摘要长度**: 自动控制在 200 字符以内
4. **去重处理**: 相同的关键信息只保留一次
5. **最多保留 10 个关键点**: 避免摘要过长

## 测试用例

参考 `src-tauri/src/services/ai_chat_service.rs` 中的测试：

- `test_summarize_conversation_history_within_limit`: 测试消息数在限制内
- `test_summarize_conversation_history_exceeds_limit`: 测试消息数超过限制
- `test_extract_task_operation`: 测试任务操作提取
- `test_extract_pomodoro_operation`: 测试番茄钟操作提取
- `test_is_greeting`: 测试问候语识别
- `test_extract_query_operation`: 测试查询操作提取
- `test_extract_user_preference`: 测试用户偏好提取

运行测试：

```bash
cd src-tauri
cargo test summarize_conversation_history
```

## 扩展建议

未来可以考虑的增强：

1. **语义相似度合并**: 使用向量数据库合并相似的操作
2. **时间权重**: 较新的消息权重更高
3. **重要性评分**: 基于消息内容计算重要性
4. **自定义规则**: 允许用户自定义重要关键词
5. **智能阈值**: 根据对话内容动态调整压缩阈值
