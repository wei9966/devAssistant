<template>
  <div class="prompt-manager">
    <!-- 顶部操作栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <n-select
          v-model:value="selectedModule"
          :options="moduleOptions"
          placeholder="筛选模块"
          clearable
          style="width: 160px"
        />
        <n-input
          v-model:value="searchKeyword"
          placeholder="搜索提示词..."
          clearable
          style="width: 200px"
        >
          <template #prefix>
            <n-icon :component="SearchOutline" />
          </template>
        </n-input>
      </div>
      <div class="toolbar-right">
        <n-button @click="handleRefreshCache" :loading="refreshing">
          <template #icon>
            <n-icon :component="RefreshOutline" />
          </template>
          刷新缓存
        </n-button>
        <n-popconfirm @positive-click="handleResetAll">
          <template #trigger>
            <n-button type="warning">
              <template #icon>
                <n-icon :component="RefreshCircleOutline" />
              </template>
              重置全部
            </n-button>
          </template>
          确定要重置所有提示词为默认值吗？
        </n-popconfirm>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <n-spin size="medium" />
      <span>加载中...</span>
    </div>

    <!-- 提示词列表 -->
    <div v-else class="prompt-list">
      <div v-for="(group, module) in groupedPrompts" :key="module" class="prompt-group">
        <div class="group-header">
          <h3 class="group-title">{{ getModuleName(module as string) }}</h3>
          <n-tag size="small" type="info">{{ group.length }} 个提示词</n-tag>
        </div>

        <div class="prompt-cards">
          <div
            v-for="prompt in group"
            :key="prompt.prompt_key"
            class="prompt-card"
            :class="{ 'prompt-disabled': !prompt.enabled }"
          >
            <div class="card-header">
              <div class="card-title-row">
                <span class="card-title">{{ prompt.name }}</span>
                <n-tag v-if="prompt.is_system" size="tiny" type="success">系统</n-tag>
                <n-tag v-if="!prompt.enabled" size="tiny" type="warning">已禁用</n-tag>
              </div>
              <div class="card-actions">
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <n-button text size="small" @click="handlePreview(prompt)">
                      <template #icon>
                        <n-icon :component="EyeOutline" />
                      </template>
                    </n-button>
                  </template>
                  预览
                </n-tooltip>
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <n-button text size="small" @click="handleEdit(prompt)">
                      <template #icon>
                        <n-icon :component="CreateOutline" />
                      </template>
                    </n-button>
                  </template>
                  编辑
                </n-tooltip>
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <n-button text size="small" @click="handleReset(prompt.prompt_key)">
                      <template #icon>
                        <n-icon :component="RefreshOutline" />
                      </template>
                    </n-button>
                  </template>
                  重置
                </n-tooltip>
              </div>
            </div>

            <div class="card-description">{{ prompt.description || '暂无描述' }}</div>

            <div class="card-key">
              <code>{{ prompt.prompt_key }}</code>
            </div>

            <div v-if="prompt.variables" class="card-variables">
              <span class="variables-label">变量:</span>
              <n-tag
                v-for="v in parseVariables(prompt.variables)"
                :key="v"
                size="tiny"
                class="variable-tag"
              >
                {{ v }}
              </n-tag>
            </div>
          </div>
        </div>
      </div>

      <n-empty v-if="Object.keys(groupedPrompts).length === 0" description="暂无匹配的提示词" />
    </div>

    <!-- 编辑弹窗 -->
    <n-modal
      v-model:show="showEditModal"
      preset="card"
      :title="editingPrompt ? `编辑提示词: ${editingPrompt.name}` : '编辑提示词'"
      style="width: 800px; max-width: 90vw;"
      :mask-closable="false"
    >
      <div v-if="editingPrompt" class="edit-form">
        <div class="form-item">
          <div class="form-label">
            <span>启用状态</span>
          </div>
          <n-switch v-model:value="editForm.enabled" />
        </div>

        <div class="form-item">
          <div class="form-label">
            <span>系统提示词 (System Prompt)</span>
            <n-tag size="tiny" type="info">可选</n-tag>
          </div>
          <n-input
            v-model:value="editForm.systemPrompt"
            type="textarea"
            placeholder="系统提示词，用于设定 AI 的角色和行为..."
            :autosize="{ minRows: 4, maxRows: 10 }"
          />
        </div>

        <div class="form-item">
          <div class="form-label">
            <span>用户提示词 (User Prompt)</span>
            <n-tag size="tiny" type="warning">必填</n-tag>
          </div>
          <n-input
            v-model:value="editForm.userPrompt"
            type="textarea"
            placeholder="用户提示词，支持 {变量名} 占位符..."
            :autosize="{ minRows: 6, maxRows: 15 }"
          />
        </div>

        <div v-if="editingPrompt.variables" class="form-item">
          <div class="form-label">
            <span>可用变量</span>
          </div>
          <div class="variables-list">
            <n-tag
              v-for="v in parseVariables(editingPrompt.variables)"
              :key="v"
              size="small"
              class="variable-tag clickable"
              @click="insertVariable(v)"
            >
              {{"{"}}{{ v }}{{"}"}}
            </n-tag>
          </div>
          <div class="variables-hint">点击变量可插入到用户提示词中</div>
        </div>
      </div>

      <template #footer>
        <div class="modal-footer">
          <n-button @click="showEditModal = false">取消</n-button>
          <n-button type="primary" @click="handleSave" :loading="saving">
            保存
          </n-button>
        </div>
      </template>
    </n-modal>

    <!-- 预览弹窗 -->
    <n-modal
      v-model:show="showPreviewModal"
      preset="card"
      :title="previewingPrompt ? `预览: ${previewingPrompt.name}` : '预览提示词'"
      style="width: 800px; max-width: 90vw;"
    >
      <div v-if="previewingPrompt" class="preview-content">
        <div class="preview-section">
          <div class="preview-label">系统提示词</div>
          <pre class="preview-text">{{ previewingPrompt.system_prompt || '(无)' }}</pre>
        </div>

        <div class="preview-section">
          <div class="preview-label">用户提示词 (原始)</div>
          <pre class="preview-text">{{ previewingPrompt.user_prompt }}</pre>
        </div>

        <n-divider>渲染预览</n-divider>

        <div class="preview-variables">
          <div class="preview-label">测试变量</div>
          <div class="test-variables">
            <div
              v-for="v in parseVariables(previewingPrompt.variables)"
              :key="v"
              class="test-variable-item"
            >
              <span class="test-var-name">{{ v }}:</span>
              <n-input
                v-model:value="testVariables[v]"
                size="small"
                placeholder="输入测试值..."
                style="width: 200px"
              />
            </div>
          </div>
          <n-button size="small" @click="handleRenderPreview" :loading="rendering">
            渲染预览
          </n-button>
        </div>

        <div v-if="renderedPreview" class="preview-section">
          <div class="preview-label">渲染结果</div>
          <pre class="preview-text rendered">{{ renderedPreview.user }}</pre>
        </div>
      </div>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  NSelect, NInput, NButton, NIcon, NSpin, NTag, NEmpty,
  NModal, NSwitch, NDivider, NTooltip, NPopconfirm, useMessage
} from 'naive-ui';
import {
  SearchOutline, RefreshOutline, RefreshCircleOutline,
  EyeOutline, CreateOutline
} from '@vicons/ionicons5';
import { promptApi } from '@/api/promptApi';
import type { AiPrompt, RenderedPrompt } from '@/api/promptApi';

const message = useMessage();

// 状态
const loading = ref(false);
const refreshing = ref(false);
const saving = ref(false);
const rendering = ref(false);
const prompts = ref<AiPrompt[]>([]);
const selectedModule = ref<string | null>(null);
const searchKeyword = ref('');

// 编辑相关
const showEditModal = ref(false);
const editingPrompt = ref<AiPrompt | null>(null);
const editForm = ref({
  systemPrompt: '' as string | null,
  userPrompt: '',
  enabled: true,
});

// 预览相关
const showPreviewModal = ref(false);
const previewingPrompt = ref<AiPrompt | null>(null);
const testVariables = ref<Record<string, string>>({});
const renderedPreview = ref<RenderedPrompt | null>(null);

// 模块选项
const moduleOptions = [
  { label: '工作日志', value: 'work_log' },
  { label: '任务管理', value: 'task' },
  { label: '应用启动器', value: 'app_launcher' },
  { label: '屏幕上下文', value: 'screen_context' },
];

// 模块名称映射
function getModuleName(module: string): string {
  const names: Record<string, string> = {
    'work_log': '工作日志',
    'task': '任务管理',
    'app_launcher': '应用启动器',
    'screen_context': '屏幕上下文',
  };
  return names[module] || module;
}

// 解析变量
function parseVariables(variables: string | null): string[] {
  if (!variables) return [];
  try {
    return JSON.parse(variables);
  } catch {
    return [];
  }
}

// 分组后的提示词
const groupedPrompts = computed(() => {
  let filtered = prompts.value;

  // 按模块筛选
  if (selectedModule.value) {
    filtered = filtered.filter(p => p.module === selectedModule.value);
  }

  // 按关键词搜索
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase();
    filtered = filtered.filter(p =>
      p.name.toLowerCase().includes(keyword) ||
      p.prompt_key.toLowerCase().includes(keyword) ||
      (p.description && p.description.toLowerCase().includes(keyword))
    );
  }

  // 按模块分组
  const groups: Record<string, AiPrompt[]> = {};
  for (const prompt of filtered) {
    if (!groups[prompt.module]) {
      groups[prompt.module] = [];
    }
    groups[prompt.module].push(prompt);
  }

  return groups;
});

// 加载提示词
async function loadPrompts() {
  loading.value = true;
  try {
    prompts.value = await promptApi.getAllPrompts();
  } catch (error) {
    console.error('加载提示词失败:', error);
    message.error('加载提示词失败');
  } finally {
    loading.value = false;
  }
}

// 刷新缓存
async function handleRefreshCache() {
  refreshing.value = true;
  try {
    await promptApi.refreshCache();
    await loadPrompts();
    message.success('缓存已刷新');
  } catch (error) {
    console.error('刷新缓存失败:', error);
    message.error('刷新缓存失败');
  } finally {
    refreshing.value = false;
  }
}

// 重置全部
async function handleResetAll() {
  try {
    await promptApi.resetAllPrompts();
    await loadPrompts();
    message.success('所有提示词已重置为默认值');
  } catch (error) {
    console.error('重置失败:', error);
    message.error('重置失败');
  }
}

// 重置单个
async function handleReset(promptKey: string) {
  try {
    await promptApi.resetPrompt(promptKey);
    await loadPrompts();
    message.success('提示词已重置');
  } catch (error) {
    console.error('重置失败:', error);
    message.error('重置失败');
  }
}

// 编辑提示词
function handleEdit(prompt: AiPrompt) {
  editingPrompt.value = prompt;
  editForm.value = {
    systemPrompt: prompt.system_prompt,
    userPrompt: prompt.user_prompt,
    enabled: prompt.enabled,
  };
  showEditModal.value = true;
}

// 插入变量
function insertVariable(variable: string) {
  editForm.value.userPrompt += `{${variable}}`;
}

// 保存编辑
async function handleSave() {
  if (!editingPrompt.value) return;

  if (!editForm.value.userPrompt.trim()) {
    message.warning('用户提示词不能为空');
    return;
  }

  saving.value = true;
  try {
    await promptApi.updatePrompt(editingPrompt.value.prompt_key, {
      systemPrompt: editForm.value.systemPrompt,
      userPrompt: editForm.value.userPrompt,
      enabled: editForm.value.enabled,
    });
    await loadPrompts();
    showEditModal.value = false;
    message.success('保存成功');
  } catch (error) {
    console.error('保存失败:', error);
    message.error('保存失败');
  } finally {
    saving.value = false;
  }
}

// 预览提示词
function handlePreview(prompt: AiPrompt) {
  previewingPrompt.value = prompt;
  renderedPreview.value = null;

  // 初始化测试变量
  const vars = parseVariables(prompt.variables);
  testVariables.value = {};
  for (const v of vars) {
    testVariables.value[v] = '';
  }

  showPreviewModal.value = true;
}

// 渲染预览
async function handleRenderPreview() {
  if (!previewingPrompt.value) return;

  rendering.value = true;
  try {
    renderedPreview.value = await promptApi.renderPromptPreview(
      previewingPrompt.value.prompt_key,
      testVariables.value
    );
  } catch (error) {
    console.error('渲染失败:', error);
    message.error('渲染失败');
  } finally {
    rendering.value = false;
  }
}

onMounted(() => {
  loadPrompts();
});
</script>

<style scoped>
.prompt-manager {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* 工具栏 */
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

/* 加载状态 */
.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 48px;
  color: var(--text-muted);
}

/* 提示词列表 */
.prompt-list {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.prompt-group {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.group-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

/* 提示词卡片 */
.prompt-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.prompt-card {
  background: var(--card-bg);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  transition: all 0.2s;
}

.prompt-card:hover {
  border-color: color-mix(in srgb, var(--accent-primary) 50%, transparent);
  box-shadow: 0 4px 12px var(--shadow-color);
}

.prompt-card.prompt-disabled {
  opacity: 0.6;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.card-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.card-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.card-actions {
  display: flex;
  gap: 4px;
}

.card-description {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.5;
}

.card-key {
  margin-top: auto;
}

.card-key code {
  font-size: 11px;
  background: var(--bg-surface);
  padding: 4px 8px;
  border-radius: 4px;
  color: var(--accent-primary);
  font-family: 'Monaco', 'Menlo', monospace;
}

.card-variables {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.variables-label {
  font-size: 12px;
  color: var(--text-dim);
}

.variable-tag {
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 11px;
}

.variable-tag.clickable {
  cursor: pointer;
}

.variable-tag.clickable:hover {
  opacity: 0.8;
}

/* 编辑表单 */
.edit-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.variables-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.variables-hint {
  font-size: 12px;
  color: var(--text-dim);
  margin-top: 4px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

/* 预览内容 */
.preview-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.preview-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.preview-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted);
}

.preview-text {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  padding: 12px;
  margin: 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 200px;
  overflow-y: auto;
}

.preview-text.rendered {
  background: var(--accent-glow);
  border-color: color-mix(in srgb, var(--accent-primary) 30%, transparent);
}

.preview-variables {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: var(--bg-surface);
  border-radius: 8px;
}

.test-variables {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.test-variable-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.test-var-name {
  font-size: 13px;
  color: var(--text-muted);
  min-width: 120px;
  font-family: 'Monaco', 'Menlo', monospace;
}

/* 滚动条 */
.preview-text::-webkit-scrollbar {
  width: 6px;
}

.preview-text::-webkit-scrollbar-track {
  background: transparent;
}

.preview-text::-webkit-scrollbar-thumb {
  background: var(--border-default);
  border-radius: 3px;
}

.preview-text::-webkit-scrollbar-thumb:hover {
  background: var(--border-hover);
}
</style>
