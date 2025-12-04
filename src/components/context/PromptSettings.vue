<template>
  <div class="prompt-settings">
    <section class="settings-card">
      <div class="card-header">
        <h3 class="card-title">提示词配置</h3>
        <n-button size="small" @click="handleReset" :loading="resetting">
          恢复默认
        </n-button>
      </div>
      <div class="card-content">
        <!-- 加载状态 -->
        <div v-if="loading" class="loading-state">
          <n-spin size="small" />
          <span>加载中...</span>
        </div>

        <template v-else>
          <n-tabs type="segment" animated>
            <!-- 截图分析 -->
            <n-tab-pane name="screenshot" tab="截图分析">
              <div class="prompt-editor">
                <div class="editor-section">
                  <div class="editor-label">系统提示词</div>
                  <div class="editor-desc">定义AI的角色和基本行为规则</div>
                  <n-input
                    v-model:value="prompts.screenshot_analyze.system"
                    type="textarea"
                    :rows="10"
                    placeholder="请输入系统提示词..."
                  />
                </div>
                <div class="editor-section">
                  <div class="editor-label">用户提示词模板</div>
                  <div class="editor-desc">具体的分析指令，支持变量：{screenshot}</div>
                  <n-input
                    v-model:value="prompts.screenshot_analyze.user"
                    type="textarea"
                    :rows="4"
                    placeholder="请输入用户提示词模板..."
                  />
                </div>
              </div>
            </n-tab-pane>

            <!-- 智能合并 -->
            <n-tab-pane name="merging" tab="智能合并">
              <div class="prompt-editor">
                <div class="editor-section">
                  <div class="editor-label">系统提示词</div>
                  <div class="editor-desc">定义批量合并上下文的策略</div>
                  <n-input
                    v-model:value="prompts.batch_merging.system"
                    type="textarea"
                    :rows="10"
                    placeholder="请输入系统提示词..."
                  />
                </div>
                <div class="editor-section">
                  <div class="editor-label">用户提示词模板</div>
                  <div class="editor-desc">合并指令，支持变量：{contexts}</div>
                  <n-input
                    v-model:value="prompts.batch_merging.user"
                    type="textarea"
                    :rows="4"
                    placeholder="请输入用户提示词模板..."
                  />
                </div>
              </div>
            </n-tab-pane>

            <!-- 日报生成 -->
            <n-tab-pane name="daily" tab="日报生成">
              <div class="prompt-editor">
                <div class="editor-section">
                  <div class="editor-label">系统提示词</div>
                  <div class="editor-desc">定义日报生成的风格和结构</div>
                  <n-input
                    v-model:value="prompts.daily_report.system"
                    type="textarea"
                    :rows="10"
                    placeholder="请输入系统提示词..."
                  />
                </div>
                <div class="editor-section">
                  <div class="editor-label">用户提示词模板</div>
                  <div class="editor-desc">日报生成指令，支持变量：{date}, {activities}</div>
                  <n-input
                    v-model:value="prompts.daily_report.user"
                    type="textarea"
                    :rows="4"
                    placeholder="请输入用户提示词模板..."
                  />
                </div>
              </div>
            </n-tab-pane>

            <!-- 周报生成 -->
            <n-tab-pane name="weekly" tab="周报生成">
              <div class="prompt-editor">
                <div class="editor-section">
                  <div class="editor-label">系统提示词</div>
                  <div class="editor-desc">定义周报生成的风格和结构</div>
                  <n-input
                    v-model:value="prompts.weekly_report.system"
                    type="textarea"
                    :rows="10"
                    placeholder="请输入系统提示词..."
                  />
                </div>
                <div class="editor-section">
                  <div class="editor-label">用户提示词模板</div>
                  <div class="editor-desc">周报生成指令，支持变量：{week_range}, {daily_summaries}</div>
                  <n-input
                    v-model:value="prompts.weekly_report.user"
                    type="textarea"
                    :rows="4"
                    placeholder="请输入用户提示词模板..."
                  />
                </div>
              </div>
            </n-tab-pane>
          </n-tabs>

          <div class="action-buttons">
            <n-button type="primary" @click="handleSave" :loading="saving">
              保存配置
            </n-button>
          </div>
        </template>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { NButton, NTabs, NTabPane, NInput, NSpin, useMessage } from 'naive-ui';
import { promptApi } from '@/api/promptApi';
import type { PromptSettings } from '@/types/prompts';

const message = useMessage();

const loading = ref(true);
const saving = ref(false);
const resetting = ref(false);

const defaultPrompts: PromptSettings = {
  screenshot_analyze: {
    system: '',
    user: '',
  },
  batch_merging: {
    system: '',
    user: '',
  },
  daily_report: {
    system: '',
    user: '',
  },
  weekly_report: {
    system: '',
    user: '',
  },
};

const prompts = ref<PromptSettings>({ ...defaultPrompts });

onMounted(async () => {
  await loadConfig();
  loading.value = false;
});

async function loadConfig() {
  try {
    const config = await promptApi.getConfig();
    prompts.value = { ...config };
  } catch (error) {
    console.error('加载提示词配置失败:', error);
    message.error('加载配置失败');
  }
}

async function handleSave() {
  saving.value = true;
  try {
    await promptApi.saveConfig(prompts.value);
    message.success('提示词配置已保存');
  } catch (error: any) {
    message.error(error || '保存失败');
    console.error('保存提示词配置失败:', error);
  } finally {
    saving.value = false;
  }
}

async function handleReset() {
  resetting.value = true;
  try {
    const config = await promptApi.resetConfig();
    prompts.value = { ...config };
    message.success('已恢复为默认配置');
  } catch (error: any) {
    message.error(error || '重置失败');
    console.error('重置提示词配置失败:', error);
  } finally {
    resetting.value = false;
  }
}
</script>

<style scoped>
.prompt-settings {
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

.prompt-editor {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 20px 0;
}

.editor-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.editor-label {
  font-size: 14px;
  font-weight: 500;
  color: rgb(203, 213, 225);
}

.editor-desc {
  font-size: 12px;
  color: rgb(100, 116, 139);
  margin-bottom: 4px;
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
:deep(.n-tabs) {
  --n-tab-text-color: rgb(148, 163, 184);
  --n-tab-text-color-active: rgb(99, 102, 241);
  --n-tab-text-color-hover: rgb(203, 213, 225);
  --n-bar-color: rgb(99, 102, 241);
}

:deep(.n-tabs-tab-pad) {
  border-bottom: 1px solid rgba(51, 65, 85, 0.4);
}

:deep(.n-input) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-border-focus: 1px solid rgb(99, 102, 241);
  --n-color: rgb(2, 6, 23);
  --n-text-color: rgb(203, 213, 225);
  --n-caret-color: rgb(99, 102, 241);
  --n-placeholder-color: rgb(100, 116, 139);
}

:deep(.n-input__textarea-el) {
  font-family: 'Monaco', 'Menlo', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
}

:deep(.n-button--primary-type) {
  --n-color: rgb(99, 102, 241);
  --n-color-hover: rgb(79, 70, 229);
  --n-text-color: rgb(255, 255, 255);
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2);
}

:deep(.n-button) {
  --n-color: rgb(30, 41, 59);
  --n-color-hover: rgb(51, 65, 85);
  --n-text-color: rgb(203, 213, 225);
  --n-border: 1px solid rgb(51, 65, 85);
}
</style>
