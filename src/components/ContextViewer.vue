<template>
  <div class="context-viewer">
    <div v-if="isEmpty" class="empty-state">
      <p>暂无上下文信息</p>
    </div>

    <template v-else>
      <!-- 文件上下文 -->
      <div v-if="context.files && context.files.length > 0" class="context-section">
        <n-divider title-placement="left">
          <template #default>
            <span>打开的文件</span>
          </template>
        </n-divider>
        <n-list :clickable="false">
          <n-list-item v-for="file in context.files" :key="file.path">
            <template #prefix>
              <n-icon><DocumentTextOutline /></n-icon>
            </template>
            <div class="file-item">
              <code>{{ file.path }}</code>
              <span v-if="file.line || file.column" class="location">
                {{ file.line }}:{{ file.column }}
              </span>
            </div>
          </n-list-item>
        </n-list>
      </div>

      <!-- SQL上下文 -->
      <div v-if="context.last_sql" class="context-section">
        <n-divider title-placement="left">
          <template #default>
            <span>最后执行的SQL</span>
          </template>
        </n-divider>
        <n-code :code="context.last_sql" language="sql" :show-line-numbers="false" />
        <n-button
          type="primary"
          text
          size="small"
          @click="copySql"
          style="margin-top: 8px"
          title="复制SQL"
        >
          <template #icon>
            <n-icon><CopyOutline /></n-icon>
          </template>
          复制
        </n-button>
      </div>

      <!-- 浏览器标签页 -->
      <div v-if="context.browser_tabs && context.browser_tabs.length > 0" class="context-section">
        <n-divider title-placement="left">
          <template #default>
            <span>浏览器标签页</span>
          </template>
        </n-divider>
        <n-list :clickable="false">
          <n-list-item v-for="(tab, index) in context.browser_tabs" :key="index">
            <template #prefix>
              <n-icon><GlobeOutline /></n-icon>
            </template>
            <a :href="tab" target="_blank" class="tab-link">
              {{ formatUrl(tab) }}
            </a>
          </n-list-item>
        </n-list>
      </div>

      <!-- 备注 -->
      <div v-if="context.notes" class="context-section">
        <n-divider title-placement="left">
          <template #default>
            <span>备注</span>
          </template>
        </n-divider>
        <div class="notes-content">{{ context.notes }}</div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NDivider, NList, NListItem, NIcon, NCode, NButton, useMessage } from 'naive-ui';
import { DocumentTextOutline, GlobeOutline, CopyOutline } from '@vicons/ionicons5';
import type { WorkContext } from '@/types/context';

const props = defineProps<{
  context: WorkContext;
}>();

const message = useMessage();

const isEmpty = computed(() => {
  return (
    (!props.context.files || props.context.files.length === 0) &&
    !props.context.last_sql &&
    (!props.context.browser_tabs || props.context.browser_tabs.length === 0) &&
    !props.context.notes
  );
});

const formatUrl = (url: string) => {
  try {
    const urlObj = new URL(url);
    return urlObj.hostname + urlObj.pathname;
  } catch {
    return url.length > 50 ? url.substring(0, 50) + '...' : url;
  }
};

const copySql = () => {
  if (props.context.last_sql) {
    navigator.clipboard.writeText(props.context.last_sql).then(() => {
      message.success('已复制到剪贴板');
    });
  }
};
</script>

<style scoped>
.context-viewer {
  font-size: 13px;
}

.empty-state {
  text-align: center;
  color: #999;
  padding: 20px;
}

.context-section {
  margin-bottom: 16px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-item code {
  background: #f5f5f5;
  padding: 2px 6px;
  border-radius: 2px;
  font-family: monospace;
  flex: 1;
  word-break: break-all;
}

.location {
  color: #999;
  font-size: 12px;
  white-space: nowrap;
}

.tab-link {
  color: #1890ff;
  text-decoration: none;
  word-break: break-all;
}

.tab-link:hover {
  text-decoration: underline;
}

.notes-content {
  padding: 8px;
  background: #f5f5f5;
  border-radius: 4px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
