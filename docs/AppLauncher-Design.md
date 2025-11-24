# 软件快速启动功能 - 开发设计文档

## 📋 一、功能概述

**AppLauncher（应用启动器）** 是一个智能化的软件快速启动工具，帮助用户通过快捷键或搜索快速定位并启动常用软件，解决 Windows 桌面/开始菜单查找效率低下的问题。

---

## 🎯 二、核心功能特性

### 2.1 应用管理
- ✅ **自动扫描系统已安装应用**（从开始菜单、注册表、常见安装路径）
- ✅ **手动添加自定义应用**（支持exe文件、快捷方式、文件夹、网址）
- ✅ **应用分类管理**（开发工具、办公软件、浏览器、设计工具、其他等）
- ✅ **自定义分类标签**（用户可创建个性化分类，如"前端开发"、"游戏"等）
- ✅ **应用图标自动提取**（从exe文件提取图标显示）
- ✅ **启动次数统计**（记录每个应用的使用频率）

### 2.2 快速搜索与启动
- ✅ **全局快捷键唤起**（如 `Ctrl+Shift+Space`）
- ✅ **模糊搜索**（支持拼音首字母、中英文混合搜索）
- ✅ **智能排序**（根据启动频率、最近使用时间、搜索匹配度排序）
- ✅ **键盘快速操作**（`↑↓` 选择，`Enter` 启动，`Esc` 关闭）
- ✅ **多应用批量启动**（可选中多个应用一键启动）

### 2.3 个性化配置
- ✅ **自定义全局快捷键**
- ✅ **自定义应用启动参数**（如启动命令行参数）
- ✅ **置顶常用应用**（Pin功能）
- ✅ **隐藏不需要的应用**
- ✅ **导入/导出配置**（方便多设备同步）

### 2.4 增强功能
- ✅ **应用分组工作流**（一键启动一组应用，如"前端开发"组启动VSCode、Chrome、Terminal）
- ✅ **快速打开文件夹**（支持项目目录收藏）
- ✅ **网址快捷方式**（如快速打开GitHub、Claude等）
- ✅ **最近启动历史**（查看最近打开的应用）

---

## 🏗️ 三、技术架构设计

### 3.1 前端（Vue3 + TypeScript）

#### 路由结构
```
/app-launcher          # 应用启动器主页面
  ├─ 搜索框 + 快捷键提示
  ├─ 应用列表（网格/列表视图）
  ├─ 分类标签栏
  └─ 应用详情/编辑对话框

/app-launcher/settings  # 启动器设置页
  ├─ 应用扫描与管理
  ├─ 分类管理
  ├─ 快捷键配置
  └─ 工作流管理
```

#### 组件设计
```
components/appLauncher/
├─ AppCard.vue               # 应用卡片组件
├─ AppSearchBar.vue          # 搜索栏组件
├─ AppCategoryFilter.vue     # 分类筛选组件
├─ AppEditDialog.vue         # 应用编辑对话框
├─ WorkflowEditDialog.vue    # 工作流编辑对话框
├─ AppIconDisplay.vue        # 应用图标显示组件
└─ AppQuickLaunchModal.vue   # 全局快速启动弹窗
```

#### Store状态管理（Pinia）
```typescript
// stores/appLauncherStore.ts
interface AppItem {
  id: string
  name: string
  path: string
  icon?: string          // base64图标或图标路径
  category: string
  tags: string[]
  launchCount: number
  lastLaunchedAt?: number
  isPinned: boolean
  isHidden: boolean
  launchArgs?: string
}

interface Category {
  id: string
  name: string
  color: string
  icon?: string
}

interface Workflow {
  id: string
  name: string
  appIds: string[]
  launchDelay?: number  // 应用启动间隔（ms）
}

interface AppLauncherState {
  apps: AppItem[]
  categories: Category[]
  workflows: Workflow[]
  searchKeyword: string
  selectedCategory: string
  viewMode: 'grid' | 'list'
  globalShortcut: string
}
```

### 3.2 后端（Rust + Tauri）

#### 数据库表设计（SQLite）
```sql
-- 应用表
CREATE TABLE apps (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    icon TEXT,
    category TEXT,
    tags TEXT,              -- JSON数组
    launch_count INTEGER DEFAULT 0,
    last_launched_at INTEGER,
    is_pinned INTEGER DEFAULT 0,
    is_hidden INTEGER DEFAULT 0,
    launch_args TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 分类表
CREATE TABLE categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT,
    icon TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL
);

-- 工作流表
CREATE TABLE workflows (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    app_ids TEXT NOT NULL,  -- JSON数组
    launch_delay INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 启动历史表
CREATE TABLE launch_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id TEXT NOT NULL,
    launched_at INTEGER NOT NULL,
    FOREIGN KEY (app_id) REFERENCES apps(id)
);
```

#### Rust命令设计
```rust
// commands/app_launcher_commands.rs
#[tauri::command]
async fn scan_installed_apps() -> Result<Vec<AppItem>, String>

#[tauri::command]
async fn get_all_apps() -> Result<Vec<AppItem>, String>

#[tauri::command]
async fn add_app(app: AppItem) -> Result<(), String>

#[tauri::command]
async fn update_app(id: String, app: AppItem) -> Result<(), String>

#[tauri::command]
async fn delete_app(id: String) -> Result<(), String>

#[tauri::command]
async fn launch_app(id: String) -> Result<(), String>

#[tauri::command]
async fn launch_workflow(workflow_id: String) -> Result<(), String>

#[tauri::command]
async fn extract_icon(exe_path: String) -> Result<String, String>

#[tauri::command]
async fn get_categories() -> Result<Vec<Category>, String>

#[tauri::command]
async fn save_category(category: Category) -> Result<(), String>

#[tauri::command]
async fn get_workflows() -> Result<Vec<Workflow>, String>

#[tauri::command]
async fn save_workflow(workflow: Workflow) -> Result<(), String>

#[tauri::command]
async fn get_launch_history(limit: usize) -> Result<Vec<HistoryItem>, String>

#[tauri::command]
async fn search_apps(keyword: String, category: Option<String>) -> Result<Vec<AppItem>, String>
```

#### 系统扫描逻辑
```rust
// services/app_scanner_service.rs
// 扫描以下位置查找已安装应用：
// 1. HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
// 2. HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
// 3. C:\ProgramData\Microsoft\Windows\Start Menu\Programs
// 4. C:\Users\{user}\AppData\Roaming\Microsoft\Windows\Start Menu\Programs
// 5. C:\Program Files
// 6. C:\Program Files (x86)
```

#### 图标提取（使用Windows API）
```rust
// utils/icon_extractor.rs
// 使用 winapi 或 windows-rs crate 提取exe图标
// 转换为 base64 或保存到本地缓存目录
```

---

## 🎨 四、UI/UX 设计

### 4.1 主界面布局
```
┌─────────────────────────────────────────┐
│  🔍 搜索应用或网址...          [设置]   │
├─────────────────────────────────────────┤
│  [全部] [开发] [办公] [设计] [其他] ... │
├─────────────────────────────────────────┤
│  📌 置顶应用                             │
│  ┌────┐ ┌────┐ ┌────┐                   │
│  │VSC │ │Chr │ │Term│                   │
│  └────┘ └────┘ └────┘                   │
│                                          │
│  🔥 常用应用（按使用频率排序）           │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐           │
│  │Git │ │Post│ │Fig │ │Slck│           │
│  └────┘ └────┘ └────┘ └────┘           │
│                                          │
│  📂 所有应用                             │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐           │
│  │App1│ │App2│ │App3│ │App4│           │
│  └────┘ └────┘ └────┘ └────┘           │
└─────────────────────────────────────────┘
```

### 4.2 快速启动弹窗（全局快捷键唤起）
```
┌─────────────────────────────────────┐
│  🔍 输入搜索...                     │
├─────────────────────────────────────┤
│  > VSCode                          5│  ← 选中高亮
│    Visual Studio Code                │
│    D:\Programs\VSCode\Code.exe       │
│                                      │
│    Chrome                          8│
│    Google Chrome                     │
│    C:\Program Files\Google\...       │
│                                      │
│    Terminal                        3│
│    Windows Terminal                  │
│    C:\Program Files\WindowsApps\...  │
└─────────────────────────────────────┘
    ↑↓ 选择  Enter 启动  Esc 关闭
```

### 4.3 颜色主题（继承现有indigo主题）
- 主色调：indigo-500/600（#6366f1）
- 背景：slate-950/900
- 卡片背景：半透明深色
- 悬停效果：indigo发光效果

---

## 📅 五、开发计划（分阶段）

### 阶段1：核心功能（MVP）
1. ✅ 数据库表设计与迁移
2. ✅ 应用扫描功能（扫描开始菜单）
3. ✅ 应用列表展示（网格视图）
4. ✅ 手动添加应用
5. ✅ 应用启动功能
6. ✅ 基本搜索功能

### 阶段2：增强功能
1. ✅ 图标提取与显示
2. ✅ 分类管理
3. ✅ 智能搜索与排序（使用频率）
4. ✅ 置顶/隐藏应用
5. ✅ 全局快捷键唤起快速启动窗口

### 阶段3：高级功能
1. ✅ 工作流管理（批量启动）
2. ✅ 启动历史记录
3. ✅ 导入/导出配置
4. ✅ 应用启动参数配置
5. ✅ 拼音搜索支持

---

## 📦 六、依赖库

### Rust依赖（Cargo.toml）
```toml
[dependencies]
winreg = "0.52"          # Windows注册表读取
winapi = "0.3"           # Windows API
image = "0.24"           # 图标处理
base64 = "0.21"          # base64编码
serde_json = "1.0"       # JSON处理
```

### 前端依赖（已包含）
- Vue3
- Naive UI
- Pinia
- VueRouter

---

## 🔑 七、关键技术点

1. **系统应用扫描**：通过读取Windows注册表和开始菜单快捷方式
2. **图标提取**：使用Windows Shell API提取exe图标
3. **全局快捷键**：使用Tauri的global-shortcut插件
4. **模糊搜索**：前端实现拼音首字母匹配
5. **进程启动**：使用Rust的`std::process::Command`启动应用

---

## ⚙️ 八、用户配置示例

```json
{
  "globalShortcut": "Ctrl+Shift+Space",
  "defaultView": "grid",
  "scanPaths": [
    "C:\\Program Files",
    "C:\\Program Files (x86)",
    "D:\\Programs"
  ],
  "categories": [
    { "id": "dev", "name": "开发工具", "color": "#6366f1" },
    { "id": "office", "name": "办公软件", "color": "#10b981" },
    { "id": "design", "name": "设计工具", "color": "#f59e0b" }
  ],
  "workflows": [
    {
      "id": "frontend-dev",
      "name": "前端开发环境",
      "apps": ["vscode", "chrome", "terminal"],
      "launchDelay": 500
    }
  ]
}
```

---

## 📊 九、预期效果

- ✅ **快速启动**：`Ctrl+Shift+Space` 全局唤起，输入关键词即刻启动
- ✅ **智能推荐**：根据使用习惯自动排序
- ✅ **优雅界面**：与现有DevAssistant风格统一，深色主题+indigo主色
- ✅ **高度自定义**：支持分类、标签、工作流等个性化配置
- ✅ **性能优异**：Rust后端保证扫描和启动速度

---

## 📝 开发注意事项

1. **图标缓存**：提取的图标应缓存到本地避免重复提取
2. **扫描性能**：首次扫描可能较慢，需要后台异步进行并显示进度
3. **权限问题**：某些系统目录可能需要管理员权限
4. **跨平台考虑**：当前设计针对Windows，未来可扩展到macOS/Linux
5. **数据库迁移**：新增表需要在`src-tauri/src/db/migrations.rs`中添加迁移逻辑

---

**文档版本**: v1.0
**创建时间**: 2025-11-24
**最后更新**: 2025-11-24
