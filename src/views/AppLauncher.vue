<template>
  <div
    class="app-launcher"
    :class="{ 'is-dragging': draggingApp !== null }"
  >
    <!-- Header -->
    <div class="launcher-header">
      <h2 class="page-title">应用启动器</h2>
      <n-space>
        <n-button type="primary" @click="handleFullScanAndSync" :loading="fullScanning">
          <template #icon>
            <n-icon><SyncOutline /></n-icon>
          </template>
          {{ fullScanProgress || '智能扫描' }}
        </n-button>
        <n-button @click="handleScanApps" :loading="scanning">
          <template #icon>
            <n-icon><ScanOutline /></n-icon>
          </template>
          扫描目录
        </n-button>
        <n-button @click="handleRefreshIcons" :loading="refreshingIcons">
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          刷新图标
        </n-button>
        <n-button
          size="small"
          :loading="aiClassifying"
          :disabled="!aiStore.isEnabled"
          @click="handleAiClassifyApps"
        >
          <template #icon>
            <n-icon><SparklesOutline /></n-icon>
          </template>
          {{ aiClassifyProgress || 'AI分类' }}
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
    <div class="filter-section" :class="{ 'drop-zone-active': draggingApp !== null }">
      <AppCategoryFilter
        :categories="categories"
        :selected-category="selectedCategory"
        :category-counts="categoryCounts"
        :hovered-category-id="hoveredCategoryId"
        :is-dragging="draggingApp !== null"
        @select="handleSelectCategory"
        @manage="handleManageCategories"
      />
    </div>

    <!-- View Mode Toggle -->
    <div class="toolbar">
      <div class="toolbar-left">
        <span class="app-count">{{ filteredApps.length }} 个应用</span>
        <!-- 批量选择模式切换 -->
        <n-button
          size="small"
          :type="batchSelectMode ? 'primary' : 'default'"
          @click="toggleBatchSelectMode"
        >
          <template #icon>
            <n-icon><CheckboxOutline /></n-icon>
          </template>
          {{ batchSelectMode ? '退出批量' : '批量选择' }}
        </n-button>
        <!-- 批量操作按钮 -->
        <template v-if="batchSelectMode">
          <n-button size="small" @click="handleSelectAll">
            {{ isAllSelected ? '取消全选' : '全选' }}
          </n-button>
          <n-button
            size="small"
            type="error"
            :disabled="selectedAppIds.size === 0"
            @click="handleBatchDelete"
          >
            <template #icon>
              <n-icon><TrashOutline /></n-icon>
            </template>
            删除选中 ({{ selectedAppIds.size }})
          </n-button>
        </template>
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

    <!-- Apps Grid/List with Virtual Scrolling -->
    <VirtualGrid
      ref="virtualGridRef"
      :items="unpinnedApps"
      :item-width="viewMode === 'grid' ? 200 : 0"
      :item-height="160"
      :gap="16"
      :buffer="3"
      :class="['apps-container', { 'list-mode': viewMode === 'list' }]"
    >
      <!-- 置顶应用 (不使用虚拟滚动) -->
      <template #pinned>
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
              :is-being-dragged="draggingApp?.id === app.id"
              :batch-select-mode="batchSelectMode"
              :is-selected="selectedAppIds.has(app.id)"
              @launch="handleLaunch"
              @pin="handlePin"
              @edit="handleEdit"
              @delete="handleDelete"
              @mousedown-drag="handleMouseDragStart"
              @toggle-select="handleToggleSelect"
              @show-in-folder="handleShowInFolder"
            />
          </div>
        </div>
      </template>

      <!-- 所有应用区域标题 -->
      <template #header>
        <div class="section-header">
          <n-icon size="16" color="#94a3b8"><AppsOutline /></n-icon>
          <span>{{ selectedCategory === 'all' ? '所有应用' : currentCategoryName }}</span>
        </div>
      </template>

      <!-- 虚拟滚动的应用卡片 -->
      <template #item="{ item: app }">
        <AppCard
          :app="app"
          :is-being-dragged="draggingApp?.id === app.id"
          :batch-select-mode="batchSelectMode"
          :is-selected="selectedAppIds.has(app.id)"
          @launch="handleLaunch"
          @pin="handlePin"
          @edit="handleEdit"
          @delete="handleDelete"
          @mousedown-drag="handleMouseDragStart"
          @toggle-select="handleToggleSelect"
          @show-in-folder="handleShowInFolder"
        />
      </template>

      <!-- 空状态 -->
      <template #empty>
        <n-empty
          v-if="filteredApps.length === 0"
          description="暂无应用，点击【添加应用】或【扫描应用】开始"
          class="empty-state"
        >
          <template #icon>
            <n-icon size="64" color="#64748b"><AppsOutline /></n-icon>
          </template>
        </n-empty>
      </template>
    </VirtualGrid>

    <!-- 应用编辑对话框 -->
    <AppEditDialog
      v-model:show="showEditDialog"
      :app="currentApp"
      :categories="categories"
      @submit="handleSaveApp"
      @save-and-new="handleSaveAndNewApp"
      @cancel="handleCancelEdit"
    />

    <!-- 工作流编辑对话框 -->
    <WorkflowEditDialog
      v-model:show="showWorkflowDialog"
      :workflow="currentWorkflow"
      :apps="allApps"
      :categories="categories"
      :workflows="workflows"
      @submit="handleSaveWorkflow"
      @cancel="handleCancelWorkflow"
      @edit="handleEditWorkflow"
      @delete="handleDeleteWorkflow"
      @launch="handleLaunchWorkflow"
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

    <!-- 启动器设置对话框 -->
    <LauncherSettingsDialog
      v-model:show="showLauncherSettings"
      @saved="handleSettingsSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h, onMounted, onUnmounted } from 'vue';
import { useAiStore } from '@/stores/aiStore';
import { aiApi } from '@/api/aiApi';
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
  SparklesOutline,
  CheckboxOutline,
  SyncOutline,
} from '@vicons/ionicons5';
import AppCard from '@/components/appLauncher/AppCard.vue';
import AppSearchBar from '@/components/appLauncher/AppSearchBar.vue';
import AppCategoryFilter from '@/components/appLauncher/AppCategoryFilter.vue';
import AppEditDialog from '@/components/appLauncher/AppEditDialog.vue';
import WorkflowEditDialog from '@/components/appLauncher/WorkflowEditDialog.vue';
import AppQuickLaunchModal from '@/components/appLauncher/AppQuickLaunchModal.vue';
import CategoryManager from '@/components/appLauncher/CategoryManager.vue';
import LauncherSettingsDialog from '@/components/appLauncher/LauncherSettingsDialog.vue';
import VirtualGrid from '@/components/appLauncher/VirtualGrid.vue';
import type { AppItem, Category, Workflow } from '@/types/appLauncher';
import { DEFAULT_CATEGORIES, ItemType } from '@/types/appLauncher';

const message = useMessage();
const dialog = useDialog();
const aiStore = useAiStore();

// 状态
const scanning = ref(false);
const refreshingIcons = ref(false);
const aiClassifying = ref(false);
const fullScanning = ref(false);
const fullScanProgress = ref('');
const searchKeyword = ref('');
const selectedCategory = ref(localStorage.getItem('appLauncher_selectedCategory') || 'all');
const viewMode = ref<'grid' | 'list'>('grid');
const showEditDialog = ref(false);
const showWorkflowDialog = ref(false);
const showQuickLaunch = ref(false);
const showCategoryManager = ref(false);
const showLauncherSettings = ref(false);
const currentApp = ref<AppItem | null>(null);
const currentWorkflow = ref<Workflow | null>(null);

// 批量选择状态
const batchSelectMode = ref(false);
const selectedAppIds = ref<Set<string>>(new Set());

// 鼠标拖拽状态
const draggingApp = ref<AppItem | null>(null);
const ghostElement = ref<HTMLElement | null>(null);
const hoveredCategoryId = ref<string | null>(null);

// 应用数据（从数据库加载）
const allApps = ref<AppItem[]>([]);
const categories = ref<Category[]>(DEFAULT_CATEGORIES.map((cat) => ({ ...cat, createdAt: Date.now() })));
const workflows = ref<Workflow[]>([]);

// 虚拟滚动组件引用
const virtualGridRef = ref<InstanceType<typeof VirtualGrid> | null>(null);

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

    // 只在数据库完全为空时（首次使用）才初始化默认分类
    // 如果数据库中有分类数据，说明用户已经开始使用，不应该自动添加被删除的分类
    if (!Array.isArray(dbCategories) || dbCategories.length === 0) {
      console.log('首次使用，正在初始化默认分类');
      const defaultCats = DEFAULT_CATEGORIES.map((cat) => ({ ...cat, createdAt: Date.now() }));
      for (const category of defaultCats) {
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
      categories.value = DEFAULT_CATEGORIES.map((cat) => ({ ...cat, createdAt: Date.now() }));
    }
  } catch (error) {
    console.error('加载分类失败:', error);
    // 出错时使用默认分类
    categories.value = DEFAULT_CATEGORIES.map((cat) => ({ ...cat, createdAt: Date.now() }));
  }
};

// 从数据库加载工作流
const loadWorkflowsFromDatabase = async () => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const dbWorkflows = await invoke('get_workflows');
    if (Array.isArray(dbWorkflows)) {
      workflows.value = dbWorkflows;
    }
  } catch (error) {
    console.error('加载工作流失败:', error);
  }
};

// 组件挂载时加载数据
onMounted(async () => {
  await Promise.all([
    loadAppsFromDatabase(),
    loadCategoriesFromDatabase(),
    loadWorkflowsFromDatabase(),
    aiStore.loadConfig(),
  ]);

  // 验证保存的分类是否仍然存在，如果不存在则回退到"全部"
  const savedCategory = selectedCategory.value;
  if (savedCategory !== 'all' && !categories.value.some(cat => cat.id === savedCategory)) {
    selectedCategory.value = 'all';
    localStorage.setItem('appLauncher_selectedCategory', 'all');
  }
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

// 判断是否全选
const isAllSelected = computed(() => {
  return filteredApps.value.length > 0 && selectedAppIds.value.size === filteredApps.value.length;
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
    label: '启动器设置',
    key: 'launcher-settings',
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
              // 保存到数据库，后端会自动提取图标并返回完整数据
              const savedApp = await invoke<AppItem>('add_app', { app: newApp });

              // 更新本地状态（使用后端返回的数据，包含图标）
              allApps.value.push(savedApp);
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

// 智能扫描并同步 - 扫描所有来源（开始菜单+注册表+shell:AppsFolder）并同步到数据库
const handleFullScanAndSync = async () => {
  fullScanning.value = true;
  fullScanProgress.value = '扫描中...';
  try {
    const { invoke } = await import('@tauri-apps/api/core');

    // 调用一键扫描并同步命令
    fullScanProgress.value = '正在扫描系统应用...';
    const result = await invoke<{ added: number; updated: number; removed: number; unchanged: number }>('scan_and_sync_apps');

    // 重新加载应用列表
    await loadAppsFromDatabase();

    // 显示结果
    const parts = [];
    if (result.added > 0) parts.push(`新增 ${result.added} 个`);
    if (result.updated > 0) parts.push(`更新 ${result.updated} 个`);
    if (result.removed > 0) parts.push(`移除 ${result.removed} 个`);

    if (parts.length > 0) {
      message.success(`扫描完成：${parts.join('，')}`);
    } else {
      message.info('扫描完成，应用列表已是最新');
    }
  } catch (error) {
    message.error('扫描失败: ' + error);
    console.error('智能扫描失败:', error);
  } finally {
    fullScanning.value = false;
    fullScanProgress.value = '';
  }
};

const handleSearch = (keyword: string) => {
  searchKeyword.value = keyword;
  // 搜索时滚动到顶部
  virtualGridRef.value?.scrollToTop();
};

const handleClearSearch = () => {
  searchKeyword.value = '';
  virtualGridRef.value?.scrollToTop();
};

const handleSelectCategory = (categoryId: string) => {
  selectedCategory.value = categoryId;
  // 保存选中的分类到本地存储
  localStorage.setItem('appLauncher_selectedCategory', categoryId);
  // 切换分类时滚动到顶部
  virtualGridRef.value?.scrollToTop();
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
        itemType: data.itemType || currentApp.value.itemType,
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
        itemType: data.itemType || ItemType.Application,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };

      // 保存到数据库，后端会自动提取图标并返回完整数据
      const savedApp = await invoke<AppItem>('add_app', { app: newApp });

      // 更新本地状态（使用后端返回的数据，包含图标）
      allApps.value.push(savedApp);

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

// 保存并新增应用
const handleSaveAndNewApp = async (data: any) => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');

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
      itemType: data.itemType || ItemType.Application,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };

    // 保存到数据库，后端会自动提取图标并返回完整数据
    const savedApp = await invoke<AppItem>('add_app', { app: newApp });

    // 更新本地状态（使用后端返回的数据，包含图标）
    allApps.value.push(savedApp);

    // 不关闭对话框，让用户继续添加
  } catch (error) {
    message.error('保存失败: ' + error);
    console.error('保存应用失败:', error);
  }
};

// ========== 批量选择功能 ==========

// 切换批量选择模式
const toggleBatchSelectMode = () => {
  batchSelectMode.value = !batchSelectMode.value;
  if (!batchSelectMode.value) {
    // 退出批量模式时清空选择
    selectedAppIds.value.clear();
  }
};

// 全选/取消全选
const handleSelectAll = () => {
  if (isAllSelected.value) {
    selectedAppIds.value.clear();
  } else {
    selectedAppIds.value = new Set(filteredApps.value.map((app) => app.id));
  }
};

// 切换单个应用的选择状态
const handleToggleSelect = (appId: string) => {
  if (selectedAppIds.value.has(appId)) {
    selectedAppIds.value.delete(appId);
  } else {
    selectedAppIds.value.add(appId);
  }
  // 触发响应式更新
  selectedAppIds.value = new Set(selectedAppIds.value);
};

// 批量删除
const handleBatchDelete = () => {
  const count = selectedAppIds.value.size;
  if (count === 0) return;

  dialog.warning({
    title: '批量删除应用',
    content: `确定要删除选中的 ${count} 个应用吗？此操作不可恢复。`,
    positiveText: '确定删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        let deletedCount = 0;

        for (const appId of selectedAppIds.value) {
          try {
            await invoke('delete_app', { appId });
            deletedCount++;
          } catch (error) {
            console.error('删除应用失败:', appId, error);
          }
        }

        // 更新本地状态
        allApps.value = allApps.value.filter((app) => !selectedAppIds.value.has(app.id));
        selectedAppIds.value.clear();
        batchSelectMode.value = false;

        message.success(`成功删除 ${deletedCount} 个应用`);
      } catch (error) {
        message.error('批量删除失败: ' + error);
        console.error('批量删除失败:', error);
      }
    },
  });
};

// 在文件夹中显示应用
const handleShowInFolder = async (appPath: string) => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('show_in_folder', { path: appPath });
  } catch (error) {
    message.error('打开文件位置失败: ' + error);
    console.error('打开文件位置失败:', error);
  }
};

const handleManageCategories = () => {
  showCategoryManager.value = true;
};

// ========== 鼠标拖拽实现 ==========

// 鼠标按下开始拖拽
const handleMouseDragStart = (app: AppItem, event: MouseEvent, cardElement: HTMLElement) => {
  console.log('📦 Mouse drag started:', app.name);

  // 防止点击事件
  event.preventDefault();
  event.stopPropagation();

  draggingApp.value = app;

  // 创建幽灵元素
  createGhostElement(cardElement, event);

  // 添加全局鼠标事件
  document.addEventListener('mousemove', handleGlobalMouseMove);
  document.addEventListener('mouseup', handleGlobalMouseUp);

  // 添加拖拽样式
  document.body.style.cursor = 'grabbing';
  document.body.style.userSelect = 'none';
};

// 创建拖拽时的幽灵元素
const createGhostElement = (element: HTMLElement, event: MouseEvent) => {
  const ghost = element.cloneNode(true) as HTMLElement;
  ghost.className = 'drag-ghost';
  ghost.style.cssText = `
    position: fixed;
    pointer-events: none;
    z-index: 10000;
    opacity: 0.85;
    transform: scale(0.95) rotate(-2deg);
    box-shadow: 0 20px 60px rgba(99, 102, 241, 0.4);
    border-radius: 16px;
    width: ${element.offsetWidth}px;
    height: ${element.offsetHeight}px;
    left: ${event.clientX - element.offsetWidth / 2}px;
    top: ${event.clientY - element.offsetHeight / 2}px;
    transition: transform 0.1s ease;
  `;

  document.body.appendChild(ghost);
  ghostElement.value = ghost;
};

// 全局鼠标移动
const handleGlobalMouseMove = (event: MouseEvent) => {
  if (!draggingApp.value || !ghostElement.value) return;

  // 更新幽灵元素位置
  ghostElement.value.style.left = `${event.clientX - ghostElement.value.offsetWidth / 2}px`;
  ghostElement.value.style.top = `${event.clientY - ghostElement.value.offsetHeight / 2}px`;

  // 检测鼠标是否在分类标签上
  checkCategoryHover(event);
};

// 检测分类悬停
const checkCategoryHover = (event: MouseEvent) => {
  const categoryTabs = document.querySelectorAll('.category-tab');
  let foundCategory: string | null = null;

  categoryTabs.forEach((tab) => {
    const rect = tab.getBoundingClientRect();
    if (
      event.clientX >= rect.left &&
      event.clientX <= rect.right &&
      event.clientY >= rect.top &&
      event.clientY <= rect.bottom
    ) {
      foundCategory = tab.getAttribute('data-category-id');
    }
  });

  hoveredCategoryId.value = foundCategory;
};

// 全局鼠标释放
const handleGlobalMouseUp = async (event: MouseEvent) => {
  console.log('📦 Mouse drag ended, hovered category:', hoveredCategoryId.value);

  if (draggingApp.value && hoveredCategoryId.value) {
    // 执行分类更新
    await updateAppCategory(draggingApp.value, hoveredCategoryId.value);
  }

  // 清理拖拽状态
  cleanupDrag();
};

// 更新应用分类
const updateAppCategory = async (app: AppItem, categoryId: string) => {
  if (categoryId === 'all') {
    message.info('无法拖拽到"全部"分类');
    return;
  }

  if (app.category === categoryId) {
    message.info('应用已在该分类中');
    return;
  }

  try {
    const updatedApp = { ...app, category: categoryId };
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('update_app', { app: updatedApp });

    // 更新本地状态
    const index = allApps.value.findIndex((a) => a.id === app.id);
    if (index !== -1) {
      allApps.value[index] = updatedApp;
    }

    const categoryName = categories.value.find((c) => c.id === categoryId)?.name || categoryId;
    message.success(`已将「${app.name}」移动到「${categoryName}」分类`);
  } catch (error) {
    message.error('更新分类失败: ' + error);
    console.error('更新分类失败:', error);
  }
};

// 清理拖拽状态
const cleanupDrag = () => {
  draggingApp.value = null;
  hoveredCategoryId.value = null;

  if (ghostElement.value) {
    ghostElement.value.remove();
    ghostElement.value = null;
  }

  document.removeEventListener('mousemove', handleGlobalMouseMove);
  document.removeEventListener('mouseup', handleGlobalMouseUp);

  document.body.style.cursor = '';
  document.body.style.userSelect = '';
};

// 组件卸载时清理
onUnmounted(() => {
  cleanupDrag();
});

// AI 分类应用
const aiClassifyProgress = ref('');
const AI_CLASSIFY_BATCH_SIZE = 10; // 每批处理的应用数量，可以根据需要调整

const handleAiClassifyApps = async () => {
  if (!aiStore.isEnabled) {
    message.warning('AI 功能未启用，请先在设置中配置 AI');
    return;
  }

  if (allApps.value.length === 0) {
    message.warning('暂无应用可分类');
    return;
  }

  // 只处理未分类的应用（category 为空、undefined 或 'other'）
  const unclassifiedApps = allApps.value.filter(
    (app) => !app.category || app.category === '' || app.category === 'other'
  );

  if (unclassifiedApps.length === 0) {
    message.info('所有应用都已分类，无需重新分类');
    return;
  }

  aiClassifying.value = true;
  const totalApps = unclassifiedApps.length;
  let processedCount = 0;
  let updatedCount = 0;

  try {
    const { invoke } = await import('@tauri-apps/api/core');

    // 获取分类列表（排除"全部"和"其他"）
    const categoryNames = categories.value
      .filter((cat) => cat.id !== 'all' && cat.id !== 'other')
      .map((cat) => cat.name);

    // 分批处理
    for (let i = 0; i < unclassifiedApps.length; i += AI_CLASSIFY_BATCH_SIZE) {
      const batch = unclassifiedApps.slice(i, i + AI_CLASSIFY_BATCH_SIZE);
      const batchNum = Math.floor(i / AI_CLASSIFY_BATCH_SIZE) + 1;
      const totalBatches = Math.ceil(unclassifiedApps.length / AI_CLASSIFY_BATCH_SIZE);

      aiClassifyProgress.value = `正在处理第 ${batchNum}/${totalBatches} 批 (剩余 ${totalApps - processedCount} 个)...`;

      // 准备当前批次的应用数据
      const batchApps = batch.map((app) => ({
        id: app.id,
        name: app.name,
        path: app.path,
      }));

      // 调用 AI 分类当前批次
      const results = await aiApi.classifyApps(batchApps, categoryNames);

      // 更新当前批次的应用分类
      for (const result of results) {
        const app = allApps.value.find((a) => a.id === result.appId);
        if (app) {
          const category = categories.value.find((cat) => cat.name === result.category);
          if (category && category.id !== 'other') {
            app.category = category.id;
            if (result.tags && result.tags.length > 0) {
              app.tags = result.tags;
            }
            // 保存到数据库
            await invoke('update_app', { app });
            updatedCount++;
          }
        }
        processedCount++;
      }

      aiClassifyProgress.value = `已完成 ${processedCount}/${totalApps} 个应用`;
    }

    message.success(`AI 分类完成，已更新 ${updatedCount} 个应用`);
  } catch (error) {
    console.error('AI 分类失败:', error);
    message.error(`AI 分类失败 (已处理 ${processedCount}/${totalApps}): ` + (error as Error).message);
  } finally {
    aiClassifying.value = false;
    aiClassifyProgress.value = '';
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
    case 'launcher-settings':
      showLauncherSettings.value = true;
      break;
    case 'clear-all':
      handleClearAllApps();
      break;
  }
};

const handleSettingsSaved = () => {
  message.success('设置已保存,下次扫描时将生效');
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

const handleSaveWorkflow = async (data: any) => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const now = Date.now();
    const workflow: Workflow = {
      id: currentWorkflow.value?.id || `workflow_${now}`,
      name: data.name,
      appIds: data.appIds,
      launchDelay: data.launchDelay || 500,
      createdAt: currentWorkflow.value?.createdAt || now,
      updatedAt: now,
    };

    await invoke('save_workflow', { workflow });
    await loadWorkflowsFromDatabase();
    message.success('工作流保存成功');
    showWorkflowDialog.value = false;
    currentWorkflow.value = null;
  } catch (error) {
    message.error('保存工作流失败: ' + error);
    console.error('保存工作流失败:', error);
  }
};

const handleCancelWorkflow = () => {
  currentWorkflow.value = null;
};

const handleEditWorkflow = (workflow: Workflow) => {
  currentWorkflow.value = workflow;
};

const handleDeleteWorkflow = async (workflowId: string) => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('delete_workflow', { workflowId });
    await loadWorkflowsFromDatabase();
    message.success('工作流已删除');
  } catch (error) {
    message.error('删除工作流失败: ' + error);
    console.error('删除工作流失败:', error);
  }
};

const handleLaunchWorkflow = async (workflowId: string) => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('launch_workflow', { workflowId });
    message.success('工作流启动成功');
    // 重新加载应用列表以更新启动次数
    await loadAppsFromDatabase();
  } catch (error) {
    message.error('启动工作流失败: ' + error);
    console.error('启动工作流失败:', error);
  }
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

/* Apps Container (VirtualGrid) */
.apps-container {
  flex: 1;
  min-height: 0; /* 重要：允许flex子项收缩 */
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

/* 拖拽状态样式 */
.app-launcher.is-dragging .filter-section {
  position: relative;
  z-index: 100;
}

.filter-section.drop-zone-active {
  padding: 8px;
  margin: -8px;
  background: rgba(99, 102, 241, 0.05);
  border-radius: 16px;
  border: 2px dashed rgba(99, 102, 241, 0.3);
  animation: dropZonePulse 1.5s ease-in-out infinite;
}

@keyframes dropZonePulse {
  0%, 100% {
    border-color: rgba(99, 102, 241, 0.3);
    background: rgba(99, 102, 241, 0.05);
  }
  50% {
    border-color: rgba(99, 102, 241, 0.6);
    background: rgba(99, 102, 241, 0.1);
  }
}

.app-launcher.is-dragging .apps-container {
  opacity: 0.7;
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
