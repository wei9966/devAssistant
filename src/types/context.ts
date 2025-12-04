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

// 屏幕上下文采集相关类型
export interface ContextSettings {
  captureEnabled: boolean;
  captureInterval: number;
  similarityThreshold: number;
  retentionDays: number;
  excludedApps: string[];
  saveScreenshots: boolean;
}

export interface CaptureStatus {
  isRunning: boolean;
  lastCaptureAt: string | null;
  totalCapturesToday: number;
  skippedCount: number;
}

export interface ScreenContext {
  id: number;
  capturedAt: string;
  appName: string | null;
  windowTitle: string | null;
  activityType: string;
  description: string;
  keyContent: string | null;
}

export interface DayStats {
  totalCount: number;
  appDistribution: Record<string, number>;
  activityDistribution: Record<string, number>;
  timeRange: [string, string] | null;
}

// 日报上下文摘要
export interface DayContextSummary {
  // 时间统计
  activeTimeRange: {
    start: string;        // 最早活动时间
    end: string;          // 最晚活动时间
    totalMinutes: number; // 总活跃时长
  };

  // 应用使用统计
  appUsage: {
    appName: string;
    minutes: number;
    percentage: number;
  }[];

  // 活动类型分布
  activityDistribution: {
    coding: number;       // 编码时间占比
    browsing: number;     // 浏览时间占比
    document: number;     // 文档时间占比
    meeting: number;      // 会议时间占比
    communication: number;// 沟通时间占比
    other: number;        // 其他时间占比
  };

  // 关键活动摘要
  keyActivities: {
    time: string;
    description: string;
    appName: string;
  }[];
}
