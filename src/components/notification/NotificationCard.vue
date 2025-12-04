<template>
  <div
    class="notification-card"
    :class="{ 'unread': !notification.isRead }"
    @click="handleClick"
  >
    <!-- 未读指示条 -->
    <div v-if="!notification.isRead" class="unread-indicator"></div>

    <div class="card-content">
      <!-- 图标和标题 -->
      <div class="card-header">
        <div class="icon-wrapper" :class="`icon-${notification.notificationType}`">
          <span class="icon">{{ getIcon(notification.notificationType) }}</span>
        </div>
        <div class="title-section">
          <h4 class="title">{{ notification.title }}</h4>
          <span class="time">{{ formatTime(notification.createdAt) }}</span>
        </div>
      </div>

      <!-- 内容摘要 -->
      <div class="content-preview">
        {{ truncateContent(notification.content) }}
      </div>

      <!-- 操作按钮 -->
      <div class="actions" @click.stop>
        <n-button
          text
          size="small"
          @click="handleIgnore"
          class="action-btn"
        >
          忽略
        </n-button>
        <n-button
          text
          size="small"
          type="primary"
          @click="handleViewDetail"
          class="action-btn"
        >
          查看详情
        </n-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NButton } from 'naive-ui'
import type { Notification } from '@/types/notification'
import { formatDistanceToNow } from 'date-fns'
import { zhCN } from 'date-fns/locale'

interface Props {
  notification: Notification
}

interface Emits {
  (e: 'markRead', id: number): void
  (e: 'ignore', id: number): void
  (e: 'viewDetail', notification: Notification): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const getIcon = (type: string): string => {
  const icons = {
    tip: '💡',
    daily_report: '📊',
    weekly_report: '🌟'
  }
  return icons[type as keyof typeof icons] || '📢'
}

const formatTime = (timestamp: string): string => {
  try {
    const date = new Date(timestamp)
    return formatDistanceToNow(date, { addSuffix: true, locale: zhCN })
  } catch {
    return timestamp
  }
}

const truncateContent = (content: string): string => {
  const maxLength = 120
  if (content.length <= maxLength) return content
  return content.substring(0, maxLength) + '...'
}

const handleClick = () => {
  if (!props.notification.isRead) {
    emit('markRead', props.notification.id)
  }
}

const handleIgnore = () => {
  emit('ignore', props.notification.id)
}

const handleViewDetail = () => {
  if (!props.notification.isRead) {
    emit('markRead', props.notification.id)
  }
  emit('viewDetail', props.notification)
}
</script>

<style scoped>
.notification-card {
  position: relative;
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.6);
  border-radius: 12px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.2s ease;
  backdrop-filter: blur(8px);
}

.notification-card:hover {
  background: rgba(15, 23, 42, 0.7);
  border-color: rgba(99, 102, 241, 0.4);
  transform: translateX(4px);
}

.notification-card.unread {
  border-left: 3px solid #60a5fa;
}

.unread-indicator {
  position: absolute;
  left: -1px;
  top: 0;
  bottom: 0;
  width: 3px;
  background: linear-gradient(180deg, #60a5fa 0%, #3b82f6 100%);
  border-radius: 12px 0 0 12px;
}

.card-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.card-header {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.icon-wrapper {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-size: 20px;
}

.icon-tip {
  background: rgba(251, 191, 36, 0.1);
  border: 1px solid rgba(251, 191, 36, 0.2);
}

.icon-daily_report {
  background: rgba(96, 165, 250, 0.1);
  border: 1px solid rgba(96, 165, 250, 0.2);
}

.icon-weekly_report {
  background: rgba(167, 139, 250, 0.1);
  border: 1px solid rgba(167, 139, 250, 0.2);
}

.title-section {
  flex: 1;
  min-width: 0;
}

.title {
  font-size: 14px;
  font-weight: 600;
  color: rgb(226, 232, 240);
  margin: 0 0 4px 0;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
}

.time {
  font-size: 12px;
  color: rgb(100, 116, 139);
}

.content-preview {
  font-size: 13px;
  color: rgb(148, 163, 184);
  line-height: 1.6;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
  padding-left: 52px;
}

.actions {
  display: flex;
  gap: 16px;
  padding-left: 52px;
}

.action-btn {
  font-size: 12px;
  color: rgb(148, 163, 184);
  padding: 4px 8px;
}

.action-btn:hover {
  color: rgb(226, 232, 240);
}

:deep(.n-button--primary-type) {
  color: #818cf8 !important;
}

:deep(.n-button--primary-type:hover) {
  color: #a78bfa !important;
}
</style>
