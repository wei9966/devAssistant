<template>
  <div class="notification-center">
    <!-- 顶部标题栏 -->
    <div class="page-header">
      <div class="header-left">
        <h2 class="page-title">通知中心</h2>
        <span class="count-badge">{{ totalCount }} 条通知</span>
      </div>
      <div class="header-actions">
        <n-button
          size="small"
          @click="handleMarkAllRead"
          :disabled="unreadCount === 0"
        >
          全部已读
        </n-button>
        <n-button
          size="small"
          @click="handleClearAll"
        >
          清空
        </n-button>
        <n-button
          size="small"
          @click="handleGenerateDailyReport"
          :loading="generatingDaily"
        >
          手动生成日报
        </n-button>
        <n-button
          size="small"
          @click="handleGenerateWeeklyReport"
          :loading="generatingWeekly"
        >
          手动生成周报
        </n-button>
        <n-button
          size="small"
          type="primary"
          @click="showSettings = true"
        >
          <template #icon>
            <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" style="width: 16px; height: 16px;">
              <path d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              <path d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </template>
          设置
        </n-button>
      </div>
    </div>

    <!-- 筛选标签 -->
    <div class="filter-tabs">
      <div
        v-for="tab in filterTabs"
        :key="tab.value"
        class="filter-tab"
        :class="{ active: activeFilter === tab.value }"
        @click="handleFilterChange(tab.value)"
      >
        <span class="tab-label">{{ tab.label }}</span>
        <span v-if="tab.count !== undefined" class="tab-count">{{ tab.count }}</span>
      </div>
    </div>

    <!-- 通知列表 -->
    <div class="notifications-container">
      <n-spin :show="loading">
        <!-- 加载状态 -->
        <div v-if="loading" class="loading-placeholder">
          <div class="loading-text">加载中...</div>
        </div>

        <!-- 通知列表 -->
        <div v-else-if="notifications.length > 0" class="notifications-list">
          <NotificationCard
            v-for="notification in notifications"
            :key="notification.id"
            :notification="notification"
            @mark-read="handleMarkRead"
            @ignore="handleIgnore"
            @view-detail="handleViewDetail"
          />
        </div>

        <!-- 空状态 -->
        <div v-else class="empty-state">
          <div class="empty-icon">
            <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M15 17H20L18.5951 15.5951C18.2141 15.2141 18 14.6973 18 14.1585V11C18 8.38757 16.3304 6.16509 14 5.34142V5C14 3.89543 13.1046 3 12 3C10.8954 3 10 3.89543 10 5V5.34142C7.66962 6.16509 6 8.38757 6 11V14.1585C6 14.6973 5.78595 15.2141 5.40493 15.5951L4 17H9M15 17V18C15 19.6569 13.6569 21 12 21C10.3431 21 9 19.6569 9 18V17M15 17H9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
          <h3 class="empty-title">暂无通知</h3>
          <p class="empty-desc">{{ getEmptyMessage() }}</p>
        </div>
      </n-spin>
    </div>

    <!-- 设置模态框 -->
    <NotificationSettings
      v-model:show="showSettings"
      @saved="handleSettingsSaved"
    />

    <!-- 详情模态框 -->
    <n-modal
      v-model:show="showDetail"
      preset="card"
      :title="currentNotification?.title"
      class="notification-detail-modal"
      :style="{ width: '700px', maxWidth: '90vw' }"
      :bordered="false"
    >
      <div class="detail-content">
        <div class="detail-meta">
          <span class="detail-type" :class="`type-${currentNotification?.notificationType}`">
            {{ getTypeLabel(currentNotification?.notificationType) }}
          </span>
          <span class="detail-time">{{ formatDetailTime(currentNotification?.createdAt) }}</span>
        </div>
        <div class="detail-text" v-html="formatContent(currentNotification?.content)"></div>
      </div>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NButton, NSpin, NModal, useMessage, useDialog } from 'naive-ui'
import { notificationApi } from '@/api/notificationApi'
import type { Notification, NotificationType } from '@/types/notification'
import NotificationCard from '@/components/notification/NotificationCard.vue'
import NotificationSettings from '@/components/notification/NotificationSettings.vue'

const message = useMessage()
const dialog = useDialog()

const loading = ref(false)
const generatingDaily = ref(false)
const generatingWeekly = ref(false)
const showSettings = ref(false)
const showDetail = ref(false)
const currentNotification = ref<Notification | null>(null)

const notifications = ref<Notification[]>([])
const activeFilter = ref<'all' | 'unread' | NotificationType>('all')

type FilterValue = 'all' | 'unread' | NotificationType

const filterTabs = computed(() => {
  const unreadCount = notifications.value.filter(n => !n.isRead).length
  const tipCount = notifications.value.filter(n => n.notificationType === 'tip').length
  const dailyCount = notifications.value.filter(n => n.notificationType === 'daily_report').length
  const weeklyCount = notifications.value.filter(n => n.notificationType === 'weekly_report').length
  const activityCount = notifications.value.filter(n => n.notificationType === 'activity_summary').length

  return [
    { label: '全部', value: 'all' as FilterValue, count: notifications.value.length },
    { label: '未读', value: 'unread' as FilterValue, count: unreadCount },
    { label: 'Tips', value: 'tip' as FilterValue, count: tipCount },
    { label: '日报', value: 'daily_report' as FilterValue, count: dailyCount },
    { label: '周报', value: 'weekly_report' as FilterValue, count: weeklyCount },
    { label: '活动总结', value: 'activity_summary' as FilterValue, count: activityCount }
  ]
})

const totalCount = computed(() => notifications.value.length)
const unreadCount = computed(() => notifications.value.filter(n => !n.isRead).length)

onMounted(() => {
  loadNotifications()
})

async function loadNotifications() {
  loading.value = true
  try {
    const params: any = {}

    if (activeFilter.value === 'unread') {
      params.isRead = false
    } else if (activeFilter.value !== 'all') {
      params.notificationType = activeFilter.value
    }

    notifications.value = await notificationApi.list(params)
  } catch (error) {
    console.error('加载通知失败:', error)
    message.error('加载通知失败')
  } finally {
    loading.value = false
  }
}

function handleFilterChange(filter: FilterValue) {
  activeFilter.value = filter
  loadNotifications()
}

async function handleMarkRead(id: number) {
  try {
    await notificationApi.markRead(id)
    const notification = notifications.value.find(n => n.id === id)
    if (notification) {
      notification.isRead = true
    }
  } catch (error) {
    console.error('标记已读失败:', error)
    message.error('标记已读失败')
  }
}

async function handleMarkAllRead() {
  try {
    await notificationApi.markAllRead()
    notifications.value.forEach(n => {
      n.isRead = true
    })
    message.success('已全部标记为已读')
  } catch (error) {
    console.error('标记已读失败:', error)
    message.error('操作失败')
  }
}

async function handleIgnore(id: number) {
  try {
    await notificationApi.delete(id)
    notifications.value = notifications.value.filter(n => n.id !== id)
    message.success('已忽略')
  } catch (error) {
    console.error('删除通知失败:', error)
    message.error('操作失败')
  }
}

function handleClearAll() {
  dialog.warning({
    title: '确认清空',
    content: '确定要清空所有通知吗？此操作不可撤销。',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await notificationApi.clearAll()
        notifications.value = []
        message.success('已清空所有通知')
      } catch (error) {
        console.error('清空通知失败:', error)
        message.error('操作失败')
      }
    }
  })
}

async function handleGenerateDailyReport() {
  generatingDaily.value = true
  try {
    await notificationApi.generateDailyReport()
    message.success('日报生成成功')
    await loadNotifications()
  } catch (error) {
    console.error('生成日报失败:', error)
    message.error('生成日报失败')
  } finally {
    generatingDaily.value = false
  }
}

async function handleGenerateWeeklyReport() {
  generatingWeekly.value = true
  try {
    await notificationApi.generateWeeklyReport()
    message.success('周报生成成功')
    await loadNotifications()
  } catch (error) {
    console.error('生成周报失败:', error)
    message.error('生成周报失败')
  } finally {
    generatingWeekly.value = false
  }
}

function handleViewDetail(notification: Notification) {
  currentNotification.value = notification
  showDetail.value = true
}

function handleSettingsSaved() {
  message.success('设置已保存')
}

function getEmptyMessage(): string {
  if (activeFilter.value === 'unread') {
    return '所有通知都已读完了'
  } else if (activeFilter.value === 'tip') {
    return '暂无工作提示'
  } else if (activeFilter.value === 'daily_report') {
    return '暂无日报'
  } else if (activeFilter.value === 'weekly_report') {
    return '暂无周报'
  } else if (activeFilter.value === 'activity_summary') {
    return '暂无活动总结'
  }
  return '暂时没有任何通知'
}

function getTypeLabel(type?: NotificationType): string {
  const labels = {
    tip: '💡 工作提示',
    daily_report: '📊 每日报告',
    weekly_report: '🌟 每周总结',
    activity_summary: '⏱️ 活动总结'
  }
  return type ? labels[type] : ''
}

function formatDetailTime(timestamp?: string): string {
  if (!timestamp) return ''
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function formatContent(content?: string): string {
  if (!content) return ''
  // 将换行符转换为 <br>
  return content.replace(/\n/g, '<br>')
}
</script>

<style scoped>
.notification-center {
  display: flex;
  flex-direction: column;
  height: calc(100vh - 80px);
  gap: 20px;
}

/* 顶部标题栏 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.page-title {
  font-size: 24px;
  font-weight: 700;
  color: rgb(226, 232, 240);
  margin: 0;
  background: linear-gradient(135deg, #818cf8 0%, #a78bfa 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.count-badge {
  font-size: 12px;
  color: rgb(148, 163, 184);
  padding: 4px 12px;
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 12px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

/* 筛选标签 */
.filter-tabs {
  display: flex;
  gap: 12px;
  padding: 12px;
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.6);
  border-radius: 12px;
  overflow-x: auto;
}

.filter-tab {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  background: transparent;
  border: 1px solid transparent;
}

.filter-tab:hover {
  background: rgba(51, 65, 85, 0.5);
}

.filter-tab.active {
  background: rgba(99, 102, 241, 0.15);
  border-color: rgba(99, 102, 241, 0.3);
}

.tab-label {
  font-size: 13px;
  font-weight: 500;
  color: rgb(148, 163, 184);
}

.filter-tab.active .tab-label {
  color: rgb(167, 139, 250);
}

.tab-count {
  font-size: 11px;
  color: rgb(100, 116, 139);
  font-weight: 600;
  background: rgba(51, 65, 85, 0.5);
  padding: 2px 6px;
  border-radius: 8px;
  min-width: 20px;
  text-align: center;
}

.filter-tab.active .tab-count {
  background: rgba(99, 102, 241, 0.2);
  color: rgb(167, 139, 250);
}

/* 通知容器 */
.notifications-container {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.notifications-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
  max-height: 100%;
  overflow-y: auto;
  padding-right: 8px;
  padding-bottom: 20px;
}

/* 自定义滚动条 */
.notifications-list::-webkit-scrollbar {
  width: 6px;
}

.notifications-list::-webkit-scrollbar-track {
  background: transparent;
}

.notifications-list::-webkit-scrollbar-thumb {
  background: rgba(148, 163, 184, 0.2);
  border-radius: 3px;
}

.notifications-list::-webkit-scrollbar-thumb:hover {
  background: rgba(148, 163, 184, 0.3);
}

/* 加载状态 */
.loading-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
}

.loading-text {
  color: rgb(148, 163, 184);
  font-size: 14px;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 300px;
  text-align: center;
  padding: 40px;
}

.empty-icon {
  width: 80px;
  height: 80px;
  margin-bottom: 24px;
  color: rgb(71, 85, 105);
  opacity: 0.5;
}

.empty-icon svg {
  width: 100%;
  height: 100%;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: rgb(148, 163, 184);
  margin: 0 0 8px 0;
}

.empty-desc {
  font-size: 14px;
  color: rgb(100, 116, 139);
  margin: 0;
}

/* 详情模态框 */
.detail-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-height: 60vh;
  overflow-y: auto;
}

.detail-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(51, 65, 85, 0.6);
}

.detail-type {
  font-size: 12px;
  font-weight: 600;
  padding: 4px 12px;
  border-radius: 6px;
}

.type-tip {
  background: rgba(251, 191, 36, 0.1);
  color: #fbbf24;
  border: 1px solid rgba(251, 191, 36, 0.2);
}

.type-daily_report {
  background: rgba(96, 165, 250, 0.1);
  color: #60a5fa;
  border: 1px solid rgba(96, 165, 250, 0.2);
}

.type-weekly_report {
  background: rgba(167, 139, 250, 0.1);
  color: #a78bfa;
  border: 1px solid rgba(167, 139, 250, 0.2);
}

.type-activity_summary {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
  border: 1px solid rgba(34, 197, 94, 0.2);
}

.detail-time {
  font-size: 12px;
  color: rgb(100, 116, 139);
}

.detail-text {
  font-size: 14px;
  color: rgb(203, 213, 225);
  line-height: 1.8;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 50vh;
  overflow-y: auto;
  padding-right: 8px;
}

/* 详情模态框滚动条样式 */
.detail-content::-webkit-scrollbar,
.detail-text::-webkit-scrollbar {
  width: 6px;
}

.detail-content::-webkit-scrollbar-track,
.detail-text::-webkit-scrollbar-track {
  background: transparent;
}

.detail-content::-webkit-scrollbar-thumb,
.detail-text::-webkit-scrollbar-thumb {
  background: rgba(148, 163, 184, 0.2);
  border-radius: 3px;
}

.detail-content::-webkit-scrollbar-thumb:hover,
.detail-text::-webkit-scrollbar-thumb:hover {
  background: rgba(148, 163, 184, 0.3);
}

/* Naive UI 样式覆盖 */
:deep(.n-card) {
  background: rgba(15, 23, 42, 0.95);
  border: 1px solid rgba(51, 65, 85, 0.6);
}

:deep(.n-card-header) {
  border-bottom: 1px solid rgba(51, 65, 85, 0.6);
  padding: 20px 24px;
}

:deep(.n-card-header .n-card-header__main) {
  color: rgb(226, 232, 240);
  font-size: 16px;
  font-weight: 600;
}

:deep(.n-button) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-color: transparent;
  --n-color-hover: rgba(99, 102, 241, 0.1);
  --n-text-color: rgb(203, 213, 225);
}

:deep(.n-button--primary-type) {
  --n-color: rgb(99, 102, 241);
  --n-color-hover: rgb(79, 70, 229);
  --n-text-color: rgb(255, 255, 255);
  --n-border: none;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.2);
}

:deep(.n-button[disabled]) {
  opacity: 0.5;
}

:deep(.n-spin-container) {
  height: 100%;
  display: flex;
  flex-direction: column;
}

:deep(.n-spin-content) {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
</style>
