<template>
  <div class="report-center">
    <div class="report-header">
      <h2 class="report-title">报表中心</h2>
      <div class="header-actions">
        <n-button-group>
          <n-button
            size="small"
            :type="currentTab === 'tasks' ? 'primary' : 'default'"
            @click="currentTab = 'tasks'"
          >
            <template #icon><n-icon :component="CheckboxOutline" /></template>
            任务报表
          </n-button>
          <n-button
            size="small"
            :type="currentTab === 'statistics' ? 'primary' : 'default'"
            @click="currentTab = 'statistics'"
          >
            <template #icon><n-icon :component="StatsChartOutline" /></template>
            数据统计
          </n-button>
          <n-button
            size="small"
            :type="currentTab === 'timeline' ? 'primary' : 'default'"
            @click="currentTab = 'timeline'"
          >
            <template #icon><n-icon :component="TimeOutline" /></template>
            时间线
          </n-button>
          <n-button
            size="small"
            :type="currentTab === 'browse' ? 'primary' : 'default'"
            @click="currentTab = 'browse'"
          >
            <template #icon><n-icon :component="EyeOutline" /></template>
            浏览记录
          </n-button>
          <n-button
            size="small"
            :type="currentTab === 'screenshot' ? 'primary' : 'default'"
            @click="currentTab = 'screenshot'"
          >
            <template #icon><n-icon :component="ImagesOutline" /></template>
            截图回顾
          </n-button>
          <n-button
            size="small"
            :type="currentTab === 'pomodoro' ? 'primary' : 'default'"
            @click="currentTab = 'pomodoro'"
          >
            <template #icon><n-icon :component="TimerOutline" /></template>
            番茄钟报表
          </n-button>
        </n-button-group>
      </div>
    </div>

    <!-- 任务报表视图 -->
    <div v-if="currentTab === 'tasks'" class="tab-content">
      <TaskReport />
    </div>

    <!-- 数据统计视图 -->
    <div v-else-if="currentTab === 'statistics'" class="tab-content">
      <DataStatistics />
    </div>

    <!-- 时间线视图 -->
    <div v-else-if="currentTab === 'timeline'" class="tab-content">
      <ContextTimeline />
    </div>

    <!-- 浏览记录视图 -->
    <div v-else-if="currentTab === 'browse'" class="tab-content">
      <ContextBrowser />
    </div>

    <!-- 截图回顾视图 -->
    <div v-else-if="currentTab === 'screenshot'" class="tab-content">
      <ScreenshotGallery />
    </div>

    <!-- 番茄钟报表视图 -->
    <div v-else-if="currentTab === 'pomodoro'" class="tab-content">
      <PomodoroReport />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { NButton, NButtonGroup, NIcon } from 'naive-ui';
import {
  TimeOutline,
  EyeOutline,
  CheckboxOutline,
  StatsChartOutline,
  ImagesOutline,
  TimerOutline
} from '@vicons/ionicons5';
import ContextTimeline from '@/components/context/ContextTimeline.vue';
import ContextBrowser from '@/components/context/ContextBrowser.vue';
import TaskReport from '@/components/report/TaskReport.vue';
import DataStatistics from '@/views/DataStatistics.vue';
import ScreenshotGallery from '@/views/ScreenshotGallery.vue';
import PomodoroReport from '@/components/report/PomodoroReport.vue';

const currentTab = ref<'tasks' | 'statistics' | 'timeline' | 'browse' | 'screenshot' | 'pomodoro'>('tasks');
</script>

<style scoped>
.report-center {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 32px;
  background: linear-gradient(to bottom right, #0f172a, #1e293b);
}

.report-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.report-title {
  font-size: 20px;
  font-weight: 600;
  color: #f1f5f9;
  letter-spacing: -0.025em;
  margin: 0;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.tab-content {
  flex: 1;
  overflow: auto;
  background: rgba(15, 23, 42, 0.4);
  border-radius: 20px;
  border: 1px solid rgba(51, 65, 85, 0.5);
  backdrop-filter: blur(12px);
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

/* 按钮组样式 */
:deep(.n-button-group) {
  display: inline-flex;
}

:deep(.n-button-group .n-button) {
  border-radius: 0;
}

:deep(.n-button-group .n-button:first-child) {
  border-top-left-radius: 8px;
  border-bottom-left-radius: 8px;
}

:deep(.n-button-group .n-button:last-child) {
  border-top-right-radius: 8px;
  border-bottom-right-radius: 8px;
}

:deep(.n-button--default-type) {
  background-color: #1e293b;
  border-color: #334155;
  color: #cbd5e1;
}

:deep(.n-button--default-type:hover) {
  background-color: #334155;
  border-color: #475569;
}

:deep(.primary-button),
:deep(.n-button--primary-type) {
  background-color: #6366f1 !important;
  border-color: #6366f1 !important;
  color: #ffffff !important;
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2) !important;
}

:deep(.n-button--primary-type:hover) {
  background-color: #4f46e5 !important;
  border-color: #4f46e5 !important;
}

/* 滚动条样式 */
.tab-content::-webkit-scrollbar {
  width: 8px;
}

.tab-content::-webkit-scrollbar-track {
  background: transparent;
}

.tab-content::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 4px;
}

.tab-content::-webkit-scrollbar-thumb:hover {
  background: #475569;
}
</style>
