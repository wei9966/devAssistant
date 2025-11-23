// TypeScript 类型定义

// 将在后续任务中添加具体的类型定义

export interface Task {
  id: number
  title: string
  description?: string
  status: 'pending' | 'in_progress' | 'completed'
  created_at: string
  updated_at: string
}

export interface SqlRecord {
  id: number
  sql: string
  database: string
  executed_at: string
  execution_time?: number
}

export interface WorkContext {
  id: number
  date: string
  tasks: string[]
  notes: string
}
