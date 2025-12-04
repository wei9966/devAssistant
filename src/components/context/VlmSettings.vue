<template>
  <div class="vlm-settings">
    <section class="settings-card">
      <div class="card-header">
        <h3 class="card-title">VLM 服务配置</h3>
        <span class="status-badge" :class="{ 'status-active': vlmConfig.enabled }">
          {{ vlmConfig.enabled ? '已启用' : '未启用' }}
        </span>
      </div>
      <div class="card-content">
        <!-- 加载状态 -->
        <div v-if="loading" class="loading-state">
          <n-spin size="small" />
          <span>加载中...</span>
        </div>

        <template v-else>
          <!-- 启用 VLM -->
          <div class="setting-item">
            <div class="setting-info">
              <div class="setting-label">启用 VLM 服务</div>
              <div class="setting-desc">启用后可使用视觉模型分析屏幕截图</div>
            </div>
            <div class="setting-control">
              <n-switch v-model:value="vlmConfig.enabled" />
            </div>
          </div>

          <!-- VLM 提供商选择 -->
          <div class="setting-item">
            <div class="setting-info">
              <div class="setting-label">VLM 提供商</div>
              <div class="setting-desc">选择视觉语言模型服务提供商</div>
            </div>
            <div class="setting-control">
              <n-select
                v-model:value="vlmConfig.provider"
                :options="providerOptions"
                :disabled="!vlmConfig.enabled"
                style="min-width: 200px"
                @update:value="handleProviderChange"
              />
            </div>
          </div>

          <!-- API Key -->
          <div class="setting-item-vertical">
            <div class="setting-info">
              <div class="setting-label">API Key *</div>
              <div class="setting-desc">
                {{ getProviderApiKeyDesc() }}
              </div>
            </div>
            <n-input
              v-model:value="vlmConfig.apiKey"
              type="password"
              placeholder="请输入 API Key"
              show-password-on="click"
              :disabled="!vlmConfig.enabled"
            />
          </div>

          <!-- 自定义配置 -->
          <div class="custom-config">
            <div class="section-title">自定义配置（可选）</div>

            <!-- Base URL -->
            <div class="setting-item-vertical">
              <div class="setting-info">
                <div class="setting-label">Base URL</div>
                <div class="setting-desc">留空使用默认地址：{{ currentPreset?.defaultBaseUrl || '无' }}</div>
              </div>
              <n-input
                v-model:value="vlmConfig.baseUrl"
                :placeholder="currentPreset?.defaultBaseUrl || ''"
                :disabled="!vlmConfig.enabled"
              />
            </div>

            <!-- 模型名称 -->
            <div class="setting-item-vertical">
              <div class="setting-info">
                <div class="setting-label">模型名称</div>
                <div class="setting-desc">留空使用默认模型：{{ currentPreset?.defaultModel || '无' }}</div>
              </div>
              <n-input
                v-model:value="vlmConfig.model"
                :placeholder="currentPreset?.defaultModel || ''"
                :disabled="!vlmConfig.enabled"
              />
            </div>
          </div>

          <!-- 高级选项 -->
          <n-collapse>
            <n-collapse-item title="高级选项" name="advanced">
              <div class="advanced-settings">
                <!-- 最大图片尺寸 -->
                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">最大图片尺寸</div>
                    <div class="setting-desc">超过此大小的图片将被压缩</div>
                  </div>
                  <div class="setting-control">
                    <n-input-number
                      v-model:value="vlmConfig.maxImageSize"
                      :min="1024"
                      :max="20480"
                      :step="1024"
                      :disabled="!vlmConfig.enabled"
                      style="width: 150px"
                    >
                      <template #suffix>KB</template>
                    </n-input-number>
                  </div>
                </div>

                <!-- 图片压缩质量 -->
                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">图片压缩质量</div>
                    <div class="setting-desc">0-100，越高质量越好但体积越大</div>
                  </div>
                  <div class="slider-control">
                    <n-slider
                      v-model:value="vlmConfig.imageQuality"
                      :min="50"
                      :max="100"
                      :step="5"
                      :disabled="!vlmConfig.enabled"
                    />
                    <span class="slider-value">{{ vlmConfig.imageQuality }}</span>
                  </div>
                </div>

                <!-- 请求超时 -->
                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">请求超时</div>
                    <div class="setting-desc">VLM API 请求的超时时间</div>
                  </div>
                  <div class="setting-control">
                    <n-input-number
                      v-model:value="vlmConfig.timeout"
                      :min="10"
                      :max="120"
                      :step="5"
                      :disabled="!vlmConfig.enabled"
                      style="width: 120px"
                    >
                      <template #suffix>秒</template>
                    </n-input-number>
                  </div>
                </div>
              </div>
            </n-collapse-item>
          </n-collapse>

          <!-- 操作按钮 -->
          <div class="action-buttons">
            <n-button
              @click="handleTest"
              :loading="testing"
              :disabled="!vlmConfig.apiKey || !vlmConfig.enabled"
            >
              测试连接
            </n-button>
            <n-button
              type="primary"
              @click="handleSave"
              :loading="saving"
              :disabled="!vlmConfig.apiKey"
            >
              保存配置
            </n-button>
          </div>

          <!-- 测试结果 -->
          <n-alert
            v-if="testResult !== null"
            :type="testResult ? 'success' : 'error'"
            class="test-result"
          >
            {{ testResult ? 'VLM 连接测试成功！' : 'VLM 连接测试失败，请检查配置' }}
          </n-alert>
        </template>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  NSwitch,
  NSelect,
  NInput,
  NInputNumber,
  NSlider,
  NCollapse,
  NCollapseItem,
  NButton,
  NSpin,
  NAlert,
  useMessage,
} from 'naive-ui';
import { vlmApi } from '@/api/vlmApi';
import { VLM_PROVIDER_PRESETS } from '@/types/vlm';
import type { VlmConfig, VlmProvider } from '@/types/vlm';

const message = useMessage();

const loading = ref(true);
const saving = ref(false);
const testing = ref(false);
const testResult = ref<boolean | null>(null);

const defaultConfig: VlmConfig = {
  provider: 'qwen-vl',
  apiKey: '',
  baseUrl: '',
  model: '',
  enabled: false,
  maxImageSize: 5120,
  imageQuality: 80,
  timeout: 30,
};

const vlmConfig = ref<VlmConfig>({ ...defaultConfig });

const providerOptions = Object.entries(VLM_PROVIDER_PRESETS).map(([id, preset]) => ({
  label: preset.name,
  value: id as VlmProvider,
}));

const currentPreset = computed(() => {
  return VLM_PROVIDER_PRESETS[vlmConfig.value.provider];
});

onMounted(async () => {
  await loadConfig();
  loading.value = false;
});

async function loadConfig() {
  try {
    const config = await vlmApi.getConfig();
    if (config) {
      vlmConfig.value = {
        provider: config.provider,
        apiKey: '',  // API Key 需要用户重新输入
        baseUrl: config.baseUrl || '',
        model: config.model || '',
        enabled: config.enabled,
        maxImageSize: config.maxImageSize || 5120,
        imageQuality: config.imageQuality || 80,
        timeout: config.timeout || 30,
      };
    }
  } catch (error) {
    console.error('加载 VLM 配置失败:', error);
    message.error('加载配置失败');
  }
}

function handleProviderChange() {
  // 当切换提供商时，清空自定义配置
  vlmConfig.value.baseUrl = '';
  vlmConfig.value.model = '';
  // 更新最大图片尺寸为新提供商的默认值
  if (currentPreset.value) {
    vlmConfig.value.maxImageSize = currentPreset.value.maxImageSize;
  }
}

function getProviderApiKeyDesc(): string {
  const preset = currentPreset.value;
  if (!preset) return '';

  const descriptions: Record<string, string> = {
    'qwen-vl': '从 dashscope.console.aliyun.com 获取',
    'deepseek-vl': '从 platform.deepseek.com 获取',
    'openai': '从 platform.openai.com 获取',
    'doubao': '从 console.volcengine.com 获取',
    'kimi': '从 platform.moonshot.cn 获取',
    'claude': '从 console.anthropic.com 获取',
    'custom': '使用自定义 API 服务的密钥',
  };

  return descriptions[vlmConfig.value.provider] || '';
}

async function handleTest() {
  testing.value = true;
  testResult.value = null;
  try {
    // 先保存配置再测试
    await vlmApi.saveConfig(vlmConfig.value);
    const result = await vlmApi.testConnection();
    testResult.value = result;
    if (result) {
      message.success('VLM 连接测试成功');
    } else {
      message.error('VLM 连接测试失败');
    }
  } catch (error: any) {
    testResult.value = false;
    message.error(error || '连接测试失败');
  } finally {
    testing.value = false;
  }
}

async function handleSave() {
  saving.value = true;
  try {
    await vlmApi.saveConfig(vlmConfig.value);
    message.success('VLM 配置已保存');
  } catch (error: any) {
    message.error(error || '保存失败');
    console.error('保存 VLM 配置失败:', error);
  } finally {
    saving.value = false;
  }
}
</script>

<style scoped>
.vlm-settings {
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
  max-width: 300px;
}

.slider-value {
  font-size: 13px;
  font-weight: 500;
  color: rgb(203, 213, 225);
  min-width: 40px;
  text-align: right;
}

.custom-config {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
  background: rgba(30, 41, 59, 0.3);
  border-radius: 12px;
  border: 1px solid rgba(51, 65, 85, 0.4);
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  color: rgb(148, 163, 184);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.advanced-settings {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding-top: 12px;
}

.action-buttons {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid rgba(51, 65, 85, 0.4);
}

.test-result {
  margin-top: 8px;
}

/* Naive UI 样式覆盖 */
:deep(.n-switch) {
  --n-rail-color: rgb(51, 65, 85);
  --n-rail-color-active: rgb(99, 102, 241);
}

:deep(.n-select) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-border-focus: 1px solid rgb(99, 102, 241);
  --n-color: rgb(2, 6, 23);
  --n-text-color: rgb(203, 213, 225);
}

:deep(.n-input) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-border-focus: 1px solid rgb(99, 102, 241);
  --n-color: rgb(2, 6, 23);
  --n-text-color: rgb(203, 213, 225);
}

:deep(.n-input-number) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-border-focus: 1px solid rgb(99, 102, 241);
  --n-color: rgb(2, 6, 23);
  --n-text-color: rgb(203, 213, 225);
}

:deep(.n-slider) {
  --n-fill-color: rgb(99, 102, 241);
  --n-fill-color-hover: rgb(79, 70, 229);
  --n-handle-color: rgb(255, 255, 255);
  --n-rail-color: rgb(51, 65, 85);
}

:deep(.n-collapse) {
  --n-item-border-color: rgba(51, 65, 85, 0.4);
  --n-title-text-color: rgb(203, 213, 225);
}

:deep(.n-alert) {
  --n-border: 1px solid;
}

:deep(.n-button--primary-type) {
  --n-color: rgb(99, 102, 241);
  --n-color-hover: rgb(79, 70, 229);
  --n-text-color: rgb(255, 255, 255);
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2);
}
</style>
