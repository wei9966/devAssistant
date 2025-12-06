<template>
  <div class="settings-page">
    <h2 class="page-title">设置</h2>

    <div class="settings-container">
      <n-tabs v-model:value="activeTab" type="line" animated>
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

                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">日志级别</div>
                    <div class="setting-desc">设置应用日志的详细程度，调试时可设为 Debug</div>
                  </div>
                  <div class="setting-control">
                    <n-select
                      v-model:value="settings.logLevel"
                      :options="logLevelOptions"
                      class="log-level-select"
                    />
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

                  <div class="setting-item">
                    <div class="setting-info">
                      <div class="setting-label">快速任务</div>
                      <div class="setting-desc">快速创建新任务</div>
                    </div>
                    <div class="shortcut-input">
                      <n-input
                        v-model:value="shortcuts.quickTask"
                        placeholder="如: Ctrl+Shift+T"
                        @keydown="handleShortcutKeyDown($event, 'quickTask')"
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

            <!-- AI 智能助手配置 -->
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">AI 智能助手</h3>
                <span class="status-badge" :class="{ 'status-active': aiConfig.enabled }">
                  {{ aiConfig.enabled ? '已启用' : '未启用' }}
                </span>
              </div>
              <div class="card-content">
                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">启用 AI 功能</div>
                    <div class="setting-desc">启用后可使用 AI 辅助分类、生成等功能</div>
                  </div>
                  <div class="setting-control">
                    <n-switch v-model:value="aiConfig.enabled" />
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">AI 提供商</div>
                    <div class="setting-desc">选择 AI 服务提供商</div>
                  </div>
                  <div class="setting-control">
                    <n-select
                      v-model:value="aiConfig.provider"
                      :options="aiProviderOptions"
                      class="provider-select"
                    />
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">API Key *</div>
                    <div class="setting-desc">
                      {{ aiConfig.provider === 'deepseek'
                        ? '从 platform.deepseek.com 获取'
                        : '从 dashscope.console.aliyun.com 获取'
                      }}
                    </div>
                  </div>
                  <div class="setting-control-wide">
                    <n-input
                      v-model:value="aiConfig.apiKey"
                      type="password"
                      placeholder="请输入 API Key"
                      show-password-on="click"
                    />
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">
                      {{ aiConfig.provider === 'custom' ? 'Base URL（必填）' : '自定义 Base URL（可选）' }}
                    </div>
                    <div class="setting-desc">
                      {{ aiConfig.provider === 'custom' ? '请输入 OpenAI 兼容的 API 地址，如 https://apis.iflow.cn' : '留空使用默认地址' }}
                    </div>
                  </div>
                  <div class="setting-control-wide">
                    <n-input
                      v-model:value="aiConfig.baseUrl"
                      :placeholder="aiConfig.provider === 'custom' ? '例如: https://apis.iflow.cn' : getDefaultBaseUrl()"
                      :status="aiConfig.provider === 'custom' && !aiConfig.baseUrl ? 'error' : undefined"
                    />
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">
                      {{ aiConfig.provider === 'custom' ? '模型名称（必填）' : '自定义模型（可选）' }}
                    </div>
                    <div class="setting-desc">
                      {{ aiConfig.provider === 'custom' ? '请输入要使用的模型名称' : '留空使用默认模型' }}
                    </div>
                  </div>
                  <div class="setting-control-wide">
                    <n-input
                      v-model:value="aiConfig.model"
                      :placeholder="aiConfig.provider === 'custom' ? '例如: gpt-4o-mini' : getDefaultModel()"
                      :status="aiConfig.provider === 'custom' && !aiConfig.model ? 'error' : undefined"
                    />
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">最大输出 Token</div>
                    <div class="setting-desc">AI 响应的最大 token 数，根据模型调整（默认 4096）</div>
                  </div>
                  <div class="setting-control">
                    <n-input-number
                      v-model:value="aiConfig.maxTokens"
                      :min="512"
                      :max="32768"
                      :step="512"
                      style="width: 150px"
                    />
                  </div>
                </div>

                <div class="ai-actions">
                  <n-button
                    @click="testAiConnection"
                    :loading="testingAi"
                    :disabled="!isAiConfigValid"
                  >
                    测试连接
                  </n-button>
                  <n-button
                    type="primary"
                    @click="saveAiConfig"
                    :loading="savingAi"
                    :disabled="!isAiConfigValid"
                  >
                    保存配置
                  </n-button>
                </div>

                <n-alert v-if="aiTestResult !== null" :type="aiTestResult ? 'success' : 'error'" class="ai-test-result">
                  {{ aiTestResult ? 'AI 连接测试成功！' : 'AI 连接测试失败，请检查配置' }}
                </n-alert>
              </div>
            </section>
          </div>
        </n-tab-pane>

        <!-- AI 日志标签 -->
        <n-tab-pane name="ai-logs" tab="AI日志">
          <div class="tab-content">
            <!-- 统计卡片 -->
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">调用统计</h3>
                <n-button text @click="loadAiLogs" :loading="loadingAiLogs">
                  <template #icon>
                    <n-icon :component="ReloadOutline" />
                  </template>
                  刷新
                </n-button>
              </div>
              <div class="card-content">
                <div v-if="loadingAiStats" class="loading-state">
                  <n-spin size="small" />
                  <span>加载中...</span>
                </div>
                <n-grid v-else :cols="4" :x-gap="16" :y-gap="16">
                  <n-gi>
                    <div class="stat-card">
                      <div class="stat-value">{{ aiLogStats?.totalCalls || 0 }}</div>
                      <div class="stat-label">总调用次数</div>
                    </div>
                  </n-gi>
                  <n-gi>
                    <div class="stat-card stat-success">
                      <div class="stat-value">{{ aiLogStats?.successCount || 0 }}</div>
                      <div class="stat-label">成功次数</div>
                    </div>
                  </n-gi>
                  <n-gi>
                    <div class="stat-card stat-error">
                      <div class="stat-value">{{ aiLogStats?.errorCount || 0 }}</div>
                      <div class="stat-label">失败次数</div>
                    </div>
                  </n-gi>
                  <n-gi>
                    <div class="stat-card">
                      <div class="stat-value">{{ formatDuration(aiLogStats?.avgDurationMs || 0) }}</div>
                      <div class="stat-label">平均耗时</div>
                    </div>
                  </n-gi>
                </n-grid>

                <!-- 模块统计 -->
                <div v-if="aiLogStats?.callsByModule?.length" class="module-stats">
                  <div class="module-stats-title">按模块统计</div>
                  <div class="module-tags">
                    <n-tag v-for="[module, count] in aiLogStats.callsByModule" :key="module" size="small">
                      {{ getModuleName(module) }}: {{ count }}
                    </n-tag>
                  </div>
                </div>
              </div>
            </section>

            <!-- 日志列表 -->
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">调用日志</h3>
                <n-space>
                  <n-select
                    v-model:value="aiLogFilter.module"
                    :options="moduleOptions"
                    placeholder="筛选模块"
                    clearable
                    size="small"
                    style="width: 120px;"
                    @update:value="loadAiLogs"
                  />
                  <n-select
                    v-model:value="aiLogFilter.status"
                    :options="statusOptions"
                    placeholder="筛选状态"
                    clearable
                    size="small"
                    style="width: 100px;"
                    @update:value="loadAiLogs"
                  />
                  <n-button text type="error" @click="handleClearAiLogs">
                    <template #icon>
                      <n-icon :component="TrashOutline" />
                    </template>
                    清空日志
                  </n-button>
                </n-space>
              </div>
              <div class="card-content">
                <div v-if="loadingAiLogs" class="loading-state">
                  <n-spin size="small" />
                  <span>加载中...</span>
                </div>
                <n-empty v-else-if="!aiLogs.length" description="暂无日志记录" />
                <n-data-table
                  v-else
                  :columns="aiLogColumns"
                  :data="aiLogs"
                  :max-height="400"
                  :row-key="(row: AiLog) => row.id"
                  size="small"
                />
                <div v-if="aiLogs.length >= 50" class="load-more">
                  <n-button text @click="loadMoreAiLogs" :loading="loadingMoreLogs">
                    加载更多
                  </n-button>
                </div>
              </div>
            </section>
          </div>
        </n-tab-pane>

        <!-- AI 提示词管理标签 -->
        <n-tab-pane name="prompts" tab="提示词">
          <div class="tab-content">
            <PromptManager />
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

            <!-- 定时任务设置 -->
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">定时任务</h3>
                <span class="status-badge" :class="{ 'status-active': schedulerConfig.enableActivitySummary || schedulerConfig.enableTips || schedulerConfig.enableTodoPrediction }">
                  {{ (schedulerConfig.enableActivitySummary || schedulerConfig.enableTips || schedulerConfig.enableTodoPrediction) ? '已启用' : '全部关闭' }}
                </span>
              </div>
              <div class="card-content">
                <!-- Activity 总结 -->
                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">Activity 总结</div>
                    <div class="setting-desc">定时分析屏幕截图生成活动总结</div>
                  </div>
                  <div class="setting-control">
                    <n-switch v-model:value="schedulerConfig.enableActivitySummary" />
                  </div>
                </div>

                <div v-if="schedulerConfig.enableActivitySummary" class="setting-item sub-setting">
                  <div class="setting-info">
                    <div class="setting-label">执行间隔</div>
                    <div class="setting-desc">每隔多少分钟执行一次</div>
                  </div>
                  <div class="setting-control">
                    <n-input-number
                      v-model:value="schedulerConfig.activitySummaryIntervalMinutes"
                      :min="5"
                      :max="120"
                      :step="5"
                      style="width: 120px"
                    >
                      <template #suffix>分钟</template>
                    </n-input-number>
                  </div>
                </div>

                <!-- 智能提示 Tips -->
                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">智能提示 (Tips)</div>
                    <div class="setting-desc">基于 AI 分析用户活动，生成工作建议和提醒</div>
                  </div>
                  <div class="setting-control">
                    <n-switch v-model:value="schedulerConfig.enableTips" />
                  </div>
                </div>

                <div v-if="schedulerConfig.enableTips" class="setting-item sub-setting">
                  <div class="setting-info">
                    <div class="setting-label">执行间隔</div>
                    <div class="setting-desc">每隔多少分钟执行一次</div>
                  </div>
                  <div class="setting-control">
                    <n-input-number
                      v-model:value="schedulerConfig.tipsIntervalMinutes"
                      :min="15"
                      :max="240"
                      :step="15"
                      style="width: 120px"
                    >
                      <template #suffix>分钟</template>
                    </n-input-number>
                  </div>
                </div>

                <!-- TODO 预测 -->
                <div class="setting-item">
                  <div class="setting-info">
                    <div class="setting-label">TODO 预测</div>
                    <div class="setting-desc">AI 分析截图内容，智能预测可能需要处理的待办任务</div>
                  </div>
                  <div class="setting-control">
                    <n-switch v-model:value="schedulerConfig.enableTodoPrediction" />
                  </div>
                </div>

                <div v-if="schedulerConfig.enableTodoPrediction" class="setting-item sub-setting">
                  <div class="setting-info">
                    <div class="setting-label">执行间隔</div>
                    <div class="setting-desc">每隔多少分钟执行一次</div>
                  </div>
                  <div class="setting-control">
                    <n-input-number
                      v-model:value="schedulerConfig.todoPredictionIntervalMinutes"
                      :min="30"
                      :max="480"
                      :step="30"
                      style="width: 120px"
                    >
                      <template #suffix>分钟</template>
                    </n-input-number>
                  </div>
                </div>

                <div class="scheduler-actions">
                  <n-button @click="saveSchedulerConfig" :loading="savingScheduler">
                    保存定时任务设置
                  </n-button>
                  <n-text depth="3" style="font-size: 12px;">
                    注意：修改设置后需要重启应用才能生效
                  </n-text>
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

        <!-- 屏幕上下文标签 -->
        <n-tab-pane name="context" tab="屏幕上下文">
          <div class="tab-content">
            <ContextManager />
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
                  <p><strong>当前版本:</strong> v{{ appVersion }}</p>
                  <p><strong>技术栈:</strong> Tauri + Rust + Vue3 + TypeScript + Naive UI</p>
                </div>
              </div>
            </section>

            <!-- 检查更新 -->
            <section class="settings-card">
              <div class="card-header">
                <h3 class="card-title">软件更新</h3>
              </div>
              <div class="card-content">
                <div class="update-section">
                  <div class="update-info">
                    <div v-if="!checkingUpdate && !updateAvailable">
                      <p class="update-status">点击下方按钮检查是否有新版本</p>
                    </div>
                    <div v-else-if="checkingUpdate">
                      <p class="update-status checking">
                        <n-spin size="small" />
                        正在检查更新...
                      </p>
                    </div>
                    <div v-else-if="updateAvailable">
                      <p class="update-status available">
                        发现新版本: <strong>v{{ newVersion }}</strong>
                      </p>
                      <div class="update-notes" v-if="updateNotes">
                        <div class="notes-label">更新内容:</div>
                        <div class="notes-content" v-html="renderedUpdateNotes"></div>
                      </div>
                    </div>
                    <div v-else-if="updateError">
                      <p class="update-status error">{{ updateError }}</p>
                    </div>
                  </div>
                  <div class="update-actions">
                    <n-button
                      @click="handleCheckUpdate"
                      :loading="checkingUpdate"
                      :disabled="isUpdating"
                    >
                      <template #icon>
                        <n-icon :component="RefreshOutline" />
                      </template>
                      检查更新
                    </n-button>
                    <n-button
                      v-if="updateAvailable"
                      type="primary"
                      @click="handleDownloadUpdate"
                      :loading="isUpdating"
                    >
                      <template #icon>
                        <n-icon :component="DownloadOutline" />
                      </template>
                      {{ isUpdating ? `下载中 ${updateProgress}%` : '立即更新' }}
                    </n-button>
                    <n-button
                      quaternary
                      size="small"
                      @click="handleResetUpdateCheck"
                    >
                      重置自动检查
                    </n-button>
                  </div>
                  <n-progress
                    v-if="isUpdating"
                    type="line"
                    :percentage="updateProgress"
                    :height="6"
                    :border-radius="3"
                    color="#6366f1"
                    rail-color="rgba(30, 41, 59, 0.5)"
                    style="margin-top: 16px;"
                  />
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
import { ref, onMounted, computed, h } from 'vue';
import { useRoute } from 'vue-router';
import { NTabs, NTabPane, NSpace, NSwitch, NSelect, NInput, NInputNumber, NButton, NIcon, NSpin, NAlert, NDataTable, NTag, NEmpty, NStatistic, NGrid, NGi, NProgress, NText, useMessage, useDialog } from 'naive-ui';
import { TimeOutline, RefreshOutline, InformationCircleOutline, TrashOutline, ReloadOutline, DownloadOutline } from '@vicons/ionicons5';
import { marked } from 'marked';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { getVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { aiApi } from '@/api/aiApi';
import { AI_PROVIDERS } from '@/types/ai';
import type { AiProvider, AiLog, AiLogStats } from '@/types/ai';
import ContextManager from '@/components/context/ContextManager.vue';
import PromptManager from '@/components/settings/PromptManager.vue';

const route = useRoute();
const message = useMessage();
const dialog = useDialog();

const activeTab = ref('general');

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
  logLevel: 'info',
});

// 日志级别选项
const logLevelOptions = [
  { label: 'Trace (最详细)', value: 'trace' },
  { label: 'Debug (调试)', value: 'debug' },
  { label: 'Info (信息)', value: 'info' },
  { label: 'Warn (警告)', value: 'warn' },
  { label: 'Error (错误)', value: 'error' },
  { label: 'Off (关闭)', value: 'off' },
];

const shortcuts = ref({
  taskBoard: 'Ctrl+Shift+N',
  sqlHistory: 'Ctrl+Shift+S',
  appLauncher: 'Ctrl+Shift+Space',
  quickTask: 'Ctrl+Shift+T',
});

// AI 配置
const aiConfig = ref({
  provider: 'deepseek' as AiProvider,
  apiKey: '',
  baseUrl: '',
  model: '',
  enabled: false,
  maxTokens: 4096,
});

const testingAi = ref(false);
const savingAi = ref(false);
const aiTestResult = ref<boolean | null>(null);

// 定时任务调度器配置
const schedulerConfig = ref({
  activitySummaryIntervalMinutes: 30,
  tipsIntervalMinutes: 60,
  todoPredictionIntervalMinutes: 120,
  enableActivitySummary: false,
  enableTips: false,
  enableTodoPrediction: false,
});
const savingScheduler = ref(false);

// 更新相关变量
const appVersion = ref('');
const checkingUpdate = ref(false);
const updateAvailable = ref(false);
const newVersion = ref('');
const updateNotes = ref('');
const updateError = ref('');
const isUpdating = ref(false);
const updateProgress = ref(0);
let currentUpdate: any = null;

// 渲染更新说明为 HTML
const renderedUpdateNotes = computed(() => {
  if (!updateNotes.value) return '';
  return marked(updateNotes.value);
});

const aiProviderOptions = AI_PROVIDERS.map(p => ({
  label: p.name,
  value: p.id,
}));

// AI 配置验证：自定义提供商需要填写 baseUrl 和 model
const isAiConfigValid = computed(() => {
  if (!aiConfig.value.apiKey) return false;
  if (aiConfig.value.provider === 'custom') {
    return !!(aiConfig.value.baseUrl && aiConfig.value.model);
  }
  return true;
});

// AI 日志相关
const aiLogs = ref<AiLog[]>([]);
const aiLogStats = ref<AiLogStats | null>(null);
const loadingAiLogs = ref(false);
const loadingAiStats = ref(false);
const loadingMoreLogs = ref(false);
const aiLogFilter = ref({
  module: null as string | null,
  status: null as string | null,
});

const moduleOptions = [
  { label: '工作日志', value: 'work_log' },
  { label: '任务', value: 'task' },
  { label: '应用启动器', value: 'app_launcher' },
];

const statusOptions = [
  { label: '成功', value: 'success' },
  { label: '失败', value: 'error' },
];

const aiLogColumns = [
  {
    title: '时间',
    key: 'createdAt',
    width: 160,
    render: (row: AiLog) => formatTime(row.createdAt),
  },
  {
    title: '模块',
    key: 'module',
    width: 100,
    render: (row: AiLog) => getModuleName(row.module),
  },
  {
    title: '操作',
    key: 'action',
    width: 120,
    render: (row: AiLog) => getActionName(row.action),
  },
  {
    title: '状态',
    key: 'status',
    width: 80,
    render: (row: AiLog) => {
      return h(NTag, {
        type: row.status === 'success' ? 'success' : 'error',
        size: 'small',
      }, () => row.status === 'success' ? '成功' : '失败');
    },
  },
  {
    title: '耗时',
    key: 'durationMs',
    width: 80,
    render: (row: AiLog) => row.durationMs ? `${row.durationMs}ms` : '-',
  },
  {
    title: '提示词',
    key: 'prompt',
    ellipsis: {
      tooltip: true,
    },
  },
];

function getModuleName(module: string): string {
  const names: Record<string, string> = {
    'work_log': '工作日志',
    'task': '任务',
    'app_launcher': '应用启动器',
  };
  return names[module] || module;
}

function getActionName(action: string): string {
  const names: Record<string, string> = {
    'generate': '生成',
    'polish': '润色',
    'weekly_report': '周报',
    'classify': '分类',
    'enhance_description': '增强描述',
    'generate_subtasks': '生成子任务',
    'summarize': '总结',
    'classify_apps': '应用分类',
    'recommend_workflows': '推荐工作流',
    'generate_description': '生成描述',
  };
  return names[action] || action;
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${Math.round(ms)}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

async function loadAiLogs() {
  loadingAiLogs.value = true;
  loadingAiStats.value = true;
  try {
    const [logs, stats] = await Promise.all([
      aiApi.getLogs({
        module: aiLogFilter.value.module || undefined,
        status: aiLogFilter.value.status || undefined,
        limit: 50,
      }),
      aiApi.getLogStats(),
    ]);
    aiLogs.value = logs;
    aiLogStats.value = stats;
  } catch (error) {
    console.error('加载 AI 日志失败:', error);
    message.error('加载 AI 日志失败');
  } finally {
    loadingAiLogs.value = false;
    loadingAiStats.value = false;
  }
}

async function loadMoreAiLogs() {
  loadingMoreLogs.value = true;
  try {
    const logs = await aiApi.getLogs({
      module: aiLogFilter.value.module || undefined,
      status: aiLogFilter.value.status || undefined,
      limit: 50,
      offset: aiLogs.value.length,
    });
    aiLogs.value = [...aiLogs.value, ...logs];
  } catch (error) {
    console.error('加载更多日志失败:', error);
    message.error('加载更多日志失败');
  } finally {
    loadingMoreLogs.value = false;
  }
}

function handleClearAiLogs() {
  dialog.warning({
    title: '清空 AI 日志',
    content: '确定要清空所有 AI 调用日志吗？此操作不可撤销！',
    positiveText: '清空',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        const count = await aiApi.clearLogs();
        message.success(`已清空 ${count} 条日志`);
        await loadAiLogs();
      } catch (error) {
        message.error('清空日志失败');
        console.error('清空日志失败:', error);
      }
    },
  });
}

onMounted(async () => {
  // 检查 URL 参数，如果有 tab 参数则切换到对应标签
  const tabParam = route.query.tab as string;
  if (tabParam) {
    activeTab.value = tabParam;
  }

  await loadAppVersion();
  await loadSettings();
  await loadShortcuts();
  await loadAiConfig();
  await loadAiLogs();
  await loadDatabasePath();
  await loadAutostartStatus();
  await loadAlwaysOnTopStatus();
  await loadSchedulerConfig();
});

async function loadSettings() {
  try {
    const appSettings = await invoke<any>('get_app_settings');

    // 映射后端设置到前端设置
    settings.value = {
      theme: appSettings.theme || 'auto',
      alwaysOnTop: appSettings.always_on_top || false,
      autoStart: appSettings.auto_start || false,
      clipboardMonitor: appSettings.enable_clipboard_monitoring || true,
      clipboardInterval: appSettings.clipboard_interval || 2,
      gitEnabled: appSettings.git_enabled || false,
      gitPath: appSettings.git_path || '',
      aiEnabled: appSettings.enable_ai || false,
      claudeApiKey: '',
      taskNotification: appSettings.task_notification || true,
      staleTaskDays: appSettings.stale_task_days || 3,
      worklogReminder: appSettings.worklog_reminder || true,
      logLevel: appSettings.log_level || 'info',
    };
  } catch (error) {
    console.error('加载设置失败:', error);
    // 使用默认值
  }
}

async function loadShortcuts() {
  loadingShortcuts.value = true;
  try {
    const config = await invoke<{
      task_board: string;
      sql_history: string;
      app_launcher: string;
      quick_task: string;
    }>('get_shortcut_config');

    shortcuts.value = {
      taskBoard: config.task_board,
      sqlHistory: config.sql_history,
      appLauncher: config.app_launcher,
      quickTask: config.quick_task,
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
        quick_task: shortcuts.value.quickTask,
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

function handleShortcutKeyDown(event: KeyboardEvent, field: 'taskBoard' | 'sqlHistory' | 'appLauncher' | 'quickTask') {
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

// 加载数据库路径
async function loadDatabasePath() {
  try {
    const path = await invoke<string>('get_database_path');
    dataPath.value = path;
  } catch (error) {
    console.error('获取数据库路径失败:', error);
  }
}

// 加载自启动状态
async function loadAutostartStatus() {
  try {
    const status = await invoke<{ enabled: boolean }>('get_autostart_status');
    settings.value.autoStart = status.enabled;
  } catch (error) {
    console.error('获取自启动状态失败:', error);
  }
}

// 加载窗口置顶状态
async function loadAlwaysOnTopStatus() {
  try {
    const isOnTop = await invoke<boolean>('get_always_on_top');
    settings.value.alwaysOnTop = isOnTop;
  } catch (error) {
    console.error('获取窗口置顶状态失败:', error);
  }
}

async function handleSave() {
  saving.value = true;
  try {
    // 保存快捷键
    await saveShortcuts();

    // 保存所有应用设置到数据库
    await invoke('save_app_settings', {
      settings: {
        theme: settings.value.theme,
        language: 'zh-CN',
        always_on_top: settings.value.alwaysOnTop,
        start_minimized: false,
        minimize_to_tray: true,
        auto_save_context: true,
        stale_task_days: settings.value.staleTaskDays,
        completed_tasks_retention_days: 7,
        enable_clipboard_monitoring: settings.value.clipboardMonitor,
        clipboard_interval: settings.value.clipboardInterval,
        sql_history_limit: 100,
        auto_detect_sql_type: true,
        enable_notifications: true,
        notify_on_task_complete: true,
        notify_on_sql_detected: false,
        task_notification: settings.value.taskNotification,
        worklog_reminder: settings.value.worklogReminder,
        enable_ai: settings.value.aiEnabled,
        ai_model: 'claude-3-sonnet-20240229',
        enable_git_integration: settings.value.gitEnabled,
        auto_detect_branch: true,
        git_enabled: settings.value.gitEnabled,
        git_path: settings.value.gitPath,
        auto_start: settings.value.autoStart,
        log_level: settings.value.logLevel,
      },
    });

    // 应用窗口置顶设置
    await invoke('set_always_on_top', { alwaysOnTop: settings.value.alwaysOnTop });

    // 应用开机自启动设置
    await invoke('set_autostart', { enable: settings.value.autoStart });

    // 动态设置日志级别
    await invoke('set_log_level', { level: settings.value.logLevel });

    message.success('设置已保存');
  } catch (error) {
    message.error('保存失败');
    console.error(error);
  } finally {
    saving.value = false;
  }
}

async function openDataFolder() {
  try {
    await invoke('open_data_folder');
    message.success('已打开数据文件夹');
  } catch (error: any) {
    message.error(error || '打开文件夹失败');
    console.error('打开数据文件夹失败:', error);
  }
}

async function handleExport() {
  try {
    // 使用Tauri的保存文件对话框
    const { save } = await import('@tauri-apps/plugin-dialog');
    const filePath = await save({
      defaultPath: 'dev_assistant_backup.db',
      filters: [{
        name: 'Database',
        extensions: ['db']
      }]
    });

    if (filePath) {
      await invoke('export_database', { exportPath: filePath });
      message.success('数据导出成功');
    }
  } catch (error: any) {
    message.error(error || '导出数据失败');
    console.error('导出数据失败:', error);
  }
}

function handleClearData() {
  dialog.warning({
    title: '清空数据',
    content: '确定要清空所有数据吗？此操作不可撤销！注意：应用设置将会保留。',
    positiveText: '清空',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await invoke('clear_all_data');
        message.success('数据已清空');
      } catch (error: any) {
        message.error(error || '清空数据失败');
        console.error('清空数据失败:', error);
      }
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

// AI 配置相关方法
function getDefaultBaseUrl() {
  const provider = AI_PROVIDERS.find(p => p.id === aiConfig.value.provider);
  return provider?.defaultUrl || '';
}

function getDefaultModel() {
  const provider = AI_PROVIDERS.find(p => p.id === aiConfig.value.provider);
  return provider?.defaultModel || '';
}

async function loadAiConfig() {
  try {
    const config = await aiApi.getConfig();
    if (config) {
      aiConfig.value.provider = config.provider;
      aiConfig.value.baseUrl = config.baseUrl || '';
      aiConfig.value.model = config.model || '';
      aiConfig.value.enabled = config.enabled;
      aiConfig.value.maxTokens = config.maxTokens || 4096;
      // apiKey 需要用户重新输入，不从服务端加载
    }
  } catch (error) {
    console.error('加载 AI 配置失败:', error);
  }
}

async function testAiConnection() {
  testingAi.value = true;
  aiTestResult.value = null;
  try {
    // 先保存配置再测试
    await aiApi.saveConfig({
      provider: aiConfig.value.provider,
      apiKey: aiConfig.value.apiKey,
      baseUrl: aiConfig.value.baseUrl || undefined,
      model: aiConfig.value.model || undefined,
      enabled: aiConfig.value.enabled,
      maxTokens: aiConfig.value.maxTokens,
    });
    const result = await aiApi.testConnection();
    aiTestResult.value = result;
    if (result) {
      message.success('AI 连接测试成功');
    } else {
      message.error('AI 连接测试失败');
    }
  } catch (error: any) {
    aiTestResult.value = false;
    message.error(error || '连接测试失败');
  } finally {
    testingAi.value = false;
  }
}

async function saveAiConfig() {
  savingAi.value = true;
  try {
    await aiApi.saveConfig({
      provider: aiConfig.value.provider,
      apiKey: aiConfig.value.apiKey,
      baseUrl: aiConfig.value.baseUrl || undefined,
      model: aiConfig.value.model || undefined,
      enabled: aiConfig.value.enabled,
      maxTokens: aiConfig.value.maxTokens,
    });
    message.success('AI 配置已保存');
  } catch (error: any) {
    message.error(error || '保存失败');
  } finally {
    savingAi.value = false;
  }
}

// 加载定时任务调度器配置
async function loadSchedulerConfig() {
  try {
    const config = await invoke<{
      activity_summary_interval_minutes: number;
      tips_interval_minutes: number;
      todo_prediction_interval_minutes: number;
      enable_activity_summary: boolean;
      enable_tips: boolean;
      enable_todo_prediction: boolean;
    }>('get_scheduler_config');

    schedulerConfig.value = {
      activitySummaryIntervalMinutes: config.activity_summary_interval_minutes,
      tipsIntervalMinutes: config.tips_interval_minutes,
      todoPredictionIntervalMinutes: config.todo_prediction_interval_minutes,
      enableActivitySummary: config.enable_activity_summary,
      enableTips: config.enable_tips,
      enableTodoPrediction: config.enable_todo_prediction,
    };
  } catch (error) {
    console.error('加载定时任务配置失败:', error);
  }
}

// 保存定时任务调度器配置
async function saveSchedulerConfig() {
  savingScheduler.value = true;
  try {
    await invoke('save_scheduler_config', {
      config: {
        activity_summary_interval_minutes: schedulerConfig.value.activitySummaryIntervalMinutes,
        tips_interval_minutes: schedulerConfig.value.tipsIntervalMinutes,
        todo_prediction_interval_minutes: schedulerConfig.value.todoPredictionIntervalMinutes,
        enable_activity_summary: schedulerConfig.value.enableActivitySummary,
        enable_tips: schedulerConfig.value.enableTips,
        enable_todo_prediction: schedulerConfig.value.enableTodoPrediction,
      },
    });
    message.success('定时任务配置已保存，重启应用后生效');
  } catch (error: any) {
    message.error(error || '保存定时任务配置失败');
  } finally {
    savingScheduler.value = false;
  }
}

// 加载应用版本
async function loadAppVersion() {
  try {
    appVersion.value = await getVersion();
  } catch (error) {
    console.error('获取应用版本失败:', error);
    appVersion.value = '1.0.0';
  }
}

// 记录更新日志
async function logUpdateInfo(msg: string) {
  try {
    await invoke('log_update_info', { message: msg });
  } catch (e) {
    // 静默失败
  }
}

// 检查更新
async function handleCheckUpdate() {
  checkingUpdate.value = true;
  updateAvailable.value = false;
  updateError.value = '';
  newVersion.value = '';
  updateNotes.value = '';

  try {
    await logUpdateInfo(`[手动检查] ========== 开始手动检查更新 ==========`);
    await logUpdateInfo(`[手动检查] 当前应用版本: ${appVersion.value}`);

    // 先手动请求看看服务器返回什么
    try {
      const testUrl = `http://d.wbdao.cn:9900/update/update.php?target=windows-x86_64&current_version=${appVersion.value}`;
      await logUpdateInfo(`[手动检查] 请求URL: ${testUrl}`);
      const response = await fetch(testUrl);
      await logUpdateInfo(`[手动检查] HTTP状态码: ${response.status}`);
      const text = await response.text();
      await logUpdateInfo(`[手动检查] 服务器原始响应: ${text.substring(0, 500)}`);
    } catch (e: any) {
      await logUpdateInfo(`[手动检查] 手动请求失败: ${e?.message || e}`);
    }

    const update = await check();

    if (update) {
      await logUpdateInfo(`[手动检查] 发现新版本: ${update.version}`);
      await logUpdateInfo(`[手动检查] 发布日期: ${update.date || '未知'}`);
      await logUpdateInfo(`[手动检查] 更新说明: ${update.body?.substring(0, 100) || '无'}...`);

      updateAvailable.value = true;
      newVersion.value = update.version;
      updateNotes.value = update.body || '';
      currentUpdate = update;
      message.success(`发现新版本: v${update.version}`);
    } else {
      await logUpdateInfo(`[手动检查] 当前已是最新版本，无需更新`);
      message.info('当前已是最新版本');
    }
  } catch (error: any) {
    await logUpdateInfo(`[手动检查] 检查更新失败: ${error?.message || error}`);
    updateError.value = `检查更新失败: ${error?.message || error}`;
    message.error('检查更新失败');
  } finally {
    await logUpdateInfo(`[手动检查] ========== 手动检查更新结束 ==========`);
    checkingUpdate.value = false;
  }
}

// 重置自动更新检查时间（用于调试）
function handleResetUpdateCheck() {
  try {
    localStorage.removeItem('last_update_check_time');
    localStorage.removeItem('skipped_version');
    message.success('已重置自动更新检查，下次启动时将自动检查更新');
  } catch (error) {
    message.error('重置失败');
  }
}

// 下载并安装更新
async function handleDownloadUpdate() {
  if (!currentUpdate) {
    message.error('没有可用的更新');
    return;
  }

  isUpdating.value = true;
  updateProgress.value = 0;

  try {
    let downloaded = 0;
    let contentLength = 0;

    await currentUpdate.downloadAndInstall((event: any) => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength || 0;
          console.log(`开始下载, 总大小: ${contentLength} 字节`);
          break;
        case 'Progress':
          downloaded += event.data.chunkLength;
          if (contentLength > 0) {
            updateProgress.value = Math.round((downloaded / contentLength) * 100);
          }
          break;
        case 'Finished':
          updateProgress.value = 100;
          console.log('下载完成');
          break;
      }
    });

    message.success('更新下载完成，即将重启应用...');

    // 等待一小段时间让用户看到消息
    setTimeout(async () => {
      await relaunch();
    }, 1500);
  } catch (error: any) {
    console.error('下载更新失败:', error);
    message.error(`下载更新失败: ${error?.message || error}`);
    isUpdating.value = false;
    updateProgress.value = 0;
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
  position: sticky;
  top: -16px;
  background: rgb(2, 6, 23);
  padding-top: 16px;
  padding-bottom: 16px;
  margin-top: -16px;
  z-index: 100;
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

/* 日志级别选择器 */
.log-level-select {
  min-width: 160px;
}

/* AI 配置 */
.provider-select {
  min-width: 200px;
}

.ai-actions {
  display: flex;
  gap: 12px;
  margin-top: 16px;
}

.ai-test-result {
  margin-top: 16px;
}

/* AI 日志样式 */
.stat-card {
  background: rgba(30, 41, 59, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.6);
  border-radius: 12px;
  padding: 16px;
  text-align: center;
}

.stat-card .stat-value {
  font-size: 24px;
  font-weight: 600;
  color: rgb(226, 232, 240);
  margin-bottom: 4px;
}

.stat-card .stat-label {
  font-size: 12px;
  color: rgb(148, 163, 184);
}

.stat-card.stat-success .stat-value {
  color: rgb(52, 211, 153);
}

.stat-card.stat-error .stat-value {
  color: rgb(248, 113, 113);
}

.module-stats {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid rgba(51, 65, 85, 0.4);
}

.module-stats-title {
  font-size: 13px;
  color: rgb(148, 163, 184);
  margin-bottom: 12px;
}

.module-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.load-more {
  text-align: center;
  margin-top: 16px;
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
  padding-top: 16px;
  padding-bottom: 16px;
  position: sticky;
  bottom: -32px;
  background: rgb(2, 6, 23);
  margin-bottom: -32px;
  z-index: 98;
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

:deep(.n-tabs-nav) {
  position: sticky;
  top: 52px;
  background: rgb(2, 6, 23);
  z-index: 99;
  padding-bottom: 8px;
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

/* 更新部分样式 */
.update-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.update-info {
  flex: 1;
}

.update-status {
  font-size: 14px;
  color: rgb(148, 163, 184);
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.update-status.checking {
  color: rgb(99, 102, 241);
}

.update-status.available {
  color: rgb(52, 211, 153);
}

.update-status.available strong {
  color: rgb(74, 222, 128);
}

.update-status.error {
  color: rgb(248, 113, 113);
}

.update-notes {
  margin-top: 16px;
  padding: 16px;
  background: rgba(2, 6, 23, 0.8);
  border: 1px solid rgba(51, 65, 85, 0.6);
  border-radius: 8px;
  max-height: 200px;
  overflow-y: auto;
}

.notes-label {
  font-size: 13px;
  font-weight: 500;
  color: rgb(148, 163, 184);
  margin-bottom: 8px;
}

.notes-content {
  font-size: 13px;
  line-height: 1.6;
  color: rgb(203, 213, 225);
}

.notes-content :deep(h2) {
  font-size: 14px;
  font-weight: 600;
  color: rgb(226, 232, 240);
  margin: 12px 0 8px 0;
}

.notes-content :deep(h3) {
  font-size: 13px;
  font-weight: 500;
  color: rgb(203, 213, 225);
  margin: 8px 0 6px 0;
}

.notes-content :deep(ul) {
  margin: 4px 0;
  padding-left: 20px;
}

.notes-content :deep(li) {
  margin: 4px 0;
}

.update-actions {
  display: flex;
  gap: 12px;
  margin-top: 8px;
}

.update-notes::-webkit-scrollbar {
  width: 6px;
}

.update-notes::-webkit-scrollbar-track {
  background: transparent;
}

.update-notes::-webkit-scrollbar-thumb {
  background: rgba(51, 65, 85, 0.5);
  border-radius: 3px;
}

.update-notes::-webkit-scrollbar-thumb:hover {
  background: rgba(71, 85, 105, 0.7);
}

/* 子设置项缩进 */
.sub-setting {
  padding-left: 24px;
  border-left: 2px solid rgba(51, 65, 85, 0.6);
  margin-left: 8px;
}

/* 定时任务操作区 */
.scheduler-actions {
  display: flex;
  align-items: center;
  gap: 16px;
  padding-top: 16px;
  border-top: 1px solid rgba(51, 65, 85, 0.4);
  margin-top: 8px;
}
</style>
