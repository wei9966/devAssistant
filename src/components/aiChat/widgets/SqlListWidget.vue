<template>
  <div class="widget-container sql-list-widget">
    <div class="widget-header">
      <div class="widget-title">
        <n-icon size="20" class="widget-icon">
          <CodeOutline />
        </n-icon>
        <span>SQL 列表</span>
      </div>
      <div class="widget-stats">
        共 {{ data.total }} 条记录
      </div>
    </div>

    <div class="widget-content">
      <div v-if="data.sql_list.length > 0" class="sql-list">
        <div v-for="sql in data.sql_list" :key="sql.id" class="sql-item">
          <div class="sql-item-header">
            <div class="sql-type-badge" :class="`type-${sql.sql_type.toLowerCase()}`">
              {{ sql.sql_type }}
            </div>
            <div class="sql-item-meta">
              <n-icon size="12" class="meta-icon">
                <TimeOutline />
              </n-icon>
              <span>{{ formatDate(sql.created_at) }}</span>
            </div>
            <div v-if="sql.is_favorite" class="favorite-indicator">
              <n-icon size="14" :color="starColor">
                <Star />
              </n-icon>
            </div>
          </div>

          <div class="sql-content">
            <pre class="sql-code">{{ sql.content }}</pre>
          </div>

          <div v-if="sql.tables && sql.tables.length > 0" class="sql-tables">
            <n-icon size="12" class="table-icon">
              <GridOutline />
            </n-icon>
            <span class="table-list">{{ sql.tables.join(', ') }}</span>
          </div>

          <div v-if="sql.note" class="sql-note">
            <n-icon size="12">
              <DocumentTextOutline />
            </n-icon>
            <span>{{ sql.note }}</span>
          </div>

          <div class="sql-actions">
            <n-button size="tiny" @click="handleCopy(sql.content)">
              <template #icon>
                <n-icon><CopyOutline /></n-icon>
              </template>
              复制
            </n-button>
            <n-button
              size="tiny"
              :type="sql.is_favorite ? 'warning' : 'default'"
              @click="handleFavorite(sql)"
            >
              <template #icon>
                <n-icon>
                  <component :is="sql.is_favorite ? Star : StarOutline" />
                </n-icon>
              </template>
              {{ sql.is_favorite ? '取消收藏' : '收藏' }}
            </n-button>
          </div>
        </div>
      </div>

      <div v-else class="empty-state">
        <n-icon size="48" class="empty-icon">
          <CodeOutline />
        </n-icon>
        <p>暂无 SQL 记录</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { NButton, NIcon, useMessage } from 'naive-ui';
import {
  CodeOutline,
  TimeOutline,
  Star,
  StarOutline,
  CopyOutline,
  GridOutline,
  DocumentTextOutline,
} from '@vicons/ionicons5';
import dayjs from 'dayjs';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

interface SqlData {
  id: string | number;
  content: string;
  sql_type: string;
  tables?: string[];
  is_favorite: boolean;
  created_at: string;
  note?: string;
}

interface SqlSearchResponse {
  total: number;
  sql_list: SqlData[];
}

const props = defineProps<{
  data: SqlSearchResponse;
}>();

const emit = defineEmits<{
  copy: [content: string];
  favorite: [sql: SqlData];
}>();

const message = useMessage();
const starColor = 'var(--warning)'; // 使用 CSS 变量代替硬编码颜色

const formatDate = (date: string) => {
  return dayjs(date).format('YYYY-MM-DD HH:mm');
};

const handleCopy = async (content: string) => {
  try {
    await writeText(content);
    message.success('已复制到剪贴板');
    emit('copy', content);
  } catch (error) {
    message.error('复制失败');
    console.error('Copy failed:', error);
  }
};

const handleFavorite = (sql: SqlData) => {
  emit('favorite', sql);
};
</script>

<style scoped>
.widget-container {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 12px;
  padding: 16px;
  transition: all 0.3s;
}

.widget-container:hover {
  background: var(--card-hover-bg);
  border-color: var(--card-hover-border);
  box-shadow: var(--shadow-md);
}

.widget-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-default);
}

.widget-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.widget-icon {
  color: var(--accent-primary);
}

.widget-stats {
  font-size: 12px;
  color: var(--text-muted);
  padding: 4px 12px;
  background: var(--bg-elevated);
  border-radius: 12px;
  border: 1px solid var(--border-default);
}

.widget-content {
  display: flex;
  flex-direction: column;
}

.sql-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.sql-item {
  padding: 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  transition: all 0.2s;
}

.sql-item:hover {
  background: var(--bg-elevated);
  border-color: var(--border-hover);
  box-shadow: var(--shadow-sm);
}

.sql-item-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.sql-type-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.sql-type-badge.type-select {
  background: rgba(16, 185, 129, 0.15);
  color: var(--success);
  border: 1px solid var(--success);
}

.sql-type-badge.type-insert {
  background: rgba(99, 102, 241, 0.15);
  color: var(--accent-primary);
  border: 1px solid var(--accent-primary);
}

.sql-type-badge.type-update {
  background: rgba(245, 158, 11, 0.15);
  color: var(--warning);
  border: 1px solid var(--warning);
}

.sql-type-badge.type-delete {
  background: rgba(239, 68, 68, 0.15);
  color: var(--error);
  border: 1px solid var(--error);
}

.sql-type-badge.type-create,
.sql-type-badge.type-alter {
  background: rgba(139, 92, 246, 0.15);
  color: var(--accent-secondary);
  border: 1px solid var(--accent-secondary);
}

.sql-item-meta {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
  flex: 1;
}

.meta-icon {
  color: var(--text-dim);
}

.favorite-indicator {
  display: flex;
  align-items: center;
}

.sql-content {
  margin-bottom: 10px;
  padding: 10px;
  background: var(--bg-base);
  border: 1px solid var(--border-default);
  border-radius: 6px;
  overflow-x: auto;
}

.sql-code {
  margin: 0;
  padding: 0;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary);
  white-space: pre-wrap;
  word-break: break-all;
}

.sql-tables {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
  font-size: 11px;
  color: var(--text-muted);
}

.table-icon {
  color: var(--accent-primary);
}

.table-list {
  color: var(--text-secondary);
  font-weight: 500;
}

.sql-note {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  margin-bottom: 8px;
  background: var(--bg-elevated);
  border-radius: 6px;
  font-size: 12px;
  color: var(--text-muted);
  border-left: 2px solid var(--accent-primary);
}

.sql-actions {
  display: flex;
  gap: 6px;
  justify-content: flex-end;
  opacity: 0.7;
  transition: opacity 0.2s;
}

.sql-item:hover .sql-actions {
  opacity: 1;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--text-dim);
}

.empty-icon {
  opacity: 0.3;
  margin-bottom: 12px;
}

.empty-state p {
  margin: 0;
  font-size: 14px;
}
</style>
