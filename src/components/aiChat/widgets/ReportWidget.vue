<template>
  <div class="widget-container report-widget">
    <div class="widget-header">
      <div class="widget-title">
        <n-icon size="20" class="widget-icon">
          <DocumentTextOutline />
        </n-icon>
        <span>{{ getReportTitle() }}</span>
      </div>
      <div class="widget-stats">
        {{ formatDateInfo() }}
      </div>
    </div>

    <div class="widget-content">
      <!-- 报告摘要 -->
      <div v-if="data.report?.summary" class="report-summary">
        <div class="summary-label">摘要</div>
        <div class="summary-content">{{ data.report.summary }}</div>
      </div>

      <!-- Markdown 内容渲染 -->
      <div v-if="data.markdown" class="report-markdown">
        <div v-html="renderedMarkdown" class="markdown-body"></div>
      </div>

      <!-- 结构化内容 (如果有) -->
      <div v-if="data.report?.work_content" class="work-content">
        <div class="content-section-title">工作内容</div>
        <div
          v-for="(section, index) in data.report.work_content"
          :key="index"
          class="content-section"
        >
          <div class="section-category">{{ section.category }}</div>
          <ul class="section-items">
            <li v-for="(item, idx) in section.items" :key="idx">{{ item }}</li>
          </ul>
        </div>
      </div>

      <!-- 时间统计 (如果有) -->
      <div v-if="data.report?.time_stats" class="time-stats-summary">
        <div class="content-section-title">时间统计</div>
        <div class="time-stats-grid">
          <div class="time-stat-item">
            <span class="stat-label">总时长</span>
            <span class="stat-value">{{ data.report.time_stats.total_hours }}h</span>
          </div>
          <div class="time-stat-item">
            <span class="stat-label">活动时段</span>
            <span class="stat-value">{{ data.report.time_stats.active_period }}</span>
          </div>
        </div>
        <div v-if="data.report.time_stats.breakdown" class="breakdown-list">
          <div
            v-for="(item, index) in data.report.time_stats.breakdown"
            :key="index"
            class="breakdown-item"
          >
            <span class="breakdown-category">{{ item.category }}</span>
            <div class="breakdown-bar">
              <div
                class="breakdown-fill"
                :style="{ width: `${item.percentage}%` }"
              ></div>
            </div>
            <span class="breakdown-value">{{ item.hours }}h ({{ item.percentage }}%)</span>
          </div>
        </div>
      </div>

      <!-- 亮点 (如果有) -->
      <div v-if="data.report?.highlights && data.report.highlights.length > 0" class="highlights">
        <div class="content-section-title">亮点</div>
        <ul class="highlights-list">
          <li v-for="(highlight, index) in data.report.highlights" :key="index">
            <n-icon size="14" class="highlight-icon">
              <StarOutline />
            </n-icon>
            {{ highlight }}
          </li>
        </ul>
      </div>

      <!-- 操作按钮 -->
      <div class="report-actions">
        <n-button size="small" @click="handleCopy">
          <template #icon>
            <n-icon><CopyOutline /></n-icon>
          </template>
          复制
        </n-button>
        <n-button size="small" type="primary" @click="handleExport">
          <template #icon>
            <n-icon><DownloadOutline /></n-icon>
          </template>
          导出
        </n-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NButton, NIcon, useMessage } from 'naive-ui';
import { DocumentTextOutline, CopyOutline, DownloadOutline, StarOutline } from '@vicons/ionicons5';
import { marked } from 'marked';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';

interface WorkContent {
  category: string;
  items: string[];
}

interface TimeStats {
  total_hours: number;
  active_period: string;
  breakdown?: Array<{
    category: string;
    hours: number;
    percentage: number;
  }>;
}

interface ReportData {
  summary?: string;
  work_content?: WorkContent[];
  time_stats?: TimeStats;
  highlights?: string[];
}

interface DailyReportResponse {
  date: string;
  report: ReportData;
  markdown?: string;
}

interface WeeklyReportResponse {
  week: string;
  date_range: string;
  report: ReportData;
  markdown?: string;
}

type ReportResponse = DailyReportResponse | WeeklyReportResponse;

const props = defineProps<{
  data: ReportResponse;
}>();

const emit = defineEmits<{
  copy: [];
  export: [];
}>();

const message = useMessage();

const isWeeklyReport = computed(() => 'week' in props.data);

const getReportTitle = () => {
  return isWeeklyReport.value ? '周报' : '日报';
};

const formatDateInfo = () => {
  if (isWeeklyReport.value) {
    const weeklyData = props.data as WeeklyReportResponse;
    return weeklyData.date_range || weeklyData.week;
  }
  const dailyData = props.data as DailyReportResponse;
  return dailyData.date;
};

const renderedMarkdown = computed(() => {
  if (!props.data.markdown) return '';
  return marked.parse(props.data.markdown);
});

const handleCopy = async () => {
  try {
    const content = props.data.markdown || JSON.stringify(props.data.report, null, 2);
    await writeText(content);
    message.success('已复制到剪贴板');
    emit('copy');
  } catch (error) {
    message.error('复制失败');
    console.error('Copy failed:', error);
  }
};

const handleExport = async () => {
  try {
    const fileName = isWeeklyReport.value
      ? `周报_${(props.data as WeeklyReportResponse).week}.md`
      : `日报_${(props.data as DailyReportResponse).date}.md`;

    const filePath = await save({
      defaultPath: fileName,
      filters: [
        {
          name: 'Markdown',
          extensions: ['md'],
        },
      ],
    });

    if (filePath) {
      const content = props.data.markdown || JSON.stringify(props.data.report, null, 2);
      await writeTextFile(filePath, content);
      message.success('导出成功');
      emit('export');
    }
  } catch (error) {
    message.error('导出失败');
    console.error('Export failed:', error);
  }
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
  gap: 16px;
}

.report-summary {
  padding: 12px;
  background: var(--bg-surface);
  border-left: 3px solid var(--accent-primary);
  border-radius: 8px;
}

.summary-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  margin-bottom: 6px;
  letter-spacing: 0.5px;
}

.summary-content {
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-primary);
}

.report-markdown {
  padding: 16px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  overflow-x: auto;
}

.markdown-body {
  color: var(--text-primary);
  font-size: 13px;
  line-height: 1.7;
}

.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3) {
  color: var(--text-primary);
  margin-top: 16px;
  margin-bottom: 8px;
}

.markdown-body :deep(h1) {
  font-size: 18px;
  border-bottom: 2px solid var(--border-default);
  padding-bottom: 8px;
}

.markdown-body :deep(h2) {
  font-size: 16px;
}

.markdown-body :deep(h3) {
  font-size: 14px;
  color: var(--text-secondary);
}

.markdown-body :deep(p) {
  margin: 8px 0;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  padding-left: 24px;
  margin: 8px 0;
}

.markdown-body :deep(li) {
  margin: 4px 0;
}

.markdown-body :deep(code) {
  background: var(--bg-elevated);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: var(--accent-primary);
}

.markdown-body :deep(pre) {
  background: var(--bg-base);
  padding: 12px;
  border-radius: 6px;
  overflow-x: auto;
  border: 1px solid var(--border-default);
}

.markdown-body :deep(pre code) {
  background: none;
  padding: 0;
  color: var(--text-secondary);
}

.content-section-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.work-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.content-section {
  padding: 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 8px;
}

.section-category {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent-primary);
  margin-bottom: 8px;
}

.section-items {
  margin: 0;
  padding-left: 20px;
  list-style-type: disc;
}

.section-items li {
  font-size: 12px;
  line-height: 1.6;
  color: var(--text-secondary);
  margin: 4px 0;
}

.time-stats-summary {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.time-stats-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.time-stat-item {
  padding: 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-label {
  font-size: 11px;
  color: var(--text-muted);
}

.stat-value {
  font-size: 15px;
  font-weight: 700;
  color: var(--accent-primary);
}

.breakdown-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.breakdown-item {
  display: flex;
  align-items: center;
  gap: 10px;
}

.breakdown-category {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
  min-width: 60px;
}

.breakdown-bar {
  flex: 1;
  height: 6px;
  background: var(--progress-bg);
  border-radius: 3px;
  overflow: hidden;
}

.breakdown-fill {
  height: 100%;
  background: var(--progress-fill);
  border-radius: 3px;
  transition: width 0.6s ease;
}

.breakdown-value {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 80px;
  text-align: right;
}

.highlights {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.highlights-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.highlights-list li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-surface);
  border-left: 2px solid var(--warning);
  border-radius: 6px;
  font-size: 12px;
  line-height: 1.6;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.highlight-icon {
  color: var(--warning);
  margin-top: 2px;
  flex-shrink: 0;
}

.report-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding-top: 12px;
  border-top: 1px solid var(--border-default);
}
</style>
