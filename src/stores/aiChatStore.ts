import { defineStore } from 'pinia';
import { ref, computed, onUnmounted } from 'vue';
import dayjs from 'dayjs';
import { aiChatApi } from '@/api/aiChatApi';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { ChatMessage as ApiChatMessage, ChatSession as ApiChatSession, OperationContext } from '@/api/aiChatApi';

// AI 思考状态类型
export interface ThinkingStatus {
  status: string;
  detail?: string;
  step: number;
  totalSteps?: number;
}

// 定义消息类型（用于 Store 内部使用）
export interface ChatMessage {
  id: string | number;
  sender: 'user' | 'ai' | 'system';
  content?: string;
  type?: 'text' | 'tasks' | 'task_detail' | 'sql' | 'query_result' | 'stats' | 'time_stats' | 'report' | 'pomodoro' | 'pomodoro_started' | string;
  data?: any;
  timestamp?: number | string;
}

// 定义会话类型
export interface ChatSession {
  id: string;
  title: string;
  createdAt: number | string;
  updatedAt: number | string;
  messageCount: number;
}

// 定义仪表盘统计类型
export interface DashboardStats {
  totalSessions: number;
  totalMessages: number;
  todaySessions: number;
  todayMessages: number;
  averageMessagesPerSession: number;
}

export const useAiChatStore = defineStore('aiChat', () => {
  // ========== 状态定义 ==========
  const messages = ref<ChatMessage[]>([]);
  const sessions = ref<ChatSession[]>([]);
  const currentSessionId = ref<string | null>(null);
  const isLoading = ref(false);
  const currentAction = ref<string | null>(null);
  const error = ref<string | null>(null);
  const dashboardStats = ref<DashboardStats | null>(null);
  const sidebarOpen = ref(true);
  const thinkingStatus = ref<ThinkingStatus | null>(null);
  // 操作上下文 - 用于代词解析（如"它"、"这个任务"、"刚才那个"）
  const operationContext = ref<OperationContext | null>(null);

  // 事件监听器取消函数
  let unlistenThinking: UnlistenFn | null = null;

  // ========== 计算属性 ==========

  // 返回非系统消息的对话历史
  const chatHistory = computed(() =>
    messages.value.filter(msg => msg.sender !== 'system')
  );

  // 是否有消息
  const hasMessages = computed(() => messages.value.length > 0);

  // 当前会话信息
  const currentSession = computed(() =>
    sessions.value.find(session => session.id === currentSessionId.value) || null
  );

  // ========== Actions ==========

  /**
   * 发送用户消息
   * @param content 消息内容
   */
  async function sendMessage(content: string): Promise<void> {
    console.log('[sendMessage] ==========================================');
    console.log('[sendMessage] 开始发送消息');
    console.log('[sendMessage] 内容:', content.substring(0, 50));
    console.log('[sendMessage] 当前会话 ID:', currentSessionId.value);
    console.log('[sendMessage] 所有会话数量:', sessions.value.length);

    if (!content.trim()) {
      error.value = '消息内容不能为空';
      return;
    }

    // 重要：如果没有当前会话，先创建一个
    if (!currentSessionId.value) {
      console.log('[sendMessage] ⚠️ 没有当前会话，自动创建新会话...');
      await createNewSession();
      console.log('[sendMessage] ✅ 新会话已创建, sessionId:', currentSessionId.value);
    }

    // 再次检查会话 ID
    if (!currentSessionId.value) {
      console.error('[sendMessage] ❌ 创建会话失败，无法发送消息');
      error.value = '创建会话失败，请重试';
      return;
    }

    error.value = null;
    isLoading.value = true;
    currentAction.value = '正在思考...';

    // 保存当前会话 ID 的本地副本（避免异步操作中值变化）
    const sessionId = currentSessionId.value;
    console.log('[sendMessage] 使用会话 ID:', sessionId);

    // 添加用户消息到本地
    const userMessage: ChatMessage = {
      id: Date.now(),
      sender: 'user',
      content: content.trim(),
      type: 'text',
      timestamp: Date.now(),
    };
    messages.value.push(userMessage);
    console.log('[sendMessage] 用户消息已添加到本地, 当前消息数:', messages.value.length);

    try {
      // 先保存用户消息到数据库
      try {
        await aiChatApi.saveMessage(sessionId, {
          role: 'user',
          content: content.trim(),
        });
      } catch (saveErr) {
        console.error('[sendMessage] 保存用户消息失败:', saveErr);
      }

      // 构建对话历史（转换为 API 所需格式）
      const chatHistory: ApiChatMessage[] = messages.value.map(msg => ({
        role: msg.sender === 'user' ? 'user' : msg.sender === 'ai' ? 'assistant' : 'system',
        content: msg.content || '',
      }));

      // 调用后端 API，传递 sessionId 和操作上下文用于代词解析
      const response = await aiChatApi.chat(chatHistory, sessionId, operationContext.value || undefined);

      // 调试：打印收到的响应
      console.log('[AI Chat] Response:', {
        responseType: response.responseType,
        hasData: !!response.data,
        dataKeys: response.data ? Object.keys(response.data) : [],
        hasContext: !!response.context,
      });

      // 更新操作上下文（用于后续对话的代词解析）
      if (response.context) {
        operationContext.value = response.context;
        console.log('[AI Chat] 操作上下文已更新:', response.context);
      }

      // 解析响应并添加 AI 消息
      const aiMessage: ChatMessage = {
        id: Date.now() + 1,
        sender: 'ai',
        content: response.content,
        type: (response.responseType as any) || 'text',
        data: response.data,
        timestamp: Date.now(),
      };
      messages.value.push(aiMessage);

      // 保存 AI 消息到数据库
      try {
        await aiChatApi.saveMessage(sessionId, {
          role: 'assistant',
          content: response.content,
        });
      } catch (saveErr) {
        console.error('[sendMessage] 保存AI消息失败:', saveErr);
      }

      // 更新当前会话
      {
        const session = sessions.value.find(s => s.id === sessionId);
        if (session) {
          session.updatedAt = Date.now();
          session.messageCount = (session.messageCount || 0) + 2; // 用户消息 + AI 响应

          // 如果是新会话（标题为"新对话"），根据第一条消息生成标题
          if (session.title === '新对话' && content.trim().length > 0) {
            const newTitle = content.trim().substring(0, 20) + (content.trim().length > 20 ? '...' : '');
            session.title = newTitle;
            try {
              await aiChatApi.updateSessionTitle(session.id, newTitle);
            } catch (titleErr) {
              console.error('[sendMessage] 更新会话标题失败:', titleErr);
            }
          }

          // 保存会话更新
          try {
            await aiChatApi.saveSession({
              id: session.id,
              title: session.title,
              createdAt: typeof session.createdAt === 'number' ? session.createdAt : new Date(session.createdAt).getTime(),
              updatedAt: Date.now(),
              messageCount: session.messageCount,
            });
          } catch (sessionErr) {
            console.error('[sendMessage] 更新会话失败:', sessionErr);
          }
        }
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : '发送消息失败';
      console.error('[sendMessage] ❌ 发送消息失败:', err);

      // 发生错误时，移除用户消息（因为已经保存到数据库，下次加载时可能会有）
      messages.value = messages.value.filter(msg => msg.id !== userMessage.id);
    } finally {
      isLoading.value = false;
      currentAction.value = null;
      console.log('[sendMessage] 完成, isLoading:', isLoading.value);
    }
  }

  /**
   * 清空当前对话
   */
  function clearMessages(): void {
    messages.value = [];
    error.value = null;
    operationContext.value = null;  // 清空操作上下文

    // 更新当前会话的消息计数
    if (currentSessionId.value) {
      const session = sessions.value.find(s => s.id === currentSessionId.value);
      if (session) {
        session.messageCount = 0;
        session.updatedAt = Date.now();
      }
    }
  }

  /**
   * 加载历史会话列表
   */
  async function loadSessions(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      // 调用后端 API 获取会话列表
      const apiSessions = await aiChatApi.getSessions();

      // 转换为 Store 内部格式
      sessions.value = apiSessions.map(session => ({
        id: session.id,
        title: session.title,
        createdAt: session.createdAt,
        updatedAt: session.updatedAt,
        messageCount: session.messageCount || 0,
      }));
    } catch (err) {
      error.value = err instanceof Error ? err.message : '加载会话列表失败';
      console.error('加载会话列表失败:', err);
      sessions.value = [];
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 切换会话
   * @param sessionId 会话 ID
   */
  async function selectSession(sessionId: string): Promise<void> {
    console.log('[selectSession] ========== 开始加载会话 ==========');
    console.log('[selectSession] 目标会话:', sessionId);
    console.log('[selectSession] 当前会话:', currentSessionId.value);

    if (sessionId === currentSessionId.value) {
      console.log('[selectSession] 已是当前会话，跳过加载');
      return;
    }

    // 先重置状态，确保干净的开始
    isLoading.value = true;
    error.value = null;
    currentAction.value = '正在加载会话...';
    thinkingStatus.value = null;  // 清除思考状态
    operationContext.value = null;  // 切换会话时清空操作上下文

    try {
      // 调用后端 API 加载会话消息
      console.log('[selectSession] 调用 API 获取消息...');
      let apiMessages;
      try {
        apiMessages = await aiChatApi.getSessionMessages(sessionId);
        console.log('[selectSession] API 返回:', apiMessages);
        console.log('[selectSession] API 返回消息数量:', apiMessages?.length ?? 'null/undefined');
      } catch (apiErr) {
        console.error('[selectSession] API 调用失败:', apiErr);
        throw apiErr;
      }

      if (!apiMessages || !Array.isArray(apiMessages)) {
        console.error('[selectSession] API 返回数据格式错误:', apiMessages);
        messages.value = [];
        currentSessionId.value = sessionId;
        return;
      }

      // 打印每条消息的详细信息
      apiMessages.forEach((msg, idx) => {
        console.log(`[selectSession]   ${idx + 1}. [${msg.role}] ${msg.content?.substring(0, 50)}...`);
      });

      // 转换为 Store 内部格式，确保类型正确
      const convertedMessages: ChatMessage[] = apiMessages.map((msg, index) => {
        // 明确类型转换
        let sender: 'user' | 'ai' | 'system';
        if (msg.role === 'user') {
          sender = 'user';
        } else if (msg.role === 'assistant') {
          sender = 'ai';
        } else {
          sender = 'system';
        }

        return {
          id: `${sessionId}_${index}_${Date.now()}`,
          sender,
          content: msg.content,
          type: 'text' as const,
          timestamp: Date.now() - (apiMessages.length - index) * 1000,
        };
      });

      console.log('[selectSession] 转换后消息数量:', convertedMessages.length);
      console.log('[selectSession] 转换后消息:', JSON.stringify(convertedMessages, null, 2));

      // 更新 store 的 messages - 直接替换整个数组
      messages.value = convertedMessages;
      console.log('[selectSession] messages.value 已更新, 长度:', messages.value.length);

      currentSessionId.value = sessionId;
      console.log('[selectSession] ✅ 加载完成');
      console.log('[selectSession] messages.value:', messages.value);
      console.log('[selectSession] messages.value.length:', messages.value.length);
    } catch (err) {
      error.value = err instanceof Error ? err.message : '加载会话失败';
      console.error('[selectSession] ❌ 加载会话失败:', err);
      messages.value = [];
    } finally {
      // 确保加载状态被正确重置
      isLoading.value = false;
      currentAction.value = null;
      thinkingStatus.value = null;
      console.log('[selectSession] 加载状态已重置:');
      console.log('[selectSession]   - isLoading:', isLoading.value);
      console.log('[selectSession]   - currentAction:', currentAction.value);
      console.log('[selectSession] ========== 加载结束 ==========');
    }
  }

  /**
   * 创建新会话
   */
  async function createNewSession(): Promise<void> {
    isLoading.value = true;
    error.value = null;
    currentAction.value = '正在创建新会话...';

    try {
      const now = Date.now();
      const newSession: ChatSession = {
        id: `session_${now}`,
        title: '新对话',
        createdAt: now,
        updatedAt: now,
        messageCount: 0,
      };

      // 调用后端 API 保存新会话
      await aiChatApi.saveSession({
        id: newSession.id,
        title: newSession.title,
        createdAt: now,
        updatedAt: now,
        messageCount: 0,
      });

      // 添加到本地会话列表
      sessions.value.unshift(newSession);
      currentSessionId.value = newSession.id;
      messages.value = [];
      operationContext.value = null;  // 新会话清空操作上下文
    } catch (err) {
      error.value = err instanceof Error ? err.message : '创建会话失败';
      console.error('创建会话失败:', err);
    } finally {
      isLoading.value = false;
      currentAction.value = null;
    }
  }

  /**
   * 删除会话
   * @param sessionId 会话 ID
   */
  async function deleteSession(sessionId: string): Promise<void> {
    isLoading.value = true;
    error.value = null;
    currentAction.value = '正在删除会话...';

    try {
      // 调用后端 API 删除会话
      await aiChatApi.deleteSession(sessionId);

      // 从本地列表中移除
      sessions.value = sessions.value.filter(s => s.id !== sessionId);

      // 如果删除的是当前会话，切换到第一个会话或清空
      if (currentSessionId.value === sessionId) {
        if (sessions.value.length > 0) {
          currentSessionId.value = sessions.value[0].id;
          await selectSession(sessions.value[0].id);
        } else {
          currentSessionId.value = null;
          messages.value = [];
        }
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : '删除会话失败';
      console.error('删除会话失败:', err);
    } finally {
      isLoading.value = false;
      currentAction.value = null;
    }
  }

  /**
   * 加载仪表盘统计
   */
  async function loadDashboardStats(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      // 调用后端 API 获取统计数据
      const apiStats = await aiChatApi.getDashboardStats();

      // 转换为 Store 内部格式
      // 注意：后端返回的是任务相关统计，这里做简单映射
      // 如果需要真正的会话统计，需要后端提供专门的接口
      dashboardStats.value = {
        totalSessions: sessions.value.length,
        totalMessages: sessions.value.reduce((sum, s) => sum + s.messageCount, 0),
        todaySessions: sessions.value.filter(s =>
          dayjs(s.createdAt).isSame(dayjs(), 'day')
        ).length,
        todayMessages: messages.value.filter(m =>
          dayjs(m.timestamp).isSame(dayjs(), 'day')
        ).length,
        averageMessagesPerSession: sessions.value.length > 0
          ? Math.round(
              sessions.value.reduce((sum, s) => sum + s.messageCount, 0) / sessions.value.length
            )
          : 0,
      };
    } catch (err) {
      error.value = err instanceof Error ? err.message : '加载统计数据失败';
      console.error('加载统计数据失败:', err);
      dashboardStats.value = null;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 切换侧边栏
   */
  function toggleSidebar(): void {
    sidebarOpen.value = !sidebarOpen.value;
  }

  /**
   * 设置当前操作状态
   * @param action 操作描述
   */
  function setCurrentAction(action: string | null): void {
    currentAction.value = action;
  }

  /**
   * 设置 AI 思考状态事件监听
   */
  async function setupThinkingListener(): Promise<void> {
    // 如果已经有监听器，先清理
    if (unlistenThinking) {
      unlistenThinking();
      unlistenThinking = null;
    }

    try {
      unlistenThinking = await listen<ThinkingStatus>('ai-thinking-status', (event) => {
        const status = event.payload;
        thinkingStatus.value = status;

        // 同时更新 currentAction 以保持兼容
        if (status.status === '完成') {
          currentAction.value = null;
          thinkingStatus.value = null;
        } else {
          currentAction.value = status.detail ? `${status.status} ${status.detail}` : status.status;
        }

        console.log('[AI Chat] 思考状态:', status, 'isLoading:', isLoading.value);
      });
      console.log('[AI Chat] 思考状态监听器已设置');
    } catch (err) {
      console.error('设置思考状态监听失败:', err);
    }
  }

  /**
   * 清理事件监听器
   */
  function cleanupListeners(): void {
    if (unlistenThinking) {
      unlistenThinking();
      unlistenThinking = null;
    }
    thinkingStatus.value = null;
  }

  /**
   * 初始化 Store（可在应用启动时调用）
   */
  async function initialize(): Promise<void> {
    // 设置思考状态事件监听
    await setupThinkingListener();

    await loadSessions();

    // 如果有会话，默认选择第一个
    if (sessions.value.length > 0 && !currentSessionId.value) {
      await selectSession(sessions.value[0].id);
    }
  }

  return {
    // 状态
    messages,
    sessions,
    currentSessionId,
    isLoading,
    currentAction,
    error,
    dashboardStats,
    sidebarOpen,
    thinkingStatus,
    operationContext,

    // 计算属性
    chatHistory,
    hasMessages,
    currentSession,

    // 方法
    sendMessage,
    clearMessages,
    loadSessions,
    selectSession,
    createNewSession,
    deleteSession,
    loadDashboardStats,
    toggleSidebar,
    setCurrentAction,
    initialize,
    setupThinkingListener,
    cleanupListeners,
  };
});
