import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { sqlApi } from '@/api/sqlApi';
import type {
  SqlRecord,
  SqlCategory,
  SqlClassifyResult,
  AiProvider,
  SqlTemplate,
  ConsolidateResult,
  TemplateGroupBy,
  TemplateGroupStats,
} from '@/types/sql';

export const useSqlStore = defineStore('sql', () => {
  // 状态
  const recentSqls = ref<SqlRecord[]>([]);
  const favoriteSqls = ref<SqlRecord[]>([]);
  const categories = ref<SqlCategory[]>([]);
  const categoryCounts = ref<Map<number, number>>(new Map());
  const loading = ref(false);
  const aiConfigured = ref(false);
  const aiClassifying = ref(false);

  // 模板相关状态
  const templates = ref<SqlTemplate[]>([]);
  const hotTemplates = ref<SqlTemplate[]>([]);
  const templateGroupStats = ref<TemplateGroupStats | null>(null);
  const currentGroupBy = ref<TemplateGroupBy>(null);
  const isConsolidating = ref(false);
  const isIdentifyingScenes = ref(false);
  const lastConsolidateResult = ref<ConsolidateResult | null>(null);

  // 计算属性
  const totalRecentCount = computed(() => recentSqls.value.length);
  const totalFavoriteCount = computed(() => favoriteSqls.value.length);

  // 按SQL类型统计
  const sqlsByType = computed(() => {
    const result: Record<string, number> = {};
    recentSqls.value.forEach((sql) => {
      const type = sql.sqlType || 'UNKNOWN';
      result[type] = (result[type] || 0) + 1;
    });
    return result;
  });

  // 按数据库统计
  const sqlsByDatabase = computed(() => {
    const result: Record<string, number> = {};
    recentSqls.value.forEach((sql) => {
      const db = sql.databaseName || 'Unknown';
      result[db] = (result[db] || 0) + 1;
    });
    return result;
  });

  // 按分类统计（多标签）
  const sqlsByCategory = computed(() => {
    const result: Record<string, number> = {};
    recentSqls.value.forEach((sql) => {
      if (sql.categories && sql.categories.length > 0) {
        sql.categories.forEach((cat) => {
          result[cat.name] = (result[cat.name] || 0) + 1;
        });
      } else {
        result['未分类'] = (result['未分类'] || 0) + 1;
      }
    });
    return result;
  });

  // 未分类数量
  const uncategorizedCount = computed(() => {
    return recentSqls.value.filter(
      (sql) => !sql.categories || sql.categories.length === 0 || !sql.name
    ).length;
  });

  // 模板相关计算属性
  // 按业务场景分组的模板
  const templatesByScene = computed(() => {
    const result: Record<string, SqlTemplate[]> = {};
    templates.value.forEach((template) => {
      const scene = template.businessScene || '未分类';
      if (!result[scene]) {
        result[scene] = [];
      }
      result[scene].push(template);
    });
    return result;
  });

  // 按表名分组的模板
  const templatesByTable = computed(() => {
    const result: Record<string, SqlTemplate[]> = {};
    templates.value.forEach((template) => {
      template.tableNames.forEach((tableName) => {
        if (!result[tableName]) {
          result[tableName] = [];
        }
        result[tableName].push(template);
      });
    });
    return result;
  });

  // 未分类场景的模板数量
  const unclassifiedTemplateCount = computed(() => {
    return templates.value.filter((template) => !template.businessScene).length;
  });

  // 操作
  async function loadRecentSqls(limit: number = 50) {
    loading.value = true;
    try {
      recentSqls.value = await sqlApi.getRecentSqls(limit);
    } catch (error) {
      console.error('加载最近SQL失败:', error);
    } finally {
      loading.value = false;
    }
  }

  async function loadFavoriteSqls() {
    loading.value = true;
    try {
      favoriteSqls.value = await sqlApi.getFavoriteSqls();
    } catch (error) {
      console.error('加载收藏SQL失败:', error);
    } finally {
      loading.value = false;
    }
  }

  async function loadCategories() {
    try {
      categories.value = await sqlApi.getCategories();
    } catch (error) {
      console.error('加载分类失败:', error);
    }
  }

  async function loadCategoryCounts() {
    try {
      const counts = await sqlApi.getCategorySqlCounts();
      categoryCounts.value = new Map(counts);
    } catch (error) {
      console.error('加载分类数量失败:', error);
    }
  }

  async function saveSql(sqlText: string, source: string = 'manual') {
    try {
      const id = await sqlApi.saveSql(sqlText, source);
      await loadRecentSqls();
      return id;
    } catch (error) {
      console.error('保存SQL失败:', error);
      throw error;
    }
  }

  async function toggleFavorite(sqlId: number) {
    try {
      await sqlApi.toggleFavorite(sqlId);
      await loadRecentSqls();
      await loadFavoriteSqls();
    } catch (error) {
      console.error('切换收藏状态失败:', error);
      throw error;
    }
  }

  async function deleteSql(sqlId: number) {
    try {
      await sqlApi.deleteSql(sqlId);
      await loadRecentSqls();
      await loadFavoriteSqls();
    } catch (error) {
      console.error('删除SQL失败:', error);
      throw error;
    }
  }

  // 更新 SQL 名称和分类（多标签）
  async function updateSqlNameCategories(
    sqlId: number,
    name?: string,
    categoryIds: number[] = []
  ) {
    try {
      await sqlApi.updateSqlNameAndCategories(sqlId, name, categoryIds);
      await loadRecentSqls();
      await loadFavoriteSqls();
      await loadCategoryCounts();
    } catch (error) {
      console.error('更新SQL分类失败:', error);
      throw error;
    }
  }

  // 兼容旧接口
  async function updateSqlNameCategory(
    sqlId: number,
    name?: string,
    categoryId?: number
  ) {
    try {
      await sqlApi.updateSqlNameCategory(sqlId, name, categoryId);
      await loadRecentSqls();
      await loadFavoriteSqls();
    } catch (error) {
      console.error('更新SQL分类失败:', error);
      throw error;
    }
  }

  // 添加分类（支持 AI 提示词）
  async function addCategory(
    name: string,
    description?: string,
    color?: string,
    icon?: string,
    aiPrompt?: string
  ) {
    try {
      const id = await sqlApi.addCategory(name, description, color, icon, aiPrompt);
      await loadCategories();
      return id;
    } catch (error) {
      console.error('添加分类失败:', error);
      throw error;
    }
  }

  // 更新分类（支持 AI 提示词）
  async function updateCategory(
    categoryId: number,
    name: string,
    description?: string,
    color?: string,
    icon?: string,
    aiPrompt?: string
  ) {
    try {
      await sqlApi.updateCategory(categoryId, name, description, color, icon, aiPrompt);
      await loadCategories();
    } catch (error) {
      console.error('更新分类失败:', error);
      throw error;
    }
  }

  // 删除分类
  async function deleteCategory(categoryId: number) {
    try {
      await sqlApi.deleteCategory(categoryId);
      await loadCategories();
      await loadRecentSqls();
      await loadCategoryCounts();
    } catch (error) {
      console.error('删除分类失败:', error);
      throw error;
    }
  }

  // 配置 AI
  async function configureAi(
    provider: AiProvider,
    apiKey: string,
    baseUrl?: string,
    model?: string
  ) {
    try {
      await sqlApi.configureAi(provider, apiKey, baseUrl, model);
      aiConfigured.value = true;
    } catch (error) {
      console.error('配置AI失败:', error);
      throw error;
    }
  }

  // 检查 AI 状态
  async function checkAiStatus() {
    try {
      aiConfigured.value = await sqlApi.getAiStatus();
    } catch (error) {
      console.error('检查AI状态失败:', error);
      aiConfigured.value = false;
    }
  }

  // 测试 AI 连接
  async function testAiConnection(): Promise<boolean> {
    try {
      return await sqlApi.testAiConnection();
    } catch (error) {
      console.error('测试AI连接失败:', error);
      return false;
    }
  }

  // AI 自动分类
  async function aiClassifySqls(
    sqlIds?: number[],
    limit?: number
  ): Promise<SqlClassifyResult[]> {
    aiClassifying.value = true;
    try {
      const results = await sqlApi.aiClassifySqls(sqlIds, limit);
      await loadRecentSqls();
      await loadFavoriteSqls();
      await loadCategoryCounts();
      return results;
    } catch (error) {
      console.error('AI分类失败:', error);
      throw error;
    } finally {
      aiClassifying.value = false;
    }
  }

  // 搜索SQL（本地过滤）
  function searchSqls(keyword: string): SqlRecord[] {
    if (!keyword) return recentSqls.value;

    const lowerKeyword = keyword.toLowerCase();
    return recentSqls.value.filter((sql) =>
      sql.sqlText.toLowerCase().includes(lowerKeyword) ||
      sql.databaseName?.toLowerCase().includes(lowerKeyword) ||
      sql.description?.toLowerCase().includes(lowerKeyword) ||
      sql.tags?.toLowerCase().includes(lowerKeyword) ||
      sql.name?.toLowerCase().includes(lowerKeyword) ||
      sql.categories?.some((cat) => cat.name.toLowerCase().includes(lowerKeyword))
    );
  }

  // 按分类筛选 SQL
  function filterSqlsByCategory(categoryId: number): SqlRecord[] {
    return recentSqls.value.filter((sql) =>
      sql.categories?.some((cat) => cat.id === categoryId)
    );
  }

  // 模板相关操作
  // 加载模板列表
  async function loadTemplates(groupBy?: TemplateGroupBy, limit?: number) {
    loading.value = true;
    try {
      templates.value = await sqlApi.getSqlTemplates(groupBy, limit);
      currentGroupBy.value = groupBy || null;
    } catch (error) {
      console.error('加载模板列表失败:', error);
      throw error;
    } finally {
      loading.value = false;
    }
  }

  // 加载热门模板
  async function loadHotTemplates(limit?: number) {
    loading.value = true;
    try {
      hotTemplates.value = await sqlApi.getHotSqlTemplates(limit);
    } catch (error) {
      console.error('加载热门模板失败:', error);
      throw error;
    } finally {
      loading.value = false;
    }
  }

  // 加载分组统计
  async function loadTemplateGroupStats() {
    try {
      templateGroupStats.value = await sqlApi.getTemplateGroupStats();
    } catch (error) {
      console.error('加载分组统计失败:', error);
      throw error;
    }
  }

  // 执行智能整合
  async function consolidateTemplates(): Promise<ConsolidateResult> {
    isConsolidating.value = true;
    try {
      const result = await sqlApi.consolidateSqlTemplates();
      lastConsolidateResult.value = result;
      // 整合后重新加载模板列表
      await loadTemplates(currentGroupBy.value);
      await loadTemplateGroupStats();
      return result;
    } catch (error) {
      console.error('智能整合失败:', error);
      throw error;
    } finally {
      isConsolidating.value = false;
    }
  }

  // AI识别业务场景
  async function identifyScenes(templateIds?: number[]): Promise<number> {
    isIdentifyingScenes.value = true;
    try {
      const count = await sqlApi.identifyTemplateScenes(templateIds);
      // 识别后重新加载模板列表
      await loadTemplates(currentGroupBy.value);
      await loadTemplateGroupStats();
      return count;
    } catch (error) {
      console.error('AI识别业务场景失败:', error);
      throw error;
    } finally {
      isIdentifyingScenes.value = false;
    }
  }

  // 切换模板收藏
  async function toggleTemplateFavorite(templateId: number) {
    try {
      await sqlApi.toggleTemplateFavorite(templateId);
      // 更新本地状态
      const template = templates.value.find((t) => t.id === templateId);
      if (template) {
        template.isFavorite = !template.isFavorite;
      }
      const hotTemplate = hotTemplates.value.find((t) => t.id === templateId);
      if (hotTemplate) {
        hotTemplate.isFavorite = !hotTemplate.isFavorite;
      }
    } catch (error) {
      console.error('切换模板收藏失败:', error);
      throw error;
    }
  }

  // 获取模板变体
  async function getTemplateVariants(templateId: number): Promise<SqlRecord[]> {
    try {
      return await sqlApi.getTemplateVariants(templateId);
    } catch (error) {
      console.error('获取模板变体失败:', error);
      throw error;
    }
  }

  // 按表名获取模板
  async function getTemplatesByTable(tableName: string): Promise<SqlTemplate[]> {
    try {
      return await sqlApi.getTemplatesByTable(tableName);
    } catch (error) {
      console.error('按表名获取模板失败:', error);
      throw error;
    }
  }

  // 设置分组方式
  function setGroupBy(groupBy: TemplateGroupBy) {
    currentGroupBy.value = groupBy;
  }

  return {
    recentSqls,
    favoriteSqls,
    categories,
    categoryCounts,
    loading,
    aiConfigured,
    aiClassifying,
    totalRecentCount,
    totalFavoriteCount,
    sqlsByType,
    sqlsByDatabase,
    sqlsByCategory,
    uncategorizedCount,
    loadRecentSqls,
    loadFavoriteSqls,
    loadCategories,
    loadCategoryCounts,
    saveSql,
    toggleFavorite,
    deleteSql,
    updateSqlNameCategory,
    updateSqlNameCategories,
    addCategory,
    updateCategory,
    deleteCategory,
    configureAi,
    checkAiStatus,
    testAiConnection,
    aiClassifySqls,
    searchSqls,
    filterSqlsByCategory,
    // 模板相关状态和方法
    templates,
    hotTemplates,
    templateGroupStats,
    currentGroupBy,
    isConsolidating,
    isIdentifyingScenes,
    lastConsolidateResult,
    templatesByScene,
    templatesByTable,
    unclassifiedTemplateCount,
    loadTemplates,
    loadHotTemplates,
    loadTemplateGroupStats,
    consolidateTemplates,
    identifyScenes,
    toggleTemplateFavorite,
    getTemplateVariants,
    getTemplatesByTable,
    setGroupBy,
  };
});
