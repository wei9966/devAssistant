import type { Task } from './task';

// ========== 消息相关类型 ==========

/**
 * 消息角色
 */
export type MessageRole = 'user' | 'assistant' | 'system';

/**
 * 消息类型
 */
export type MessageType = 'text' | 'tasks' | 'sql' | 'stats' | 'report' | 'pomodoro';

/**
 * 对话消息
 */
export interface ChatMessage {
  id: string;
  role: MessageRole;
  content: string;
  timestamp: Date;
  type?: MessageType;
  data?: any;  // 结构化数据
  loading?: boolean;
}

// ========== 请求响应类型 ==========

/**
 * 聊天请求
 */
export interface ChatRequest {
  messages: {
    role: MessageRole;
    content: string;
  }[];
}

/**
 * 聊天响应
 */
export interface ChatResponse {
  content: string;
  type?: MessageType;
  data?: any;
}

// ========== Function Call 相关类型 ==========

/**
 * Function 调用
 */
export interface FunctionCall {
  name: string;
  arguments: string;  // JSON 字符串
}

// ========== 业务数据类型 ==========

/**
 * 任务查询响应
 */
export interface TasksResponse {
  total: number;
  tasks: TaskInfo[];
}

/**
 * 任务信息（AI 返回的简化版本）
 */
export interface TaskInfo {
  id: string;
  title: string;
  description?: string;
  quadrant?: 1 | 2 | 3 | 4;
  status: 'todo' | 'active' | 'done' | 'deferred';
  dueDate?: string;
  completedAt?: string;
  tags?: string[];
  pomodoroCount?: number;
  focusMinutes?: number;
}

/**
 * SQL 搜索响应
 */
export interface SqlSearchResponse {
  total: number;
  sqlList: SqlInfo[];
}

/**
 * SQL 信息
 */
export interface SqlInfo {
  id: string;
  content: string;
  sqlType: 'SELECT' | 'UPDATE' | 'INSERT' | 'DELETE' | 'CREATE' | 'ALTER';
  tables: string[];
  isFavorite: boolean;
  createdAt: string;
  note?: string;
}

/**
 * 时间统计响应
 */
export interface TimeStatsResponse {
  dateRange: string;
  totalMinutes: number;
  activeTime?: string;
  byCategory: CategoryStats[];
  byApp?: AppStats[];
}

/**
 * 分类统计
 */
export interface CategoryStats {
  category: string;
  minutes: number;
  percentage: number;
  apps?: string[];
}

/**
 * 应用统计
 */
export interface AppStats {
  app: string;
  minutes: number;
  percentage: number;
}

/**
 * 番茄钟记录响应
 */
export interface PomodoroResponse {
  totalCount: number;
  totalMinutes: number;
  records: PomodoroRecord[];
  byTask?: TaskPomodoroStats[];
}

/**
 * 番茄钟记录
 */
export interface PomodoroRecord {
  id: string;
  startTime: string;
  endTime?: string;
  durationMinutes: number;
  taskId?: string;
  taskTitle?: string;
  completed: boolean;
}

/**
 * 任务番茄钟统计
 */
export interface TaskPomodoroStats {
  taskId: string;
  taskTitle: string;
  count: number;
  minutes: number;
}

/**
 * 番茄钟开始响应
 */
export interface StartPomodoroResponse {
  success: boolean;
  pomodoro: {
    id: string;
    durationMinutes: number;
    taskId?: string;
    taskTitle?: string;
    startTime: string;
    expectedEndTime: string;
  };
  message: string;
}

/**
 * 任务创建响应
 */
export interface CreateTaskResponse {
  success: boolean;
  task: {
    id: string;
    title: string;
    quadrant?: number;
    dueDate?: string;
    createdAt: string;
  };
  message: string;
}

/**
 * 任务更新响应
 */
export interface UpdateTaskResponse {
  success: boolean;
  message: string;
}

/**
 * 日报响应
 */
export interface DailyReportResponse {
  date: string;
  report: {
    summary: string;
    workContent: WorkContent[];
    timeStats: ReportTimeStats;
    highlights: string[];
  };
  markdown: string;
}

/**
 * 工作内容
 */
export interface WorkContent {
  category: string;
  items: string[];
}

/**
 * 报告时间统计
 */
export interface ReportTimeStats {
  totalHours: number;
  activePeriod: string;
  breakdown: {
    category: string;
    hours: number;
    percentage: number;
  }[];
}

/**
 * 周报响应
 */
export interface WeeklyReportResponse {
  week: string;
  dateRange: string;
  report: {
    summary: string;
    completedTasks: {
      total: number;
      byCategory: {
        category: string;
        count: number;
      }[];
      highlights: string[];
    };
    timeStats: {
      totalHours: number;
      byCategory: {
        category: string;
        hours: number;
        percentage: number;
      }[];
    };
    comparisonWithLastWeek?: {
      tasksDiff: string;
      hoursDiff: string;
      efficiencyChange: string;
    };
    nextWeekPlan?: string[];
  };
  markdown: string;
}

/**
 * 效率分析响应
 */
export interface EfficiencyAnalysis {
  dateRange: string;
  analysis: {
    overview: {
      totalWorkHours: number;
      totalTasksCompleted: number;
      totalPomodoro: number;
      avgDailyFocusHours: number;
    };
    timeAnalysis: {
      mostProductiveDay: string;
      mostProductiveHour: string;
      codingPercentage: number;
      meetingPercentage: number;
      fragmentationScore: number;
      insight: string;
    };
    taskAnalysis: {
      completionRate: number;
      quadrantDistribution: {
        q1: number;
        q2: number;
        q3: number;
        q4: number;
      };
      avgTaskDuration: number;
      insight: string;
    };
    focusAnalysis: {
      avgPomodoroPerDay: number;
      completionRate: number;
      bestFocusDay: string;
      insight: string;
    };
    suggestions: string[];
  };
}

// ========== UI 相关类型 ==========

/**
 * 快捷提问配置
 */
export interface QuickQuestion {
  icon: string;
  label: string;
  question: string;
}

/**
 * 对话会话
 */
export interface ChatSession {
  id: string;
  title: string;
  lastMessage?: string;
  createdAt: string;
  updatedAt: string;
  messageCount: number;
}

/**
 * 聊天状态
 */
export interface ChatState {
  messages: ChatMessage[];
  isLoading: boolean;
  currentAction: string | null;
  error: string | null;
}

// ========== 常量定义 ==========

/**
 * 默认快捷提问列表
 */
export const DEFAULT_QUICK_QUESTIONS: QuickQuestion[] = [
  {
    icon: '📋',
    label: '今天的任务',
    question: '今天我有哪些待办任务?'
  },
  {
    icon: '✅',
    label: '本周完成',
    question: '这周我完成了多少任务?'
  },
  {
    icon: '⏱️',
    label: '时间统计',
    question: '今天的时间都花在哪了?'
  },
  {
    icon: '📊',
    label: '效率分析',
    question: '分析一下我本周的工作效率'
  },
  {
    icon: '📝',
    label: '生成日报',
    question: '帮我生成今天的日报'
  }
];

/**
 * 消息类型中文名称
 */
export const MESSAGE_TYPE_LABELS: Record<MessageType, string> = {
  text: '文本',
  tasks: '任务',
  sql: 'SQL',
  stats: '统计',
  report: '报告',
  pomodoro: '番茄钟'
};
