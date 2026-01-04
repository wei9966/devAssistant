import { invoke } from '@tauri-apps/api/core';

// 聊天消息类型
export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

// 任务上下文（用于代词解析）
export interface TaskContext {
  id: number;
  title: string;
}

// 番茄钟上下文
export interface PomodoroContext {
  id: number;
  taskId?: number;
  taskTitle?: string;
}

// 操作上下文 - 记录最近操作的实体，用于代词解析（如"它"、"这个任务"）
export interface OperationContext {
  lastTask?: TaskContext;
  lastPomodoro?: PomodoroContext;
  recentTasks?: TaskContext[];
}

// 聊天响应类型
export interface ChatResponse {
  content: string;
  responseType: string;
  data?: any;
  context?: OperationContext;  // 操作上下文，用于后续对话的代词解析
}

// 仪表盘统计数据
export interface DashboardStats {
  todoCount: number;
  todayDoneCount: number;
  todayFocusMinutes: number;
  weeklyDoneCount: number;
}

// 聊天会话
export interface ChatSession {
  id: string;
  title: string;
  createdAt: number;
  updatedAt: number;
  messageCount?: number;
}

// AI 处理过程日志（用于调试和追踪重试过程）
export interface ProcessLog {
  id: number;
  requestId: string;
  sessionId?: string;
  stepNumber: number;
  stepType: string;
  userInput?: string;
  aiRequest?: string;
  aiResponse?: string;
  functionName?: string;
  functionArgs?: string;
  functionResult?: string;
  errorMessage?: string;
  durationMs?: number;
  createdAt: string;
}

export const aiChatApi = {
  /**
   * 发送对话消息
   * @param messages 消息列表
   * @param sessionId 可选的会话ID，用于关联处理日志
   * @param context 可选的操作上下文，用于代词解析
   * @returns 聊天响应
   */
  async chat(messages: ChatMessage[], sessionId?: string, context?: OperationContext): Promise<ChatResponse> {
    return await invoke('ai_assistant_chat', { messages, sessionId, context });
  },

  /**
   * 获取仪表盘统计数据
   * @returns 仪表盘统计
   */
  async getDashboardStats(): Promise<DashboardStats> {
    return await invoke('ai_get_dashboard_stats');
  },

  /**
   * 保存会话
   * @param session 会话信息
   */
  async saveSession(session: ChatSession): Promise<void> {
    // 将时间转换为 ISO 字符串格式
    const createdAt = typeof session.createdAt === 'number'
      ? new Date(session.createdAt).toISOString()
      : session.createdAt;
    const updatedAt = typeof session.updatedAt === 'number'
      ? new Date(session.updatedAt).toISOString()
      : session.updatedAt;

    await invoke('ai_save_chat_session', {
      session: {
        id: session.id,
        title: session.title,
        createdAt,
        updatedAt,
      }
    });
  },

  /**
   * 获取会话列表
   * @returns 会话列表
   */
  async getSessions(): Promise<ChatSession[]> {
    return await invoke('ai_get_chat_sessions');
  },

  /**
   * 删除会话
   * @param sessionId 会话ID
   */
  async deleteSession(sessionId: string): Promise<void> {
    await invoke('ai_delete_chat_session', { sessionId });
  },

  /**
   * 获取会话消息
   * @param sessionId 会话ID
   * @returns 消息列表
   */
  async getSessionMessages(sessionId: string): Promise<ChatMessage[]> {
    return await invoke('ai_get_session_messages', { sessionId });
  },

  /**
   * 保存会话消息
   * @param sessionId 会话ID
   * @param message 消息内容
   */
  async saveMessage(sessionId: string, message: ChatMessage): Promise<void> {
    await invoke('ai_save_chat_message', {
      sessionId,
      role: message.role,
      content: message.content,
    });
  },

  /**
   * 更新会话标题（通过重新保存会话实现）
   * @param sessionId 会话ID
   * @param title 新标题
   */
  async updateSessionTitle(sessionId: string, title: string): Promise<void> {
    // 通过重新保存会话来更新标题
    const now = new Date().toISOString();
    await invoke('ai_save_chat_session', {
      session: {
        id: sessionId,
        title: title,
        createdAt: now,
        updatedAt: now,
      }
    });
  },

  /**
   * 清空会话消息
   * @param sessionId 会话ID
   */
  async clearSessionMessages(sessionId: string): Promise<void> {
    await invoke('ai_clear_session_messages', { sessionId });
  },

  /**
   * 获取 AI 处理过程日志（用于调试和追踪重试过程）
   * @param requestId 按请求ID查询
   * @param sessionId 按会话ID查询
   * @param limit 最大返回数量
   * @returns 处理日志列表
   */
  async getProcessLogs(requestId?: string, sessionId?: string, limit?: number): Promise<ProcessLog[]> {
    return await invoke('ai_get_process_logs', { requestId, sessionId, limit });
  },

  /**
   * 清空 AI 处理过程日志
   * @param sessionId 可选，只清空指定会话的日志
   * @returns 删除的记录数
   */
  async clearProcessLogs(sessionId?: string): Promise<number> {
    return await invoke('ai_clear_process_logs', { sessionId });
  },
};
