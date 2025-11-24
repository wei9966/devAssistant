import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { appLauncherApi } from '@/api/appLauncherApi';
import type {
  AppItem,
  Category,
  Workflow,
  LaunchHistory,
  AppSearchFilter,
  AppSortType,
  ViewMode,
  CreateAppInput,
  UpdateAppInput,
  SearchResultItem,
} from '@/types/appLauncher';
import { DEFAULT_CATEGORIES } from '@/types/appLauncher';

// 拼音首字母映射（简化版，实际可使用pinyin库）
const getPinyinInitials = (str: string): string => {
  // 简化实现：这里只返回字符串本身，后续可集成pinyin库
  // 如 "Visual Studio Code" => "vsc"
  return str
    .toLowerCase()
    .split(/\s+/)
    .map((word) => word[0] || '')
    .join('');
};

export const useAppLauncherStore = defineStore('appLauncher', () => {
  // ==================== 状态 ====================
  const apps = ref<AppItem[]>([]);
  const categories = ref<Category[]>([]);
  const workflows = ref<Workflow[]>([]);
  const launchHistory = ref<LaunchHistory[]>([]);

  const searchKeyword = ref('');
  const selectedCategory = ref('all');
  const sortType = ref<AppSortType>('relevance');
  const viewMode = ref<ViewMode>('grid');

  const loading = ref(false);
  const error = ref<string | null>(null);
  const isScanning = ref(false);

  // ==================== 计算属性 ====================

  // 置顶应用
  const pinnedApps = computed(() => apps.value.filter((app) => app.isPinned && !app.isHidden));

  // 可见应用（未隐藏）
  const visibleApps = computed(() => apps.value.filter((app) => !app.isHidden));

  // 过滤后的应用列表
  const filteredApps = computed(() => {
    let result = visibleApps.value;

    // 分类过滤
    if (selectedCategory.value && selectedCategory.value !== 'all') {
      result = result.filter((app) => app.category === selectedCategory.value);
    }

    // 搜索过滤
    if (searchKeyword.value.trim()) {
      const keyword = searchKeyword.value.toLowerCase().trim();
      result = result.filter((app) => {
        const name = app.name.toLowerCase();
        const pinyin = getPinyinInitials(app.name);
        const tags = app.tags.join(' ').toLowerCase();
        const path = app.path.toLowerCase();

        return (
          name.includes(keyword) ||
          pinyin.includes(keyword) ||
          tags.includes(keyword) ||
          path.includes(keyword)
        );
      });
    }

    return result;
  });

  // 搜索结果（带匹配度分数）
  const searchResults = computed((): SearchResultItem[] => {
    if (!searchKeyword.value.trim()) {
      return filteredApps.value.map((app) => ({
        ...app,
        score: 1,
        matchType: 'name' as const,
      }));
    }

    const keyword = searchKeyword.value.toLowerCase().trim();

    return filteredApps.value.map((app) => {
      let score = 0;
      let matchType: 'name' | 'pinyin' | 'tag' = 'name';

      const name = app.name.toLowerCase();
      const pinyin = getPinyinInitials(app.name);
      const tags = app.tags.join(' ').toLowerCase();

      // 名称完全匹配（最高分）
      if (name === keyword) {
        score = 1.0;
        matchType = 'name';
      }
      // 名称开头匹配
      else if (name.startsWith(keyword)) {
        score = 0.9;
        matchType = 'name';
      }
      // 拼音完全匹配
      else if (pinyin === keyword) {
        score = 0.85;
        matchType = 'pinyin';
      }
      // 拼音开头匹配
      else if (pinyin.startsWith(keyword)) {
        score = 0.8;
        matchType = 'pinyin';
      }
      // 名称包含
      else if (name.includes(keyword)) {
        score = 0.7;
        matchType = 'name';
      }
      // 标签匹配
      else if (tags.includes(keyword)) {
        score = 0.6;
        matchType = 'tag';
      }
      // 其他匹配
      else {
        score = 0.5;
      }

      // 根据使用频率调整分数
      if (app.launchCount > 0) {
        score += Math.min(app.launchCount / 100, 0.1);
      }

      // 置顶应用加分
      if (app.isPinned) {
        score += 0.05;
      }

      return {
        ...app,
        score: Math.min(score, 1),
        matchType,
      };
    });
  });

  // 排序后的应用列表
  const sortedApps = computed(() => {
    const results = [...searchResults.value];

    switch (sortType.value) {
      case 'name':
        results.sort((a, b) => a.name.localeCompare(b.name));
        break;

      case 'launchCount':
        results.sort((a, b) => b.launchCount - a.launchCount);
        break;

      case 'lastLaunched':
        results.sort((a, b) => {
          const aTime = a.lastLaunchedAt || 0;
          const bTime = b.lastLaunchedAt || 0;
          return bTime - aTime;
        });
        break;

      case 'created':
        results.sort((a, b) => b.createdAt - a.createdAt);
        break;

      case 'relevance':
      default:
        // 按相关度和使用频率排序
        results.sort((a, b) => {
          if (Math.abs(b.score - a.score) > 0.01) {
            return b.score - a.score;
          }
          return b.launchCount - a.launchCount;
        });
        break;
    }

    return results;
  });

  // 最近启动的应用
  const recentApps = computed(() => {
    return [...apps.value]
      .filter((app) => !app.isHidden && app.lastLaunchedAt)
      .sort((a, b) => (b.lastLaunchedAt || 0) - (a.lastLaunchedAt || 0))
      .slice(0, 10);
  });

  // 常用应用（按启动次数排序）
  const frequentApps = computed(() => {
    return [...apps.value]
      .filter((app) => !app.isHidden && app.launchCount > 0)
      .sort((a, b) => b.launchCount - a.launchCount)
      .slice(0, 12);
  });

  // 当前分类的应用数量
  const currentCategoryCount = computed(() => {
    if (selectedCategory.value === 'all') {
      return visibleApps.value.length;
    }
    return visibleApps.value.filter((app) => app.category === selectedCategory.value).length;
  });

  // ==================== Actions ====================

  // 加载所有应用
  async function loadApps() {
    loading.value = true;
    error.value = null;
    try {
      apps.value = await appLauncherApi.getAllApps();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载应用失败';
      console.error('加载应用失败:', e);
    } finally {
      loading.value = false;
    }
  }

  // 扫描系统应用
  async function scanApps() {
    isScanning.value = true;
    error.value = null;
    try {
      const scannedApps = await appLauncherApi.scanInstalledApps();
      // 合并扫描结果与现有应用（避免重复）
      const existingIds = new Set(apps.value.map((app) => app.id));
      const newApps = scannedApps.filter((app) => !existingIds.has(app.id));
      apps.value = [...apps.value, ...newApps];
      return newApps.length;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '扫描应用失败';
      console.error('扫描应用失败:', e);
      throw e;
    } finally {
      isScanning.value = false;
    }
  }

  // 添加应用
  async function addApp(input: CreateAppInput) {
    loading.value = true;
    error.value = null;
    try {
      const appId = await appLauncherApi.addApp(input);
      await loadApps();
      return appId;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '添加应用失败';
      console.error('添加应用失败:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  // 更新应用
  async function updateApp(id: string, updates: UpdateAppInput) {
    try {
      await appLauncherApi.updateApp(id, updates);
      await loadApps();
    } catch (e) {
      console.error('更新应用失败:', e);
      throw e;
    }
  }

  // 删除应用
  async function deleteApp(id: string) {
    try {
      await appLauncherApi.deleteApp(id);
      await loadApps();
    } catch (e) {
      console.error('删除应用失败:', e);
      throw e;
    }
  }

  // 启动应用
  async function launchApp(id: string) {
    try {
      await appLauncherApi.launchApp(id);
      // 重新加载应用列表以更新启动次数和时间
      await loadApps();
    } catch (e) {
      console.error('启动应用失败:', e);
      throw e;
    }
  }

  // 批量启动应用
  async function launchMultipleApps(ids: string[], delay?: number) {
    try {
      await appLauncherApi.launchMultipleApps(ids, delay);
      await loadApps();
    } catch (e) {
      console.error('批量启动应用失败:', e);
      throw e;
    }
  }

  // 置顶/取消置顶应用
  async function togglePinApp(id: string) {
    const app = apps.value.find((a) => a.id === id);
    if (!app) return;

    try {
      if (app.isPinned) {
        await appLauncherApi.unpinApp(id);
      } else {
        await appLauncherApi.pinApp(id);
      }
      await loadApps();
    } catch (e) {
      console.error('切换置顶状态失败:', e);
      throw e;
    }
  }

  // 隐藏/显示应用
  async function toggleHideApp(id: string) {
    const app = apps.value.find((a) => a.id === id);
    if (!app) return;

    try {
      if (app.isHidden) {
        await appLauncherApi.showApp(id);
      } else {
        await appLauncherApi.hideApp(id);
      }
      await loadApps();
    } catch (e) {
      console.error('切换隐藏状态失败:', e);
      throw e;
    }
  }

  // 加载分类
  async function loadCategories() {
    try {
      const dbCategories = await appLauncherApi.getCategories();
      // 合并默认分类和数据库分类
      const allCategories = [
        ...DEFAULT_CATEGORIES.map((cat) => ({ ...cat, createdAt: Date.now() })),
        ...dbCategories,
      ];
      // 去重（以id为准）
      const uniqueCategories = Array.from(
        new Map(allCategories.map((cat) => [cat.id, cat])).values()
      );
      categories.value = uniqueCategories.sort((a, b) => a.sortOrder - b.sortOrder);
    } catch (e) {
      console.error('加载分类失败:', e);
    }
  }

  // 保存分类
  async function saveCategory(category: Omit<Category, 'createdAt'>) {
    try {
      await appLauncherApi.saveCategory(category);
      await loadCategories();
    } catch (e) {
      console.error('保存分类失败:', e);
      throw e;
    }
  }

  // 删除分类
  async function deleteCategory(id: string) {
    try {
      await appLauncherApi.deleteCategory(id);
      await loadCategories();
    } catch (e) {
      console.error('删除分类失败:', e);
      throw e;
    }
  }

  // 加载工作流
  async function loadWorkflows() {
    try {
      workflows.value = await appLauncherApi.getWorkflows();
    } catch (e) {
      console.error('加载工作流失败:', e);
    }
  }

  // 保存工作流
  async function saveWorkflow(workflow: Omit<Workflow, 'createdAt' | 'updatedAt'>) {
    try {
      await appLauncherApi.saveWorkflow(workflow);
      await loadWorkflows();
    } catch (e) {
      console.error('保存工作流失败:', e);
      throw e;
    }
  }

  // 删除工作流
  async function deleteWorkflow(id: string) {
    try {
      await appLauncherApi.deleteWorkflow(id);
      await loadWorkflows();
    } catch (e) {
      console.error('删除工作流失败:', e);
      throw e;
    }
  }

  // 启动工作流
  async function launchWorkflow(id: string) {
    try {
      await appLauncherApi.launchWorkflow(id);
      await loadApps();
    } catch (e) {
      console.error('启动工作流失败:', e);
      throw e;
    }
  }

  // 加载启动历史
  async function loadLaunchHistory(limit: number = 20) {
    try {
      launchHistory.value = await appLauncherApi.getLaunchHistory(limit);
    } catch (e) {
      console.error('加载启动历史失败:', e);
    }
  }

  // 搜索应用
  function searchApps(keyword: string) {
    searchKeyword.value = keyword;
  }

  // 切换分类
  function selectCategory(categoryId: string) {
    selectedCategory.value = categoryId;
  }

  // 切换排序方式
  function setSortType(type: AppSortType) {
    sortType.value = type;
  }

  // 切换视图模式
  function setViewMode(mode: ViewMode) {
    viewMode.value = mode;
  }

  // 清空搜索
  function clearSearch() {
    searchKeyword.value = '';
  }

  // 初始化数据
  async function initialize() {
    await Promise.all([loadApps(), loadCategories(), loadWorkflows(), loadLaunchHistory()]);
  }

  // 导出配置
  async function exportConfig(): Promise<string> {
    try {
      return await appLauncherApi.exportConfig();
    } catch (e) {
      console.error('导出配置失败:', e);
      throw e;
    }
  }

  // 导入配置
  async function importConfig(configJson: string) {
    try {
      await appLauncherApi.importConfig(configJson);
      await initialize();
    } catch (e) {
      console.error('导入配置失败:', e);
      throw e;
    }
  }

  return {
    // State
    apps,
    categories,
    workflows,
    launchHistory,
    searchKeyword,
    selectedCategory,
    sortType,
    viewMode,
    loading,
    error,
    isScanning,

    // Computed
    pinnedApps,
    visibleApps,
    filteredApps,
    searchResults,
    sortedApps,
    recentApps,
    frequentApps,
    currentCategoryCount,

    // Actions
    loadApps,
    scanApps,
    addApp,
    updateApp,
    deleteApp,
    launchApp,
    launchMultipleApps,
    togglePinApp,
    toggleHideApp,
    loadCategories,
    saveCategory,
    deleteCategory,
    loadWorkflows,
    saveWorkflow,
    deleteWorkflow,
    launchWorkflow,
    loadLaunchHistory,
    searchApps,
    selectCategory,
    setSortType,
    setViewMode,
    clearSearch,
    initialize,
    exportConfig,
    importConfig,
  };
});
