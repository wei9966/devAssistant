import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { taskCategoryApi } from '@/api/taskCategoryApi';
import { CATEGORY_LABELS } from '@/types/task';
import type { TaskCategoryDefinition } from '@/types/task';

export const useTaskCategoryStore = defineStore('taskCategories', () => {
  const categories = ref<TaskCategoryDefinition[]>([]);
  const loading = ref(false);
  const loaded = ref(false);

  const visibleCategories = computed(() => categories.value.filter(category => !category.isHidden));
  const hiddenCategories = computed(() => categories.value.filter(category => category.isHidden));

  async function loadCategories(force = false) {
    if (loaded.value && !force) return;
    loading.value = true;
    try {
      categories.value = await taskCategoryApi.getAll();
      loaded.value = true;
    } finally {
      loading.value = false;
    }
  }

  function getByKey(key?: string) {
    return categories.value.find(category => category.key === key);
  }

  function getLabel(key?: string) {
    if (!key) return '其他';
    return getByKey(key)?.name || CATEGORY_LABELS[key] || key;
  }

  function getColor(key?: string) {
    return getByKey(key)?.color;
  }

  function getCustomColor(key?: string) {
    const category = getByKey(key);
    return category && !category.isSystem ? category.color : undefined;
  }

  async function createCategory(name: string, color: string) {
    const category = await taskCategoryApi.create(name, color);
    await loadCategories(true);
    return category;
  }

  async function updateCategory(id: number, name: string, color: string) {
    const category = await taskCategoryApi.update(id, name, color);
    await loadCategories(true);
    return category;
  }

  async function setCategoryHidden(id: number, isHidden: boolean) {
    const category = await taskCategoryApi.setHidden(id, isHidden);
    await loadCategories(true);
    return category;
  }

  async function deleteCategory(id: number) {
    await taskCategoryApi.delete(id);
    await loadCategories(true);
  }

  return {
    categories,
    visibleCategories,
    hiddenCategories,
    loading,
    loaded,
    loadCategories,
    getByKey,
    getLabel,
    getColor,
    getCustomColor,
    createCategory,
    updateCategory,
    setCategoryHidden,
    deleteCategory,
  };
});
