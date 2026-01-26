# AI Chat Widget 组件

这些组件用于在 AI 对话界面中展示结构化数据，提供美观的数据可视化和交互功能。

## 组件列表

### 1. TaskListWidget - 任务列表展示

展示任务列表，按四象限分组显示。

```vue
<template>
  <TaskListWidget
    :data="tasksData"
    @complete="handleTaskComplete"
  />
</template>

<script setup>
import { TaskListWidget } from '@/components/aiChat/widgets';

const tasksData = {
  total: 5,
  tasks: [
    {
      id: 1,
      title: '修复登录 bug',
      quadrant: 'urgent_important',
      status: 'todo',
      due_date: '2025-12-26',
      tags: ['bug', 'urgent']
    }
  ]
};

const handleTaskComplete = (task) => {
  console.log('完成任务:', task);
};
</script>
```

**Props:**
- `data: TasksResponse` - 任务数据

**Events:**
- `complete: (task) => void` - 完成任务事件

---

### 2. SqlListWidget - SQL 列表展示

展示 SQL 记录列表，支持代码高亮、复制和收藏功能。

```vue
<template>
  <SqlListWidget
    :data="sqlData"
    @copy="handleCopy"
    @favorite="handleFavorite"
  />
</template>

<script setup>
import { SqlListWidget } from '@/components/aiChat/widgets';

const sqlData = {
  total: 3,
  sql_list: [
    {
      id: 1,
      content: 'SELECT * FROM users WHERE id = 1',
      sql_type: 'SELECT',
      tables: ['users'],
      is_favorite: false,
      created_at: '2025-12-25 10:30:00',
      note: '查询用户信息'
    }
  ]
};

const handleCopy = (content) => {
  console.log('复制 SQL:', content);
};

const handleFavorite = (sql) => {
  console.log('收藏状态变更:', sql);
};
</script>
```

**Props:**
- `data: SqlSearchResponse` - SQL 数据

**Events:**
- `copy: (content: string) => void` - 复制事件
- `favorite: (sql) => void` - 收藏/取消收藏事件

---

### 3. TimeStatsWidget - 时间统计展示

展示时间统计数据，包括总时长、分类统计和应用使用情况。

```vue
<template>
  <TimeStatsWidget :data="timeStats" />
</template>

<script setup>
import { TimeStatsWidget } from '@/components/aiChat/widgets';

const timeStats = {
  date_range: 'today',
  total_minutes: 494,
  active_time: '08:55 - 17:09',
  by_category: [
    {
      category: '编程',
      minutes: 182,
      percentage: 36.8,
      apps: ['VS Code', 'DataGrip']
    },
    {
      category: '文档',
      minutes: 97,
      percentage: 19.6,
      apps: ['Word', 'Notion']
    }
  ],
  by_app: [
    { app: 'VS Code', minutes: 150, percentage: 30.4 },
    { app: 'Chrome', minutes: 55, percentage: 11.1 }
  ]
};
</script>
```

**Props:**
- `data: TimeStatsResponse` - 时间统计数据

---

### 4. ReportWidget - 报告展示

展示日报或周报，支持 Markdown 渲染、复制和导出功能。

```vue
<template>
  <ReportWidget
    :data="reportData"
    @copy="handleCopy"
    @export="handleExport"
  />
</template>

<script setup>
import { ReportWidget } from '@/components/aiChat/widgets';

// 日报数据
const dailyReport = {
  date: '2025-12-25',
  report: {
    summary: '今日完成 5 项任务，专注工作 4.5 小时',
    work_content: [
      {
        category: '系统配置与维护',
        items: [
          '整理所有工厂的排产大屏地址',
          '修改生产节拍大屏系数'
        ]
      }
    ],
    time_stats: {
      total_hours: 8.2,
      active_period: '08:55 - 17:09',
      breakdown: [
        { category: '编程', hours: 3.0, percentage: 37 },
        { category: '文档', hours: 1.6, percentage: 20 }
      ]
    },
    highlights: [
      '完成排产大屏配置更新，覆盖所有工厂',
      '解决了一个影响用户体验的关键 bug'
    ]
  },
  markdown: '## 2025-12-25 工作日报\n\n...'
};

// 周报数据
const weeklyReport = {
  week: '2025-W52',
  date_range: '2025-12-23 ~ 2025-12-27',
  report: {
    summary: '本周完成 12 项任务，总工作时长 42 小时',
    completed_tasks: {
      total: 12,
      by_category: [
        { category: '系统配置与维护', count: 4 },
        { category: '功能优化与修复', count: 5 }
      ]
    }
  },
  markdown: '## 2025-W52 周报\n\n...'
};

const handleCopy = () => {
  console.log('已复制报告');
};

const handleExport = () => {
  console.log('已导出报告');
};
</script>
```

**Props:**
- `data: DailyReportResponse | WeeklyReportResponse` - 报告数据

**Events:**
- `copy: () => void` - 复制事件
- `export: () => void` - 导出事件

---

## 样式特性

所有 Widget 组件都遵循项目主题系统，支持：

- ✅ **主题自适应** - 使用 CSS 变量，自动适配深色/浅色主题
- ✅ **响应式设计** - 适配不同屏幕尺寸
- ✅ **卡片式布局** - 统一的卡片样式，圆角边框
- ✅ **悬停效果** - 平滑的悬停动画和阴影效果
- ✅ **渐变色彩** - 使用主题渐变色增强视觉效果
- ✅ **无障碍支持** - 良好的对比度和可读性

## 主题变量

组件使用以下主题变量（来自 `src/themes/variables.css`）：

```css
/* 背景色 */
--card-bg
--card-hover-bg
--bg-surface
--bg-elevated
--bg-base

/* 文字颜色 */
--text-primary
--text-secondary
--text-muted
--text-dim

/* 强调色 */
--accent-primary
--accent-secondary
--accent-glow

/* 状态色 */
--success
--warning
--error
--info

/* 边框 */
--border-default
--border-hover

/* 进度条 */
--progress-bg
--progress-fill

/* 阴影 */
--shadow-sm
--shadow-md
```

## 注意事项

1. **禁止硬编码颜色** - 必须使用 CSS 变量
2. **保持一致性** - 所有 Widget 遵循相同的设计语言
3. **性能优化** - 使用 CSS transition 而非 animation
4. **可访问性** - 确保足够的颜色对比度

## 开发建议

如需添加新的 Widget 组件：

1. 复制现有组件作为模板
2. 严格遵循主题变量使用规范
3. 保持与其他 Widget 一致的样式风格
4. 在 `index.ts` 中导出新组件和类型定义
5. 更新此 README 文档
