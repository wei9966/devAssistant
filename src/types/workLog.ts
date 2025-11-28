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
