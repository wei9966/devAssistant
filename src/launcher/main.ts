import { createApp } from 'vue'
import { createPinia } from 'pinia'
import LauncherApp from './LauncherApp.vue'

// 样式导入
import '../styles/index.css'

const app = createApp(LauncherApp)
const pinia = createPinia()

app.use(pinia)
app.mount('#launcher')
