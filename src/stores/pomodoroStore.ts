import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import {
  createPomodoroSession,
  startPomodoro,
  pausePomodoro,
  resumePomodoro,
  cancelPomodoro,
  completePomodoro,
  recordDistraction,
  updateFocusTime,
  getActiveSession,
  getTodaySessions,
  getTodayStats,
  getFocusApps,
  addFocusApp,
  removeFocusApp,
  aiAnalyzeInterruption,
  aiResumeSuggestion,
  aiQuickResume,
} from '@/api/pomodoroApi';
import type {
  PomodoroSession,
  FocusApp,
  PomodoroDailyStats,
  CreatePomodoroRequest,
  CompletePomodoroRequest,
} from '@/types/pomodoro';

// AI 中断分析结果类型
export interface InterruptionAnalysis {
  interruptionType: string;
  relevance: string;
  contextSwitch: boolean;
  mainActivities: string[];
  summary: string;
}

// AI 恢复建议结果类型
export interface ResumeSuggestion {
  canContinue: boolean;
  contextReminder: string;
  nextAction: string;
  adjustedGoal?: string;
  estimatedTimeToRefocus: number;
  tips: string[];
}

export const usePomodoroStore = defineStore('pomodoro', () => {
  // 状态
  const currentSession = ref<PomodoroSession | null>(null);
  const todaySessions = ref<PomodoroSession[]>([]);
  const todayStats = ref<PomodoroDailyStats | null>(null);
  const focusApps = ref<FocusApp[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // 暂停恢复相关状态
  const pauseStartTime = ref<Date | null>(null);
  const resumeLoading = ref(false);
  const interruptionAnalysis = ref<InterruptionAnalysis | null>(null);
  const resumeSuggestion = ref<ResumeSuggestion | null>(null);
  const quickResumeTip = ref<string | null>(null);

  // 定时器
  let timer: number | null = null;

  // 计算属性
  const isActive = computed(() => {
    return currentSession.value !== null &&
           currentSession.value.status === 'focusing';
  });

  const focusProgress = computed(() => {
    if (!currentSession.value) return 0;
    const totalSeconds = currentSession.value.durationMinutes * 60;
    const focusedSeconds = currentSession.value.actualFocusSeconds;
    return Math.min(100, (focusedSeconds / totalSeconds) * 100);
  });

  const remainingTime = computed(() => {
    if (!currentSession.value) return 0;
    const totalSeconds = currentSession.value.durationMinutes * 60;
    const focusedSeconds = currentSession.value.actualFocusSeconds;
    return Math.max(0, totalSeconds - focusedSeconds);
  });

  // 定时器管理
  function startTimer() {
    if (timer !== null || !currentSession.value) return;

    timer = window.setInterval(async () => {
      if (!currentSession.value || currentSession.value.status !== 'focusing') {
        stopTimer();
        return;
      }

      // 更新本地状态
      currentSession.value.actualFocusSeconds++;

      // 每10秒同步到后端
      if (currentSession.value.actualFocusSeconds % 10 === 0) {
        try {
          await updateFocusTime(
            currentSession.value.id!,
            currentSession.value.actualFocusSeconds
          );
        } catch (e) {
          console.error('同步专注时间失败:', e);
        }
      }

      // 检查是否到时间
      const totalSeconds = currentSession.value.durationMinutes * 60;
      if (currentSession.value.actualFocusSeconds >= totalSeconds) {
        stopTimer();
        // 可以在这里触发通知或自动完成
      }
    }, 1000);
  }

  function stopTimer() {
    if (timer !== null) {
      window.clearInterval(timer);
      timer = null;
    }
  }

  // Actions
  async function loadActiveSession() {
    loading.value = true;
    error.value = null;
    try {
      currentSession.value = await getActiveSession();

      // 如果有活跃会话且状态为focusing，启动定时器
      if (currentSession.value && currentSession.value.status === 'focusing') {
        startTimer();
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载活跃会话失败';
      console.error('加载活跃会话失败:', e);
    } finally {
      loading.value = false;
    }
  }

  async function loadTodaySessions() {
    try {
      todaySessions.value = await getTodaySessions();
    } catch (e) {
      console.error('加载今日会话失败:', e);
    }
  }

  async function loadTodayStats() {
    try {
      todayStats.value = await getTodayStats();
    } catch (e) {
      console.error('加载今日统计失败:', e);
    }
  }

  async function loadFocusApps() {
    try {
      focusApps.value = await getFocusApps();
    } catch (e) {
      console.error('加载白名单应用失败:', e);
    }
  }

  async function createSession(request: CreatePomodoroRequest) {
    loading.value = true;
    error.value = null;
    try {
      const session = await createPomodoroSession(request);
      currentSession.value = session;
      await loadTodaySessions();
      return session;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '创建会话失败';
      console.error('创建会话失败:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function startSession(sessionId: number) {
    try {
      const session = await startPomodoro(sessionId);
      currentSession.value = session;
      await loadTodaySessions();

      // 启动定时器
      startTimer();
    } catch (e) {
      console.error('开始会话失败:', e);
      throw e;
    }
  }

  async function pauseSession(sessionId: number) {
    try {
      // 停止定时器
      stopTimer();

      // 记录暂停开始时间
      pauseStartTime.value = new Date();

      const session = await pausePomodoro(sessionId);
      currentSession.value = session;
      await loadTodaySessions();
    } catch (e) {
      console.error('暂停会话失败:', e);
      throw e;
    }
  }

  async function resumeSession(sessionId: number) {
    try {
      const session = await resumePomodoro(sessionId);
      currentSession.value = session;
      await loadTodaySessions();

      // 清除暂停状态
      pauseStartTime.value = null;
      interruptionAnalysis.value = null;
      resumeSuggestion.value = null;
      quickResumeTip.value = null;

      // 重新启动定时器
      startTimer();
    } catch (e) {
      console.error('恢复会话失败:', e);
      throw e;
    }
  }

  // 计算中断时长（分钟）
  function getInterruptionDuration(): number {
    if (!pauseStartTime.value) return 0;
    const now = new Date();
    const durationMs = now.getTime() - pauseStartTime.value.getTime();
    return Math.round(durationMs / 60000);
  }

  // 获取快速恢复提示
  async function fetchQuickResumeTip(): Promise<string> {
    if (!currentSession.value) {
      throw new Error('没有活跃会话');
    }

    resumeLoading.value = true;
    try {
      const focusGoal = currentSession.value.focusGoal || '完成任务';
      const lastActivity = '刚才的工作'; // 简单描述

      const tip = await aiQuickResume(focusGoal, lastActivity);
      quickResumeTip.value = tip;
      return tip;
    } finally {
      resumeLoading.value = false;
    }
  }

  // 获取完整的中断分析和恢复建议
  async function fetchResumeSuggestions(activitySummaries: string): Promise<void> {
    if (!currentSession.value) {
      throw new Error('没有活跃会话');
    }

    resumeLoading.value = true;
    try {
      const taskTitle = currentSession.value.task?.title || '当前任务';
      const focusGoal = currentSession.value.focusGoal || '完成任务';
      const interruptionMinutes = getInterruptionDuration();
      const elapsedSeconds = currentSession.value.actualFocusSeconds;
      const totalSeconds = currentSession.value.durationMinutes * 60;
      const remainingSeconds = Math.max(0, totalSeconds - elapsedSeconds);

      // 1. 获取中断分析
      const analysisResult = await aiAnalyzeInterruption(
        taskTitle,
        focusGoal,
        interruptionMinutes,
        activitySummaries
      );

      // 尝试解析 JSON
      try {
        interruptionAnalysis.value = JSON.parse(analysisResult);
      } catch {
        // 如果不是 JSON，创建简单对象
        interruptionAnalysis.value = {
          interruptionType: '未知',
          relevance: '未知',
          contextSwitch: false,
          mainActivities: [],
          summary: analysisResult
        };
      }

      // 2. 获取恢复建议
      const suggestionResult = await aiResumeSuggestion(
        taskTitle,
        focusGoal,
        undefined, // progressBeforeInterruption
        analysisResult,
        elapsedSeconds,
        remainingSeconds
      );

      // 尝试解析 JSON
      try {
        resumeSuggestion.value = JSON.parse(suggestionResult);
      } catch {
        // 如果不是 JSON，创建简单对象
        resumeSuggestion.value = {
          canContinue: true,
          contextReminder: suggestionResult,
          nextAction: '继续之前的工作',
          estimatedTimeToRefocus: 5,
          tips: []
        };
      }
    } finally {
      resumeLoading.value = false;
    }
  }

  // 清除恢复状态
  function clearResumeState() {
    interruptionAnalysis.value = null;
    resumeSuggestion.value = null;
    quickResumeTip.value = null;
  }

  async function cancelSession(sessionId: number) {
    try {
      // 停止定时器
      stopTimer();

      const session = await cancelPomodoro(sessionId);
      currentSession.value = null;
      await loadTodaySessions();
      await loadTodayStats();
      return session;
    } catch (e) {
      console.error('取消会话失败:', e);
      throw e;
    }
  }

  async function completeSession(
    sessionId: number,
    request: CompletePomodoroRequest
  ) {
    try {
      // 停止定时器
      stopTimer();

      const session = await completePomodoro(sessionId, request);
      currentSession.value = null;
      await loadTodaySessions();
      await loadTodayStats();
      return session;
    } catch (e) {
      console.error('完成会话失败:', e);
      throw e;
    }
  }

  async function recordSessionDistraction(sessionId: number) {
    try {
      const session = await recordDistraction(sessionId);
      if (currentSession.value?.id === sessionId) {
        currentSession.value = session;
      }
      await loadTodaySessions();
    } catch (e) {
      console.error('记录分心失败:', e);
      throw e;
    }
  }

  async function updateSessionFocusTime(sessionId: number, seconds: number) {
    try {
      await updateFocusTime(sessionId, seconds);
      if (currentSession.value?.id === sessionId) {
        currentSession.value.actualFocusSeconds = seconds;
      }
    } catch (e) {
      console.error('更新专注时间失败:', e);
      throw e;
    }
  }

  async function addFocusApplication(name: string, processName?: string) {
    try {
      const app = await addFocusApp(name, processName);
      await loadFocusApps();
      return app;
    } catch (e) {
      console.error('添加白名单应用失败:', e);
      throw e;
    }
  }

  async function removeFocusApplication(appId: number) {
    try {
      await removeFocusApp(appId);
      await loadFocusApps();
    } catch (e) {
      console.error('删除白名单应用失败:', e);
      throw e;
    }
  }

  return {
    // State
    currentSession,
    todaySessions,
    todayStats,
    focusApps,
    loading,
    error,

    // 暂停恢复相关状态
    pauseStartTime,
    resumeLoading,
    interruptionAnalysis,
    resumeSuggestion,
    quickResumeTip,

    // Computed
    isActive,
    focusProgress,
    remainingTime,

    // Actions
    loadActiveSession,
    loadTodaySessions,
    loadTodayStats,
    loadFocusApps,
    createSession,
    startSession,
    pauseSession,
    resumeSession,
    cancelSession,
    completeSession,
    recordDistraction: recordSessionDistraction,
    updateFocusTime: updateSessionFocusTime,
    addFocusApp: addFocusApplication,
    removeFocusApp: removeFocusApplication,

    // 恢复相关方法
    getInterruptionDuration,
    fetchQuickResumeTip,
    fetchResumeSuggestions,
    clearResumeState,
  };
});
