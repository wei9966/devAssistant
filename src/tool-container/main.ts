import { createApp } from 'vue'
import ToolContainerApp from './ToolContainerApp.vue'
import '../styles/index.css'
// 主题系统
import { initThemeSync } from '../themes'

// 初始化主题
initThemeSync()

const app = createApp(ToolContainerApp)
app.mount('#tool-container')
