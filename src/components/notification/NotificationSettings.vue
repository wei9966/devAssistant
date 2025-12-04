<template>
  <n-modal
    v-model:show="showModal"
    preset="card"
    title="通知设置"
    class="notification-settings-modal"
    :style="{ width: '600px', maxWidth: '90vw' }"
    :bordered="false"
    :segmented="{ content: true, footer: true }"
  >
    <div class="settings-content">
      <!-- Tips 设置 -->
      <section class="settings-section">
        <div class="section-header">
          <h4 class="section-title">💡 工作提示</h4>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">启用工作提示</div>
            <div class="setting-desc">定期推送工作提醒和健康建议</div>
          </div>
          <n-switch v-model:value="settings.tipsEnabled" />
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">提示间隔</div>
            <div class="setting-desc">每隔多少分钟推送一次提示</div>
          </div>
          <n-input-number
            v-model:value="settings.tipsIntervalMinutes"
            :min="15"
            :max="180"
            :step="15"
            :disabled="!settings.tipsEnabled"
            style="width: 120px"
          >
            <template #suffix>分钟</template>
          </n-input-number>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">每日最大次数</div>
            <div class="setting-desc">每天最多接收多少条提示</div>
          </div>
          <n-input-number
            v-model:value="settings.tipsMaxPerDay"
            :min="1"
            :max="20"
            :disabled="!settings.tipsEnabled"
            style="width: 120px"
          >
            <template #suffix>次</template>
          </n-input-number>
        </div>
      </section>

      <!-- 日报设置 -->
      <section class="settings-section">
        <div class="section-header">
          <h4 class="section-title">📊 每日工作报告</h4>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">启用日报</div>
            <div class="setting-desc">每天自动生成工作日报</div>
          </div>
          <n-switch v-model:value="settings.dailyReportEnabled" />
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">生成时间</div>
            <div class="setting-desc">每天几点生成日报</div>
          </div>
          <n-time-picker
            v-model:formatted-value="settings.dailyReportTime"
            format="HH:mm"
            :disabled="!settings.dailyReportEnabled"
            style="width: 120px"
          />
        </div>
      </section>

      <!-- 周报设置 -->
      <section class="settings-section">
        <div class="section-header">
          <h4 class="section-title">🌟 每周工作总结</h4>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">启用周报</div>
            <div class="setting-desc">每周自动生成工作总结</div>
          </div>
          <n-switch v-model:value="settings.weeklyReportEnabled" />
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">生成日期</div>
            <div class="setting-desc">每周几生成周报</div>
          </div>
          <n-select
            v-model:value="settings.weeklyReportDay"
            :options="weekDayOptions"
            :disabled="!settings.weeklyReportEnabled"
            style="width: 120px"
          />
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <div class="setting-label">生成时间</div>
            <div class="setting-desc">几点生成周报</div>
          </div>
          <n-time-picker
            v-model:formatted-value="settings.weeklyReportTime"
            format="HH:mm"
            :disabled="!settings.weeklyReportEnabled"
            style="width: 120px"
          />
        </div>
      </section>
    </div>

    <template #footer>
      <div class="modal-footer">
        <n-button @click="handleCancel">取消</n-button>
        <n-button type="primary" @click="handleSave" :loading="saving">
          保存设置
        </n-button>
      </div>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  NModal,
  NSwitch,
  NInputNumber,
  NTimePicker,
  NSelect,
  NButton,
  useMessage
} from 'naive-ui'
import { notificationApi } from '@/api/notificationApi'
import type { NotificationSettings } from '@/types/notification'

interface Props {
  show: boolean
}

interface Emits {
  (e: 'update:show', value: boolean): void
  (e: 'saved'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const message = useMessage()
const showModal = ref(false)
const saving = ref(false)

const defaultSettings: NotificationSettings = {
  tipsEnabled: true,
  tipsIntervalMinutes: 30,
  tipsMaxPerDay: 10,
  dailyReportEnabled: true,
  dailyReportTime: '18:00',
  weeklyReportEnabled: true,
  weeklyReportDay: 5,
  weeklyReportTime: '18:00'
}

const settings = ref<NotificationSettings>({ ...defaultSettings })
const originalSettings = ref<NotificationSettings>({ ...defaultSettings })

const weekDayOptions = [
  { label: '周日', value: 0 },
  { label: '周一', value: 1 },
  { label: '周二', value: 2 },
  { label: '周三', value: 3 },
  { label: '周四', value: 4 },
  { label: '周五', value: 5 },
  { label: '周六', value: 6 }
]

watch(() => props.show, async (newVal) => {
  showModal.value = newVal
  if (newVal) {
    await loadSettings()
  }
})

watch(showModal, (newVal) => {
  if (!newVal) {
    emit('update:show', false)
  }
})

async function loadSettings() {
  try {
    const data = await notificationApi.getSettings()
    settings.value = { ...data }
    originalSettings.value = { ...data }
  } catch (error) {
    console.error('加载设置失败:', error)
    message.error('加载设置失败')
  }
}

async function handleSave() {
  saving.value = true
  try {
    await notificationApi.updateSettings(settings.value)
    originalSettings.value = { ...settings.value }
    message.success('设置已保存')
    emit('saved')
    showModal.value = false
  } catch (error) {
    console.error('保存设置失败:', error)
    message.error('保存设置失败')
  } finally {
    saving.value = false
  }
}

function handleCancel() {
  settings.value = { ...originalSettings.value }
  showModal.value = false
}
</script>

<style scoped>
.settings-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 8px 0;
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-header {
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(51, 65, 85, 0.6);
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: rgb(226, 232, 240);
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-label {
  font-size: 13px;
  font-weight: 500;
  color: rgb(203, 213, 225);
  margin-bottom: 4px;
}

.setting-desc {
  font-size: 12px;
  color: rgb(100, 116, 139);
  line-height: 1.4;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
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

:deep(.n-card__content) {
  padding: 24px;
}

:deep(.n-card__footer) {
  border-top: 1px solid rgba(51, 65, 85, 0.6);
  padding: 16px 24px;
}

:deep(.n-switch) {
  --n-rail-color: rgb(51, 65, 85);
  --n-rail-color-active: rgb(99, 102, 241);
}

:deep(.n-input-number) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-border-focus: 1px solid rgb(99, 102, 241);
  --n-color: rgb(15, 23, 42);
  --n-text-color: rgb(203, 213, 225);
}

:deep(.n-time-picker) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-border-focus: 1px solid rgb(99, 102, 241);
  --n-color: rgb(15, 23, 42);
  --n-text-color: rgb(203, 213, 225);
}

:deep(.n-select) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-border-focus: 1px solid rgb(99, 102, 241);
  --n-color: rgb(15, 23, 42);
  --n-text-color: rgb(203, 213, 225);
}

:deep(.n-button--primary-type) {
  --n-color: rgb(99, 102, 241);
  --n-color-hover: rgb(79, 70, 229);
  --n-text-color: rgb(255, 255, 255);
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2);
}
</style>
