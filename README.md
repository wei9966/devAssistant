# DevAssistant

> 个人开发效率工具 | Personal Developer Productivity Tool

![Version](https://img.shields.io/badge/version-1.1.2-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

**DevAssistant** 是一款专为开发者打造的桌面效率工具，集成任务管理、番茄钟、SQL历史记录、工作日志、应用启动器、屏幕上下文分析等多种实用功能，帮助开发者提升工作效率。

![主界面](pic/introduce/微信图片_20251222133607_258_1.png)

---

## 目录

- [特性亮点](#特性亮点)
- [技术架构](#技术架构)
- [功能展示](#功能展示)
- [安装使用](#安装使用)
- [开发指南](#开发指南)
- [项目结构](#项目结构)
- [快捷键](#快捷键)
- [常见问题](#常见问题)

---

## 特性亮点

### 智能任务管理
- **四象限任务看板**：按重要/紧急维度分类，可拖拽调整
- **AI 智能分类**：自动分析任务内容，推荐所属象限
- **AI 预测任务**：基于屏幕上下文智能预测待办事项
- **多视图支持**：看板视图、日历视图、悬浮窗模式

### 专注时钟
- 基于番茄工作法的专注计时
- 关联任务记录专注时长
- 休息提醒与统计报表

### SQL 历史管理
- 自动捕获剪贴板中的 SQL 语句
- 语法高亮与智能分类
- 收藏与快速搜索

### 工作日志
- 日报、周报、周计划一站式管理
- AI 辅助生成工作内容
- 自动关联已完成任务

### 应用启动器
- 系统应用扫描与分类管理
- 工作流组合启动
- 全局快捷键快速搜索

### 屏幕上下文
- 定时自动截图记录
- VLM 视觉模型智能分析
- 活动时间线与回顾

### AI 集成
- 支持 DeepSeek、通义千问、OpenAI 等多种模型
- VLM 视觉语言模型（Qwen-VL、GPT-4V、Claude 等）
- 可自定义 API 端点

---

## 技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                        DevAssistant                          │
├─────────────────────────────────────────────────────────────┤
│  Frontend (Vue 3 + TypeScript)                               │
│  ┌─────────────┬─────────────┬─────────────┬──────────────┐ │
│  │   Views     │  Components │   Stores    │    API       │ │
│  │  8 Pages    │   40+ UI    │  7 Pinia    │  16 Modules  │ │
│  └─────────────┴─────────────┴─────────────┴──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  Tauri Bridge (IPC)                                          │
├─────────────────────────────────────────────────────────────┤
│  Backend (Rust)                                              │
│  ┌─────────────┬─────────────┬─────────────┬──────────────┐ │
│  │  Commands   │  Services   │   Models    │  Database    │ │
│  │  22 Modules │  22 Services│   8 Models  │   SQLite     │ │
│  └─────────────┴─────────────┴─────────────┴──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | Vue 3 + TypeScript |
| 状态管理 | Pinia |
| UI 组件库 | Naive UI |
| 路由 | Vue Router |
| 图表库 | ECharts |
| 桌面框架 | Tauri 2.0 |
| 后端语言 | Rust |
| 数据库 | SQLite |
| AI 集成 | OpenAI 兼容 API |
| VLM 集成 | 多平台视觉模型 |

---

## 功能展示

### 任务管理

**任务看板** - 四象限分类，拖拽管理

![任务看板](pic/introduce/微信图片_20251222133607_258_1.png)

**新建任务** - AI 智能分类推荐

![新建任务](pic/introduce/微信图片_20251222133616_259_1.png)

**任务悬浮窗** - 精简视图，随时查看

![任务悬浮窗](pic/introduce/微信图片_20251222133622_260_1.png)

**任务日历** - 月视图/周视图切换

![任务日历](pic/introduce/微信图片_20251222133736_261_1.png)

**AI 预测任务** - 智能识别待办事项

![AI预测任务](pic/introduce/微信图片_20251222133738_263_1.png)

### 专注时钟

**番茄时钟** - 保持专注，提升效率

![专注时钟](pic/introduce/微信图片_20251222133740_266_1.png)

### SQL 管理

**SQL 历史** - 自动记录，语法高亮

![SQL历史](pic/introduce/微信图片_20251222133743_267_1.png)

**SQL 搜索** - 关键词快速定位

![SQL搜索](pic/introduce/微信图片_20251222133753_269_1.png)

### 工作日志

**日报** - 记录每日工作内容

![日报](pic/introduce/微信图片_20251222133815_270_1.png)

**周报** - 汇总一周工作成果

![周报](pic/introduce/微信图片_20251222133817_271_1.png)

**周计划** - 规划下周工作安排

![周计划](pic/introduce/微信图片_20251222133818_272_1.png)

### 应用启动器

**应用管理** - 分类管理常用应用

![应用启动器](pic/introduce/微信图片_20251222133820_273_1.png)

**快捷搜索** - 全局快捷键快速启动

![快捷搜索](pic/introduce/微信图片_20251222133831_275_1.png)

### 报表中心

**数据统计** - 任务完成情况分析

![报表中心](pic/introduce/微信图片_20251222133836_276_1.png)

### 通知中心

**消息管理** - 任务提醒与系统通知

![通知中心](pic/introduce/微信图片_20251222133843_277_1.png)

### 工具箱

**实用工具** - 文本转换、端口检查等

![工具箱](pic/introduce/微信图片_20251222133847_278_1.png)

![文本转换器](pic/introduce/微信图片_20251222133902_280_1.png)

### 系统设置

**通用设置** - 主题、自启动等配置

![通用设置](pic/introduce/微信图片_20251222133906_281_1.png)

**AI 集成** - 配置 AI 服务

![AI集成](pic/introduce/微信图片_20251222133924_282_1.png)

**提示词管理** - 自定义 AI 提示词

![提示词管理](pic/introduce/微信图片_20251222133938_284_1.png)

**屏幕上下文** - 截图采集配置

![屏幕上下文](pic/introduce/微信图片_20251222133944_286_1.png)

---

## 安装使用

### 系统要求

- Windows 10/11 (64-bit)
- WebView2 Runtime (Windows 通常已预装)

### 下载安装

1. 前往 [Releases](https://github.com/wei9966/devAssistant/releases) 下载最新版本
2. 运行安装程序 `DevAssistant_x.x.x_x64-setup.exe`
3. 按提示完成安装

### 配置 AI 功能（可选）

1. 进入 **设置 > 集成**
2. 启用 AI 功能
3. 配置 API Key 和 Base URL（支持 OpenAI 兼容 API）
4. 测试连接

---

## 开发指南

### 环境要求

- Node.js 18+
- Rust 1.70+
- Tauri CLI 2.0

### 克隆项目

```bash
git clone https://github.com/wei9966/devAssistant.git
cd DevAssistant
```

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 生产构建

```bash
npm run tauri build
```

---

## 项目结构

```
DevAssistant/
├── src/                          # 前端源码
│   ├── api/                      # API 调用封装
│   ├── components/               # Vue 组件
│   │   ├── appLauncher/          # 应用启动器组件
│   │   ├── screenshot/           # 截图相关组件
│   │   ├── statistics/           # 统计图表组件
│   │   ├── context/              # 上下文组件
│   │   └── notification/         # 通知组件
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
│   │   ├── commands/             # Tauri 命令
│   │   ├── services/             # 业务逻辑服务
│   │   ├── models/               # 数据模型
│   │   ├── db/                   # 数据库管理
│   │   ├── prompts/              # AI 提示词
│   │   └── utils/                # 工具函数
│   └── config/                   # 配置文件
├── docs/                         # 文档
└── pic/                          # 截图资源
```

---

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl + K` | 全局搜索 |
| `Ctrl + N` | 新建任务 |
| `Ctrl + Space` | 唤起应用启动器 |
| `Esc` | 关闭弹窗 |

> 更多快捷键可在设置中自定义配置

---

## 常见问题

### Q: 如何配置 AI 功能？

进入 **设置 > 集成**，启用 AI 功能并配置 API Key 和 Base URL。支持任何 OpenAI 兼容的 API 服务（如 DeepSeek、通义千问等）。

### Q: 屏幕截图会占用很多存储空间吗？

可以在 **设置 > 屏幕上下文** 中配置数据保留天数，系统会自动清理过期数据。还可以调整截图间隔和相似度阈值来控制存储量。

### Q: 如何导出工作日志？

在工作日志页面，点击导出按钮可将日报/周报导出为常用格式。

### Q: 支持哪些 AI 模型？

**对话 AI**：DeepSeek、通义千问、OpenAI 及任何 OpenAI 兼容 API

**视觉 AI (VLM)**：Qwen-VL、DeepSeek-VL、GPT-4V、Claude、豆包、Kimi 等

---

## 许可证

[MIT License](LICENSE)

---

## 致谢

感谢所有为本项目做出贡献的开发者！

如有问题或建议，欢迎提交 [Issue](https://github.com/wei9966/devAssistant/issues) 或 [Pull Request](https://github.com/wei9966/devAssistant/pulls)。
