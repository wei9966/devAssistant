<template>
  <div :class="['message-bubble', `message-${message.sender}`]">
    <!-- 头像 -->
    <div class="message-avatar">
      <n-icon v-if="isAi" size="18" class="avatar-icon-ai">
        <ChatbubblesOutline />
      </n-icon>
      <n-icon v-else size="18" class="avatar-icon-user">
        <PersonOutline />
      </n-icon>
    </div>

    <!-- 内容区域 -->
    <div class="message-content-wrapper">
      <div :class="['message-content', { 'content-ai': isAi, 'content-user': !isAi }]">
        <!-- 文本内容 - AI消息支持Markdown渲染 -->
        <div v-if="message.content" class="message-text">
          <div v-if="isAi" class="markdown-content" v-html="renderedContent"></div>
          <span v-else>{{ message.content }}</span>
        </div>

        <!-- 任务列表类型 -->
        <div v-if="message.type === 'tasks' && message.data" class="message-widget">
          <slot name="tasks" :data="message.data">
            <div class="widget-tasks">
              <div class="widget-header">
                <n-icon size="16"><ListOutline /></n-icon>
                <span>任务列表 ({{ message.data.total || 0 }} 项)</span>
              </div>
              <div class="widget-content">
                <div v-if="message.data.tasks && message.data.tasks.length > 0">
                  <div v-for="task in message.data.tasks" :key="task.id" class="task-item">
                    <n-icon size="14" :color="getTaskStatusColor(task.status)">
                      <CheckmarkCircleOutline v-if="task.status === 'done'" />
                      <EllipseOutline v-else />
                    </n-icon>
                    <span>{{ task.title }}</span>
                  </div>
                </div>
                <div v-else class="empty-state">
                  <span>暂无任务</span>
                </div>
              </div>
            </div>
          </slot>
        </div>

        <!-- SQL 查询类型 -->
        <div v-if="message.type === 'sql' && message.data" class="message-widget">
          <slot name="sql" :data="message.data">
            <div class="widget-sql">
              <div class="widget-header">
                <n-icon size="16"><CodeOutline /></n-icon>
                <span>SQL 查询 ({{ message.data.total || 1 }} 条)</span>
              </div>
              <div class="widget-content">
                <!-- 多条 SQL 记录 -->
                <div v-if="message.data.sql_list && message.data.sql_list.length > 0" class="sql-list">
                  <div v-for="(sqlItem, index) in message.data.sql_list" :key="index" class="sql-item">
                    <div class="sql-item-header">
                      <span class="sql-item-title">{{ sqlItem.title || `SQL #${index + 1}` }}</span>
                      <span v-if="sqlItem.category" class="sql-item-category">{{ sqlItem.category }}</span>
                    </div>
                    <pre class="sql-code">{{ sqlItem.content || sqlItem }}</pre>
                  </div>
                </div>
                <!-- 单条 SQL (兼容旧格式) -->
                <pre v-else-if="message.data.sql" class="sql-code">{{ message.data.sql }}</pre>
              </div>
            </div>
          </slot>
        </div>

        <!-- 查询结果类型 (query_data 返回的动态表格数据) -->
        <div v-if="message.type === 'query_result' && message.data" class="message-widget">
          <slot name="query_result" :data="message.data">
            <div class="widget-query-result">
              <div class="widget-header">
                <n-icon size="16"><SearchOutline /></n-icon>
                <span>{{ message.data.description || '查询结果' }} ({{ message.data.row_count || 0 }} 条)</span>
              </div>
              <div class="widget-content">
                <!-- 无数据时显示 -->
                <div v-if="!message.data.rows || message.data.rows.length === 0" class="empty-state">
                  <span>没有找到匹配的数据</span>
                </div>
                <!-- 有数据时显示表格 -->
                <div v-else class="query-result-table-wrapper">
                  <table class="query-result-table">
                    <thead>
                      <tr>
                        <th v-for="col in displayColumns" :key="col">{{ formatColumnName(col) }}</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr v-for="(row, rowIndex) in displayRows" :key="rowIndex">
                        <td v-for="col in displayColumns" :key="col">
                          {{ formatCellValue(row[col]) }}
                        </td>
                      </tr>
                    </tbody>
                  </table>
                  <div v-if="message.data.rows.length > 10" class="table-footer">
                    显示前 10 条，共 {{ message.data.row_count }} 条记录
                  </div>
                </div>
              </div>
            </div>
          </slot>
        </div>

        <!-- 任务详情类型 -->
        <div v-if="message.type === 'task_detail' && message.data" class="message-widget">
          <slot name="task_detail" :data="message.data">
            <div class="widget-task-detail">
              <div class="widget-header">
                <n-icon size="16"><DocumentTextOutline /></n-icon>
                <span>任务详情</span>
              </div>
              <div class="widget-content">
                <div class="task-detail-grid">
                  <div class="detail-row">
                    <span class="detail-label">标题</span>
                    <span class="detail-value">{{ message.data.title }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">状态</span>
                    <span :class="['detail-status', `status-${message.data.status}`]">
                      {{ getStatusText(message.data.status) }}
                    </span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">优先级</span>
                    <span class="detail-value">{{ getPriorityText(message.data.priority) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">创建时间</span>
                    <span class="detail-value">{{ message.data.created_at }}</span>
                  </div>
                  <div v-if="message.data.updated_at" class="detail-row">
                    <span class="detail-label">更新时间</span>
                    <span class="detail-value">{{ message.data.updated_at }}</span>
                  </div>
                  <div v-if="message.data.completed_at" class="detail-row">
                    <span class="detail-label">完成时间</span>
                    <span class="detail-value">{{ message.data.completed_at }}</span>
                  </div>
                  <div v-if="message.data.description" class="detail-row detail-desc">
                    <span class="detail-label">描述</span>
                    <span class="detail-value">{{ message.data.description }}</span>
                  </div>
                </div>
              </div>
            </div>
          </slot>
        </div>

        <!-- 统计数据类型 (兼容 stats 和 time_stats) -->
        <div v-if="(message.type === 'stats' || message.type === 'time_stats') && message.data" class="message-widget">
          <slot name="stats" :data="message.data">
            <div class="widget-stats">
              <div class="widget-header">
                <n-icon size="16"><StatsChartOutline /></n-icon>
                <span>{{ getStatsPeriodTitle(message.data) }}</span>
              </div>
              <div class="widget-content">
                <div class="stats-grid">
                  <!-- 时间统计格式 -->
                  <template v-if="message.type === 'time_stats'">
                    <div class="stat-item">
                      <div class="stat-label">专注时长</div>
                      <div class="stat-value">{{ formatMinutes(message.data.total_focus_time_minutes) }}</div>
                    </div>
                    <div class="stat-item">
                      <div class="stat-label">完成任务</div>
                      <div class="stat-value">{{ message.data.completed_tasks }}</div>
                    </div>
                    <div class="stat-item">
                      <div class="stat-label">番茄钟</div>
                      <div class="stat-value">{{ message.data.pomodoro_count }}</div>
                    </div>
                  </template>
                  <!-- 通用统计格式 -->
                  <template v-else-if="message.data.stats">
                    <div v-for="(stat, key) in message.data.stats" :key="key" class="stat-item">
                      <div class="stat-label">{{ stat.label }}</div>
                      <div class="stat-value">{{ stat.value }}</div>
                    </div>
                  </template>
                </div>
              </div>
            </div>
          </slot>
        </div>

        <!-- 报告类型 -->
        <div v-if="message.type === 'report' && message.data" class="message-widget">
          <slot name="report" :data="message.data">
            <div class="widget-report">
              <div class="widget-header">
                <n-icon size="16"><DocumentTextOutline /></n-icon>
                <span>{{ message.data.title }}</span>
              </div>
              <div class="widget-content">
                <div class="report-content">
                  {{ message.data.content }}
                </div>
              </div>
            </div>
          </slot>
        </div>

        <!-- 番茄钟类型 -->
        <div v-if="(message.type === 'pomodoro' || message.type === 'pomodoro_started') && message.data" class="message-widget">
          <slot name="pomodoro" :data="message.data">
            <div class="widget-pomodoro">
              <div class="widget-header">
                <n-icon size="16"><TimeOutline /></n-icon>
                <span>番茄钟</span>
              </div>
              <div class="widget-content">
                <div class="pomodoro-info">
                  <div class="pomodoro-icon">🍅</div>
                  <div class="pomodoro-details">
                    <div class="pomodoro-task">{{ message.data.task_title || '专注中' }}</div>
                    <div class="pomodoro-duration">{{ message.data.duration || 25 }} 分钟</div>
                  </div>
                  <div class="pomodoro-status">进行中</div>
                </div>
                <div v-if="message.data.today_count" class="pomodoro-today">
                  今日已完成 {{ message.data.today_count }} 个番茄钟
                </div>
              </div>
            </div>
          </slot>
        </div>

        <!-- 自定义内容插槽 -->
        <slot name="custom" :message="message"></slot>
      </div>

      <!-- 时间戳 -->
      <div class="message-timestamp">
        {{ formatTime(message.timestamp) }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NIcon } from 'naive-ui';
import {
  ChatbubblesOutline,
  PersonOutline,
  ListOutline,
  CodeOutline,
  StatsChartOutline,
  DocumentTextOutline,
  CheckmarkCircleOutline,
  EllipseOutline,
  TimeOutline,
  SearchOutline
} from '@vicons/ionicons5';
import dayjs from 'dayjs';

// 定义消息类型
export interface ChatMessage {
  id: string | number;
  sender: 'user' | 'ai' | 'system';
  content?: string;
  type?: 'text' | 'tasks' | 'sql' | 'stats' | 'time_stats' | 'report' | 'pomodoro' | 'pomodoro_started' | string;
  data?: any;
  timestamp?: number | string;
}

interface Props {
  message: ChatMessage;
}

const props = defineProps<Props>();

// 是否为 AI 消息
const isAi = computed(() => props.message.sender === 'ai');

// 改进的 Markdown 渲染函数
const renderMarkdown = (text: string): string => {
  if (!text) return '';

  // 按行处理，更好地处理列表
  const lines = text.split('\n');
  const result: string[] = [];
  let inOrderedList = false;
  let inUnorderedList = false;
  let listCounter = 0;

  for (let i = 0; i < lines.length; i++) {
    let line = lines[i];

    // 转义 HTML 标签（安全处理）
    line = line
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    // 检查是否是有序列表项
    const orderedMatch = line.match(/^\d+\.\s+(.+)$/);
    // 检查是否是无序列表项
    const unorderedMatch = line.match(/^[-*]\s+(.+)$/);

    if (orderedMatch) {
      if (!inOrderedList) {
        if (inUnorderedList) {
          result.push('</ul>');
          inUnorderedList = false;
        }
        result.push('<ol class="md-ol">');
        inOrderedList = true;
        listCounter = 0;
      }
      listCounter++;
      const content = processInlineMarkdown(orderedMatch[1]);
      result.push(`<li class="md-li-num">${content}</li>`);
    } else if (unorderedMatch) {
      if (!inUnorderedList) {
        if (inOrderedList) {
          result.push('</ol>');
          inOrderedList = false;
        }
        result.push('<ul class="md-ul">');
        inUnorderedList = true;
      }
      const content = processInlineMarkdown(unorderedMatch[1]);
      result.push(`<li class="md-li">${content}</li>`);
    } else {
      // 关闭列表
      if (inOrderedList && line.trim() !== '') {
        result.push('</ol>');
        inOrderedList = false;
      }
      if (inUnorderedList && line.trim() !== '') {
        result.push('</ul>');
        inUnorderedList = false;
      }

      // 处理其他 Markdown 元素
      if (line.match(/^### (.+)$/)) {
        line = line.replace(/^### (.+)$/, '<h4 class="md-h4">$1</h4>');
      } else if (line.match(/^## (.+)$/)) {
        line = line.replace(/^## (.+)$/, '<h3 class="md-h3">$1</h3>');
      } else if (line.match(/^# (.+)$/)) {
        line = line.replace(/^# (.+)$/, '<h2 class="md-h2">$1</h2>');
      } else if (line.trim() === '') {
        // 空行，跳过但不关闭列表（允许列表项之间有空行）
        if (!inOrderedList && !inUnorderedList) {
          result.push('<br>');
        }
        continue;
      } else {
        line = processInlineMarkdown(line);
      }
      result.push(line);
    }
  }

  // 关闭未关闭的列表
  if (inOrderedList) result.push('</ol>');
  if (inUnorderedList) result.push('</ul>');

  return result.join('\n');
};

// 处理行内 Markdown（粗体、斜体、代码等）
const processInlineMarkdown = (text: string): string => {
  return text
    // 粗体
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    // 斜体
    .replace(/\*([^*]+)\*/g, '<em>$1</em>')
    // 行内代码
    .replace(/`([^`]+)`/g, '<code class="md-code">$1</code>');
};

// 渲染后的内容
const renderedContent = computed(() => {
  return renderMarkdown(props.message.content || '');
});

// 格式化时间
const formatTime = (timestamp?: number | string) => {
  if (!timestamp) {
    return dayjs().format('HH:mm');
  }
  return dayjs(timestamp).format('HH:mm');
};

// 获取任务状态颜色
const getTaskStatusColor = (status: string) => {
  const colors: Record<string, string> = {
    'done': 'var(--success)',
    'active': 'var(--accent-primary)',
    'todo': 'var(--text-dim)'
  };
  return colors[status] || 'var(--text-dim)';
};

// 获取统计时间段标题
const getStatsPeriodTitle = (data: any) => {
  if (!data?.period) return '数据统计';
  const periodMap: Record<string, string> = {
    'today': '今日统计',
    'week': '本周统计',
    'month': '本月统计'
  };
  return periodMap[data.period] || '数据统计';
};

// 格式化分钟数
const formatMinutes = (minutes: number) => {
  if (!minutes || minutes <= 0) return '0 分钟';
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (hours > 0) {
    return `${hours}h ${mins}m`;
  }
  return `${mins} 分钟`;
};

// 获取状态文本
const getStatusText = (status: string) => {
  const statusMap: Record<string, string> = {
    'todo': '待办',
    'active': '进行中',
    'done': '已完成'
  };
  return statusMap[status] || status;
};

// 获取优先级文本
const getPriorityText = (priority: number) => {
  const priorityMap: Record<number, string> = {
    1: '高优先级 🔴',
    2: '中优先级 🟡',
    3: '低优先级 🟢'
  };
  return priorityMap[priority] || '未知';
};

// query_result 表格显示的行数（最多10行）
const displayRows = computed(() => {
  if (!props.message.data?.rows) return [];
  return props.message.data.rows.slice(0, 10);
});

// query_result 显示的列（过滤掉一些技术字段）
const displayColumns = computed(() => {
  if (!props.message.data?.columns) return [];
  // 过滤掉不需要展示给用户的技术字段
  const hiddenColumns = ['sql', 'query'];
  return props.message.data.columns.filter((col: string) => !hiddenColumns.includes(col.toLowerCase()));
});

// 格式化列名（让数据库字段名更友好）
const formatColumnName = (col: string): string => {
  const columnNameMap: Record<string, string> = {
    'id': 'ID',
    'title': '标题',
    'description': '描述',
    'status': '状态',
    'priority': '优先级',
    'category': '分类',
    'created_at': '创建时间',
    'updated_at': '更新时间',
    'completed_at': '完成时间',
    'started_at': '开始时间',
    'due_date': '截止日期',
    'progress': '进度',
    'notes': '备注',
    'task_id': '任务ID',
    'duration_minutes': '时长(分钟)',
    'actual_focus_seconds': '实际专注(秒)',
    'focus_rate': '专注率',
    'distraction_count': '分心次数',
    'focus_goal': '专注目标',
    'name': '名称',
    'content': '内容',
    'content_type': '类型',
    'is_read': '已读',
    'is_favorite': '收藏',
  };
  return columnNameMap[col.toLowerCase()] || col;
};

// 格式化单元格值
const formatCellValue = (value: any): string => {
  if (value === null || value === undefined) return '-';
  if (typeof value === 'string') {
    // 截断过长的字符串
    if (value.length > 50) {
      return value.substring(0, 47) + '...';
    }
    return value;
  }
  if (typeof value === 'number') {
    return value.toString();
  }
  if (typeof value === 'boolean') {
    return value ? '是' : '否';
  }
  return String(value);
};
</script>

<style scoped>
.message-bubble {
  display: flex;
  gap: 12px;
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 用户消息右对齐 */
.message-user {
  flex-direction: row-reverse;
}

/* 头像 */
.message-avatar {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: var(--shadow-sm);
}

.message-ai .message-avatar {
  background: var(--accent-primary);
}

.message-user .message-avatar {
  background: var(--bg-elevated);
}

.avatar-icon-ai {
  color: white;
}

.avatar-icon-user {
  color: var(--text-secondary);
}

/* 内容区域 */
.message-content-wrapper {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-width: 80%;
}

.message-user .message-content-wrapper {
  align-items: flex-end;
}

.message-ai .message-content-wrapper {
  align-items: flex-start;
}

/* 消息内容 */
.message-content {
  padding: 12px 16px;
  border-radius: 16px;
  font-size: 14px;
  line-height: 1.6;
  box-shadow: var(--shadow-sm);
  word-wrap: break-word;
  overflow-wrap: break-word;
}

/* AI 消息样式 */
.content-ai {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  color: var(--text-primary);
  border-top-left-radius: 4px;
}

/* 用户消息样式 */
.content-user {
  background: var(--accent-primary);
  color: white;
  border-top-right-radius: 4px;
}

/* 文本内容 */
.message-text {
  white-space: pre-wrap;
}

/* Markdown 渲染样式 */
.markdown-content {
  line-height: 1.7;
}

.markdown-content :deep(p) {
  margin: 0;
}

.markdown-content :deep(h2.md-h2) {
  font-size: 18px;
  font-weight: 700;
  margin: 16px 0 12px 0;
  color: var(--text-primary);
  border-bottom: 1px solid var(--border-default);
  padding-bottom: 8px;
}

.markdown-content :deep(h3.md-h3) {
  font-size: 16px;
  font-weight: 600;
  margin: 14px 0 10px 0;
  color: var(--text-primary);
}

.markdown-content :deep(h4.md-h4) {
  font-size: 14px;
  font-weight: 600;
  margin: 12px 0 8px 0;
  color: var(--text-secondary);
}

.markdown-content :deep(strong) {
  font-weight: 600;
  color: var(--text-primary);
}

.markdown-content :deep(em) {
  font-style: italic;
}

.markdown-content :deep(.md-code) {
  background: var(--bg-base);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  color: var(--accent-secondary);
}

.markdown-content :deep(.md-pre) {
  background: var(--bg-base);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  padding: 12px;
  margin: 10px 0;
  overflow-x: auto;
}

.markdown-content :deep(.md-pre code) {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  color: var(--text-secondary);
  white-space: pre-wrap;
}

.markdown-content :deep(.md-table) {
  width: 100%;
  border-collapse: collapse;
  margin: 12px 0;
  font-size: 13px;
}

.markdown-content :deep(.md-table td) {
  padding: 8px 12px;
  border: 1px solid var(--border-default);
  background: var(--bg-elevated);
}

.markdown-content :deep(.md-table tr:first-child td) {
  background: var(--card-bg);
  font-weight: 600;
  color: var(--text-secondary);
}

.markdown-content :deep(.md-ul) {
  margin: 8px 0;
  padding-left: 24px;
  list-style-type: disc;
}

.markdown-content :deep(.md-ol) {
  margin: 8px 0;
  padding-left: 24px;
  list-style-type: decimal;
}

.markdown-content :deep(.md-li) {
  margin: 4px 0;
  color: var(--text-secondary);
  display: list-item;
}

.markdown-content :deep(.md-li-num) {
  margin: 4px 0;
  color: var(--text-secondary);
  display: list-item;
}

.markdown-content :deep(br) {
  display: block;
  content: "";
  margin-top: 4px;
}

/* 时间戳 */
.message-timestamp {
  font-size: 11px;
  color: var(--text-dim);
  padding: 0 8px;
  opacity: 0.7;
}

/* 消息小部件 */
.message-widget {
  margin-top: 12px;
}

.widget-tasks,
.widget-sql,
.widget-stats,
.widget-report,
.widget-query-result {
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  overflow: hidden;
}

.widget-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: var(--card-bg);
  border-bottom: 1px solid var(--border-default);
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.widget-content {
  padding: 12px;
}

/* 任务列表 */
.task-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-default);
  font-size: 13px;
  color: var(--text-primary);
}

.task-item:last-child {
  border-bottom: none;
}

/* 空状态 */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  color: var(--text-muted);
  font-size: 13px;
}

/* SQL 列表 */
.sql-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.sql-item {
  background: var(--bg-base);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  overflow: hidden;
}

.sql-item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--card-bg);
  border-bottom: 1px solid var(--border-default);
}

.sql-item-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.sql-item-category {
  font-size: 11px;
  padding: 2px 8px;
  background: var(--accent-primary);
  color: white;
  border-radius: 4px;
}

.sql-item .sql-code {
  border: none;
  border-radius: 0;
  margin: 0;
}

/* SQL 代码 */
.sql-code {
  background: var(--bg-base);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  padding: 12px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  color: var(--accent-secondary);
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

/* 统计数据 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 12px;
}

.stat-item {
  background: var(--card-bg);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  padding: 12px;
  text-align: center;
}

.stat-label {
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 4px;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--accent-primary);
}

/* 报告 */
.report-content {
  font-size: 13px;
  line-height: 1.8;
  color: var(--text-secondary);
  white-space: pre-wrap;
}

/* 任务详情 */
.widget-task-detail {
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  overflow: hidden;
}

.task-detail-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.detail-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-default);
}

.detail-row:last-child {
  border-bottom: none;
}

.detail-label {
  min-width: 70px;
  font-size: 12px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.detail-value {
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
}

.detail-status {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;
}

.detail-status.status-todo {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.detail-status.status-active {
  background: rgba(59, 130, 246, 0.15);
  color: var(--accent-primary);
}

.detail-status.status-done {
  background: rgba(34, 197, 94, 0.15);
  color: var(--success);
}

.detail-desc {
  flex-direction: column;
  gap: 4px;
}

.detail-desc .detail-value {
  white-space: pre-wrap;
  line-height: 1.6;
}

/* 番茄钟 */
.widget-pomodoro {
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  overflow: hidden;
}

.pomodoro-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.pomodoro-icon {
  font-size: 32px;
}

.pomodoro-details {
  flex: 1;
}

.pomodoro-task {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.pomodoro-duration {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 2px;
}

.pomodoro-status {
  padding: 4px 10px;
  background: var(--success);
  color: white;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 600;
}

.pomodoro-today {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border-default);
  font-size: 12px;
  color: var(--text-muted);
  text-align: center;
}

/* 查询结果表格 */
.query-result-table-wrapper {
  overflow-x: auto;
  margin-bottom: 12px;
}

.query-result-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

.query-result-table th {
  background: var(--card-bg);
  padding: 8px 12px;
  text-align: left;
  font-weight: 600;
  color: var(--text-secondary);
  border: 1px solid var(--border-default);
  white-space: nowrap;
}

.query-result-table td {
  padding: 8px 12px;
  border: 1px solid var(--border-default);
  color: var(--text-primary);
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.query-result-table tr:hover td {
  background: var(--bg-hover);
}

.table-footer {
  margin-top: 8px;
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
}

/* 响应式 */
@media (max-width: 768px) {
  .message-content-wrapper {
    max-width: 85%;
  }

  .message-avatar {
    width: 32px;
    height: 32px;
  }

  .message-content {
    padding: 10px 14px;
    font-size: 13px;
  }
}
</style>
