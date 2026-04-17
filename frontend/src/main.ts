import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import { VueQueryPlugin } from '@tanstack/vue-query'
import router from './router'
import App from './App.vue'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(ElementPlus)
app.use(VueQueryPlugin)
app.mount('#app')
