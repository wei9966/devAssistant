# DevAssistant 项目架构文档

> 最后更新：2025-12-06

## 一、项目概述

DevAssistant 是一款基于 Tauri + Vue 3 的桌面效率工具，集成了任务管理、屏幕截图分析、AI 辅助、应用启动器、SQL 历史管理等多种功能。

### 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | Vue 3 + TypeScript |
| 状态管理 | Pinia |
| UI 组件库 | Naive UI |
| 路由 | Vue Router |
| 桌面框架 | Tauri 2.0 |
| 后端语言 | Rust |
| 数据库 | SQLite |
| AI 集成 | DeepSeek / 阿里通义千问 / 自定义 |
| VLM 集成 | Qwen-VL / DeepSeek-VL / OpenAI / Claude 等 |

---

## 二、目录结构总览

```
DevAssistant/
├── src/                          # 前端源码
│   ├── api/                      # API 调用封装
│   ├── components/               # Vue 组件
│   ├── stores/                   # Pinia 状态管理
│   ├── types/                    # TypeScript 类型定义
│   ├── views/                    # 页面视图
│   ├── router/                   # 路由配置
│   ├── launcher/                 # 启动器窗口
│   ├── quick-task/               # 快速任务窗口
│   ├── sql-panel/                # SQL 面板窗口
│   ├── clipboard-history/        # 剪贴板历史窗口
│   ├── task-calendar/            # 任务日历窗口
│   ├── task-float/               # 任务浮窗
│   └── utils/                    # 工具函数
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── commands/             # Tauri 命令（前端可调用）
│   │   ├── services/             # 业务逻辑服务
│   │   ├── models/               # 数据模型
│   │   ├── db/                   # 数据库管理
│   │   ├── prompts/              # AI 提示词
│   │   └── utils/                # 工具函数
│   └── config/                   # 配置文件
└── docs/                         # 文档
```

---

## 三、前端架构

### 3.1 页面视图 (src/views/)

| 文件 | 功能描述 |
|------|---------|
| `TaskBoard.vue` | 任务看板 - 三列看板（待办/进行中/已完成），支持拖拽、AI 分类、AI 生成子任务 |
| `SqlHistory.vue` | SQL 历史 - SQL 记录管理、收藏、多标签分类、AI 自动分类 |
| `WorkLog.vue` | 工作日志 - 日常工作记录、AI 生成日报 |
| `AppLauncher.vue` | 应用启动器 - 系统应用管理、分类、搜索、AI 智能分类 |
| `ScreenshotGallery.vue` | 截图回顾 - 按日期查看活动截图、时间轴展示、渐进式加载 |
| `DataStatistics.vue` | 数据统计 - 热力图、日趋势、应用使用、活动类型统计 |
| `NotificationCenter.vue` | 通知中心 - 系统通知、日报/周报生成 |
| `Settings.vue` | 设置页面 - AI/VLM 配置、数据库、通知等系统设置 |

### 3.2 组件库 (src/components/)

#### 任务相关组件
| 文件 | 功能描述 |
|------|---------|
| `TaskCard.vue` | 任务卡片 - 显示单个任务信息和操作菜单 |
| `TaskDetailModal.vue` | 任务详情弹窗 - 完整信息展示和编辑 |
| `TaskImport.vue` | 任务导入组件 |
| `QuadrantSelector.vue` | 四象限选择器 - 优先级矩阵选择 |
| `CategorySelector.vue` | 分类选择器 |
| `PrioritySelector.vue` | 优先级选择器 |
| `TagSelector.vue` | 标签选择器 |
| `TagManager.vue` | 标签管理弹窗 |
| `Badge.vue` | 徽章组件 |
| `QuickTaskModal.vue` | 快速任务创建弹窗 |

#### 应用启动器组件 (components/appLauncher/)
| 文件 | 功能描述 |
|------|---------|
| `CyberpunkLauncher.vue` | 赛博朋克风格启动器主界面 |
| `AppCard.vue` | 应用卡片 |
| `AppSearchBar.vue` | 应用搜索栏 |
| `AppCategoryFilter.vue` | 分类过滤器 |
| `CategoryManager.vue` | 分类管理 |
| `AppEditDialog.vue` | 应用编辑对话框 |
| `AppQuickLaunchModal.vue` | 快速启动弹窗 |
| `WorkflowEditDialog.vue` | 工作流编辑 |
| `LauncherSettingsDialog.vue` | 启动器设置 |

#### 截图相关组件 (components/screenshot/)
| 文件 | 功能描述 |
|------|---------|
| `ActivityTimelineItem.vue` | 活动时间线项 - 支持懒加载和并发控制 |
| `DateNavigation.vue` | 日期导航组件 |
| `ScreenshotDetailModal.vue` | 截图详情弹窗 - 支持上下翻页 |
| `ScreenshotGrid.vue` | 截图网格展示 |

#### 统计图表组件 (components/statistics/)
| 文件 | 功能描述 |
|------|---------|
| `HeatmapChart.vue` | 热力图 |
| `LineChart.vue` | 折线图 |
| `BarChart.vue` | 柱状图 |
| `PieChart.vue` | 饼图 |

#### 上下文组件 (components/context/)
| 文件 | 功能描述 |
|------|---------|
| `ContextManager.vue` | 上下文管理器 - Tab 容器 |
| `ContextTimeline.vue` | 上下文时间线 - 分组折叠、渐进加载 |
| `ContextBrowser.vue` | 上下文浏览器 |
| `ContextSettings.vue` | 采集设置 |
| `VlmSettings.vue` | VLM 配置 |

#### 通知组件 (components/notification/)
| 文件 | 功能描述 |
|------|---------|
| `NotificationBell.vue` | 通知铃铛 - 显示未读数 |
| `NotificationCard.vue` | 通知卡片 |
| `NotificationSettings.vue` | 通知设置 |

#### 其他组件
| 文件 | 功能描述 |
|------|---------|
| `ContextViewer.vue` | 上下文查看器 |
| `SqlItem.vue` | SQL 项展示 |
| `sql/CyberpunkSqlModal.vue` | 赛博朋克风格 SQL 弹窗 |

### 3.3 状态管理 (src/stores/)

| 文件 | Store 名称 | 功能描述 |
|------|-----------|---------|
| `taskStore.ts` | `useTaskStore` | 任务状态 - CRUD、状态转换、过时检测 |
| `sqlStore.ts` | `useSqlStore` | SQL 记录状态 - 历史、收藏、分类、AI 分类 |
| `aiStore.ts` | `useAiStore` | AI 配置状态 - 配置管理、连接测试 |
| `appLauncherStore.ts` | `useAppLauncherStore` | 启动器状态 - 应用、分类、工作流、历史 |
| `settingsStore.ts` | `useSettingsStore` | 系统设置状态 |
| `workLogStore.ts` | `useWorkLogStore` | 工作日志状态 |
| `weeklyPlanStore.ts` | `useWeeklyPlanStore` | 周计划状态 |

### 3.4 API 层 (src/api/)

所有 API 通过 Tauri 的 `invoke` 函数调用后端 Rust 命令。

| 文件 | API 对象 | 功能描述 |
|------|---------|---------|
| `taskApi.ts` | `taskApi` | 任务 CRUD、状态转换、象限统计 |
| `sqlApi.ts` | `sqlApi` | SQL 保存、查询、分类管理 |
| `aiApi.ts` | `aiApi` | AI 配置、聊天、任务分类、应用分类 |
| `vlmApi.ts` | `vlmApi` | VLM 配置、图片分析 |
| `tagApi.ts` | `tagApi` | 标签 CRUD、任务-标签关联 |
| `notificationApi.ts` | `notificationApi` | 通知管理、日报周报生成 |
| `screenshotApi.ts` | `screenshotApi` | 截图列表、详情、活动分组 |
| `statisticsApi.ts` | `statisticsApi` | 热力图、趋势、应用使用统计 |
| `workLogApi.ts` | `workLogApi` | 工作日志 CRUD、上下文摘要 |
| `appLauncherApi.ts` | `appLauncherApi` | 应用扫描、启动、分类、工作流 |
| `contextApi.ts` | `contextApi` | 上下文采集、配置、摘要 |
| `clipboardApi.ts` | `clipboardApi` | 剪贴板历史管理 |
| `timelineApi.ts` | `timelineApi` | 时间线数据 |
| `logApi.ts` | `logApi` | 日志查询 |
| `windowApi.ts` | `windowApi` | 窗口管理 |
| `weeklyPlanApi.ts` | `weeklyPlanApi` | 周计划管理 |

### 3.5 类型定义 (src/types/)

| 文件 | 主要类型 |
|------|---------|
| `task.ts` | `Task`, `TaskQuadrant`, `Tag`, `WorkContext`, `FileContext` |
| `ai.ts` | `AiConfig`, `AiProvider`, `TaskClassifyResult`, `AppClassifyResult`, `AiLog` |
| `vlm.ts` | `VlmConfig`, `VlmProvider`, `VlmResponse` |
| `sql.ts` | `SqlRecord`, `SqlCategory`, `SqlClassifyResult` |
| `notification.ts` | `Notification`, `NotificationType`, `NotificationSettings` |
| `appLauncher.ts` | `AppItem`, `Category`, `Workflow`, `ItemType` |
| `context.ts` | `ContextSettings`, `ScreenContext`, `DayContextSummary` |
| `clipboard.ts` | `ClipboardHistory`, `ClipboardConfig`, `ClipboardContentType` |
| `timeline.ts` | `TimelineItem`, `TimelineGroup`, `TimelineFilter` |
| `workLog.ts` | `WorkLog`, `WeeklyPlan`, `WeekContextSummary` |

### 3.6 多窗口入口

| 窗口 | 入口文件 | App 组件 | 功能描述 |
|------|---------|---------|---------|
| 主窗口 | `src/main.ts` | `App.vue` | 主应用，包含所有页面视图和嵌入式模态框 |
| 启动器 | `src/launcher/main.ts` | `LauncherApp.vue` | 赛博朋克风格应用启动器，支持搜索、分类、快捷键 |
| 快速任务 | `src/quick-task/main.ts` | `QuickTaskApp.vue` | 极简/展开两种模式，支持标签检测、AI 分析 |
| SQL 面板 | `src/sql-panel/main.ts` | `SqlPanelApp.vue` | SQL 历史查询，双栏布局 |
| 剪贴板 | `src/clipboard-history/main.ts` | `ClipboardHistoryApp.vue` | 剪贴板历史，支持搜索、类型过滤、置顶 |
| 任务日历 | `src/task-calendar/main.ts` | `TaskCalendarApp.vue` | 月视图/周视图日历 |
| 任务浮窗 | `src/task-float/main.ts` | `TaskFloatApp.vue` | 浮窗式任务列表 |

### 3.7 路由配置 (src/router/index.ts)

| 路由路径 | 页面组件 | 说明 |
|---------|---------|------|
| `/task-board` | TaskBoard | 任务看板 |
| `/sql-history` | SqlHistory | SQL 历史 |
| `/work-log` | WorkLog | 工作日志 |
| `/app-launcher` | AppLauncher | 应用启动器 |
| `/screenshot-gallery` | ScreenshotGallery | 截图回顾 |
| `/data-statistics` | DataStatistics | 数据统计 |
| `/notification-center` | NotificationCenter | 通知中心 |
| `/settings` | Settings | 设置 |

---

## 四、后端架构

### 4.1 命令层 (src-tauri/src/commands/)

前端通过 `invoke` 调用的所有 Tauri 命令。

| 文件 | 功能描述 | 主要命令 |
|------|---------|---------|
| `ai_commands.rs` | AI 相关 | `save_ai_config`, `get_ai_config`, `test_ai_connection`, `ai_chat`, `ai_classify_task`, `ai_classify_apps` |
| `vlm_commands.rs` | VLM 相关 | `vlm_save_config`, `vlm_get_config`, `vlm_test_connection`, `vlm_analyze_image` |
| `task_commands.rs` | 任务管理 | `get_all_tasks`, `create_task`, `start_task`, `pause_task`, `complete_task`, `delete_task`, `defer_task` |
| `sql_commands.rs` | SQL 管理 | `save_sql`, `get_recent_sqls`, `get_favorite_sqls`, `toggle_favorite_sql`, 分类管理 |
| `sql_ai_commands.rs` | SQL AI 分类 | `ai_classify_sqls`, `manual_classify_sql` |
| `work_log_commands.rs` | 工作日志 | `save_work_log`, `get_work_log`, `work_log_generate_with_context` |
| `context_commands.rs` | 屏幕上下文 | `context_start_capture`, `context_stop_capture`, `context_get_config`, `context_list_by_date` |
| `screenshot_commands.rs` | 截图回顾 | `screenshot_list`, `screenshot_get_detail`, `screenshot_get_image`, `screenshot_get_activities` |
| `notification_commands.rs` | 通知中心 | `notification_list`, `notification_mark_read`, `notification_generate_daily_report` |
| `report_commands.rs` | 日报 | `report_generate`, `report_get`, `report_list`, `report_delete` |
| `tips_commands.rs` | 智能提示 | `tips_generate`, `tips_list`, `tips_analyze_pattern` |
| `statistics_commands.rs` | 数据统计 | `statistics_get_heatmap`, `statistics_get_daily_trend`, `statistics_get_app_usage` |
| `app_launcher_commands.rs` | 应用启动器 | `scan_installed_apps`, `launch_app`, `get_all_apps`, 分类/工作流管理 |
| `clipboard_commands.rs` | 剪贴板 | `get_clipboard_history`, `copy_from_clipboard_history`, `toggle_clipboard_pin` |
| `tag_commands.rs` | 标签管理 | `create_tag`, `update_tag`, `delete_tag`, `add_tag_to_task` |
| `timeline_commands.rs` | 时间线 | `get_timeline`, `merge_timeline_items`, `get_activities_by_date` |
| `settings_commands.rs` | 应用设置 | `get_app_settings`, `save_app_settings`, `export_app_settings` |
| `system_commands.rs` | 系统 | `get_system_info`, `get_database_path`, `export_database`, `get_crash_logs` |
| `window_commands.rs` | 窗口管理 | 窗口显示、隐藏、位置 |
| `shortcut_commands.rs` | 快捷键 | 全局快捷键配置 |
| `prompt_commands.rs` | 提示词 | 提示词配置管理 |
| `weekly_plan_commands.rs` | 周计划 | 周计划 CRUD |
| `autostart_commands.rs` | 自启动 | 开机自启动配置 |
| `db_repair_commands.rs` | 数据库修复 | 数据库修复工具 |

### 4.2 服务层 (src-tauri/src/services/)

| 文件 | 功能描述 |
|------|---------|
| `ai_service.rs` | AI 服务 - 与 DeepSeek/Qwen 等提供商通信，执行各类 AI 任务 |
| `vlm_service.rs` | VLM 服务 - 与视觉语言模型通信，分析图片 |
| `task_service.rs` | 任务服务 - 任务 CRUD、状态转换核心逻辑 |
| `sql_service.rs` | SQL 服务 - SQL 历史记录管理 |
| `sql_ai_service.rs` | SQL AI 服务 - AI 自动分类 SQL |
| `work_log_service.rs` | 工作日志服务 - 日志 CRUD、上下文摘要 |
| `report_service.rs` | 日报服务 - 日报生成（AI 驱动） |
| `context_manager_service.rs` | 上下文管理服务 - 协调截图采集、变化检测、定时任务 |
| `context_store_service.rs` | 上下文存储服务 - 数据库存储和查询 |
| `screen_capture_service.rs` | 屏幕截图服务 - 实际截图采集 |
| `screenshot_batch_processor_service.rs` | 批量截图处理服务 - VLM 批量分析 |
| `activity_summary_service.rs` | 活动总结服务 - 生成活动摘要 |
| `notification_service.rs` | 通知服务 - 通知 CRUD 和管理 |
| `tips_service.rs` | 智能提示服务 - 分析活动模式，生成提示 |
| `app_launcher_service.rs` | 应用启动器服务 - 应用管理、启动、工作流 |
| `app_scanner_service.rs` | 应用扫描服务 - 系统应用扫描 |
| `clipboard_history_service.rs` | 剪贴板历史服务 - 监控和管理剪贴板 |
| `tag_service.rs` | 标签服务 - 标签管理 |
| `settings_service.rs` | 设置服务 - 全局设置存储 |
| `weekly_plan_service.rs` | 周计划服务 |
| `scheduler_service.rs` | 定时任务调度器 - 活动总结、智能提示定时执行 |
| `prompt_manager_service.rs` | 提示词管理服务 - 加载 YAML 提示词配置（单例） |
| `prompt_service.rs` | 提示词服务 |
| `git_service.rs` | Git 服务（保留） |

### 4.3 数据模型 (src-tauri/src/models/)

| 文件 | 主要结构体 |
|------|---------|
| `task.rs` | `Task`, `TaskStatus`, `TaskCategory`, `TaskPriority`, `TaskQuadrant`, `Tag`, `WorkContext` |
| `work_log.rs` | `WorkLog` |
| `screen_context.rs` | `ScreenContext`, `DayStats`, `DailySummary`, `DayContextSummary`, `WeekContextSummary` |
| `clipboard.rs` | `ClipboardHistory`, `ClipboardConfig`, `ClipboardContentType` |
| `app_launcher.rs` | `AppItem`, `Category`, `Workflow`, `LaunchHistory`, `ItemType` |
| `weekly_plan.rs` | `WeeklyPlan`, `WeeklyPlanStatus` |
| `sql_record.rs` | `SqlRecord` |
| `app_launcher_settings.rs` | `AppLauncherSettings` |

### 4.4 数据库 (src-tauri/src/db/)

| 文件 | 功能描述 |
|------|---------|
| `connection.rs` | 数据库连接 - SQLite 初始化、线程安全连接管理 |
| `migrations.rs` | 数据库迁移 - 创建所有表和索引 |

**主要数据表：**
- `tasks` - 任务表
- `tags` / `task_tags` - 标签和关联表
- `sql_history` / `sql_categories` / `sql_category_mappings` - SQL 相关表
- `work_logs` - 工作日志
- `screen_contexts` - 屏幕上下文
- `clipboard_history` - 剪贴板历史
- `apps` / `categories` / `workflows` / `launch_history` - 应用启动器相关表
- `notifications` - 通知
- `daily_reports` - 日报
- `activity_summaries` - 活动总结
- `tips` - 智能提示
- `ai_config` / `vlm_config` - AI/VLM 配置
- `notification_settings` / `app_launcher_settings` - 各类设置
- `weekly_plans` - 周计划

### 4.5 提示词 (src-tauri/src/prompts/)

| 文件 | 功能描述 |
|------|---------|
| `activity_prompts.rs` | 活动分析提示词模板 |
| `report_prompts.rs` | 日报生成提示词模板 |
| `tips_prompts.rs` | 智能提示提示词模板 |

### 4.6 配置文件 (src-tauri/config/)

| 文件 | 功能描述 |
|------|---------|
| `prompts_zh.yaml` | 中文提示词配置 - 截图分析、日报生成、智能提醒等提示词模板 |

### 4.7 工具函数 (src-tauri/src/utils/)

| 文件 | 功能描述 |
|------|---------|
| `crash_logger.rs` | 崩溃日志 - panic 处理、日志记录、旧日志清理 |

---

## 五、核心功能模块

### 5.1 任务管理

**功能特点：**
- 三列看板（待办/进行中/已完成）
- 四象限优先级矩阵（紧急/重要维度）
- 多标签支持
- AI 智能分类
- AI 生成子任务
- 任务延期和过时检测
- 任务导入

**数据流：**
```
前端 TaskBoard.vue
  ↓ taskApi.createTask()
  ↓ invoke('create_task')
后端 task_commands.rs
  ↓ TaskService::create_task()
数据库 tasks 表
```

### 5.2 屏幕上下文采集

**功能特点：**
- 定时自动截图
- 变化检测（哈希比对）
- VLM 图片分析（可选）
- 活动分组和时间线展示
- 日报/周报生成

**数据流：**
```
ContextManager.start()
  ↓ 定时触发
ScreenCaptureService.capture_screen()
  ↓ 检测变化
ContextStoreService.save()
  ↓ 可选 VLM 分析
ScreenshotBatchProcessor.process()
  ↓
数据库 screen_contexts 表
```

### 5.3 AI 集成

**支持的 AI 提供商：**
- DeepSeek
- 阿里通义千问 (Qwen)
- 自定义 API

**AI 功能：**
- 任务分类和描述增强
- 子任务生成
- 工作日志生成和润色
- SQL 自动分类
- 应用分类
- 日报生成

### 5.4 VLM 视觉语言模型

**支持的 VLM 提供商：**
- Qwen-VL (阿里通义千问视觉版)
- DeepSeek-VL
- OpenAI (GPT-4V)
- Claude
- 豆包
- Kimi
- 自定义

**VLM 功能：**
- 截图内容分析
- 活动类型识别
- 关键信息提取

### 5.5 应用启动器

**功能特点：**
- 系统应用扫描
- 赛博朋克风格 UI
- 拼音/首字母搜索
- 分类管理（8个默认分类）
- 工作流（多应用组合启动）
- AI 智能分类
- 使用频率排序

### 5.6 数据统计

**统计维度：**
- 热力图（按日期/小时）
- 日趋势图
- 小时分布图
- 应用使用统计
- 活动类型分布

---

## 六、全局快捷键

| 快捷键 | 功能 |
|--------|------|
| 可配置 | 打开任务看板 |
| 可配置 | 打开 SQL 历史 |
| 可配置 | 打开应用启动器 |
| 可配置 | 打开快速任务 |
| 可配置 | 打开剪贴板历史 |

---

## 七、安全性设计

1. **API Key 保护**：AI/VLM API Key 返回时部分掩蔽（只显示前后 4 位）
2. **线程安全**：使用 `Arc<Mutex>` 保证数据库连接线程安全
3. **错误处理**：禁用 Windows 系统错误对话框，自定义 panic 处理
4. **日志管理**：崩溃日志记录，支持日志级别配置

---

## 八、文件统计

| 类别 | 文件数量 |
|------|---------|
| 前端 Views | 8 |
| 前端 Components | 40+ |
| 前端 Stores | 7 |
| 前端 API | 16 |
| 前端 Types | 11 |
| 后端 Commands | 22 |
| 后端 Services | 22 |
| 后端 Models | 8 |
| **总计** | **130+** |

---

## 九、开发说明

### 开发环境要求
- Node.js 18+
- Rust 1.70+
- Tauri CLI

### 构建命令
```bash
# 开发模式
npm run tauri dev

# 生产构建
npm run tauri build
```

### 项目约定
- 不随意生成 md 文档
- 大型调整使用工作流 MCP 进行任务分组
- 不需要执行 `npm run dev` 进行测试
- 不进行项目编译测试，由用户自行验证
