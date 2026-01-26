<template>
  <Teleport to="body">
    <Transition name="sedentary-modal">
      <div v-if="show" class="sedentary-overlay" @click.self="handleClose">
        <div class="sedentary-modal">
          <!-- 关闭按钮 -->
          <button class="close-btn" @click="handleClose">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>

          <!-- 主要内容 -->
          <div class="modal-content">
            <!-- 休息图标 - 咖啡杯动画 -->
            <div class="icon-container">
              <div class="icon-glow"></div>
              <svg class="rest-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M17 8h1a4 4 0 0 1 0 8h-1" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                <path d="M3 8h14v9a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4V8Z" stroke="currentColor" stroke-width="2"/>
                <path d="M6 1v3M10 1v3M14 1v3" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              </svg>
            </div>

            <!-- 标题 -->
            <h1 class="modal-title">该休息一下了</h1>

            <!-- 工作时长 -->
            <p class="work-duration">您已连续工作 {{ formatDuration(workDuration) }}</p>

            <!-- 提示文案 -->
            <p class="tip-text">{{ currentTip }}</p>

            <!-- 倒计时（可选）-->
            <div v-if="countdown > 0" class="countdown">
              <span>{{ countdown }}</span> 秒后自动关闭
            </div>

            <!-- 操作按钮 -->
            <button class="confirm-btn" @click="handleClose">
              <span>我知道了，继续工作</span>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

const props = defineProps<{
  show: boolean
  workDuration: number // 连续工作秒数
  tips: string[]
}>()

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void
  (e: 'close'): void
}>()

// 倒计时（60秒后自动关闭）
const countdown = ref(60)
let countdownTimer: number | null = null

// 随机选择一条提示
const currentTip = computed(() => {
  if (props.tips.length === 0) {
    return '起来活动一下，喝杯水吧！'
  }
  const index = Math.floor(Math.random() * props.tips.length)
  return props.tips[index]
})

// 格式化时长
function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (hours > 0) {
    return `${hours}小时${minutes}分钟`
  }
  return `${minutes}分钟`
}

// 关闭处理
async function handleClose() {
  // 取消窗口置顶
  try {
    const currentWindow = getCurrentWindow()
    await currentWindow.setAlwaysOnTop(false)
  } catch (e) {
    console.error('取消窗口置顶失败:', e)
  }
  emit('update:show', false)
  emit('close')
}

// 监听显示状态，启动/停止倒计时
watch(() => props.show, (newVal) => {
  if (newVal) {
    countdown.value = 60
    countdownTimer = window.setInterval(() => {
      countdown.value--
      if (countdown.value <= 0) {
        handleClose()
      }
    }, 1000)
  } else {
    if (countdownTimer) {
      clearInterval(countdownTimer)
      countdownTimer = null
    }
  }
})

onUnmounted(() => {
  if (countdownTimer) {
    clearInterval(countdownTimer)
  }
})
</script>

<style scoped>
/* 全屏遮罩 */
.sedentary-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

/* 弹框容器 */
.sedentary-modal {
  position: relative;
  width: 100%;
  max-width: 480px;
  padding: 48px;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 24px;
  box-shadow: var(--shadow-lg), 0 0 60px var(--accent-glow);
  text-align: center;
}

/* 关闭按钮 */
.close-btn {
  position: absolute;
  top: 16px;
  right: 16px;
  width: 40px;
  height: 40px;
  border: none;
  background: var(--bg-hover);
  border-radius: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  color: var(--text-secondary);
}

.close-btn:hover {
  background: var(--bg-active);
  color: var(--text-primary);
  transform: scale(1.05);
}

.close-btn svg {
  width: 20px;
  height: 20px;
}

/* 图标容器 */
.icon-container {
  position: relative;
  width: 120px;
  height: 120px;
  margin: 0 auto 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 呼吸光效 */
.icon-glow {
  position: absolute;
  width: 100%;
  height: 100%;
  background: var(--accent-glow);
  border-radius: 50%;
  animation: breathe 2s ease-in-out infinite;
}

@keyframes breathe {
  0%, 100% {
    transform: scale(1);
    opacity: 0.5;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.3;
  }
}

/* 休息图标 */
.rest-icon {
  width: 64px;
  height: 64px;
  color: var(--accent-primary);
  position: relative;
  z-index: 1;
}

/* 标题 */
.modal-title {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 12px;
  letter-spacing: -0.025em;
}

/* 工作时长 */
.work-duration {
  font-size: 16px;
  color: var(--text-secondary);
  margin: 0 0 24px;
}

/* 提示文案 */
.tip-text {
  font-size: 20px;
  color: var(--accent-primary);
  margin: 0 0 32px;
  line-height: 1.6;
  font-weight: 500;
}

/* 倒计时 */
.countdown {
  font-size: 14px;
  color: var(--text-muted);
  margin-bottom: 24px;
}

.countdown span {
  font-weight: 600;
  color: var(--accent-secondary);
}

/* 确认按钮 */
.confirm-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 14px 32px;
  background: var(--accent-primary);
  color: #ffffff;
  border: none;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 4px 12px var(--accent-glow);
}

.confirm-btn:hover {
  background: var(--accent-primary-hover);
  transform: translateY(-2px);
  box-shadow: 0 6px 20px var(--accent-glow);
}

/* 入场/出场动画 */
.sedentary-modal-enter-active,
.sedentary-modal-leave-active {
  transition: all 0.3s ease;
}

.sedentary-modal-enter-active .sedentary-modal,
.sedentary-modal-leave-active .sedentary-modal {
  transition: all 0.3s ease;
}

.sedentary-modal-enter-from,
.sedentary-modal-leave-to {
  opacity: 0;
}

.sedentary-modal-enter-from .sedentary-modal,
.sedentary-modal-leave-to .sedentary-modal {
  transform: scale(0.9);
  opacity: 0;
}
</style>
