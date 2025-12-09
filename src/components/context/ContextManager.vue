<template>
  <div class="context-manager">
    <n-tabs type="line" animated>
      <n-tab-pane name="capture" tab="采集设置">
        <ContextSettings />
      </n-tab-pane>
      <n-tab-pane name="vlm" tab="VLM配置">
        <VlmSettings />
      </n-tab-pane>
    </n-tabs>
  </div>
</template>

<script setup lang="ts">
import { NTabs, NTabPane } from 'naive-ui';
import ContextSettings from './ContextSettings.vue';
import VlmSettings from './VlmSettings.vue';
</script>

<style scoped>
.context-manager {
  display: flex;
  flex-direction: column;
}

:deep(.n-tabs-nav) {
  position: sticky;
  top: 100px;
  background: rgb(2, 6, 23);
  z-index: 90;
  padding-bottom: 8px;
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
</style>
