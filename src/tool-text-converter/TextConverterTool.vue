<template>
  <div class="text-converter-tool">
    <!-- Main Content -->
    <div class="converter-container">
      <!-- 左侧输入 -->
      <div class="panel input-panel">
        <div class="panel-header">
          <span class="panel-title">输入文本</span>
          <span class="char-count">{{ inputText.length }} 字符</span>
        </div>
        <textarea
          v-model="inputText"
          class="text-area"
          placeholder="请输入要转换的文本..."
          @input="handleAutoConvert"
        ></textarea>
      </div>

      <!-- 右侧输出 -->
      <div class="panel output-panel">
        <div class="panel-header">
          <span class="panel-title">转换结果</span>
          <div class="panel-actions">
            <button class="icon-btn" @click="copyResult" title="复制结果">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
              </svg>
            </button>
          </div>
        </div>
        <div class="output-wrapper">
          <div class="line-numbers" ref="lineNumbersRef">{{ lineNumbers }}</div>
          <div
            class="output-content"
            ref="outputRef"
            @scroll="syncScroll"
            v-html="outputHtml"
          ></div>
        </div>
      </div>
    </div>

    <!-- 操作按钮区域 -->
    <div class="actions-container">
      <!-- 引号转换 -->
      <div class="action-section">
        <div class="section-title">引号转换</div>
        <div class="button-group">
          <button class="action-btn btn-purple" @click="convertQuote('single')">单引号</button>
          <button class="action-btn btn-purple" @click="convertQuote('double')">双引号</button>
          <button class="action-btn btn-purple" @click="convertQuote('none')">无引号</button>
          <button class="action-btn btn-orange" @click="removeQuotes('single')">去单引</button>
          <button class="action-btn btn-orange" @click="removeQuotes('double')">去双引</button>
        </div>
      </div>

      <!-- JSON 操作 -->
      <div class="action-section">
        <div class="section-title">JSON 操作</div>
        <div class="button-group">
          <button class="action-btn btn-green" @click="formatJSON">格式化</button>
          <button class="action-btn btn-green" @click="compressJSON">压缩</button>
          <button class="action-btn btn-gray" @click="escapeJSON">转义</button>
          <button class="action-btn btn-gray" @click="unescapeJSON">反转义</button>
        </div>
      </div>

      <!-- 其他操作 -->
      <div class="action-section">
        <div class="section-title">其他操作</div>
        <div class="button-group">
          <button class="action-btn btn-blue" @click="toUpperCase">转大写</button>
          <button class="action-btn btn-blue" @click="toLowerCase">转小写</button>
          <button class="action-btn btn-gray" @click="copyResult">复制</button>
          <button class="action-btn btn-gray" @click="clearAll">清空</button>
        </div>
      </div>
    </div>

    <!-- 复制成功提示 -->
    <transition name="fade">
      <div v-if="showCopyMessage" class="copy-message">复制成功!</div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

// 状态
const inputText = ref('')
const outputText = ref('')
const outputHtml = ref('')
const showCopyMessage = ref(false)
const lineNumbersRef = ref<HTMLElement | null>(null)
const outputRef = ref<HTMLElement | null>(null)

// 计算行号
const lineNumbers = computed(() => {
  const text = outputText.value || ''
  const lines = text.split('\n')
  return lines.map((_, i) => i + 1).join('\n')
})

// 同步滚动
function syncScroll() {
  if (lineNumbersRef.value && outputRef.value) {
    lineNumbersRef.value.scrollTop = outputRef.value.scrollTop
  }
}

// 设置输出
function setOutput(text: string, isHtml = false) {
  outputText.value = text
  if (isHtml) {
    outputHtml.value = text
  } else {
    // 转义 HTML 并保留换行
    outputHtml.value = text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/\n/g, '<br>')
  }
}

// 检查是否为纯数字列表
function isNumberList(lines: string[]): boolean {
  if (lines.length === 0) return false
  return lines.every(line => {
    const trimmed = line.trim()
    return trimmed !== '' && /^-?\d+\.?\d*$/.test(trimmed)
  })
}

// 检查是否为文本列表
function isTextList(lines: string[]): boolean {
  if (lines.length === 0) return false
  return lines.some(line => /[a-zA-Z\u4e00-\u9fa5]/.test(line.trim()))
}

// JSON 语法高亮
function highlightJSON(json: string): string | null {
  try {
    const parsed = JSON.parse(json)
    const formatted = JSON.stringify(parsed, null, 2)

    const highlighted = formatted
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/("([^"\\]|\\.)*")\s*:/g, '<span class="json-key">$1</span>:')
      .replace(/:\s*("([^"\\]|\\.)*")/g, ': <span class="json-string">$1</span>')
      .replace(/\[\s*("([^"\\]|\\.)*")/g, '[ <span class="json-string">$1</span>')
      .replace(/,\s*("([^"\\]|\\.)*")/g, ', <span class="json-string">$1</span>')
      .replace(/:\s*(-?\d+\.?\d*)/g, ': <span class="json-number">$1</span>')
      .replace(/\[\s*(-?\d+\.?\d*)/g, '[ <span class="json-number">$1</span>')
      .replace(/,\s*(-?\d+\.?\d*)/g, ', <span class="json-number">$1</span>')
      .replace(/:\s*(true|false)/g, ': <span class="json-boolean">$1</span>')
      .replace(/\[\s*(true|false)/g, '[ <span class="json-boolean">$1</span>')
      .replace(/,\s*(true|false)/g, ', <span class="json-boolean">$1</span>')
      .replace(/:\s*(null)/g, ': <span class="json-null">$1</span>')
      .replace(/\[\s*(null)/g, '[ <span class="json-null">$1</span>')
      .replace(/,\s*(null)/g, ', <span class="json-null">$1</span>')
      .replace(/\n/g, '<br>')

    return highlighted
  } catch {
    return null
  }
}

// 自动转换
function handleAutoConvert() {
  const input = inputText.value.trim()

  if (!input) {
    setOutput('')
    return
  }

  // 尝试解析为 JSON
  const highlighted = highlightJSON(input)
  if (highlighted) {
    outputText.value = JSON.stringify(JSON.parse(input), null, 2)
    outputHtml.value = highlighted
    return
  }

  // 检查是否为换行分隔的列表
  const lines = input.split('\n').filter(line => line.trim() !== '')

  if (lines.length > 1 || (lines.length === 1 && !lines[0].includes(','))) {
    if (isNumberList(lines)) {
      // 纯数字列表：无引号，逗号分隔
      const result = lines.map(line => line.trim()).join(',')
      setOutput(result)
      return
    } else if (isTextList(lines)) {
      // 文本列表：单引号，逗号分隔
      const result = lines.map(line => `'${line.trim()}'`).join(',')
      setOutput(result)
      return
    }
  }

  // 其他情况保持原样
  setOutput(input)
}

// 引号转换
function convertQuote(quoteType: 'single' | 'double' | 'none') {
  const input = inputText.value
  if (!input.trim()) {
    return
  }

  const lines = input.split('\n').filter(line => line.trim() !== '')
  const quote = quoteType === 'single' ? "'" : quoteType === 'double' ? '"' : ''

  const result = lines
    .map(line => {
      const cleanLine = line.trim().replace(/\s+/g, ' ')
      return quote + cleanLine + quote
    })
    .join(',')

  setOutput(result)
  copyToClipboard(result)
}

// 去除引号
function removeQuotes(quoteType: 'single' | 'double') {
  const input = inputText.value
  if (!input.trim()) {
    return
  }

  const quote = quoteType === 'single' ? "'" : '"'
  const lines = input.split('\n').filter(line => line.trim() !== '')

  const result = lines
    .map(line => {
      return line.trim()
        .replace(new RegExp(`^${quote}|${quote}$`, 'g'), '')
        .replace(new RegExp(quote, 'g'), '')
    })
    .join(',')

  setOutput(result)
  copyToClipboard(result)
}

// JSON 格式化
function formatJSON() {
  try {
    const input = inputText.value
    const highlighted = highlightJSON(input)
    if (highlighted) {
      outputText.value = JSON.stringify(JSON.parse(input), null, 2)
      outputHtml.value = highlighted
    } else {
      throw new Error('Invalid JSON')
    }
  } catch {
    console.error('JSON 格式化失败')
  }
}

// JSON 压缩
function compressJSON() {
  try {
    const input = inputText.value
    const parsed = JSON.parse(input)
    const result = JSON.stringify(parsed)
    setOutput(result)
    copyToClipboard(result)
  } catch {
    console.error('JSON 压缩失败')
  }
}

// JSON 转义
function escapeJSON() {
  try {
    const input = inputText.value
    const parsed = JSON.parse(input)
    const jsonString = JSON.stringify(parsed)

    const escaped = jsonString
      .replace(/\\/g, '\\\\')
      .replace(/"/g, '\\"')
      .replace(/\n/g, '\\n')
      .replace(/\r/g, '\\r')
      .replace(/\t/g, '\\t')

    setOutput(escaped)
    copyToClipboard(escaped)
  } catch {
    console.error('JSON 转义失败')
  }
}

// JSON 反转义
function unescapeJSON() {
  try {
    const input = inputText.value

    const unescaped = input
      .replace(/\\"/g, '"')
      .replace(/\\n/g, '\n')
      .replace(/\\r/g, '\r')
      .replace(/\\t/g, '\t')
      .replace(/\\\\/g, '\\')

    const highlighted = highlightJSON(unescaped)
    if (highlighted) {
      outputText.value = JSON.stringify(JSON.parse(unescaped), null, 2)
      outputHtml.value = highlighted
      copyToClipboard(outputText.value)
    } else {
      throw new Error('Invalid JSON')
    }
  } catch {
    console.error('JSON 反转义失败')
  }
}

// 转大写
function toUpperCase() {
  const input = inputText.value
  if (!input.trim()) {
    return
  }
  const result = input.toUpperCase()
  setOutput(result)
  copyToClipboard(result)
}

// 转小写
function toLowerCase() {
  const input = inputText.value
  if (!input.trim()) {
    return
  }
  const result = input.toLowerCase()
  setOutput(result)
  copyToClipboard(result)
}

// 复制结果
function copyResult() {
  const text = outputText.value
  if (!text.trim()) {
    return
  }
  copyToClipboard(text)
}

// 复制到剪贴板
async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    showCopyMessage.value = true
    setTimeout(() => {
      showCopyMessage.value = false
    }, 2000)
  } catch {
    console.error('复制失败')
  }
}

// 清空
function clearAll() {
  inputText.value = ''
  setOutput('')
}
</script>

<style scoped>
.text-converter-tool {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  gap: 12px;
  background: color-mix(in srgb, var(--bg-base, #0c0c14) 95%, transparent);
}

/* 主内容区 */
.converter-container {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  min-height: 0;
}

.panel {
  display: flex;
  flex-direction: column;
  background: color-mix(in srgb, var(--card-bg, rgba(30, 41, 59, 0.5)) 100%, transparent);
  border: 1px solid var(--card-border, rgba(148, 163, 184, 0.1));
  border-radius: 10px;
  overflow: hidden;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: color-mix(in srgb, var(--bg-elevated, rgba(0, 0, 0, 0.2)) 100%, transparent);
  border-bottom: 1px solid var(--border-default, rgba(148, 163, 184, 0.1));
}

.panel-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted, #94a3b8);
}

.char-count {
  font-size: 11px;
  color: var(--text-dim, #64748b);
}

.panel-actions {
  display: flex;
  gap: 4px;
}

.icon-btn {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 10%, transparent);
  border: none;
  border-radius: 5px;
  padding: 5px;
  color: var(--text-muted, #94a3b8);
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-btn:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 20%, transparent);
  color: var(--text-primary, #e2e8f0);
}

.text-area {
  flex: 1;
  background: transparent;
  border: none;
  padding: 12px;
  color: var(--text-primary, #e2e8f0);
  font-size: 13px;
  font-family: 'Consolas', 'Monaco', monospace;
  line-height: 1.5;
  resize: none;
  outline: none;
}

.text-area::placeholder {
  color: var(--text-dim, #64748b);
}

.output-wrapper {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.line-numbers {
  width: 32px;
  padding: 12px 6px;
  background: color-mix(in srgb, var(--bg-elevated, rgba(0, 0, 0, 0.15)) 100%, transparent);
  color: var(--text-dim, #64748b);
  font-size: 12px;
  font-family: 'Consolas', 'Monaco', monospace;
  line-height: 1.5;
  text-align: right;
  user-select: none;
  overflow: hidden;
  white-space: pre;
}

.output-content {
  flex: 1;
  padding: 12px;
  color: var(--text-primary, #e2e8f0);
  font-size: 13px;
  font-family: 'Consolas', 'Monaco', monospace;
  line-height: 1.5;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

/* JSON 语法高亮 */
.output-content :deep(.json-key) {
  color: #f87171;
}

.output-content :deep(.json-string) {
  color: #60a5fa;
}

.output-content :deep(.json-number) {
  color: #4ade80;
}

.output-content :deep(.json-boolean) {
  color: #c084fc;
}

.output-content :deep(.json-null) {
  color: var(--text-muted, #94a3b8);
}

/* 操作按钮区域 */
.actions-container {
  flex-shrink: 0;
  display: flex;
  gap: 16px;
  padding: 12px;
  background: color-mix(in srgb, var(--card-bg, rgba(30, 41, 59, 0.5)) 100%, transparent);
  border: 1px solid var(--card-border, rgba(148, 163, 184, 0.1));
  border-radius: 10px;
  flex-wrap: wrap;
}

.action-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.section-title {
  font-size: 11px;
  font-weight: 500;
  color: var(--text-dim, #64748b);
}

.button-group {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.action-btn {
  padding: 6px 12px;
  border: none;
  border-radius: 5px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-purple {
  background: color-mix(in srgb, #8b5cf6 20%, transparent);
  color: #a78bfa;
}

.btn-purple:hover {
  background: color-mix(in srgb, #8b5cf6 30%, transparent);
}

.btn-orange {
  background: color-mix(in srgb, #fb923c 20%, transparent);
  color: #fb923c;
}

.btn-orange:hover {
  background: color-mix(in srgb, #fb923c 30%, transparent);
}

.btn-green {
  background: color-mix(in srgb, #22c55e 20%, transparent);
  color: #4ade80;
}

.btn-green:hover {
  background: color-mix(in srgb, #22c55e 30%, transparent);
}

.btn-blue {
  background: color-mix(in srgb, #3b82f6 20%, transparent);
  color: #60a5fa;
}

.btn-blue:hover {
  background: color-mix(in srgb, #3b82f6 30%, transparent);
}

.btn-gray {
  background: color-mix(in srgb, var(--text-muted, #94a3b8) 15%, transparent);
  color: var(--text-muted, #94a3b8);
}

.btn-gray:hover {
  background: color-mix(in srgb, var(--text-muted, #94a3b8) 25%, transparent);
}

/* 复制成功提示 */
.copy-message {
  position: fixed;
  bottom: 60px;
  left: 50%;
  transform: translateX(-50%);
  background: color-mix(in srgb, #22c55e 90%, transparent);
  color: white;
  padding: 8px 20px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  z-index: 1000;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* 滚动条 */
.output-content::-webkit-scrollbar,
.text-area::-webkit-scrollbar {
  width: 5px;
}

.output-content::-webkit-scrollbar-track,
.text-area::-webkit-scrollbar-track {
  background: transparent;
}

.output-content::-webkit-scrollbar-thumb,
.text-area::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 30%, transparent);
  border-radius: 3px;
}

.output-content::-webkit-scrollbar-thumb:hover,
.text-area::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--accent-primary, #6366f1) 50%, transparent);
}
</style>
