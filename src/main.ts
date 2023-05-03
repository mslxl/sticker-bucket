import 'normalize.css'
import 'animate.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import './scss/styles.scss'

import { createApp } from "vue"
import { createPinia } from 'pinia'
import { ElInfiniteScroll } from 'element-plus'

import App from "./App.vue"
import router from './router'

import { useSettings } from './store/settings'
import { isDebug } from './scripts/rs/debug'

const pinia = createPinia()

createApp(App)
  .use(router)
  .use(pinia)
  .use(ElInfiniteScroll)
  .mount("#app")


const settings = useSettings()
isDebug().then(debug => {
  settings.debug = debug
  // if (!debug) {
  //   window.oncontextmenu = () => false
  // }
})