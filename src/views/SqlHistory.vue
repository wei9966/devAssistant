<template>
  <div class="sql-history">
    <div class="history-header">
      <h2 class="history-title">SQL 执行记录</h2>
      <div class="header-actions">
        <!-- 视图切换按钮组 -->
        <n-button-group>
          <n-button
            size="small"
            :type="currentView === 'list' ? 'primary' : 'default'"
            @click="currentView = 'list'"
          >
            <template #icon><n-icon :component="ListIcon" /></template>
            列表
          </n-button>
          <n-button
            size="small"
            :type="currentView === 'template' ? 'primary' : 'default'"
            @click="currentView = 'template'"
          >
            <template #icon><n-icon :component="GridIcon" /></template>
            整合
          </n-button>
        </n-button-group>

        <div class="search-box" v-if="currentView === 'list'">
          <n-icon class="search-icon" :component="SearchIcon" />
          <input
            v-model="searchKeyword"
            type="text"
            placeholder="搜索历史记录..."
            class="search-input"
          />
        </div>
        <n-select
          v-if="currentView === 'list'"
          v-model:value="selectedType"
          size="small"
          :options="typeOptions"
          style="width: 120px"
          placeholder="SQL类型"
        />
        <n-select
          v-if="currentView === 'list'"
          v-model:value="selectedCategory"
          size="small"
          :options="categoryOptions"
          style="width: 120px"
          placeholder="分类"
        />
        <n-button
          v-if="currentView === 'list'"
          size="small"
          :class="showFavorites ? 'primary-button' : ''"
          @click="showFavorites = !showFavorites"
        >
          {{ showFavorites ? '仅收藏' : '全部' }}
        </n-button>
        <n-button
          v-if="currentView === 'list'"
          size="small"
          @click="goToAiSettings"
          :type="aiStore.isEnabled ? 'success' : 'default'"
        >
          {{ aiStore.isEnabled ? 'AI已配置' : 'AI配置' }}
        </n-button>
        <n-button
          v-if="currentView === 'list'"
          size="small"
          :loading="sqlStore.aiClassifying"
          :disabled="!aiStore.isEnabled || sqlStore.uncategorizedCount === 0"
          @click="handleAiClassify"
        >
          AI分类 ({{ sqlStore.uncategorizedCount }})
        </n-button>
        <n-button
          v-if="currentView === 'list'"
          size="small"
          @click="showCategoryModal = true"
        >
          分类管理
        </n-button>
        <n-button
          v-if="currentView === 'list'"
          size="small"
          @click="currentView = 'template'"
        >
          <template #icon><n-icon :component="FlashIcon" /></template>
          智能整合
        </n-button>
        <n-button
          v-if="currentView === 'list'"
          size="small"
          class="primary-button"
          @click="showAddModal = true"
        >
          新增SQL
        </n-button>
      </div>
    </div>

    <!-- 列表视图 -->
    <div v-if="currentView === 'list'" class="table-container">
      <div class="table-wrapper">
        <table class="sql-table">
          <thead class="table-head">
            <tr>
              <th class="col-status">状态</th>
              <th class="col-name">名称</th>
              <th class="col-sql">SQL 语句</th>
              <th class="col-category">分类</th>
              <th class="col-time">时间</th>
              <th class="col-actions">操作</th>
            </tr>
          </thead>
          <tbody class="table-body">
            <tr
              v-for="sql in filteredSqls"
              :key="sql.id"
              class="table-row"
            >
              <td class="col-status">
                <n-icon
                  :component="getStatusIcon(sql.sqlType)"
                  :color="getStatusColor(sql.sqlType)"
                  size="18"
                />
              </td>
              <td class="col-name">
                <span
                  v-if="sql.name"
                  class="sql-name"
                  @click="handleEditSql(sql)"
                  title="点击编辑"
                >
                  {{ sql.name }}
                </span>
                <span
                  v-else
                  class="sql-name-empty"
                  @click="handleEditSql(sql)"
                  title="点击添加名称"
                >
                  未命名
                </span>
              </td>
              <td class="col-sql">
                <div class="sql-cell">
                  <code class="sql-code">
                    <span class="sql-keyword">{{ getFirstKeyword(sql.sqlText) }}</span>
                    <span class="sql-rest">{{ getRestOfSql(sql.sqlText) }}</span>
                  </code>
                  <n-button
                    v-if="isSqlTruncated(sql.sqlText)"
                    text
                    size="tiny"
                    @click="handleShowFullSql(sql.sqlText)"
                    class="expand-btn"
                    title="查看完整SQL"
                  >
                    <template #icon>
                      <n-icon :component="ExpandIcon" size="14" />
                    </template>
                  </n-button>
                </div>
              </td>
              <td class="col-category">
                <div class="category-tags" @click="handleEditSql(sql)">
                  <template v-if="sql.categories && sql.categories.length > 0">
                    <n-tag
                      v-for="cat in sql.categories.slice(0, 3)"
                      :key="cat.id"
                      size="small"
                      :style="{ backgroundColor: cat.color || '#6366f1', color: '#fff' }"
                      class="category-tag"
                    >
                      {{ cat.name }}
                    </n-tag>
                    <span v-if="sql.categories.length > 3" class="more-tags">
                      +{{ sql.categories.length - 3 }}
                    </span>
                  </template>
                  <span
                    v-else
                    class="category-empty"
                    title="点击分类"
                  >
                    未分类
                  </span>
                </div>
              </td>
              <td class="col-time">
                <span class="time-text">{{ formatTime(sql.executedAt) }}</span>
              </td>
              <td class="col-actions">
                <div class="action-buttons">
                  <n-button
                    text
                    size="small"
                    @click="handleCopy(sql.sqlText)"
                    class="action-btn"
                    title="复制"
                  >
                    <template #icon>
                      <n-icon :component="CopyIcon" />
                    </template>
                  </n-button>
                  <n-button
                    text
                    size="small"
                    @click="handleEditSql(sql)"
                    class="action-btn"
                    title="编辑"
                  >
                    <template #icon>
                      <n-icon :component="EditIcon" />
                    </template>
                  </n-button>
                  <n-button
                    text
                    size="small"
                    @click="handleToggleFavorite(sql.id)"
                    class="action-btn"
                    :title="sql.isFavorite ? '取消收藏' : '收藏'"
                  >
                    <template #icon>
                      <n-icon :component="sql.isFavorite ? StarIcon : StarOutlineIcon" />
                    </template>
                  </n-button>
                  <n-button
                    text
                    size="small"
                    @click="handleDelete(sql.id)"
                    class="action-btn"
                    title="删除"
                  >
                    <template #icon>
                      <n-icon :component="TrashIcon" />
                    </template>
                  </n-button>
                </div>
              </td>
            </tr>
            <tr v-if="filteredSqls.length === 0">
              <td colspan="6" class="empty-row">
                <n-empty description="暂无SQL记录" />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="table-footer">
        <span class="footer-text">
          共显示 {{ filteredSqls.length }} 条记录
          <span v-if="searchKeyword"> · 搜索关键词: "{{ searchKeyword }}"</span>
        </span>
      </div>
    </div>

    <!-- 整合视图 -->
    <div v-else-if="currentView === 'template'" class="template-view-container">
      <SqlTemplateView @copy-sql="handleCopySql" />
    </div>

    <!-- 新增SQL对话框 -->
    <n-modal
      v-model:show="showAddModal"
      preset="dialog"
      title="新增SQL"
      positive-text="保存"
      negative-text="取消"
      :positive-button-props="{ disabled: !isFormValid }"
      @positive-click="handleAddSql"
    >
      <n-form
        ref="formRef"
        :model="formData"
        :rules="formRules"
        label-placement="top"
      >
        <n-form-item label="SQL语句" path="sqlText">
          <n-input
            v-model:value="formData.sqlText"
            type="textarea"
            placeholder="请输入SQL语句"
            :autosize="{
              minRows: 6,
              maxRows: 15
            }"
          />
        </n-form-item>
        <n-form-item label="来源标记" path="source">
          <n-input
            v-model:value="formData.source"
            placeholder="例如：手动添加、导入等（可选）"
          />
        </n-form-item>
      </n-form>
    </n-modal>

    <!-- 查看完整SQL对话框 -->
    <n-modal
      v-model:show="showSqlModal"
      preset="dialog"
      title="完整SQL语句"
      positive-text="复制"
      negative-text="关闭"
      style="width: 800px;"
      @positive-click="handleCopyFullSql"
    >
      <div class="full-sql-container">
        <code class="full-sql-code">{{ fullSqlText }}</code>
      </div>
    </n-modal>

    <!-- 编辑SQL对话框 -->
    <n-modal
      v-model:show="showEditModal"
      preset="dialog"
      title="编辑 SQL 信息"
      positive-text="保存"
      negative-text="取消"
      style="width: 600px;"
      @positive-click="handleSaveEdit"
    >
      <n-form
        ref="editFormRef"
        :model="editForm"
        label-placement="top"
      >
        <n-form-item label="SQL 名称" path="name">
          <n-input
            v-model:value="editForm.name"
            placeholder="为这条 SQL 取个名字"
          />
        </n-form-item>
        <n-form-item label="分类标签（可多选）" path="categoryIds">
          <n-select
            v-model:value="editForm.categoryIds"
            :options="editCategoryOptions"
            placeholder="选择分类标签"
            multiple
            clearable
          />
        </n-form-item>
        <n-form-item label="SQL 语句">
          <div class="full-sql-container" style="max-height: 200px;">
            <code class="full-sql-code">{{ editForm.sqlText }}</code>
          </div>
        </n-form-item>
      </n-form>
    </n-modal>

    <!-- 分类管理对话框 -->
    <n-modal
      v-model:show="showCategoryModal"
      preset="dialog"
      title="分类管理"
      style="width: 800px;"
    >
      <div class="category-manager">
        <div class="category-add-form">
          <n-input
            v-model:value="newCategory.name"
            placeholder="分类名称"
            style="width: 100px;"
          />
          <n-input
            v-model:value="newCategory.description"
            placeholder="描述（可选）"
            style="width: 120px;"
          />
          <n-color-picker
            v-model:value="newCategory.color"
            :swatches="colorSwatches"
            style="width: 80px;"
          />
          <n-input
            v-model:value="newCategory.aiPrompt"
            placeholder="AI 识别规则提示词（可选）"
            style="flex: 1;"
          />
          <n-button type="primary" @click="handleAddCategory">添加</n-button>
        </div>
        <div class="category-list">
          <div
            v-for="cat in sqlStore.categories"
            :key="cat.id"
            class="category-item"
          >
            <n-tag
              :style="{ backgroundColor: cat.color || '#6366f1', color: '#fff' }"
              size="small"
            >
              {{ cat.name }}
            </n-tag>
            <span class="category-desc">{{ cat.description || '-' }}</span>
            <span class="category-prompt" :title="cat.aiPrompt">
              {{ cat.aiPrompt ? cat.aiPrompt.substring(0, 30) + '...' : '无规则' }}
            </span>
            <n-tag v-if="cat.isSystem" size="tiny" type="info">系统</n-tag>
            <div class="category-actions">
              <n-button
                text
                size="tiny"
                @click="handleEditCategory(cat)"
              >
                编辑
              </n-button>
              <n-button
                text
                size="tiny"
                type="error"
                @click="handleDeleteCategory(cat.id)"
                :disabled="cat.isSystem"
              >
                删除
              </n-button>
            </div>
          </div>
        </div>
      </div>
    </n-modal>

    <!-- 编辑分类对话框 -->
    <n-modal
      v-model:show="showEditCategoryModal"
      preset="dialog"
      title="编辑分类"
      positive-text="保存"
      negative-text="取消"
      style="width: 500px;"
      @positive-click="handleSaveCategoryEdit"
    >
      <n-form
        :model="editCategoryForm"
        label-placement="top"
      >
        <n-form-item label="分类名称">
          <n-input
            v-model:value="editCategoryForm.name"
            placeholder="请输入分类名称"
          />
        </n-form-item>
        <n-form-item label="描述">
          <n-input
            v-model:value="editCategoryForm.description"
            placeholder="分类描述（可选）"
          />
        </n-form-item>
        <n-form-item label="颜色">
          <n-color-picker
            v-model:value="editCategoryForm.color"
            :swatches="colorSwatches"
          />
        </n-form-item>
        <n-form-item label="AI 识别规则提示词">
          <n-input
            v-model:value="editCategoryForm.aiPrompt"
            type="textarea"
            placeholder="例如：识别规则：表名和字段名使用小写字母和下划线命名（snake_case）"
            :autosize="{ minRows: 3, maxRows: 6 }"
          />
        </n-form-item>
      </n-form>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { NButton, NEmpty, NModal, NForm, NFormItem, NInput, NIcon, NSelect, NTag, NColorPicker, NButtonGroup, useMessage } from 'naive-ui';
import type { FormInst, FormRules } from 'naive-ui';
import { useSqlStore } from '@/stores/sqlStore';
import { useAiStore } from '@/stores/aiStore';
import type { SqlRecord, SqlCategory, AiProvider } from '@/types/sql';
import {
  CheckmarkCircle,
  AlertCircle,
  CloseCircle,
  Copy,
  Star,
  StarOutline,
  Trash,
  Search,
  Expand,
  Create,
  ListOutline,
  GridOutline,
  Flash
} from '@vicons/ionicons5';
import SqlTemplateView from '@/components/sql/SqlTemplateView.vue';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/zh-cn';

dayjs.extend(relativeTime);
dayjs.locale('zh-cn');

const sqlStore = useSqlStore();
const aiStore = useAiStore();
const router = useRouter();
const message = useMessage();
const showFavorites = ref(false);
const showAddModal = ref(false);
const showSqlModal = ref(false);
const showEditModal = ref(false);
const showCategoryModal = ref(false);
const showEditCategoryModal = ref(false);
const fullSqlText = ref('');
const searchKeyword = ref('');
const selectedType = ref<string | null>(null);
const selectedCategory = ref<number | null>(null);
const formRef = ref<FormInst | null>(null);
let autoRefreshTimer: number | null = null;

// 颜色选择预设
const colorSwatches = [
  '#6366f1', '#8b5cf6', '#a855f7', '#d946ef',
  '#ec4899', '#f43f5e', '#ef4444', '#f97316',
  '#f59e0b', '#eab308', '#84cc16', '#22c55e',
  '#10b981', '#14b8a6', '#06b6d4', '#0ea5e9',
  '#3b82f6', '#00758f', '#cc2927', '#64748b'
];

// 新分类表单
const newCategory = ref({
  name: '',
  description: '',
  color: '#6366f1',
  aiPrompt: ''
});

// 编辑分类表单
const editCategoryForm = ref({
  id: 0,
  name: '',
  description: '',
  color: '#6366f1',
  icon: '',
  aiPrompt: ''
});

// 图标组件
const CheckCircleIcon = CheckmarkCircle;
const WarningCircleIcon = AlertCircle;
const ErrorCircleIcon = CloseCircle;
const CopyIcon = Copy;
const StarIcon = Star;
const StarOutlineIcon = StarOutline;
const TrashIcon = Trash;
const SearchIcon = Search;
const ExpandIcon = Expand;
const EditIcon = Create;
const ListIcon = ListOutline;
const GridIcon = GridOutline;
const FlashIcon = Flash;

// 当前视图状态
const currentView = ref<'list' | 'template'>('list');

// 编辑表单（支持多标签）
const editForm = ref({
  id: 0,
  name: '',
  categoryIds: [] as number[],
  sqlText: ''
});

// 编辑分类选项
const editCategoryOptions = computed(() => {
  return sqlStore.categories.map(cat => ({
    label: cat.name,
    value: cat.id
  }));
});

// 分类筛选选项
const categoryOptions = computed(() => [
  { label: '全部分类', value: null },
  { label: '未分类', value: -1 },
  ...sqlStore.categories.map(cat => ({
    label: cat.name,
    value: cat.id
  }))
]);

// 表单数据
const formData = ref({
  sqlText: '',
  source: 'manual'
});

// 表单验证规则
const formRules: FormRules = {
  sqlText: [
    {
      required: true,
      message: '请输入SQL语句',
      trigger: ['blur', 'input']
    },
    {
      min: 5,
      message: 'SQL语句至少需要5个字符',
      trigger: ['blur', 'input']
    }
  ]
};

// SQL类型选项
const typeOptions = computed(() => [
  { label: '全部类型', value: null },
  { label: 'SELECT', value: 'SELECT' },
  { label: 'INSERT', value: 'INSERT' },
  { label: 'UPDATE', value: 'UPDATE' },
  { label: 'DELETE', value: 'DELETE' },
  { label: 'CREATE', value: 'CREATE' },
  { label: 'ALTER', value: 'ALTER' },
  { label: 'DROP', value: 'DROP' },
]);

// 表单是否有效
const isFormValid = computed(() => {
  return formData.value.sqlText.trim().length >= 5;
});

const displaySqls = computed(() => {
  return showFavorites.value ? sqlStore.favoriteSqls : sqlStore.recentSqls;
});

// 过滤后的SQL列表（支持多标签筛选）
const filteredSqls = computed(() => {
  let sqls = displaySqls.value;

  // 按类型筛选
  if (selectedType.value) {
    sqls = sqls.filter(sql => sql.sqlType === selectedType.value);
  }

  // 按分类筛选（多标签）
  if (selectedCategory.value !== null) {
    if (selectedCategory.value === -1) {
      // 未分类
      sqls = sqls.filter(sql => !sql.categories || sql.categories.length === 0);
    } else {
      sqls = sqls.filter(sql =>
        sql.categories?.some(cat => cat.id === selectedCategory.value)
      );
    }
  }

  // 按关键字搜索
  if (searchKeyword.value.trim()) {
    const keyword = searchKeyword.value.toLowerCase();
    sqls = sqls.filter(sql =>
      sql.sqlText.toLowerCase().includes(keyword) ||
      sql.name?.toLowerCase().includes(keyword) ||
      sql.categories?.some(cat => cat.name.toLowerCase().includes(keyword))
    );
  }

  return sqls;
});

onMounted(async () => {
  await sqlStore.loadRecentSqls();
  await sqlStore.loadFavoriteSqls();
  await sqlStore.loadCategories();
  await aiStore.loadConfig();

  // 启动自动刷新，每5秒检查一次新的SQL
  autoRefreshTimer = window.setInterval(async () => {
    await sqlStore.loadRecentSqls();
  }, 5000);
});

onUnmounted(() => {
  // 清理定时器
  if (autoRefreshTimer) {
    clearInterval(autoRefreshTimer);
    autoRefreshTimer = null;
  }
});

function handleCopy(sqlText: string) {
  navigator.clipboard.writeText(sqlText);
  message.success('已复制到剪贴板');
}

async function handleToggleFavorite(sqlId: number | undefined) {
  if (!sqlId) return;
  await sqlStore.toggleFavorite(sqlId);
}

async function handleDelete(sqlId: number | undefined) {
  if (!sqlId) return;
  await sqlStore.deleteSql(sqlId);
  message.success('SQL已删除');
}

async function handleAddSql() {
  if (!formRef.value) return;

  try {
    // 验证表单
    await formRef.value.validate();

    // 保存SQL
    await sqlStore.saveSql(
      formData.value.sqlText.trim(),
      formData.value.source.trim() || 'manual'
    );

    // 显示成功提示
    message.success('SQL添加成功');

    // 关闭对话框
    showAddModal.value = false;

    // 重置表单
    resetForm();
  } catch (error) {
    console.error('添加SQL失败:', error);
    if (error instanceof Error && !error.message.includes('验证')) {
      message.error('添加SQL失败，请重试');
    }
  }
}

function resetForm() {
  formData.value = {
    sqlText: '',
    source: 'manual'
  };
  formRef.value?.restoreValidation();
}

// 格式化时间
function formatTime(time: string | undefined): string {
  if (!time) return '-';
  return dayjs(time).fromNow();
}

// 格式化耗时（模拟数据，实际应从数据库获取）
function formatDuration(sql: any): string {
  // 如果有实际的执行时间字段，使用它
  // 这里模拟一些数据
  const types = ['SELECT', 'UPDATE', 'DELETE', 'INSERT'];
  const type = sql.sqlType?.toUpperCase();

  if (!types.includes(type || '')) {
    return '-';
  }

  // 模拟不同类型的执行时间
  const durations: Record<string, string> = {
    'SELECT': `${Math.floor(Math.random() * 50) + 10}ms`,
    'INSERT': `${Math.floor(Math.random() * 30) + 20}ms`,
    'UPDATE': `${Math.floor(Math.random() * 60) + 30}ms`,
    'DELETE': `${Math.floor(Math.random() * 100) + 50}ms`
  };

  return durations[type || 'SELECT'] || '-';
}

// 获取状态图标
function getStatusIcon(sqlType: string | undefined) {
  const type = sqlType?.toUpperCase();
  if (type === 'DELETE' || type === 'DROP') {
    return ErrorCircleIcon;
  } else if (type === 'UPDATE' || type === 'ALTER') {
    return WarningCircleIcon;
  }
  return CheckCircleIcon;
}

// 获取状态颜色
function getStatusColor(sqlType: string | undefined) {
  const type = sqlType?.toUpperCase();
  if (type === 'DELETE' || type === 'DROP') {
    return '#ef4444'; // red
  } else if (type === 'UPDATE' || type === 'ALTER') {
    return '#f59e0b'; // amber
  }
  return '#10b981'; // emerald
}

// 获取SQL语句的第一个关键字
function getFirstKeyword(sqlText: string | undefined | null): string {
  if (!sqlText) return '';
  const firstWord = sqlText.trim().split(/\s+/)[0];
  return firstWord.toUpperCase();
}

// 获取SQL语句的剩余部分
function getRestOfSql(sqlText: string | undefined | null): string {
  if (!sqlText) return '';
  const trimmed = sqlText.trim();
  const firstSpace = trimmed.indexOf(' ');
  if (firstSpace === -1) return '';

  const rest = trimmed.substring(firstSpace);
  // 截断过长的SQL
  const maxLength = 60;
  if (rest.length > maxLength) {
    return rest.substring(0, maxLength) + '...';
  }
  return rest;
}

// 判断SQL是否被截断
function isSqlTruncated(sqlText: string | undefined | null): boolean {
  if (!sqlText) return false;
  const trimmed = sqlText.trim();
  const firstSpace = trimmed.indexOf(' ');
  if (firstSpace === -1) return false;

  const rest = trimmed.substring(firstSpace);
  return rest.length > 60;
}

// 显示完整SQL
function handleShowFullSql(sqlText: string) {
  fullSqlText.value = sqlText;
  showSqlModal.value = true;
}

// 复制完整SQL
function handleCopyFullSql() {
  navigator.clipboard.writeText(fullSqlText.value);
  message.success('已复制到剪贴板');
}

// 跳转到 AI 设置页面
function goToAiSettings() {
  router.push('/settings?tab=integrations');
}

// AI 自动分类
async function handleAiClassify() {
  if (!aiStore.isEnabled) {
    message.warning('请先在设置页面配置 AI');
    return;
  }
  try {
    const results = await sqlStore.aiClassifySqls(undefined, 20);
    message.success(`成功分类 ${results.length} 条 SQL`);
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    message.error('AI 分类失败: ' + (errorMsg || '未知错误'));
  }
}

// 编辑 SQL（支持多标签）
function handleEditSql(sql: SqlRecord) {
  editForm.value = {
    id: sql.id || 0,
    name: sql.name || '',
    categoryIds: sql.categories?.map(cat => cat.id) || [],
    sqlText: sql.sqlText
  };
  showEditModal.value = true;
}

// 保存编辑（多标签）
async function handleSaveEdit() {
  try {
    await sqlStore.updateSqlNameCategories(
      editForm.value.id,
      editForm.value.name || undefined,
      editForm.value.categoryIds
    );
    message.success('保存成功');
    showEditModal.value = false;
  } catch (error) {
    message.error('保存失败');
    return false;
  }
}

// 添加分类
async function handleAddCategory() {
  if (!newCategory.value.name.trim()) {
    message.error('请输入分类名称');
    return;
  }

  try {
    await sqlStore.addCategory(
      newCategory.value.name.trim(),
      newCategory.value.description || undefined,
      newCategory.value.color,
      undefined,
      newCategory.value.aiPrompt || undefined
    );
    message.success('分类添加成功');
    // 重置表单
    newCategory.value = {
      name: '',
      description: '',
      color: '#6366f1',
      aiPrompt: ''
    };
  } catch (error) {
    message.error('添加分类失败');
  }
}

// 编辑分类
function handleEditCategory(cat: SqlCategory) {
  editCategoryForm.value = {
    id: cat.id || 0,
    name: cat.name,
    description: cat.description || '',
    color: cat.color || '#6366f1',
    icon: cat.icon || '',
    aiPrompt: cat.aiPrompt || ''
  };
  showEditCategoryModal.value = true;
}

// 保存分类编辑
async function handleSaveCategoryEdit() {
  if (!editCategoryForm.value.name.trim()) {
    message.error('分类名称不能为空');
    return false;
  }

  try {
    await sqlStore.updateCategory(
      editCategoryForm.value.id,
      editCategoryForm.value.name.trim(),
      editCategoryForm.value.description || undefined,
      editCategoryForm.value.color,
      editCategoryForm.value.icon || undefined,
      editCategoryForm.value.aiPrompt || undefined
    );
    message.success('分类更新成功');
    showEditCategoryModal.value = false;
  } catch (error) {
    message.error('更新分类失败');
    return false;
  }
}

// 删除分类
async function handleDeleteCategory(categoryId: number | undefined) {
  if (!categoryId) return;

  try {
    await sqlStore.deleteCategory(categoryId);
    message.success('分类删除成功');
  } catch (error) {
    message.error('删除分类失败');
  }
}

// 处理复制SQL事件
function handleCopySql(sql: string) {
  navigator.clipboard.writeText(sql);
  message.success('已复制SQL');
}
</script>

<style scoped>
.sql-history {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 32px;
  background: linear-gradient(to bottom right, #0f172a, #1e293b);
}

/* Header */
.history-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
  gap: 16px;
}

.history-title {
  font-size: 20px;
  font-weight: 600;
  color: #f1f5f9;
  white-space: nowrap;
  flex-shrink: 0;
  letter-spacing: -0.025em;
  margin: 0;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  flex: 1;
  justify-content: flex-end;
  min-width: 0;
}

/* Search Box */
.search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: #64748b;
  font-size: 16px;
  pointer-events: none;
}

.search-input {
  background: #0f172a;
  border: 1px solid #334155;
  color: #cbd5e1;
  padding: 8px 16px 8px 36px;
  border-radius: 8px;
  font-size: 14px;
  width: 256px;
  outline: none;
  transition: all 0.2s;
}

.search-input::placeholder {
  color: #64748b; /* slate-500 */
}

.search-input:focus {
  border-color: rgba(99, 102, 241, 0.5);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

/* Table Container */
.table-container {
  flex: 1;
  background: rgba(15, 23, 42, 0.4);
  border-radius: 20px;
  border: 1px solid rgba(51, 65, 85, 0.5);
  overflow: hidden;
  backdrop-filter: blur(12px);
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

.table-wrapper {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

/* Custom Scrollbar */
.table-wrapper::-webkit-scrollbar {
  width: 8px;
}

.table-wrapper::-webkit-scrollbar-track {
  background: transparent;
}

.table-wrapper::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 4px;
}

.table-wrapper::-webkit-scrollbar-thumb:hover {
  background: #475569;
}

/* Table */
.sql-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}

.table-head {
  background: rgba(15, 23, 42, 0.8);
  color: #94a3b8;
  font-weight: 500;
  position: sticky;
  top: 0;
  z-index: 10;
  backdrop-filter: blur(12px);
  border-bottom: 1px solid #1e293b;
}

.table-head th {
  padding: 16px 24px;
  text-align: left;
  font-weight: 500;
  font-size: 13px;
  letter-spacing: 0.025em;
}

.table-body {
  color: #cbd5e1;
}

.table-row {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border-bottom: 1px solid rgba(30, 41, 59, 0.5);
  position: relative;
}

.table-row:hover {
  background: rgba(30, 41, 59, 0.5);
}

.table-row:hover td {
  position: relative;
  z-index: 1;
}

.table-row td {
  padding: 16px 24px;
}

/* Column Widths */
.col-status {
  width: 60px;
}

.col-name {
  width: 150px;
}

.col-sql {
  min-width: 300px;
}

.col-category {
  width: 100px;
}

.col-time {
  width: 100px;
}

.col-actions {
  width: 120px;
  text-align: right;
}

/* SQL Name */
.sql-name {
  color: #cbd5e1;
  font-size: 13px;
  cursor: pointer;
  transition: color 0.2s;
  display: block;
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sql-name:hover {
  color: #a78bfa;
}

.sql-name-empty {
  color: #64748b;
  font-size: 12px;
  cursor: pointer;
  font-style: italic;
}

.sql-name-empty:hover {
  color: #94a3b8;
}

/* Category Tag */
.category-tag {
  cursor: pointer;
  transition: opacity 0.2s;
  font-size: 11px;
}

.category-tag:hover {
  opacity: 0.8;
}

.category-empty {
  color: #64748b;
  font-size: 12px;
  cursor: pointer;
  font-style: italic;
}

.category-empty:hover {
  color: #94a3b8;
}

/* Category Tags Container */
.category-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  cursor: pointer;
  min-height: 24px;
  align-items: center;
}

.more-tags {
  font-size: 11px;
  color: #64748b;
  margin-left: 2px;
}

/* Category Manager */
.category-manager {
  padding: 16px 0;
}

.category-add-form {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 16px;
  border-bottom: 1px solid #334155;
}

.category-list {
  max-height: 400px;
  overflow-y: auto;
}

.category-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 8px;
  margin-bottom: 4px;
  background: rgba(15, 23, 42, 0.5);
}

.category-item:hover {
  background: rgba(30, 41, 59, 0.5);
}

.category-desc {
  flex: 0 0 120px;
  font-size: 12px;
  color: #94a3b8;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.category-prompt {
  flex: 1;
  font-size: 11px;
  color: #64748b;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.category-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

/* SQL Cell */
.sql-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* SQL Code */
.sql-code {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  color: #cbd5e1;
  background: rgba(15, 23, 42, 0.5);
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid rgba(51, 65, 85, 0.5);
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  max-width: 500px;
}

/* Expand Button */
.expand-btn {
  flex-shrink: 0;
  opacity: 0.6;
  transition: opacity 0.2s;
  color: #6366f1 !important;
}

.expand-btn:hover {
  opacity: 1;
  background: rgba(99, 102, 241, 0.1) !important;
}

:deep(.expand-btn .n-icon) {
  color: #6366f1;
}

.sql-keyword {
  color: #a78bfa;
  font-weight: 600;
}

.sql-rest {
  color: #cbd5e1;
}

/* Duration Text */
.duration-text {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  color: #94a3b8;
}

/* Time Text */
.time-text {
  font-size: 12px;
  color: #64748b;
}

/* Action Buttons */
.action-buttons {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
  opacity: 0;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  transform: translateX(8px);
}

.table-row:hover .action-buttons {
  opacity: 1;
  transform: translateX(0);
}

.action-btn {
  padding: 8px;
  border-radius: 8px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  color: #64748b !important;
}

.action-btn:hover {
  background: rgba(99, 102, 241, 0.1) !important;
  color: #a78bfa !important;
}

:deep(.action-btn .n-icon) {
  color: #94a3b8;
}

:deep(.action-btn:hover .n-icon) {
  color: #cbd5e1;
}

/* Empty Row */
.empty-row {
  text-align: center;
  padding: 48px 24px !important;
}

/* Table Footer */
.table-footer {
  padding: 12px;
  border-top: 1px solid #1e293b;
  background: rgba(15, 23, 42, 0.3);
  text-align: center;
}

.footer-text {
  font-size: 12px;
  color: #94a3b8; /* slate-400 */
}

/* Button Overrides for Dark Theme */
:deep(.n-button) {
  transition: all 0.2s;
}

/* 主要按钮样式 */
:deep(.primary-button) {
  background-color: #6366f1 !important;
  border-color: #6366f1 !important;
  color: #ffffff !important;
  box-shadow: 0 10px 15px -3px rgba(99, 102, 241, 0.2) !important;
  transition: all 0.2s !important;
}

:deep(.primary-button:hover) {
  background-color: #4f46e5 !important;
  border-color: #4f46e5 !important;
}

:deep(.primary-button:active) {
  background-color: #4338ca !important;
  border-color: #4338ca !important;
}

:deep(.n-button--default-type) {
  background-color: #1e293b;
  border-color: #334155;
  color: #cbd5e1;
}

:deep(.n-button--default-type:hover) {
  background-color: #334155;
  border-color: #475569;
}

/* Modal Overrides */
:deep(.n-dialog) {
  background-color: #1e293b;
  color: #cbd5e1;
}

:deep(.n-dialog__title) {
  color: #f1f5f9;
}

:deep(.n-input) {
  background-color: #0f172a;
  border-color: #334155;
  color: #cbd5e1;
}

:deep(.n-input:hover) {
  border-color: #475569;
}

:deep(.n-input:focus) {
  border-color: #6366f1;
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);
}

:deep(.n-input::placeholder) {
  color: #64748b; /* slate-500 */
}

:deep(.n-form-item-label) {
  color: #cbd5e1;
}

/* 完整SQL容器 */
.full-sql-container {
  background: #0f172a;
  border: 1px solid #334155;
  border-radius: 8px;
  padding: 16px;
  max-height: 500px;
  overflow-y: auto;
}

.full-sql-code {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  color: #cbd5e1;
  white-space: pre-wrap;
  word-break: break-all;
  line-height: 1.6;
  display: block;
}

.full-sql-container::-webkit-scrollbar {
  width: 8px;
}

.full-sql-container::-webkit-scrollbar-track {
  background: transparent;
}

.full-sql-container::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 4px;
}

.full-sql-container::-webkit-scrollbar-thumb:hover {
  background: #475569;
}

/* 整合视图容器 */
.template-view-container {
  flex: 1;
  display: flex;
  overflow: hidden;
  background: rgba(15, 23, 42, 0.4);
  border-radius: 20px;
  border: 1px solid rgba(51, 65, 85, 0.5);
  backdrop-filter: blur(12px);
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

:deep(.template-view-container .sql-template-view) {
  width: 100%;
  padding: 0;
  background: transparent;
}

/* 按钮组样式 */
:deep(.n-button-group) {
  display: inline-flex;
}

:deep(.n-button-group .n-button) {
  border-radius: 0;
}

:deep(.n-button-group .n-button:first-child) {
  border-top-left-radius: 8px;
  border-bottom-left-radius: 8px;
}

:deep(.n-button-group .n-button:last-child) {
  border-top-right-radius: 8px;
  border-bottom-right-radius: 8px;
}

/* Responsive adjustments */
@media (max-width: 1024px) {
  .search-input {
    width: 200px;
  }

  .sql-code {
    max-width: 300px;
  }

  .header-actions {
    flex-wrap: wrap;
  }
}
</style>
