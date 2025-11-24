import { invoke } from '@tauri-apps/api/core';
import type {
  AppItem,
  Category,
  Workflow,
  LaunchHistory,
  CreateAppInput,
  UpdateAppInput,
} from '@/types/appLauncher';

export const appLauncherApi = {
  // ==================== 应用管理 ====================

  // 扫描系统已安装应用
  async scanInstalledApps(): Promise<AppItem[]> {
    return await invoke('scan_installed_apps');
  },

  // 获取所有应用
  async getAllApps(): Promise<AppItem[]> {
    return await invoke('get_all_apps');
  },

  // 添加应用
  async addApp(app: CreateAppInput): Promise<string> {
    return await invoke('add_app', { app });
  },

  // 更新应用
  async updateApp(id: string, updates: UpdateAppInput): Promise<void> {
    await invoke('update_app', { id, updates });
  },

  // 删除应用
  async deleteApp(id: string): Promise<void> {
    await invoke('delete_app', { id });
  },

  // 搜索应用
  async searchApps(keyword: string, category?: string): Promise<AppItem[]> {
    return await invoke('search_apps', {
      keyword,
      category: category || null,
    });
  },

  // ==================== 应用启动 ====================

  // 启动应用
  async launchApp(id: string): Promise<void> {
    await invoke('launch_app', { id });
  },

  // 批量启动应用
  async launchMultipleApps(ids: string[], delay?: number): Promise<void> {
    await invoke('launch_multiple_apps', {
      ids,
      delay: delay || 0,
    });
  },

  // 启动工作流
  async launchWorkflow(workflowId: string): Promise<void> {
    await invoke('launch_workflow', { workflowId });
  },

  // ==================== 分类管理 ====================

  // 获取所有分类
  async getCategories(): Promise<Category[]> {
    return await invoke('get_categories');
  },

  // 保存分类
  async saveCategory(category: Omit<Category, 'createdAt'>): Promise<string> {
    return await invoke('save_category', { category });
  },

  // 删除分类
  async deleteCategory(id: string): Promise<void> {
    await invoke('delete_category', { id });
  },

  // 更新分类排序
  async updateCategoryOrder(categoryIds: string[]): Promise<void> {
    await invoke('update_category_order', { categoryIds });
  },

  // ==================== 工作流管理 ====================

  // 获取所有工作流
  async getWorkflows(): Promise<Workflow[]> {
    return await invoke('get_workflows');
  },

  // 保存工作流
  async saveWorkflow(workflow: Omit<Workflow, 'createdAt' | 'updatedAt'>): Promise<string> {
    return await invoke('save_workflow', { workflow });
  },

  // 删除工作流
  async deleteWorkflow(id: string): Promise<void> {
    await invoke('delete_workflow', { id });
  },

  // 更新工作流
  async updateWorkflow(
    id: string,
    updates: {
      name?: string;
      appIds?: string[];
      launchDelay?: number;
    }
  ): Promise<void> {
    await invoke('update_workflow', { id, updates });
  },

  // ==================== 图标处理 ====================

  // 提取exe图标
  async extractIcon(exePath: string): Promise<string> {
    return await invoke('extract_icon', { exePath });
  },

  // ==================== 启动历史 ====================

  // 获取启动历史
  async getLaunchHistory(limit: number = 20): Promise<LaunchHistory[]> {
    return await invoke('get_launch_history', { limit });
  },

  // 清除启动历史
  async clearLaunchHistory(): Promise<void> {
    await invoke('clear_launch_history');
  },

  // 获取最近启动的应用
  async getRecentApps(limit: number = 10): Promise<AppItem[]> {
    return await invoke('get_recent_apps', { limit });
  },

  // ==================== 应用操作 ====================

  // 置顶应用
  async pinApp(id: string): Promise<void> {
    await invoke('update_app', {
      id,
      updates: { isPinned: true },
    });
  },

  // 取消置顶
  async unpinApp(id: string): Promise<void> {
    await invoke('update_app', {
      id,
      updates: { isPinned: false },
    });
  },

  // 隐藏应用
  async hideApp(id: string): Promise<void> {
    await invoke('update_app', {
      id,
      updates: { isHidden: true },
    });
  },

  // 显示应用
  async showApp(id: string): Promise<void> {
    await invoke('update_app', {
      id,
      updates: { isHidden: false },
    });
  },

  // ==================== 配置管理 ====================

  // 导出配置
  async exportConfig(): Promise<string> {
    return await invoke('export_app_launcher_config');
  },

  // 导入配置
  async importConfig(configJson: string): Promise<void> {
    await invoke('import_app_launcher_config', { configJson });
  },

  // ==================== 系统集成 ====================

  // 打开文件选择对话框
  async selectFile(
    filters?: { name: string; extensions: string[] }[]
  ): Promise<string | null> {
    return await invoke('select_file', { filters: filters || null });
  },

  // 打开文件夹选择对话框
  async selectFolder(): Promise<string | null> {
    return await invoke('select_folder');
  },

  // 验证路径是否存在
  async validatePath(path: string): Promise<boolean> {
    return await invoke('validate_path', { path });
  },

  // 获取文件图标（快捷方式、exe等）
  async getFileIcon(path: string): Promise<string | null> {
    try {
      return await invoke('get_file_icon', { path });
    } catch (error) {
      console.error('获取文件图标失败:', error);
      return null;
    }
  },
};
