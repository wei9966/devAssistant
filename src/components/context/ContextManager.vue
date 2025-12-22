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
  background: var(--bg-base);
  z-index: 90;
  padding-bottom: 8px;
}

:deep(.n-tabs-nav-scroll-content) {
  border-bottom: 1px solid var(--border-default);
}

:deep(.n-tabs-tab) {
  --n-tab-text-color: var(--text-muted);
  --n-tab-text-color-active: var(--accent-primary);
  --n-tab-text-color-hover: var(--text-secondary);
}

:deep(.n-tabs-bar) {
  --n-bar-color: var(--accent-primary);
}
</style>
