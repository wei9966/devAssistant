// 剪切板内容类型
export type ClipboardContentType = 'text' | 'image' | 'file'

// 剪切板历史记录
export interface ClipboardHistory {
  id: number
  content_type: ClipboardContentType
  content: string
  preview: string | null
  image_path: string | null
  source_app: string | null
  is_pinned: boolean
  created_at: string
}

// 剪切板配置
export interface ClipboardConfig {
  image_save_dir: string
  image_archive_dir: string
  max_history: number
  auto_copy_image_path: boolean
  enabled: boolean
  shortcut: string
}

// 剪切板查询参数
export interface ClipboardQueryParams {
  limit?: number
  offset?: number
  content_type?: ClipboardContentType
  keyword?: string
  pinned_only?: boolean
}

// 历史记录分组（按日期）
export interface ClipboardHistoryGroup {
  date: string
  label: string
  items: ClipboardHistory[]
}
