/**
 * AI Chat 组件导出
 *
 * 使用示例:
 * import { AiChat, MessageBubble, LoadingMessage, WelcomeMessage } from '@/components/aiChat';
 */

export { default as AiChat } from './AiChat.vue';
export { default as ChatSidebar } from './ChatSidebar.vue';
export { default as MessageBubble } from './MessageBubble.vue';
export { default as LoadingMessage } from './LoadingMessage.vue';
export { default as WelcomeMessage } from './WelcomeMessage.vue';

// 导出类型
export type { ChatMessage } from './MessageBubble.vue';
