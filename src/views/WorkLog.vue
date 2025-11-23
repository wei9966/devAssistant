<template>
  <div class="worklog-page">
    <!-- 左右分栏布局 -->
    <div class="worklog-container">
      <!-- 左侧编辑器区域 -->
      <div class="editor-section">
        <!-- 顶部标题和操作按钮 -->
        <div class="header-actions">
          <h2 class="page-title">工作日志</h2>
          <div class="action-buttons">
            <n-button
              type="success"
              secondary
              @click="handleGenerateWeekly"
              :loading="generating"
              class="ai-button"
            >
              <template #icon>
                <n-icon>
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 3v18m0-18l4 4m-4-4L8 7" />
                    <path d="M12 21l4-4m-4 4l-4-4" opacity="0.5" />
                  </svg>
                </n-icon>
              </template>
              AI 生成周报
            </n-button>
            <n-button secondary @click="handleArchive">
              历史归档
            </n-button>
          </div>
        </div>

        <!-- 编辑器卡片 -->
        <div class="editor-card">
          <!-- 顶部：日期和图标 -->
          <div class="editor-header">
            <div class="date-info">
              <div class="icon-box">
                <n-icon size="20" color="#a78bfa">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
                    <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
                  </svg>
                </n-icon>
              </div>
              <div class="date-content">
                <div class="current-date">{{ formatDateFull(selectedDate) }}</div>
                <div class="date-subtitle">{{ getDateSubtitle() }}</div>
              </div>
            </div>
            <n-date-picker
              v-model:value="selectedDate"
              type="date"
              clearable
              class="date-picker"
            />
            <n-button
              text
              @click="handleSave"
              :loading="saving"
              class="save-button"
            >
              <template #icon>
                <n-icon size="20">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
                    <polyline points="17 21 17 13 7 13 7 21" />
                    <polyline points="7 3 7 8 15 8" />
                  </svg>
                </n-icon>
              </template>
            </n-button>
          </div>

          <!-- 中间：可扩展的textarea -->
          <n-spin :show="workLogStore.loading">
            <textarea
              v-model="currentLog"
              class="editor-textarea"
              placeholder="今天做了什么？无论是写代码、开会还是摸鱼，记录下来吧..."
            />
          </n-spin>

          <!-- 底部：标签列表 -->
          <div class="editor-footer">
            <div class="tags-container">
              <n-tag
                v-for="tag in currentTags"
                :key="tag"
                :bordered="false"
                class="log-tag"
                closable
                @close="handleRemoveTag(tag)"
              >
                {{ tag }}
              </n-tag>
              <n-button
                text
                size="small"
                @click="showAddTag = true"
                class="add-tag-btn"
              >
                + 添加标签
              </n-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 右侧时间轴（大屏显示） -->
      <div class="timeline-section">
        <h3 class="timeline-title">最近记录</h3>
        <div class="timeline-container">
          <!-- 垂直线 -->
          <div class="timeline-line" />

          <!-- 时间节点 -->
          <div
            v-for="(log, index) in workLogStore.recentLogs"
            :key="log.date"
            class="timeline-item"
            :class="{ 'timeline-item-active': isToday(log.date) }"
            @click="selectLog(log)"
          >
            <!-- 圆点 -->
            <div
              class="timeline-dot"
              :class="{
                'dot-active': isToday(log.date),
                'dot-default': !isToday(log.date)
              }"
            />
            <!-- 内容 -->
            <div class="timeline-content">
              <div class="timeline-date">{{ formatDate(log.date) }}</div>
              <div class="timeline-text">
                {{ truncate(log.content, 60) || '暂无内容' }}
              </div>
            </div>
          </div>

          <!-- 空状态 -->
          <div v-if="workLogStore.recentLogs.length === 0" class="timeline-empty">
            <n-empty description="暂无历史记录" size="small" />
          </div>
        </div>
      </div>
    </div>

    <!-- 添加标签对话框 -->
    <n-modal v-model:show="showAddTag" preset="dialog" title="添加标签">
      <n-input
        v-model:value="newTag"
        placeholder="输入标签名称，如 #Frontend"
        @keyup.enter="handleAddTag"
      />
      <template #action>
        <n-button @click="showAddTag = false">取消</n-button>
        <n-button class="primary-button" @click="handleAddTag">添加</n-button>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, onBeforeUnmount } from 'vue';
import {
  NButton,
  NDatePicker,
  NSpin,
  NTag,
  NEmpty,
  NModal,
  NInput,
  NIcon,
  useMessage,
  useDialog
} from 'naive-ui';
import dayjs from 'dayjs';
import { useWorkLogStore } from '@/stores/workLogStore';
import type { WorkLog } from '@/types/workLog';

const message = useMessage();
const dialog = useDialog();
const workLogStore = useWorkLogStore();

const selectedDate = ref<number>(Date.now());
const currentLog = ref('');
const currentTags = ref<string[]>(['#Rust', '#Frontend', '#Bugfix']);
const saving = ref(false);
const generating = ref(false);
const showAddTag = ref(false);
const newTag = ref('');

// LocalStorage 键名
const DRAFT_STORAGE_KEY = 'worklog_draft';
const TAGS_STORAGE_KEY = 'worklog_tags';

// 保存草稿到 LocalStorage
function saveDraft() {
  const dateStr = dayjs(selectedDate.value).format('YYYY-MM-DD');
  const draft = {
    date: dateStr,
    content: currentLog.value,
    tags: currentTags.value,
    timestamp: Date.now()
  };
  localStorage.setItem(DRAFT_STORAGE_KEY, JSON.stringify(draft));
}

// 从 LocalStorage 加载草稿
function loadDraft() {
  try {
    const draftStr = localStorage.getItem(DRAFT_STORAGE_KEY);
    if (draftStr) {
      const draft = JSON.parse(draftStr);
      const dateStr = dayjs(selectedDate.value).format('YYYY-MM-DD');

      // 只有当日期匹配时才加载草稿
      if (draft.date === dateStr && draft.content) {
        // 如果当前没有内容,使用草稿
        if (!currentLog.value) {
          currentLog.value = draft.content;
          if (draft.tags && draft.tags.length > 0) {
            currentTags.value = draft.tags;
          }
        }
      }
    }
  } catch (error) {
    console.error('加载草稿失败:', error);
  }
}

// 清除草稿
function clearDraft() {
  localStorage.removeItem(DRAFT_STORAGE_KEY);
}

onMounted(async () => {
  await loadLogs();
  // 加载草稿
  loadDraft();
});

// 监听日期变化,自动加载对应日志
watch(selectedDate, async (newDate) => {
  if (newDate) {
    const dateStr = dayjs(newDate).format('YYYY-MM-DD');
    await workLogStore.loadWorkLog(dateStr);
    currentLog.value = workLogStore.currentLog?.content || '';
    // 加载草稿
    loadDraft();
  }
});

// 监听内容变化,自动保存草稿
watch(currentLog, () => {
  if (currentLog.value) {
    saveDraft();
  }
});

// 监听标签变化,自动保存草稿
watch(currentTags, () => {
  saveDraft();
}, { deep: true });

// 页面卸载前保存草稿
onBeforeUnmount(() => {
  if (currentLog.value) {
    saveDraft();
  }
});

async function loadLogs() {
  try {
    // 加载最近7天的日志
    await workLogStore.loadRecentLogs(7);

    // 加载当前选中日期的日志
    const dateStr = dayjs(selectedDate.value).format('YYYY-MM-DD');
    await workLogStore.loadWorkLog(dateStr);
    currentLog.value = workLogStore.currentLog?.content || '';
  } catch (error) {
    message.error('加载日志失败');
    console.error(error);
  }
}

async function handleSave() {
  if (!currentLog.value.trim()) {
    message.warning('日志内容不能为空');
    return;
  }

  saving.value = true;
  try {
    const dateStr = dayjs(selectedDate.value).format('YYYY-MM-DD');
    await workLogStore.saveWorkLog(dateStr, 'daily', currentLog.value, false);
    message.success('日志已保存');
    // 清除草稿
    clearDraft();
    // 重新加载最近日志
    await workLogStore.loadRecentLogs(7);
  } catch (error) {
    message.error('保存失败');
    console.error(error);
  } finally {
    saving.value = false;
  }
}

async function handleGenerateWeekly() {
  generating.value = true;
  try {
    // TODO: 调用AI生成周报
    message.info('周报生成功能开发中');
  } catch (error) {
    message.error('生成失败');
    console.error(error);
  } finally {
    generating.value = false;
  }
}

function handleArchive() {
  message.info('历史归档功能开发中');
}

function selectLog(log: WorkLog) {
  selectedDate.value = new Date(log.date).getTime();
  currentLog.value = log.content;
}

function handleAddTag() {
  if (!newTag.value.trim()) {
    return;
  }
  const tag = newTag.value.startsWith('#') ? newTag.value : `#${newTag.value}`;
  if (!currentTags.value.includes(tag)) {
    currentTags.value.push(tag);
  }
  newTag.value = '';
  showAddTag.value = false;
}

function handleRemoveTag(tag: string) {
  const index = currentTags.value.indexOf(tag);
  if (index > -1) {
    currentTags.value.splice(index, 1);
  }
}

function formatDateFull(timestamp: number) {
  return dayjs(timestamp).format('YYYY年 MM月 DD日');
}

function formatDate(date: string) {
  return dayjs(date).format('YYYY-MM-DD');
}

function getDateSubtitle() {
  const day = dayjs(selectedDate.value).format('dddd');
  const dayMap: { [key: string]: string } = {
    'Monday': '星期一',
    'Tuesday': '星期二',
    'Wednesday': '星期三',
    'Thursday': '星期四',
    'Friday': '星期五',
    'Saturday': '星期六',
    'Sunday': '星期日'
  };
  return `${dayMap[day] || day} · 此时此刻`;
}

function isToday(date: string) {
  return dayjs(date).isSame(dayjs(), 'day');
}

function truncate(text: string, length: number) {
  if (!text) return '';
  return text.length > length ? text.substring(0, length) + '...' : text;
}
</script>

<style scoped>
.worklog-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: linear-gradient(to bottom right, #020617, #0f172a);
  overflow: hidden;
}

.worklog-container {
  flex: 1;
  display: flex;
  gap: 24px;
  padding: 32px;
  padding-top: 16px;
  min-height: 0;
}

/* 左侧编辑器区域 */
.editor-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.header-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.page-title {
  font-size: 20px;
  font-weight: 600;
  color: #f1f5f9;
  letter-spacing: -0.01em;
  margin: 0;
}

.action-buttons {
  display: flex;
  gap: 12px;
}

.ai-button {
  --n-color: rgba(16, 185, 129, 0.1) !important;
  --n-color-hover: rgba(16, 185, 129, 0.2) !important;
  --n-text-color: #10b981 !important;
  --n-border: 1px solid rgba(16, 185, 129, 0.2) !important;
}

.editor-card {
  flex: 1;
  background: rgba(15, 23, 42, 0.4);
  border: 1px solid rgba(148, 163, 184, 0.1);
  border-radius: 20px;
  padding: 24px;
  backdrop-filter: blur(12px);
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  min-height: 0;
  position: relative;
  transition: all 0.3s ease;
}

.editor-card:focus-within {
  border-color: rgba(99, 102, 241, 0.3);
  box-shadow: 0 8px 16px -4px rgba(0, 0, 0, 0.2), 0 0 0 1px rgba(99, 102, 241, 0.1);
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 16px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
  margin-bottom: 16px;
}

.date-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon-box {
  background: rgba(167, 139, 250, 0.1);
  padding: 8px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.date-content {
  display: flex;
  flex-direction: column;
}

.current-date {
  color: #e2e8f0;
  font-weight: 500;
  font-size: 15px;
}

.date-subtitle {
  color: #64748b;
  font-size: 12px;
  margin-top: 2px;
}

.date-picker {
  width: 200px;
}

.save-button {
  color: #94a3b8;
  transition: color 0.2s;
}

.save-button:hover {
  color: #10b981;
}

.editor-textarea {
  flex: 1;
  width: 100%;
  box-sizing: border-box;
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  color: #cbd5e1;
  font-size: 14px;
  line-height: 1.7;
  font-family: inherit;
  padding: 12px;
  min-height: 300px;
  overflow-y: auto;
}

.editor-textarea::placeholder {
  color: #64748b; /* slate-500 */
}

.editor-textarea::-webkit-scrollbar {
  width: 6px;
}

.editor-textarea::-webkit-scrollbar-track {
  background: transparent;
}

.editor-textarea::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 3px;
}

.editor-textarea::-webkit-scrollbar-thumb:hover {
  background: #475569;
}

.editor-footer {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid rgba(148, 163, 184, 0.1);
}

.tags-container {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  align-items: center;
}

.log-tag {
  background: rgba(51, 65, 85, 0.5) !important;
  color: #94a3b8 !important; /* slate-400 */
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.log-tag:hover {
  color: #a78bfa !important;
}

.add-tag-btn {
  color: #94a3b8; /* slate-400 */
  font-size: 12px;
  padding: 4px 8px;
  transition: color 0.2s;
}

.add-tag-btn:hover {
  color: #a78bfa;
}

/* 右侧时间轴 */
.timeline-section {
  width: 280px;
  background: rgba(15, 23, 42, 0.3);
  border-left: 1px solid rgba(148, 163, 184, 0.1);
  padding: 20px;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow: hidden;
}

/* 只在很小的屏幕上隐藏时间轴 */
@media (max-width: 768px) {
  .timeline-section {
    display: none;
  }

  .worklog-container {
    padding: 20px;
  }
}

.timeline-title {
  font-size: 11px;
  font-weight: 700;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  padding: 0 4px;
  margin: 0 0 24px 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.timeline-container {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 24px;
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding-right: 4px;
}

/* 时间轴滚动条样式 */
.timeline-container::-webkit-scrollbar {
  width: 4px;
}

.timeline-container::-webkit-scrollbar-track {
  background: transparent;
}

.timeline-container::-webkit-scrollbar-thumb {
  background: rgba(51, 65, 85, 0.5);
  border-radius: 2px;
}

.timeline-container::-webkit-scrollbar-thumb:hover {
  background: rgba(71, 85, 105, 0.7);
}

.timeline-line {
  position: absolute;
  left: 5px;
  top: 12px;
  bottom: 12px;
  width: 1px;
  background: rgba(51, 65, 85, 0.8);
}

.timeline-item {
  position: relative;
  padding-left: 24px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.timeline-item:hover {
  transform: translateX(2px);
}

.timeline-item:hover .timeline-content {
  background: rgba(30, 41, 59, 0.6);
  border-color: rgba(71, 85, 105, 0.5);
}

.timeline-dot {
  position: absolute;
  left: 2px;
  top: 6px;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  border: 2px solid #0f172a;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 10;
}

.dot-active {
  background: #6366f1;
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.3);
  width: 11px;
  height: 11px;
  left: 0;
  top: 4px;
}

.dot-default {
  background: #1e293b;
}

.timeline-item:hover .dot-default {
  background: #a78bfa;
  border-color: #0f172a;
  box-shadow: 0 0 0 2px rgba(167, 139, 250, 0.2);
}

.timeline-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border-radius: 10px;
  background: rgba(15, 23, 42, 0.4);
  border: 1px solid rgba(51, 65, 85, 0.3);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.timeline-date {
  font-size: 11px;
  color: #64748b;
  font-family: 'Consolas', 'Monaco', monospace;
  font-weight: 500;
}

.timeline-item-active .timeline-date {
  color: #a78bfa;
}

.timeline-text {
  font-size: 13px;
  color: #94a3b8;
  line-height: 1.6;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}

.timeline-item-active .timeline-text {
  color: #cbd5e1;
  font-weight: 400;
}

.timeline-empty {
  padding: 32px 0;
  display: flex;
  justify-content: center;
  align-items: center;
}

/* 主要按钮样式 */
:deep(.primary-button) {
  background-color: #6366f1 !important;
  border-color: #6366f1 !important;
  color: #ffffff !important;
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2) !important;
  transition: all 0.2s !important;
}

:deep(.primary-button:hover) {
  background-color: #4f46e5 !important;
  border-color: #4f46e5 !important;
}

:deep(.primary-button:active) {
  background-color: #4338ca !important;
  border-color: #4338ca !important;
}
</style>
