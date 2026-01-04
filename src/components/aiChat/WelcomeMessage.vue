<template>
  <div class="welcome-message">
    <!-- AI 头像 -->
    <div class="welcome-avatar">
      <n-icon size="24" class="avatar-icon">
        <ChatbubblesOutline />
      </n-icon>
      <div class="avatar-glow"></div>
    </div>

    <!-- 欢迎内容 -->
    <div class="welcome-content">
      <!-- 标题 -->
      <div class="welcome-header">
        <h2 class="welcome-title">你好！我是 DevAssistant AI</h2>
        <p class="welcome-subtitle">你的智能开发助手</p>
      </div>

      <!-- 日期和统计摘要 -->
      <div class="welcome-summary">
        <div class="summary-card">
          <div class="summary-date">
            <n-icon size="16" class="summary-icon">
              <CalendarOutline />
            </n-icon>
            <span>{{ currentDate }}</span>
          </div>

          <div v-if="stats" class="summary-stats">
            <div class="stat-chip">
              <n-icon size="14"><CheckmarkCircleOutline /></n-icon>
              <span>待办 {{ stats.todoCount }} 个</span>
            </div>
            <div class="stat-chip">
              <n-icon size="14"><TrendingUpOutline /></n-icon>
              <span>本周完成 {{ stats.weeklyDone }} 个</span>
            </div>
            <div class="stat-chip">
              <n-icon size="14"><TimeOutline /></n-icon>
              <span>今日专注 {{ stats.todayFocus }}m</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 功能介绍 -->
      <div class="welcome-features">
        <h3 class="features-title">我可以帮你做什么？</h3>
        <div class="features-grid">
          <div class="feature-card">
            <n-icon size="20" class="feature-icon">
              <ListOutline />
            </n-icon>
            <div class="feature-text">
              <div class="feature-name">任务管理</div>
              <div class="feature-desc">查询、创建和分析任务</div>
            </div>
          </div>

          <div class="feature-card">
            <n-icon size="20" class="feature-icon">
              <CodeOutline />
            </n-icon>
            <div class="feature-text">
              <div class="feature-name">SQL 助手</div>
              <div class="feature-desc">搜索历史 SQL 查询</div>
            </div>
          </div>

          <div class="feature-card">
            <n-icon size="20" class="feature-icon">
              <StatsChartOutline />
            </n-icon>
            <div class="feature-text">
              <div class="feature-name">数据分析</div>
              <div class="feature-desc">工作效率统计分析</div>
            </div>
          </div>

          <div class="feature-card">
            <n-icon size="20" class="feature-icon">
              <DocumentTextOutline />
            </n-icon>
            <div class="feature-text">
              <div class="feature-name">报告生成</div>
              <div class="feature-desc">自动生成工作周报</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 示例提问 -->
      <div class="welcome-suggestions">
        <h3 class="suggestions-title">试试这些提问</h3>
        <div class="suggestions-list">
          <div
            v-for="(suggestion, index) in suggestions"
            :key="index"
            class="suggestion-chip"
            @click="handleSuggestionClick(suggestion.text)"
          >
            <n-icon size="14" class="suggestion-icon">
              <component :is="suggestion.icon" />
            </n-icon>
            <span>{{ suggestion.text }}</span>
            <n-icon size="14" class="suggestion-arrow">
              <ChevronForwardOutline />
            </n-icon>
          </div>
        </div>
      </div>

      <!-- 底部提示 -->
      <div class="welcome-footer">
        <p class="footer-tip">
          <n-icon size="14"><BulbOutline /></n-icon>
          提示：你可以用自然语言与我交流，我会尽力理解并帮助你
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NIcon } from 'naive-ui';
import {
  ChatbubblesOutline,
  CalendarOutline,
  CheckmarkCircleOutline,
  TrendingUpOutline,
  TimeOutline,
  ListOutline,
  CodeOutline,
  StatsChartOutline,
  DocumentTextOutline,
  ChevronForwardOutline,
  BulbOutline,
  SearchOutline,
  CreateOutline,
  TimerOutline
} from '@vicons/ionicons5';
import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn';

dayjs.locale('zh-cn');

// 用户统计数据
interface UserStats {
  todoCount: number;
  weeklyDone: number;
  todayFocus: number; // 分钟数
}

interface Suggestion {
  text: string;
  icon: any;
}

interface Props {
  stats?: UserStats;
  suggestions?: Suggestion[];
}

const props = withDefaults(defineProps<Props>(), {
  stats: undefined,
  suggestions: () => [
    { text: '这周我完成了多少任务？', icon: CheckmarkCircleOutline },
    { text: '搜索关于订单的 SQL', icon: SearchOutline },
    { text: '生成本周工作周报', icon: CreateOutline },
    { text: '开始 25 分钟番茄钟', icon: TimerOutline }
  ]
});

const emit = defineEmits<{
  'suggestion-click': [text: string];
}>();

// 当前日期
const currentDate = computed(() => {
  return dayjs().format('YYYY年M月D日 dddd');
});

// 处理建议点击
const handleSuggestionClick = (text: string) => {
  emit('suggestion-click', text);
};
</script>

<style scoped>
.welcome-message {
  display: flex;
  gap: 16px;
  padding: 20px;
  animation: fadeInUp 0.6s ease-out;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 头像 */
.welcome-avatar {
  position: relative;
  width: 48px;
  height: 48px;
  border-radius: 12px;
  background: var(--accent-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: var(--shadow-md);
}

.avatar-icon {
  color: white;
  position: relative;
  z-index: 1;
}

.avatar-glow {
  position: absolute;
  inset: -4px;
  border-radius: 14px;
  background: var(--accent-glow);
  animation: glow 3s ease-in-out infinite;
  z-index: 0;
}

@keyframes glow {
  0%, 100% {
    opacity: 0.3;
    transform: scale(0.95);
  }
  50% {
    opacity: 0.6;
    transform: scale(1.05);
  }
}

/* 内容区域 */
.welcome-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* 标题 */
.welcome-header {
  animation: slideIn 0.6s ease-out 0.1s both;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(-10px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.welcome-title {
  font-size: 24px;
  font-weight: 700;
  background: var(--brand-title);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  margin: 0 0 8px 0;
}

.welcome-subtitle {
  font-size: 14px;
  color: var(--text-muted);
  margin: 0;
}

/* 日期和统计 */
.welcome-summary {
  animation: slideIn 0.6s ease-out 0.2s both;
}

.summary-card {
  background: var(--card-bg);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--shadow-sm);
}

.summary-date {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 12px;
}

.summary-icon {
  color: var(--accent-primary);
}

.summary-stats {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.stat-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  font-size: 12px;
  color: var(--text-muted);
  transition: all 0.2s ease;
}

.stat-chip:hover {
  border-color: var(--border-hover);
  background: var(--bg-hover);
}

/* 功能介绍 */
.welcome-features {
  animation: slideIn 0.6s ease-out 0.3s both;
}

.features-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-secondary);
  margin: 0 0 12px 0;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
}

.feature-card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 10px;
  transition: all 0.2s ease;
  cursor: default;
}

.feature-card:hover {
  border-color: var(--border-hover);
  background: var(--bg-hover);
  box-shadow: var(--shadow-sm);
}

.feature-icon {
  color: var(--accent-primary);
  flex-shrink: 0;
}

.feature-text {
  flex: 1;
}

.feature-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.feature-desc {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.4;
}

/* 示例提问 */
.welcome-suggestions {
  animation: slideIn 0.6s ease-out 0.4s both;
}

.suggestions-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-secondary);
  margin: 0 0 12px 0;
}

.suggestions-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.suggestion-chip {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 10px;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.suggestion-chip:hover {
  border-color: var(--accent-primary);
  background: var(--bg-hover);
  color: var(--accent-primary);
  box-shadow: var(--shadow-sm);
}

.suggestion-icon {
  color: var(--accent-secondary);
  flex-shrink: 0;
}

.suggestion-chip:hover .suggestion-icon {
  color: var(--accent-primary);
}

.suggestion-arrow {
  margin-left: auto;
  color: var(--text-dim);
  opacity: 0;
  transition: all 0.2s ease;
}

.suggestion-chip:hover .suggestion-arrow {
  opacity: 1;
  transform: translateX(4px);
}

/* 底部提示 */
.welcome-footer {
  animation: slideIn 0.6s ease-out 0.5s both;
}

.footer-tip {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
}

.footer-tip :deep(.n-icon) {
  color: var(--warning);
  flex-shrink: 0;
}

/* 响应式 */
@media (max-width: 768px) {
  .welcome-message {
    flex-direction: column;
    padding: 16px;
    gap: 16px;
  }

  .welcome-avatar {
    width: 40px;
    height: 40px;
  }

  .welcome-title {
    font-size: 20px;
  }

  .features-grid {
    grid-template-columns: 1fr;
  }

  .stat-chip {
    font-size: 11px;
    padding: 5px 10px;
  }
}
</style>
