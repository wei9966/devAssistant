/**
 * AI Chat Widget 组件导出
 * 用于展示 AI 对话中返回的结构化数据
 */

export { default as TaskListWidget } from './TaskListWidget.vue';
export { default as SqlListWidget } from './SqlListWidget.vue';
export { default as TimeStatsWidget } from './TimeStatsWidget.vue';
export { default as ReportWidget } from './ReportWidget.vue';

// 类型导出
export interface TasksResponse {
  total: number;
  tasks: Array<{
    id: string | number;
    title: string;
    description?: string;
    quadrant: import('@/types/task').TaskQuadrant;
    status: string;
    due_date?: string;
    completed_at?: string;
    tags?: string[];
    pomodoro_count?: number;
    focus_minutes?: number;
  }>;
}

export interface SqlSearchResponse {
  total: number;
  sql_list: Array<{
    id: string | number;
    content: string;
    sql_type: string;
    tables?: string[];
    is_favorite: boolean;
    created_at: string;
    note?: string;
  }>;
}

export interface TimeStatsResponse {
  date_range: string;
  total_minutes: number;
  active_time?: string;
  by_category?: Array<{
    category: string;
    minutes: number;
    percentage: number;
    apps?: string[];
  }>;
  by_app?: Array<{
    app: string;
    minutes: number;
    percentage: number;
  }>;
}

export interface DailyReportResponse {
  date: string;
  report: {
    summary?: string;
    work_content?: Array<{
      category: string;
      items: string[];
    }>;
    time_stats?: {
      total_hours: number;
      active_period: string;
      breakdown?: Array<{
        category: string;
        hours: number;
        percentage: number;
      }>;
    };
    highlights?: string[];
  };
  markdown?: string;
}

export interface WeeklyReportResponse {
  week: string;
  date_range: string;
  report: {
    summary?: string;
    completed_tasks?: {
      total: number;
      by_category?: Array<{
        category: string;
        count: number;
      }>;
      highlights?: string[];
    };
    time_stats?: {
      total_hours: number;
      by_category?: Array<{
        category: string;
        hours: number;
        percentage: number;
      }>;
    };
    comparison_with_last_week?: {
      tasks_diff?: string;
      hours_diff?: string;
      efficiency_change?: string;
    };
    next_week_plan?: string[];
  };
  markdown?: string;
}
