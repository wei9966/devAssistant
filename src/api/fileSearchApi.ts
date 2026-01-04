import { invoke } from '@tauri-apps/api/core'

/**
 * 文件搜索结果接口
 */
export interface FileSearchResult {
  /** 文件名 */
  name: string
  /** 文件完整路径 */
  path: string
  /** 文件大小（字节） */
  size: number
  /** 修改时间（ISO 格式字符串） */
  modified_time: string
  /** 是否为目录 */
  is_dir: boolean
  /** 文件类型（扩展名或类型描述） */
  file_type: string
}

/**
 * 搜索文件
 * @param keyword 搜索关键词
 * @param searchPaths 搜索路径列表（可选，为空则搜索所有磁盘）
 * @param maxResults 最大结果数量（可选，默认500）
 * @returns 搜索结果列表
 */
export async function searchFiles(
  keyword: string,
  searchPaths?: string[],
  maxResults?: number
): Promise<FileSearchResult[]> {
  return await invoke('search_files', {
    keyword,
    searchPaths: searchPaths || [],
    maxResults: maxResults || 500
  })
}

/**
 * 使用系统默认程序打开文件
 * @param path 文件路径
 */
export async function openFile(path: string): Promise<void> {
  await invoke('open_file', { path })
}

/**
 * 在文件管理器中打开文件所在目录
 * @param path 文件路径
 */
export async function openFileInFolder(path: string): Promise<void> {
  await invoke('open_file_in_folder', { path })
}

/**
 * 获取所有磁盘驱动器
 * @returns 驱动器列表（如 ['C:', 'D:']）
 */
export async function getAllDrives(): Promise<string[]> {
  return await invoke('get_all_drives')
}
