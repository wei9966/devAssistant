# 批量截图分析服务 - 快速开始

## 文件位置

- 服务实现: `src-tauri/src/services/screenshot_batch_processor_service.rs`
- 集成指南: `src-tauri/BATCH_PROCESSOR_INTEGRATION.md`

## 核心功能

批量处理截图的 VLM 分析，参考 MineContext Python 版本：

```python
# MineContext 参考配置
batch_size = 20        # 20张图片触发
batch_timeout = 10     # 10秒超时
# 实际触发: 积累20张 或 超过20秒(timeout*2)
```

## 快速使用

### 1. 创建实例

```rust
use crate::services::screenshot_batch_processor_service::ScreenshotBatchProcessor;

let db_path = "path/to/database.db".to_string();
let processor = ScreenshotBatchProcessor::new(db_path);
```

### 2. 启动并运行

```rust
// 启动处理器
processor.start().await?;

// 在后台运行处理循环
let processor_clone = processor.clone();
tokio::spawn(async move {
    if let Err(e) = processor_clone.run_processing_loop().await {
        log::error!("批量处理循环错误: {}", e);
    }
});
```

### 3. 添加截图到队列

```rust
use crate::services::screenshot_batch_processor_service::BatchItem;

processor.enqueue(BatchItem {
    context_id: 123,
    screenshot_path: "/path/to/screenshot.png".to_string(),
}).await?;
```

### 4. 手动批量处理（补充历史数据）

```rust
// 获取未处理的截图
let unprocessed_ids = processor.get_unprocessed_screenshot_ids(Some(50)).await?;

// 批量处理
let result = processor.process_screenshots_manually(unprocessed_ids).await?;

println!("处理完成: 成功 {}, 失败 {}, 耗时 {}ms",
    result.successful,
    result.failed,
    result.processing_time_ms
);
```

## 配置参数

```rust
// 设置批量大小（默认 20）
processor.set_batch_size(30).await?;

// 设置超时时间（默认 10 秒，实际触发为 20 秒）
processor.set_batch_timeout(15).await?;
```

## 导出类型

```rust
pub use screenshot_batch_processor_service::{
    BatchAnalysisResult,    // 批量处理结果
    BatchItem,              // 批量处理项
    ProcessorState,         // 处理器状态
    ScreenshotBatchProcessor, // 批量处理器
};
```

## 关键接口

| 方法 | 描述 |
|------|------|
| `new(db_path)` | 创建处理器 |
| `start()` | 启动处理器 |
| `stop()` | 停止处理器 |
| `enqueue(item)` | 添加截图到队列 |
| `run_processing_loop()` | 运行处理循环（需要在后台运行） |
| `process_screenshots_manually(ids)` | 手动触发批量处理 |
| `count_unprocessed_screenshots()` | 统计未处理截图数量 |
| `get_unprocessed_screenshot_ids(limit)` | 获取未处理截图列表 |
| `get_state()` | 获取处理器状态 |

## 工作原理

1. **队列接收**: 通过 `mpsc::channel` 接收截图任务
2. **触发条件**:
   - 队列达到 20 张（可配置）
   - 超过 20 秒无新截图（10秒超时 × 2）
3. **并发分析**: 使用信号量限制 5 个并发 VLM 请求
4. **自动更新**: 分析完成后更新 `screen_contexts` 表的 `description` 字段

## 性能特点

- 异步处理，不阻塞主线程
- 批量处理减少数据库开销
- 并发控制避免资源耗尽
- 单个失败不影响其他任务

## 注意事项

1. 必须先调用 `start()` 再运行 `run_processing_loop()`
2. `run_processing_loop()` 是长期运行的任务，需要在后台 spawn
3. 确保 VLM 服务已配置并启用
4. 截图文件必须存在且可读

## 日志级别

```
INFO  - 启动/停止/批量触发信息
DEBUG - 队列状态、单个任务处理
ERROR - 处理失败详情
```

## 错误处理

所有方法返回 `Result<T>`，使用 `anyhow::Error` 提供详细错误信息：

```rust
match processor.enqueue(item).await {
    Ok(()) => log::info!("截图已加入队列"),
    Err(e) => log::error!("加入队列失败: {}", e),
}
```
