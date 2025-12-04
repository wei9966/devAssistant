# Screenshot Components

截图相关的 Vue 组件集合。

## 组件列表

### ActivityTimelineItem

时间轴活动项组件，用于显示活动分组及其相关截图。

#### 功能特性

- 活动类型标签显示
- 悬停展示完整描述（Popover）
- 截图缩略图网格展示（110x60px）
- 支持点击预览完整图片
- 懒加载缩略图，分批并行加载
- 响应式设计

#### Props

```typescript
interface ActivityGroup {
  activityType: string        // 活动类型: coding, browsing, document, meeting, communication, other
  description: string          // 活动描述
  screenshots: ScreenshotRecord[]  // 截图列表
}

interface Props {
  activity: ActivityGroup
}
```

#### Events

```typescript
interface Emits {
  (e: 'preview', screenshot: ScreenshotRecord): void  // 点击截图时触发
}
```

#### 使用示例

```vue
<template>
  <ActivityTimelineItem
    :activity="activityGroup"
    @preview="handlePreview"
  />
</template>

<script setup lang="ts">
import ActivityTimelineItem, { type ActivityGroup } from '@/components/screenshot/ActivityTimelineItem.vue'
import type { ScreenshotRecord } from '@/api/screenshotApi'

const activityGroup: ActivityGroup = {
  activityType: 'coding',
  description: '开发新功能模块，完成用户认证和权限管理功能',
  screenshots: [
    // ScreenshotRecord 数组
  ]
}

function handlePreview(screenshot: ScreenshotRecord) {
  // 处理截图预览
}
</script>
```

### ScreenshotGrid

截图网格展示组件。

#### 功能特性

- 响应式网格布局
- 缩略图懒加载
- 活动类型标签
- 截图信息展示

#### 使用示例

```vue
<template>
  <ScreenshotGrid
    :screenshots="screenshots"
    @select="handleSelect"
  />
</template>

<script setup lang="ts">
import ScreenshotGrid from '@/components/screenshot/ScreenshotGrid.vue'
import type { ScreenshotRecord } from '@/api/screenshotApi'

const screenshots = ref<ScreenshotRecord[]>([])

function handleSelect(screenshot: ScreenshotRecord) {
  // 处理选择
}
</script>
```

### ScreenshotDetailModal

截图详情弹窗组件。

#### 功能特性

- 完整尺寸图片预览
- 截图元数据展示
- 键盘导航支持

### DateNavigation

日期导航组件。

#### 功能特性

- 日期选择
- 快速导航（今天、昨天等）
- 日期范围筛选

## 设计规范

### 颜色系统

- **编码开发 (coding)**: 绿色 `rgb(52, 211, 153)`
- **网页浏览 (browsing)**: 蓝色 `rgb(96, 165, 250)`
- **文档处理 (document)**: 黄色 `rgb(251, 191, 36)`
- **会议沟通 (meeting)**: 红色 `rgb(248, 113, 113)`
- **即时通讯 (communication)**: 紫色 `rgb(167, 139, 250)`
- **其他 (other)**: 灰色 `rgb(148, 163, 184)`

### 尺寸规范

- **缩略图**: 110x70px (ActivityTimelineItem)
- **网格项**: 280px 最小宽度 (ScreenshotGrid)
- **圆角**: 8px (缩略图), 12px (卡片)
- **间距**: 8px (缩略图间距), 12-16px (内边距)

### 动画效果

- **悬停缩放**: `scale(1.05)`, 200ms ease
- **阴影**: `0 4px 12px rgba(99, 102, 241, 0.3)`
- **过渡**: all 0.2s - 0.3s ease

## 性能优化

### 图片加载策略

1. **分批并行加载**: 每批 5-10 张缩略图
2. **懒加载**: 使用 `loading="lazy"` 属性
3. **缓存机制**: 已加载的缩略图存储在 `ref` 中
4. **加载状态**: 显示加载指示器

### 示例：分批加载实现

```typescript
// 分批并行加载缩略图
const batchSize = 5
for (let i = 0; i < screenshots.length; i += batchSize) {
  const batch = screenshots.slice(i, i + batchSize)
  await Promise.all(batch.map(s => loadThumbnail(s)))
}
```

## API 依赖

### screenshotApi

```typescript
// 获取截图列表
await screenshotApi.list(params)

// 获取缩略图 (Base64)
await screenshotApi.getImage(path, true)

// 获取完整图片 (Base64)
await screenshotApi.getImage(path, false)
```

## 类型定义

```typescript
// 截图记录
interface ScreenshotRecord {
  id: number
  capturedAt: string
  appName: string | null
  windowTitle: string | null
  activityType: string
  description: string
  keyContent: string | null
  screenshotPath: string | null
  processingTimeMs: number | null
}

// 活动分组
interface ActivityGroup {
  activityType: string
  description: string
  screenshots: ScreenshotRecord[]
}
```
