<template>
  <div class="toolbox">
    <!-- Header -->
    <div class="toolbox-header">
      <h2 class="page-title">工具箱</h2>
      <n-space>
        <n-input
          v-model:value="searchKeyword"
          placeholder="搜索工具..."
          clearable
          style="width: 240px"
        >
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
      </n-space>
    </div>

    <!-- Tools Grid -->
    <div class="tools-container">
      <div class="tools-grid">
        <n-card
          v-for="tool in filteredTools"
          :key="tool.id"
          class="tool-card"
          hoverable
          @click="handleOpenTool(tool.id)"
        >
          <div class="tool-content">
            <div class="tool-icon">
              <n-icon :component="tool.icon" size="48" :color="tool.color" />
            </div>
            <div class="tool-info">
              <h3 class="tool-name">{{ tool.name }}</h3>
              <p class="tool-description">{{ tool.description }}</p>
            </div>
          </div>
        </n-card>
      </div>

      <!-- Empty State -->
      <n-empty
        v-if="filteredTools.length === 0"
        description="未找到匹配的工具"
        class="empty-state"
      >
        <template #icon>
          <n-icon size="64" :color="'var(--text-muted)'"><BuildOutline /></n-icon>
        </template>
      </n-empty>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { NSpace, NInput, NIcon, NCard, NEmpty, useMessage } from 'naive-ui'
import {
  SearchOutline,
  BuildOutline,
  ServerOutline,
  CodeSlashOutline,
} from '@vicons/ionicons5'
import { openToolContainer } from '@/api/toolApi'

const message = useMessage()

// 搜索关键词
const searchKeyword = ref('')

// 工具列表
interface Tool {
  id: string
  name: string
  description: string
  icon: any
  color: string
}

const tools = ref<Tool[]>([
  {
    id: 'text-converter',
    name: '文本转换器',
    description: '文本格式转换、JSON格式化、引号处理等',
    icon: CodeSlashOutline,
    color: '#22c55e',
  },
  {
    id: 'port-checker',
    name: '端口检查器',
    description: '检查端口占用情况，快速杀掉占用进程',
    icon: ServerOutline,
    color: '#6366f1',
  },
])

// 过滤后的工具列表
const filteredTools = computed(() => {
  if (!searchKeyword.value) {
    return tools.value
  }

  const keyword = searchKeyword.value.toLowerCase()
  return tools.value.filter(
    (tool) =>
      tool.name.toLowerCase().includes(keyword) ||
      tool.description.toLowerCase().includes(keyword)
  )
})

// 打开工具
const handleOpenTool = async (toolId: string) => {
  try {
    await openToolContainer(toolId)
  } catch (error) {
    message.error('打开工具失败: ' + error)
    console.error('打开工具失败:', error)
  }
}
</script>

<style scoped>
.toolbox {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 20px;
  padding: 24px;
}

/* Header */
.toolbox-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

/* Tools Container */
.tools-container {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

/* Tools Grid */
.tools-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Tool Card */
.tool-card {
  cursor: pointer;
  transition: all 0.3s ease;
  background: var(--card-bg);
  border: 1px solid var(--card-border);
}

.tool-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 24px color-mix(in srgb, var(--accent-primary) 20%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary) 30%, transparent);
}

.tool-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
  align-items: center;
  text-align: center;
  padding: 8px;
}

.tool-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 80px;
  height: 80px;
  border-radius: 16px;
  background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
  transition: all 0.3s ease;
}

.tool-card:hover .tool-icon {
  background: color-mix(in srgb, var(--accent-primary) 20%, transparent);
  transform: scale(1.05);
}

.tool-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.tool-name {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.tool-description {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
  line-height: 1.5;
}

/* Empty State */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
}

/* Custom Scrollbar */
.tools-container::-webkit-scrollbar {
  width: 8px;
}

.tools-container::-webkit-scrollbar-track {
  background: color-mix(in srgb, var(--bg-surface) 30%, transparent);
  border-radius: 4px;
}

.tools-container::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--accent-primary) 30%, transparent);
  border-radius: 4px;
  transition: background 0.2s;
}

.tools-container::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--accent-primary) 50%, transparent);
}

/* Responsive Design */
@media (max-width: 1200px) {
  .tools-grid {
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  }
}

@media (max-width: 768px) {
  .toolbox {
    padding: 16px;
    gap: 16px;
  }

  .toolbox-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .tools-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }
}
</style>
