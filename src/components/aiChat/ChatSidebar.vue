<template>
  <div class="chat-sidebar-wrapper">
    <div class="sidebar-header">
      <h2 class="sidebar-title">
        <n-icon size="16" class="title-icon"><TimeOutline /></n-icon>
        历史记录
      </h2>
      <n-button text class="close-btn-mobile" @click="emit('close')">
        <template #icon><n-icon><CloseOutline /></n-icon></template>
      </n-button>
    </div>

    <div class="sidebar-actions">
      <n-button type="primary" block class="new-chat-btn" @click="handleNewChat">
        <template #icon><n-icon><AddOutline /></n-icon></template>
        新对话
      </n-button>
    </div>

    <div class="sidebar-search">
      <n-input v-model:value="searchQuery" placeholder="搜索对话..." size="small" clearable>
        <template #prefix><n-icon size="14"><SearchOutline /></n-icon></template>
      </n-input>
    </div>

    <div class="history-list">
      <!-- 有会话记录时显示分组列表 -->
      <template v-if="filteredHistory.length > 0">
        <div v-for="group in filteredHistory" :key="group.label" class="history-group">
          <div class="group-label">{{ group.label }}</div>
          <div class="group-items">
            <div
              v-for="item in group.items"
              :key="item.id"
              class="history-item"
              :class="{ active: currentSessionId === item.id }"
              @click="handleSelectSession(item.id)"
            >
              <div class="item-content">
                <n-icon size="14" class="item-icon"><ChatbubbleOutline /></n-icon>
                <span class="item-title">{{ item.title }}</span>
              </div>
              <n-button text size="tiny" class="item-delete" @click.stop="handleDeleteSession(item.id)">
                <template #icon><n-icon size="12"><TrashOutline /></n-icon></template>
              </n-button>
            </div>
          </div>
        </div>
      </template>

      <!-- 无会话记录时显示空状态 -->
      <div v-else class="empty-state">
        <n-icon size="48" class="empty-icon"><ChatbubbleOutline /></n-icon>
        <p class="empty-text">暂无对话记录</p>
        <p class="empty-hint">点击上方"新对话"开始聊天</p>
      </div>
    </div>

    <div class="sidebar-footer">
      <div class="user-info">
        <div class="user-avatar"><n-icon size="16"><PersonOutline /></n-icon></div>
        <div class="user-details">
          <div class="user-name">Developer</div>
          <div class="user-plan">AI 助手</div>
        </div>
        <n-button text size="small" class="settings-btn">
          <template #icon><n-icon size="16"><SettingsOutline /></n-icon></template>
        </n-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { NButton, NIcon, NInput } from 'naive-ui';
import { AddOutline, TimeOutline, SearchOutline, ChatbubbleOutline, TrashOutline, PersonOutline, SettingsOutline, CloseOutline } from '@vicons/ionicons5';
import { useAiChatStore } from '@/stores/aiChatStore';
import dayjs from 'dayjs';

interface Props {
  currentSessionId?: string | null;
}

const props = withDefaults(defineProps<Props>(), {
  currentSessionId: null,
});

const emit = defineEmits<{
  'new-chat': [];
  'select-session': [sessionId: string];
  'close': [];
}>();

const aiChatStore = useAiChatStore();
const searchQuery = ref('');

// 将会话按日期分组
const groupedHistory = computed(() => {
  const sessions = aiChatStore.sessions;
  if (!sessions || sessions.length === 0) return [];

  const today = dayjs().startOf('day');
  const yesterday = dayjs().subtract(1, 'day').startOf('day');
  const weekAgo = dayjs().subtract(7, 'day').startOf('day');

  const groups: { label: string; items: typeof sessions }[] = [
    { label: '今天', items: [] },
    { label: '昨天', items: [] },
    { label: '最近7天', items: [] },
    { label: '更早', items: [] },
  ];

  sessions.forEach(session => {
    const sessionDate = dayjs(session.updatedAt);

    if (sessionDate.isAfter(today) || sessionDate.isSame(today, 'day')) {
      groups[0].items.push(session);
    } else if (sessionDate.isAfter(yesterday) || sessionDate.isSame(yesterday, 'day')) {
      groups[1].items.push(session);
    } else if (sessionDate.isAfter(weekAgo)) {
      groups[2].items.push(session);
    } else {
      groups[3].items.push(session);
    }
  });

  // 过滤掉空的分组
  return groups.filter(group => group.items.length > 0);
});

// 搜索过滤
const filteredHistory = computed(() => {
  if (!searchQuery.value.trim()) return groupedHistory.value;

  const query = searchQuery.value.toLowerCase();
  return groupedHistory.value
    .map(group => ({
      ...group,
      items: group.items.filter(item =>
        item.title.toLowerCase().includes(query)
      )
    }))
    .filter(group => group.items.length > 0);
});

const handleNewChat = () => {
  emit('new-chat');
};

const handleSelectSession = (sessionId: string) => {
  console.log('[ChatSidebar] ==========================================');
  console.log('[ChatSidebar] 点击会话:', sessionId);
  console.log('[ChatSidebar] 当前选中会话:', props.currentSessionId);
  console.log('[ChatSidebar] 即将发送 select-session 事件');
  emit('select-session', sessionId);
  console.log('[ChatSidebar] select-session 事件已发送');
  console.log('[ChatSidebar] ==========================================');
};

const handleDeleteSession = async (sessionId: string) => {
  try {
    await aiChatStore.deleteSession(sessionId);
  } catch (error) {
    console.error('删除会话失败:', error);
  }
};

// 组件挂载时加载会话列表
onMounted(async () => {
  if (aiChatStore.sessions.length === 0) {
    await aiChatStore.loadSessions();
  }
});
</script>

<style scoped>
.chat-sidebar-wrapper { display: flex; flex-direction: column; height: 100%; background: var(--sidebar-bg); }
.sidebar-header { display: flex; align-items: center; justify-content: space-between; padding: 16px; border-bottom: 1px solid var(--sidebar-border); }
.sidebar-title { display: flex; align-items: center; gap: 8px; font-size: 13px; font-weight: 600; color: var(--text-primary); margin: 0; }
.title-icon { color: var(--accent-primary); }
.close-btn-mobile { color: var(--text-muted); display: none; }
@media (max-width: 768px) { .close-btn-mobile { display: flex; } }
.sidebar-actions { padding: 12px 16px; }
.new-chat-btn { background: var(--accent-primary); border-radius: 8px; font-size: 13px; box-shadow: var(--shadow-sm); }
.new-chat-btn:hover { background: var(--accent-primary-hover); transform: translateY(-1px); box-shadow: var(--shadow-md); }
.sidebar-search { padding: 0 16px 12px; }
.sidebar-search :deep(.n-input) { background: var(--input-bg); border: 1px solid var(--input-border); border-radius: 8px; }
.sidebar-search :deep(.n-input:focus-within) { border-color: var(--input-focus-border); }
.sidebar-search :deep(.n-input__input-el) { color: var(--text-primary); font-size: 12px; }
.sidebar-search :deep(.n-input__placeholder) { color: var(--text-dim); }
.history-list { flex: 1; overflow-y: auto; padding: 8px; }
.history-list::-webkit-scrollbar { width: 4px; }
.history-list::-webkit-scrollbar-track { background: var(--scrollbar-track); }
.history-list::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb); border-radius: 2px; }
.history-list::-webkit-scrollbar-thumb:hover { background: var(--scrollbar-thumb-hover); }
.history-group { margin-bottom: 16px; }
.group-label { font-size: 10px; font-weight: 600; color: var(--text-dim); text-transform: uppercase; letter-spacing: 0.5px; padding: 8px 12px 4px; }
.group-items { display: flex; flex-direction: column; gap: 2px; }
.history-item { display: flex; align-items: center; justify-content: space-between; padding: 10px 12px; border-radius: 8px; cursor: pointer; transition: all 0.2s ease; color: var(--text-secondary); }
.history-item:hover { background: var(--bg-hover); color: var(--text-primary); }
.history-item.active { background: var(--card-bg); border: 1px solid var(--border-active); color: var(--text-primary); }
.item-content { display: flex; align-items: center; gap: 8px; flex: 1; min-width: 0; }
.item-icon { flex-shrink: 0; color: var(--text-dim); }
.history-item.active .item-icon { color: var(--accent-primary); }
.item-title { font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.item-delete { opacity: 0; color: var(--text-dim); transition: opacity 0.2s ease; }
.history-item:hover .item-delete { opacity: 1; }
.item-delete:hover { color: var(--error); }
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 40px 20px; text-align: center; }
.empty-icon { color: var(--text-dim); opacity: 0.5; margin-bottom: 12px; }
.empty-text { font-size: 13px; color: var(--text-muted); margin: 0 0 4px 0; }
.empty-hint { font-size: 11px; color: var(--text-dim); margin: 0; }
.sidebar-footer { padding: 12px 16px; border-top: 1px solid var(--sidebar-border); }
.user-info { display: flex; align-items: center; gap: 10px; }
.user-avatar { width: 32px; height: 32px; border-radius: 8px; background: var(--card-bg); border: 1px solid var(--card-border); display: flex; align-items: center; justify-content: center; color: var(--text-secondary); flex-shrink: 0; }
.user-details { flex: 1; min-width: 0; }
.user-name { font-size: 12px; font-weight: 500; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.user-plan { font-size: 10px; color: var(--text-dim); }
.settings-btn { color: var(--text-muted); flex-shrink: 0; }
.settings-btn:hover { color: var(--text-primary); }
</style>
