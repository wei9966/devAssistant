<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider>
      <div class="quick-task-wrapper" @keydown="handleGlobalKeyDown">
        <div class="modal-container">
          <!-- 核心模态框 -->
          <div
            class="modal-content"
            :class="{ expanded: isExpanded }"
          >
            <!-- 顶部：极简输入区 (Spotlight Area) -->
            <div class="input-section" :class="{ 'has-border': isExpanded }">
              <!-- 左侧图标 (状态指示器) -->
              <div class="input-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="24" height="24">
                  <line x1="12" y1="5" x2="12" y2="19"></line>
                  <line x1="5" y1="12" x2="19" y2="12"></line>
                </svg>
              </div>

              <!-- 输入框核心 -->
              <div class="input-wrapper">
                <input
                  ref="inputRef"
                  v-model="inputValue"
                  type="text"
                  class="task-input"
                  placeholder="输入任务标题... (尝试输入 #前端 !高优)"
                  @keydown.enter="handleQuickCreate"
                />

                <!-- 智能解析展示区 (当输入框有值时显示) -->
                <div v-if="inputValue && (detectedTags.length > 0 || hasHighPriority)" class="parsed-tags">
                  <!-- 已有标签（绑定到系统） -->
                  <span
                    v-for="tag in parsedTagsInfo.existingTags"
                    :key="'existing-' + tag.id"
                    class="parsed-tag tag-existing"
                    :style="{ borderColor: tag.color, color: tag.color }"
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="10" height="10">
                      <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
                      <line x1="7" y1="7" x2="7.01" y2="7"></line>
                    </svg>
                    {{ tag.name }}
                  </span>
                  <!-- 新标签（将被创建） -->
                  <span
                    v-for="tagName in parsedTagsInfo.newTags"
                    :key="'new-' + tagName"
                    class="parsed-tag tag-new"
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="10" height="10">
                      <line x1="12" y1="5" x2="12" y2="19"></line>
                      <line x1="5" y1="12" x2="19" y2="12"></line>
                    </svg>
                    {{ tagName }}
                    <span class="new-badge">新</span>
                  </span>
                  <!-- 高优先级标记 -->
                  <span v-if="hasHighPriority" class="parsed-tag tag-priority">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="10" height="10">
                      <line x1="12" y1="19" x2="12" y2="5"></line>
                      <polyline points="5 12 12 5 19 12"></polyline>
                    </svg>
                    高优先级
                  </span>
                </div>
              </div>

              <!-- 右侧操作区 -->
              <div class="input-actions">
                <div class="hint-badge">
                  <kbd>↵</kbd> 创建
                </div>

                <div class="divider"></div>

                <button
                  class="expand-btn"
                  :class="{ active: isExpanded }"
                  @click="toggleExpand"
                  :title="isExpanded ? '收起详情' : '展开详情'"
                >
                  <svg v-if="isExpanded" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="20" height="20">
                    <polyline points="4 14 10 14 10 20"></polyline>
                    <polyline points="20 10 14 10 14 4"></polyline>
                    <line x1="14" y1="10" x2="21" y2="3"></line>
                    <line x1="3" y1="21" x2="10" y2="14"></line>
                  </svg>
                  <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="20" height="20">
                    <polyline points="15 3 21 3 21 9"></polyline>
                    <polyline points="9 21 3 21 3 15"></polyline>
                    <line x1="21" y1="3" x2="14" y2="10"></line>
                    <line x1="3" y1="21" x2="10" y2="14"></line>
                  </svg>
                </button>

                <button
                  class="close-btn"
                  @click="hideWindow"
                  title="关闭 (Esc)"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="20" height="20">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                  </svg>
                </button>
              </div>
            </div>

            <!-- 下半部分：扩展详情区 (Expanded Area) -->
            <div class="expanded-section" :class="{ show: isExpanded }">
              <div class="expanded-content">
                <!-- 左列：描述与基础信息 -->
                <div class="left-column">
                  <div class="form-group">
                    <label class="form-label">描述</label>
                    <textarea
                      v-model="description"
                      class="form-textarea"
                      placeholder="添加更详细的任务描述、代码片段或备注..."
                      rows="4"
                    ></textarea>
                  </div>

                  <!-- 快速分类 (Pills) -->
                  <div class="form-group">
                    <label class="form-label">分类</label>
                    <div class="category-pills">
                      <button
                        v-for="cat in categories"
                        :key="cat.value"
                        class="category-pill"
                        :class="{ active: selectedCategory === cat.value }"
                        @click="selectedCategory = cat.value"
                      >
                        <svg :class="['cat-icon', cat.value]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                          <path v-if="cat.value === 'backend'" d="M16 18l6-6-6-6M8 6l-6 6 6 6"/>
                          <path v-else-if="cat.value === 'database'" d="M12 2C8.13 2 5 3.79 5 6v12c0 2.21 3.13 4 7 4s7-1.79 7-4V6c0-2.21-3.13-4-7-4z"/>
                          <path v-else-if="cat.value === 'feature'" d="M3 3h7v7H3zM14 3h7v7h-7zM14 14h7v7h-7zM3 14h7v7H3z"/>
                          <path v-else d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                        </svg>
                        {{ cat.label }}
                      </button>
                    </div>
                  </div>
                </div>

                <!-- 右列：四象限选择器 -->
                <div class="right-column">
                  <label class="form-label">
                    优先级矩阵
                  </label>

                  <div class="quadrant-grid">
                    <!-- 象限 1: 紧急且重要 -->
                    <button
                      class="quadrant-btn q1"
                      :class="{ active: selectedQuadrant === 'urgent_important' }"
                      @click="selectedQuadrant = 'urgent_important'"
                    >
                      <div class="quadrant-header">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
                          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
                        </svg>
                        <svg v-if="selectedQuadrant === 'urgent_important'" class="check-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                          <polyline points="22 4 12 14.01 9 11.01"></polyline>
                        </svg>
                      </div>
                      <span class="quadrant-label">紧急且重要</span>
                    </button>

                    <!-- 象限 2: 重要不紧急 -->
                    <button
                      class="quadrant-btn q2"
                      :class="{ active: selectedQuadrant === 'not_urgent_important' }"
                      @click="selectedQuadrant = 'not_urgent_important'"
                    >
                      <div class="quadrant-header">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
                          <circle cx="12" cy="12" r="10"></circle>
                          <polyline points="12 6 12 12 16 14"></polyline>
                        </svg>
                        <svg v-if="selectedQuadrant === 'not_urgent_important'" class="check-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                          <polyline points="22 4 12 14.01 9 11.01"></polyline>
                        </svg>
                      </div>
                      <span class="quadrant-label">重要不紧急</span>
                    </button>

                    <!-- 象限 3: 紧急不重要 -->
                    <button
                      class="quadrant-btn q3"
                      :class="{ active: selectedQuadrant === 'urgent_not_important' }"
                      @click="selectedQuadrant = 'urgent_not_important'"
                    >
                      <div class="quadrant-header">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
                          <circle cx="12" cy="12" r="10"></circle>
                          <line x1="12" y1="8" x2="12" y2="12"></line>
                          <line x1="12" y1="16" x2="12.01" y2="16"></line>
                        </svg>
                        <svg v-if="selectedQuadrant === 'urgent_not_important'" class="check-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                          <polyline points="22 4 12 14.01 9 11.01"></polyline>
                        </svg>
                      </div>
                      <span class="quadrant-label">紧急不重要</span>
                    </button>

                    <!-- 象限 4: 不紧急不重要 -->
                    <button
                      class="quadrant-btn q4"
                      :class="{ active: selectedQuadrant === 'not_urgent_not_important' }"
                      @click="selectedQuadrant = 'not_urgent_not_important'"
                    >
                      <div class="quadrant-header">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
                          <circle cx="12" cy="12" r="10"></circle>
                          <line x1="8" y1="12" x2="16" y2="12"></line>
                        </svg>
                        <svg v-if="selectedQuadrant === 'not_urgent_not_important'" class="check-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                          <polyline points="22 4 12 14.01 9 11.01"></polyline>
                        </svg>
                      </div>
                      <span class="quadrant-label">不急不重要</span>
                    </button>
                  </div>
                </div>
              </div>

              <!-- 底部操作栏 -->
              <div class="footer-section">
                <div class="footer-info">
                  <span :class="['ai-status', { disabled: !isAiEnabled }]">
                    <span :class="['status-dot', { active: isAiEnabled }]"></span>
                    {{ isAiEnabled ? 'AI 智能分析已启用' : 'AI 未启用' }}
                  </span>
                  <span v-if="parsedTagsInfo.newTags.length > 0" class="new-tags-hint">
                    <span class="status-dot orange"></span>
                    {{ parsedTagsInfo.newTags.length }} 个新标签将被创建
                  </span>
                </div>
                <div class="footer-actions">
                  <button class="btn-secondary" @click="hideWindow" :disabled="isCreating">
                    取消 (Esc)
                  </button>
                  <button class="btn-primary" @click="handleCreate" :disabled="isCreating">
                    <template v-if="isCreating">
                      <svg class="spinner" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                        <circle cx="12" cy="12" r="10" stroke-dasharray="60" stroke-dashoffset="20"></circle>
                      </svg>
                      创建中...
                    </template>
                    <template v-else>
                      {{ isAiEnabled ? 'AI 智能创建' : '创建任务' }}
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                        <line x1="22" y1="2" x2="11" y2="13"></line>
                        <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
                      </svg>
                    </template>
                  </button>
                </div>
              </div>
            </div>

            <!-- 快捷键提示 (极简模式下显示) -->
            <div v-if="!isExpanded" class="keyboard-hints">
              <span><kbd>Tab</kbd> 展开</span>
              <span><kbd>Esc</kbd> 关闭</span>
            </div>
          </div>

          <!-- 底部提示 -->
          <div class="modal-footer-hint">
            <p>DevAssistant Quick Task v2.0</p>
          </div>
        </div>
      </div>
      <MessageHandler ref="messageHandler" />
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, defineComponent } from 'vue'
import { NConfigProvider, NMessageProvider, darkTheme, useMessage } from 'naive-ui'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { taskApi } from '@/api/taskApi'
import { tagApi } from '@/api/tagApi'
import { aiApi } from '@/api/aiApi'
import type { TaskQuadrant, Tag } from '@/types/task'
import type { TaskClassifyResult } from '@/types/ai'

// 消息处理组件
const MessageHandler = defineComponent({
  setup(_, { expose }) {
    const message = useMessage()
    expose({ message })
    return () => null
  }
})

const appWindow = getCurrentWindow()
const inputRef = ref<HTMLInputElement>()
const messageHandler = ref<{ message: ReturnType<typeof useMessage> } | null>(null)

// 状态
const inputValue = ref('')
const description = ref('')
const isExpanded = ref(false)
const selectedCategory = ref<string>('other')
const selectedQuadrant = ref<TaskQuadrant>('not_urgent_important')
const isCreating = ref(false) // 创建中状态
const isAiEnabled = ref(false) // AI 是否启用

// 标签系统
const allTags = ref<Tag[]>([]) // 所有已有标签
const selectedTagIds = ref<number[]>([]) // 选中的已有标签 ID
const newTagNames = ref<string[]>([]) // 需要创建的新标签名

// 分类选项
const categories = [
  { value: 'backend', label: '后端' },
  { value: 'database', label: '数据库' },
  { value: 'feature', label: '前端' },
  { value: 'docs', label: '文档' },
]

// 预定义的标签颜色
const tagColors = [
  '#8b5cf6', '#6366f1', '#3b82f6', '#06b6d4', '#10b981',
  '#22c55e', '#84cc16', '#eab308', '#f97316', '#ef4444',
]

// 获取随机标签颜色
const getRandomColor = (): string => {
  return tagColors[Math.floor(Math.random() * tagColors.length)]
}

// 加载所有标签
const loadAllTags = async () => {
  try {
    allTags.value = await tagApi.getAllTags()
  } catch (error) {
    console.error('加载标签失败:', error)
  }
}

// 检查 AI 是否启用
const checkAiEnabled = async () => {
  try {
    isAiEnabled.value = await aiApi.isEnabled()
  } catch (error) {
    console.error('检查 AI 状态失败:', error)
    isAiEnabled.value = false
  }
}

// 智能解析标签 - 检测输入中的 #标签
const detectedTags = computed(() => {
  const tags = inputValue.value.match(/#[\w\u4e00-\u9fa5]+/g)
  if (tags) {
    return tags.map(t => t.replace('#', ''))
  }
  return []
})

// 解析标签状态 - 区分已有标签和新标签
const parsedTagsInfo = computed(() => {
  const existingTags: Tag[] = []
  const newTags: string[] = []

  for (const tagName of detectedTags.value) {
    // 查找是否已有同名标签（不区分大小写）
    const existingTag = allTags.value.find(
      t => t.name.toLowerCase() === tagName.toLowerCase()
    )
    if (existingTag) {
      existingTags.push(existingTag)
    } else {
      newTags.push(tagName)
    }
  }

  return { existingTags, newTags }
})

// 检测高优先级标记
const hasHighPriority = computed(() => {
  return inputValue.value.includes('!')
})

// 监听解析到的标签变化，更新选中状态
watch(parsedTagsInfo, ({ existingTags, newTags }) => {
  selectedTagIds.value = existingTags.map(t => t.id!).filter(id => id !== undefined)
  newTagNames.value = newTags
}, { immediate: true })

// 根据解析结果自动设置分类
watch(detectedTags, (tags) => {
  for (const tag of tags) {
    const lowerTag = tag.toLowerCase()
    if (['前端', 'frontend', 'vue', 'react', 'ui'].includes(lowerTag)) {
      selectedCategory.value = 'feature'
      break
    } else if (['后端', 'backend', 'api', 'server', 'java', 'rust'].includes(lowerTag)) {
      selectedCategory.value = 'backend'
      break
    } else if (['数据库', 'database', 'db', 'sql', 'mysql'].includes(lowerTag)) {
      selectedCategory.value = 'database'
      break
    } else if (['文档', 'docs', 'doc', 'readme'].includes(lowerTag)) {
      selectedCategory.value = 'docs'
      break
    }
  }
})

// 根据高优先级标记设置四象限
watch(hasHighPriority, (hasPriority) => {
  if (hasPriority) {
    selectedQuadrant.value = 'urgent_important'
  }
})

// 方法
const hideWindow = async () => {
  // 重置窗口大小（收起状态）
  if (isExpanded.value) {
    await appWindow.setSize({ type: 'Logical', width: 700, height: 130 })
    isExpanded.value = false
  }
  await appWindow.hide()
}

const toggleExpand = async () => {
  isExpanded.value = !isExpanded.value
  // 动态调整窗口大小
  if (isExpanded.value) {
    await appWindow.setSize({ type: 'Logical', width: 700, height: 550 })
  } else {
    await appWindow.setSize({ type: 'Logical', width: 700, height: 130 })
  }
}

const resetForm = () => {
  inputValue.value = ''
  description.value = ''
  isExpanded.value = false
  selectedCategory.value = 'other'
  selectedQuadrant.value = 'not_urgent_important'
  selectedTagIds.value = []
  newTagNames.value = []
}

/**
 * 创建新标签并返回标签 ID 列表
 * @param tagNames 需要创建的标签名称列表
 * @returns 创建的标签 ID 列表
 */
const createNewTags = async (tagNames: string[]): Promise<number[]> => {
  const newTagIds: number[] = []
  for (const name of tagNames) {
    try {
      const tagId = await tagApi.createTag(name, getRandomColor())
      newTagIds.push(tagId)
    } catch (error) {
      console.error(`创建标签 "${name}" 失败:`, error)
    }
  }
  // 刷新标签列表
  await loadAllTags()
  return newTagIds
}

/**
 * 使用 AI 分析任务并创建
 * @param title 任务标题
 * @param desc 任务描述
 */
const createTaskWithAi = async (title: string, desc?: string): Promise<number | null> => {
  try {
    // 调用 AI 分类任务
    const aiResult: TaskClassifyResult = await aiApi.classifyTask(title, desc)

    // 使用 AI 推荐的分类、优先级和四象限
    const taskId = await taskApi.createTask(
      title,
      desc,
      aiResult.category,
      aiResult.priority,
      aiResult.quadrant
    )

    // 处理 AI 推荐的标签
    if (aiResult.suggestedTags && aiResult.suggestedTags.length > 0 && taskId) {
      const aiTagIds: number[] = []

      for (const suggestedTag of aiResult.suggestedTags) {
        // 查找是否已有同名标签
        const existingTag = allTags.value.find(
          t => t.name.toLowerCase() === suggestedTag.toLowerCase()
        )
        if (existingTag && existingTag.id) {
          aiTagIds.push(existingTag.id)
        } else {
          // 创建新标签
          try {
            const newTagId = await tagApi.createTag(suggestedTag, getRandomColor())
            aiTagIds.push(newTagId)
          } catch (error) {
            console.error(`创建 AI 推荐标签 "${suggestedTag}" 失败:`, error)
          }
        }
      }

      // 添加标签到任务
      if (aiTagIds.length > 0) {
        await tagApi.addTagsToTask(taskId, aiTagIds)
      }
    }

    return taskId
  } catch (error) {
    console.error('AI 分析失败:', error)
    return null
  }
}

/**
 * 创建任务并添加标签（不使用 AI）
 * @param title 任务标题
 * @param desc 任务描述
 * @param category 分类
 * @param priority 优先级
 * @param quadrant 四象限
 */
const createTaskWithTags = async (
  title: string,
  desc?: string,
  category?: string,
  priority?: number,
  quadrant?: TaskQuadrant
): Promise<number | null> => {
  try {
    // 创建任务
    const taskId = await taskApi.createTask(
      title,
      desc,
      category || selectedCategory.value,
      priority || (hasHighPriority.value ? 1 : 2),
      quadrant || selectedQuadrant.value
    )

    if (taskId) {
      // 收集所有需要添加的标签 ID
      const allTagIds = [...selectedTagIds.value]

      // 创建新标签并获取 ID
      if (newTagNames.value.length > 0) {
        const createdTagIds = await createNewTags(newTagNames.value)
        allTagIds.push(...createdTagIds)
      }

      // 添加标签到任务
      if (allTagIds.length > 0) {
        await tagApi.addTagsToTask(taskId, allTagIds)
      }
    }

    return taskId
  } catch (error) {
    console.error('创建任务失败:', error)
    return null
  }
}

const handleGlobalKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    e.preventDefault()
    hideWindow()
  } else if (e.key === 'Tab') {
    e.preventDefault()
    toggleExpand()
  }
}

// 快速创建（极简模式）- 优先使用 AI 分析
const handleQuickCreate = async () => {
  if (!inputValue.value.trim()) {
    messageHandler.value?.message.warning('请输入任务标题')
    return
  }

  if (isCreating.value) return
  isCreating.value = true

  try {
    // 清理标题中的标签和优先级标记
    const cleanTitle = inputValue.value
      .replace(/#[\w\u4e00-\u9fa5]+/g, '')
      .replace(/!/g, '')
      .trim()

    let taskId: number | null = null

    // 如果 AI 启用，优先使用 AI 分析创建任务
    if (isAiEnabled.value) {
      messageHandler.value?.message.info('AI 正在分析任务...', { duration: 1500 })
      taskId = await createTaskWithAi(cleanTitle, undefined)

      if (taskId) {
        // AI 创建成功后，还需要添加用户手动输入的标签
        const userTagIds = [...selectedTagIds.value]
        if (newTagNames.value.length > 0) {
          const createdTagIds = await createNewTags(newTagNames.value)
          userTagIds.push(...createdTagIds)
        }
        if (userTagIds.length > 0) {
          await tagApi.addTagsToTask(taskId, userTagIds)
        }
        messageHandler.value?.message.success('任务已通过 AI 智能创建!')
      }
    }

    // 如果 AI 未启用或 AI 创建失败，使用传统方式创建
    if (!taskId) {
      taskId = await createTaskWithTags(cleanTitle)
      if (taskId) {
        messageHandler.value?.message.success('任务已快速创建!')
      }
    }

    if (taskId) {
      resetForm()
      hideWindow()
    } else {
      messageHandler.value?.message.error('创建任务失败')
    }
  } catch (error) {
    console.error('创建任务失败:', error)
    messageHandler.value?.message.error('创建任务失败')
  } finally {
    isCreating.value = false
  }
}

// 完整创建（展开模式）- 优先使用 AI 分析
const handleCreate = async () => {
  if (!inputValue.value.trim()) {
    messageHandler.value?.message.warning('请输入任务标题')
    return
  }

  if (isCreating.value) return
  isCreating.value = true

  try {
    // 清理标题中的标签和优先级标记
    const cleanTitle = inputValue.value
      .replace(/#[\w\u4e00-\u9fa5]+/g, '')
      .replace(/!/g, '')
      .trim()

    const desc = description.value || undefined
    let taskId: number | null = null

    // 如果 AI 启用，优先使用 AI 分析创建任务
    if (isAiEnabled.value) {
      messageHandler.value?.message.info('AI 正在分析任务...', { duration: 1500 })
      taskId = await createTaskWithAi(cleanTitle, desc)

      if (taskId) {
        // AI 创建成功后，还需要添加用户手动输入的标签
        const userTagIds = [...selectedTagIds.value]
        if (newTagNames.value.length > 0) {
          const createdTagIds = await createNewTags(newTagNames.value)
          userTagIds.push(...createdTagIds)
        }
        if (userTagIds.length > 0) {
          await tagApi.addTagsToTask(taskId, userTagIds)
        }
        messageHandler.value?.message.success('任务已通过 AI 智能创建!')
      }
    }

    // 如果 AI 未启用或 AI 创建失败，使用传统方式创建（展开模式使用用户选择的分类和四象限）
    if (!taskId) {
      taskId = await createTaskWithTags(
        cleanTitle,
        desc,
        selectedCategory.value,
        hasHighPriority.value ? 1 : 2,
        selectedQuadrant.value
      )
      if (taskId) {
        messageHandler.value?.message.success('任务创建成功!')
      }
    }

    if (taskId) {
      resetForm()
      hideWindow()
    } else {
      messageHandler.value?.message.error('创建任务失败')
    }
  } catch (error) {
    console.error('创建任务失败:', error)
    messageHandler.value?.message.error('创建任务失败')
  } finally {
    isCreating.value = false
  }
}

// 初始化
onMounted(async () => {
  // 加载标签和检查 AI 状态
  await Promise.all([
    loadAllTags(),
    checkAiEnabled()
  ])

  nextTick(() => {
    inputRef.value?.focus()
  })

  // 监听窗口焦点变化
  appWindow.onFocusChanged(async ({ payload: focused }) => {
    if (focused) {
      // 窗口获得焦点时刷新数据并聚焦输入框
      await Promise.all([
        loadAllTags(),
        checkAiEnabled()
      ])
      nextTick(() => {
        inputRef.value?.focus()
      })
    } else {
      // 窗口失去焦点时自动隐藏
      hideWindow()
    }
  })
})
</script>

<style scoped>
.quick-task-wrapper {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 0;
  background: transparent;
}

.modal-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
}

.modal-content {
  width: 100%;
  background: rgba(15, 23, 42, 0.98);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(139, 92, 246, 0.3);
  border-radius: 8px;
  box-shadow:
    0 25px 50px -12px rgba(0, 0, 0, 0.5),
    0 0 0 1px rgba(139, 92, 246, 0.2);
  overflow: hidden;
  transition: all 0.3s ease;
}

/* 输入区样式 */
.input-section {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px 24px;
  border-bottom: 1px solid transparent;
  transition: border-color 0.3s ease;
}

.input-section.has-border {
  border-bottom-color: rgba(51, 65, 85, 0.5);
}

.input-icon {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #8b5cf6 0%, #6366f1 100%);
  border-radius: 12px;
  box-shadow: 0 8px 20px rgba(139, 92, 246, 0.3);
  flex-shrink: 0;
  color: #fff;
}

.input-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-input {
  width: 100%;
  background: transparent;
  border: none;
  outline: none;
  font-size: 20px;
  color: #e2e8f0;
  font-weight: 400;
  height: 48px;
  font-family: inherit;
}

.task-input::placeholder {
  color: #64748b;
}

.parsed-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.parsed-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 500;
}

/* 已有标签样式 */
.parsed-tag.tag-existing {
  background: rgba(139, 92, 246, 0.15);
  border: 1px solid;
}

/* 新标签样式 */
.parsed-tag.tag-new {
  background: rgba(34, 197, 94, 0.15);
  color: #86efac;
  border: 1px solid rgba(34, 197, 94, 0.4);
}

.parsed-tag .new-badge {
  font-size: 9px;
  padding: 1px 4px;
  background: rgba(34, 197, 94, 0.3);
  border-radius: 3px;
  margin-left: 4px;
}

/* 高优先级标签 */
.parsed-tag.tag-priority {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.input-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.hint-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #64748b;
  padding: 6px 12px;
  background: rgba(51, 65, 85, 0.5);
  border: 1px solid rgba(71, 85, 105, 0.6);
  border-radius: 8px;
}

.hint-badge kbd {
  font-size: 11px;
  font-family: monospace;
}

.divider {
  width: 1px;
  height: 24px;
  background: rgba(51, 65, 85, 0.8);
  margin: 0 4px;
}

.expand-btn,
.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  color: #64748b;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.expand-btn:hover,
.close-btn:hover {
  background: rgba(51, 65, 85, 0.5);
  color: #e2e8f0;
}

.expand-btn.active {
  background: rgba(139, 92, 246, 0.2);
  color: #a78bfa;
}

/* 展开区域样式 */
.expanded-section {
  max-height: 0;
  opacity: 0;
  overflow: hidden;
  transition: all 0.4s ease;
}

.expanded-section.show {
  max-height: 600px;
  opacity: 1;
}

.expanded-content {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 32px;
  padding: 24px;
}

.left-column,
.right-column {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.form-label {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.form-textarea {
  width: 100%;
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.8);
  border-radius: 12px;
  padding: 14px;
  color: #cbd5e1;
  font-size: 14px;
  font-family: inherit;
  resize: none;
  outline: none;
  transition: all 0.2s ease;
}

.form-textarea::placeholder {
  color: #475569;
}

.form-textarea:focus {
  border-color: rgba(139, 92, 246, 0.5);
  box-shadow: 0 0 0 3px rgba(139, 92, 246, 0.1);
}

/* 分类按钮 */
.category-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.category-pill {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  background: rgba(51, 65, 85, 0.3);
  border: 1px solid rgba(51, 65, 85, 0.6);
  border-radius: 10px;
  color: #94a3b8;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.category-pill:hover {
  background: rgba(51, 65, 85, 0.5);
  border-color: rgba(139, 92, 246, 0.4);
  color: #e2e8f0;
}

.category-pill.active {
  background: rgba(139, 92, 246, 0.15);
  border-color: rgba(139, 92, 246, 0.5);
  color: #c4b5fd;
}

/* 四象限选择器 */
.quadrant-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  height: 200px;
}

.quadrant-btn {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 14px;
  border-radius: 12px;
  border: 1px solid rgba(51, 65, 85, 0.6);
  background: rgba(51, 65, 85, 0.2);
  cursor: pointer;
  transition: all 0.2s ease;
}

.quadrant-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.quadrant-label {
  font-size: 12px;
  font-weight: 500;
}

.check-icon {
  flex-shrink: 0;
}

/* Q1: 紧急且重要 - 红色 */
.quadrant-btn.q1 {
  color: #64748b;
}
.quadrant-btn.q1:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.4);
}
.quadrant-btn.q1.active {
  background: rgba(239, 68, 68, 0.15);
  border-color: rgba(239, 68, 68, 0.5);
  color: #fca5a5;
  box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.1);
}
.quadrant-btn.q1.active .check-icon {
  color: #ef4444;
}

/* Q2: 重要不紧急 - 橙色 */
.quadrant-btn.q2 {
  color: #64748b;
}
.quadrant-btn.q2:hover {
  background: rgba(249, 115, 22, 0.1);
  border-color: rgba(249, 115, 22, 0.4);
}
.quadrant-btn.q2.active {
  background: rgba(249, 115, 22, 0.15);
  border-color: rgba(249, 115, 22, 0.5);
  color: #fdba74;
  box-shadow: 0 0 0 3px rgba(249, 115, 22, 0.1);
}
.quadrant-btn.q2.active .check-icon {
  color: #f97316;
}

/* Q3: 紧急不重要 - 蓝色 */
.quadrant-btn.q3 {
  color: #64748b;
}
.quadrant-btn.q3:hover {
  background: rgba(59, 130, 246, 0.1);
  border-color: rgba(59, 130, 246, 0.4);
}
.quadrant-btn.q3.active {
  background: rgba(59, 130, 246, 0.15);
  border-color: rgba(59, 130, 246, 0.5);
  color: #93c5fd;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}
.quadrant-btn.q3.active .check-icon {
  color: #3b82f6;
}

/* Q4: 不紧急不重要 - 灰色 */
.quadrant-btn.q4 {
  color: #64748b;
}
.quadrant-btn.q4:hover {
  background: rgba(100, 116, 139, 0.2);
  border-color: rgba(100, 116, 139, 0.5);
}
.quadrant-btn.q4.active {
  background: rgba(100, 116, 139, 0.25);
  border-color: rgba(100, 116, 139, 0.6);
  color: #cbd5e1;
  box-shadow: 0 0 0 3px rgba(100, 116, 139, 0.1);
}
.quadrant-btn.q4.active .check-icon {
  color: #94a3b8;
}

/* 底部操作栏 */
.footer-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: rgba(15, 23, 42, 0.5);
  border-top: 1px solid rgba(51, 65, 85, 0.5);
}

.footer-info {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 12px;
  color: #64748b;
}

.ai-status,
.system-status,
.new-tags-hint {
  display: flex;
  align-items: center;
  gap: 6px;
}

.ai-status.disabled {
  color: #475569;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #475569;
}

.status-dot.active {
  background: #6366f1;
  box-shadow: 0 0 8px rgba(99, 102, 241, 0.6);
  animation: pulse 2s infinite;
}

.status-dot.green {
  background: #22c55e;
  box-shadow: 0 0 8px rgba(34, 197, 94, 0.6);
}

.status-dot.orange {
  background: #f97316;
  box-shadow: 0 0 8px rgba(249, 115, 22, 0.6);
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

/* 加载动画 */
.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* 禁用状态 */
.btn-primary:disabled,
.btn-secondary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.footer-actions {
  display: flex;
  gap: 12px;
}

.btn-secondary {
  padding: 10px 20px;
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 14px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-secondary:hover {
  background: rgba(51, 65, 85, 0.5);
  color: #e2e8f0;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 24px;
  background: linear-gradient(135deg, #8b5cf6 0%, #6366f1 100%);
  border: none;
  color: #fff;
  font-size: 14px;
  font-weight: 500;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 8px 20px rgba(139, 92, 246, 0.25);
}

.btn-primary:hover {
  background: linear-gradient(135deg, #a78bfa 0%, #818cf8 100%);
  transform: translateY(-1px);
  box-shadow: 0 10px 25px rgba(139, 92, 246, 0.35);
}

/* 快捷键提示 */
.keyboard-hints {
  display: flex;
  justify-content: center;
  gap: 20px;
  padding: 14px 24px;
  border-top: 1px solid rgba(51, 65, 85, 0.4);
  font-size: 11px;
  color: #475569;
}

.keyboard-hints kbd {
  display: inline-block;
  padding: 3px 8px;
  background: rgba(51, 65, 85, 0.5);
  border: 1px solid rgba(71, 85, 105, 0.6);
  border-radius: 5px;
  font-size: 10px;
  font-family: monospace;
  margin: 0 3px;
}

/* 底部提示 - 隐藏，因为窗口填满 */
.modal-footer-hint {
  display: none;
}
</style>
