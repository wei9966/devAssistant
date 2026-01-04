<template>
  <div class="widget-container time-stats-widget">
    <div class="widget-header">
      <div class="widget-title">
        <n-icon size="20" class="widget-icon">
          <TimeOutline />
        </n-icon>
        <span>时间统计</span>
      </div>
      <div class="widget-stats">
        {{ data.date_range === 'today' ? '今日' : formatDateRange(data.date_range) }}
      </div>
    </div>

    <div class="widget-content">
      <!-- 总时长 & 活动时段 -->
      <div class="stats-overview">
        <div class="stats-card">
          <div class="stats-label">总时长</div>
          <div class="stats-value">{{ formatMinutes(data.total_minutes) }}</div>
        </div>
        <div class="stats-card">
          <div class="stats-label">活动时段</div>
          <div class="stats-value time-range">{{ data.active_time || '-' }}</div>
        </div>
      </div>

      <!-- 分类统计 -->
      <div v-if="data.by_category && data.by_category.length > 0" class="category-stats">
        <div class="stats-section-title">分类统计</div>
        <div class="category-list">
          <div
            v-for="(category, index) in data.by_category"
            :key="index"
            class="category-item"
          >
            <div class="category-header">
              <div class="category-info">
                <span class="category-name">{{ category.category }}</span>
                <span class="category-time">{{ formatMinutes(category.minutes) }}</span>
              </div>
              <span class="category-percentage">{{ category.percentage.toFixed(1) }}%</span>
            </div>
            <div class="progress-bar-container">
              <div
                class="progress-bar-fill"
                :style="{
                  width: `${category.percentage}%`,
                  background: getCategoryColor(category.category, index),
                }"
              ></div>
            </div>
            <div v-if="category.apps && category.apps.length > 0" class="category-apps">
              <n-icon size="12" class="apps-icon">
                <AppsOutline />
              </n-icon>
              <span>{{ category.apps.join(', ') }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 应用统计 (可选) -->
      <div v-if="data.by_app && data.by_app.length > 0" class="app-stats">
        <div class="stats-section-title">应用统计 (前 5)</div>
        <div class="app-list">
          <div v-for="(app, index) in data.by_app.slice(0, 5)" :key="index" class="app-item">
            <div class="app-info">
              <span class="app-rank">{{ index + 1 }}</span>
              <span class="app-name">{{ app.app }}</span>
            </div>
            <div class="app-stats-right">
              <span class="app-time">{{ formatMinutes(app.minutes) }}</span>
              <span class="app-percentage">{{ app.percentage.toFixed(1) }}%</span>
            </div>
          </div>
        </div>
      </div>

      <div v-if="data.total_minutes === 0" class="empty-state">
        <n-icon size="48" class="empty-icon">
          <TimeOutline />
        </n-icon>
        <p>暂无时间数据</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NIcon } from 'naive-ui';
import { TimeOutline, AppsOutline } from '@vicons/ionicons5';

interface CategoryStat {
  category: string;
  minutes: number;
  percentage: number;
  apps?: string[];
}

interface AppStat {
  app: string;
  minutes: number;
  percentage: number;
}

interface TimeStatsResponse {
  date_range: string;
  total_minutes: number;
  active_time?: string;
  by_category?: CategoryStat[];
  by_app?: AppStat[];
}

defineProps<{
  data: TimeStatsResponse;
}>();

const formatMinutes = (minutes: number) => {
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (hours > 0) {
    return `${hours}h ${mins}m`;
  }
  return `${mins}m`;
};

const formatDateRange = (range: string) => {
  const rangeMap: Record<string, string> = {
    today: '今日',
    yesterday: '昨日',
    this_week: '本周',
    last_week: '上周',
    this_month: '本月',
  };
  return rangeMap[range] || range;
};

const getCategoryColor = (category: string, index: number) => {
  const categoryColors: Record<string, string> = {
    编程: 'var(--progress-fill)',
    文档: 'linear-gradient(90deg, #10b981 0%, #059669 100%)',
    浏览: 'linear-gradient(90deg, #3b82f6 0%, #2563eb 100%)',
    会议: 'linear-gradient(90deg, #f59e0b 0%, #d97706 100%)',
    其他: 'linear-gradient(90deg, #64748b 0%, #475569 100%)',
  };
  return categoryColors[category] || `linear-gradient(90deg, var(--accent-primary) 0%, var(--accent-secondary) 100%)`;
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

.stats-overview {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.stats-card {
  padding: 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  text-align: center;
  transition: all 0.2s;
}

.stats-card:hover {
  background: var(--bg-elevated);
  border-color: var(--border-hover);
  transform: translateY(-2px);
}

.stats-label {
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 6px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stats-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--accent-primary);
  line-height: 1.2;
}

.stats-value.time-range {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.stats-section-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.category-stats,
.app-stats {
  display: flex;
  flex-direction: column;
}

.category-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.category-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.category-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.category-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.category-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.category-time {
  font-size: 12px;
  color: var(--text-muted);
}

.category-percentage {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent-primary);
  padding: 2px 8px;
  background: var(--accent-glow);
  border-radius: 10px;
}

.progress-bar-container {
  height: 6px;
  background: var(--progress-bg);
  border-radius: 3px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.6s ease;
}

.category-apps {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-muted);
  padding-left: 4px;
}

.apps-icon {
  color: var(--text-dim);
}

.app-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.app-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 6px;
  transition: all 0.2s;
}

.app-item:hover {
  background: var(--bg-elevated);
  border-color: var(--border-hover);
}

.app-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.app-rank {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  font-size: 11px;
  font-weight: 700;
  color: var(--text-primary);
  background: var(--accent-glow);
  border-radius: 50%;
}

.app-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.app-stats-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.app-time {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
}

.app-percentage {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 45px;
  text-align: right;
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
