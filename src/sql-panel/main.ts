import { createApp } from 'vue'
import { createPinia } from 'pinia'
import SqlPanelApp from './SqlPanelApp.vue'
import '../styles/index.css'

const app = createApp(SqlPanelApp)
const pinia = createPinia()

app.use(pinia)
app.mount('#sql-panel')
