# AI Chat 组件

DevAssistant AI Chat 消息展示相关 Vue 组件库。

## 组件列表

### 1. MessageBubble.vue - 消息气泡组件

**功能**：显示用户和 AI 的消息气泡，支持多种消息类型。

**Props**：
- `message` (ChatMessage) - 消息对象

**消息类型**：
- `text` - 纯文本消息
- `tasks` - 任务列表
- `sql` - SQL 查询
- `stats` - 统计数据
- `report` - 报告

**插槽**：
- `tasks` - 自定义任务列表渲染
- `sql` - 自定义 SQL 显示
- `stats` - 自定义统计数据显示
- `report` - 自定义报告显示
- `custom` - 自定义内容

**使用示例**：
```vue
<template>
  <MessageBubble :message="message">
    <!-- 可选：自定义任务列表显示 -->
    <template #tasks="{ data }">
      <CustomTaskList :tasks="data.tasks" />
    </template>
  </MessageBubble>
</template>

<script setup>
import { MessageBubble } from '@/components/aiChat';

const message = {
  id: 1,
  sender: 'ai',
  content: '这是一条 AI 消息',
  type: 'text',
  timestamp: Date.now()
};
</script>
```

---

### 2. LoadingMessage.vue - 加载状态组件

**功能**：显示 AI 正在思考的加载动画。

**Props**：
- `action` (string, 可选) - 当前操作描述，如 "正在查询任务..."

**使用示例**：
```vue
<template>
  <LoadingMessage action="正在查询任务..." />
</template>

<script setup>
import { LoadingMessage } from '@/components/aiChat';
</script>
```

---

### 3. WelcomeMessage.vue - 欢迎消息组件

**功能**：显示欢迎语、功能介绍和示例提问。

**Props**：
- `stats` (UserStats, 可选) - 用户统计摘要
  ```ts
  interface UserStats {
    todoCount: number;     // 待办任务数
    weeklyDone: number;    // 本周完成数
    todayFocus: number;    // 今日专注时间（分钟）
  }
  ```
- `suggestions` (Suggestion[], 可选) - 自定义建议提问列表
  ```ts
  interface Suggestion {
    text: string;
    icon: Component;
  }
  ```

**事件**：
- `suggestion-click` - 点击建议提问时触发，参数为提问文本

**使用示例**：
```vue
<template>
  <WelcomeMessage
    :stats="userStats"
    :suggestions="customSuggestions"
    @suggestion-click="handleSuggestionClick"
  />
</template>

<script setup>
import { WelcomeMessage } from '@/components/aiChat';
import { CheckmarkCircleOutline } from '@vicons/ionicons5';

const userStats = {
  todoCount: 12,
  weeklyDone: 18,
  todayFocus: 145
};

const customSuggestions = [
  { text: '我的任务完成了多少？', icon: CheckmarkCircleOutline }
];

const handleSuggestionClick = (text: string) => {
  console.log('用户点击建议:', text);
};
</script>
```

---

## 样式系统

所有组件都使用主题变量系统（`src/themes/variables.css`），**禁止硬编码颜色值**。

### 常用主题变量：

**背景色**：
- `--bg-base` - 基础背景
- `--bg-surface` - 表面背景
- `--bg-elevated` - 提升背景
- `--bg-hover` - 悬停背景

**文字颜色**：
- `--text-primary` - 主要文字
- `--text-secondary` - 次要文字
- `--text-muted` - 弱化文字
- `--text-dim` - 最淡文字

**品牌色**：
- `--accent-primary` - 主强调色
- `--accent-secondary` - 次强调色
- `--accent-glow` - 发光效果

**状态色**：
- `--success` - 成功
- `--warning` - 警告
- `--error` - 错误
- `--info` - 信息

**边框**：
- `--border-default` - 默认边框
- `--border-hover` - 悬停边框
- `--border-active` - 激活边框

**阴影**：
- `--shadow-sm` - 小阴影
- `--shadow-md` - 中阴影
- `--shadow-lg` - 大阴影
- `--shadow-glow` - 发光阴影

---

## 完整使用示例

```vue
<template>
  <div class="chat-container">
    <!-- 欢迎消息 -->
    <WelcomeMessage
      v-if="messages.length === 0"
      :stats="userStats"
      @suggestion-click="handleSend"
    />

    <!-- 消息列表 -->
    <MessageBubble
      v-for="msg in messages"
      :key="msg.id"
      :message="msg"
    />

    <!-- 加载状态 -->
    <LoadingMessage v-if="isLoading" action="正在思考..." />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { MessageBubble, LoadingMessage, WelcomeMessage } from '@/components/aiChat';
import type { ChatMessage } from '@/components/aiChat';

const messages = ref<ChatMessage[]>([]);
const isLoading = ref(false);
const userStats = {
  todoCount: 12,
  weeklyDone: 18,
  todayFocus: 145
};

const handleSend = async (text: string) => {
  // 添加用户消息
  messages.value.push({
    id: Date.now(),
    sender: 'user',
    content: text,
    type: 'text',
    timestamp: Date.now()
  });

  // 显示加载
  isLoading.value = true;

  // 模拟 AI 回复
  setTimeout(() => {
    messages.value.push({
      id: Date.now() + 1,
      sender: 'ai',
      content: '这是 AI 的回复',
      type: 'text',
      timestamp: Date.now()
    });
    isLoading.value = false;
  }, 1000);
};
</script>

<style scoped>
.chat-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 20px;
}
</style>
```

---

## 注意事项

1. **主题变量**：必须使用 CSS 变量，不要硬编码颜色
2. **响应式**：所有组件都支持移动端适配
3. **动画**：使用 CSS 动画提升用户体验
4. **类型安全**：使用 TypeScript 类型定义
5. **可扩展性**：提供插槽支持自定义渲染
