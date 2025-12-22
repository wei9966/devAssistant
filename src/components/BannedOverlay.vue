<template>
  <div v-if="visible" class="banned-overlay">
    <div class="banned-dialog">
      <div class="banned-icon">
        <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
          <path d="M15 9L9 15M9 9L15 15" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </div>

      <h2 class="banned-title">设备已被限制使用</h2>

      <div class="banned-reason">
        <span class="reason-label">原因：</span>
        <span class="reason-text">{{ reason }}</span>
      </div>

      <p class="banned-hint">
        如有疑问，您可以提交申诉，我们会尽快处理。
      </p>

      <!-- 申诉状态提示 -->
      <div v-if="hasPendingAppeal" class="pending-appeal-notice">
        <svg viewBox="0 0 24 24" fill="none" class="notice-icon">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
          <path d="M12 8v4M12 16h.01" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
        <span>您已有待处理的申诉，请耐心等待</span>
      </div>

      <!-- 申诉表单 -->
      <div v-if="!hasPendingAppeal && showAppealForm" class="appeal-form">
        <div class="form-group">
          <label>联系方式（邮箱/电话）</label>
          <input
            v-model="contact"
            type="text"
            placeholder="请输入您的联系方式"
            :disabled="submitting"
          />
        </div>
        <div class="form-group">
          <label>申诉理由</label>
          <textarea
            v-model="appealReason"
            placeholder="请详细说明您的申诉理由（至少10个字）"
            rows="4"
            :disabled="submitting"
          ></textarea>
        </div>
        <div class="form-actions">
          <button class="btn btn-secondary" @click="showAppealForm = false" :disabled="submitting">
            取消
          </button>
          <button class="btn btn-primary" @click="submitAppeal" :disabled="submitting || !isFormValid">
            {{ submitting ? '提交中...' : '提交申诉' }}
          </button>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div v-if="!showAppealForm" class="banned-actions">
        <button v-if="!hasPendingAppeal" class="btn btn-primary" @click="showAppealForm = true">
          提交申诉
        </button>
        <button class="btn btn-secondary" @click="checkAppealStatus">
          查看申诉状态
        </button>
      </div>

      <!-- 申诉记录 -->
      <div v-if="appealHistory.length > 0" class="appeal-history">
        <h3>申诉记录</h3>
        <div v-for="appeal in appealHistory" :key="appeal.id" class="appeal-item">
          <div class="appeal-header">
            <span class="appeal-time">{{ appeal.created_at }}</span>
            <span :class="['appeal-status', `status-${appeal.status}`]">
              {{ appeal.status_text }}
            </span>
          </div>
          <div v-if="appeal.admin_reply" class="appeal-reply">
            <span class="reply-label">回复：</span>
            {{ appeal.admin_reply }}
          </div>
        </div>
      </div>

      <div class="banned-footer">
        <span class="footer-text">部分功能已被限制，解除限制后可正常使用</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { deviceTracker, type AppealStatus } from '@/services/deviceTracker'

const props = defineProps<{
  visible: boolean
  reason: string
  hasPendingAppeal?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'appeal-submitted'): void
}>()

const showAppealForm = ref(false)
const contact = ref('')
const appealReason = ref('')
const submitting = ref(false)
const appealHistory = ref<AppealStatus[]>([])

const isFormValid = computed(() => {
  return contact.value.length >= 5 && appealReason.value.length >= 10
})

const submitAppeal = async () => {
  if (!isFormValid.value || submitting.value) return

  submitting.value = true

  try {
    const result = await deviceTracker.submitAppeal(contact.value, appealReason.value)

    if (result.success) {
      alert('申诉提交成功，我们会尽快处理')
      showAppealForm.value = false
      contact.value = ''
      appealReason.value = ''
      emit('appeal-submitted')
      // 刷新申诉状态
      await checkAppealStatus()
    } else {
      alert(result.message)
    }
  } catch (error) {
    alert('提交失败，请稍后重试')
  } finally {
    submitting.value = false
  }
}

const checkAppealStatus = async () => {
  try {
    appealHistory.value = await deviceTracker.getAppealStatus()
  } catch (error) {
    console.error('获取申诉状态失败:', error)
  }
}
</script>

<style scoped>
.banned-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(8px);
}

.banned-dialog {
  background: linear-gradient(145deg, #1e1e2e 0%, #2d2d44 100%);
  border-radius: 16px;
  padding: 40px;
  max-width: 480px;
  width: 90%;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.banned-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto 20px;
  color: #ef4444;
}

.banned-icon svg {
  width: 100%;
  height: 100%;
}

.banned-title {
  text-align: center;
  font-size: 24px;
  font-weight: 600;
  color: #ffffff;
  margin-bottom: 20px;
}

.banned-reason {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 8px;
  padding: 12px 16px;
  margin-bottom: 16px;
}

.reason-label {
  color: #ef4444;
  font-weight: 500;
}

.reason-text {
  color: #fca5a5;
}

.banned-hint {
  color: #a1a1aa;
  font-size: 14px;
  text-align: center;
  margin-bottom: 24px;
}

.pending-appeal-notice {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(251, 191, 36, 0.1);
  border: 1px solid rgba(251, 191, 36, 0.3);
  border-radius: 8px;
  padding: 12px 16px;
  margin-bottom: 20px;
  color: #fbbf24;
  font-size: 14px;
}

.notice-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.appeal-form {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  color: #d4d4d8;
  font-size: 14px;
  margin-bottom: 8px;
}

.form-group input,
.form-group textarea {
  width: 100%;
  padding: 12px;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: #ffffff;
  font-size: 14px;
  resize: vertical;
}

.form-group input:focus,
.form-group textarea:focus {
  outline: none;
  border-color: #6366f1;
}

.form-group input::placeholder,
.form-group textarea::placeholder {
  color: #71717a;
}

.form-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.banned-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
  margin-bottom: 20px;
}

.btn {
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  color: #ffffff;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(99, 102, 241, 0.3);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.1);
  color: #d4d4d8;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.btn-secondary:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.15);
}

.appeal-history {
  margin-top: 24px;
  padding-top: 24px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.appeal-history h3 {
  color: #d4d4d8;
  font-size: 16px;
  margin-bottom: 16px;
}

.appeal-item {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 12px;
}

.appeal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.appeal-time {
  color: #71717a;
  font-size: 12px;
}

.appeal-status {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.status-0 {
  background: rgba(251, 191, 36, 0.2);
  color: #fbbf24;
}

.status-1 {
  background: rgba(34, 197, 94, 0.2);
  color: #22c55e;
}

.status-2 {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.appeal-reply {
  color: #a1a1aa;
  font-size: 13px;
}

.reply-label {
  color: #6366f1;
}

.banned-footer {
  text-align: center;
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.footer-text {
  color: #71717a;
  font-size: 12px;
}
</style>
