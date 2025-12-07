// Screen Capture Service
// 屏幕截图采集和变化检测服务

use crate::models::screen_context::ScreenContext;
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use img_hash::{HashAlg, HasherConfig, image::DynamicImage};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(windows)]
use winapi::um::winuser::{GetCursorPos, GetForegroundWindow, GetWindowTextW};
#[cfg(windows)]
use winapi::shared::windef::POINT;

/// 活动窗口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWindowInfo {
    pub app_name: Option<String>,
    pub window_title: Option<String>,
}

/// 采集状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureStatus {
    pub is_running: bool,
    pub last_capture_at: Option<String>,
    pub total_captures_today: u32,
    pub skipped_count: u32,
    pub is_paused_by_idle: bool,  // 新增：是否因空闲而暂停
    pub idle_seconds: u64,        // 新增：当前空闲秒数
}

/// 屏幕截图服务
pub struct ScreenCaptureService;

impl ScreenCaptureService {
    /// 创建新的屏幕截图服务实例
    pub fn new() -> Self {
        Self
    }

    /// 获取鼠标所在的显示器
    fn get_monitor_at_cursor(&self) -> Option<xcap::Monitor> {
        #[cfg(windows)]
        {
            let cursor_pos = unsafe {
                let mut point = POINT { x: 0, y: 0 };
                if GetCursorPos(&mut point) != 0 {
                    Some((point.x, point.y))
                } else {
                    None
                }
            };

            if let Some((cx, cy)) = cursor_pos {
                if let Ok(monitors) = xcap::Monitor::all() {
                    for monitor in monitors {
                        let x = monitor.x();
                        let y = monitor.y();
                        let w = monitor.width() as i32;
                        let h = monitor.height() as i32;

                        // 检查鼠标是否在这个显示器范围内
                        if cx >= x && cx < x + w && cy >= y && cy < y + h {
                            return Some(monitor);
                        }
                    }
                }
            }
        }
        None
    }

    /// 截取当前屏幕，返回 base64 编码的图片
    pub fn capture_screen(&self) -> Result<String> {
        // 优先截取鼠标所在的屏幕
        let screen = self.get_monitor_at_cursor()
            .or_else(|| {
                // 如果获取失败，使用主屏幕或第一个屏幕
                xcap::Monitor::all().ok()?.into_iter()
                    .find(|m| m.is_primary())
                    .or_else(|| xcap::Monitor::all().ok()?.into_iter().next())
            })
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

    /// 基于应用名称自动识别活动类型
    pub fn detect_activity_type(&self, app_name: Option<&str>, window_title: Option<&str>) -> String {
        let app = app_name.unwrap_or("").to_lowercase();
        let title = window_title.unwrap_or("").to_lowercase();

        // 编码类工具
        let coding_apps = [
            "visual studio", "vs code", "vscode", "code", "intellij", "idea",
            "pycharm", "webstorm", "phpstorm", "rider", "clion", "goland",
            "sublime", "atom", "notepad++", "vim", "neovim", "emacs",
            "eclipse", "netbeans", "android studio", "xcode", "cursor",
            "terminal", "powershell", "cmd", "iterm", "warp", "hyper",
            "git", "github desktop", "sourcetree", "fork", "gitkraken",
        ];

        // 浏览器
        let browser_apps = [
            "chrome", "firefox", "edge", "safari", "opera", "brave",
            "vivaldi", "arc", "chromium",
        ];

        // 通讯工具
        let chat_apps = [
            "微信", "wechat", "qq", "钉钉", "dingtalk", "飞书", "feishu", "lark",
            "slack", "discord", "telegram", "teams", "zoom", "skype",
            "企业微信", "wecom", "whatsapp", "line",
        ];

        // 文档工具
        let document_apps = [
            "word", "excel", "powerpoint", "wps", "notion", "obsidian",
            "typora", "markdown", "onenote", "evernote", "印象笔记",
            "语雀", "yuque", "confluence", "wiki", "pdf", "acrobat",
            "preview", "pages", "numbers", "keynote",
        ];

        // 设计工具
        let design_apps = [
            "figma", "sketch", "photoshop", "illustrator", "xd",
            "indesign", "affinity", "canva", "pixelmator", "gimp",
            "blender", "maya", "3ds max", "cinema 4d", "zbrush",
        ];

        // 检查应用名称
        for keyword in coding_apps.iter() {
            if app.contains(keyword) || title.contains(keyword) {
                return "coding".to_string();
            }
        }

        for keyword in browser_apps.iter() {
            if app.contains(keyword) {
                return "browsing".to_string();
            }
        }

        for keyword in chat_apps.iter() {
            if app.contains(keyword) || title.contains(keyword) {
                return "chatting".to_string();
            }
        }

        for keyword in document_apps.iter() {
            if app.contains(keyword) || title.contains(keyword) {
                return "document".to_string();
            }
        }

        for keyword in design_apps.iter() {
            if app.contains(keyword) || title.contains(keyword) {
                return "design".to_string();
            }
        }

        "other".to_string()
    }

    /// 保存截图到文件（直接从 xcap 截图保存）
    /// 截图会保存到 dir/YYYY-MM-DD/ 子目录中，按日期分类存储
    /// 同时生成缩略图保存到 dir/YYYY-MM-DD/thumbs/ 子目录中
    pub fn save_screenshot_to_file(&self, dir: &PathBuf) -> Result<String> {
        // 生成当前日期和时间
        let now = chrono::Local::now();

        // 创建日期子文件夹 (格式: YYYY-MM-DD)
        let date_folder = now.format("%Y-%m-%d").to_string();
        let full_dir = dir.join(&date_folder);

        // 确保日期目录存在
        std::fs::create_dir_all(&full_dir).context("创建截图日期目录失败")?;

        // 生成文件名
        let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
        let nanos = now.timestamp_subsec_micros() % 1000000;
        let filename = format!("screenshot_{}_{:06}.png", timestamp, nanos);
        let file_path = full_dir.join(&filename);

        // 直接截图并保存（使用 xcap 的原生保存功能）
        // 优先截取鼠标所在的屏幕
        let screen = self.get_monitor_at_cursor()
            .or_else(|| {
                xcap::Monitor::all().ok()?.into_iter()
                    .find(|m| m.is_primary())
                    .or_else(|| xcap::Monitor::all().ok()?.into_iter().next())
            })
            .context("没有可用的屏幕")?;
        let xcap_buffer = screen.capture_image().context("截图失败")?;

        // 使用 image crate 保存（xcap 返回的是 image::ImageBuffer）
        xcap_buffer.save(&file_path).context("保存截图文件失败")?;

        // 异步生成缩略图（不阻塞主流程）
        // 提取图片尺寸和原始像素数据，避免跨 crate 类型问题
        let width = xcap_buffer.width();
        let height = xcap_buffer.height();
        let raw_pixels: Vec<u8> = xcap_buffer.into_raw();
        let thumb_dir = full_dir.join("thumbs");
        let thumb_filename = format!("thumb_{}_{:06}.jpg", timestamp, nanos);
        let thumb_path = thumb_dir.join(&thumb_filename);

        std::thread::spawn(move || {
            if let Err(e) = Self::generate_and_save_thumbnail_from_raw(
                width, height, &raw_pixels, &thumb_dir, &thumb_path
            ) {
                eprintln!("生成缩略图失败: {}", e);
            }
        });

        Ok(file_path.to_string_lossy().to_string())
    }

    /// 从原始像素数据生成并保存缩略图
    fn generate_and_save_thumbnail_from_raw(
        width: u32,
        height: u32,
        raw_pixels: &[u8],
        thumb_dir: &PathBuf,
        thumb_path: &PathBuf,
    ) -> Result<()> {
        use image::{GenericImageView, ImageBuffer, Rgba, DynamicImage};

        // 确保缩略图目录存在
        std::fs::create_dir_all(thumb_dir).context("创建缩略图目录失败")?;

        // 从原始数据重建 ImageBuffer
        let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
            width,
            height,
            raw_pixels.to_vec(),
        ).context("无法从原始数据创建图片")?;

        // 转换为 DynamicImage
        let dynamic_img = DynamicImage::ImageRgba8(img_buffer);

        // 计算缩略图尺寸（最大宽度或高度为 300px）
        let (w, h) = dynamic_img.dimensions();
        let max_size = 300u32;

        let (thumb_width, thumb_height) = if w > h {
            if w > max_size {
                let ratio = max_size as f32 / w as f32;
                (max_size, (h as f32 * ratio) as u32)
            } else {
                (w, h)
            }
        } else {
            if h > max_size {
                let ratio = max_size as f32 / h as f32;
                ((w as f32 * ratio) as u32, max_size)
            } else {
                (w, h)
            }
        };

        // 使用 Nearest 算法快速生成缩略图
        let thumbnail = dynamic_img.resize(
            thumb_width,
            thumb_height,
            image::imageops::FilterType::Nearest,
        );

        // 保存为 JPEG 格式（更小的文件大小）
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 70);
        thumbnail.write_with_encoder(encoder).context("编码缩略图失败")?;

        // 写入文件
        std::fs::write(thumb_path, &buffer).context("保存缩略图失败")?;

        Ok(())
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
        self.capture_once_with_save(last_hash, threshold, false, None)
    }

    /// 执行一次完整的截图采集，支持保存截图文件
    pub fn capture_once_with_save(
        &self,
        last_hash: Option<&str>,
        threshold: f32,
        save_screenshot: bool,
        screenshot_dir: Option<&PathBuf>,
    ) -> Result<Option<ScreenContext>> {
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

        // 自动识别活动类型
        let activity_type = self.detect_activity_type(
            window_info.app_name.as_deref(),
            window_info.window_title.as_deref(),
        );

        // 保存截图文件（如果启用）
        let screenshot_path = if save_screenshot {
            if let Some(dir) = screenshot_dir {
                match self.save_screenshot_to_file(dir) {
                    Ok(path) => Some(path),
                    Err(e) => {
                        eprintln!("保存截图失败: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        // 获取当前时间（使用本地时间格式，避免时区问题）
        let captured_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // 计算处理时间
        let processing_time_ms = start_time.elapsed().as_millis() as i64;

        // 创建 ScreenContext（符合现有数据库模型）
        Ok(Some(ScreenContext {
            id: None,
            captured_at,
            app_name: window_info.app_name,
            window_title: window_info.window_title,
            activity_type,
            description: "".to_string(), // 可由 VLM 分析后填充
            key_content: None,
            screenshot_hash: Some(image_hash),
            screenshot_path,
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
