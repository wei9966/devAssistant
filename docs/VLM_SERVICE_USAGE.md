# VLM Service 更新说明

## 概述
VLM服务已更新，现在支持使用PromptService/PromptManager来管理和渲染Prompt模板。

## 新增功能

### 1. 新的数据结构

#### ScreenshotAnalysisResponse
用于解析VLM返回的JSON格式截图分析结果：

```rust
pub struct ScreenshotAnalysisResponse {
    pub title: String,           // 简洁的活动标题
    pub summary: String,         // 详细的活动描述
    pub keywords: Vec<String>,   // 关键词列表
    pub importance: i32,         // 重要性评分 (0-10)
    pub app_name: String,        // 应用程序名称
    pub activity_type: String,   // 活动类型 (coding|browsing|chatting等)
}
```

### 2. 新的方法

#### analyze_screenshot
使用PromptService的截图分析方法：

```rust
pub async fn analyze_screenshot(
    &self,
    image_base64: &str,
    history: Option<&str>,
) -> Result<ScreenshotAnalysisResponse>
```

**参数：**
- `image_base64`: Base64编码的截图数据
- `history`: 可选的历史记录文本

**返回：**
- `ScreenshotAnalysisResponse`: 结构化的分析结果

**使用示例：**

```rust
use crate::services::vlm_service::{VlmService, ScreenshotAnalysisResponse};

let vlm_service = VlmService::new(db_path);

// 不带历史记录
let result = vlm_service
    .analyze_screenshot(&image_base64, None)
    .await?;

// 带历史记录
let history = "用户之前在编写Rust代码";
let result = vlm_service
    .analyze_screenshot(&image_base64, Some(history))
    .await?;

println!("标题: {}", result.title);
println!("摘要: {}", result.summary);
println!("关键词: {:?}", result.keywords);
println!("重要性: {}", result.importance);
println!("应用: {}", result.app_name);
println!("活动类型: {}", result.activity_type);
```

### 3. Prompt模板管理

新方法会从PromptService加载`screenshot_analyze`模板，支持以下变量替换：

- `{{current_time}}`: 当前时间 (YYYY-MM-DD HH:MM:SS)
- `{{history}}`: 历史记录文本

**Prompt模板格式：**
```rust
PromptConfig {
    system: "系统提示词，使用{{变量名}}作为占位符",
    user: "用户提示词，使用{{变量名}}作为占位符"
}
```

### 4. 容错机制

当PromptService加载失败时，自动回退到默认Prompt：

```rust
async fn analyze_screenshot_fallback(
    &self,
    config: &VlmConfig,
    image_base64: &str,
    history: Option<&str>,
) -> Result<ScreenshotAnalysisResponse>
```

### 5. 多Provider支持

新方法支持以下Provider的system/user prompt分离：

- **Claude API**: 使用`analyze_with_claude_messages`
- **OpenAI Compatible API**: 使用`analyze_with_openai_compatible_messages`
  - Qwen VL
  - DeepSeek VL
  - OpenAI GPT-4V
  - Doubao
  - Kimi
  - 自定义兼容API

## 兼容性

### 保持向后兼容
原有的`analyze_image`方法仍然保留：

```rust
pub async fn analyze_image(
    &self,
    image_base64: &str,
    prompt: &str
) -> Result<String>
```

这确保现有代码不会受到影响。

## 迁移指南

### 从旧方法迁移到新方法

**旧方法：**
```rust
let prompt = format!("请分析这张截图...");
let response = vlm_service.analyze_image(&image_base64, &prompt).await?;
// 需要手动解析JSON字符串
```

**新方法：**
```rust
let response = vlm_service.analyze_screenshot(&image_base64, None).await?;
// 直接获得结构化数据
println!("标题: {}", response.title);
```

## 配置要求

确保PromptService配置中包含`screenshot_analyze`模板：

```json
{
  "screenshot_analyze": {
    "system": "系统提示词...",
    "user": "用户提示词，当前时间: {{current_time}}"
  }
}
```

## 示例集成

### 在Activity Summary Service中使用

可以更新`activity_summary_service.rs`以使用新方法：

```rust
// 替代原有的 analyze_image 调用
match vlm_service.analyze_screenshot(&image_base64, None).await {
    Ok(analysis) => {
        // 使用结构化的分析结果
        let summary_text = analysis.summary;
        let activity_type = analysis.activity_type;
        // ...
    }
    Err(e) => {
        log::error!("VLM分析失败: {}", e);
        // 回退逻辑
    }
}
```

## JSON响应格式

VLM API应返回以下JSON格式：

```json
{
  "title": "编写Rust VLM服务代码",
  "summary": "用户正在VS Code中编写Rust代码，实现VLM服务的截图分析功能",
  "keywords": ["Rust", "VLM", "代码编写", "VS Code"],
  "importance": 8,
  "app_name": "Visual Studio Code",
  "activity_type": "coding"
}
```

## 错误处理

新方法包含完善的错误处理：

1. **配置验证**: 检查VLM是否启用且配置有效
2. **Prompt加载失败**: 自动回退到默认Prompt
3. **JSON解析失败**: 返回详细的错误信息，包含原始响应
4. **API调用失败**: 返回HTTP状态码和错误详情

## 性能优化

- 使用全局单例缓存PromptService配置
- 避免重复加载配置文件
- 支持热重载Prompt模板

## 下一步

建议更新以下服务以使用新的`analyze_screenshot`方法：

1. `activity_summary_service.rs`
2. `screen_capture_service.rs` (如果直接调用VLM)
3. 任何其他直接使用VLM的服务

## 测试

建议添加单元测试验证：

1. Prompt变量替换功能
2. JSON响应解析
3. 回退机制
4. 各Provider的API调用

---

**更新时间**: 2025-12-04
**版本**: v1.0
