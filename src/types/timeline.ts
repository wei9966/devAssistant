// 时间线项类型
export interface TimelineItem {
  id: string;
  title: string;
  summary: string;
  keywords: string[];
  importance: number;
  startTime: string;
  endTime?: string;
  appName?: string;
  activityType: 'coding' | 'browsing' | 'chatting' | 'document' | 'design' | 'other';
  thumbnailPath?: string;  // 缩略图路径
  merged?: boolean;  // 是否是合并后的项
  mergedCount?: number;  // 合并了多少个原始项
}

// 时间线分组
export interface TimelineGroup {
  date: string;  // YYYY-MM-DD
  displayDate: string;  // 显示用的日期文本
  items: TimelineItem[];
  summary?: string;  // 这一天的总结
}

// 时间线过滤选项
export interface TimelineFilter {
  dateRange?: [string, string];
  activityTypes?: string[];
  keywords?: string[];
  minImportance?: number;
}
