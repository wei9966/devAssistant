<template>
  <div class="context-manager">
    <n-tabs type="line" animated>
      <n-tab-pane name="timeline" tab="时间线">
        <ContextTimeline />
      </n-tab-pane>
      <n-tab-pane name="browse" tab="浏览记录">
        <ContextBrowser />
      </n-tab-pane>
      <n-tab-pane name="capture" tab="采集设置">
        <ContextSettings />
      </n-tab-pane>
      <n-tab-pane name="vlm" tab="VLM配置">
        <VlmSettings />
      </n-tab-pane>
      <n-tab-pane name="prompts" tab="提示词配置">
        <PromptSettings />
      </n-tab-pane>
    </n-tabs>
  </div>
</template>

<script setup lang="ts">
import { NTabs, NTabPane } from 'naive-ui';
import ContextTimeline from './ContextTimeline.vue';
import ContextBrowser from './ContextBrowser.vue';
import ContextSettings from './ContextSettings.vue';
import VlmSettings from './VlmSettings.vue';
import PromptSettings from './PromptSettings.vue';
</script>

<style scoped>
.context-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
}

:deep(.n-tabs) {
  height: 100%;
  display: flex;
  flex-direction: column;
}

:deep(.n-tabs-nav-scroll-content) {
  border-bottom: 1px solid rgba(51, 65, 85, 0.6);
}

:deep(.n-tabs-tab) {
  --n-tab-text-color: rgb(148, 163, 184);
  --n-tab-text-color-active: rgb(99, 102, 241);
  --n-tab-text-color-hover: rgb(203, 213, 225);
}

:deep(.n-tabs-bar) {
  --n-bar-color: rgb(99, 102, 241);
}

:deep(.n-tab-pane) {
  height: 100%;
  overflow-y: auto;
  padding: 0;
}

/* 滚动条样式 */
:deep(.n-tab-pane)::-webkit-scrollbar {
  width: 8px;
}

:deep(.n-tab-pane)::-webkit-scrollbar-track {
  background: transparent;
}

:deep(.n-tab-pane)::-webkit-scrollbar-thumb {
  background: rgba(51, 65, 85, 0.5);
  border-radius: 4px;
}

:deep(.n-tab-pane)::-webkit-scrollbar-thumb:hover {
  background: rgba(71, 85, 105, 0.7);
}
</style>
