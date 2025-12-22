import { createApp } from 'vue'
import { createPinia } from 'pinia'
import TaskCalendarApp from './TaskCalendarApp.vue'
import '../styles/index.css'
// 主题系统
import { initThemeSync } from '../themes'

// 初始化主题
initThemeSync()

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
  const app = createApp(TaskCalendarApp)
  const pinia = createPinia()

  app.use(pinia)
  app.mount('#task-calendar')
} catch (e: any) {
  document.body.innerHTML = `<div style="color: red; padding: 20px; background: #1a1a2e;">
    <h2>Mount Error:</h2>
    <pre>${e.message}\n${e.stack}</pre>
  </div>`
}
