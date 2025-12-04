<template>
  <div class="notification-bell" @click="handleClick">
    <n-badge :value="unreadCount" :max="99" :show="unreadCount > 0">
      <div class="bell-icon">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          class="icon"
        >
          <path
            d="M15 17H20L18.5951 15.5951C18.2141 15.2141 18 14.6973 18 14.1585V11C18 8.38757 16.3304 6.16509 14 5.34142V5C14 3.89543 13.1046 3 12 3C10.8954 3 10 3.89543 10 5V5.34142C7.66962 6.16509 6 8.38757 6 11V14.1585C6 14.6973 5.78595 15.2141 5.40493 15.5951L4 17H9M15 17V18C15 19.6569 13.6569 21 12 21C10.3431 21 9 19.6569 9 18V17M15 17H9"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </div>
    </n-badge>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { NBadge } from 'naive-ui'
import { notificationApi } from '@/api/notificationApi'

const router = useRouter()
const unreadCount = ref(0)
let refreshTimer: number | null = null

onMounted(() => {
  loadUnreadCount()
  // 每30秒刷新一次未读数量
  refreshTimer = window.setInterval(() => {
    loadUnreadCount()
  }, 30000)
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})

async function loadUnreadCount() {
  try {
    unreadCount.value = await notificationApi.getUnreadCount()
  } catch (error) {
    console.error('获取未读数量失败:', error)
  }
}

function handleClick() {
  router.push({ name: 'notification-center' })
}

// 暴露刷新方法供外部调用
defineExpose({
  refresh: loadUnreadCount
})
</script>

<style scoped>
.notification-bell {
  cursor: pointer;
  padding: 8px;
  border-radius: 8px;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.notification-bell:hover {
  background: rgba(99, 102, 241, 0.1);
}

.bell-icon {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgb(148, 163, 184);
  transition: color 0.2s ease;
}

.notification-bell:hover .bell-icon {
  color: rgb(167, 139, 250);
}

.icon {
  width: 100%;
  height: 100%;
}

:deep(.n-badge-sup) {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  font-weight: 600;
  font-size: 10px;
  box-shadow: 0 2px 8px rgba(239, 68, 68, 0.4);
}
</style>
