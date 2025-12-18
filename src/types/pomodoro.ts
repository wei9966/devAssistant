import type { Task } from './task'

// 番茄钟会话状态
export type PomodoroStatus = 'pending' | 'focusing' | 'paused' | 'completed' | 'cancelled'

// 番茄钟阶段（UI场景）
export type PomodoroPhase = 'prep' | 'focusing' | 'distracted' | 'report' | 'stats'

// 番茄钟会话
export interface PomodoroSession {
  id?: number
  taskId?: number
  task?: Task  // 关联的任务详情
  durationMinutes: number
  status: PomodoroStatus
  phase: PomodoroPhase
  focusGoal?: string
  aiSuggestion?: string
  actualFocusSeconds: number
  distractionCount: number
  focusRate: number
  feedback?: string
  progressUpdate?: string
  startedAt?: string
  pausedAt?: string
  completedAt?: string
  createdAt?: string
}

// 专注应用（白名单）
export interface FocusApp {
  id?: number
  name: string
  processName?: string
  isDefault: boolean
  createdAt?: string
}

// 每日统计
export interface PomodoroDailyStats {
  id?: number
  date: string
  totalSessions: number
  completedSessions: number
  totalFocusMinutes: number
  avgFocusRate: number
  totalDistractions: number
  appUsage?: AppUsageStats[]
  aiInsight?: string
  createdAt?: string
  updatedAt?: string
}

// 应用使用统计
export interface AppUsageStats {
  appName: string
  minutes: number
  percentage: number
}

// 创建番茄钟请求
export interface CreatePomodoroRequest {
  taskId?: number
  durationMinutes?: number
  focusGoal?: string
  focusApps?: string[]
}

// 完成番茄钟请求
export interface CompletePomodoroRequest {
  feedback?: string
  progressUpdate?: string
}

// AI任务拆解建议
export interface AiFocusSuggestion {
  suggestedGoal: string
  subTasks: string[]
  estimatedPomodoros: number
  tips?: string
}

// AI专注力分析结果
export interface AiFocusAnalysis {
  focusRate: number
  productivityScore: number
  summary: string
  suggestions: string[]
  nextSessionTip?: string
}

// 状态的中文映射
export const POMODORO_STATUS_LABELS: Record<PomodoroStatus, string> = {
  pending: '待开始',
  focusing: '专注中',
  paused: '已暂停',
  completed: '已完成',
  cancelled: '已取消'
}

// 阶段的中文映射
export const POMODORO_PHASE_LABELS: Record<PomodoroPhase, string> = {
  prep: '准备阶段',
  focusing: '专注中',
  distracted: '分心提醒',
  report: '结束报告',
  stats: '统计看板'
}

// 阶段配置（用于UI展示）
export const POMODORO_PHASE_CONFIG: Record<PomodoroPhase, {
  label: string
  color: string
  bgColor: string
  icon: string
  description: string
}> = {
  prep: {
    label: '准备阶段',
    color: '#6366f1',
    bgColor: 'rgba(99, 102, 241, 0.1)',
    icon: 'brain',
    description: 'AI分析任务，制定专注目标'
  },
  focusing: {
    label: '专注中',
    color: '#10b981',
    bgColor: 'rgba(16, 185, 129, 0.1)',
    icon: 'timer',
    description: '保持专注，AI守护你的注意力'
  },
  distracted: {
    label: '分心提醒',
    color: '#f43f5e',
    bgColor: 'rgba(244, 63, 94, 0.1)',
    icon: 'alert',
    description: '检测到注意力偏离，请重新聚焦'
  },
  report: {
    label: '结束报告',
    color: '#8b5cf6',
    bgColor: 'rgba(139, 92, 246, 0.1)',
    icon: 'clipboard',
    description: '记录本次专注的成果'
  },
  stats: {
    label: '统计看板',
    color: '#f59e0b',
    bgColor: 'rgba(245, 158, 11, 0.1)',
    icon: 'chart',
    description: '查看专注数据与AI洞察'
  }
}

// 默认专注时长选项（分钟）
export const DEFAULT_DURATION_OPTIONS = [15, 25, 45, 60]

// 默认专注应用
export const DEFAULT_FOCUS_APPS: FocusApp[] = [
  { name: 'VS Code', processName: 'Code.exe', isDefault: true },
  { name: 'Figma', processName: 'Figma.exe', isDefault: true },
  { name: 'Chrome', processName: 'chrome.exe', isDefault: false },
  { name: 'Notion', processName: 'Notion.exe', isDefault: false }
]
