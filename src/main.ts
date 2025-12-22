import { createApp } from 'vue'
import App from './App.vue'
import { pinia } from './store'
import { router } from './router'
import { listen } from '@tauri-apps/api/event'

// 样式导入 - 主题变量必须先加载
import './themes/variables.css'
import './styles/index.css'
import './styles/global.css'
import './assets/styles/main.css'

// 创建Vue应用实例
const app = createApp(App)

// 安装Pinia状态管理
app.use(pinia)

// 安装Vue Router
app.use(router)

// 挂载应用
app.mount('#app')

// 禁用浏览器默认右键菜单（生产环境）
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => {
    e.preventDefault()
  })
}

// 监听全局快捷键导航事件
listen<string>('navigate-to', (event) => {
  const route = event.payload
  console.log('收到导航请求:', route)

  // 如果是应用启动器，触发全屏模态框而不是路由跳转
  if (route === '/app-launcher') {
    // 通过自定义事件通知 App.vue 打开启动器模态框
    window.dispatchEvent(new CustomEvent('open-cyberpunk-launcher'))
  } else if (route === '/quick-task') {
    // 触发快速任务创建模态框
    window.dispatchEvent(new CustomEvent('open-quick-task-modal'))
  } else if (route === '/sql-history') {
    // 触发SQL历史模态框
    window.dispatchEvent(new CustomEvent('open-cyberpunk-sql'))
  } else {
    router.push(route)
  }
})

// 开发环境日志
if (import.meta.env.DEV) {
  console.log('DevAssistant 应用已启动')
  console.log('环境:', import.meta.env.MODE)
}
