<template>
  <n-drawer
    v-model:show="showDrawer"
    :width="drawerWidth"
    placement="right"
    :show-mask="true"
    :mask-closable="true"
    :close-on-esc="true"
    class="notification-drawer"
  >
    <n-drawer-content :body-content-style="{ padding: 0, height: '100%' }">
      <NotificationCenter mode="drawer" @close="showDrawer = false" />
    </n-drawer-content>
  </n-drawer>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NDrawer, NDrawerContent } from 'naive-ui';
import NotificationCenter from '@/views/NotificationCenter.vue';

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

// 抽屉宽度 - 响应式适配（与 AI 助手抽屉保持一致）
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
.notification-drawer :deep(.n-drawer-body-content-wrapper) {
  height: 100%;
}

.notification-drawer :deep(.n-drawer-content) {
  background: var(--bg-base);
}
</style>
