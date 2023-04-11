import 'normalize.css'
import 'animate.css'
import './scss/styles.scss'
import { createApp } from "vue"
import { createPinia } from 'pinia'

import App from "./App.vue"
import router from './router'

const pinia = createPinia()
createApp(App)
  .use(router)
  .use(pinia)
  .mount("#app")
