export interface Task {
  id?: number;
  title: string;
  description?: string;
  category: 'backend' | 'database' | 'feature' | 'docs' | 'other';
  priority: 1 | 2 | 3;  // 1-高 | 2-中 | 3-低
  status: 'todo' | 'active' | 'done' | 'deferred';
  quadrant?: TaskQuadrant;
  tags?: Tag[];
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

export type TaskQuadrant =
  | 'urgent_important'           // 紧急且重要
  | 'urgent_not_important'       // 紧急不重要
  | 'not_urgent_important'       // 不紧急但重要
  | 'not_urgent_not_important';  // 不紧急不重要

export interface Tag {
  id?: number;
  name: string;
  color: string;
  createdAt?: string;
  updatedAt?: string;
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
  backend: '后端开发',
  database: '数据库',
  feature: '功能开发',
  docs: '文档',
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

// 四象限的中文映射
export const QUADRANT_LABELS: Record<TaskQuadrant, string> = {
  urgent_important: '紧急且重要',
  urgent_not_important: '紧急不重要',
  not_urgent_important: '不紧急但重要',
  not_urgent_not_important: '不紧急不重要',
};

// 四象限配置（用于UI展示）
export const QUADRANT_CONFIG: Record<TaskQuadrant, {
  label: string;
  shortLabel: string;
  color: string;
  bgColor: string;
  borderColor: string;
  description: string;
}> = {
  urgent_important: {
    label: '紧急且重要',
    shortLabel: '紧急重要',
    color: '#f43f5e',
    bgColor: 'rgba(244, 63, 94, 0.15)',
    borderColor: 'rgba(244, 63, 94, 0.3)',
    description: '立即处理，优先级最高',
  },
  urgent_not_important: {
    label: '紧急不重要',
    shortLabel: '紧急',
    color: '#f59e0b',
    bgColor: 'rgba(245, 158, 11, 0.15)',
    borderColor: 'rgba(245, 158, 11, 0.3)',
    description: '尽快处理或委派他人',
  },
  not_urgent_important: {
    label: '不紧急但重要',
    shortLabel: '重要',
    color: '#6366f1',
    bgColor: 'rgba(99, 102, 241, 0.15)',
    borderColor: 'rgba(99, 102, 241, 0.3)',
    description: '计划安排，重点关注',
  },
  not_urgent_not_important: {
    label: '不紧急不重要',
    shortLabel: '一般',
    color: '#64748b',
    bgColor: 'rgba(100, 116, 139, 0.15)',
    borderColor: 'rgba(100, 116, 139, 0.3)',
    description: '低优先级，有时间再处理',
  },
};
