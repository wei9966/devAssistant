<template>
  <n-modal
    v-model:show="modalVisible"
    :mask-closable="true"
    :close-on-esc="true"
    :auto-focus="true"
    class="quick-task-modal"
    transform-origin="center"
  >
    <div class="modal-container" @keydown="handleKeyDown">
      <!-- 核心模态框 -->
      <div
        class="modal-content"
        :class="{ expanded: isExpanded }"
      >
        <!-- 顶部：极简输入区 (Spotlight Area) -->
        <div class="input-section" :class="{ 'has-border': isExpanded }">
          <!-- 左侧图标 (状态指示器) -->
          <div class="input-icon">
            <n-icon size="24" color="#fff">
              <AddOutline />
            </n-icon>
          </div>

          <!-- 输入框核心 -->
          <div class="input-wrapper">
            <input
              ref="inputRef"
              v-model="inputValue"
              type="text"
              class="task-input"
              :class="{ 'is-loading': isCreating }"
              :placeholder="isCreating ? '正在创建任务...' : '输入任务标题... (尝试输入 #前端 !高优)'"
              :disabled="isCreating"
              @keydown.enter="handleQuickCreate"
            />

            <!-- 智能解析展示区 (当输入框有值时显示) -->
            <div v-if="inputValue && (detectedTags.length > 0 || hasHighPriority)" class="parsed-tags">
              <span
                v-for="tag in detectedTags"
                :key="tag"
                class="parsed-tag tag-normal"
              >
                <n-icon size="10"><PricetagOutline /></n-icon>
                {{ tag }}
              </span>
              <span v-if="hasHighPriority" class="parsed-tag tag-priority">
                <n-icon size="10"><ArrowUpOutline /></n-icon>
                高优先级
              </span>
            </div>
          </div>

          <!-- 右侧操作区 -->
          <div class="input-actions">
            <!-- Loading 指示器 -->
            <div v-if="isCreating" class="loading-indicator">
              <n-icon size="16" class="spinner"><SyncOutline /></n-icon>
              <span>创建中...</span>
            </div>
            <div v-else class="hint-badge">
              <kbd>↵</kbd> 创建
            </div>

            <div class="divider"></div>

            <button
              class="expand-btn"
              :class="{ active: isExpanded }"
              @click="toggleExpand"
              :title="isExpanded ? '收起详情' : '展开详情'"
            >
              <n-icon size="20">
                <ContractOutline v-if="isExpanded" />
                <ExpandOutline v-else />
              </n-icon>
            </button>

            <button
              class="close-btn"
              @click="handleClose"
              title="关闭 (Esc)"
            >
              <n-icon size="20"><CloseOutline /></n-icon>
            </button>
          </div>
        </div>

        <!-- 下半部分：扩展详情区 (Expanded Area) -->
        <div class="expanded-section" :class="{ show: isExpanded }">
          <div class="expanded-content">
            <!-- 左列：描述与基础信息 -->
            <div class="left-column">
              <div class="form-group">
                <label class="form-label">描述</label>
                <textarea
                  v-model="description"
                  class="form-textarea"
                  placeholder="添加更详细的任务描述、代码片段或备注..."
                  rows="4"
                ></textarea>
              </div>

              <!-- 快速分类 (Pills) -->
              <div class="form-group">
                <label class="form-label">分类</label>
                <div class="category-pills">
                  <button
                    v-for="cat in categories"
                    :key="cat.value"
                    class="category-pill"
                    :class="{ active: selectedCategory === cat.value }"
                    @click="selectedCategory = cat.value"
                  >
                    <n-icon size="14"><component :is="cat.icon" /></n-icon>
                    {{ cat.label }}
                  </button>
                </div>
              </div>
            </div>

            <!-- 右列：四象限选择器 -->
            <div class="right-column">
              <label class="form-label">
                优先级矩阵
                <span class="form-label-hint" @click="showQuadrantHelp = true">什么是这个?</span>
              </label>

              <div class="quadrant-grid">
                <!-- 象限 1: 紧急且重要 -->
                <button
                  class="quadrant-btn q1"
                  :class="{ active: selectedQuadrant === 'urgent_important' }"
                  @click="selectedQuadrant = 'urgent_important'"
                >
                  <div class="quadrant-header">
                    <n-icon size="16"><FlashOutline /></n-icon>
                    <n-icon v-if="selectedQuadrant === 'urgent_important'" size="14" class="check-icon">
                      <CheckmarkCircleOutline />
                    </n-icon>
                  </div>
                  <span class="quadrant-label">紧急且重要</span>
                </button>

                <!-- 象限 2: 重要不紧急 -->
                <button
                  class="quadrant-btn q2"
                  :class="{ active: selectedQuadrant === 'not_urgent_important' }"
                  @click="selectedQuadrant = 'not_urgent_important'"
                >
                  <div class="quadrant-header">
                    <n-icon size="16"><TimeOutline /></n-icon>
                    <n-icon v-if="selectedQuadrant === 'not_urgent_important'" size="14" class="check-icon">
                      <CheckmarkCircleOutline />
                    </n-icon>
                  </div>
                  <span class="quadrant-label">重要不紧急</span>
                </button>

                <!-- 象限 3: 紧急不重要 -->
                <button
                  class="quadrant-btn q3"
                  :class="{ active: selectedQuadrant === 'urgent_not_important' }"
                  @click="selectedQuadrant = 'urgent_not_important'"
                >
                  <div class="quadrant-header">
                    <n-icon size="16"><AlertCircleOutline /></n-icon>
                    <n-icon v-if="selectedQuadrant === 'urgent_not_important'" size="14" class="check-icon">
                      <CheckmarkCircleOutline />
                    </n-icon>
                  </div>
                  <span class="quadrant-label">紧急不重要</span>
                </button>

                <!-- 象限 4: 不紧急不重要 -->
                <button
                  class="quadrant-btn q4"
                  :class="{ active: selectedQuadrant === 'not_urgent_not_important' }"
                  @click="selectedQuadrant = 'not_urgent_not_important'"
                >
                  <div class="quadrant-header">
                    <n-icon size="16"><RemoveCircleOutline /></n-icon>
                    <n-icon v-if="selectedQuadrant === 'not_urgent_not_important'" size="14" class="check-icon">
                      <CheckmarkCircleOutline />
                    </n-icon>
                  </div>
                  <span class="quadrant-label">不急不重要</span>
                </button>
              </div>
            </div>
          </div>

          <!-- 底部操作栏 -->
          <div class="footer-section">
            <div class="footer-info">
              <span class="ai-status">
                <span class="status-dot"></span>
                AI 助手已就绪
              </span>
              <span class="system-status">
                <span class="status-dot green"></span>
                系统正常
              </span>
            </div>
            <div class="footer-actions">
              <button class="btn-secondary" @click="handleClose">
                取消 (Esc)
              </button>
              <button class="btn-primary" @click="handleCreate">
                创建任务
                <n-icon size="14"><SendOutline /></n-icon>
              </button>
            </div>
          </div>
        </div>

        <!-- 快捷键提示 (极简模式下显示) -->
        <div v-if="!isExpanded" class="keyboard-hints">
          <span><kbd>↑</kbd><kbd>↓</kbd> 选择</span>
          <span><kbd>Tab</kbd> 展开</span>
          <span><kbd>Esc</kbd> 关闭</span>
        </div>
      </div>

      <!-- 底部提示 -->
      <div class="modal-footer-hint">
        <p>DevAssistant Quick Task v2.0</p>
        <p>点击右上角图标或使用 Tab 键切换焦点</p>
      </div>
    </div>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { NModal, NIcon, useMessage } from 'naive-ui';
import {
  AddOutline,
  PricetagOutline,
  ArrowUpOutline,
  ExpandOutline,
  ContractOutline,
  CloseOutline,
  FlashOutline,
  TimeOutline,
  AlertCircleOutline,
  RemoveCircleOutline,
  CheckmarkCircleOutline,
  SendOutline,
  CodeOutline,
  ServerOutline,
  GridOutline,
  DocumentTextOutline,
  SyncOutline,
} from '@vicons/ionicons5';
import { taskApi } from '@/api/taskApi';
import type { TaskQuadrant } from '@/types/task';

const props = withDefaults(
  defineProps<{
    show: boolean;
  }>(),
  {}
);

const emit = defineEmits<{
  'update:show': [value: boolean];
  'created': [taskId: number];
}>();

const message = useMessage();
const inputRef = ref<HTMLInputElement>();

// 状态
const inputValue = ref('');
const description = ref('');
const isExpanded = ref(false);
const selectedCategory = ref<string>('other');
const selectedQuadrant = ref<TaskQuadrant>('not_urgent_important');
const showQuadrantHelp = ref(false);
const isCreating = ref(false);

// 分类选项
const categories = [
  { value: 'backend', label: '后端', icon: CodeOutline },
  { value: 'database', label: '数据库', icon: ServerOutline },
  { value: 'feature', label: '前端', icon: GridOutline },
  { value: 'docs', label: '文档', icon: DocumentTextOutline },
];

// 计算属性
const modalVisible = computed({
  get: () => props.show,
  set: (val) => emit('update:show', val),
});

// 智能解析标签
const detectedTags = computed(() => {
  const tags = inputValue.value.match(/#[\w\u4e00-\u9fa5]+/g);
  if (tags) {
    return tags.map(t => t.replace('#', ''));
  }
  return [];
});

// 检测高优先级标记
const hasHighPriority = computed(() => {
  return inputValue.value.includes('!');
});

// 根据解析结果自动设置分类
watch(detectedTags, (tags) => {
  for (const tag of tags) {
    const lowerTag = tag.toLowerCase();
    if (['前端', 'frontend', 'vue', 'react', 'ui'].includes(lowerTag)) {
      selectedCategory.value = 'feature';
      break;
    } else if (['后端', 'backend', 'api', 'server', 'java', 'rust'].includes(lowerTag)) {
      selectedCategory.value = 'backend';
      break;
    } else if (['数据库', 'database', 'db', 'sql', 'mysql'].includes(lowerTag)) {
      selectedCategory.value = 'database';
      break;
    } else if (['文档', 'docs', 'doc', 'readme'].includes(lowerTag)) {
      selectedCategory.value = 'docs';
      break;
    }
  }
});

// 根据高优先级标记设置四象限
watch(hasHighPriority, (hasPriority) => {
  if (hasPriority) {
    selectedQuadrant.value = 'urgent_important';
  }
});

// 方法
const toggleExpand = () => {
  isExpanded.value = !isExpanded.value;
};

const handleClose = () => {
  modalVisible.value = false;
  resetForm();
};

const resetForm = () => {
  inputValue.value = '';
  description.value = '';
  isExpanded.value = false;
  selectedCategory.value = 'other';
  selectedQuadrant.value = 'not_urgent_important';
  isCreating.value = false;
};

// 快速创建（极简模式）
const handleQuickCreate = async () => {
  if (!inputValue.value.trim()) {
    message.warning('请输入任务标题');
    return;
  }

  if (isCreating.value) return;
  isCreating.value = true;

  try {
    // 清理标题中的标签
    const cleanTitle = inputValue.value
      .replace(/#[\w\u4e00-\u9fa5]+/g, '')
      .replace(/!/g, '')
      .trim();

    const priority = hasHighPriority.value ? 1 : 2;

    const taskId = await taskApi.createTask(
      cleanTitle,
      undefined,
      selectedCategory.value,
      priority,
      selectedQuadrant.value
    );

    message.success('任务已快速创建!');
    emit('created', taskId);
    handleClose();
  } catch (error) {
    console.error('创建任务失败:', error);
    message.error('创建任务失败');
  } finally {
    isCreating.value = false;
  }
};

// 完整创建（展开模式）
const handleCreate = async () => {
  if (!inputValue.value.trim()) {
    message.warning('请输入任务标题');
    return;
  }

  if (isCreating.value) return;
  isCreating.value = true;

  try {
    // 清理标题中的标签
    const cleanTitle = inputValue.value
      .replace(/#[\w\u4e00-\u9fa5]+/g, '')
      .replace(/!/g, '')
      .trim();

    const priority = hasHighPriority.value ? 1 : 2;

    const taskId = await taskApi.createTask(
      cleanTitle,
      description.value || undefined,
      selectedCategory.value,
      priority,
      selectedQuadrant.value
    );

    message.success('任务创建成功!');
    emit('created', taskId);
    handleClose();
  } catch (error) {
    console.error('创建任务失败:', error);
    message.error('创建任务失败');
  } finally {
    isCreating.value = false;
  }
};

// 键盘事件处理
const handleKeyDown = (event: KeyboardEvent) => {
  if (event.key === 'Tab') {
    event.preventDefault();
    toggleExpand();
  }
};

// 监听显示状态，自动聚焦
watch(modalVisible, (visible) => {
  if (visible) {
    nextTick(() => {
      inputRef.value?.focus();
    });
  } else {
    resetForm();
  }
});
</script>

<style scoped>
.quick-task-modal {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 15vh;
}

.quick-task-modal :deep(.n-modal) {
  max-width: 680px;
  width: 90%;
  margin: 0;
  background: transparent;
  box-shadow: none;
}

.modal-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
}

.modal-content {
  width: 100%;
  background: rgba(15, 23, 42, 0.98);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(139, 92, 246, 0.3);
  border-radius: 20px;
  box-shadow:
    0 25px 50px -12px rgba(0, 0, 0, 0.5),
    0 0 0 1px rgba(139, 92, 246, 0.2),
    0 0 60px rgba(139, 92, 246, 0.1);
  overflow: hidden;
  transition: all 0.3s ease;
}

.modal-content.expanded {
  min-height: 500px;
}

/* 输入区样式 */
.input-section {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px 24px;
  border-bottom: 1px solid transparent;
  transition: border-color 0.3s ease;
}

.input-section.has-border {
  border-bottom-color: rgba(51, 65, 85, 0.5);
}

.input-icon {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #8b5cf6 0%, #6366f1 100%);
  border-radius: 12px;
  box-shadow: 0 8px 20px rgba(139, 92, 246, 0.3);
  flex-shrink: 0;
}

.input-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-input {
  width: 100%;
  background: transparent;
  border: none;
  outline: none;
  font-size: 20px;
  color: #e2e8f0;
  font-weight: 400;
  height: 48px;
  font-family: inherit;
}

.task-input::placeholder {
  color: #64748b;
}

.task-input.is-loading {
  opacity: 0.6;
  cursor: not-allowed;
}

.task-input:disabled {
  background: transparent;
}

.parsed-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.parsed-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 500;
}

.parsed-tag.tag-normal {
  background: rgba(139, 92, 246, 0.2);
  color: #c4b5fd;
  border: 1px solid rgba(139, 92, 246, 0.3);
}

.parsed-tag.tag-priority {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.input-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.hint-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #64748b;
  padding: 6px 12px;
  background: rgba(51, 65, 85, 0.5);
  border: 1px solid rgba(71, 85, 105, 0.6);
  border-radius: 8px;
}

.hint-badge kbd {
  font-size: 11px;
  font-family: monospace;
}

/* Loading 指示器样式 */
.loading-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #a78bfa;
  padding: 6px 14px;
  background: rgba(139, 92, 246, 0.15);
  border: 1px solid rgba(139, 92, 246, 0.3);
  border-radius: 8px;
  animation: pulse-bg 1.5s ease-in-out infinite;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@keyframes pulse-bg {
  0%, 100% {
    background: rgba(139, 92, 246, 0.15);
  }
  50% {
    background: rgba(139, 92, 246, 0.25);
  }
}

.divider {
  width: 1px;
  height: 24px;
  background: rgba(51, 65, 85, 0.8);
  margin: 0 4px;
}

.expand-btn,
.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  color: #64748b;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.expand-btn:hover,
.close-btn:hover {
  background: rgba(51, 65, 85, 0.5);
  color: #e2e8f0;
}

.expand-btn.active {
  background: rgba(139, 92, 246, 0.2);
  color: #a78bfa;
}

/* 展开区域样式 */
.expanded-section {
  max-height: 0;
  opacity: 0;
  overflow: hidden;
  transition: all 0.4s ease;
}

.expanded-section.show {
  max-height: 600px;
  opacity: 1;
}

.expanded-content {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 32px;
  padding: 24px;
}

.left-column,
.right-column {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.form-label {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.form-label-hint {
  font-size: 10px;
  color: #a78bfa;
  cursor: pointer;
  text-transform: none;
  letter-spacing: normal;
}

.form-label-hint:hover {
  text-decoration: underline;
}

.form-textarea {
  width: 100%;
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.8);
  border-radius: 12px;
  padding: 14px;
  color: #cbd5e1;
  font-size: 14px;
  font-family: inherit;
  resize: none;
  outline: none;
  transition: all 0.2s ease;
}

.form-textarea::placeholder {
  color: #475569;
}

.form-textarea:focus {
  border-color: rgba(139, 92, 246, 0.5);
  box-shadow: 0 0 0 3px rgba(139, 92, 246, 0.1);
}

/* 分类按钮 */
.category-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.category-pill {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  background: rgba(51, 65, 85, 0.3);
  border: 1px solid rgba(51, 65, 85, 0.6);
  border-radius: 10px;
  color: #94a3b8;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.category-pill:hover {
  background: rgba(51, 65, 85, 0.5);
  border-color: rgba(139, 92, 246, 0.4);
  color: #e2e8f0;
}

.category-pill.active {
  background: rgba(139, 92, 246, 0.15);
  border-color: rgba(139, 92, 246, 0.5);
  color: #c4b5fd;
}

/* 四象限选择器 */
.quadrant-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  height: 200px;
}

.quadrant-btn {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 14px;
  border-radius: 12px;
  border: 1px solid rgba(51, 65, 85, 0.6);
  background: rgba(51, 65, 85, 0.2);
  cursor: pointer;
  transition: all 0.2s ease;
}

.quadrant-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.quadrant-label {
  font-size: 12px;
  font-weight: 500;
}

.check-icon {
  flex-shrink: 0;
}

/* Q1: 紧急且重要 - 红色 */
.quadrant-btn.q1 {
  color: #64748b;
}
.quadrant-btn.q1:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.4);
}
.quadrant-btn.q1.active {
  background: rgba(239, 68, 68, 0.15);
  border-color: rgba(239, 68, 68, 0.5);
  color: #fca5a5;
  box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.1);
}
.quadrant-btn.q1.active .check-icon {
  color: #ef4444;
}

/* Q2: 重要不紧急 - 橙色 */
.quadrant-btn.q2 {
  color: #64748b;
}
.quadrant-btn.q2:hover {
  background: rgba(249, 115, 22, 0.1);
  border-color: rgba(249, 115, 22, 0.4);
}
.quadrant-btn.q2.active {
  background: rgba(249, 115, 22, 0.15);
  border-color: rgba(249, 115, 22, 0.5);
  color: #fdba74;
  box-shadow: 0 0 0 3px rgba(249, 115, 22, 0.1);
}
.quadrant-btn.q2.active .check-icon {
  color: #f97316;
}

/* Q3: 紧急不重要 - 蓝色 */
.quadrant-btn.q3 {
  color: #64748b;
}
.quadrant-btn.q3:hover {
  background: rgba(59, 130, 246, 0.1);
  border-color: rgba(59, 130, 246, 0.4);
}
.quadrant-btn.q3.active {
  background: rgba(59, 130, 246, 0.15);
  border-color: rgba(59, 130, 246, 0.5);
  color: #93c5fd;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}
.quadrant-btn.q3.active .check-icon {
  color: #3b82f6;
}

/* Q4: 不紧急不重要 - 灰色 */
.quadrant-btn.q4 {
  color: #64748b;
}
.quadrant-btn.q4:hover {
  background: rgba(100, 116, 139, 0.2);
  border-color: rgba(100, 116, 139, 0.5);
}
.quadrant-btn.q4.active {
  background: rgba(100, 116, 139, 0.25);
  border-color: rgba(100, 116, 139, 0.6);
  color: #cbd5e1;
  box-shadow: 0 0 0 3px rgba(100, 116, 139, 0.1);
}
.quadrant-btn.q4.active .check-icon {
  color: #94a3b8;
}

/* 底部操作栏 */
.footer-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: rgba(15, 23, 42, 0.5);
  border-top: 1px solid rgba(51, 65, 85, 0.5);
}

.footer-info {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 12px;
  color: #64748b;
}

.ai-status,
.system-status {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #6366f1;
  box-shadow: 0 0 8px rgba(99, 102, 241, 0.6);
}

.status-dot.green {
  background: #22c55e;
  box-shadow: 0 0 8px rgba(34, 197, 94, 0.6);
}

.footer-actions {
  display: flex;
  gap: 12px;
}

.btn-secondary {
  padding: 10px 20px;
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 14px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-secondary:hover {
  background: rgba(51, 65, 85, 0.5);
  color: #e2e8f0;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 24px;
  background: linear-gradient(135deg, #8b5cf6 0%, #6366f1 100%);
  border: none;
  color: #fff;
  font-size: 14px;
  font-weight: 500;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 8px 20px rgba(139, 92, 246, 0.25);
}

.btn-primary:hover {
  background: linear-gradient(135deg, #a78bfa 0%, #818cf8 100%);
  transform: translateY(-1px);
  box-shadow: 0 10px 25px rgba(139, 92, 246, 0.35);
}

/* 快捷键提示 */
.keyboard-hints {
  display: flex;
  justify-content: center;
  gap: 20px;
  padding: 14px 24px;
  border-top: 1px solid rgba(51, 65, 85, 0.4);
  font-size: 11px;
  color: #475569;
}

.keyboard-hints kbd {
  display: inline-block;
  padding: 3px 8px;
  background: rgba(51, 65, 85, 0.5);
  border: 1px solid rgba(71, 85, 105, 0.6);
  border-radius: 5px;
  font-size: 10px;
  font-family: monospace;
  margin: 0 3px;
}

/* 底部提示 */
.modal-footer-hint {
  text-align: center;
  font-size: 11px;
  color: #475569;
  line-height: 1.6;
}

/* 响应式 */
@media (max-width: 768px) {
  .expanded-content {
    grid-template-columns: 1fr;
    gap: 24px;
  }

  .quadrant-grid {
    height: auto;
  }

  .hint-badge {
    display: none;
  }
}
</style>
