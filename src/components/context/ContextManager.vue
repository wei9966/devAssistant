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

:deep(.n-tabs) {
  --n-tab-text-color: var(--text-muted);
  --n-tab-text-color-active: var(--accent-primary);
  --n-tab-text-color-hover: var(--text-secondary);
  --n-bar-color: var(--accent-primary);
  --n-tab-padding: 10px 16px;
  --n-tab-gap: 8px;
}

:deep(.n-tabs-nav) {
  position: sticky;
  top: 100px;
  background: var(--bg-base);
  z-index: 90;
  padding: 8px 0 12px 0;
}

:deep(.n-tabs-tab) {
  border-radius: 8px;
  transition: all 0.2s ease;
}

:deep(.n-tabs-tab:hover:not(.n-tabs-tab--active)) {
  background-color: var(--bg-hover);
}

:deep(.n-tabs-bar) {
  border-radius: 2px;
}
</style>
