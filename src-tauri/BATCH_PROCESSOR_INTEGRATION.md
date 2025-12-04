# 批量截图分析服务集成指南

## 概述

`ScreenshotBatchProcessor` 服务实现了批量 VLM 分析功能，参考 MineContext Python 版本的设计，提供高效的截图批量处理能力。

## 核心特性

- **批量大小**: 默认 20 张截图触发
- **超时机制**: 默认 10 秒超时（实际触发时间为 20 秒）
- **并发处理**: 最多 5 个并发 VLM 请求
- **自动更新**: 分析完成后自动更新数据库

## 集成步骤

### 1. 在 ContextManager 中集成

修改 `src-tauri/src/services/context_manager_service.rs`：

```rust
use super::screenshot_batch_processor_service::{BatchItem, ScreenshotBatchProcessor};

pub struct ContextManager {
    state: Arc<Mutex<ContextManagerState>>,
    capture_service: Arc<ScreenCaptureService>,
    batch_processor: Arc<ScreenshotBatchProcessor>, // 新增
}

impl ContextManager {
    pub fn new() -> Self {
        let db_path = Self::get_db_path();

        Self {
            state: Arc::new(Mutex::new(ContextManagerState {
                is_running: false,
                last_hash: None,
                last_capture_at: None,
                total_captures_today: 0,
                skipped_count: 0,
                config: ContextConfig::default(),
            })),
            capture_service: Arc::new(ScreenCaptureService::new()),
            batch_processor: Arc::new(ScreenshotBatchProcessor::new(db_path)), // 新增
        }
    }

    /// 启动管理器（包含批量处理器）
    pub async fn start(&self) -> Result<()> {
        // ... 原有代码 ...

        // 启动批量处理器
        self.batch_processor.start().await?;

        // 在后台运行批量处理循环
        let processor = self.batch_processor.clone();
        tokio::spawn(async move {
            if let Err(e) = processor.run_processing_loop().await {
                log::error!("批量处理循环错误: {}", e);
            }
        });

        Ok(())
    }

    /// 采集截图后，如果保存了文件，则加入批量处理队列
    async fn capture_and_enqueue(&self, conn: &Connection) -> Result<()> {
        let config = self.get_config().await;

        // 执行截图采集
        let context = self.capture_service
            .capture_once_with_save(
                self.state.lock().await.last_hash.as_deref(),
                config.similarity_threshold,
                config.save_screenshots,
                Some(&config.get_screenshot_dir()),
            )?;

        if let Some(ctx) = context {
            // 保存到数据库
            let context_id = ContextStoreService::save_context(conn, &ctx)?;

            // 如果有截图路径，加入批量处理队列
            if let Some(screenshot_path) = ctx.screenshot_path {
                self.batch_processor.enqueue(BatchItem {
                    context_id,
                    screenshot_path,
                }).await?;

                log::debug!("截图已加入批量处理队列，ID: {}", context_id);
            }

            // 更新状态
            let mut state = self.state.lock().await;
            state.last_hash = ctx.screenshot_hash;
            state.total_captures_today += 1;
        } else {
            let mut state = self.state.lock().await;
            state.skipped_count += 1;
        }

        Ok(())
    }
}
```

### 2. 创建 Tauri 命令

在 `src-tauri/src/commands/context_commands.rs` 中添加：

```rust
use crate::services::screenshot_batch_processor_service::{
    BatchItem, BatchAnalysisResult, ScreenshotBatchProcessor,
};

/// 查询未处理的截图数量
#[tauri::command]
pub async fn count_unprocessed_screenshots(
    db_conn: State<'_, DbConnection>,
) -> Result<usize, String> {
    let db_path = get_db_path();
    let processor = ScreenshotBatchProcessor::new(db_path);

    processor
        .count_unprocessed_screenshots()
        .await
        .map_err(|e| e.to_string())
}

/// 手动触发批量处理
#[tauri::command]
pub async fn process_screenshots_batch(
    db_conn: State<'_, DbConnection>,
    limit: Option<usize>,
) -> Result<BatchAnalysisResult, String> {
    let db_path = get_db_path();
    let processor = ScreenshotBatchProcessor::new(db_path);

    // 获取未处理的截图 ID
    let ids = processor
        .get_unprocessed_screenshot_ids(limit)
        .await
        .map_err(|e| e.to_string())?;

    // 批量处理
    processor
        .process_screenshots_manually(ids)
        .await
        .map_err(|e| e.to_string())
}

/// 配置批量处理参数
#[tauri::command]
pub async fn configure_batch_processor(
    batch_size: Option<usize>,
    timeout_secs: Option<u64>,
) -> Result<(), String> {
    let db_path = get_db_path();
    let processor = ScreenshotBatchProcessor::new(db_path);

    if let Some(size) = batch_size {
        processor.set_batch_size(size).await.map_err(|e| e.to_string())?;
    }

    if let Some(timeout) = timeout_secs {
        processor.set_batch_timeout(timeout).await.map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

### 3. 注册命令

在 `src-tauri/src/main.rs` 中注册命令：

```rust
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... 其他命令 ...
            count_unprocessed_screenshots,
            process_screenshots_batch,
            configure_batch_processor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 前端集成

### TypeScript API

在 `src/api/contextApi.ts` 中添加：

```typescript
import { invoke } from '@tauri-apps/api/core';

export interface BatchAnalysisResult {
  totalProcessed: number;
  successful: number;
  failed: number;
  processingTimeMs: number;
}

export interface BatchProcessorConfig {
  batchSize?: number;
  timeoutSecs?: number;
}

export const contextApi = {
  // 查询未处理的截图数量
  async countUnprocessedScreenshots(): Promise<number> {
    return await invoke('count_unprocessed_screenshots');
  },

  // 手动触发批量处理
  async processScreenshotsBatch(limit?: number): Promise<BatchAnalysisResult> {
    return await invoke('process_screenshots_batch', { limit });
  },

  // 配置批量处理参数
  async configureBatchProcessor(config: BatchProcessorConfig): Promise<void> {
    return await invoke('configure_batch_processor', {
      batchSize: config.batchSize,
      timeoutSecs: config.timeoutSecs,
    });
  },
};
```

### Vue 组件示例

```vue
<template>
  <div class="batch-processor-panel">
    <h3>批量截图分析</h3>

    <div class="stats">
      <p>未处理截图: {{ unprocessedCount }}</p>
      <p v-if="processing">处理中...</p>
      <p v-if="lastResult">
        最后处理结果: 成功 {{ lastResult.successful }}, 失败 {{ lastResult.failed }},
        耗时 {{ lastResult.processingTimeMs }}ms
      </p>
    </div>

    <div class="actions">
      <button @click="refreshCount">刷新统计</button>
      <button @click="processBatch(50)" :disabled="processing">
        处理 50 张
      </button>
      <button @click="processBatch()" :disabled="processing">
        处理全部
      </button>
    </div>

    <div class="config">
      <h4>配置</h4>
      <label>
        批量大小:
        <input v-model.number="config.batchSize" type="number" min="1" max="100" />
      </label>
      <label>
        超时时间(秒):
        <input v-model.number="config.timeoutSecs" type="number" min="5" max="60" />
      </label>
      <button @click="saveConfig">保存配置</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { contextApi } from '@/api/contextApi';
import type { BatchAnalysisResult } from '@/api/contextApi';

const unprocessedCount = ref(0);
const processing = ref(false);
const lastResult = ref<BatchAnalysisResult | null>(null);
const config = ref({
  batchSize: 20,
  timeoutSecs: 10,
});

async function refreshCount() {
  unprocessedCount.value = await contextApi.countUnprocessedScreenshots();
}

async function processBatch(limit?: number) {
  if (processing.value) return;

  processing.value = true;
  try {
    lastResult.value = await contextApi.processScreenshotsBatch(limit);
    await refreshCount();
  } catch (error) {
    console.error('批量处理失败:', error);
    alert(`批量处理失败: ${error}`);
  } finally {
    processing.value = false;
  }
}

async function saveConfig() {
  try {
    await contextApi.configureBatchProcessor(config.value);
    alert('配置已保存');
  } catch (error) {
    console.error('保存配置失败:', error);
    alert(`保存配置失败: ${error}`);
  }
}

onMounted(() => {
  refreshCount();
});
</script>
```

## 性能调优建议

1. **批量大小**: 根据 VLM API 速度调整，一般 20-50 张为佳
2. **并发数**: 代码中默认 5 个并发，可根据 API 限流调整
3. **超时时间**: 根据网络延迟和 API 响应时间调整

## 监控和日志

批量处理器会输出详细的日志：

```
[INFO] 批量截图处理器已启动
[DEBUG] 队列新增截图，当前待处理: 1/20
[INFO] 队列达到批量大小 (20), 开始批量处理
[INFO] 开始批量处理 20 张截图
[DEBUG] 开始分析截图 ID: 123, 路径: /path/to/screenshot.png
[DEBUG] 截图分析完成 (ID: 123)
[INFO] 批量处理完成: 总计 20, 成功 18, 失败 2, 耗时 12345 ms
```

## 故障排查

### 问题: 批量处理不触发

- 检查处理器是否已启动: `processor.start().await`
- 检查是否运行处理循环: `run_processing_loop()`
- 查看日志确认截图是否加入队列

### 问题: VLM 分析失败率高

- 检查 VLM 配置是否正确
- 检查 API Key 是否有效
- 检查网络连接
- 调整并发数和超时时间

### 问题: 数据库更新失败

- 检查数据库文件权限
- 检查 context_id 是否有效
- 查看数据库日志

## 最佳实践

1. 在应用启动时启动批量处理器
2. 定期查询未处理截图数量，提醒用户处理
3. 在应用退出前等待批量处理完成
4. 记录处理结果用于统计和分析
