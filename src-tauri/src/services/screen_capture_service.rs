// Screen Capture Service
// 屏幕截图采集和变化检测服务

use crate::models::screen_context::ScreenContext;
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use img_hash::{HashAlg, HasherConfig, image::DynamicImage, image::ImageOutputFormat};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[cfg(windows)]
use winapi::um::winuser::{GetForegroundWindow, GetWindowTextW};

/// 活动窗口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWindowInfo {
    pub app_name: Option<String>,
    pub window_title: Option<String>,
}

/// 采集状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureStatus {
    pub is_running: bool,
    pub last_capture_at: Option<String>,
    pub total_captures_today: u32,
    pub skipped_count: u32,
}

/// 屏幕截图服务
pub struct ScreenCaptureService;

impl ScreenCaptureService {
    /// 创建新的屏幕截图服务实例
    pub fn new() -> Self {
        Self
    }

    /// 截取当前屏幕，返回 base64 编码的图片
    pub fn capture_screen(&self) -> Result<String> {
        // 获取所有屏幕
        let screens = xcap::Monitor::all().context("无法获取屏幕列表")?;

        // 使用第一个屏幕（主屏幕）
        let screen = screens
            .into_iter()
            .next()
            .context("没有可用的屏幕")?;

        // 截图
        let xcap_buffer = screen.capture_image().context("截图失败")?;

        // 转换为 DynamicImage (使用 img_hash 的 image 类型)
        // 需要从 xcap 的 ImageBuffer 转换到 img_hash::image 的 ImageBuffer
        use img_hash::image::{ImageBuffer, Rgba};
        let width = xcap_buffer.width();
        let height = xcap_buffer.height();
        let raw_pixels: Vec<u8> = xcap_buffer.into_raw();

        let img_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, raw_pixels)
            .context("无法创建 ImageBuffer")?;
        let image = DynamicImage::ImageRgba8(img_buffer);

        // 压缩并转换为 base64
        self.image_to_base64(&image)
    }

    /// 获取当前活动窗口信息
    pub fn get_active_window_info(&self) -> ActiveWindowInfo {
        #[cfg(windows)]
        {
            self.get_active_window_info_windows()
        }

        #[cfg(not(windows))]
        {
            ActiveWindowInfo {
                app_name: None,
                window_title: None,
            }
        }
    }

    /// Windows 平台获取活动窗口信息
    #[cfg(windows)]
    fn get_active_window_info_windows(&self) -> ActiveWindowInfo {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return ActiveWindowInfo {
                    app_name: None,
                    window_title: None,
                };
            }

            // 获取窗口标题
            let mut title: [u16; 512] = [0; 512];
            let len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);

            let window_title = if len > 0 {
                String::from_utf16_lossy(&title[0..len as usize])
            } else {
                String::new()
            };

            // 尝试从窗口标题提取应用名称（简化版本）
            let app_name = self.extract_app_name_from_title(&window_title);

            ActiveWindowInfo {
                app_name: Some(app_name),
                window_title: if window_title.is_empty() {
                    None
                } else {
                    Some(window_title)
                },
            }
        }
    }

    /// 从窗口标题提取应用名称（简化版）
    fn extract_app_name_from_title(&self, title: &str) -> String {
        // 尝试从标题中提取应用名称
        // 常见格式: "文件名 - 应用名" 或 "应用名"
        if let Some(pos) = title.rfind(" - ") {
            title[pos + 3..].to_string()
        } else if let Some(pos) = title.rfind(" — ") {
            title[pos + 3..].to_string()
        } else {
            title.to_string()
        }
    }

    /// 计算图片的感知哈希值
    pub fn calculate_image_hash(&self, image: &DynamicImage) -> Result<String> {
        let hasher = HasherConfig::new()
            .hash_alg(HashAlg::Gradient)
            .hash_size(8, 8)
            .to_hasher();

        // img_hash 3.2 可以直接hash DynamicImage
        let hash = hasher.hash_image(image);

        // 转换为 base64
        let hash_bytes: &[u8] = hash.as_bytes();
        Ok(general_purpose::STANDARD.encode(hash_bytes))
    }

    /// 比较两个哈希值，返回相似度 (0-1)
    pub fn compare_hashes(&self, hash1: &str, hash2: &str) -> Result<f32> {
        let bytes1 = general_purpose::STANDARD
            .decode(hash1)
            .context("解析 hash1 失败")?;
        let bytes2 = general_purpose::STANDARD
            .decode(hash2)
            .context("解析 hash2 失败")?;

        let h1: img_hash::ImageHash<Box<[u8]>> = img_hash::ImageHash::from_bytes(&bytes1)
            .map_err(|_| anyhow::anyhow!("从字节创建 hash1 失败"))?;
        let h2: img_hash::ImageHash<Box<[u8]>> = img_hash::ImageHash::from_bytes(&bytes2)
            .map_err(|_| anyhow::anyhow!("从字节创建 hash2 失败"))?;

        let distance = h1.dist(&h2);
        // 使用固定的 hash size (8x8 = 64)
        let max_distance = 64.0_f32;
        let similarity = 1.0 - (distance as f32 / max_distance);

        Ok(similarity)
    }

    /// 判断是否应该跳过本次采集（基于相似度阈值）
    pub fn should_skip_capture(
        &self,
        current_hash: &str,
        last_hash: Option<&str>,
        threshold: f32,
    ) -> Result<bool> {
        if let Some(last) = last_hash {
            let similarity = self.compare_hashes(current_hash, last)?;
            Ok(similarity >= threshold)
        } else {
            Ok(false)
        }
    }

    /// 将图片转换为 base64 字符串（压缩后）
    fn image_to_base64(&self, image: &DynamicImage) -> Result<String> {
        use img_hash::image::GenericImageView;

        // 简化：直接对图片数据进行base64编码
        // 格式: width(4字节) + height(4字节) + raw_rgba_bytes
        let width = image.width();
        let height = image.height();

        let mut buffer = Vec::new();
        buffer.extend_from_slice(&width.to_le_bytes());
        buffer.extend_from_slice(&height.to_le_bytes());
        buffer.extend_from_slice(image.as_bytes());

        Ok(general_purpose::STANDARD.encode(&buffer))
    }

    /// 从 base64 解码图片
    pub fn base64_to_image(&self, base64_str: &str) -> Result<DynamicImage> {
        use img_hash::image::{ImageBuffer, Rgba};

        let bytes = general_purpose::STANDARD
            .decode(base64_str)
            .context("Base64 解码失败")?;

        // 解析格式: width(4字节) + height(4字节) + raw_rgba_bytes
        if bytes.len() < 8 {
            return Err(anyhow::anyhow!("数据长度不足"));
        }

        let width = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let height = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let pixel_data = &bytes[8..];

        let img_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
            width,
            height,
            pixel_data.to_vec(),
        ).context("无法从原始数据创建图片")?;

        Ok(DynamicImage::ImageRgba8(img_buffer))
    }

    /// 执行一次完整的截图采集，返回 ScreenContext
    pub fn capture_once(&self, last_hash: Option<&str>, threshold: f32) -> Result<Option<ScreenContext>> {
        let start_time = std::time::Instant::now();

        // 截取屏幕
        let screenshot_base64 = self.capture_screen()?;

        // 解码图片以计算哈希
        let image = self.base64_to_image(&screenshot_base64)?;

        // 计算哈希
        let image_hash = self.calculate_image_hash(&image)?;

        // 检查是否应该跳过
        if self.should_skip_capture(&image_hash, last_hash, threshold)? {
            return Ok(None);
        }

        // 获取活动窗口信息
        let window_info = self.get_active_window_info();

        // 获取当前时间
        let captured_at = chrono::Local::now().to_rfc3339();

        // 计算处理时间
        let processing_time_ms = start_time.elapsed().as_millis() as i64;

        // 创建 ScreenContext（符合现有数据库模型）
        Ok(Some(ScreenContext {
            id: None,
            captured_at,
            app_name: window_info.app_name,
            window_title: window_info.window_title,
            activity_type: "unknown".to_string(), // 需要 VLM 分析后确定
            description: "".to_string(), // 需要 VLM 分析后填充
            key_content: None,
            screenshot_hash: Some(image_hash),
            screenshot_path: None, // 可选：如果需要保存原始截图
            processing_time_ms: Some(processing_time_ms),
        }))
    }
}

impl Default for ScreenCaptureService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageBuffer;

    #[test]
    fn test_extract_app_name() {
        let service = ScreenCaptureService::new();

        assert_eq!(
            service.extract_app_name_from_title("Document.txt - Notepad"),
            "Notepad"
        );

        assert_eq!(
            service.extract_app_name_from_title("Chrome"),
            "Chrome"
        );
    }

    #[test]
    fn test_hash_similarity() {
        let service = ScreenCaptureService::new();

        // 创建两个相同的小图片
        let img1 = DynamicImage::ImageRgb8(
            ImageBuffer::from_fn(100, 100, |x, y| {
                image::Rgb([((x + y) % 256) as u8, 128, 200])
            })
        );

        let hash1 = service.calculate_image_hash(&img1).unwrap();
        let hash2 = hash1.clone();

        let similarity = service.compare_hashes(&hash1, &hash2).unwrap();
        assert!((similarity - 1.0).abs() < 0.01, "相同图片的相似度应该接近 1.0");
    }
}
