// Screenshot Batch Processor Service
// 批量截图分析服务 - 实现批量VLM分析处理
//
// ## 功能说明
//
// 本服务实现了批量截图分析功能，参考 MineContext Python 版本的设计：
// - 批量大小: 默认 20 张截图
// - 超时时间: 默认 10 秒
// - 触发条件: 当队列积累 20 张截图，或距离上次处理超过 20 秒时自动触发批量处理
//
// ## 核心特性
//
// 1. **异步队列处理**: 使用 tokio mpsc 通道接收截图任务
// 2. **智能触发机制**: 通过 tokio::select! 实现双条件触发（数量/超时）
// 3. **并发 VLM 分析**: 使用信号量限制并发数（默认 5 个），提高处理效率
// 4. **自动更新数据库**: 分析完成后自动更新 screen_contexts 表的 description 字段
//
// ## 使用示例
//
// ```rust
// // 创建处理器实例
// let processor = ScreenshotBatchProcessor::new(db_path.clone());
//
// // 配置批量参数（可选）
// processor.set_batch_size(30).await?;
// processor.set_batch_timeout(15).await?;
//
// // 启动处理器
// processor.start().await?;
//
// // 在后台运行处理循环
// tokio::spawn(async move {
//     processor.run_processing_loop().await
// });
//
// // 添加截图到队列（可从其他服务调用）
// processor.enqueue(BatchItem {
//     context_id: 123,
//     screenshot_path: "/path/to/screenshot.png".to_string(),
// }).await?;
//
// // 查询未处理的截图
// let unprocessed_count = processor.count_unprocessed_screenshots().await?;
//
// // 手动触发批量处理（用于补充处理历史数据）
// let ids = processor.get_unprocessed_screenshot_ids(Some(50)).await?;
// let result = processor.process_screenshots_manually(ids).await?;
// println!("处理完成: 成功 {}, 失败 {}", result.successful, result.failed);
// ```
//
// ## 性能优化
//
// - 使用信号量控制并发数，避免资源耗尽
// - 批量处理减少数据库连接开销
// - 异步 I/O 提高吞吐量
//
// ## 错误处理
//
// - 单个截图分析失败不影响其他截图的处理
// - 详细的日志记录便于问题排查
// - 返回统计结果包含成功和失败计数

use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{Duration, sleep, timeout};

use crate::services::vlm_service::VlmService;
use crate::services::context_store_service::ContextStoreService;

/// 批量处理项
#[derive(Debug, Clone)]
pub struct BatchItem {
    pub context_id: i64,
    pub screenshot_path: String,
}

/// 处理器状态
#[derive(Debug, Clone)]
pub struct ProcessorState {
    pub is_running: bool,
    pub batch_size: usize,
    pub batch_timeout_secs: u64,
}

impl Default for ProcessorState {
    fn default() -> Self {
        Self {
            is_running: false,
            batch_size: 20,        // 默认 20 张图片触发
            batch_timeout_secs: 10, // 默认 10 秒超时
        }
    }
}

/// 批量分析结果
#[derive(Debug)]
pub struct BatchAnalysisResult {
    pub total_processed: usize,
    pub successful: usize,
    pub failed: usize,
    pub processing_time_ms: u64,
}

/// 截图批量处理器
pub struct ScreenshotBatchProcessor {
    db_path: String,
    vlm_service: Arc<VlmService>,
    sender: Arc<Mutex<Option<mpsc::Sender<BatchItem>>>>,
    state: Arc<Mutex<ProcessorState>>,
}

impl Clone for ScreenshotBatchProcessor {
    fn clone(&self) -> Self {
        Self {
            db_path: self.db_path.clone(),
            vlm_service: Arc::clone(&self.vlm_service),
            sender: Arc::clone(&self.sender),
            state: Arc::clone(&self.state),
        }
    }
}

impl ScreenshotBatchProcessor {
    /// 创建新的批量处理器
    pub fn new(db_path: String) -> Self {
        Self {
            db_path: db_path.clone(),
            vlm_service: Arc::new(VlmService::new(db_path)),
            sender: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(ProcessorState::default())),
        }
    }

    /// 配置批量大小
    pub async fn set_batch_size(&self, size: usize) -> Result<()> {
        let mut state = self.state.lock().await;
        state.batch_size = size;
        Ok(())
    }

    /// 配置批量超时时间（秒）
    pub async fn set_batch_timeout(&self, timeout_secs: u64) -> Result<()> {
        let mut state = self.state.lock().await;
        state.batch_timeout_secs = timeout_secs;
        Ok(())
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> ProcessorState {
        self.state.lock().await.clone()
    }

    /// 启动批量处理器
    pub async fn start(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        if state.is_running {
            return Err(anyhow!("批量处理器已经在运行中"));
        }
        state.is_running = true;
        drop(state);

        log::info!("批量截图处理器已启动");
        Ok(())
    }

    /// 停止批量处理器
    pub async fn stop(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        state.is_running = false;
        log::info!("批量截图处理器已停止");
        Ok(())
    }

    /// 添加截图到处理队列
    pub async fn enqueue(&self, item: BatchItem) -> Result<()> {
        let sender = self.sender.lock().await;
        if let Some(tx) = sender.as_ref() {
            tx.send(item).await
                .context("添加截图到队列失败")?;
            Ok(())
        } else {
            Err(anyhow!("处理器未启动，无法添加任务"))
        }
    }

    /// 运行批量处理循环（主处理逻辑）
    pub async fn run_processing_loop(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<BatchItem>(100);

        // 设置内部 sender
        {
            let mut sender = self.sender.lock().await;
            *sender = Some(tx.clone());
        }

        let mut pending_batch: Vec<BatchItem> = Vec::new();
        let state = self.state.clone();
        let vlm_service = self.vlm_service.clone();
        let db_path = self.db_path.clone();

        loop {
            let current_state = state.lock().await.clone();

            if !current_state.is_running {
                log::info!("批量处理器已停止运行");
                break;
            }

            let batch_size = current_state.batch_size;
            let timeout_duration = Duration::from_secs(current_state.batch_timeout_secs);

            // 使用 tokio::select! 实现两种触发条件
            tokio::select! {
                // 条件 1: 接收到新的截图任务
                Some(item) = rx.recv() => {
                    pending_batch.push(item);
                    log::debug!("队列新增截图，当前待处理: {}/{}", pending_batch.len(), batch_size);

                    // 达到批量大小，立即处理
                    if pending_batch.len() >= batch_size {
                        log::info!("队列达到批量大小 ({}), 开始批量处理", batch_size);
                        if let Err(e) = Self::process_batch_internal(
                            &db_path,
                            &vlm_service,
                            &mut pending_batch,
                        ).await {
                            log::error!("批量处理失败: {}", e);
                        }
                    }
                }

                // 条件 2: 超时触发（距离上次处理超过 timeout * 2）
                _ = sleep(timeout_duration * 2) => {
                    if !pending_batch.is_empty() {
                        log::info!("超时触发批量处理，待处理: {} 张", pending_batch.len());
                        if let Err(e) = Self::process_batch_internal(
                            &db_path,
                            &vlm_service,
                            &mut pending_batch,
                        ).await {
                            log::error!("批量处理失败: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 内部批量处理逻辑
    async fn process_batch_internal(
        db_path: &str,
        vlm_service: &Arc<VlmService>,
        batch: &mut Vec<BatchItem>,
    ) -> Result<BatchAnalysisResult> {
        if batch.is_empty() {
            return Ok(BatchAnalysisResult {
                total_processed: 0,
                successful: 0,
                failed: 0,
                processing_time_ms: 0,
            });
        }

        let start_time = std::time::Instant::now();
        let total = batch.len();

        log::info!("开始批量处理 {} 张截图", total);

        // 并发分析，限制并发数为 5
        let semaphore = Arc::new(tokio::sync::Semaphore::new(5));
        let mut tasks = Vec::new();

        for item in batch.drain(..) {
            let vlm = vlm_service.clone();
            let db_path_owned = db_path.to_string();
            let permit = semaphore.clone().acquire_owned().await
                .context("获取并发许可失败")?;

            let task = tokio::spawn(async move {
                let _permit = permit; // 保持许可直到任务完成

                log::debug!("开始分析截图 ID: {}, 路径: {}", item.context_id, item.screenshot_path);

                // 读取截图文件并转换为 base64
                let image_base64 = match Self::load_image_as_base64(&item.screenshot_path).await {
                    Ok(data) => data,
                    Err(e) => {
                        log::error!("读取截图文件失败 (ID: {}): {}", item.context_id, e);
                        return Err(e);
                    }
                };

                // 调用 VLM 分析
                let analysis = match vlm.analyze_screenshot(&image_base64, None).await {
                    Ok(result) => result,
                    Err(e) => {
                        log::error!("VLM 分析失败 (ID: {}): {}", item.context_id, e);
                        return Err(e);
                    }
                };

                // 更新数据库
                let description = format!(
                    "{}\n应用: {}\n活动: {}",
                    analysis.summary,
                    analysis.app_name,
                    analysis.activity_type
                );
                let key_content = Some(analysis.keywords.join(", "));

                // 打开数据库连接并更新
                let conn = Connection::open(&db_path_owned)
                    .context("打开数据库连接失败")?;

                ContextStoreService::update_description(
                    &conn,
                    item.context_id,
                    &description,
                    key_content.as_deref(),
                ).context("更新数据库失败")?;

                log::debug!("截图分析完成 (ID: {})", item.context_id);
                Ok(())
            });

            tasks.push(task);
        }

        // 等待所有任务完成
        let results: Vec<_> = {
            let mut results = Vec::new();
            for task in tasks {
                results.push(task.await);
            }
            results
        };

        // 统计结果
        let mut successful = 0;
        let mut failed = 0;

        for result in results {
            match result {
                Ok(Ok(())) => successful += 1,
                Ok(Err(e)) => {
                    log::warn!("任务执行失败: {}", e);
                    failed += 1;
                }
                Err(e) => {
                    log::error!("任务崩溃: {}", e);
                    failed += 1;
                }
            }
        }

        let processing_time = start_time.elapsed();
        log::info!(
            "批量处理完成: 总计 {}, 成功 {}, 失败 {}, 耗时 {} ms",
            total,
            successful,
            failed,
            processing_time.as_millis()
        );

        Ok(BatchAnalysisResult {
            total_processed: total,
            successful,
            failed,
            processing_time_ms: processing_time.as_millis() as u64,
        })
    }

    /// 读取图片文件并转换为 base64
    async fn load_image_as_base64(path: &str) -> Result<String> {
        // 读取文件
        let image_data = tokio::fs::read(path).await
            .context(format!("读取文件失败: {}", path))?;

        // 使用 image crate 解析图片
        let img = image::load_from_memory(&image_data)
            .context("解析图片失败")?;

        // 转换为 PNG 格式并编码为 base64
        let mut png_data = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_data);
        img.write_to(&mut cursor, image::ImageFormat::Png)
            .context("编码 PNG 失败")?;

        // 转换为 base64
        use base64::{engine::general_purpose, Engine as _};
        Ok(general_purpose::STANDARD.encode(&png_data))
    }

    /// 手动触发批量处理指定的截图（用于补充处理历史数据）
    pub async fn process_screenshots_manually(
        &self,
        screenshot_ids: Vec<i64>,
    ) -> Result<BatchAnalysisResult> {
        let start_time = std::time::Instant::now();

        log::info!("手动触发批量处理 {} 张截图", screenshot_ids.len());

        // 从数据库加载截图信息
        let conn = Connection::open(&self.db_path)
            .context("打开数据库连接失败")?;

        let mut batch_items = Vec::new();
        for id in screenshot_ids {
            let path: Option<String> = conn.query_row(
                "SELECT screenshot_path FROM screen_contexts WHERE id = ?",
                rusqlite::params![id],
                |row| row.get(0),
            ).ok();

            if let Some(screenshot_path) = path {
                batch_items.push(BatchItem {
                    context_id: id,
                    screenshot_path,
                });
            } else {
                log::warn!("截图 ID {} 没有截图路径，跳过", id);
            }
        }

        drop(conn);

        // 调用内部批量处理
        Self::process_batch_internal(
            &self.db_path,
            &self.vlm_service,
            &mut batch_items,
        ).await
    }

    /// 查询未分析的截图数量
    pub async fn count_unprocessed_screenshots(&self) -> Result<usize> {
        let conn = Connection::open(&self.db_path)
            .context("打开数据库连接失败")?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM screen_contexts
             WHERE screenshot_path IS NOT NULL
             AND (description IS NULL OR description = '')",
            [],
            |row| row.get(0),
        ).context("查询未处理截图数量失败")?;

        Ok(count as usize)
    }

    /// 获取未分析的截图列表（用于批量补充处理）
    pub async fn get_unprocessed_screenshot_ids(&self, limit: Option<usize>) -> Result<Vec<i64>> {
        let conn = Connection::open(&self.db_path)
            .context("打开数据库连接失败")?;

        let limit_clause = if let Some(l) = limit {
            format!("LIMIT {}", l)
        } else {
            String::new()
        };

        let mut stmt = conn.prepare(&format!(
            "SELECT id FROM screen_contexts
             WHERE screenshot_path IS NOT NULL
             AND (description IS NULL OR description = '')
             ORDER BY captured_at DESC
             {}",
            limit_clause
        )).context("准备查询语句失败")?;

        let ids: Vec<i64> = stmt
            .query_map([], |row| row.get(0))
            .context("查询未处理截图失败")?
            .collect::<Result<Vec<_>, _>>()
            .context("收集结果失败")?;

        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_processor_creation() {
        let processor = ScreenshotBatchProcessor::new(":memory:".to_string());
        let state = processor.get_state().await;

        assert_eq!(state.batch_size, 20);
        assert_eq!(state.batch_timeout_secs, 10);
        assert!(!state.is_running);
    }

    #[tokio::test]
    async fn test_batch_configuration() {
        let processor = ScreenshotBatchProcessor::new(":memory:".to_string());

        processor.set_batch_size(30).await.unwrap();
        processor.set_batch_timeout(15).await.unwrap();

        let state = processor.get_state().await;
        assert_eq!(state.batch_size, 30);
        assert_eq!(state.batch_timeout_secs, 15);
    }

    #[tokio::test]
    async fn test_start_stop() {
        let processor = ScreenshotBatchProcessor::new(":memory:".to_string());

        assert!(!processor.get_state().await.is_running);

        processor.start().await.unwrap();
        assert!(processor.get_state().await.is_running);

        processor.stop().await.unwrap();
        assert!(!processor.get_state().await.is_running);
    }
}
