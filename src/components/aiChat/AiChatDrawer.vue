<template>
  <n-drawer
    v-model:show="showDrawer"
    :width="drawerWidth"
    placement="right"
    :show-mask="true"
    :mask-closable="true"
    :close-on-esc="true"
    class="ai-chat-drawer"
  >
    <n-drawer-content :body-content-style="{ padding: 0, height: '100%' }">
      <AiChat />
    </n-drawer-content>
  </n-drawer>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NDrawer, NDrawerContent } from 'naive-ui';
import AiChat from './AiChat.vue';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void;
}>();

const showDrawer = computed({
  get: () => props.show,
  set: (value: boolean) => emit('update:show', value),
});

// 抽屉宽度 - 响应式适配
const drawerWidth = computed(() => {
  if (typeof window !== 'undefined') {
    const screenWidth = window.innerWidth;
    if (screenWidth < 768) {
      return '100%';
    } else if (screenWidth < 1200) {
      return 700;
    } else {
      return 850;
    }
  }
  return 850;
});
</script>

<style scoped>
.ai-chat-drawer :deep(.n-drawer-body-content-wrapper) {
  height: 100%;
}

.ai-chat-drawer :deep(.n-drawer-content) {
  background: var(--bg-base);
}
</style>
