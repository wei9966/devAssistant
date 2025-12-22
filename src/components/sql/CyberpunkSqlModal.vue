<template>
  <!-- 背景遮罩 (Backdrop) -->
  <Teleport to="body">
    <Transition name="backdrop-fade">
      <div
        v-if="isVisible"
        class="sql-backdrop"
        @click.self="close"
      >
        <!-- 模态框主体 -->
        <Transition name="modal-scale">
          <div
            v-if="isVisible"
            class="sql-modal"
            @keydown="handleKeyDown"
          >
            <!-- Toast 提示 -->
            <Transition name="toast-bounce">
              <div v-if="showToast" class="toast-notification">
                <svg class="toast-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M20 6L9 17l-5-5"/>
                </svg>
                <span>SQL 已复制到剪贴板</span>
              </div>
            </Transition>

            <!-- 1. 顶部搜索栏 -->
            <div class="modal-header">
              <div class="search-icon-wrapper">
                <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="11" cy="11" r="8"/>
                  <path d="m21 21-4.35-4.35"/>
                </svg>
              </div>
              <input
                ref="searchInputRef"
                type="text"
                v-model="searchQuery"
                placeholder="搜索 SQL 历史 (输入 'select', 'order', '时间'...)"
                class="search-input"
              />
              <div class="search-hints">
                <span class="hint-badge">ESC 关闭</span>
              </div>
            </div>

            <!-- 2. 主体内容区 (双栏布局) -->
            <div class="modal-body">
              <!-- 左侧：列表区 (40%) -->
              <div class="list-panel" ref="listRef">
                <div v-if="filteredSqls.length === 0" class="empty-state">
                  <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d="M9.75 9.75l4.5 4.5m0-4.5l-4.5 4.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  <p>没有找到相关 SQL</p>
                </div>
                <div
                  v-else
                  v-for="(sql, index) in filteredSqls"
                  :key="sql.id"
                  :class="['list-item', { selected: selectedIndex === index }]"
                  @click="selectedIndex = index"
                  @mouseenter="selectedIndex = index"
                >
                  <!-- 选中态的左侧指示条 -->
                  <div v-if="selectedIndex === index" class="selection-indicator"></div>

                  <div class="item-header">
                    <span :class="['item-title', { 'title-selected': selectedIndex === index }]">
                      {{ sql.name || '未命名' }}
                    </span>
                    <!-- 状态图标 -->
                    <div class="status-icons">
                      <svg v-if="getStatusType(sql.sqlType) === 'success'" class="status-icon success" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
                      </svg>
                      <svg v-else-if="getStatusType(sql.sqlType) === 'warning'" class="status-icon warning" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
                      </svg>
                      <svg v-else-if="getStatusType(sql.sqlType) === 'danger'" class="status-icon danger" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 2C6.47 2 2 6.47 2 12s4.47 10 10 10 10-4.47 10-10S17.53 2 12 2zm5 13.59L15.59 17 12 13.41 8.41 17 7 15.59 10.59 12 7 8.41 8.41 7 12 10.59 15.59 7 17 8.41 13.41 12 17 15.59z"/>
                      </svg>
                    </div>
                  </div>

                  <!-- SQL 预览 (截断) -->
                  <div class="item-preview">
                    {{ truncateSql(sql.sqlText, 60) }}
                  </div>

                  <!-- 底部元数据 -->
                  <div class="item-meta">
                    <span :class="['type-badge', getTypeBadgeClass(sql.sqlType)]">
                      {{ sql.sqlType || 'SQL' }}
                    </span>
                    <span v-if="sql.categories && sql.categories.length > 0" class="category-badge">
                      {{ sql.categories[0].name }}
                    </span>
                    <span class="time-badge">{{ formatTime(sql.executedAt) }}</span>
                  </div>
                </div>
              </div>

              <!-- 右侧：预览区 (60%) -->
              <div class="preview-panel">
                <template v-if="selectedSql">
                  <!-- 预览头部 -->
                  <div class="preview-header">
                    <span class="preview-label">
                      <svg class="label-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M16 18l6-6-6-6M8 6l-6 6 6 6"/>
                      </svg>
                      PREVIEW
                    </span>
                    <div class="preview-actions">
                      <button class="action-btn copy-btn" @click="handleCopy(selectedSql.sqlText)">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <rect x="9" y="9" width="13" height="13" rx="2"/>
                          <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
                        </svg>
                        复制
                      </button>
                      <button class="action-btn edit-btn" @click="handleEdit(selectedSql)">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/>
                          <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
                        </svg>
                        编辑
                      </button>
                    </div>
                  </div>

                  <!-- 代码展示区 -->
                  <div class="preview-content">
                    <div class="sql-info">
                      <h2 class="sql-title">{{ selectedSql.name || '未命名 SQL' }}</h2>
                      <div class="sql-meta-info">
                        <span v-if="selectedSql.categories && selectedSql.categories.length > 0">
                          分类: {{ selectedSql.categories.map(c => c.name).join(', ') }}
                        </span>
                        <span v-else>分类: 未分类</span>
                        <span class="meta-separator">•</span>
                        <span>上次执行: {{ formatTime(selectedSql.executedAt) }}</span>
                      </div>
                    </div>

                    <!-- SQL 代码块 -->
                    <div class="code-block">
                      <span class="code-lang-label">SQL</span>
                      <pre class="sql-code"><code v-html="highlightSql(selectedSql.sqlText)"></code></pre>
                    </div>

                    <!-- 收藏状态 -->
                    <div class="favorite-section" v-if="selectedSql">
                      <button
                        :class="['favorite-btn', { active: selectedSql.isFavorite }]"
                        @click="handleToggleFavorite(selectedSql.id)"
                      >
                        <svg v-if="selectedSql.isFavorite" class="star-icon filled" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                        </svg>
                        <svg v-else class="star-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                        </svg>
                        {{ selectedSql.isFavorite ? '已收藏' : '收藏' }}
                      </button>
                    </div>
                  </div>
                </template>

                <!-- 无选中项 -->
                <div v-else class="preview-empty">
                  <svg class="empty-preview-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
                    <path d="M16 18l6-6-6-6M8 6l-6 6 6 6"/>
                  </svg>
                  <p>选择左侧列表查看详情</p>
                </div>
              </div>
            </div>

            <!-- 3. 底部状态栏 -->
            <div class="modal-footer">
              <div class="keyboard-shortcuts">
                <span class="shortcut"><kbd>↵</kbd> 复制</span>
                <span class="shortcut"><kbd>↑↓</kbd> 导航</span>
                <span class="shortcut"><kbd>ESC</kbd> 关闭</span>
              </div>
              <div class="record-count">
                {{ filteredSqls.length }} 条记录
              </div>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useSqlStore } from '@/stores/sqlStore'
import type { SqlRecord } from '@/types/sql'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import 'dayjs/locale/zh-cn'

dayjs.extend(relativeTime)
dayjs.locale('zh-cn')

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'edit': [sql: SqlRecord]
}>()

const sqlStore = useSqlStore()

// 状态
const searchInputRef = ref<HTMLInputElement>()
const listRef = ref<HTMLDivElement>()
const searchQuery = ref('')
const selectedIndex = ref(0)
const showToast = ref(false)

// 计算属性
const isVisible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

// 过滤后的SQL列表
const filteredSqls = computed(() => {
  let sqls = sqlStore.recentSqls

  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    sqls = sqls.filter(sql =>
      sql.sqlText.toLowerCase().includes(query) ||
      sql.name?.toLowerCase().includes(query) ||
      sql.categories?.some(cat => cat.name.toLowerCase().includes(query))
    )
  }

  return sqls.slice(0, 50) // 最多显示50条
})

// 当前选中的SQL
const selectedSql = computed(() => {
  return filteredSqls.value[selectedIndex.value] || null
})

// 方法
const close = () => {
  isVisible.value = false
}

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    e.preventDefault()
    close()
  } else if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (selectedIndex.value < filteredSqls.value.length - 1) {
      selectedIndex.value++
      scrollToSelected()
    }
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (selectedIndex.value > 0) {
      selectedIndex.value--
      scrollToSelected()
    }
  } else if (e.key === 'Enter') {
    e.preventDefault()
    if (selectedSql.value) {
      handleCopy(selectedSql.value.sqlText)
    }
  }
}

const scrollToSelected = () => {
  nextTick(() => {
    const listElement = listRef.value
    if (!listElement) return
    const selectedElement = listElement.querySelector('.list-item.selected')
    if (selectedElement) {
      selectedElement.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
    }
  })
}

const handleCopy = async (sqlText: string) => {
  try {
    await navigator.clipboard.writeText(sqlText)
    showToast.value = true
    setTimeout(() => {
      showToast.value = false
    }, 2000)
  } catch (error) {
    console.error('复制失败:', error)
  }
}

const handleEdit = (sql: SqlRecord) => {
  emit('edit', sql)
  close()
}

const handleToggleFavorite = async (sqlId: number | undefined) => {
  if (!sqlId) return
  await sqlStore.toggleFavorite(sqlId)
}

// 获取状态类型
const getStatusType = (sqlType: string | undefined): string => {
  const type = sqlType?.toUpperCase()
  if (type === 'DELETE' || type === 'DROP') {
    return 'danger'
  } else if (type === 'UPDATE' || type === 'ALTER') {
    return 'warning'
  }
  return 'success'
}

// 获取类型徽章样式
const getTypeBadgeClass = (sqlType: string | undefined): string => {
  const type = sqlType?.toUpperCase()
  if (type === 'DELETE' || type === 'DROP') {
    return 'type-danger'
  } else if (type === 'UPDATE' || type === 'ALTER') {
    return 'type-warning'
  } else if (type === 'INSERT' || type === 'CREATE') {
    return 'type-success'
  }
  return 'type-info'
}

// 截断SQL
const truncateSql = (sql: string, maxLength: number): string => {
  if (!sql) return ''
  if (sql.length <= maxLength) return sql
  return sql.substring(0, maxLength) + '...'
}

// 格式化时间
const formatTime = (time: string | undefined): string => {
  if (!time) return '-'
  return dayjs(time).fromNow()
}

// SQL语法高亮
const highlightSql = (sql: string): string => {
  if (!sql) return ''

  const keywords = /\b(SELECT|FROM|WHERE|GROUP BY|ORDER BY|UPDATE|SET|DELETE|INSERT|INTO|VALUES|AND|OR|AS|SUM|COUNT|LIMIT|TOP|JOIN|LEFT|RIGHT|INNER|OUTER|ON|HAVING|DISTINCT|CREATE|ALTER|DROP|TABLE|INDEX|VIEW|DATABASE|NOT|NULL|IN|LIKE|BETWEEN|EXISTS|CASE|WHEN|THEN|ELSE|END|ASC|DESC|UNION|ALL|AVG|MAX|MIN)\b/gi
  const strings = /'([^']*)'/g
  const numbers = /\b(\d+)\b/g

  let html = sql
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(keywords, '<span class="sql-keyword">$1</span>')
    .replace(strings, '<span class="sql-string">\'$1\'</span>')
    .replace(numbers, '<span class="sql-number">$1</span>')

  return html
}

// 监听显示状态
watch(isVisible, async (visible) => {
  if (visible) {
    searchQuery.value = ''
    selectedIndex.value = 0

    // 加载SQL列表
    if (sqlStore.recentSqls.length === 0) {
      await sqlStore.loadRecentSqls()
    }

    // 聚焦搜索框
    nextTick(() => {
      searchInputRef.value?.focus()
    })
  }
})

// 监听搜索变化，重置选中索引
watch(searchQuery, () => {
  selectedIndex.value = 0
})
</script>

<style scoped>
/* ============================================ */
/* 背景遮罩                                      */
/* ============================================ */
.sql-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: var(--modal-backdrop, rgba(2, 6, 23, 0.85));
  backdrop-filter: blur(12px);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

/* ============================================ */
/* 模态框主体                                    */
/* ============================================ */
.sql-modal {
  width: 900px;
  height: 600px;
  background: var(--bg-elevated, #0f172a);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
  border: 1px solid var(--border-default, rgba(51, 65, 85, 0.8));
  box-shadow:
    0 0 0 1px rgba(6, 182, 212, 0.1),
    0 0 20px rgba(6, 182, 212, 0.1),
    0 0 40px rgba(6, 182, 212, 0.05),
    0 25px 50px -12px rgba(0, 0, 0, 0.5);
  animation: neon-pulse 3s infinite;
}

@keyframes neon-pulse {
  0%, 100% {
    box-shadow:
      0 0 0 1px rgba(6, 182, 212, 0.1),
      0 0 5px rgba(6, 182, 212, 0.1),
      0 0 15px rgba(6, 182, 212, 0.05),
      0 25px 50px -12px rgba(0, 0, 0, 0.5);
  }
  50% {
    box-shadow:
      0 0 0 1px rgba(6, 182, 212, 0.2),
      0 0 15px rgba(6, 182, 212, 0.2),
      0 0 30px rgba(6, 182, 212, 0.1),
      0 25px 50px -12px rgba(0, 0, 0, 0.5);
  }
}

/* ============================================ */
/* Toast 提示                                    */
/* ============================================ */
.toast-notification {
  position: absolute;
  top: -60px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 100;
  background: var(--accent-primary, linear-gradient(135deg, #06b6d4 0%, #0891b2 100%));
  color: var(--bg-elevated, #0f172a);
  padding: 10px 24px;
  border-radius: 9999px;
  font-weight: 600;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  box-shadow: 0 10px 25px rgba(6, 182, 212, 0.4);
}

.toast-icon {
  width: 18px;
  height: 18px;
}

/* ============================================ */
/* Header: 搜索栏                               */
/* ============================================ */
.modal-header {
  height: 64px;
  border-bottom: 1px solid var(--border-default, #1e293b);
  display: flex;
  align-items: center;
  padding: 0 24px;
  flex-shrink: 0;
  background: var(--bg-surface, rgba(15, 23, 42, 0.8));
}

.search-icon-wrapper {
  margin-right: 16px;
}

.search-icon {
  width: 24px;
  height: 24px;
  color: var(--text-secondary, #64748b);
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 18px;
  font-weight: 400;
  color: var(--text-primary, #e2e8f0);
  height: 100%;
}

.search-input::placeholder {
  color: var(--text-muted, #475569);
}

.search-hints {
  display: flex;
  gap: 8px;
}

.hint-badge {
  background: var(--bg-surface, #1e293b);
  color: var(--text-secondary, #64748b);
  font-size: 11px;
  padding: 4px 10px;
  border-radius: 4px;
  border: 1px solid var(--border-default, #334155);
  font-family: 'Consolas', 'Monaco', monospace;
}

/* ============================================ */
/* Body: 双栏布局                               */
/* ============================================ */
.modal-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

/* 左侧列表 */
.list-panel {
  width: 40%;
  border-right: 1px solid var(--border-default, #1e293b);
  overflow-y: auto;
  background: var(--bg-surface, rgba(15, 23, 42, 0.5));
}

.list-panel::-webkit-scrollbar {
  width: 6px;
}

.list-panel::-webkit-scrollbar-track {
  background: transparent;
}

.list-panel::-webkit-scrollbar-thumb {
  background: var(--border-default, #334155);
  border-radius: 3px;
}

.list-panel::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted, #475569);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted, #475569);
  padding: 32px;
}

.empty-icon {
  width: 48px;
  height: 48px;
  margin-bottom: 12px;
  opacity: 0.5;
}

/* 列表项 */
.list-item {
  position: relative;
  padding: 12px 16px;
  cursor: pointer;
  transition: all 0.15s ease;
  border-bottom: 1px solid var(--border-default, rgba(30, 41, 59, 0.5));
}

.list-item:hover {
  background: var(--bg-surface, rgba(30, 41, 59, 0.5));
}

.list-item.selected {
  background: var(--bg-elevated, rgba(30, 41, 59, 0.8));
}

.selection-indicator {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: var(--accent-primary);
  box-shadow: 0 0 10px var(--accent-glow);
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 6px;
}

.item-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary, #94a3b8);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 85%;
  transition: color 0.15s ease;
}

.item-title.title-selected {
  color: var(--text-primary, #e0f2fe);
}

.status-icons {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.status-icon {
  width: 16px;
  height: 16px;
}

.status-icon.success {
  color: var(--success);
}

.status-icon.warning {
  color: var(--warning);
}

.status-icon.danger {
  color: var(--error);
}

.item-preview {
  font-size: 11px;
  font-family: 'Consolas', 'Monaco', monospace;
  color: var(--text-muted, #475569);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 8px;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.type-badge {
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 3px;
  font-weight: 500;
}

.type-info {
  background: rgba(59, 130, 246, 0.2);
  color: #60a5fa;
  border: 1px solid rgba(59, 130, 246, 0.3);
}

.type-success {
  background: rgba(34, 197, 94, 0.2);
  color: #4ade80;
  border: 1px solid rgba(34, 197, 94, 0.3);
}

.type-warning {
  background: rgba(249, 115, 22, 0.2);
  color: #fb923c;
  border: 1px solid rgba(249, 115, 22, 0.3);
}

.type-danger {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.category-badge {
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 3px;
  background: rgba(139, 92, 246, 0.2);
  color: #a78bfa;
  border: 1px solid rgba(139, 92, 246, 0.3);
}

.time-badge {
  font-size: 10px;
  color: var(--text-muted, #475569);
  margin-left: auto;
}

/* ============================================ */
/* 右侧预览区                                    */
/* ============================================ */
.preview-panel {
  width: 60%;
  background: var(--bg-base, #0b1120);
  display: flex;
  flex-direction: column;
}

.preview-header {
  height: 48px;
  border-bottom: 1px solid var(--border-default, #1e293b);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  background: var(--bg-elevated, #0f172a);
  flex-shrink: 0;
}

.preview-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  font-family: 'Consolas', 'Monaco', monospace;
  color: var(--text-secondary, #64748b);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.label-icon {
  width: 14px;
  height: 14px;
}

.preview-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid;
}

.action-btn svg {
  width: 14px;
  height: 14px;
}

.copy-btn {
  background: rgba(6, 182, 212, 0.1);
  border-color: rgba(6, 182, 212, 0.3);
  color: #22d3ee;
}

.copy-btn:hover {
  background: rgba(6, 182, 212, 0.2);
  border-color: rgba(6, 182, 212, 0.5);
}

.edit-btn {
  background: var(--bg-surface, #1e293b);
  border-color: var(--border-default, #334155);
  color: var(--text-secondary, #94a3b8);
}

.edit-btn:hover {
  background: var(--border-default, #334155);
  color: var(--text-primary, #e2e8f0);
}

/* 预览内容 */
.preview-content {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}

.preview-content::-webkit-scrollbar {
  width: 6px;
}

.preview-content::-webkit-scrollbar-track {
  background: transparent;
}

.preview-content::-webkit-scrollbar-thumb {
  background: var(--border-default, #334155);
  border-radius: 3px;
}

.sql-info {
  margin-bottom: 20px;
}

.sql-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #f1f5f9);
  margin-bottom: 8px;
}

.sql-meta-info {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-secondary, #64748b);
}

.meta-separator {
  color: var(--text-muted, #475569);
}

/* 代码块 */
.code-block {
  position: relative;
  background: var(--input-bg, rgba(30, 41, 59, 0.5));
  border: 1px solid var(--border-default, rgba(51, 65, 85, 0.5));
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.code-lang-label {
  position: absolute;
  top: 8px;
  right: 12px;
  font-size: 10px;
  color: var(--text-muted, #475569);
  font-family: 'Consolas', 'Monaco', monospace;
  text-transform: uppercase;
}

.sql-code {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-primary, #cbd5e1);
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0;
}

/* SQL 语法高亮 */
:deep(.sql-keyword) {
  color: #c084fc;
  font-weight: 600;
}

:deep(.sql-string) {
  color: #4ade80;
}

:deep(.sql-number) {
  color: #fcd34d;
}

/* 收藏按钮 */
.favorite-section {
  border-top: 1px solid var(--border-default, #1e293b);
  padding-top: 16px;
}

.favorite-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
  background: var(--bg-surface, #1e293b);
  border: 1px solid var(--border-default, #334155);
  color: var(--text-secondary, #94a3b8);
}

.favorite-btn:hover {
  background: var(--border-default, #334155);
  color: var(--text-primary, #e2e8f0);
}

.favorite-btn.active {
  background: rgba(251, 191, 36, 0.1);
  border-color: rgba(251, 191, 36, 0.3);
  color: #fbbf24;
}

.star-icon {
  width: 18px;
  height: 18px;
}

.star-icon.filled {
  color: #fbbf24;
}

/* 空预览状态 */
.preview-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted, #475569);
}

.empty-preview-icon {
  width: 64px;
  height: 64px;
  margin-bottom: 16px;
  opacity: 0.2;
}

/* ============================================ */
/* Footer: 状态栏                               */
/* ============================================ */
.modal-footer {
  height: 40px;
  border-top: 1px solid var(--border-default, #1e293b);
  background: var(--bg-surface, rgba(15, 23, 42, 0.8));
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  flex-shrink: 0;
}

.keyboard-shortcuts {
  display: flex;
  gap: 16px;
}

.shortcut {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-secondary, #64748b);
}

.shortcut kbd {
  display: inline-block;
  padding: 2px 6px;
  background: var(--bg-surface, #1e293b);
  border-radius: 3px;
  font-size: 10px;
  font-family: 'Consolas', 'Monaco', monospace;
  color: var(--text-secondary, #94a3b8);
  border: 1px solid var(--border-default, #334155);
}

.record-count {
  font-size: 11px;
  color: var(--text-secondary, #64748b);
}

/* ============================================ */
/* 动画                                         */
/* ============================================ */

/* 背景遮罩过渡 */
.backdrop-fade-enter-active,
.backdrop-fade-leave-active {
  transition: opacity 0.3s ease;
}

.backdrop-fade-enter-from,
.backdrop-fade-leave-to {
  opacity: 0;
}

/* 模态框进出动画 */
.modal-scale-enter-active,
.modal-scale-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.modal-scale-enter-from,
.modal-scale-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(10px);
}

.modal-scale-enter-to,
.modal-scale-leave-from {
  opacity: 1;
  transform: scale(1) translateY(0);
}

/* Toast 动画 */
.toast-bounce-enter-active {
  animation: toast-in 0.4s ease;
}

.toast-bounce-leave-active {
  animation: toast-out 0.3s ease;
}

@keyframes toast-in {
  0% {
    opacity: 0;
    transform: translateX(-50%) translateY(-20px);
  }
  50% {
    transform: translateX(-50%) translateY(5px);
  }
  100% {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

@keyframes toast-out {
  0% {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
  100% {
    opacity: 0;
    transform: translateX(-50%) translateY(-20px);
  }
}

/* ============================================ */
/* 响应式                                       */
/* ============================================ */
@media (max-width: 960px) {
  .sql-modal {
    width: 95vw;
    height: 85vh;
  }

  .list-panel {
    width: 35%;
  }

  .preview-panel {
    width: 65%;
  }
}

@media (max-width: 768px) {
  .modal-body {
    flex-direction: column;
  }

  .list-panel {
    width: 100%;
    height: 40%;
    border-right: none;
    border-bottom: 1px solid var(--border-default, #1e293b);
  }

  .preview-panel {
    width: 100%;
    height: 60%;
  }

  .keyboard-shortcuts {
    display: none;
  }
}
</style>
