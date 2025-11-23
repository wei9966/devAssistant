import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { sqlApi } from '@/api/sqlApi';
import type { SqlRecord } from '@/types/sql';

export const useSqlStore = defineStore('sql', () => {
  // 状态
  const recentSqls = ref<SqlRecord[]>([]);
  const favoriteSqls = ref<SqlRecord[]>([]);
  const loading = ref(false);

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

  // 搜索SQL（本地过滤）
  function searchSqls(keyword: string): SqlRecord[] {
    if (!keyword) return recentSqls.value;

    const lowerKeyword = keyword.toLowerCase();
    return recentSqls.value.filter((sql) =>
      sql.sqlText.toLowerCase().includes(lowerKeyword) ||
      sql.databaseName?.toLowerCase().includes(lowerKeyword) ||
      sql.description?.toLowerCase().includes(lowerKeyword) ||
      sql.tags?.toLowerCase().includes(lowerKeyword)
    );
  }

  return {
    recentSqls,
    favoriteSqls,
    loading,
    totalRecentCount,
    totalFavoriteCount,
    sqlsByType,
    sqlsByDatabase,
    loadRecentSqls,
    loadFavoriteSqls,
    saveSql,
    toggleFavorite,
    deleteSql,
    searchSqls,
  };
});
