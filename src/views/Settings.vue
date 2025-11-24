<template>
  <div class="settings-page">
    <h2 class="page-title">设置</h2>

    <div class="settings-container">
      <n-tabs type="line" animated>
        <!-- 通用设置标签 -->
        <n-tab-pane name="general" tab="通用">
          <div class="tab-content">
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">通用设置</h3>
              </div>
              <div class="card-content">
                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">开机自启</div>
                    <div class="setting-desc">设置应用启动时的默认行为</div>
                  </div>
                  <div class="setting-control">
                    <n-switch v-model:value="settings.autoStart" />
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">主题偏好</div>
                    <div class="setting-desc">当前锁定为：Midnight Dark</div>
                  </div>
                  <div class="setting-control">
                    <n-select
                      v-model:value="settings.theme"
                      :options="[
                        { label: '跟随系统', value: 'auto' },
                        { label: '深色模式', value: 'dark' },
                        { label: '浅色模式', value: 'light' },
                      ]"
                      class="theme-select"
                    />
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">窗口置顶</div>
                    <div class="setting-desc">窗口始终显示在最前面</div>
                  </div>
                  <div class="setting-control">
                    <n-switch v-model:value="settings.alwaysOnTop" />
                  </div>
                </div>
              </div>
            </section>

            <!-- 剪贴板监听 -->
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">剪贴板监听</h3>
                <span class="status-badge">{{ settings.clipboardMonitor ? '运行中' : '已停止' }}</span>
              </div>
              <div class="card-content">
                <div class="setting-item">
                  <div class="setting-info-with-icon">
                    <div class="icon-wrapper">
                      <n-icon :component="TimeOutline" size="18" />
                    </div>
                    <div>
                      <div class="setting-label">监听间隔</div>
                      <div class="setting-desc">设置轮询剪贴板的频率（秒）</div>
                    </div>
                  </div>
                  <div class="number-adjuster">
                    <button class="adjuster-btn" @click="decreaseInterval">-</button>
                    <span class="adjuster-value">{{ settings.clipboardInterval }}</span>
                    <button class="adjuster-btn" @click="increaseInterval">+</button>
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">启用监听</div>
                    <div class="setting-desc">自动监控剪贴板变化</div>
                  </div>
                  <div class="setting-control">
                    <n-switch v-model:value="settings.clipboardMonitor" />
                  </div>
                </div>
              </div>
            </section>
          </div>
        </n-tab-pane>

        <!-- 快捷键设置标签 -->
        <n-tab-pane name="shortcuts" tab="快捷键">
          <div class="tab-content">
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">全局快捷键</h3>
                <n-button text @click="resetShortcuts" :disabled="loadingShortcuts">
                  <template #icon>
                    <n-icon :component="RefreshOutline" />
                  </template>
                  重置为默认
                </n-button>
              </div>
              <div class="card-content">
                <div v-if="loadingShortcuts" class="loading-state">
                  <n-spin size="small" />
                  <span>加载中...</span>
                </div>
                <template v-else>
                  <div class="setting-item">
                    <div class="setting-info">
                      <div class="setting-label">任务看板</div>
                      <div class="setting-desc">快速打开任务看板页面</div>
                    </div>
                    <div class="shortcut-input">
                      <n-input
                        v-model:value="shortcuts.taskBoard"
                        placeholder="如: Ctrl+Shift+N"
                        @keydown="handleShortcutKeyDown($event, 'taskBoard')"
                      />
                    </div>
                  </div>

                  <div class="setting-item">
                    <div class="setting-info">
                      <div class="setting-label">SQL历史</div>
                      <div class="setting-desc">快速打开SQL历史记录</div>
                    </div>
                    <div class="shortcut-input">
                      <n-input
                        v-model:value="shortcuts.sqlHistory"
                        placeholder="如: Ctrl+Shift+S"
                        @keydown="handleShortcutKeyDown($event, 'sqlHistory')"
                      />
                    </div>
                  </div>

                  <div class="setting-item">
                    <div class="setting-info">
                      <div class="setting-label">应用启动器</div>
                      <div class="setting-desc">快速打开应用启动器</div>
                    </div>
                    <div class="shortcut-input">
                      <n-input
                        v-model:value="shortcuts.appLauncher"
                        placeholder="如: Ctrl+Shift+Space"
                        @keydown="handleShortcutKeyDown($event, 'appLauncher')"
                      />
                    </div>
                  </div>

                  <div class="shortcut-tips">
                    <n-alert type="info" :bordered="false">
                      <template #icon>
                        <n-icon :component="InformationCircleOutline" />
                      </template>
                      <div>
                        <p><strong>提示：</strong></p>
                        <ul>
                          <li>点击输入框后直接按下快捷键组合</li>
                          <li>支持的修饰键：Ctrl、Shift、Alt</li>
                          <li>建议使用组合键以避免冲突</li>
                          <li>修改后需要点击保存按钮才能生效</li>
                        </ul>
                      </div>
                    </n-alert>
                  </div>
                </template>
              </div>
            </section>
          </div>
        </n-tab-pane>

        <!-- 集成设置标签 -->
        <n-tab-pane name="integrations" tab="集成">
          <div class="tab-content">
            <!-- Git集成 -->
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">Git 集成</h3>
              </div>
              <div class="card-content">
                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">启用 Git</div>
                    <div class="setting-desc">启用版本控制功能</div>
                  </div>
                  <div class="setting-control">
                    <n-switch v-model:value="settings.gitEnabled" />
                  </div>
                </div>

                <div v-if="settings.gitEnabled" class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">仓库路径</div>
                    <div class="setting-desc">Git 仓库的本地路径</div>
                  </div>
                  <div class="setting-control-wide">
                    <n-input
                      v-model:value="settings.gitPath"
                      placeholder="请输入 Git 仓库路径"
                    />
                  </div>
                </div>
              </div>
            </section>

            <!-- AI集成 -->
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">AI 集成</h3>
              </div>
              <div class="card-content">
                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">启用 AI</div>
                    <div class="setting-desc">使用 Claude AI 辅助功能</div>
                  </div>
                  <div class="setting-control">
                    <n-switch v-model:value="settings.aiEnabled" />
                  </div>
                </div>

                <div v-if="settings.aiEnabled" class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">Claude API Key</div>
                    <div class="setting-desc">从 Anthropic 获取的 API 密钥</div>
                  </div>
                  <div class="setting-control-wide">
                    <n-input
                      v-model:value="settings.claudeApiKey"
                      type="password"
                      placeholder="请输入 Claude API Key"
                      show-password-on="click"
                    />
                  </div>
                </div>
              </div>
            </section>
          </div>
        </n-tab-pane>

        <!-- 通知设置标签 -->
        <n-tab-pane name="notifications" tab="通知">
          <div class="tab-content">
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">通知设置</h3>
              </div>
              <div class="card-content">
                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">任务提醒</div>
                    <div class="setting-desc">自动提醒待办任务</div>
                  </div>
                  <div class="setting-control">
                    <n-switch v-model:value="settings.taskNotification" />
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">僵尸任务天数</div>
                    <div class="setting-desc">多少天未完成算作僵尸任务</div>
                  </div>
                  <div class="number-adjuster">
                    <button class="adjuster-btn" @click="decreaseStaleDays">-</button>
                    <span class="adjuster-value">{{ settings.staleTaskDays }}</span>
                    <button class="adjuster-btn" @click="increaseStaleDays">+</button>
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">日志生成提醒</div>
                    <div class="setting-desc">每日提醒生成工作日志</div>
                  </div>
                  <div class="setting-control">
                    <n-switch v-model:value="settings.worklogReminder" />
                  </div>
                </div>
              </div>
            </section>
          </div>
        </n-tab-pane>

        <!-- 数据管理标签 -->
        <n-tab-pane name="data" tab="数据">
          <div class="tab-content">
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">数据管理</h3>
              </div>
              <div class="card-content">
                <div class="data-path-info">
                  <div class="setting-label">数据库位置</div>
                  <code class="db-path">{{ dataPath }}</code>
                </div>
                <n-space class="action-buttons">
                  <n-button @click="openDataFolder">打开数据文件夹</n-button>
                  <n-button @click="handleExport">导出数据</n-button>
                  <n-button type="error" @click="handleClearData">清空数据</n-button>
                </n-space>
              </div>
            </section>
          </div>
        </n-tab-pane>

        <!-- 关于标签 -->
        <n-tab-pane name="about" tab="关于">
          <div class="tab-content">
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">关于</h3>
              </div>
              <div class="card-content">
                <div class="about-info">
                  <p><strong>应用名称:</strong> DevAssistant</p>
                  <p><strong>版本:</strong> 0.1.0</p>
                  <p><strong>技术栈:</strong> Tauri + Rust + Vue3 + TypeScript + Naive UI</p>
                </div>
              </div>
            </section>
          </div>
        </n-tab-pane>
      </n-tabs>

      <!-- 保存按钮 -->
      <div class="save-section">
        <n-button type="primary" @click="handleSave" :loading="saving" size="large">
          保存设置
        </n-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { NTabs, NTabPane, NSpace, NSwitch, NSelect, NInput, NButton, NIcon, NSpin, NAlert, useMessage, useDialog } from 'naive-ui';
import { TimeOutline, RefreshOutline, InformationCircleOutline } from '@vicons/ionicons5';
import { invoke } from '@tauri-apps/api/core';

const message = useMessage();
const dialog = useDialog();

const saving = ref(false);
const loadingShortcuts = ref(false);
const dataPath = ref('~/.dev-assistant/db.sqlite');

const settings = ref({
  theme: 'auto',
  alwaysOnTop: false,
  autoStart: false,
  clipboardMonitor: true,
  clipboardInterval: 2,
  gitEnabled: true,
  gitPath: '',
  aiEnabled: false,
  claudeApiKey: '',
  taskNotification: true,
  staleTaskDays: 3,
  worklogReminder: true,
});

const shortcuts = ref({
  taskBoard: 'Ctrl+Shift+N',
  sqlHistory: 'Ctrl+Shift+S',
  appLauncher: 'Ctrl+Shift+Space',
});

onMounted(() => {
  loadSettings();
  loadShortcuts();
});

function loadSettings() {
  // TODO: 从API加载设置
}

async function loadShortcuts() {
  loadingShortcuts.value = true;
  try {
    const config = await invoke<{
      task_board: string;
      sql_history: string;
      app_launcher: string;
    }>('get_shortcut_config');

    shortcuts.value = {
      taskBoard: config.task_board,
      sqlHistory: config.sql_history,
      appLauncher: config.app_launcher,
    };
  } catch (error) {
    console.error('加载快捷键配置失败:', error);
    message.error('加载快捷键配置失败');
  } finally {
    loadingShortcuts.value = false;
  }
}

async function saveShortcuts() {
  try {
    await invoke('update_shortcut_config', {
      config: {
        task_board: shortcuts.value.taskBoard,
        sql_history: shortcuts.value.sqlHistory,
        app_launcher: shortcuts.value.appLauncher,
      },
    });
    message.success('快捷键已更新');
  } catch (error: any) {
    message.error(error || '更新快捷键失败');
    console.error('更新快捷键失败:', error);
    throw error;
  }
}

async function resetShortcuts() {
  dialog.warning({
    title: '重置快捷键',
    content: '确定要重置所有快捷键为默认值吗?',
    positiveText: '重置',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await invoke('reset_shortcut_config');
        await loadShortcuts();
        message.success('快捷键已重置');
      } catch (error) {
        message.error('重置快捷键失败');
        console.error('重置快捷键失败:', error);
      }
    },
  });
}

function handleShortcutKeyDown(event: KeyboardEvent, field: 'taskBoard' | 'sqlHistory' | 'appLauncher') {
  event.preventDefault();

  const modifiers: string[] = [];
  if (event.ctrlKey) modifiers.push('Ctrl');
  if (event.shiftKey) modifiers.push('Shift');
  if (event.altKey) modifiers.push('Alt');

  let key = event.key;

  // 处理特殊键
  if (key === ' ') key = 'Space';
  else if (key.length === 1) key = key.toUpperCase();
  else if (key.startsWith('F') && key.length <= 3) key = key; // F1-F12
  else return; // 忽略其他特殊键

  if (modifiers.length === 0) {
    message.warning('请使用组合键（需要包含 Ctrl、Shift 或 Alt）');
    return;
  }

  const shortcut = [...modifiers, key].join('+');
  shortcuts.value[field] = shortcut;
}

async function handleSave() {
  saving.value = true;
  try {
    // 保存快捷键
    await saveShortcuts();

    // TODO: 保存其他设置

    message.success('设置已保存');
  } catch (error) {
    message.error('保存失败');
    console.error(error);
  } finally {
    saving.value = false;
  }
}

function openDataFolder() {
  // TODO: 调用Tauri API打开文件夹
  message.info('打开数据文件夹');
}

function handleExport() {
  // TODO: 导出数据
  message.info('导出数据功能开发中');
}

function handleClearData() {
  dialog.warning({
    title: '清空数据',
    content: '确定要清空所有数据吗？此操作不可撤销！',
    positiveText: '清空',
    negativeText: '取消',
    onPositiveClick: () => {
      // TODO: 调用API清空数据
      message.success('数据已清空');
    },
  });
}

// 数字调节器方法
function decreaseInterval() {
  if (settings.value.clipboardInterval > 1) {
    settings.value.clipboardInterval--;
  }
}

function increaseInterval() {
  if (settings.value.clipboardInterval < 60) {
    settings.value.clipboardInterval++;
  }
}

function decreaseStaleDays() {
  if (settings.value.staleTaskDays > 1) {
    settings.value.staleTaskDays--;
  }
}

function increaseStaleDays() {
  if (settings.value.staleTaskDays < 30) {
    settings.value.staleTaskDays++;
  }
}
</script>

<style scoped>
.settings-page {
  padding: 32px;
  padding-top: 16px;
  height: 100%;
  overflow-y: auto;
}

.page-title {
  font-size: 20px;
  font-weight: 600;
  color: rgb(241, 245, 249);
  letter-spacing: -0.025em;
  margin-bottom: 24px;
}

.settings-container {
  max-width: 800px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.tab-content {
  padding-top: 24px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* 设置卡片 */
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
  background: rgba(16, 185, 129, 0.1);
  color: rgb(52, 211, 153);
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.card-content {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* 设置项 */
.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.setting-info {
  flex: 1;
}

.setting-info-with-icon {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  background: rgb(30, 41, 59);
  color: rgb(148, 163, 184);
  border-radius: 8px;
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

.setting-control-wide {
  flex: 1;
  max-width: 400px;
}

/* 快捷键输入框 */
.shortcut-input {
  min-width: 220px;
}

.shortcut-tips {
  margin-top: 8px;
}

.shortcut-tips ul {
  margin: 8px 0 0 0;
  padding-left: 20px;
}

.shortcut-tips li {
  font-size: 13px;
  line-height: 1.6;
  color: rgb(148, 163, 184);
}

.shortcut-tips p {
  margin: 0 0 4px 0;
}

/* 加载状态 */
.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 32px;
  color: rgb(148, 163, 184);
}

/* 主题选择器 */
.theme-select {
  min-width: 140px;
}

/* 数字调节器 */
.number-adjuster {
  display: flex;
  align-items: center;
  background: rgb(2, 6, 23);
  border: 1px solid rgb(51, 65, 85);
  border-radius: 8px;
  padding: 4px;
  gap: 4px;
}

.adjuster-btn {
  padding: 8px 12px;
  background: transparent;
  border: none;
  color: rgb(148, 163, 184);
  font-size: 14px;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
  font-weight: 500;
}

.adjuster-btn:hover {
  background: rgb(30, 41, 59);
  color: rgb(255, 255, 255);
}

.adjuster-btn:active {
  transform: scale(0.95);
}

.adjuster-value {
  width: 48px;
  text-align: center;
  color: rgb(226, 232, 240);
  font-size: 14px;
  font-family: 'Monaco', 'Menlo', monospace;
  font-weight: 500;
}

/* 数据管理 */
.data-path-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.db-path {
  background: rgb(2, 6, 23);
  padding: 8px 12px;
  border-radius: 6px;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 12px;
  color: rgb(148, 163, 184);
  border: 1px solid rgba(51, 65, 85, 0.5);
  display: inline-block;
  word-break: break-all;
}

.action-buttons {
  margin-top: 8px;
}

/* 关于信息 */
.about-info {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.about-info p {
  margin: 0;
  font-size: 14px;
  color: rgb(203, 213, 225);
  line-height: 1.5;
}

.about-info strong {
  color: rgb(226, 232, 240);
  font-weight: 600;
}

/* 保存按钮区域 */
.save-section {
  display: flex;
  justify-content: flex-end;
  padding-top: 8px;
}

/* 滚动条样式 */
.settings-page::-webkit-scrollbar {
  width: 8px;
}

.settings-page::-webkit-scrollbar-track {
  background: transparent;
}

.settings-page::-webkit-scrollbar-thumb {
  background: rgba(51, 65, 85, 0.5);
  border-radius: 4px;
}

.settings-page::-webkit-scrollbar-thumb:hover {
  background: rgba(71, 85, 105, 0.7);
}

/* Naive UI 组件自定义样式 */
:deep(.n-tabs) {
  --n-tab-text-color: rgb(148, 163, 184);
  --n-tab-text-color-active: rgb(99, 102, 241);
  --n-tab-text-color-hover: rgb(203, 213, 225);
  --n-bar-color: rgb(99, 102, 241);
  --n-tab-border-color: rgba(51, 65, 85, 0.6);
}

:deep(.n-switch) {
  --n-rail-color: rgb(51, 65, 85);
  --n-rail-color-active: rgb(99, 102, 241);
  --n-button-color: rgb(148, 163, 184);
  --n-button-color-active: rgb(255, 255, 255);
}

:deep(.n-select) {
  --n-border: 1px solid rgb(51, 65, 85);
  --n-border-hover: 1px solid rgb(99, 102, 241);
  --n-border-focus: 1px solid rgb(99, 102, 241);
  --n-color: rgb(2, 6, 23);
  --n-text-color: rgb(203, 213, 225);
  --n-caret-color: rgb(99, 102, 241);
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

:deep(.n-alert) {
  --n-color: rgba(99, 102, 241, 0.1);
  --n-title-text-color: rgb(203, 213, 225);
  --n-content-text-color: rgb(148, 163, 184);
  --n-icon-color: rgb(99, 102, 241);
  --n-border: 1px solid rgba(99, 102, 241, 0.2);
}

:deep(.n-button) {
  --n-color: rgb(30, 41, 59);
  --n-color-hover: rgb(51, 65, 85);
  --n-color-pressed: rgb(30, 41, 59);
  --n-text-color: rgb(203, 213, 225);
  --n-border: 1px solid rgb(51, 65, 85);
}

:deep(.n-button--primary-type) {
  --n-color: rgb(99, 102, 241);
  --n-color-hover: rgb(79, 70, 229);
  --n-color-pressed: rgb(99, 102, 241);
  --n-text-color: rgb(255, 255, 255);
  --n-border: 1px solid rgb(99, 102, 241);
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2);
}

:deep(.n-button--error-type) {
  --n-color: rgba(239, 68, 68, 0.1);
  --n-color-hover: rgba(239, 68, 68, 0.2);
  --n-text-color: rgb(248, 113, 113);
  --n-border: 1px solid rgba(239, 68, 68, 0.3);
}
</style>
