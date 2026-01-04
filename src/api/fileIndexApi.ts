import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/**
 * 文件索引记录接口
 */
export interface FileIndexRecord {
  /** 记录ID */
  id?: number
  /** 文件名 */
  name: string
  /** 文件完整路径 */
  path: string
  /** 文件大小（字节） */
  size: number
  /** 修改时间（格式化字符串） */
  modifiedTime: string
  /** 是否为目录 */
  isDir: boolean
  /** 文件类型（扩展名） */
  fileType: string
  /** 所属驱动器 */
  drive: string
  /** 索引时间 */
  indexedAt?: string
}

/**
 * 索引统计信息
 */
export interface IndexStats {
  /** 总文件数 */
  totalFiles: number
  /** 总目录数 */
  totalDirs: number
  /** 已索引的驱动器列表 */
  indexedDrives: string[]
  /** 最后更新时间 */
  lastUpdated?: string
}

/**
 * 索引状态
 */
export interface IndexStatus {
  /** 是否正在索引 */
  isIndexing: boolean
  /** 是否正在监控文件变化 */
  isWatching: boolean
  /** 是否有管理员权限 */
  hasAdminPrivilege: boolean
  /** 索引统计 */
  stats?: IndexStats
  /** 当前进度消息 */
  progressMessage?: string
}

/**
 * 索引进度事件
 */
export interface IndexProgressEvent {
  /** 当前阶段 */
  stage: string
  /** 进度百分比 (0-100) */
  progress: number
  /** 消息 */
  message: string
  /** 已处理数量 */
  processed: number
  /** 总数量 */
  total: number
}

/**
 * 文件变化事件类型
 */
export type FileChangeEvent =
  | { type: 'created'; path: string }
  | { type: 'deleted'; path: string }
  | { type: 'modified'; path: string }
  | { type: 'renamed'; oldPath: string; newPath: string }

/**
 * 检查管理员权限
 * @returns 是否有管理员权限
 */
export async function checkAdminPrivilege(): Promise<boolean> {
  return await invoke('check_file_index_admin_privilege')
}

/**
 * 获取索引状态
 * @returns 索引状态信息
 */
export async function getIndexStatus(): Promise<IndexStatus> {
  return await invoke('get_file_index_status')
}

/**
 * 开始建立文件索引
 * @param drives 要索引的驱动器列表，如 ["C", "D"]，不传则索引所有驱动器
 * @returns 是否成功开始索引
 */
export async function startIndexing(drives?: string[]): Promise<boolean> {
  return await invoke('start_file_indexing', { drives })
}

/**
 * 搜索已索引的文件
 * @param keyword 搜索关键词
 * @param drive 限定搜索的驱动器（可选）
 * @param maxResults 最大结果数量（默认 100）
 * @returns 匹配的文件列表
 */
export async function searchIndexedFiles(
  keyword: string,
  drive?: string,
  maxResults?: number
): Promise<FileIndexRecord[]> {
  return await invoke('search_indexed_files', {
    keyword,
    drive,
    maxResults: maxResults || 100
  })
}

/**
 * 获取索引统计信息
 * @returns 索引统计信息
 */
export async function getIndexStats(): Promise<IndexStats> {
  return await invoke('get_file_index_stats')
}

/**
 * 清空文件索引
 * @param drive 要清空的驱动器（可选，不传则清空全部）
 * @returns 是否成功清空
 */
export async function clearIndex(drive?: string): Promise<boolean> {
  return await invoke('clear_file_index', { drive })
}

/**
 * 开始监控文件变化
 * @param drives 要监控的驱动器列表
 * @returns 是否成功开始监控
 */
export async function startFileWatching(drives?: string[]): Promise<boolean> {
  return await invoke('start_file_watching', { drives })
}

/**
 * 停止监控文件变化
 * @returns 是否成功停止
 */
export async function stopFileWatching(): Promise<boolean> {
  return await invoke('stop_file_watching')
}

/**
 * 获取所有可用驱动器
 * @returns 驱动器列表
 */
export async function getAvailableDrives(): Promise<string[]> {
  return await invoke('get_available_drives')
}

/**
 * 监听索引进度事件
 * @param callback 进度回调函数
 * @returns 取消监听函数
 */
export async function onIndexProgress(
  callback: (event: IndexProgressEvent) => void
): Promise<UnlistenFn> {
  return await listen<IndexProgressEvent>('file-index:progress', (event) => {
    callback(event.payload)
  })
}

/**
 * 监听索引完成事件
 * @param callback 完成回调函数（参数为索引的文件总数）
 * @returns 取消监听函数
 */
export async function onIndexCompleted(
  callback: (totalFiles: number) => void
): Promise<UnlistenFn> {
  return await listen<number>('file-index:completed', (event) => {
    callback(event.payload)
  })
}

/**
 * 监听索引错误事件
 * @param callback 错误回调函数
 * @returns 取消监听函数
 */
export async function onIndexError(
  callback: (error: string) => void
): Promise<UnlistenFn> {
  return await listen<string>('file-index:error', (event) => {
    callback(event.payload)
  })
}

/**
 * 监听文件变化事件
 * @param callback 变化回调函数
 * @returns 取消监听函数
 */
export async function onFileChange(
  callback: (event: FileChangeEvent) => void
): Promise<UnlistenFn> {
  return await listen<FileChangeEvent>('file-index:change', (event) => {
    callback(event.payload)
  })
}
