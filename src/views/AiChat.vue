<template>
  <div class="ai-chat-page">
    <AiChat />
  </div>
</template>

<script setup lang="ts">
import AiChat from '@/components/aiChat/AiChat.vue';
</script>

<style scoped>
.ai-chat-page {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-base);
  overflow: hidden;
}
</style>
