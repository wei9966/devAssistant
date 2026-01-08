//! NTFS MFT Binary Parser Module
//!
//! 直接解析 NTFS MFT 二进制结构，从 $STANDARD_INFORMATION 获取时间戳，
//! 从 $DATA 属性获取文件大小，避免逐文件调用 metadata()。
//!
//! 参考文档：
//! - https://flatcap.github.io/linux-ntfs/ntfs/concepts/file_record.html
//! - https://flatcap.github.io/linux-ntfs/ntfs/attributes/standard_information.html

use std::collections::HashMap;

/// MFT 记录大小（字节）
pub const MFT_RECORD_SIZE: usize = 1024;

/// 扇区大小（字节）
pub const SECTOR_SIZE: usize = 512;

/// 属性类型常量
pub const ATTR_STANDARD_INFORMATION: u32 = 0x10;
pub const ATTR_FILE_NAME: u32 = 0x30;
pub const ATTR_DATA: u32 = 0x80;
pub const ATTR_END: u32 = 0xFFFFFFFF;

/// 文件记录标志
pub const FILE_RECORD_IN_USE: u16 = 0x0001;
pub const FILE_RECORD_IS_DIRECTORY: u16 = 0x0002;

/// 文件名类型
pub const FILE_NAME_POSIX: u8 = 0;
pub const FILE_NAME_WIN32: u8 = 1;
pub const FILE_NAME_DOS: u8 = 2;
pub const FILE_NAME_WIN32_AND_DOS: u8 = 3;

/// FILE Record Header 结构
/// 偏移 0x00 - 0x2F
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct FileRecordHeader {
    /// Magic number "FILE" (0x454C4946)
    pub magic: [u8; 4],
    /// Offset to Update Sequence
    pub usa_offset: u16,
    /// Size in words of Update Sequence
    pub usa_count: u16,
    /// $LogFile Sequence Number (LSN)
    pub lsn: u64,
    /// Sequence number
    pub sequence_number: u16,
    /// Hard link count
    pub hard_link_count: u16,
    /// Offset to first attribute
    pub first_attribute_offset: u16,
    /// Flags (0x01=in use, 0x02=directory)
    pub flags: u16,
    /// Real size of FILE record
    pub real_size: u32,
    /// Allocated size of FILE record
    pub allocated_size: u32,
    /// Base FILE record reference
    pub base_record_ref: u64,
    /// Next attribute ID
    pub next_attr_id: u16,
    /// Padding (XP+)
    pub padding: u16,
    /// MFT record number (XP+)
    pub mft_record_number: u32,
}

/// 驻留属性头
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ResidentAttributeHeader {
    /// Attribute type (0x10, 0x30, 0x80, etc.)
    pub attr_type: u32,
    /// Length including header
    pub length: u32,
    /// Non-resident flag (0x00 for resident)
    pub non_resident: u8,
    /// Name length (in characters)
    pub name_length: u8,
    /// Offset to name
    pub name_offset: u16,
    /// Flags
    pub flags: u16,
    /// Attribute ID
    pub attr_id: u16,
    /// Content length
    pub content_length: u32,
    /// Offset to content
    pub content_offset: u16,
    /// Indexed flag
    pub indexed: u8,
    /// Padding
    pub padding: u8,
}

/// 非驻留属性头
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct NonResidentAttributeHeader {
    /// Attribute type
    pub attr_type: u32,
    /// Length including header
    pub length: u32,
    /// Non-resident flag (0x01 for non-resident)
    pub non_resident: u8,
    /// Name length
    pub name_length: u8,
    /// Offset to name
    pub name_offset: u16,
    /// Flags
    pub flags: u16,
    /// Attribute ID
    pub attr_id: u16,
    /// Starting VCN
    pub starting_vcn: u64,
    /// Last VCN
    pub last_vcn: u64,
    /// Offset to data runs
    pub data_runs_offset: u16,
    /// Compression unit size
    pub compression_unit_size: u16,
    /// Padding
    pub padding: u32,
    /// Allocated size
    pub allocated_size: u64,
    /// Real size (actual file size)
    pub real_size: u64,
    /// Initialized data size
    pub initialized_size: u64,
}

/// $STANDARD_INFORMATION 属性内容
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct StandardInformation {
    /// Creation time (FILETIME)
    pub creation_time: u64,
    /// Modification time (FILETIME)
    pub modification_time: u64,
    /// MFT modification time
    pub mft_modification_time: u64,
    /// Access time
    pub access_time: u64,
    /// DOS file permissions/flags
    pub dos_permissions: u32,
    /// Maximum versions
    pub max_versions: u32,
    /// Version number
    pub version_number: u32,
    /// Class ID
    pub class_id: u32,
    // Windows 2000+ 扩展字段
    /// Owner ID
    pub owner_id: u32,
    /// Security ID
    pub security_id: u32,
    /// Quota charged
    pub quota_charged: u64,
    /// Update Sequence Number
    pub usn: u64,
}

/// $FILE_NAME 属性内容
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct FileNameAttribute {
    /// Parent directory reference
    pub parent_ref: u64,
    /// Creation time
    pub creation_time: u64,
    /// Modification time
    pub modification_time: u64,
    /// MFT modification time
    pub mft_modification_time: u64,
    /// Access time
    pub access_time: u64,
    /// Allocated size
    pub allocated_size: u64,
    /// Real size
    pub real_size: u64,
    /// Flags
    pub flags: u32,
    /// Reparse value
    pub reparse_value: u32,
    /// Name length (in characters)
    pub name_length: u8,
    /// Name type (0=POSIX, 1=Win32, 2=DOS, 3=Win32+DOS)
    pub name_type: u8,
    // File name follows (Unicode, variable length)
}

/// 解析后的 MFT 文件信息（扩展版，包含大小和时间）
#[derive(Debug, Clone)]
pub struct MftFileInfoExtended {
    /// 文件引用号
    pub file_ref_number: u64,
    /// 父目录引用号
    pub parent_ref_number: u64,
    /// 文件名
    pub file_name: String,
    /// 是否为目录
    pub is_directory: bool,
    /// 驱动器盘符
    pub drive_letter: char,
    /// 文件大小（字节）
    pub size: u64,
    /// 修改时间（FILETIME 格式）
    pub modification_time: u64,
    /// 创建时间（FILETIME 格式）
    pub creation_time: u64,
}

/// MFT 解析器
pub struct MftParser {
    /// MFT 记录大小
    record_size: usize,
}

impl MftParser {
    /// 创建新的 MFT 解析器
    pub fn new() -> Self {
        Self {
            record_size: MFT_RECORD_SIZE,
        }
    }

    /// 解析单条 MFT 记录
    ///
    /// # Arguments
    /// * `record_data` - 1024 字节的 MFT 记录数据
    /// * `record_number` - 记录编号
    /// * `drive_letter` - 驱动器盘符
    ///
    /// # Returns
    /// 解析成功返回文件信息，失败返回 None
    pub fn parse_record(
        &self,
        record_data: &[u8],
        record_number: u64,
        drive_letter: char,
    ) -> Option<MftFileInfoExtended> {
        if record_data.len() < self.record_size {
            return None;
        }

        // 验证 Magic number "FILE"
        if &record_data[0..4] != b"FILE" {
            return None;
        }

        // 解析 FILE record header
        let header = unsafe {
            std::ptr::read_unaligned(record_data.as_ptr() as *const FileRecordHeader)
        };

        // 检查记录是否在使用中
        if header.flags & FILE_RECORD_IN_USE == 0 {
            return None;
        }

        // 跳过系统元数据文件（记录号 < 24）
        let file_ref = record_number & 0x0000FFFFFFFFFFFF;
        if file_ref < 24 {
            return None;
        }

        // 应用 Update Sequence 修复
        let mut fixed_data = record_data.to_vec();
        if !self.apply_update_sequence(&mut fixed_data, &header) {
            log::warn!("Update Sequence 验证失败，记录号: {}", record_number);
            // 继续处理，某些情况下仍可解析
        }

        // 遍历属性，提取信息
        let is_directory = header.flags & FILE_RECORD_IS_DIRECTORY != 0;
        let mut file_name: Option<String> = None;
        let mut parent_ref: u64 = 0;
        let mut file_size: u64 = 0;
        let mut modification_time: u64 = 0;
        let mut creation_time: u64 = 0;
        let mut best_name_type: u8 = 0xFF; // 用于选择最佳文件名

        let mut offset = header.first_attribute_offset as usize;

        while offset + 4 <= fixed_data.len() {
            // 读取属性类型
            let attr_type = u32::from_le_bytes([
                fixed_data[offset],
                fixed_data[offset + 1],
                fixed_data[offset + 2],
                fixed_data[offset + 3],
            ]);

            // 检查是否到达属性列表末尾
            if attr_type == ATTR_END || attr_type == 0 {
                break;
            }

            // 读取属性长度
            if offset + 8 > fixed_data.len() {
                break;
            }
            let attr_length = u32::from_le_bytes([
                fixed_data[offset + 4],
                fixed_data[offset + 5],
                fixed_data[offset + 6],
                fixed_data[offset + 7],
            ]) as usize;

            if attr_length == 0 || attr_length > self.record_size || offset + attr_length > fixed_data.len() {
                break;
            }

            // 读取非驻留标志
            let non_resident = fixed_data.get(offset + 8).copied().unwrap_or(0);

            match attr_type {
                ATTR_STANDARD_INFORMATION => {
                    if non_resident == 0 {
                        // 驻留属性
                        if let Some((mod_time, create_time)) = self.parse_standard_information(&fixed_data, offset) {
                            modification_time = mod_time;
                            creation_time = create_time;
                        }
                    }
                }
                ATTR_FILE_NAME => {
                    if non_resident == 0 {
                        if let Some((name, parent, name_type)) = self.parse_file_name(&fixed_data, offset) {
                            // 优先使用 Win32 或 Win32+DOS 名称
                            if name_type == FILE_NAME_WIN32 || name_type == FILE_NAME_WIN32_AND_DOS {
                                file_name = Some(name);
                                parent_ref = parent;
                                best_name_type = name_type;
                            } else if file_name.is_none() || (name_type != FILE_NAME_DOS && best_name_type == 0xFF) {
                                // 如果还没有名称，或者当前不是 DOS 名称且之前没有更好的
                                file_name = Some(name);
                                parent_ref = parent;
                                best_name_type = name_type;
                            }
                        }
                    }
                }
                ATTR_DATA => {
                    // 只处理无名称的主数据流（跳过 ADS）
                    let name_length = fixed_data.get(offset + 9).copied().unwrap_or(0);
                    if name_length == 0 {
                        file_size = self.parse_data_attribute(&fixed_data, offset, non_resident);
                    }
                }
                _ => {}
            }

            offset += attr_length;
            // 确保对齐到 8 字节边界
            offset = (offset + 7) & !7;
        }

        // 如果没有找到文件名，返回 None
        let file_name = file_name?;

        Some(MftFileInfoExtended {
            file_ref_number: record_number,
            parent_ref_number: parent_ref,
            file_name,
            is_directory,
            drive_letter,
            size: file_size,
            modification_time,
            creation_time,
        })
    }

    /// 应用 Update Sequence 修复
    fn apply_update_sequence(&self, data: &mut [u8], header: &FileRecordHeader) -> bool {
        let usa_offset = header.usa_offset as usize;
        let usa_count = header.usa_count as usize;

        if usa_count == 0 || usa_offset + usa_count * 2 > data.len() {
            return false;
        }

        // 读取 Update Sequence Number
        let usn = u16::from_le_bytes([data[usa_offset], data[usa_offset + 1]]);

        // 验证并替换每个扇区末尾的序列号
        for i in 1..usa_count {
            let sector_end = i * SECTOR_SIZE - 2;
            if sector_end + 2 > data.len() {
                break;
            }

            // 验证扇区末尾的序列号
            let sector_usn = u16::from_le_bytes([data[sector_end], data[sector_end + 1]]);
            if sector_usn != usn {
                return false;
            }

            // 用 USA 中保存的原始字节替换
            let original_offset = usa_offset + i * 2;
            if original_offset + 2 <= data.len() {
                data[sector_end] = data[original_offset];
                data[sector_end + 1] = data[original_offset + 1];
            }
        }

        true
    }

    /// 解析 $STANDARD_INFORMATION 属性
    fn parse_standard_information(&self, data: &[u8], offset: usize) -> Option<(u64, u64)> {
        // 驻留属性头
        if offset + std::mem::size_of::<ResidentAttributeHeader>() > data.len() {
            return None;
        }

        let content_offset = u16::from_le_bytes([data[offset + 20], data[offset + 21]]) as usize;
        let content_length = u32::from_le_bytes([
            data[offset + 16],
            data[offset + 17],
            data[offset + 18],
            data[offset + 19],
        ]) as usize;

        let content_start = offset + content_offset;
        if content_start + 32 > data.len() || content_length < 32 {
            return None;
        }

        // 读取时间戳
        let creation_time = u64::from_le_bytes([
            data[content_start],
            data[content_start + 1],
            data[content_start + 2],
            data[content_start + 3],
            data[content_start + 4],
            data[content_start + 5],
            data[content_start + 6],
            data[content_start + 7],
        ]);

        let modification_time = u64::from_le_bytes([
            data[content_start + 8],
            data[content_start + 9],
            data[content_start + 10],
            data[content_start + 11],
            data[content_start + 12],
            data[content_start + 13],
            data[content_start + 14],
            data[content_start + 15],
        ]);

        Some((modification_time, creation_time))
    }

    /// 解析 $FILE_NAME 属性
    fn parse_file_name(&self, data: &[u8], offset: usize) -> Option<(String, u64, u8)> {
        if offset + std::mem::size_of::<ResidentAttributeHeader>() > data.len() {
            return None;
        }

        let content_offset = u16::from_le_bytes([data[offset + 20], data[offset + 21]]) as usize;
        let content_length = u32::from_le_bytes([
            data[offset + 16],
            data[offset + 17],
            data[offset + 18],
            data[offset + 19],
        ]) as usize;

        let content_start = offset + content_offset;
        if content_start + 66 > data.len() || content_length < 66 {
            return None;
        }

        // 读取父目录引用
        let parent_ref = u64::from_le_bytes([
            data[content_start],
            data[content_start + 1],
            data[content_start + 2],
            data[content_start + 3],
            data[content_start + 4],
            data[content_start + 5],
            data[content_start + 6],
            data[content_start + 7],
        ]);

        // 读取文件名长度和类型
        let name_length = data[content_start + 64] as usize;
        let name_type = data[content_start + 65];

        if name_length == 0 || content_start + 66 + name_length * 2 > data.len() {
            return None;
        }

        // 读取 Unicode 文件名
        let name_start = content_start + 66;
        let name_bytes: Vec<u16> = (0..name_length)
            .map(|i| {
                u16::from_le_bytes([
                    data[name_start + i * 2],
                    data[name_start + i * 2 + 1],
                ])
            })
            .collect();

        let file_name = String::from_utf16_lossy(&name_bytes);

        // 只保留低 48 位作为文件引用号
        let parent_ref_clean = parent_ref & 0x0000FFFFFFFFFFFF;

        Some((file_name, parent_ref_clean, name_type))
    }

    /// 解析 $DATA 属性获取文件大小
    fn parse_data_attribute(&self, data: &[u8], offset: usize, non_resident: u8) -> u64 {
        if non_resident == 0 {
            // 驻留属性 - 文件大小 = content_length
            if offset + 20 > data.len() {
                return 0;
            }
            u32::from_le_bytes([
                data[offset + 16],
                data[offset + 17],
                data[offset + 18],
                data[offset + 19],
            ]) as u64
        } else {
            // 非驻留属性 - 文件大小 = real_size (offset 0x30)
            if offset + 56 > data.len() {
                return 0;
            }
            u64::from_le_bytes([
                data[offset + 48],
                data[offset + 49],
                data[offset + 50],
                data[offset + 51],
                data[offset + 52],
                data[offset + 53],
                data[offset + 54],
                data[offset + 55],
            ])
        }
    }

    /// 将 FILETIME 转换为格式化的时间字符串
    pub fn filetime_to_string(filetime: u64) -> String {
        if filetime == 0 {
            return "未知".to_string();
        }

        // FILETIME: 1601-01-01 00:00:00 UTC 起的 100 纳秒间隔数
        // 转换为 Unix 时间戳（秒）
        // 1601 到 1970 之间的秒数: 11644473600
        const FILETIME_UNIX_DIFF: u64 = 11644473600;

        let seconds = filetime / 10_000_000;
        if seconds < FILETIME_UNIX_DIFF {
            return "未知".to_string();
        }

        let unix_timestamp = seconds - FILETIME_UNIX_DIFF;

        // 使用 chrono 转换
        use chrono::{DateTime, Local, TimeZone};
        match Local.timestamp_opt(unix_timestamp as i64, 0) {
            chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            _ => "未知".to_string(),
        }
    }

    /// 根据文件信息构建完整路径
    pub fn build_full_paths(files: &[MftFileInfoExtended]) -> Vec<(MftFileInfoExtended, String)> {
        // 按驱动器分组构建引用号映射
        let ref_map: HashMap<(char, u64), &MftFileInfoExtended> = files
            .iter()
            .map(|f| ((f.drive_letter, f.file_ref_number & 0x0000FFFFFFFFFFFF), f))
            .collect();

        let mut result = Vec::with_capacity(files.len());

        for file in files {
            let path = Self::resolve_path(file, &ref_map);
            result.push((file.clone(), path));
        }

        result
    }

    /// 解析单个文件的完整路径
    fn resolve_path(
        file: &MftFileInfoExtended,
        ref_map: &HashMap<(char, u64), &MftFileInfoExtended>,
    ) -> String {
        let mut path_parts = vec![file.file_name.clone()];
        let mut current_parent = file.parent_ref_number & 0x0000FFFFFFFFFFFF;
        let drive_letter = file.drive_letter;

        // 限制递归深度，防止循环引用
        let mut depth = 0;
        const MAX_DEPTH: usize = 100;

        while depth < MAX_DEPTH {
            // 根目录的引用号是 5
            if current_parent == 5 {
                break;
            }

            if let Some(parent_file) = ref_map.get(&(drive_letter, current_parent)) {
                path_parts.push(parent_file.file_name.clone());
                current_parent = parent_file.parent_ref_number & 0x0000FFFFFFFFFFFF;
            } else {
                break;
            }

            depth += 1;
        }

        // 反转并连接路径，添加盘符前缀
        path_parts.reverse();
        format!("{}:\\{}", drive_letter, path_parts.join("\\"))
    }
}

impl Default for MftParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
pub mod mft_direct_reader {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::shared::minwindef::{DWORD, FALSE, LPVOID};
    use winapi::um::fileapi::{CreateFileW, GetFileSizeEx, ReadFile, SetFilePointerEx, OPEN_EXISTING};
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::winbase::FILE_BEGIN;
    use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, HANDLE, LARGE_INTEGER};

    /// 读取缓冲区大小 (16 MB = 16384 条记录)
    const READ_BUFFER_SIZE: usize = 16 * 1024 * 1024;

    /// 进度回调类型
    pub type ProgressCallback = Box<dyn Fn(usize, usize, usize) + Send>;

    /// 直接读取 $MFT 文件
    pub struct MftDirectReader {
        parser: MftParser,
    }

    impl MftDirectReader {
        pub fn new() -> Self {
            Self {
                parser: MftParser::new(),
            }
        }

        /// 读取指定驱动器的所有文件
        ///
        /// # Arguments
        /// * `drive_letter` - 驱动器盘符
        /// * `progress_callback` - 可选的进度回调函数 (bytes_read, total_bytes, files_found)
        pub fn read_drive(
            &self,
            drive_letter: char,
            progress_callback: Option<ProgressCallback>,
        ) -> Result<Vec<MftFileInfoExtended>, String> {
            let mft_path = format!("\\\\.\\{}:\\$MFT", drive_letter);

            unsafe {
                // 打开 $MFT 文件
                let mft_path_wide: Vec<u16> = OsStr::new(&mft_path)
                    .encode_wide()
                    .chain(Some(0))
                    .collect();

                let handle = CreateFileW(
                    mft_path_wide.as_ptr(),
                    GENERIC_READ,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    ptr::null_mut(),
                    OPEN_EXISTING,
                    0,
                    ptr::null_mut(),
                );

                if handle == INVALID_HANDLE_VALUE {
                    return Err(format!(
                        "无法打开 {} $MFT 文件: {}",
                        drive_letter,
                        std::io::Error::last_os_error()
                    ));
                }

                let result = self.read_mft_file(handle, drive_letter, progress_callback);

                CloseHandle(handle);

                result
            }
        }

        /// 从句柄读取 MFT 文件内容
        unsafe fn read_mft_file(
            &self,
            handle: HANDLE,
            drive_letter: char,
            progress_callback: Option<ProgressCallback>,
        ) -> Result<Vec<MftFileInfoExtended>, String> {
            // 获取文件大小
            let mut file_size: LARGE_INTEGER = std::mem::zeroed();
            if GetFileSizeEx(handle, &mut file_size) == FALSE {
                return Err(format!("无法获取 MFT 文件大小: {}", std::io::Error::last_os_error()));
            }
            let total_size = *file_size.QuadPart() as usize;
            let total_records = total_size / MFT_RECORD_SIZE;

            log::info!(
                "驱动器 {}: MFT 文件大小 {} MB, 预计 {} 条记录",
                drive_letter,
                total_size / 1024 / 1024,
                total_records
            );

            let mut files = Vec::with_capacity(total_records);
            let mut buffer = vec![0u8; READ_BUFFER_SIZE];
            let mut bytes_read_total: usize = 0;
            let mut record_number: u64 = 0;
            let mut last_progress_report = 0usize;

            // 重置文件指针到开头
            let mut pos: LARGE_INTEGER = std::mem::zeroed();
            *pos.QuadPart_mut() = 0;
            SetFilePointerEx(handle, pos, ptr::null_mut(), FILE_BEGIN);

            loop {
                let mut bytes_read: DWORD = 0;

                let success = ReadFile(
                    handle,
                    buffer.as_mut_ptr() as LPVOID,
                    READ_BUFFER_SIZE as DWORD,
                    &mut bytes_read,
                    ptr::null_mut(),
                );

                if success == FALSE || bytes_read == 0 {
                    break;
                }

                bytes_read_total += bytes_read as usize;

                // 解析缓冲区中的记录
                let records_in_buffer = bytes_read as usize / MFT_RECORD_SIZE;
                for i in 0..records_in_buffer {
                    let record_start = i * MFT_RECORD_SIZE;
                    let record_end = record_start + MFT_RECORD_SIZE;

                    if record_end <= bytes_read as usize {
                        let record_data = &buffer[record_start..record_end];

                        if let Some(file_info) = self.parser.parse_record(record_data, record_number, drive_letter) {
                            files.push(file_info);
                        }
                    }

                    record_number += 1;
                }

                // 报告进度（每 5% 报告一次）
                if let Some(ref callback) = progress_callback {
                    let progress_percent = (bytes_read_total * 100) / total_size.max(1);
                    if progress_percent >= last_progress_report + 5 {
                        callback(bytes_read_total, total_size, files.len());
                        last_progress_report = progress_percent;
                    }
                }
            }

            // 最终进度报告
            if let Some(ref callback) = progress_callback {
                callback(bytes_read_total, total_size, files.len());
            }

            log::info!(
                "驱动器 {}: 读取完成, 共 {} 条有效文件记录",
                drive_letter,
                files.len()
            );

            Ok(files)
        }

        /// 读取所有 NTFS 驱动器
        pub fn read_all_drives(
            &self,
            _progress_callback: Option<Box<dyn Fn(char, usize, usize, usize) + Send>>,
        ) -> Result<Vec<MftFileInfoExtended>, String> {
            let mut all_files = Vec::new();

            // 检查 C-Z 驱动器
            for letter in b'C'..=b'Z' {
                let drive_letter = letter as char;
                let drive_path = format!("{}:\\", drive_letter);

                if !std::path::Path::new(&drive_path).exists() {
                    continue;
                }

                // 检查是否为 NTFS
                if !Self::is_ntfs_volume(drive_letter) {
                    log::info!("驱动器 {} 不是 NTFS 格式，跳过", drive_letter);
                    continue;
                }

                log::info!("开始读取驱动器 {}", drive_letter);

                // 注意：由于闭包类型限制，这里不传递回调
                // 如需进度，可以在外层通过驱动器级别的进度计算
                match self.read_drive(drive_letter, None) {
                    Ok(files) => {
                        log::info!("驱动器 {}: 读取到 {} 个文件", drive_letter, files.len());
                        all_files.extend(files);
                    }
                    Err(e) => {
                        log::warn!("读取驱动器 {} 失败: {}", drive_letter, e);
                    }
                }
            }

            Ok(all_files)
        }

        /// 检查卷是否为 NTFS
        fn is_ntfs_volume(drive_letter: char) -> bool {
            use winapi::um::fileapi::GetVolumeInformationW;

            unsafe {
                let root_path: Vec<u16> = OsStr::new(&format!("{}:\\", drive_letter))
                    .encode_wide()
                    .chain(Some(0))
                    .collect();

                let mut fs_name_buffer: [u16; 256] = [0; 256];
                let mut flags: DWORD = 0;

                let result = GetVolumeInformationW(
                    root_path.as_ptr(),
                    ptr::null_mut(),
                    0,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &mut flags,
                    fs_name_buffer.as_mut_ptr(),
                    256,
                );

                if result == FALSE {
                    return false;
                }

                let fs_name = String::from_utf16_lossy(
                    &fs_name_buffer[..fs_name_buffer.iter().position(|&c| c == 0).unwrap_or(0)],
                );

                fs_name.eq_ignore_ascii_case("NTFS")
            }
        }
    }

    impl Default for MftDirectReader {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filetime_conversion() {
        // 2024-01-01 00:00:00 UTC
        let filetime: u64 = 133476288000000000;
        let result = MftParser::filetime_to_string(filetime);
        println!("Converted time: {}", result);
        assert!(!result.is_empty());
        assert_ne!(result, "未知");
    }

    #[test]
    fn test_parser_creation() {
        let parser = MftParser::new();
        assert_eq!(parser.record_size, MFT_RECORD_SIZE);
    }

    #[test]
    #[cfg(windows)]
    fn test_read_mft() {
        use super::mft_direct_reader::MftDirectReader;

        // 需要管理员权限
        let reader = MftDirectReader::new();
        match reader.read_drive('C', None) {
            Ok(files) => {
                println!("读取到 {} 个文件", files.len());
                for file in files.iter().take(10) {
                    println!(
                        "{} {} - {} bytes, {}",
                        if file.is_directory { "[DIR]" } else { "[FILE]" },
                        file.file_name,
                        file.size,
                        MftParser::filetime_to_string(file.modification_time)
                    );
                }
            }
            Err(e) => {
                println!("读取失败（可能需要管理员权限）: {}", e);
            }
        }
    }
}
