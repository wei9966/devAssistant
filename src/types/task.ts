export interface Task {
  id?: number;
  title: string;
  description?: string;
  category: string;
  priority: 1 | 2 | 3;  // 1-高 | 2-中 | 3-低
  status: 'todo' | 'active' | 'done' | 'deferred' | 'cancelled';
  quadrant?: TaskQuadrant;
  tags?: Tag[];
  gitBranch?: string;
  createdAt?: string;
  startedAt?: string;
  lastActiveAt?: string;
  completedAt?: string;
  dueDate?: string;      // 截止日期
  registeredAt?: string; // 登记日期（用户选择的任务日期）
  displayDate?: string;  // 日历显示日期（未完成任务可设置到任意日期显示）
  scheduledStartTime?: string; // 计划开始时间（精确到秒，用于提醒）
  estimatedHours?: number;
  actualHours?: number;
  context?: WorkContext;
  notes?: string;
  progress?: number;     // 任务进度百分比（0-100）
}

export interface TaskCategoryDefinition {
  id?: number;
  key: string;
  name: string;
  color: string;
  icon: string;
  isSystem: boolean;
  isHidden: boolean;
  sortOrder: number;
  usageCount: number;
  createdAt?: string;
  updatedAt?: string;
}

// 任务里程碑
export interface TaskMilestone {
  id?: number;
  taskId: number;
  title: string;
  description?: string;
  progressSnapshot?: number;  // 创建里程碑时的进度快照
  createdAt?: string;
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
  isFavorite?: boolean;
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
export const CATEGORY_LABELS: Record<string, string> = {
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
  cancelled: '已取消',
};

// 四象限的中文映射
export const QUADRANT_LABELS: Record<TaskQuadrant, string> = {
  urgent_important: '紧急且重要',
  urgent_not_important: '紧急不重要',
  not_urgent_important: '不紧急但重要',
  not_urgent_not_important: '不紧急不重要',
};

// 四象限配置（用于UI展示）
// 注意：使用 CSS 变量以支持主题切换
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
    color: 'var(--error)',
    bgColor: 'var(--error-bg)',
    borderColor: 'var(--error-border)',
    description: '立即处理，优先级最高',
  },
  urgent_not_important: {
    label: '紧急不重要',
    shortLabel: '紧急',
    color: 'var(--warning)',
    bgColor: 'var(--warning-bg)',
    borderColor: 'var(--warning-border)',
    description: '尽快处理或委派他人',
  },
  not_urgent_important: {
    label: '不紧急但重要',
    shortLabel: '重要',
    color: 'var(--accent-primary)',
    bgColor: 'var(--accent-glow)',
    borderColor: 'var(--accent-primary)',
    description: '计划安排，重点关注',
  },
  not_urgent_not_important: {
    label: '不紧急不重要',
    shortLabel: '一般',
    color: 'var(--text-muted)',
    bgColor: 'var(--bg-elevated)',
    borderColor: 'var(--border-default)',
    description: '低优先级，有时间再处理',
  },
};
