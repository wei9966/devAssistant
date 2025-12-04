# ActivityTimelineItem 组件使用指南

## 概述

`ActivityTimelineItem` 是一个用于在时间轴中显示活动分组和相关截图的 Vue 组件。它提供了一个优雅的界面来展示用户的工作活动，包括活动类型、描述和截图缩略图。

## 功能特性

### 核心功能

1. **活动类型标签显示** - 使用不同颜色区分不同类型的活动
2. **智能描述展示** - 自动截断长描述，悬停显示完整内容
3. **截图网格展示** - 110x60px 缩略图网格布局
4. **点击预览** - 支持点击缩略图查看完整图片
5. **性能优化** - 分批并行加载缩略图，避免阻塞

### 视觉效果

- 悬停动画效果（缩放、阴影）
- 响应式设计（移动端友好）
- 暗色主题适配
- 平滑过渡动画

## 安装和导入

```typescript
import ActivityTimelineItem, { type ActivityGroup } from '@/components/screenshot/ActivityTimelineItem.vue'
import type { ScreenshotRecord } from '@/api/screenshotApi'
```

## 基本使用

### 最简单的例子

```vue
<template>
  <ActivityTimelineItem
    :activity="myActivity"
    @preview="handlePreview"
  />
</template>

<script setup lang="ts">
import { ref } from 'vue'
import ActivityTimelineItem, { type ActivityGroup } from '@/components/screenshot/ActivityTimelineItem.vue'
import type { ScreenshotRecord } from '@/api/screenshotApi'

const myActivity: ActivityGroup = {
  activityType: 'coding',
  description: '开发用户认证模块',
  screenshots: [
    {
      id: 1,
      capturedAt: '2025-12-04T09:30:00Z',
      appName: 'VS Code',
      windowTitle: 'auth.service.ts',
      activityType: 'coding',
      description: '编写认证服务',
      keyContent: 'login function',
      screenshotPath: '/path/to/screenshot.png',
      processingTimeMs: 150
    }
  ]
}

function handlePreview(screenshot: ScreenshotRecord) {
  console.log('Preview screenshot:', screenshot)
}
</script>
```

## 数据结构

### ActivityGroup 接口

```typescript
interface ActivityGroup {
  activityType: string        // 活动类型
  description: string          // 活动描述
  screenshots: ScreenshotRecord[]  // 截图列表
}
```

### 支持的活动类型

| 类型 | 中文名称 | 颜色 |
|------|---------|------|
| `coding` | 编码开发 | 绿色 |
| `browsing` | 网页浏览 | 蓝色 |
| `document` | 文档处理 | 黄色 |
| `meeting` | 会议沟通 | 红色 |
| `communication` | 即时通讯 | 紫色 |
| `other` | 其他 | 灰色 |

## 高级用法

### 与弹窗组件集成

```vue
<template>
  <div class="timeline-container">
    <!-- 活动列表 -->
    <ActivityTimelineItem
      v-for="activity in activities"
      :key="activity.activityType"
      :activity="activity"
      @preview="handlePreview"
    />

    <!-- 预览弹窗 -->
    <ScreenshotDetailModal
      v-if="selectedScreenshot"
      v-model:show="showModal"
      :screenshot="selectedScreenshot"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import ActivityTimelineItem from '@/components/screenshot/ActivityTimelineItem.vue'
import ScreenshotDetailModal from '@/components/screenshot/ScreenshotDetailModal.vue'

const showModal = ref(false)
const selectedScreenshot = ref<ScreenshotRecord | null>(null)

function handlePreview(screenshot: ScreenshotRecord) {
  selectedScreenshot.value = screenshot
  showModal.value = true
}
</script>
```

### 从 API 加载数据

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import ActivityTimelineItem, { type ActivityGroup } from '@/components/screenshot/ActivityTimelineItem.vue'
import { screenshotApi } from '@/api/screenshotApi'

const activities = ref<ActivityGroup[]>([])
const loading = ref(true)

onMounted(async () => {
  try {
    // 获取截图列表
    const response = await screenshotApi.list({
      startDate: '2025-12-04',
      endDate: '2025-12-04',
      pageSize: 50
    })

    // 按活动类型分组
    const grouped = groupByActivityType(response.records)
    activities.value = grouped
  } catch (error) {
    console.error('加载失败:', error)
  } finally {
    loading.value = false
  }
})

// 按活动类型分组截图
function groupByActivityType(screenshots: ScreenshotRecord[]): ActivityGroup[] {
  const groups = new Map<string, ScreenshotRecord[]>()

  screenshots.forEach(shot => {
    const type = shot.activityType
    if (!groups.has(type)) {
      groups.set(type, [])
    }
    groups.get(type)!.push(shot)
  })

  return Array.from(groups.entries()).map(([activityType, screenshots]) => ({
    activityType,
    description: generateDescription(screenshots),
    screenshots
  }))
}

function generateDescription(screenshots: ScreenshotRecord[]): string {
  // 生成活动描述的逻辑
  const apps = [...new Set(screenshots.map(s => s.appName).filter(Boolean))]
  return `在 ${apps.join('、')} 中工作，共 ${screenshots.length} 次操作`
}
</script>
```

## 性能优化技巧

### 1. 分批加载

组件内部已实现分批并行加载：

```typescript
// 每批加载 5 张缩略图
const batchSize = 5
for (let i = 0; i < screenshots.length; i += batchSize) {
  const batch = screenshots.slice(i, i + batchSize)
  await Promise.all(batch.map(s => loadThumbnail(s)))
}
```

### 2. 虚拟滚动

如果活动数量很多，建议使用虚拟滚动：

```vue
<template>
  <n-virtual-list
    :item-size="200"
    :items="activities"
  >
    <template #default="{ item }">
      <ActivityTimelineItem
        :activity="item"
        @preview="handlePreview"
      />
    </template>
  </n-virtual-list>
</template>
```

### 3. 缓存策略

组件内部已实现缓存机制，避免重复加载：

```typescript
const thumbnails = ref<Record<number, string>>({})
// 只加载未缓存的缩略图
const toLoad = screenshots.filter(s => !thumbnails.value[s.id])
```

## 样式自定义

### 修改缩略图尺寸

```vue
<style scoped>
/* 自定义缩略图尺寸 */
:deep(.screenshot-thumb) {
  width: 150px;   /* 默认 110px */
  height: 100px;  /* 默认 70px */
}
</style>
```

### 修改活动类型颜色

```vue
<style scoped>
/* 自定义编码活动颜色 */
:deep(.badge-coding) {
  background: rgba(34, 197, 94, 0.15);
  color: rgb(34, 197, 94);
}
</style>
```

## 事件处理

### preview 事件

当用户点击截图缩略图时触发：

```typescript
interface Emits {
  (e: 'preview', screenshot: ScreenshotRecord): void
}
```

**使用示例：**

```vue
<ActivityTimelineItem
  :activity="activity"
  @preview="(screenshot) => {
    // 打开预览弹窗
    openPreviewModal(screenshot)

    // 记录用户行为
    logUserAction('screenshot_preview', screenshot.id)

    // 更新查看次数
    incrementViewCount(screenshot.id)
  }"
/>
```

## 常见问题

### Q1: 为什么缩略图加载很慢？

**答：** 检查以下几点：
1. 后端是否正确生成缩略图（而非返回原图）
2. 网络连接是否正常
3. 是否有太多截图同时加载（组件已实现分批加载）

### Q2: 如何显示加载进度？

**答：** 可以监听加载状态：

```vue
<script setup lang="ts">
const loadingProgress = computed(() => {
  const total = activity.screenshots.length
  const loaded = Object.keys(thumbnails.value).length
  return Math.round((loaded / total) * 100)
})
</script>

<template>
  <div class="loading-progress">
    <n-progress :percentage="loadingProgress" />
  </div>
</template>
```

### Q3: 如何处理图片加载失败？

**答：** 组件已内置错误处理，会显示占位图标。如需自定义：

```typescript
async function loadThumbnail(screenshot: ScreenshotRecord) {
  try {
    const base64 = await screenshotApi.getImage(screenshot.screenshotPath, true)
    thumbnails.value[screenshot.id] = base64
  } catch (error) {
    console.error('加载失败:', error)
    // 设置默认占位图
    thumbnails.value[screenshot.id] = DEFAULT_PLACEHOLDER_IMAGE
  }
}
```

## 完整示例

查看完整的可运行示例：

```
src/components/screenshot/ActivityTimelineItem.example.vue
```

## 相关组件

- **ScreenshotGrid** - 截图网格展示
- **ScreenshotDetailModal** - 截图详情弹窗
- **DateNavigation** - 日期导航

## 参考文档

- [Naive UI - Popover](https://www.naiveui.com/zh-CN/os-theme/components/popover)
- [Naive UI - Spin](https://www.naiveui.com/zh-CN/os-theme/components/spin)
- [Screenshot API 文档](../src/api/screenshotApi.ts)

## 变更日志

### v1.0.0 (2025-12-04)

- 初始版本发布
- 支持活动类型标签显示
- 实现缩略图网格展示
- 添加悬停描述展示
- 实现性能优化（分批加载）
- 支持响应式设计
