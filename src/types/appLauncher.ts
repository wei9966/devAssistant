// AppLauncher 类型定义
// 与 Rust 模型保持一致

export interface AppItem {
  id: string;
  name: string;
  path: string;
  icon?: string; // base64图标或图标路径
  category: string;
  tags: string[];
  launchCount: number;
  lastLaunchedAt?: number; // Unix时间戳
  isPinned: boolean;
  isHidden: boolean;
  launchArgs?: string;
  createdAt: number;
  updatedAt: number;
}

export interface Category {
  id: string;
  name: string;
  color: string;
  icon?: string;
  sortOrder: number;
  createdAt: number;
}

export interface Workflow {
  id: string;
  name: string;
  appIds: string[];
  launchDelay?: number; // 应用启动间隔（ms）
  createdAt: number;
  updatedAt: number;
}

export interface LaunchHistory {
  id: number;
  appId: string;
  launchedAt: number;
}

// 搜索过滤条件
export interface AppSearchFilter {
  keyword: string;
  category?: string;
  isPinned?: boolean;
  isHidden?: boolean;
}

// 应用排序方式
export type AppSortType =
  | 'name' // 按名称排序
  | 'launchCount' // 按启动次数排序
  | 'lastLaunched' // 按最近启动时间排序
  | 'created' // 按创建时间排序
  | 'relevance'; // 按搜索相关度排序

// 视图模式
export type ViewMode = 'grid' | 'list';

// 应用类型（用于图标显示）
export type AppType =
  | 'exe' // 可执行文件
  | 'lnk' // 快捷方式
  | 'folder' // 文件夹
  | 'url'; // 网址

// 创建/更新应用的输入参数
export interface CreateAppInput {
  name: string;
  path: string;
  icon?: string;
  category: string;
  tags?: string[];
  launchArgs?: string;
}

export interface UpdateAppInput {
  name?: string;
  path?: string;
  icon?: string;
  category?: string;
  tags?: string[];
  isPinned?: boolean;
  isHidden?: boolean;
  launchArgs?: string;
}

// 分类的中文映射（默认分类）
export const DEFAULT_CATEGORIES: Omit<Category, 'createdAt'>[] = [
  { id: 'all', name: '全部', color: '#6366f1', icon: '📦', sortOrder: 0 },
  { id: 'dev', name: '开发工具', color: '#6366f1', icon: '💻', sortOrder: 1 },
  { id: 'office', name: '办公软件', color: '#10b981', icon: '📝', sortOrder: 2 },
  { id: 'browser', name: '浏览器', color: '#3b82f6', icon: '🌐', sortOrder: 3 },
  { id: 'design', name: '设计工具', color: '#f59e0b', icon: '🎨', sortOrder: 4 },
  { id: 'media', name: '影音娱乐', color: '#ef4444', icon: '🎵', sortOrder: 5 },
  { id: 'game', name: '游戏', color: '#8b5cf6', icon: '🎮', sortOrder: 6 },
  { id: 'other', name: '其他', color: '#6b7280', icon: '📁', sortOrder: 7 },
];

// 搜索结果项（带匹配度分数）
export interface SearchResultItem extends AppItem {
  score: number; // 搜索匹配度分数（0-1）
  matchType: 'name' | 'pinyin' | 'tag'; // 匹配类型
}
