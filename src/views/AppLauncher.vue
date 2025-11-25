<template>
  <div
    class="app-launcher"
    @dragover.prevent="handleGlobalDragOver"
    @dragenter.prevent="handleGlobalDragEnter"
  >
    <!-- Header -->
    <div class="launcher-header">
      <h2 class="page-title">应用启动器</h2>
      <n-space>
        <n-button @click="handleScanApps" :loading="scanning">
          <template #icon>
            <n-icon><ScanOutline /></n-icon>
          </template>
          扫描应用
        </n-button>
        <n-button @click="handleRefreshIcons" :loading="refreshingIcons">
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          刷新图标
        </n-button>
        <n-button class="primary-button" @click="handleAddApp">
          <template #icon>
            <n-icon><AddOutline /></n-icon>
          </template>
          添加应用
        </n-button>
        <n-dropdown :options="moreOptions" @select="handleMoreAction">
          <n-button>
            <template #icon>
              <n-icon><EllipsisHorizontal /></n-icon>
            </template>
          </n-button>
        </n-dropdown>
      </n-space>
    </div>

    <!-- Search Bar -->
    <div class="search-section">
      <AppSearchBar
        v-model="searchKeyword"
        :result-count="filteredApps.length"
        @search="handleSearch"
        @clear="handleClearSearch"
      />
    </div>

    <!-- Category Filter -->
    <div class="filter-section">
      <AppCategoryFilter
        :categories="categories"
        :selected-category="selectedCategory"
        :category-counts="categoryCounts"
        @select="handleSelectCategory"
        @manage="handleManageCategories"
        @drop="handleCategoryDrop"
      />
    </div>

    <!-- View Mode Toggle -->
    <div class="toolbar">
      <div class="toolbar-left">
        <span class="app-count">{{ filteredApps.length }} 个应用</span>
      </div>
      <div class="toolbar-right">
        <n-button-group>
          <n-button
            :type="viewMode === 'grid' ? 'primary' : 'default'"
            @click="viewMode = 'grid'"
            title="网格视图"
          >
            <template #icon>
              <n-icon><GridOutline /></n-icon>
            </template>
          </n-button>
          <n-button
            :type="viewMode === 'list' ? 'primary' : 'default'"
            @click="viewMode = 'list'"
            title="列表视图"
          >
            <template #icon>
              <n-icon><ListOutline /></n-icon>
            </template>
          </n-button>
        </n-button-group>
      </div>
    </div>

    <!-- Apps Grid/List -->
    <div class="apps-container custom-scrollbar">
      <!-- 置顶应用 -->
      <div v-if="pinnedApps.length > 0" class="apps-section">
        <div class="section-header">
          <n-icon size="16" color="#6366f1"><Pin /></n-icon>
          <span>置顶应用</span>
        </div>
        <div :class="['apps-grid', viewMode]">
          <AppCard
            v-for="app in pinnedApps"
            :key="app.id"
            :app="app"
            @launch="handleLaunch"
            @pin="handlePin"
            @edit="handleEdit"
            @delete="handleDelete"
          />
        </div>
      </div>

      <!-- 所有应用 -->
      <div v-if="unpinnedApps.length > 0" class="apps-section">
        <div class="section-header">
          <n-icon size="16" color="#94a3b8"><AppsOutline /></n-icon>
          <span>{{ selectedCategory === 'all' ? '所有应用' : currentCategoryName }}</span>
        </div>
        <div :class="['apps-grid', viewMode]">
          <AppCard
            v-for="app in unpinnedApps"
            :key="app.id"
            :app="app"
            @launch="handleLaunch"
            @pin="handlePin"
            @edit="handleEdit"
            @delete="handleDelete"
          />
        </div>
      </div>

      <!-- 空状态 -->
      <n-empty
        v-if="filteredApps.length === 0"
        description="暂无应用，点击【添加应用】或【扫描应用】开始"
        class="empty-state"
      >
        <template #icon>
          <n-icon size="64" color="#64748b"><AppsOutline /></n-icon>
        </template>
      </n-empty>
    </div>

    <!-- 应用编辑对话框 -->
    <AppEditDialog
      v-model:show="showEditDialog"
      :app="currentApp"
      :categories="categories"
      @submit="handleSaveApp"
      @cancel="handleCancelEdit"
    />

    <!-- 工作流编辑对话框 -->
    <WorkflowEditDialog
      v-model:show="showWorkflowDialog"
      :workflow="currentWorkflow"
      :apps="allApps"
      :categories="categories"
      @submit="handleSaveWorkflow"
      @cancel="handleCancelWorkflow"
    />

    <!-- 全局快速启动弹窗 -->
    <AppQuickLaunchModal
      v-model:show="showQuickLaunch"
      :apps="allApps"
      @launch="handleLaunch"
    />

    <!-- 分类管理对话框 -->
    <CategoryManager
      v-model:show="showCategoryManager"
      :categories="categories"
      @save="handleSaveCategories"
      @cancel="handleCancelCategoryManager"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h, onMounted } from 'vue';
import {
  NSpace,
  NButton,
  NButtonGroup,
  NIcon,
  NDropdown,
  NEmpty,
  useMessage,
  useDialog,
} from 'naive-ui';
import {
  AddOutline,
  ScanOutline,
  RefreshOutline,
  EllipsisHorizontal,
  GridOutline,
  ListOutline,
  AppsOutline,
  Pin,
  RocketOutline,
  SettingsOutline,
  TrashOutline,
} from '@vicons/ionicons5';
import AppCard from '@/components/appLauncher/AppCard.vue';
import AppSearchBar from '@/components/appLauncher/AppSearchBar.vue';
import AppCategoryFilter from '@/components/appLauncher/AppCategoryFilter.vue';
import AppEditDialog from '@/components/appLauncher/AppEditDialog.vue';
import WorkflowEditDialog from '@/components/appLauncher/WorkflowEditDialog.vue';
import AppQuickLaunchModal from '@/components/appLauncher/AppQuickLaunchModal.vue';
import CategoryManager from '@/components/appLauncher/CategoryManager.vue';
import type { AppItem, Category, Workflow } from '@/types/appLauncher';
import { DEFAULT_CATEGORIES } from '@/types/appLauncher';

const message = useMessage();
const dialog = useDialog();

// 状态
const scanning = ref(false);
const refreshingIcons = ref(false);
const searchKeyword = ref('');
const selectedCategory = ref('all');
const viewMode = ref<'grid' | 'list'>('grid');
const showEditDialog = ref(false);
const showWorkflowDialog = ref(false);
const showQuickLaunch = ref(false);
const showCategoryManager = ref(false);
const currentApp = ref<AppItem | null>(null);
const currentWorkflow = ref<Workflow | null>(null);

// 应用数据（从数据库加载）
const allApps = ref<AppItem[]>([]);
const categories = ref<Category[]>(DEFAULT_CATEGORIES.map((cat) => ({ ...cat, createdAt: Date.now() })));
const workflows = ref<Workflow[]>([]);

// 页面加载时从数据库读取应用列表
const loadAppsFromDatabase = async () => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const apps = await invoke('get_all_apps');
    if (Array.isArray(apps)) {
      allApps.value = apps;
    }
  } catch (error) {
    console.error('加载应用列表失败:', error);
  }
};

// 从数据库加载分类
const loadCategoriesFromDatabase = async () => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    let dbCategories = await invoke('get_categories');

    // 确保所有默认分类都存在于数据库中
    const defaultCats = DEFAULT_CATEGORIES.map((cat) => ({ ...cat, createdAt: Date.now() }));
    const dbCategoryIds = new Set(Array.isArray(dbCategories) ? dbCategories.map(c => c.id) : []);

    // 找出缺失的默认分类
    const missingCategories = defaultCats.filter(cat => !dbCategoryIds.has(cat.id));

    // 保存缺失的默认分类到数据库
    if (missingCategories.length > 0) {
      console.log('正在初始化默认分类:', missingCategories.map(c => c.name).join(', '));
      for (const category of missingCategories) {
        try {
          await invoke('save_category', { category });
        } catch (error) {
          console.error('保存默认分类失败:', category.name, error);
        }
      }
      // 重新加载分类
      dbCategories = await invoke('get_categories');
    }

    // 使用数据库中的分类
    if (Array.isArray(dbCategories) && dbCategories.length > 0) {
      categories.value = dbCategories.sort((a, b) => a.sortOrder - b.sortOrder);
    } else {
      // 如果仍然没有分类（极端情况），使用默认分类
      categories.value = defaultCats;
    }
  } catch (error) {
    console.error('加载分类失败:', error);
    // 出错时使用默认分类
    categories.value = DEFAULT_CATEGORIES.map((cat) => ({ ...cat, createdAt: Date.now() }));
  }
};

// 组件挂载时加载数据
onMounted(async () => {
  await Promise.all([
    loadAppsFromDatabase(),
    loadCategoriesFromDatabase(),
  ]);
});

// 计算属性
const filteredApps = computed(() => {
  let apps = allApps.value.filter((app) => !app.isHidden);

  // 按分类筛选
  if (selectedCategory.value !== 'all') {
    apps = apps.filter((app) => app.category === selectedCategory.value);
  }

  // 按搜索关键词筛选
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase();
    apps = apps.filter((app) => app.name.toLowerCase().includes(keyword));
  }

  // 按启动次数排序
  return apps.sort((a, b) => {
    // 置顶应用优先
    if (a.isPinned && !b.isPinned) return -1;
    if (!a.isPinned && b.isPinned) return 1;
    // 按启动次数排序
    return b.launchCount - a.launchCount;
  });
});

const pinnedApps = computed(() => {
  return filteredApps.value.filter((app) => app.isPinned);
});

const unpinnedApps = computed(() => {
  return filteredApps.value.filter((app) => !app.isPinned);
});

const categoryCounts = computed(() => {
  const counts: Record<string, number> = { all: allApps.value.length };
  allApps.value.forEach((app) => {
    if (!app.isHidden) {
      counts[app.category] = (counts[app.category] || 0) + 1;
    }
  });
  return counts;
});

const currentCategoryName = computed(() => {
  return categories.value.find((cat) => cat.id === selectedCategory.value)?.name || '其他';
});

// 更多操作菜单
const moreOptions = computed(() => [
  {
    label: '工作流管理',
    key: 'workflows',
    icon: () => h(NIcon, null, { default: () => h(RocketOutline) }),
  },
  {
    label: '分类管理',
    key: 'categories',
    icon: () => h(NIcon, null, { default: () => h(SettingsOutline) }),
  },
  {
    type: 'divider',
    key: 'divider',
  },
  {
    label: '清空所有应用',
    key: 'clear-all',
    icon: () => h(NIcon, null, { default: () => h(TrashOutline) }),
  },
]);

// 事件处理
const handleAddApp = () => {
  currentApp.value = null;
  showEditDialog.value = true;
};

const handleScanApps = async () => {
  scanning.value = true;
  try {
    // 调用Tauri API选择目录
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择要扫描的目录',
    });

    if (selected && typeof selected === 'string') {
      message.info('开始扫描目录: ' + selected);

      // 调用后端API扫描目录下的所有exe文件
      const { invoke } = await import('@tauri-apps/api/core');
      const scannedApps = await invoke('scan_installed_apps', { path: selected });

      if (Array.isArray(scannedApps) && scannedApps.length > 0) {
        // 将扫描到的应用添加到列表中
        let addedCount = 0;
        for (const app of scannedApps) {
          // 检查是否已存在（避免重复）
          const exists = allApps.value.some((a) => a.path === app.path);
          if (!exists) {
            const newApp: AppItem = {
              id: Date.now().toString() + Math.random(),
              name: app.name || '未知应用',
              path: app.path,
              icon: app.icon,
              category: app.category || 'other',
              tags: app.tags || [],
              launchArgs: app.launch_args || '',
              isPinned: false,
              isHidden: false,
              launchCount: 0,
              createdAt: Date.now(),
              updatedAt: Date.now(),
            };

            try {
              // 保存到数据库
              await invoke('add_app', { app: newApp });

              // 更新本地状态
              allApps.value.push(newApp);
              addedCount++;
            } catch (error) {
              console.error('保存应用失败:', app.name, error);
            }
          }
        }
        message.success(`扫描完成！找到 ${scannedApps.length} 个应用，添加了 ${addedCount} 个新应用`);
      } else {
        message.warning('未找到任何应用');
      }
    }
  } catch (error) {
    message.error('扫描失败: ' + error);
    console.error('扫描失败:', error);
  } finally {
    scanning.value = false;
  }
};

const handleRefreshIcons = async () => {
  refreshingIcons.value = true;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const updatedCount = await invoke('refresh_all_icons') as number;

    if (updatedCount > 0) {
      message.success(`成功刷新 ${updatedCount} 个应用的图标`);
      // 重新加载应用列表以显示新图标
      await loadAppsFromDatabase();
    } else {
      message.info('所有应用的图标都是最新的');
    }
  } catch (error) {
    message.error('刷新图标失败: ' + error);
    console.error('刷新图标失败:', error);
  } finally {
    refreshingIcons.value = false;
  }
};

const handleSearch = (keyword: string) => {
  searchKeyword.value = keyword;
};

const handleClearSearch = () => {
  searchKeyword.value = '';
};

const handleSelectCategory = (categoryId: string) => {
  selectedCategory.value = categoryId;
};

const handleLaunch = async (appIdOrApp: string | AppItem) => {
  try {
    // 如果传入的是 ID，查找完整的应用对象
    let app: AppItem | undefined;
    let appId: string;

    if (typeof appIdOrApp === 'string') {
      appId = appIdOrApp;
      app = allApps.value.find((a) => a.id === appIdOrApp);
      if (!app) {
        message.error('未找到该应用');
        console.error('App not found with id:', appIdOrApp);
        return;
      }
    } else {
      app = appIdOrApp;
      appId = app.id;
    }

    console.log('尝试启动应用:', app);

    if (!app.path) {
      message.error(`应用路径不存在。应用名称: ${app.name}`);
      console.error('app.path is empty:', app);
      return;
    }

    console.log('应用路径:', app.path);

    // 调用后端命令启动应用
    const { invoke } = await import('@tauri-apps/api/core');
    const result = await invoke('launch_app', { appId });

    console.log('启动结果:', result);

    message.success(`启动 ${app.name}`);

    // 更新本地状态（后端已经更新了数据库中的启动次数）
    app.launchCount++;
    app.lastLaunchedAt = Date.now();
  } catch (error) {
    const appName = typeof appIdOrApp === 'string'
      ? allApps.value.find((a) => a.id === appIdOrApp)?.name || '未知应用'
      : appIdOrApp?.name || '未知应用';

    message.error(`启动 ${appName} 失败: ${error}`);
    console.error('启动应用失败:', {
      error,
      appIdOrApp,
    });
  }
};

const handlePin = async (appId: string) => {
  const app = allApps.value.find((a) => a.id === appId);
  if (app) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');

      // 切换置顶状态
      await invoke('toggle_app_pin', { appId });

      // 更新本地状态
      app.isPinned = !app.isPinned;
      message.success(app.isPinned ? '已置顶' : '已取消置顶');
    } catch (error) {
      message.error('操作失败: ' + error);
      console.error('切换置顶失败:', error);
    }
  }
};

const handleEdit = (app: AppItem) => {
  currentApp.value = app;
  showEditDialog.value = true;
};

const handleDelete = (appId: string) => {
  const app = allApps.value.find((a) => a.id === appId);
  if (app) {
    dialog.warning({
      title: '删除应用',
      content: `确定要删除 "${app.name}" 吗？`,
      positiveText: '删除',
      negativeText: '取消',
      onPositiveClick: async () => {
        try {
          const { invoke } = await import('@tauri-apps/api/core');

          // 从数据库删除
          await invoke('delete_app', { appId });

          // 更新本地状态
          allApps.value = allApps.value.filter((a) => a.id !== appId);
          message.success('删除成功');
        } catch (error) {
          message.error('删除失败: ' + error);
          console.error('删除应用失败:', error);
        }
      },
    });
  }
};

const handleSaveApp = async (data: any) => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');

    if (currentApp.value) {
      // 编辑现有应用
      const updatedApp: AppItem = {
        ...currentApp.value,
        name: data.name,
        path: data.path,
        category: data.category,
        tags: data.tags || [],
        launchArgs: data.launchArgs || '',
        isPinned: data.isPinned,
        icon: data.icon,
        updatedAt: Date.now(),
      };

      // 保存到数据库
      await invoke('update_app', { app: updatedApp });

      // 更新本地状态
      const index = allApps.value.findIndex((app) => app.id === currentApp.value!.id);
      if (index !== -1) {
        allApps.value[index] = updatedApp;
      }

      message.success('应用已更新');
    } else {
      // 添加新应用
      const newApp: AppItem = {
        id: Date.now().toString(),
        name: data.name,
        path: data.path,
        category: data.category,
        tags: data.tags || [],
        launchArgs: data.launchArgs || '',
        icon: data.icon,
        isPinned: data.isPinned || false,
        isHidden: false,
        launchCount: 0,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };

      // 保存到数据库
      await invoke('add_app', { app: newApp });

      // 更新本地状态
      allApps.value.push(newApp);

      message.success('应用已添加');
    }

    showEditDialog.value = false;
    currentApp.value = null;
  } catch (error) {
    message.error('保存失败: ' + error);
    console.error('保存应用失败:', error);
  }
};

const handleCancelEdit = () => {
  currentApp.value = null;
};

const handleManageCategories = () => {
  showCategoryManager.value = true;
};

// 全局拖拽处理器
const handleGlobalDragOver = (e: DragEvent) => {
  console.log('🔍 Global dragover:', {
    target: (e.target as HTMLElement)?.className,
    tagName: (e.target as HTMLElement)?.tagName,
  });
  // e.preventDefault() 已经由 .prevent 修饰符调用
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'move';
  }
};

const handleGlobalDragEnter = (e: DragEvent) => {
  console.log('🟢 Global dragenter:', (e.target as HTMLElement)?.className);
  // e.preventDefault() 已经由 .prevent 修饰符调用
};

// 处理拖放到分类
const handleCategoryDrop = async (categoryId: string, appData: any) => {
  console.log('=== AppLauncher handleCategoryDrop ===');
  console.log('Target Category ID:', categoryId);
  console.log('Dropped App Data:', appData);

  try {
    // 找到被拖拽的应用
    const app = allApps.value.find((a) => a.id === appData.id);
    console.log('Found app in allApps:', !!app);

    if (!app) {
      console.error('App not found! appData.id:', appData.id);
      message.error('未找到该应用');
      return;
    }

    console.log('App current category:', app.category);
    console.log('Target category:', categoryId);

    // 如果分类没变，不做处理
    if (app.category === categoryId) {
      console.log('Category unchanged, skipping');
      message.info('应用已在该分类中');
      return;
    }

    // 更新应用分类
    const updatedApp = { ...app, category: categoryId };
    console.log('Updated app:', updatedApp);

    // 调用后端更新
    const { invoke } = await import('@tauri-apps/api/core');
    console.log('Calling backend update_app...');
    await invoke('update_app', { app: updatedApp });
    console.log('Backend update successful');

    // 更新本地状态
    const index = allApps.value.findIndex((a) => a.id === app.id);
    if (index !== -1) {
      allApps.value[index] = updatedApp;
      console.log('Local state updated at index:', index);
    }

    // 显示成功消息
    const categoryName = categories.value.find((c) => c.id === categoryId)?.name || categoryId;
    message.success(`已将「${app.name}」移动到「${categoryName}」分类`);
    console.log('=== Drop handler complete ===');
  } catch (error) {
    message.error('更新分类失败: ' + error);
    console.error('=== Drop handler error ===', error);
  }
};

const handleMoreAction = (key: string) => {
  switch (key) {
    case 'workflows':
      showWorkflowDialog.value = true;
      break;
    case 'categories':
      handleManageCategories();
      break;
    case 'clear-all':
      handleClearAllApps();
      break;
  }
};

const handleClearAllApps = () => {
  dialog.warning({
    title: '清空所有应用',
    content: `确定要删除所有 ${allApps.value.length} 个应用吗？此操作不可恢复。`,
    positiveText: '确定删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');

        // 逐个从数据库删除
        for (const app of allApps.value) {
          await invoke('delete_app', { appId: app.id });
        }

        // 清空本地状态
        allApps.value = [];
        message.success('已清空所有应用');
      } catch (error) {
        message.error('清空失败: ' + error);
        console.error('清空应用失败:', error);
      }
    },
  });
};

const handleSaveWorkflow = (data: any) => {
  // TODO: 保存工作流到数据库
  message.success('工作流保存成功');
  showWorkflowDialog.value = false;
  currentWorkflow.value = null;
};

const handleCancelWorkflow = () => {
  currentWorkflow.value = null;
};

const handleSaveCategories = async (newCategories: Category[]) => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');

    // 获取数据库中的现有分类
    const dbCategories = await invoke('get_categories') as Category[];
    const dbCategoryIds = new Set(dbCategories.map(cat => cat.id));

    // 找出要删除的分类（在数据库中但不在新分类列表中）
    const newCategoryIds = new Set(newCategories.map(cat => cat.id));
    const categoriesToDelete = dbCategories.filter(cat => !newCategoryIds.has(cat.id));

    // 删除不再需要的分类
    for (const category of categoriesToDelete) {
      try {
        await invoke('delete_category', { id: category.id });
      } catch (error) {
        console.error('删除分类失败:', category.name, error);
      }
    }

    // 保存或更新所有分类
    for (const category of newCategories) {
      try {
        await invoke('save_category', { category });
      } catch (error) {
        console.error('保存分类失败:', category.name, error);
        message.error(`保存分类"${category.name}"失败`);
      }
    }

    // 更新本地状态
    categories.value = newCategories;
    message.success('分类已保存');
    showCategoryManager.value = false;
  } catch (error) {
    message.error('保存分类失败: ' + error);
    console.error('保存分类失败:', error);
  }
};

const handleCancelCategoryManager = () => {
  showCategoryManager.value = false;
};

// 监听全局快捷键（实际应在App.vue或store中处理）
// const handleGlobalShortcut = () => {
//   showQuickLaunch.value = true;
// };
</script>

<style scoped>
.app-launcher {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 20px;
  padding: 24px;
}

/* Header */
.launcher-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: #e2e8f0;
  margin: 0;
}

/* 主要按钮样式 */
:deep(.primary-button) {
  background-color: #6366f1;
  border-color: #6366f1;
  color: #ffffff;
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2);
  transition: all 0.2s;
}

:deep(.primary-button:hover) {
  background-color: #4f46e5;
  border-color: #4f46e5;
}

:deep(.primary-button:active) {
  background-color: #4338ca;
  border-color: #4338ca;
}

/* Toolbar */
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.app-count {
  font-size: 13px;
  color: #94a3b8;
}

.toolbar-right {
  display: flex;
  gap: 12px;
}

/* Apps Container */
.apps-container {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.apps-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: #cbd5e1;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* Grid Layout */
.apps-grid {
  display: grid;
  gap: 16px;
  animation: fadeIn 0.3s ease;
}

.apps-grid.grid {
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
}

.apps-grid.list {
  grid-template-columns: 1fr;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Empty State */
.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
}

/* Custom Scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(30, 41, 59, 0.3);
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(99, 102, 241, 0.3);
  border-radius: 4px;
  transition: background 0.2s;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(99, 102, 241, 0.5);
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .apps-grid.grid {
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  }
}

@media (max-width: 768px) {
  .app-launcher {
    padding: 16px;
    gap: 16px;
  }

  .launcher-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .apps-grid.grid {
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 12px;
  }

  .toolbar {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
}
</style>
