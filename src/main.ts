import { createApp } from 'vue'
import App from './App.vue'
import { pinia } from './store'
import { router } from './router'
import { listen } from '@tauri-apps/api/event'

// 样式导入
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

// 监听全局快捷键导航事件
listen<string>('navigate-to', (event) => {
  const route = event.payload
  console.log('收到导航请求:', route)
  router.push(route)
})

// 开发环境日志
if (import.meta.env.DEV) {
  console.log('DevAssistant 应用已启动')
  console.log('环境:', import.meta.env.MODE)
}
