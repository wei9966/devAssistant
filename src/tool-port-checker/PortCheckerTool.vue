<template>
  <div class="port-checker-tool">
    <!-- 输入区域 -->
    <div class="input-section">
      <div class="input-group">
        <label class="input-label">端口号</label>
        <div class="input-wrapper">
          <input
            ref="portInputRef"
            type="number"
            v-model.number="portNumber"
            placeholder="输入端口号（如: 3000）"
            class="port-input"
            @keydown.enter="checkPort"
            min="1"
            max="65535"
          />
          <button class="check-btn" @click="checkPort" :disabled="loading || !portNumber">
            <span v-if="loading" class="loading-spinner">⏳</span>
            <span v-else>查询</span>
          </button>
        </div>
      </div>
    </div>

    <!-- 结果表格 -->
    <div class="result-section" v-if="portResults.length > 0">
      <div class="section-title">
        <span class="section-icon">📊</span>
        <span>占用信息</span>
      </div>
      <div class="result-table-wrapper">
        <table class="result-table">
          <thead>
            <tr>
              <th>进程ID</th>
              <th>进程名</th>
              <th>协议</th>
              <th>本地地址</th>
              <th>状态</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(result, index) in portResults" :key="index">
              <td>{{ result.pid }}</td>
              <td class="process-name">{{ result.processName }}</td>
              <td>{{ result.protocol }}</td>
              <td class="address">{{ result.localAddress }}</td>
              <td>
                <span :class="['status-badge', getStatusClass(result.state)]">
                  {{ result.state }}
                </span>
              </td>
              <td>
                <button
                  class="kill-btn"
                  @click="confirmKillProcess(result.pid, result.processName)"
                  title="结束进程"
                >
                  🗑️ 结束
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="!loading && hasSearched" class="empty-state">
      <span class="empty-icon">✅</span>
      <span>端口 {{ searchedPort }} 未被占用</span>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <span class="loading-spinner">⏳</span>
      <span>正在检查端口 {{ portNumber }}...</span>
    </div>

    <!-- 确认对话框 -->
    <div v-if="showKillConfirm" class="confirm-overlay" @click="showKillConfirm = false">
      <div class="confirm-dialog" @click.stop>
        <div class="confirm-title">确认结束进程</div>
        <div class="confirm-message">
          确定要结束进程吗？
          <div class="confirm-details">
            <div>进程ID: <strong>{{ killTarget.pid }}</strong></div>
            <div>进程名: <strong>{{ killTarget.processName }}</strong></div>
          </div>
          <div class="confirm-warning">⚠️ 此操作不可撤销，请谨慎操作</div>
        </div>
        <div class="confirm-actions">
          <button class="confirm-btn cancel" @click="showKillConfirm = false">取消</button>
          <button class="confirm-btn danger" @click="killProcess">结束进程</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { message } from '@tauri-apps/plugin-dialog'

// Types
interface PortUsageInfo {
  pid: number
  processName: string
  protocol: string
  localAddress: string
  state: string
}

// Refs
const portInputRef = ref<HTMLInputElement | null>(null)

// State
const portNumber = ref<number | null>(null)
const loading = ref(false)
const portResults = ref<PortUsageInfo[]>([])
const hasSearched = ref(false)
const searchedPort = ref<number | null>(null)
const showKillConfirm = ref(false)
const killTarget = ref({ pid: 0, processName: '' })

// Methods
async function checkPort() {
  if (!portNumber.value || portNumber.value < 1 || portNumber.value > 65535) {
    await message('请输入有效的端口号（1-65535）', { title: '提示', kind: 'warning' })
    return
  }

  loading.value = true
  hasSearched.value = false
  portResults.value = []
  searchedPort.value = portNumber.value

  try {
    // 调用 Tauri 命令检查端口
    const results = await invoke<PortUsageInfo[]>('check_port_usage', { port: portNumber.value })
    portResults.value = results
    hasSearched.value = true
  } catch (error: any) {
    console.error('检查端口失败:', error)

    // 如果后端命令不存在，使用模拟数据（仅用于开发测试）
    if (error.toString().includes('command check_port_usage not found')) {
      console.warn('端口检查命令未实现，使用模拟数据')
      await message('端口检查功能需要后端支持\n\n请联系开发者添加 check_port_usage 和 kill_process_by_pid 命令',
        { title: '功能未实现', kind: 'warning' })
      // 模拟数据示例
      if (portNumber.value === 3000) {
        portResults.value = [
          {
            pid: 12345,
            processName: 'node.exe',
            protocol: 'TCP',
            localAddress: '0.0.0.0:3000',
            state: 'LISTENING'
          }
        ]
      }
      hasSearched.value = true
    } else {
      await message(`检查端口失败: ${error}`, { title: '错误', kind: 'error' })
    }
  } finally {
    loading.value = false
  }
}

function confirmKillProcess(pid: number, processName: string) {
  killTarget.value = { pid, processName }
  showKillConfirm.value = true
}

async function killProcess() {
  showKillConfirm.value = false
  const pid = killTarget.value.pid

  try {
    await invoke('kill_process_by_pid', { pid })
    await message(`已成功结束进程 ${pid}`, { title: '成功', kind: 'info' })

    // 重新检查端口
    await checkPort()
  } catch (error: any) {
    console.error('结束进程失败:', error)

    // 如果后端命令不存在
    if (error.toString().includes('command kill_process_by_pid not found')) {
      await message('结束进程功能需要后端支持\n\n请联系开发者添加 kill_process_by_pid 命令',
        { title: '功能未实现', kind: 'warning' })
    } else {
      await message(`结束进程失败: ${error}`, { title: '错误', kind: 'error' })
    }
  }
}

function getStatusClass(state: string): string {
  const stateUpper = state.toUpperCase()
  if (stateUpper === 'LISTENING') return 'status-listening'
  if (stateUpper === 'ESTABLISHED') return 'status-established'
  if (stateUpper === 'TIME_WAIT') return 'status-time-wait'
  return 'status-other'
}

// Lifecycle
onMounted(() => {
  nextTick(() => {
    portInputRef.value?.focus()
  })
})
</script>

<style scoped>
.port-checker-tool {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

/* Input Section */
.input-section {
  flex-shrink: 0;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.input-wrapper {
  display: flex;
  gap: 8px;
}

.port-input {
  flex: 1;
  background: var(--input-bg);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  padding: 10px 14px;
  color: var(--text-primary);
  font-size: 14px;
  transition: all 0.2s;
}

.port-input:focus {
  outline: none;
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px var(--accent-glow);
}

.port-input::placeholder {
  color: var(--text-dim);
}

.check-btn {
  background: var(--accent-glow);
  border: 1px solid color-mix(in srgb, var(--accent-primary) 30%, transparent);
  border-radius: 8px;
  padding: 10px 24px;
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  display: flex;
  align-items: center;
  gap: 6px;
}

.check-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent-primary) 30%, transparent);
  border-color: color-mix(in srgb, var(--accent-primary) 50%, transparent);
}

.check-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.loading-spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Result Section */
.result-section {
  margin-top: 16px;
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--accent-primary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--border-default);
  margin-bottom: 12px;
}

.section-icon {
  font-size: 14px;
}

.result-table-wrapper {
  flex: 1;
  overflow: auto;
  border-radius: 8px;
  border: 1px solid var(--border-default);
  min-height: 0;
}

.result-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.result-table thead {
  background: var(--bg-surface);
}

.result-table th {
  padding: 10px 12px;
  text-align: left;
  font-weight: 600;
  color: var(--text-muted);
  border-bottom: 1px solid var(--border-default);
}

.result-table tbody tr {
  border-bottom: 1px solid var(--border-default);
  transition: background 0.15s;
}

.result-table tbody tr:hover {
  background: var(--bg-hover);
}

.result-table td {
  padding: 10px 12px;
  color: var(--text-primary);
}

.process-name {
  font-weight: 500;
}

.address {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: var(--text-secondary);
}

.status-badge {
  display: inline-block;
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.status-listening {
  background: color-mix(in srgb, var(--success) 20%, transparent);
  color: var(--success);
  border: 1px solid color-mix(in srgb, var(--success) 30%, transparent);
}

.status-established {
  background: color-mix(in srgb, var(--info) 20%, transparent);
  color: var(--info);
  border: 1px solid color-mix(in srgb, var(--info) 30%, transparent);
}

.status-time-wait {
  background: color-mix(in srgb, var(--warning) 20%, transparent);
  color: var(--warning);
  border: 1px solid color-mix(in srgb, var(--warning) 30%, transparent);
}

.status-other {
  background: var(--bg-surface);
  color: var(--text-muted);
  border: 1px solid var(--border-default);
}

.kill-btn {
  background: color-mix(in srgb, var(--error) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--error) 20%, transparent);
  border-radius: 6px;
  padding: 5px 10px;
  color: var(--error);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 4px;
}

.kill-btn:hover {
  background: color-mix(in srgb, var(--error) 20%, transparent);
  border-color: color-mix(in srgb, var(--error) 40%, transparent);
}

/* Empty & Loading */
.empty-state,
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-muted);
  font-size: 14px;
  gap: 12px;
}

.empty-icon {
  font-size: 48px;
}

/* Confirm Dialog */
.confirm-overlay {
  position: fixed;
  inset: 0;
  background: var(--modal-backdrop, rgba(0, 0, 0, 0.5));
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.confirm-dialog {
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  padding: 24px;
  min-width: 360px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
}

.confirm-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
}

.confirm-message {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 20px;
  line-height: 1.6;
}

.confirm-details {
  margin: 12px 0;
  padding: 12px;
  background: var(--bg-surface);
  border-radius: 6px;
  font-size: 13px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.confirm-details strong {
  color: var(--text-primary);
}

.confirm-warning {
  margin-top: 12px;
  color: var(--warning);
  font-size: 12px;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.confirm-btn {
  padding: 8px 20px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.confirm-btn.cancel {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  color: var(--text-primary);
}

.confirm-btn.cancel:hover {
  background: var(--bg-hover);
}

.confirm-btn.danger {
  background: color-mix(in srgb, var(--error) 20%, transparent);
  border: 1px solid color-mix(in srgb, var(--error) 30%, transparent);
  color: var(--error);
}

.confirm-btn.danger:hover {
  background: color-mix(in srgb, var(--error) 30%, transparent);
}
</style>
