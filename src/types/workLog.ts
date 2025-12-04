export interface WorkLog {
  id?: number;
  date: string;
  logType: string;
  content: string;
  aiGenerated: boolean;
  createdAt?: string;
  updatedAt?: string;
}

// 周计划接口
export interface WeeklyPlan {
  id?: number;
  weekKey: string;              // 周标识，如 "2025-W48"（表示这是哪一周的计划）
  content: string;              // 润色后的计划内容
  taskIds: number[];            // 当前关联的任务ID列表
  originalTaskIds: number[];    // 初次创建时的任务ID（用于对比新增项）
  status: WeeklyPlanStatus;     // 状态
  createdAt?: string;
  updatedAt?: string;
}

export type WeeklyPlanStatus = 'draft' | 'confirmed';

// 周计划状态标签
export const WEEKLY_PLAN_STATUS_LABELS: Record<WeeklyPlanStatus, string> = {
  draft: '草稿',
  confirmed: '已确认',
};

// 上下文摘要相关类型
export interface ActiveTimeRange {
  start: string;
  end: string;
  totalMinutes: number;
}

export interface AppUsage {
  appName: string;
  minutes: number;
  percentage: number;
}

export interface ActivityDistribution {
  coding: number;
  browsing: number;
  document: number;
  meeting: number;
  communication: number;
  other: number;
}

export interface KeyActivity {
  time: string;
  description: string;
  appName: string;
}

export interface DayContextSummary {
  activeTimeRange: ActiveTimeRange;
  appUsage: AppUsage[];
  activityDistribution: ActivityDistribution;
  keyActivities: KeyActivity[];
}

export interface DailyActiveHours {
  date: string;
  hours: number;
}

export interface WeeklyAppRanking {
  appName: string;
  totalMinutes: number;
  trend: 'up' | 'down' | 'stable';
}

export interface WeeklyActivitySummary {
  type: string;
  totalMinutes: number;
  dailyAverage: number;
}

export interface WorkPatterns {
  mostProductiveHour: number;
  averageStartTime: string;
  averageEndTime: string;
}

export interface WeekContextSummary {
  dailyActiveHours: DailyActiveHours[];
  weeklyAppRanking: WeeklyAppRanking[];
  weeklyActivitySummary: WeeklyActivitySummary[];
  workPatterns: WorkPatterns;
}
