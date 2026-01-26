<template>
  <div class="ai-chat-container">
    <!-- 左侧历史侧边栏 -->
    <div v-if="!sidebarCollapsed" class="chat-sidebar">
      <ChatSidebar
        :current-session-id="currentSessionId"
        @new-chat="handleNewChat"
        @select-session="handleSelectSession"
        @close="toggleSidebar"
      />
    </div>

    <!-- 右侧对话区域 -->
    <div class="chat-main">
      <!-- 头部 -->
      <div class="chat-header">
        <div class="header-left">
          <n-button
            text
            class="sidebar-toggle-btn"
            @click="toggleSidebar"
          >
            <template #icon>
              <n-icon>
                <MenuOutline v-if="sidebarCollapsed" />
                <CloseOutline v-else />
              </n-icon>
            </template>
          </n-button>

          <div class="header-divider"></div>

          <div class="header-logo">
            <div class="logo-icon">
              <n-icon size="20">
                <SparklesOutline />
              </n-icon>
            </div>
            <div class="logo-text">
              <h1 class="logo-title">AI 助手</h1>
              <p class="logo-status">
                <span class="status-dot"></span>
                {{ aiStore.isEnabled ? '在线' : '离线' }}
              </p>
            </div>
          </div>
        </div>

        <div class="header-right">
          <n-button
            text
            class="clear-btn"
            @click="handleClear"
          >
            <template #icon>
              <n-icon><TrashOutline /></n-icon>
            </template>
            <span class="btn-text">清空</span>
          </n-button>
        </div>
      </div>

      <!-- 消息区域 -->
      <div ref="messagesContainer" class="messages-container">
        <!-- 日期统计卡片 -->
        <div class="date-pill">
          <span>{{ currentDateText }}</span>
          <span class="divider">|</span>
          <span>待办: {{ todoStats.todoCount }}</span>
          <span class="divider">|</span>
          <span>专注: {{ todoStats.focusTime }}m</span>
        </div>

        <!-- 欢迎消息 -->
        <WelcomeMessage
          v-if="messages.length === 0"
          @suggestion-click="handleSuggestionClick"
        />

        <!-- 消息列表 -->
        <div v-for="message in messages" :key="message.id" class="message-wrapper">
          <MessageBubble :message="message" />
        </div>

        <!-- 加载中 -->
        <LoadingMessage v-if="isTyping" :action="thinkingAction" />

        <div ref="messagesEndRef"></div>
      </div>

      <!-- 快捷提问区域 -->
      <div v-if="messages.length === 0" class="quick-questions">
        <n-button
          v-for="question in quickQuestions"
          :key="question.label"
          class="quick-question-btn"
          @click="handleQuickQuestion(question)"
        >
          <span class="question-icon">{{ question.icon }}</span>
          <span class="question-text">{{ question.label }}</span>
        </n-button>
      </div>

      <!-- 输入区域 -->
      <div class="input-area">
        <div class="input-wrapper">
          <n-input
            v-model:value="inputText"
            type="textarea"
            :placeholder="aiStore.isEnabled ? '输入你想问的...' : 'AI 未启用，请先在设置中配置'"
            :autosize="{ minRows: 1, maxRows: 5 }"
            :disabled="!aiStore.isEnabled || isTyping"
            @keydown.enter.exact.prevent="handleSend"
          />
          <n-button
            type="primary"
            class="send-btn"
            :disabled="!inputText.trim() || !aiStore.isEnabled || isTyping"
            @click="handleSend"
          >
            <template #icon>
              <n-icon><SendOutline /></n-icon>
            </template>
          </n-button>
        </div>
        <div class="input-hint">
          AI 助手可能会出错，请检查重要信息。
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted, watch } from 'vue';
import { NButton, NIcon, NInput } from 'naive-ui';
import {
  SparklesOutline,
  SendOutline,
  TrashOutline,
  MenuOutline,
  CloseOutline
} from '@vicons/ionicons5';
import { useAiChatStore } from '@/stores/aiChatStore';
import ChatSidebar from './ChatSidebar.vue';
import MessageBubble from './MessageBubble.vue';
import LoadingMessage from './LoadingMessage.vue';
import WelcomeMessage from './WelcomeMessage.vue';

// ============ Store ============
const aiChatStore = useAiChatStore();

// AI 启用状态
const aiStore = computed(() => ({
  isEnabled: true, // AI Chat 功能默认启用
}));

// ============ State ============
const sidebarCollapsed = ref(false);
const inputText = ref('');
const messagesContainer = ref<HTMLElement | null>(null);
const messagesEndRef = ref<HTMLElement | null>(null);

// 本地响应式消息列表 - 用于确保 UI 正确更新
const localMessages = ref<any[]>([]);

// 当前会话 ID - 直接从 store 获取
const currentSessionId = computed(() => aiChatStore.currentSessionId);

// 消息列表 - 使用本地副本确保响应式
const messages = computed(() => localMessages.value);

// 加载状态
const isLoading = computed(() => aiChatStore.isLoading);
const currentAction = computed(() => aiChatStore.currentAction);

// AI 正在回复时才显示加载指示器（不在加载会话时显示）
const isTyping = computed(() => {
  const action = currentAction.value;
  const loading = isLoading.value;

  // 如果正在加载会话，不显示思考指示器
  if (action === '正在加载会话...' || action === '正在创建新会话...') {
    return false;
  }

  return loading;
});

const thinkingAction = computed(() => currentAction.value || '');

// 同步 store 消息到本地（添加 watch 来同步）
watch(
  () => aiChatStore.messages,
  (newMessages) => {
    console.log('[AiChat] Store 消息变化，同步到本地:', newMessages?.length ?? 0);
    localMessages.value = newMessages ? [...newMessages] : [];
  },
  { immediate: true, deep: true }
);

// 统计数据（从 store 获取）
const currentDateText = computed(() => {
  const now = new Date();
  return now.toLocaleDateString('zh-CN', {
    month: 'long',
    day: 'numeric',
    weekday: 'long'
  });
});

// 待办统计
const todoStats = computed(() => ({
  todoCount: aiChatStore.dashboardStats?.totalMessages || 0,
  focusTime: 0, // 可以后续从其他 store 获取
}));

// 快捷提问配置
const quickQuestions = [
  { label: '今日任务', icon: '📋', prompt: '我今天有哪些任务？' },
  { label: '生成日报', icon: '📝', prompt: '帮我生成今天的日报' },
  { label: '效率分析', icon: '📈', prompt: '分析一下我这周的工作效率' },
  { label: '搜索SQL', icon: '🔍', prompt: '搜索SQL：', needInput: true },
  { label: '开始专注', icon: '🍅', prompt: '开始一个25分钟的番茄钟' },
];

// ============ Methods ============
const toggleSidebar = () => {
  sidebarCollapsed.value = !sidebarCollapsed.value;
};

const handleNewChat = async () => {
  await aiChatStore.createNewSession();
  // 新会话清空消息
  localMessages.value = [];
};

const handleSelectSession = async (sessionId: string) => {
  console.log('[AiChat] ==========================================');
  console.log('[AiChat] handleSelectSession 被调用');
  console.log('[AiChat] 目标会话 ID:', sessionId);
  console.log('[AiChat] 当前会话 ID:', currentSessionId.value);
  console.log('[AiChat] 本地消息数量（调用前）:', localMessages.value.length);

  try {
    // 调用 store 加载会话
    await aiChatStore.selectSession(sessionId);

    // 手动同步消息到本地（确保响应式）
    const storeMessages = aiChatStore.messages;
    console.log('[AiChat] Store 消息数量:', storeMessages?.length ?? 0);

    if (storeMessages && storeMessages.length > 0) {
      // 强制更新本地消息
      localMessages.value = [...storeMessages];
      console.log('[AiChat] ✅ 消息已同步到本地:', localMessages.value.length);
      console.log('[AiChat] 第一条消息:', localMessages.value[0]);
    } else {
      localMessages.value = [];
      console.log('[AiChat] ⚠️ Store 没有消息');
    }

    console.log('[AiChat] 新会话 ID:', aiChatStore.currentSessionId);
    console.log('[AiChat] ==========================================');
  } catch (err) {
    console.error('[AiChat] ❌ selectSession 失败:', err);
  }
};

const handleClear = async () => {
  if (currentSessionId.value) {
    aiChatStore.clearMessages();
    localMessages.value = [];
  }
  inputText.value = '';
};

const handleQuickQuestion = (question: any) => {
  if (question.needInput) {
    inputText.value = question.prompt;
  } else {
    inputText.value = question.prompt;
    handleSend();
  }
};

const handleSuggestionClick = (text: string) => {
  inputText.value = text;
  handleSend();
};

const scrollToBottom = async () => {
  await nextTick();
  if (messagesEndRef.value) {
    messagesEndRef.value.scrollIntoView({ behavior: 'smooth' });
  }
};

const handleSend = async () => {
  console.log('[handleSend] ==========================================');
  console.log('[handleSend] 发送按钮被点击');
  console.log('[handleSend] 输入内容:', inputText.value.trim().substring(0, 50));
  console.log('[handleSend] 当前会话:', currentSessionId.value);

  if (!inputText.value.trim() || !aiStore.value.isEnabled || isTyping.value) {
    console.log('[handleSend] 跳过发送:', {
      hasInput: !!inputText.value.trim(),
      isEnabled: aiStore.value.isEnabled,
      isTyping: isTyping.value
    });
    return;
  }

  const userInput = inputText.value.trim();
  inputText.value = '';

  console.log('[handleSend] 调用 sendMessage...');

  // 调用 store 的 sendMessage 方法
  // 注意：sendMessage 内部会自动处理会话创建
  await aiChatStore.sendMessage(userInput);

  console.log('[handleSend] sendMessage 完成');
  console.log('[handleSend] Store 消息数量:', aiChatStore.messages.length);

  // 同步消息到本地
  localMessages.value = [...aiChatStore.messages];
  console.log('[handleSend] 本地消息已同步, 数量:', localMessages.value.length);

  await scrollToBottom();
  console.log('[handleSend] ==========================================');
};

// ============ Lifecycle ============
onMounted(async () => {
  // 初始化 store（加载会话列表）
  await aiChatStore.initialize();
});

// 监听本地消息变化，自动滚动到底部
watch(
  localMessages,
  (newMessages, oldMessages) => {
    console.log('[AiChat] 📨 本地消息变化:');
    console.log('[AiChat]   旧数量:', oldMessages?.length ?? 0);
    console.log('[AiChat]   新数量:', newMessages?.length ?? 0);
    // 消息变化时滚动到底部
    scrollToBottom();
  },
  { deep: true }
);

// 监听当前会话变化
watch(
  () => aiChatStore.currentSessionId,
  (newId, oldId) => {
    console.log('[AiChat] 🔄 会话切换:', oldId, '->', newId);
  }
);
</script>

<style scoped>
.ai-chat-container {
  display: flex;
  height: 100%;
  width: 100%;
  background: var(--bg-base);
  overflow: hidden;
}

/* ========== 侧边栏 ========== */
.chat-sidebar {
  width: 280px;
  min-width: 280px;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--sidebar-border);
  display: flex;
  flex-direction: column;
  transition: all 0.3s ease;
}

/* ========== 主对话区域 ========== */
.chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--bg-surface);
}

/* ========== 头部 ========== */
.chat-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-default);
  background: var(--bg-elevated);
  backdrop-filter: blur(10px);
  height: 64px;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.sidebar-toggle-btn {
  color: var(--text-secondary);
  transition: all 0.2s ease;
}

.sidebar-toggle-btn:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.header-divider {
  width: 1px;
  height: 24px;
  background: var(--border-default);
}

.header-logo {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: var(--accent-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: var(--shadow-glow);
}

.logo-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.logo-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.logo-status {
  font-size: 11px;
  color: var(--text-muted);
  margin: 0;
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--success);
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.clear-btn {
  color: var(--text-secondary);
  transition: all 0.2s ease;
}

.clear-btn:hover {
  color: var(--error);
  background: rgba(239, 68, 68, 0.1);
}

.btn-text {
  display: none;
}

@media (min-width: 640px) {
  .btn-text {
    display: inline;
  }
}

/* ========== 消息区域 ========== */
.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 24px 16px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.messages-container::-webkit-scrollbar {
  width: 6px;
}

.messages-container::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
}

.messages-container::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 3px;
}

.messages-container::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
}

.date-pill {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 24px;
  padding: 8px 20px;
  font-size: 11px;
  color: var(--text-muted);
  align-self: center;
  backdrop-filter: blur(10px);
}

.divider {
  color: var(--border-default);
}

.message-wrapper {
  display: flex;
  flex-direction: column;
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* ========== 快捷提问 ========== */
.quick-questions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 0 16px 16px;
  justify-content: center;
}

.quick-question-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 12px;
  color: var(--text-secondary);
  font-size: 13px;
  transition: all 0.2s ease;
  cursor: pointer;
}

.quick-question-btn:hover {
  background: var(--card-hover-bg);
  border-color: var(--card-hover-border);
  color: var(--text-primary);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.question-icon {
  font-size: 16px;
}

/* ========== 输入区域 ========== */
.input-area {
  padding: 16px;
  border-top: 1px solid var(--border-default);
  background: var(--bg-elevated);
  flex-shrink: 0;
}

.input-wrapper {
  display: flex;
  gap: 12px;
  align-items: flex-end;
}

.input-wrapper :deep(.n-input) {
  flex: 1;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: 12px;
  transition: all 0.2s ease;
}

.input-wrapper :deep(.n-input:focus-within) {
  border-color: var(--input-focus-border);
  box-shadow: 0 0 0 3px var(--accent-glow);
}

.input-wrapper :deep(.n-input__textarea-el) {
  color: var(--text-primary);
  font-size: 14px;
  line-height: 1.5;
}

.input-wrapper :deep(.n-input__placeholder) {
  color: var(--text-dim);
}

.send-btn {
  min-width: 48px;
  height: 48px;
  border-radius: 12px;
  background: var(--accent-primary);
  box-shadow: var(--shadow-glow);
}

.send-btn:hover {
  background: var(--accent-primary-hover);
}

.send-btn:disabled {
  background: var(--bg-hover);
  opacity: 0.5;
}

.input-hint {
  margin-top: 12px;
  text-align: center;
  font-size: 10px;
  color: var(--text-dim);
}

/* ========== 响应式 ========== */
@media (max-width: 768px) {
  .chat-sidebar {
    position: absolute;
    left: 0;
    top: 0;
    height: 100%;
    z-index: 10;
    box-shadow: var(--shadow-lg);
  }

  .date-pill {
    font-size: 10px;
    padding: 6px 16px;
  }

  .quick-questions {
    overflow-x: auto;
    flex-wrap: nowrap;
    justify-content: flex-start;
    padding-bottom: 12px;
  }

  .quick-question-btn {
    white-space: nowrap;
  }
}
</style>
