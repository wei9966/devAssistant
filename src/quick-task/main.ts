import { createApp } from 'vue'
import { createPinia } from 'pinia'
import QuickTaskApp from './QuickTaskApp.vue'
import '../styles/index.css'

const app = createApp(QuickTaskApp)
const pinia = createPinia()

app.use(pinia)
app.mount('#quick-task')
