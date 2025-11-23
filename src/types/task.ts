export interface Task {
  id?: number;
  title: string;
  description?: string;
  category: 'dev' | 'ops' | 'study' | 'other';
  priority: 1 | 2 | 3;  // 1-高 | 2-中 | 3-低
  status: 'todo' | 'active' | 'done' | 'deferred';
  gitBranch?: string;
  createdAt?: string;
  startedAt?: string;
  lastActiveAt?: string;
  completedAt?: string;
  estimatedHours?: number;
  actualHours?: number;
  context?: WorkContext;
  notes?: string;
}

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

// 任务分类的中文映射
export const CATEGORY_LABELS: Record<Task['category'], string> = {
  dev: '开发',
  ops: '运维',
  study: '学习',
  other: '其他',
};

// 优先级的中文映射
export const PRIORITY_LABELS: Record<Task['priority'], string> = {
  1: '高',
  2: '中',
  3: '低',
};

// 状态的中文映射
export const STATUS_LABELS: Record<Task['status'], string> = {
  todo: '待办',
  active: '进行中',
  done: '已完成',
  deferred: '延后',
};
