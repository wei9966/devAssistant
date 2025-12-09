<template>
  <div class="sql-template-view">
    <!-- 工具栏 -->
    <div class="toolbar">
      <n-space align="center" :size="12">
        <n-button
          size="small"
          type="warning"
          :loading="consolidating"
          @click="handleConsolidate"
        >
          <template #icon><n-icon><FlashOutline /></n-icon></template>
          智能整合
        </n-button>
        <n-input
          v-model:value="searchKeyword"
          placeholder="搜索表名或SQL..."
          size="small"
          clearable
          style="width: 200px"
        >
          <template #prefix><n-icon><SearchOutline /></n-icon></template>
        </n-input>
        <n-button size="small" quaternary @click="toggleExpandAll">
          {{ allExpanded ? '全部收起' : '全部展开' }}
        </n-button>
      </n-space>
    </div>

    <!-- 统计信息 -->
    <div class="stats-bar">
      <span class="stat-item">模板总数 <strong>{{ totalTemplates }}</strong></span>
      <span class="stat-item">变体SQL数 <strong>{{ totalVariants }}</strong></span>
      <span v-if="topTables.length > 0" class="stat-item">
        热门表 <strong>{{ topTables.join(', ') }}</strong>
      </span>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-container">
      <n-spin size="large"><template #description>加载中...</template></n-spin>
    </div>

    <!-- 层级分组列表 -->
    <div v-else-if="groupedData.length > 0" class="group-list">
      <!-- 折叠状态提示 -->
      <div v-if="!allExpanded && expandedTables.size === 0" class="collapsed-hint">
        <n-icon :component="InformationCircleOutline" size="20" />
        <span>点击下方表名展开查看SQL模板详情</span>
      </div>
      <!-- 按表名分组 -->
      <div v-for="tableGroup in filteredGroups" :key="tableGroup.tableName" class="table-group">
        <div class="table-header" @click="toggleTableGroup(tableGroup.tableName)">
          <n-icon class="expand-icon" :class="{ expanded: expandedTables.has(tableGroup.tableName) }">
            <ChevronForward />
          </n-icon>
          <span class="table-icon">📊</span>
          <span class="table-name">{{ tableGroup.tableName || '未知表' }}</span>
          <n-tag size="small" :bordered="false" type="info">
            {{ tableGroup.totalCount }} 条SQL
          </n-tag>
        </div>

        <!-- 表展开内容：按SQL类型分组 -->
        <n-collapse-transition :show="expandedTables.has(tableGroup.tableName)">
          <div class="type-groups">
            <div v-for="typeGroup in tableGroup.types" :key="typeGroup.sqlType" class="type-group">
              <div class="type-header" @click="toggleTypeGroup(tableGroup.tableName, typeGroup.sqlType)">
                <n-icon class="expand-icon small" :class="{ expanded: isTypeExpanded(tableGroup.tableName, typeGroup.sqlType) }">
                  <ChevronForward />
                </n-icon>
                <span class="type-icon">{{ getTypeIcon(typeGroup.sqlType) }}</span>
                <span class="type-name">{{ typeGroup.sqlType || 'OTHER' }}</span>
                <n-tag size="tiny" :bordered="false" :type="getTypeTagType(typeGroup.sqlType)">
                  {{ typeGroup.templates.length }} 个模板
                </n-tag>
              </div>

              <!-- 类型展开内容：模板列表 -->
              <n-collapse-transition :show="isTypeExpanded(tableGroup.tableName, typeGroup.sqlType)">
                <div class="template-items">
                  <div v-for="template in typeGroup.templates" :key="template.id" class="template-item">
                    <div class="template-row" @click="toggleTemplate(template.id)">
                      <code class="template-sql">{{ formatTemplateSql(template.templateText) }}</code>
                      <div class="template-stats">
                        <span class="variant-badge" :title="`${template.variantCount} 个变体SQL`">
                          {{ template.variantCount }}个变体
                        </span>
                        <span class="usage-badge" :title="`使用 ${template.usageCount} 次`">
                          {{ template.usageCount }}次
                        </span>
                        <n-button text size="tiny" @click.stop="copyToClipboard(template.templateText)">
                          <n-icon><CopyOutline /></n-icon>
                        </n-button>
                      </div>
                    </div>

                    <!-- 模板展开：变体SQL列表 -->
                    <n-collapse-transition :show="expandedTemplates.has(template.id)">
                      <div v-if="expandedTemplates.has(template.id)" class="variants-list">
                        <div v-if="loadingVariants.has(template.id)" class="loading-variants">
                          <n-spin size="small" />
                        </div>
                        <template v-else-if="templateVariants.get(template.id)?.length">
                          <div
                            v-for="(variant, idx) in templateVariants.get(template.id)"
                            :key="variant.id || idx"
                            class="variant-row"
                          >
                            <span class="variant-index">{{ idx + 1 }}.</span>
                            <code class="variant-sql">{{ truncate(variant.sqlText, 100) }}</code>
                            <n-button text size="tiny" @click.stop="copyToClipboard(variant.sqlText)">
                              <n-icon><CopyOutline /></n-icon>
                            </n-button>
                          </div>
                        </template>
                        <div v-else class="no-variants">暂无变体</div>
                      </div>
                    </n-collapse-transition>
                  </div>
                </div>
              </n-collapse-transition>
            </div>
          </div>
        </n-collapse-transition>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else class="empty-container">
      <n-empty description="暂无模板数据，点击「智能整合」自动分析SQL历史">
        <template #extra>
          <n-button size="small" type="primary" :loading="consolidating" @click="handleConsolidate">
            立即整合
          </n-button>
        </template>
      </n-empty>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  NSpace, NButton, NInput, NTag, NSpin, NEmpty,
  NCollapseTransition, NIcon, useMessage
} from 'naive-ui'
import {
  FlashOutline, SearchOutline, ChevronForward, CopyOutline, InformationCircleOutline
} from '@vicons/ionicons5'
import { sqlApi } from '@/api/sqlApi'
import type { SqlTemplate, SqlRecord } from '@/types/sql'

// Emits
const emit = defineEmits<{
  (e: 'copy-sql', sql: string): void
}>()

const message = useMessage()

// State
const loading = ref(false)
const consolidating = ref(false)
const searchKeyword = ref('')
const templates = ref<SqlTemplate[]>([])
const expandedTables = ref(new Set<string>())
const expandedTypes = ref(new Set<string>()) // key: "tableName:sqlType"
const expandedTemplates = ref(new Set<number>())
const templateVariants = ref(new Map<number, SqlRecord[]>())
const loadingVariants = ref(new Set<number>())
const allExpanded = ref(false)

// 计算：按表名 → SQL类型 → 模板 层级分组
interface TemplateGroup {
  sqlType: string
  templates: SqlTemplate[]
}

interface TableGroup {
  tableName: string
  totalCount: number
  types: TemplateGroup[]
}

const groupedData = computed<TableGroup[]>(() => {
  const tableMap = new Map<string, Map<string, SqlTemplate[]>>()

  for (const t of templates.value) {
    // 获取第一个表名作为主表
    const tableName = t.tableNames?.[0]?.toUpperCase() || '未知表'
    const sqlType = t.sqlType || 'OTHER'

    if (!tableMap.has(tableName)) {
      tableMap.set(tableName, new Map())
    }
    const typeMap = tableMap.get(tableName)!
    if (!typeMap.has(sqlType)) {
      typeMap.set(sqlType, [])
    }
    typeMap.get(sqlType)!.push(t)
  }

  // 转换为数组并排序
  const result: TableGroup[] = []
  for (const [tableName, typeMap] of tableMap) {
    const types: TemplateGroup[] = []
    let totalCount = 0
    for (const [sqlType, tmpls] of typeMap) {
      types.push({ sqlType, templates: tmpls })
      totalCount += tmpls.reduce((sum, t) => sum + t.variantCount, 0)
    }
    // 按SQL类型排序
    types.sort((a, b) => {
      const order = ['SELECT', 'INSERT', 'UPDATE', 'DELETE', 'CREATE', 'ALTER', 'DROP', 'OTHER']
      return order.indexOf(a.sqlType) - order.indexOf(b.sqlType)
    })
    result.push({ tableName, totalCount, types })
  }

  // 按表的SQL数量排序
  result.sort((a, b) => b.totalCount - a.totalCount)
  return result
})

// 过滤后的分组
const filteredGroups = computed(() => {
  if (!searchKeyword.value) return groupedData.value

  const keyword = searchKeyword.value.toLowerCase()
  return groupedData.value.filter(g => {
    // 匹配表名
    if (g.tableName.toLowerCase().includes(keyword)) return true
    // 匹配SQL内容
    return g.types.some(t =>
      t.templates.some(tmpl => tmpl.templateText.toLowerCase().includes(keyword))
    )
  })
})

// 统计
const totalTemplates = computed(() => templates.value.length)
const totalVariants = computed(() => templates.value.reduce((sum, t) => sum + t.variantCount, 0))
const topTables = computed(() => groupedData.value.slice(0, 3).map(g => g.tableName))

// 生命周期
onMounted(async () => {
  await loadTemplates()
})

// 加载模板
async function loadTemplates() {
  loading.value = true
  try {
    templates.value = await sqlApi.getSqlTemplates(null, 500)
  } catch (error) {
    console.error('加载模板失败:', error)
    message.error('加载失败')
  } finally {
    loading.value = false
  }
}

// 智能整合
async function handleConsolidate() {
  consolidating.value = true
  try {
    const result = await sqlApi.consolidateSqlTemplates()
    message.success(`整合完成！新建 ${result.templatesCreated} 个，更新 ${result.templatesUpdated} 个`)
    await loadTemplates()
  } catch (error) {
    console.error('整合失败:', error)
    message.error('整合失败')
  } finally {
    consolidating.value = false
  }
}

// 展开/收起表分组
function toggleTableGroup(tableName: string) {
  if (expandedTables.value.has(tableName)) {
    expandedTables.value.delete(tableName)
  } else {
    expandedTables.value.add(tableName)
  }
}

// 展开/收起类型分组
function toggleTypeGroup(tableName: string, sqlType: string) {
  const key = `${tableName}:${sqlType}`
  if (expandedTypes.value.has(key)) {
    expandedTypes.value.delete(key)
  } else {
    expandedTypes.value.add(key)
  }
}

function isTypeExpanded(tableName: string, sqlType: string) {
  return expandedTypes.value.has(`${tableName}:${sqlType}`)
}

// 展开/收起模板
async function toggleTemplate(templateId: number) {
  if (expandedTemplates.value.has(templateId)) {
    expandedTemplates.value.delete(templateId)
  } else {
    expandedTemplates.value.add(templateId)
    await loadVariants(templateId)
  }
}

// 加载变体
async function loadVariants(templateId: number) {
  if (templateVariants.value.has(templateId)) return

  loadingVariants.value.add(templateId)
  try {
    const variants = await sqlApi.getTemplateVariants(templateId)
    templateVariants.value.set(templateId, variants)
  } catch (error) {
    console.error('加载变体失败:', error)
  } finally {
    loadingVariants.value.delete(templateId)
  }
}

// 全部展开/收起
function toggleExpandAll() {
  if (allExpanded.value) {
    expandedTables.value.clear()
    expandedTypes.value.clear()
    expandedTemplates.value.clear()
  } else {
    groupedData.value.forEach(g => {
      expandedTables.value.add(g.tableName)
      g.types.forEach(t => {
        expandedTypes.value.add(`${g.tableName}:${t.sqlType}`)
      })
    })
  }
  allExpanded.value = !allExpanded.value
}

// 复制到剪贴板
function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text)
  message.success('已复制')
  emit('copy-sql', text)
}

// 格式化模板SQL显示
function formatTemplateSql(sql: string): string {
  if (!sql) return ''
  // 只保留核心部分，去掉SELECT * 等冗余
  let formatted = sql.trim()
  if (formatted.length > 80) {
    formatted = formatted.substring(0, 80) + '...'
  }
  return formatted
}

// 截断文本
function truncate(text: string, maxLen: number): string {
  if (!text) return ''
  return text.length > maxLen ? text.substring(0, maxLen) + '...' : text
}

// 获取SQL类型图标
function getTypeIcon(sqlType: string): string {
  const icons: Record<string, string> = {
    SELECT: '🔍',
    INSERT: '➕',
    UPDATE: '✏️',
    DELETE: '🗑️',
    CREATE: '🛠️',
    ALTER: '⚙️',
    DROP: '💥',
    OTHER: '📄'
  }
  return icons[sqlType] || '📄'
}

// 获取类型标签样式
function getTypeTagType(sqlType: string): 'success' | 'warning' | 'error' | 'info' | 'default' {
  const types: Record<string, 'success' | 'warning' | 'error' | 'info' | 'default'> = {
    SELECT: 'success',
    INSERT: 'info',
    UPDATE: 'warning',
    DELETE: 'error',
    CREATE: 'default',
    ALTER: 'default',
    DROP: 'error'
  }
  return types[sqlType] || 'default'
}
</script>

<style scoped>
.sql-template-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
}

/* 工具栏 */
.toolbar {
  padding: 12px 16px;
  background: rgba(15, 23, 42, 0.6);
  border-radius: 10px;
  border: 1px solid rgba(51, 65, 85, 0.5);
}

/* 统计栏 */
.stats-bar {
  display: flex;
  gap: 24px;
  padding: 10px 16px;
  background: rgba(15, 23, 42, 0.4);
  border-radius: 8px;
  font-size: 13px;
  color: #94a3b8;
}

.stat-item strong {
  color: #a78bfa;
  margin-left: 4px;
}

/* 加载和空状态 */
.loading-container,
.empty-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 48px;
  background: rgba(15, 23, 42, 0.4);
  border-radius: 12px;
}

/* 折叠状态提示 */
.collapsed-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 16px 20px;
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 10px;
  color: #a5b4fc;
  font-size: 13px;
  margin-bottom: 12px;
}

/* 分组列表 */
.group-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.group-list::-webkit-scrollbar {
  width: 6px;
}
.group-list::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 3px;
}

/* 表分组 */
.table-group {
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(51, 65, 85, 0.5);
  border-radius: 10px;
  overflow: hidden;
}

.table-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background 0.2s;
}

.table-header:hover {
  background: rgba(30, 41, 59, 0.5);
}

.expand-icon {
  font-size: 14px;
  color: #64748b;
  transition: transform 0.2s;
}

.expand-icon.expanded {
  transform: rotate(90deg);
}

.expand-icon.small {
  font-size: 12px;
}

.table-icon {
  font-size: 18px;
}

.table-name {
  flex: 1;
  font-weight: 600;
  color: #e2e8f0;
  font-size: 14px;
}

/* 类型分组 */
.type-groups {
  padding: 0 8px 8px 24px;
}

.type-group {
  margin-top: 4px;
  background: rgba(15, 23, 42, 0.3);
  border-radius: 8px;
  overflow: hidden;
}

.type-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background 0.2s;
}

.type-header:hover {
  background: rgba(30, 41, 59, 0.4);
}

.type-icon {
  font-size: 14px;
}

.type-name {
  flex: 1;
  font-weight: 500;
  color: #cbd5e1;
  font-size: 13px;
}

/* 模板列表 */
.template-items {
  padding: 4px 8px 8px 20px;
}

.template-item {
  margin-top: 4px;
  background: rgba(15, 23, 42, 0.4);
  border: 1px solid rgba(51, 65, 85, 0.3);
  border-radius: 6px;
  overflow: hidden;
}

.template-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  cursor: pointer;
  transition: background 0.2s;
}

.template-row:hover {
  background: rgba(99, 102, 241, 0.1);
}

.template-sql {
  flex: 1;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: #94a3b8;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.template-stats {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.variant-badge,
.usage-badge {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  color: #94a3b8;
}

.variant-badge {
  background: rgba(99, 102, 241, 0.2);
  color: #a5b4fc;
}

.usage-badge {
  background: rgba(34, 197, 94, 0.2);
  color: #86efac;
}

/* 变体列表 */
.variants-list {
  padding: 8px 10px;
  background: rgba(15, 23, 42, 0.5);
  border-top: 1px solid rgba(51, 65, 85, 0.3);
}

.loading-variants {
  display: flex;
  justify-content: center;
  padding: 12px;
}

.variant-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  margin-top: 4px;
  background: rgba(30, 41, 59, 0.4);
  border-radius: 4px;
  transition: background 0.2s;
}

.variant-row:hover {
  background: rgba(51, 65, 85, 0.5);
}

.variant-row:first-child {
  margin-top: 0;
}

.variant-index {
  color: #64748b;
  font-size: 11px;
  min-width: 20px;
}

.variant-sql {
  flex: 1;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 11px;
  color: #cbd5e1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.no-variants {
  text-align: center;
  color: #64748b;
  font-size: 12px;
  padding: 12px;
}

/* Naive UI 样式 */
:deep(.n-button--default-type) {
  background: #1e293b;
  border-color: #334155;
  color: #cbd5e1;
}

:deep(.n-button--default-type:hover) {
  background: #334155;
}

:deep(.n-input) {
  --n-color: rgba(15, 23, 42, 0.8);
  --n-border: 1px solid #334155;
  --n-text-color: #cbd5e1;
}

:deep(.n-tag) {
  background: rgba(51, 65, 85, 0.5);
  border-color: transparent;
}
</style>
