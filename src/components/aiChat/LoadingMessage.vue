<template>
  <div class="loading-message">
    <!-- AI 头像 -->
    <div class="loading-avatar">
      <n-icon size="18" class="avatar-icon">
        <ChatbubblesOutline />
      </n-icon>
    </div>

    <!-- 加载内容 -->
    <div class="loading-content-wrapper">
      <div class="loading-content">
        <!-- 跳动的点点 -->
        <div class="loading-dots">
          <span class="dot"></span>
          <span class="dot"></span>
          <span class="dot"></span>
        </div>

        <!-- 操作提示文本 -->
        <div v-if="action" class="loading-action">
          {{ action }}
        </div>
      </div>

      <!-- 时间戳 -->
      <div class="loading-timestamp">
        {{ currentTime }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { NIcon } from 'naive-ui';
import { ChatbubblesOutline } from '@vicons/ionicons5';
import dayjs from 'dayjs';

interface Props {
  action?: string; // 当前操作描述，如 "正在查询任务..."
}

withDefaults(defineProps<Props>(), {
  action: ''
});

// 当前时间
const currentTime = ref(dayjs().format('HH:mm'));
let timer: NodeJS.Timeout | null = null;

onMounted(() => {
  // 每秒更新时间
  timer = setInterval(() => {
    currentTime.value = dayjs().format('HH:mm');
  }, 1000);
});

onUnmounted(() => {
  if (timer) {
    clearInterval(timer);
  }
});
</script>

<style scoped>
.loading-message {
  display: flex;
  gap: 12px;
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

/* 头像 */
.loading-avatar {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: var(--accent-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: var(--shadow-sm);
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
    box-shadow: var(--shadow-sm);
  }
  50% {
    opacity: 0.8;
    box-shadow: 0 0 20px var(--accent-glow);
  }
}

.avatar-icon {
  color: white;
}

/* 内容区域 */
.loading-content-wrapper {
  display: flex;
  flex-direction: column;
  gap: 4px;
  align-items: flex-start;
}

/* 加载内容 */
.loading-content {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 16px;
  border-top-left-radius: 4px;
  padding: 12px 20px;
  box-shadow: var(--shadow-sm);
  min-width: 80px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* 跳动的点点 */
.loading-dots {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 20px;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-dim);
  animation: bounce 1.4s ease-in-out infinite;
}

.dot:nth-child(1) {
  animation-delay: 0s;
}

.dot:nth-child(2) {
  animation-delay: 0.2s;
}

.dot:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes bounce {
  0%, 80%, 100% {
    transform: translateY(0);
    opacity: 0.5;
  }
  40% {
    transform: translateY(-10px);
    opacity: 1;
  }
}

/* 操作提示文本 */
.loading-action {
  font-size: 12px;
  color: var(--text-muted);
  font-style: italic;
  animation: fadeInOut 2s ease-in-out infinite;
}

@keyframes fadeInOut {
  0%, 100% {
    opacity: 0.5;
  }
  50% {
    opacity: 1;
  }
}

/* 时间戳 */
.loading-timestamp {
  font-size: 11px;
  color: var(--text-dim);
  padding: 0 8px;
  opacity: 0.7;
}

/* 响应式 */
@media (max-width: 768px) {
  .loading-avatar {
    width: 32px;
    height: 32px;
  }

  .loading-content {
    padding: 10px 16px;
  }

  .dot {
    width: 6px;
    height: 6px;
  }

  .loading-action {
    font-size: 11px;
  }
}
</style>
