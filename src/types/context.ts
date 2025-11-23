// 工作上下文类型定义
export interface WorkContext {
  files: FileContext[];
  lastSql?: string;
  browserTabs: string[];
  notes?: string;
}

export interface FileContext {
  path: string;
  line: number;
  column: number;
}

// Git 相关类型
export interface GitBranch {
  name: string;
  isCurrent: boolean;
}

export interface GitCommit {
  hash: string;
  message: string;
  author: string;
  date: string;
}

// 工作日志类型
export interface WorkLog {
  id?: number;
  taskId?: number;
  content: string;
  logType: 'note' | 'commit' | 'sql' | 'file';
  createdAt?: string;
  metadata?: Record<string, any>;
}
