import { invoke } from '@tauri-apps/api/core';
import type {
  SqlRecord,
  SqlCategory,
  SqlClassifyResult,
  AiProvider,
  SqlTemplate,
  ConsolidateResult,
  TemplateGroupBy,
  TemplateGroupStats
} from '@/types/sql';

export const sqlApi = {
  // 保存 SQL
  async saveSql(sqlText: string, source: string): Promise<number> {
    return await invoke('save_sql', { sqlText, source });
  },

  // 获取最近的 SQL
  async getRecentSqls(limit: number): Promise<SqlRecord[]> {
    return await invoke('get_recent_sqls', { limit });
  },

  // 获取收藏的 SQL
  async getFavoriteSqls(): Promise<SqlRecord[]> {
    return await invoke('get_favorite_sqls');
  },

  // 切换收藏状态
  async toggleFavorite(sqlId: number): Promise<void> {
    await invoke('toggle_favorite_sql', { sqlId });
  },

  // 删除 SQL
  async deleteSql(sqlId: number): Promise<void> {
    await invoke('delete_sql', { sqlId });
  },

  // 更新 SQL 名称和分类（兼容旧接口）
  async updateSqlNameCategory(
    sqlId: number,
    name?: string,
    categoryId?: number
  ): Promise<void> {
    await invoke('update_sql_name_category', { sqlId, name, categoryId });
  },

  // 更新 SQL 名称和多个分类
  async updateSqlNameAndCategories(
    sqlId: number,
    name?: string,
    categoryIds: number[] = []
  ): Promise<void> {
    await invoke('update_sql_name_and_categories', { sqlId, name, categoryIds });
  },

  // 设置 SQL 的分类标签（多个）
  async setSqlCategories(sqlId: number, categoryIds: number[]): Promise<void> {
    await invoke('set_sql_categories', { sqlId, categoryIds });
  },

  // 添加一个分类标签
  async addSqlCategoryTag(sqlId: number, categoryId: number): Promise<void> {
    await invoke('add_sql_category_tag', { sqlId, categoryId });
  },

  // 移除一个分类标签
  async removeSqlCategoryTag(sqlId: number, categoryId: number): Promise<void> {
    await invoke('remove_sql_category_tag', { sqlId, categoryId });
  },

  // 批量更新 SQL 名称和分类（兼容旧接口）
  async batchUpdateSqlNameCategory(
    updates: Array<[number, string | null, number | null]>
  ): Promise<number> {
    return await invoke('batch_update_sql_name_category', { updates });
  },

  // 批量更新 SQL 名称和多个分类
  async batchUpdateSqlNameCategories(
    updates: Array<[number, string | null, number[]]>
  ): Promise<number> {
    return await invoke('batch_update_sql_name_categories', { updates });
  },

  // 获取所有分类
  async getCategories(): Promise<SqlCategory[]> {
    return await invoke('get_sql_categories');
  },

  // 添加分类（支持 AI 提示词）
  async addCategory(
    name: string,
    description?: string,
    color?: string,
    icon?: string,
    aiPrompt?: string
  ): Promise<number> {
    return await invoke('add_sql_category', { name, description, color, icon, aiPrompt });
  },

  // 更新分类（支持 AI 提示词）
  async updateCategory(
    categoryId: number,
    name: string,
    description?: string,
    color?: string,
    icon?: string,
    aiPrompt?: string
  ): Promise<void> {
    await invoke('update_sql_category', { categoryId, name, description, color, icon, aiPrompt });
  },

  // 删除分类
  async deleteCategory(categoryId: number): Promise<void> {
    await invoke('delete_sql_category', { categoryId });
  },

  // 获取未分类的 SQL
  async getUncategorizedSqls(limit: number): Promise<SqlRecord[]> {
    return await invoke('get_uncategorized_sqls', { limit });
  },

  // 根据分类获取 SQL
  async getSqlsByCategory(categoryId: number, limit: number): Promise<SqlRecord[]> {
    return await invoke('get_sqls_by_category', { categoryId, limit });
  },

  // 获取分类的 SQL 数量统计
  async getCategorySqlCounts(): Promise<Array<[number, number]>> {
    return await invoke('get_category_sql_counts');
  },

  // === AI 相关 API ===

  // 配置 AI 服务
  async configureAi(
    provider: AiProvider,
    apiKey: string,
    baseUrl?: string,
    model?: string
  ): Promise<void> {
    await invoke('configure_sql_ai', { provider, apiKey, baseUrl, model });
  },

  // 获取 AI 配置状态
  async getAiStatus(): Promise<boolean> {
    return await invoke('get_sql_ai_status');
  },

  // 测试 AI 连接
  async testAiConnection(): Promise<boolean> {
    return await invoke('test_sql_ai_connection');
  },

  // AI 自动分类 SQL
  async aiClassifySqls(
    sqlIds?: number[],
    limit?: number
  ): Promise<SqlClassifyResult[]> {
    console.log('=== sqlApi.aiClassifySqls 被调用 ===');
    console.log('参数:', { sqlIds, limit });
    try {
      const result = await invoke('ai_classify_sqls', { sqlIds, limit });
      console.log('调用成功, 结果:', result);
      return result as SqlClassifyResult[];
    } catch (error) {
      console.error('invoke ai_classify_sqls 失败:', error);
      throw error;
    }
  },

  // 手动分类单条 SQL（支持多标签）
  async manualClassifySql(
    sqlId: number,
    name?: string,
    categoryIds?: number[]
  ): Promise<void> {
    await invoke('manual_classify_sql', { sqlId, name, categoryIds });
  },

  // === SQL 模板相关 API ===

  // 智能整合SQL历史
  async consolidateSqlTemplates(): Promise<ConsolidateResult> {
    return await invoke('consolidate_sql_templates');
  },

  // 获取模板列表
  async getSqlTemplates(groupBy?: TemplateGroupBy, limit?: number): Promise<SqlTemplate[]> {
    return await invoke('get_sql_templates', { groupBy, limit });
  },

  // 获取热门模板
  async getHotSqlTemplates(limit?: number): Promise<SqlTemplate[]> {
    return await invoke('get_hot_sql_templates', { limit });
  },

  // 获取模板的变体SQL
  async getTemplateVariants(templateId: number): Promise<SqlRecord[]> {
    return await invoke('get_template_variants', { templateId });
  },

  // 按表名获取模板
  async getTemplatesByTable(tableName: string): Promise<SqlTemplate[]> {
    return await invoke('get_templates_by_table', { tableName });
  },

  // AI识别业务场景
  async identifyTemplateScenes(templateIds?: number[]): Promise<number> {
    return await invoke('identify_template_scenes', { templateIds });
  },

  // 切换模板收藏
  async toggleTemplateFavorite(templateId: number): Promise<boolean> {
    return await invoke('toggle_template_favorite', { templateId });
  },

  // 获取分组统计
  async getTemplateGroupStats(): Promise<TemplateGroupStats> {
    return await invoke('get_template_group_stats');
  },
};
