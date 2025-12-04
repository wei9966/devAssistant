<template>
  <div class="context-settings">
    <section class="settings-card">
      <div class="card-header">
        <h3 class="card-title">屏幕上下文采集</h3>
        <span class="status-badge" :class="{ 'status-active': captureStatus?.isRunning }">
          {{ captureStatus?.isRunning ? '运行中' : '已停止' }}
        </span>
      </div>
      <div class="card-content">
        <!-- 加载状态 -->
        <div v-if="loading" class="loading-state">
          <n-spin size="small" />
          <span>加载中...</span>
        </div>

        <template v-else>
          <!-- 启用采集 -->
          <div class="setting-item">
            <div class="setting-info">
              <div class="setting-label">启用屏幕采集</div>
              <div class="setting-desc">自动采集屏幕截图并分析工作内容</div>
            </div>
            <div class="setting-control">
              <n-switch v-model:value="settings.captureEnabled" @update:value="handleToggleCapture" />
            </div>
          </div>

          <!-- 采集间隔 -->
          <div class="setting-item">
            <div class="setting-info">
              <div class="setting-label">采集间隔</div>
              <div class="setting-desc">截图采集的时间间隔（5-60秒）</div>
            </div>
            <div class="slider-control">
              <n-slider
                v-model:value="settings.captureInterval"
                :min="5"
                :max="60"
                :step="5"
                :marks="{ 5: '5s', 30: '30s', 60: '60s' }"
                :disabled="!settings.captureEnabled"
              />
              <span class="slider-value">{{ settings.captureInterval }}秒</span>
            </div>
          </div>

          <!-- 相似度阈值 -->
          <div class="setting-item">
            <div class="setting-info">
              <div class="setting-label">相似度阈值</div>
              <div class="setting-desc">跳过相似截图的阈值（0.8-1.0，越高越严格）</div>
            </div>
            <div class="slider-control">
              <n-slider
                v-model:value="settings.similarityThreshold"
                :min="0.8"
                :max="1.0"
                :step="0.01"
                :format-tooltip="(value: number) => value.toFixed(2)"
                :disabled="!settings.captureEnabled"
              />
              <span class="slider-value">{{ settings.similarityThreshold.toFixed(2) }}</span>
            </div>
          </div>

          <!-- 数据保留天数 -->
          <div class="setting-item">
            <div class="setting-info">
              <div class="setting-label">数据保留天数</div>
              <div class="setting-desc">自动清理超过指定天数的数据（1-30天）</div>
            </div>
            <div class="setting-control">
              <n-input-number
                v-model:value="settings.retentionDays"
                :min="1"
                :max="30"
                :disabled="!settings.captureEnabled"
                style="width: 120px"
              >
                <template #suffix>天</template>
              </n-input-number>
            </div>
          </div>

          <!-- 排除应用列表 -->
          <div class="setting-item-vertical">
            <div class="setting-info">
              <div class="setting-label">排除的应用</div>
              <div class="setting-desc">这些应用的窗口不会被采集（例如：密码管理器、隐私工具）</div>
            </div>
            <n-dynamic-tags
              v-model:value="settings.excludedApps"
              :disabled="!settings.captureEnabled"
              placeholder="输入应用名称后按回车添加"
            />
          </div>

          <!-- 保存原始截图 -->
          <div class="setting-item">
            <div class="setting-info">
              <div class="setting-label">保存原始截图</div>
              <div class="setting-desc">是否保存原始截图文件（占用存储空间）</div>
            </div>
            <div class="setting-control">
              <n-switch
                v-model:value="settings.saveScreenshots"
                :disabled="!settings.captureEnabled"
              />
            </div>
          </div>

          <!-- 截图保存目录 -->
          <div v-if="settings.saveScreenshots" class="setting-item-vertical">
            <div class="setting-info">
              <div class="setting-label">截图保存目录</div>
              <div class="setting-desc">
                截图文件保存的位置，留空使用默认目录
                <span v-if="defaultScreenshotDir" class="default-dir-hint">
                  （默认：{{ defaultScreenshotDir }}）
                </span>
              </div>
            </div>
            <div class="dir-input-wrapper">
              <n-input
                v-model:value="settings.screenshotDir"
                placeholder="留空使用默认目录"
                :disabled="!settings.captureEnabled"
                clearable
              />
              <n-button
                size="small"
                :disabled="!settings.captureEnabled"
                @click="handleSelectDir"
              >
                选择目录
              </n-button>
            </div>
          </div>

          <!-- 当前状态信息 -->
          <div v-if="captureStatus && settings.captureEnabled" class="status-info">
            <n-alert type="info" :bordered="false">
              <div class="status-details">
                <p v-if="captureStatus.lastCaptureAt">
                  <strong>最后采集:</strong> {{ formatTime(captureStatus.lastCaptureAt) }}
                </p>
                <p><strong>今日采集:</strong> {{ captureStatus.totalCapturesToday }} 次</p>
                <p><strong>今日跳过:</strong> {{ captureStatus.skippedCount }} 次（相似截图）</p>
              </div>
            </n-alert>
          </div>

          <!-- 操作按钮 -->
          <div class="action-buttons">
            <n-button @click="handleReset" :disabled="saving">重置</n-button>
            <n-button type="primary" @click="handleSave" :loading="saving">
              保存设置
            </n-button>
          </div>
        </template>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { NSwitch, NSlider, NInputNumber, NDynamicTags, NButton, NSpin, NAlert, NInput, useMessage } from 'naive-ui';
import { contextApi } from '@/api/contextApi';
import type { ContextSettings, CaptureStatus } from '@/types/context';
import { open } from '@tauri-apps/plugin-dialog';

const message = useMessage();

const loading = ref(true);
const saving = ref(false);
const captureStatus = ref<CaptureStatus | null>(null);
const defaultScreenshotDir = ref<string>('');

const defaultSettings: ContextSettings = {
  captureEnabled: false,
  captureInterval: 10,
  similarityThreshold: 0.95,
  retentionDays: 7,
  excludedApps: [],
  saveScreenshots: false,
  screenshotDir: null,
};

const settings = ref<ContextSettings>({ ...defaultSettings });
const originalSettings = ref<ContextSettings>({ ...defaultSettings });

onMounted(async () => {
  await loadSettings();
  await loadStatus();
  await loadDefaultDir();
  loading.value = false;

  // 定期更新状态
  setInterval(loadStatus, 5000);
});

async function loadDefaultDir() {
  try {
    defaultScreenshotDir.value = await contextApi.getDefaultScreenshotDir();
  } catch (error) {
    console.error('获取默认目录失败:', error);
  }
}

async function loadSettings() {
  try {
    const data = await contextApi.getSettings();
    settings.value = { ...data };
    originalSettings.value = { ...data };
  } catch (error) {
    console.error('加载设置失败:', error);
    message.error('加载设置失败');
  }
}

async function loadStatus() {
  try {
    captureStatus.value = await contextApi.getStatus();
  } catch (error) {
    console.error('加载状态失败:', error);
  }
}

async function handleToggleCapture(enabled: boolean) {
  try {
    if (enabled) {
      await contextApi.startCapture();
      message.success('屏幕采集已启动');
    } else {
      await contextApi.stopCapture();
      message.success('屏幕采集已停止');
    }
    await loadStatus();
  } catch (error: any) {
    message.error(error || '操作失败');
    settings.value.captureEnabled = !enabled;
  }
}

async function handleSave() {
  saving.value = true;
  try {
    await contextApi.updateSettings(settings.value);
    originalSettings.value = { ...settings.value };
    message.success('设置已保存');
  } catch (error: any) {
    message.error(error || '保存失败');
    console.error('保存设置失败:', error);
  } finally {
    saving.value = false;
  }
}

function handleReset() {
  settings.value = { ...originalSettings.value };
  message.info('已重置为上次保存的设置');
}

async function handleSelectDir() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择截图保存目录',
    });
    if (selected && typeof selected === 'string') {
      settings.value.screenshotDir = selected;
    }
  } catch (error) {
    console.error('选择目录失败:', error);
  }
}

function formatTime(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}
</script>

<style scoped>
.context-settings {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.settings-card {
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.6);
  border-radius: 16px;
  overflow: hidden;
  backdrop-filter: blur(8px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  border-bottom: 1px solid rgba(51, 65, 85, 0.6);
  background: rgba(15, 23, 42, 0.3);
}

.card-title {
  font-size: 14px;
  font-weight: 500;
  color: rgb(226, 232, 240);
  margin: 0;
}

.status-badge {
  font-size: 12px;
  font-weight: 700;
  padding: 4px 8px;
  border-radius: 4px;
  background: rgba(148, 163, 184, 0.1);
  color: rgb(148, 163, 184);
  border: 1px solid rgba(148, 163, 184, 0.2);
}

.status-active {
  background: rgba(16, 185, 129, 0.1);
  color: rgb(52, 211, 153);
  border-color: rgba(16, 185, 129, 0.2);
}

.card-content {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 32px;
  color: rgb(148, 163, 184);
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.setting-item-vertical {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.setting-info {
  flex: 1;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: rgb(203, 213, 225);
  margin-bottom: 4px;
}

.setting-desc {
  font-size: 12px;
  color: rgb(100, 116, 139);
  line-height: 1.4;
}

.setting-control {
  flex-shrink: 0;
}

.slider-control {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 16px;
  max-width: 400px;
}

.slider-value {
  font-size: 13px;
  font-weight: 500;
  color: rgb(203, 213, 225);
  min-width: 50px;
  text-align: right;
}

.status-info {
  margin-top: 8px;
}

.status-details {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.status-details p {
  margin: 0;
  font-size: 13px;
  color: rgb(148, 163, 184);
}

.status-details strong {
  color: rgb(203, 213, 225);
}

.default-dir-hint {
  color: rgb(100, 116, 139);
  font-size: 11px;
  display: block;
  margin-top: 4px;
  word-break: break-all;
}

.dir-input-wrapper {
  display: flex;
  gap: 8px;
  align-items: center;
}

.dir-input-wrapper :deep(.n-input) {
  flex: 1;
}

.action-buttons {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid rgba(51, 65, 85, 0.4);
}

/* Naive UI 样式覆盖 */
:deep(.n-switch) {
  --n-rail-color: rgb(51, 65, 85);
  --n-rail-color-active: rgb(99, 102, 241);
}

:deep(.n-slider) {
  --n-fill-color: rgb(99, 102, 241);
  --n-fill-color-hover: rgb(79, 70, 229);
  --n-handle-color: rgb(255, 255, 255);
  --n-rail-color: rgb(51, 65, 85);
}

:deep(.n-input-number) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-border-focus: 1px solid rgb(99, 102, 241);
  --n-color: rgb(2, 6, 23);
  --n-text-color: rgb(203, 213, 225);
}

:deep(.n-dynamic-tags) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
}

:deep(.n-alert) {
  --n-color: rgba(99, 102, 241, 0.1);
  --n-border: 1px solid rgba(99, 102, 241, 0.2);
}

:deep(.n-button--primary-type) {
  --n-color: rgb(99, 102, 241);
  --n-color-hover: rgb(79, 70, 229);
  --n-text-color: rgb(255, 255, 255);
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2);
}
</style>
