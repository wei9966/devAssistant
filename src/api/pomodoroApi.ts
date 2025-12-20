import { invoke } from '@tauri-apps/api/core'
import type {
  PomodoroSession,
  FocusApp,
  PomodoroDailyStats,
  CreatePomodoroRequest,
  CompletePomodoroRequest
} from '@/types/pomodoro'

// === 会话管理 ===

/**
 * 创建新的番茄钟会话
 */
export async function createPomodoroSession(request: CreatePomodoroRequest): Promise<PomodoroSession> {
  return invoke<PomodoroSession>('create_pomodoro_session', { request })
}

/**
 * 开始番茄钟
 */
export async function startPomodoro(sessionId: number): Promise<PomodoroSession> {
  return invoke<PomodoroSession>('start_pomodoro', { sessionId })
}

/**
 * 暂停番茄钟
 */
export async function pausePomodoro(sessionId: number): Promise<PomodoroSession> {
  return invoke<PomodoroSession>('pause_pomodoro', { sessionId })
}

/**
 * 恢复番茄钟
 */
export async function resumePomodoro(sessionId: number): Promise<PomodoroSession> {
  return invoke<PomodoroSession>('resume_pomodoro', { sessionId })
}

/**
 * 取消番茄钟
 */
export async function cancelPomodoro(sessionId: number): Promise<PomodoroSession> {
  return invoke<PomodoroSession>('cancel_pomodoro', { sessionId })
}

/**
 * 完成番茄钟
 */
export async function completePomodoro(
  sessionId: number,
  request: CompletePomodoroRequest
): Promise<PomodoroSession> {
  return invoke<PomodoroSession>('complete_pomodoro', { sessionId, request })
}

/**
 * 记录分心
 */
export async function recordDistraction(sessionId: number): Promise<PomodoroSession> {
  return invoke<PomodoroSession>('record_pomodoro_distraction', { sessionId })
}

/**
 * 更新专注时间
 */
export async function updateFocusTime(sessionId: number, seconds: number): Promise<void> {
  return invoke<void>('update_pomodoro_focus_time', { sessionId, seconds })
}

/**
 * 获取会话详情
 */
export async function getPomodoroSession(sessionId: number): Promise<PomodoroSession | null> {
  return invoke<PomodoroSession | null>('get_pomodoro_session', { sessionId })
}

/**
 * 获取当前活跃会话
 */
export async function getActiveSession(): Promise<PomodoroSession | null> {
  return invoke<PomodoroSession | null>('get_active_pomodoro_session')
}

/**
 * 获取今日会话列表
 */
export async function getTodaySessions(): Promise<PomodoroSession[]> {
  return invoke<PomodoroSession[]>('get_today_pomodoro_sessions')
}

/**
 * 获取指定任务的会话历史
 */
export async function getTaskSessions(taskId: number): Promise<PomodoroSession[]> {
  return invoke<PomodoroSession[]>('get_task_pomodoro_sessions', { taskId })
}

// === 白名单应用管理 ===

/**
 * 获取所有白名单应用
 */
export async function getFocusApps(): Promise<FocusApp[]> {
  return invoke<FocusApp[]>('get_pomodoro_focus_apps')
}

/**
 * 添加白名单应用
 */
export async function addFocusApp(name: string, processName?: string): Promise<FocusApp> {
  return invoke<FocusApp>('add_pomodoro_focus_app', { name, processName })
}

/**
 * 删除白名单应用
 */
export async function removeFocusApp(appId: number): Promise<void> {
  return invoke<void>('remove_pomodoro_focus_app', { appId })
}

// === 统计数据 ===

/**
 * 获取今日统计
 */
export async function getTodayStats(): Promise<PomodoroDailyStats> {
  return invoke<PomodoroDailyStats>('get_pomodoro_today_stats')
}

/**
 * 获取指定日期统计
 */
export async function getStatsByDate(date: string): Promise<PomodoroDailyStats | null> {
  return invoke<PomodoroDailyStats | null>('get_pomodoro_stats_by_date', { date })
}

/**
 * 获取日期范围统计
 */
export async function getStatsRange(
  startDate: string,
  endDate: string
): Promise<PomodoroDailyStats[]> {
  return invoke<PomodoroDailyStats[]>('get_pomodoro_stats_range', { startDate, endDate })
}

// === AI 功能 ===

/**
 * AI 任务拆解建议 - 在开始专注前调用
 * 帮助将任务拆解为25分钟可完成的小目标
 * @param taskId 任务ID，用于获取里程碑信息
 * @param taskTitle 任务标题
 * @param taskDescription 任务描述
 * @param currentProgress 当前进度百分比
 */
export async function aiTaskBreakdown(
  taskId: number | undefined,
  taskTitle: string,
  taskDescription?: string,
  currentProgress?: number
): Promise<string> {
  return invoke<string>('pomodoro_ai_task_breakdown', {
    taskId,
    taskTitle,
    taskDescription,
    currentProgress
  })
}

/**
 * AI 专注力分析 - 在完成番茄钟后调用
 * 分析专注表现并提供改进建议
 */
export async function aiFocusAnalysis(
  focusGoal: string,
  durationMinutes: number,
  actualFocusSeconds: number,
  distractionCount: number,
  userFeedback?: string
): Promise<string> {
  return invoke<string>('pomodoro_ai_focus_analysis', {
    focusGoal,
    durationMinutes,
    actualFocusSeconds,
    distractionCount,
    userFeedback
  })
}

/**
 * AI 每日复盘 - 在查看统计时调用
 * 生成每日总结和明日建议
 */
export async function aiDailyReview(date: string): Promise<string> {
  return invoke<string>('pomodoro_ai_daily_review', { date })
}

/**
 * AI 进度评估 - 在提交反馈后调用
 * 评估任务完成进度并给出建议
 */
export async function aiProgressEval(
  taskTitle: string,
  focusGoal: string,
  userFeedback: string,
  previousProgress?: number
): Promise<string> {
  return invoke<string>('pomodoro_ai_progress_eval', {
    taskTitle,
    focusGoal,
    userFeedback,
    previousProgress
  })
}

// === 任务中断与恢复 ===

/**
 * AI 中断活动分析 - 分析中断期间的屏幕活动
 * 判断中断期间的活动与原任务的关联性
 * @param originalTask 原任务名称
 * @param focusGoal 专注目标
 * @param interruptionDurationMinutes 中断时长（分钟）
 * @param activitySummaries 中断期间的活动摘要（从截图回顾获取）
 */
export async function aiAnalyzeInterruption(
  originalTask: string,
  focusGoal: string,
  interruptionDurationMinutes: number,
  activitySummaries: string
): Promise<string> {
  return invoke<string>('pomodoro_ai_analyze_interruption', {
    originalTask,
    focusGoal,
    interruptionDurationMinutes,
    activitySummaries
  })
}

/**
 * AI 任务恢复建议 - 基于中断分析生成恢复建议
 * 帮助用户快速回到工作状态
 * @param originalTask 原任务名称
 * @param focusGoal 专注目标
 * @param progressBeforeInterruption 中断前的进度（可选）
 * @param interruptionAnalysis 中断分析结果（来自 aiAnalyzeInterruption）
 * @param elapsedFocusSeconds 已专注时间（秒）
 * @param remainingSeconds 剩余时间（秒）
 */
export async function aiResumeSuggestion(
  originalTask: string,
  focusGoal: string,
  progressBeforeInterruption: number | undefined,
  interruptionAnalysis: string,
  elapsedFocusSeconds: number,
  remainingSeconds: number
): Promise<string> {
  return invoke<string>('pomodoro_ai_resume_suggestion', {
    originalTask,
    focusGoal,
    progressBeforeInterruption,
    interruptionAnalysis,
    elapsedFocusSeconds,
    remainingSeconds
  })
}

/**
 * AI 快速恢复提示 - 轻量级恢复提示
 * 无需完整分析，快速给出一句恢复提示
 * @param focusGoal 专注目标
 * @param lastActivity 最后一次活动描述
 */
export async function aiQuickResume(
  focusGoal: string,
  lastActivity: string
): Promise<string> {
  return invoke<string>('pomodoro_ai_quick_resume', {
    focusGoal,
    lastActivity
  })
}

// === 活动窗口检测 ===

/**
 * 活动窗口信息
 */
export interface ActiveWindowInfo {
  appName: string | null
  windowTitle: string | null
  processName: string | null
}

/**
 * 获取当前活动窗口信息
 * 用于番茄钟专注模式下检测用户当前使用的应用
 */
export async function getActiveWindowInfo(): Promise<ActiveWindowInfo> {
  return invoke<ActiveWindowInfo>('get_active_window_info')
}

/**
 * 运行中的应用信息
 */
export interface RunningApp {
  name: string
  processName: string
}

/**
 * 获取当前运行的应用列表
 * 用于番茄钟白名单选择
 */
export async function getRunningApps(): Promise<RunningApp[]> {
  return invoke<RunningApp[]>('get_running_apps')
}

/**
 * AI 会话分析 - 分析指定番茄钟会话
 * 分析会话的任务相关性、效率评估、改进建议等
 * @param sessionId 番茄钟会话ID
 */
export async function aiAnalyzeSession(sessionId: number): Promise<string> {
  return invoke<string>('pomodoro_ai_analyze_session', { sessionId })
}

/**
 * 获取番茄钟期间的应用使用统计
 * @param sessionId 番茄钟会话ID
 * @returns JSON格式的应用使用统计
 */
export async function getSessionAppUsage(sessionId: number): Promise<string> {
  return invoke<string>('pomodoro_get_session_app_usage', { sessionId })
}
