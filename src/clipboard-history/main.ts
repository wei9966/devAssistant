import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ClipboardHistoryApp from './ClipboardHistoryApp.vue'

// 样式导入
import '../styles/index.css'

// 全局错误捕获 - 调试用
window.onerror = (msg, url, line, col, error) => {
  document.body.innerHTML = `<div style="color: red; padding: 20px; background: #1a1a2e;">
    <h2>Error:</h2>
    <pre>${msg}\n${url}:${line}:${col}\n${error?.stack || ''}</pre>
  </div>`
  return false
}

window.addEventListener('unhandledrejection', (event) => {
  document.body.innerHTML = `<div style="color: red; padding: 20px; background: #1a1a2e;">
    <h2>Unhandled Promise Rejection:</h2>
    <pre>${event.reason}</pre>
  </div>`
})

try {
  const app = createApp(ClipboardHistoryApp)
  const pinia = createPinia()

  app.use(pinia)
  app.mount('#clipboard-history')
} catch (e: any) {
  document.body.innerHTML = `<div style="color: red; padding: 20px; background: #1a1a2e;">
    <h2>Mount Error:</h2>
    <pre>${e.message}\n${e.stack}</pre>
  </div>`
}
