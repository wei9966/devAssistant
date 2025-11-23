<template>
  <n-card>
    <template #header>
      <div class="sql-header">
        <div class="sql-meta">
          <n-tag :type="getSqlTypeColor(sql.sql_type)" size="small">{{ sql.sql_type }}</n-tag>
          <span class="db-name">{{ sql.database_name }}</span>
          <span class="exec-time">{{ formatTime(sql.executed_at) }}</span>
        </div>
      </div>
    </template>

    <div class="sql-content">
      <n-code :code="sql.sql_text" language="sql" :show-line-numbers="false" />
    </div>

    <div v-if="sql.description || sql.tags" class="sql-extra">
      <div v-if="sql.description" class="description">
        <strong>描述:</strong> {{ sql.description }}
      </div>
      <div v-if="sql.tags && sql.tags.length > 0" class="tags">
        <n-tag v-for="tag in parseTags(sql.tags)" :key="tag" size="small">{{ tag }}</n-tag>
      </div>
    </div>

    <template #footer>
      <n-space justify="space-between">
        <n-space>
          <n-button text class="text-button-primary" @click="handleCopy" title="复制SQL">
            <template #icon>
              <n-icon><CopyOutline /></n-icon>
            </template>
            复制
          </n-button>
          <n-button text class="text-button-primary" @click="toggleFav" :title="isFavorite ? '取消收藏' : '收藏'">
            <template #icon>
              <n-icon>
                <Star v-if="isFavorite" />
                <StarOutline v-else />
              </n-icon>
            </template>
          </n-button>
        </n-space>
        <n-button text type="error" @click="handleDelete" title="删除">
          <template #icon>
            <n-icon><Trash /></n-icon>
          </template>
          删除
        </n-button>
      </n-space>
    </template>
  </n-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { NCard, NTag, NCode, NButton, NIcon, NSpace, useMessage, useDialog } from 'naive-ui';
import { Star, StarOutline, CopyOutline, Trash } from '@vicons/ionicons5';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/zh-cn';
import type { SqlRecord } from '@/types/sql';

dayjs.extend(relativeTime);
dayjs.locale('zh-cn');

const props = defineProps<{
  sql: SqlRecord;
}>();

const emit = defineEmits<{
  copy: [sqlText: string];
  'toggle-favorite': [sqlId: number | undefined];
  delete: [sqlId: number | undefined];
}>();

const message = useMessage();
const dialog = useDialog();
const isFavorite = ref(props.sql.is_favorite || false);

const handleCopy = () => {
  navigator.clipboard.writeText(props.sql.sql_text).then(() => {
    message.success('已复制到剪贴板');
    emit('copy', props.sql.sql_text);
  });
};

const toggleFav = () => {
  isFavorite.value = !isFavorite.value;
  emit('toggle-favorite', props.sql.id);
};

const handleDelete = () => {
  dialog.warning({
    title: '删除SQL',
    content: '确定要删除这条SQL记录吗？',
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: () => {
      emit('delete', props.sql.id);
      message.success('SQL已删除');
    },
  });
};

const formatTime = (time: string) => {
  return dayjs(time).fromNow();
};

const getSqlTypeColor = (type: string) => {
  const colorMap: Record<string, 'success' | 'info' | 'warning' | 'error'> = {
    'SELECT': 'info',
    'INSERT': 'success',
    'UPDATE': 'warning',
    'DELETE': 'error',
    'CREATE': 'success',
    'ALTER': 'warning',
    'DROP': 'error',
  };
  return colorMap[type] || 'default';
};

const parseTags = (tags: string | string[]): string[] => {
  if (Array.isArray(tags)) return tags;
  if (typeof tags === 'string') {
    try {
      return JSON.parse(tags);
    } catch {
      return tags.split(',').map((t) => t.trim()).filter(Boolean);
    }
  }
  return [];
};
</script>

<style scoped>
.sql-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.sql-meta {
  display: flex;
  align-items: center;
  gap: 12px;
}

.db-name {
  color: #cbd5e1; /* slate-300 */
  font-size: 14px;
  font-family: monospace;
}

.exec-time {
  color: #94a3b8; /* slate-400 */
  font-size: 12px;
}

.sql-content {
  margin: 12px 0;
  background: rgba(15, 23, 42, 0.5); /* 深色背景 */
  border-radius: 4px;
  padding: 8px;
}

.sql-extra {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid rgba(51, 65, 85, 0.5);
}

.description {
  margin-bottom: 8px;
  font-size: 12px;
  color: #cbd5e1; /* slate-300 */
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

/* 文字按钮样式 */
:deep(.text-button-primary) {
  color: #6366f1;
  transition: color 0.2s;
}

:deep(.text-button-primary:hover) {
  color: #818cf8;
}
</style>
