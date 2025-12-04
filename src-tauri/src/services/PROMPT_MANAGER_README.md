# PromptManager 使用指南

## 概述

PromptManager 是 DevAssistant 项目的 Prompt 管理服务，提供基于 YAML 配置的 Prompt 模板管理、缓存、变量替换和热重载功能。

## 特性

- ✅ **YAML配置**: 使用 YAML 格式存储 Prompt，便于编辑和维护
- ✅ **全局单例**: 使用 lazy_static 和 RwLock 实现线程安全的全局缓存
- ✅ **变量替换**: 支持 `{{variable}}` 格式的模板变量替换
- ✅ **热重载**: 支持运行时重新加载配置文件
- ✅ **类型安全**: 强类型的 Prompt 配置结构
- ✅ **多场景支持**: 涵盖截图分析、日报生成、Tips提示等多个场景

## 文件结构

```
src-tauri/
├── src/services/
│   ├── prompt_manager_service.rs       # 核心服务实现
│   ├── prompt_manager_example.rs       # 使用示例（不参与编译）
│   └── mod.rs                          # 模块导出
├── config/
│   └── prompts_zh.yaml                 # Prompt配置文件
└── Cargo.toml                          # 依赖配置
```

## 配置文件格式

配置文件位于 `config/prompts_zh.yaml`，结构如下：

```yaml
# Processing模块
processing:
  extraction:
    screenshot_contextual_batch:
      system: |
        系统提示词内容...
      user: |
        用户提示词内容，支持变量 {{variable_name}}

# Generation模块
generation:
  merge_hourly_reports:
    system: "..."
    user: "..."
  generation_report:
    system: "..."
    user: "..."
  # ... 其他配置

# Merging模块
merging:
  merge_batch_items:
    system: "..."
    user: "..."

# Entity Processing模块
entity_processing:
  entity_extraction:
    system: "..."
    user: "..."
```

## 基本使用

### 1. 获取全局单例

```rust
use crate::services::PromptManager;

// 获取全局单例（首次调用会自动加载配置）
let manager = PromptManager::get_instance()?;
```

### 2. 获取 Prompt

```rust
// 获取截图分析 Prompt
let screenshot_prompt = manager.get_screenshot_prompt();
println!("System: {}", screenshot_prompt.system);
println!("User: {}", screenshot_prompt.user);

// 获取日报生成 Prompt
let report_prompt = manager.get_report_prompt();

// 获取 Tips 生成 Prompt
let tips_prompt = manager.get_tips_prompt();
```

### 3. 变量替换

```rust
use std::collections::HashMap;

// 准备变量
let mut vars = HashMap::new();
vars.insert("current_time".to_string(), "2024-12-04 10:00:00".to_string());
vars.insert("screenshot_count".to_string(), "3".to_string());

// 方式1: 渲染指定 Prompt
let rendered = manager.render_screenshot_prompt(&vars);

// 方式2: 使用静态方法渲染任意模板
let template = "当前时间: {{current_time}}";
let result = PromptManager::render_prompt(template, &vars);
```

### 4. 热重载配置

```rust
// 重新加载全局配置
PromptManager::reload_global()?;

// 或者重新加载实例配置
let mut manager = PromptManager::get_instance()?;
manager.reload()?;
```

## 可用的 Prompt 方法

### Processing 模块
- `get_screenshot_prompt()` - 截图上下文批量提取

### Generation 模块
- `get_hourly_merge_prompt()` - 小时报告合并
- `get_report_prompt()` - 日报生成
- `get_tips_prompt()` - 智能提示生成
- `get_todo_prompt()` - TODO提取
- `get_activity_monitor_prompt()` - 实时活动监控

### Merging 模块
- `get_batch_merge_prompt()` - 批量合并items
- `get_weekly_merge_prompt()` - 周报合并

### Entity Processing 模块
- `get_entity_extraction_prompt()` - 实体提取
- `get_relationship_prompt()` - 关系识别

## 完整示例

### 在 AI 服务中使用

```rust
use crate::services::{PromptManager, AiService};
use std::collections::HashMap;

pub async fn analyze_screenshot(
    screenshot_data: Vec<u8>,
    ai_service: &AiService
) -> anyhow::Result<String> {
    // 获取 PromptManager
    let manager = PromptManager::get_instance()?;

    // 准备变量
    let mut vars = HashMap::new();
    vars.insert(
        "current_time".to_string(),
        chrono::Local::now().to_rfc3339()
    );
    vars.insert("screenshot_count".to_string(), "1".to_string());

    // 渲染 Prompt
    let prompt = manager.render_screenshot_prompt(&vars);

    // 构造 AI 请求
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: prompt.system,
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!("{}\n\n[截图数据]", prompt.user),
        },
    ];

    // 调用 AI
    let response = ai_service.chat(messages).await?;

    Ok(response)
}
```

### 生成日报

```rust
pub async fn generate_daily_report(
    date: &str,
    activities: Vec<Activity>,
    ai_service: &AiService
) -> anyhow::Result<String> {
    let manager = PromptManager::get_instance()?;

    // 序列化活动数据
    let activities_json = serde_json::to_string_pretty(&activities)?;

    // 准备变量
    let mut vars = HashMap::new();
    vars.insert("date".to_string(), date.to_string());
    vars.insert("activities_json".to_string(), activities_json);

    // 渲染并调用 AI
    let prompt = manager.render_report_prompt(&vars);

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: prompt.system,
        },
        ChatMessage {
            role: "user".to_string(),
            content: prompt.user,
        },
    ];

    ai_service.chat(messages).await
}
```

## 初始化配置

在 `main.rs` 中已经添加了自动初始化：

```rust
// 初始化 PromptManager
match PromptManager::get_instance() {
    Ok(_) => {
        log_runtime("PromptManager 初始化成功");
    }
    Err(e) => {
        log_runtime(&format!("PromptManager 初始化失败: {}, 将在首次使用时加载", e));
    }
}
```

如果需要使用自定义路径：

```rust
// 在应用启动时
PromptManager::initialize("custom_config/prompts.yaml")?;
```

## 变量命名规范

在配置文件中使用变量时，建议遵循以下规范：

- 使用小写字母和下划线: `{{current_time}}`, `{{screenshot_count}}`
- 语义清晰: `{{date}}`, `{{activities_json}}`, `{{items_json}}`
- 避免使用特殊字符

## 错误处理

PromptManager 使用 `anyhow::Result` 进行错误处理：

```rust
match PromptManager::get_instance() {
    Ok(manager) => {
        // 使用 manager
    }
    Err(e) => {
        log::error!("PromptManager 初始化失败: {}", e);
        // 降级处理或使用默认 Prompt
    }
}
```

## 性能优化

- **单例模式**: 全局只加载一次配置文件
- **RwLock**: 支持多读单写，提高并发性能
- **按需解析**: 只在首次使用时加载配置
- **变量替换**: 使用简单的字符串替换，性能开销小

## 与旧版 PromptService 的对比

| 特性 | PromptService | PromptManager |
|------|---------------|---------------|
| 配置格式 | JSON | YAML |
| 存储位置 | 用户数据目录 | 项目配置目录 |
| 缓存机制 | 每次从文件读取 | 全局单例缓存 |
| 变量替换 | 不支持 | 支持 {{var}} |
| 热重载 | 不支持 | 支持 |
| 结构化配置 | 平铺 | 模块化分组 |

两者可以共存，PromptManager 适合系统内置的固定 Prompt，PromptService 适合用户自定义的 Prompt。

## 测试

运行单元测试：

```bash
cd src-tauri
cargo test prompt_manager
```

测试用例包括：
- `test_render_prompt`: 测试变量替换
- `test_render_prompt_missing_var`: 测试缺失变量处理
- `test_render_config`: 测试配置渲染

## 故障排查

### 配置文件找不到

确保 `config/prompts_zh.yaml` 文件存在于正确的位置：
- 开发环境: `DevAssistant/config/prompts_zh.yaml`
- 相对于 src-tauri 目录: `../config/prompts_zh.yaml`

### YAML 解析错误

检查 YAML 语法：
- 确保缩进使用空格（不是 Tab）
- 多行字符串使用 `|` 或 `>`
- 变量格式使用 `{{variable_name}}`

### 变量替换不生效

检查变量名是否匹配：
- 配置文件: `{{current_time}}`
- 代码中: `vars.insert("current_time", ...)`

## 扩展开发

### 添加新的 Prompt

1. 在 `config/prompts_zh.yaml` 中添加配置
2. 在 `prompt_manager_service.rs` 中添加对应的结构体字段
3. 添加 getter 方法和 render 方法
4. 在 `mod.rs` 中导出新增的类型

### 添加新的模块

如果需要添加新的 Prompt 模块（如 Analysis、Optimization 等）：

1. 在 YAML 中添加顶层模块
2. 创建对应的 Rust 结构体
3. 在 `PromptsConfig` 中添加字段
4. 实现访问方法

## 依赖

- `serde_yaml = "0.9"` - YAML 序列化/反序列化
- `once_cell = "1.19"` - 懒加载全局单例
- `anyhow = "1.0"` - 错误处理

## 许可证

同 DevAssistant 项目
