//! NTFS MFT (Master File Table) Reader Module
//!
//! 使用 Windows API 读取 NTFS 文件系统的 MFT 记录，
//! 通过 FSCTL_ENUM_USN_DATA 快速枚举所有文件。

use serde::{Deserialize, Serialize};

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::path::Path;
#[cfg(windows)]
use std::ptr;

/// MFT 文件信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MftFileInfo {
    /// 文件引用号 (File Reference Number)
    pub file_ref_number: u64,
    /// 父目录引用号
    pub parent_ref_number: u64,
    /// 文件名
    pub file_name: String,
    /// 是否为目录
    pub is_directory: bool,
    /// 驱动器盘符
    pub drive_letter: char,
}

/// MFT 读取器
pub struct MftReader;

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::mem;
    use winapi::shared::minwindef::{DWORD, FALSE, LPVOID};
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::ioapiset::DeviceIoControl;
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{
        TokenElevation, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, HANDLE,
        TOKEN_ELEVATION, TOKEN_QUERY,
    };

    // FSCTL 控制码
    const FSCTL_ENUM_USN_DATA: DWORD = 0x000900b3;

    /// MFT_ENUM_DATA_V0 结构体
    #[repr(C)]
    #[derive(Default)]
    struct MftEnumData {
        start_file_reference_number: u64,
        low_usn: i64,
        high_usn: i64,
    }

    /// USN_RECORD 结构体头部
    #[repr(C)]
    struct UsnRecordHeader {
        record_length: u32,
        major_version: u16,
        minor_version: u16,
    }

    /// USN_RECORD_V2 结构体
    #[repr(C)]
    struct UsnRecordV2 {
        record_length: u32,
        major_version: u16,
        minor_version: u16,
        file_reference_number: u64,
        parent_file_reference_number: u64,
        usn: i64,
        time_stamp: i64,
        reason: u32,
        source_info: u32,
        security_id: u32,
        file_attributes: u32,
        file_name_length: u16,
        file_name_offset: u16,
        // file_name follows
    }

    /// USN_RECORD_V3 结构体 (用于 ReFS 等)
    #[repr(C)]
    struct UsnRecordV3 {
        record_length: u32,
        major_version: u16,
        minor_version: u16,
        file_reference_number: [u8; 16], // FILE_ID_128
        parent_file_reference_number: [u8; 16], // FILE_ID_128
        usn: i64,
        time_stamp: i64,
        reason: u32,
        source_info: u32,
        security_id: u32,
        file_attributes: u32,
        file_name_length: u16,
        file_name_offset: u16,
        // file_name follows
    }

    // FILE_ATTRIBUTE_DIRECTORY
    const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;

    impl MftReader {
        /// 检查是否具有管理员权限
        pub fn check_admin_privilege() -> bool {
            unsafe {
                let mut token_handle: HANDLE = ptr::null_mut();
                let process_handle = GetCurrentProcess();

                if OpenProcessToken(process_handle, TOKEN_QUERY, &mut token_handle) == FALSE {
                    return false;
                }

                let mut elevation: TOKEN_ELEVATION = mem::zeroed();
                let mut return_length: DWORD = 0;

                let result = GetTokenInformation(
                    token_handle,
                    TokenElevation,
                    &mut elevation as *mut _ as LPVOID,
                    mem::size_of::<TOKEN_ELEVATION>() as DWORD,
                    &mut return_length,
                );

                CloseHandle(token_handle);

                result != FALSE && elevation.TokenIsElevated != 0
            }
        }

        /// 读取指定驱动器的所有文件
        ///
        /// # Arguments
        /// * `drive_letter` - 驱动器盘符 (如 'C')
        ///
        /// # Returns
        /// 返回文件信息列表
        pub fn read_drive(drive_letter: char) -> Result<Vec<MftFileInfo>, String> {
            if !Self::check_admin_privilege() {
                return Err("需要管理员权限才能读取 MFT".to_string());
            }

            let volume_path = format!("\\\\.\\{}:", drive_letter);
            Self::read_volume(&volume_path, drive_letter)
        }

        /// 读取所有 NTFS 驱动器
        pub fn read_all_drives() -> Result<Vec<MftFileInfo>, String> {
            if !Self::check_admin_privilege() {
                return Err("需要管理员权限才能读取 MFT".to_string());
            }

            let mut all_files = Vec::new();

            // 检查 A-Z 驱动器
            for letter in b'C'..=b'Z' {
                let drive = format!("{}:\\", letter as char);
                let path = Path::new(&drive);

                if path.exists() {
                    // 检查是否为 NTFS 文件系统
                    if Self::is_ntfs_volume(letter as char) {
                        match Self::read_drive(letter as char) {
                            Ok(files) => {
                                log::info!(
                                    "驱动器 {}: 读取到 {} 个文件",
                                    letter as char,
                                    files.len()
                                );
                                all_files.extend(files);
                            }
                            Err(e) => {
                                log::warn!("读取驱动器 {} 失败: {}", letter as char, e);
                            }
                        }
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

                // 转换文件系统名称
                let fs_name = String::from_utf16_lossy(
                    &fs_name_buffer[..fs_name_buffer.iter().position(|&c| c == 0).unwrap_or(0)],
                );

                fs_name.eq_ignore_ascii_case("NTFS")
            }
        }

        /// 读取指定卷的 MFT
        fn read_volume(volume_path: &str, drive_letter: char) -> Result<Vec<MftFileInfo>, String> {
            unsafe {
                // 打开卷
                let volume_path_wide: Vec<u16> = OsStr::new(volume_path)
                    .encode_wide()
                    .chain(Some(0))
                    .collect();

                let handle = CreateFileW(
                    volume_path_wide.as_ptr(),
                    GENERIC_READ,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    ptr::null_mut(),
                    OPEN_EXISTING,
                    0,
                    ptr::null_mut(),
                );

                if handle == INVALID_HANDLE_VALUE {
                    return Err(format!(
                        "无法打开卷 {}: 错误码 {}",
                        volume_path,
                        std::io::Error::last_os_error()
                    ));
                }

                let result = Self::enumerate_usn_data(handle, drive_letter);

                CloseHandle(handle);

                result
            }
        }

        /// 枚举 USN 数据
        unsafe fn enumerate_usn_data(handle: HANDLE, drive_letter: char) -> Result<Vec<MftFileInfo>, String> {
            let mut files = Vec::new();

            // 输出缓冲区 (64KB)
            let buffer_size = 64 * 1024;
            let mut buffer: Vec<u8> = vec![0u8; buffer_size];

            // 初始化枚举数据
            let mut enum_data = MftEnumData {
                start_file_reference_number: 0,
                low_usn: 0,
                high_usn: i64::MAX,
            };

            loop {
                let mut bytes_returned: DWORD = 0;

                let success = DeviceIoControl(
                    handle,
                    FSCTL_ENUM_USN_DATA,
                    &mut enum_data as *mut _ as LPVOID,
                    mem::size_of::<MftEnumData>() as DWORD,
                    buffer.as_mut_ptr() as LPVOID,
                    buffer_size as DWORD,
                    &mut bytes_returned,
                    ptr::null_mut(),
                );

                if success == FALSE {
                    let error = std::io::Error::last_os_error();
                    // ERROR_HANDLE_EOF = 38, 表示枚举结束
                    if error.raw_os_error() == Some(38) {
                        break;
                    }
                    return Err(format!("DeviceIoControl 失败: {}", error));
                }

                if bytes_returned < 8 {
                    break;
                }

                // 第一个 8 字节是下一次枚举的起始位置
                let next_usn = ptr::read_unaligned(buffer.as_ptr() as *const u64);

                // 解析 USN 记录
                let mut offset = 8usize;
                while offset < bytes_returned as usize {
                    let record_ptr = buffer.as_ptr().add(offset);

                    // 读取记录头部
                    let header = ptr::read_unaligned(record_ptr as *const UsnRecordHeader);

                    if header.record_length == 0 {
                        break;
                    }

                    // 根据版本解析记录
                    if let Some(file_info) = Self::parse_usn_record(record_ptr, &header, drive_letter) {
                        files.push(file_info);
                    }

                    offset += header.record_length as usize;

                    // 确保对齐到 8 字节边界
                    offset = (offset + 7) & !7;
                }

                // 更新起始位置
                enum_data.start_file_reference_number = next_usn;
            }

            Ok(files)
        }

        /// 解析 USN 记录
        unsafe fn parse_usn_record(
            record_ptr: *const u8,
            header: &UsnRecordHeader,
            drive_letter: char,
        ) -> Option<MftFileInfo> {
            match header.major_version {
                2 => Self::parse_usn_record_v2(record_ptr, drive_letter),
                3 => Self::parse_usn_record_v3(record_ptr, drive_letter),
                _ => None,
            }
        }

        /// 解析 USN_RECORD_V2
        unsafe fn parse_usn_record_v2(record_ptr: *const u8, drive_letter: char) -> Option<MftFileInfo> {
            let record = ptr::read_unaligned(record_ptr as *const UsnRecordV2);

            // 提取文件名
            let file_name_ptr = record_ptr.add(record.file_name_offset as usize) as *const u16;
            let file_name_len = (record.file_name_length / 2) as usize;

            let file_name_slice = std::slice::from_raw_parts(file_name_ptr, file_name_len);
            let file_name = String::from_utf16_lossy(file_name_slice);

            // 跳过系统文件 (文件引用号小于 24 的是系统元数据文件)
            let file_ref = record.file_reference_number & 0x0000FFFFFFFFFFFF;
            if file_ref < 24 {
                return None;
            }

            Some(MftFileInfo {
                file_ref_number: record.file_reference_number,
                parent_ref_number: record.parent_file_reference_number,
                file_name,
                is_directory: (record.file_attributes & FILE_ATTRIBUTE_DIRECTORY) != 0,
                drive_letter,
            })
        }

        /// 解析 USN_RECORD_V3
        unsafe fn parse_usn_record_v3(record_ptr: *const u8, drive_letter: char) -> Option<MftFileInfo> {
            let record = ptr::read_unaligned(record_ptr as *const UsnRecordV3);

            // 提取文件名
            let file_name_ptr = record_ptr.add(record.file_name_offset as usize) as *const u16;
            let file_name_len = (record.file_name_length / 2) as usize;

            let file_name_slice = std::slice::from_raw_parts(file_name_ptr, file_name_len);
            let file_name = String::from_utf16_lossy(file_name_slice);

            // 对于 V3 记录，文件引用号是 128 位的
            // 简单处理：使用低 64 位
            let file_ref = u64::from_le_bytes(record.file_reference_number[0..8].try_into().ok()?);
            let parent_ref = u64::from_le_bytes(record.parent_file_reference_number[0..8].try_into().ok()?);

            // 跳过系统文件
            let file_ref_low = file_ref & 0x0000FFFFFFFFFFFF;
            if file_ref_low < 24 {
                return None;
            }

            Some(MftFileInfo {
                file_ref_number: file_ref,
                parent_ref_number: parent_ref,
                file_name,
                is_directory: (record.file_attributes & FILE_ATTRIBUTE_DIRECTORY) != 0,
                drive_letter,
            })
        }

        /// 根据文件引用号构建完整路径
        ///
        /// 此方法使用文件引用号映射表来重建完整的文件路径
        pub fn build_full_paths(files: &[MftFileInfo]) -> Vec<(String, bool)> {
            use std::collections::HashMap;

            // 按驱动器分组构建引用号映射
            // 键: (drive_letter, file_ref_number)
            let ref_map: HashMap<(char, u64), &MftFileInfo> = files
                .iter()
                .map(|f| ((f.drive_letter, f.file_ref_number & 0x0000FFFFFFFFFFFF), f))
                .collect();

            let mut result = Vec::with_capacity(files.len());

            for file in files {
                let path = Self::resolve_path(file, &ref_map);
                result.push((path, file.is_directory));
            }

            result
        }

        /// 解析单个文件的完整路径
        fn resolve_path(
            file: &MftFileInfo,
            ref_map: &std::collections::HashMap<(char, u64), &MftFileInfo>,
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
}

#[cfg(not(windows))]
impl MftReader {
    /// 检查是否具有管理员权限 (非 Windows 平台)
    pub fn check_admin_privilege() -> bool {
        false
    }

    /// 读取指定驱动器的所有文件 (非 Windows 平台)
    pub fn read_drive(_drive_letter: char) -> Result<Vec<MftFileInfo>, String> {
        Err("MFT 读取仅支持 Windows 平台".to_string())
    }

    /// 读取所有 NTFS 驱动器 (非 Windows 平台)
    pub fn read_all_drives() -> Result<Vec<MftFileInfo>, String> {
        Err("MFT 读取仅支持 Windows 平台".to_string())
    }

    /// 根据文件引用号构建完整路径 (非 Windows 平台)
    pub fn build_full_paths(_files: &[MftFileInfo]) -> Vec<(String, bool)> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_admin_privilege() {
        let is_admin = MftReader::check_admin_privilege();
        println!("Is admin: {}", is_admin);
    }

    #[test]
    #[cfg(windows)]
    fn test_read_drive() {
        if !MftReader::check_admin_privilege() {
            println!("跳过测试: 需要管理员权限");
            return;
        }

        match MftReader::read_drive('C') {
            Ok(files) => {
                println!("读取到 {} 个文件", files.len());
                // 打印前 10 个文件
                for file in files.iter().take(10) {
                    println!(
                        "{} {} (ref: {}, parent: {})",
                        if file.is_directory { "[DIR]" } else { "[FILE]" },
                        file.file_name,
                        file.file_ref_number,
                        file.parent_ref_number
                    );
                }
            }
            Err(e) => {
                println!("读取失败: {}", e);
            }
        }
    }
}
